//! Network capability for plugins
//!
//! Provides HTTP client functionality for plugins to make external API calls.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// HTTP method
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
}

/// HTTP request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    pub url: String,
    pub method: HttpMethod,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub timeout_ms: Option<u64>,
}

/// HTTP response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

/// Network capability
///
/// Provides HTTP client functionality for plugins.
/// In production, this would use a real HTTP client like reqwest.
/// For now, this is a mock implementation for testing.
pub struct NetworkCapability {
    /// Whether to use mock responses (for testing)
    mock_mode: bool,
    /// Maximum allowed requests
    max_requests: usize,
    /// Request counter
    request_count: std::sync::atomic::AtomicUsize,
}

impl NetworkCapability {
    /// Create a new network capability
    pub fn new(mock_mode: bool, max_requests: usize) -> Self {
        Self {
            mock_mode,
            max_requests,
            request_count: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Make an HTTP request
    pub fn http_request(&self, request: HttpRequest) -> Result<HttpResponse> {
        // Check request limit
        let count = self
            .request_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if count >= self.max_requests {
            return Err(anyhow!("Network request limit exceeded"));
        }

        // Validate URL
        if request.url.is_empty() {
            return Err(anyhow!("URL cannot be empty"));
        }

        // In mock mode, return mock responses
        if self.mock_mode {
            return self.mock_http_request(&request);
        }

        // TODO: In production, use reqwest or similar HTTP client
        // For now, return a placeholder response
        Ok(HttpResponse {
            status: 200,
            headers: HashMap::new(),
            body: "Mock response - real HTTP client not yet implemented".to_string(),
        })
    }

    /// Mock HTTP request for testing
    fn mock_http_request(&self, request: &HttpRequest) -> Result<HttpResponse> {
        // Simulate different responses based on URL
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        let (status, body) = if request.url.contains("api.example.com/user") {
            (
                200,
                r#"{"id": 1, "name": "Test User", "email": "test@example.com"}"#.to_string(),
            )
        } else if request.url.contains("api.example.com/error") {
            (404, r#"{"error": "Not found"}"#.to_string())
        } else if request.url.contains("api.example.com/timeout") {
            return Err(anyhow!("Request timeout"));
        } else {
            (200, r#"{"message": "Success"}"#.to_string())
        };

        Ok(HttpResponse {
            status,
            headers,
            body,
        })
    }

    /// Get request count
    pub fn get_request_count(&self) -> usize {
        self.request_count
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Reset request count
    pub fn reset_request_count(&self) {
        self.request_count
            .store(0, std::sync::atomic::Ordering::SeqCst);
    }
}

impl Default for NetworkCapability {
    fn default() -> Self {
        Self::new(true, 1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_get_request() {
        let network = NetworkCapability::new(true, 100);

        let request = HttpRequest {
            url: "https://api.example.com/user/123".to_string(),
            method: HttpMethod::GET,
            headers: HashMap::new(),
            body: None,
            timeout_ms: Some(5000),
        };

        let response = network.http_request(request).unwrap();
        assert_eq!(response.status, 200);
        assert!(response.body.contains("Test User"));
    }

    #[test]
    fn test_http_post_request() {
        let network = NetworkCapability::new(true, 100);

        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        let request = HttpRequest {
            url: "https://api.example.com/data".to_string(),
            method: HttpMethod::POST,
            headers,
            body: Some(r#"{"name": "test"}"#.to_string()),
            timeout_ms: Some(5000),
        };

        let response = network.http_request(request).unwrap();
        assert_eq!(response.status, 200);
    }

    #[test]
    fn test_http_error_response() {
        let network = NetworkCapability::new(true, 100);

        let request = HttpRequest {
            url: "https://api.example.com/error".to_string(),
            method: HttpMethod::GET,
            headers: HashMap::new(),
            body: None,
            timeout_ms: Some(5000),
        };

        let response = network.http_request(request).unwrap();
        assert_eq!(response.status, 404);
        assert!(response.body.contains("Not found"));
    }

    #[test]
    fn test_http_timeout() {
        let network = NetworkCapability::new(true, 100);

        let request = HttpRequest {
            url: "https://api.example.com/timeout".to_string(),
            method: HttpMethod::GET,
            headers: HashMap::new(),
            body: None,
            timeout_ms: Some(1000),
        };

        let result = network.http_request(request);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("timeout"));
    }

    #[test]
    fn test_request_limit() {
        let network = NetworkCapability::new(true, 3);

        let request = HttpRequest {
            url: "https://api.example.com/data".to_string(),
            method: HttpMethod::GET,
            headers: HashMap::new(),
            body: None,
            timeout_ms: Some(5000),
        };

        // First 3 requests should succeed
        for _ in 0..3 {
            network.http_request(request.clone()).unwrap();
        }

        // 4th request should fail
        let result = network.http_request(request);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("request limit exceeded"));
    }

    #[test]
    fn test_request_counter() {
        let network = NetworkCapability::new(true, 100);
        assert_eq!(network.get_request_count(), 0);

        let request = HttpRequest {
            url: "https://api.example.com/data".to_string(),
            method: HttpMethod::GET,
            headers: HashMap::new(),
            body: None,
            timeout_ms: Some(5000),
        };

        network.http_request(request.clone()).unwrap();
        assert_eq!(network.get_request_count(), 1);

        network.http_request(request).unwrap();
        assert_eq!(network.get_request_count(), 2);

        network.reset_request_count();
        assert_eq!(network.get_request_count(), 0);
    }

    #[test]
    fn test_empty_url() {
        let network = NetworkCapability::new(true, 100);

        let request = HttpRequest {
            url: "".to_string(),
            method: HttpMethod::GET,
            headers: HashMap::new(),
            body: None,
            timeout_ms: Some(5000),
        };

        let result = network.http_request(request);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("URL cannot be empty"));
    }
}

