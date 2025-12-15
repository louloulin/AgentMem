//! Circuit Breaker Middleware
//!
//! Implements circuit breaker pattern for fault tolerance and graceful degradation.
//! This middleware protects the server from cascading failures by temporarily
//! blocking requests when a service is experiencing high error rates.

use crate::error::ServerError;
use agent_mem_performance::error_recovery::{
    CircuitBreaker, CircuitBreakerConfig, CircuitBreakerState,
};
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, warn};

/// Circuit breaker manager for different service endpoints
pub struct CircuitBreakerManager {
    /// Default circuit breaker for general operations
    default_breaker: Arc<CircuitBreaker>,
    /// Circuit breakers for specific endpoints
    endpoint_breakers: Arc<tokio::sync::RwLock<std::collections::HashMap<String, Arc<CircuitBreaker>>>>,
}

impl CircuitBreakerManager {
    /// Create a new circuit breaker manager with default configuration
    pub fn new() -> Self {
        let default_config = CircuitBreakerConfig {
            failure_threshold: 5,
            success_threshold: 2,
            reset_timeout: Duration::from_secs(60),
            timeout: Duration::from_secs(30),
        };

        Self {
            default_breaker: Arc::new(CircuitBreaker::new(default_config)),
            endpoint_breakers: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Create with custom default configuration
    pub fn with_config(config: CircuitBreakerConfig) -> Self {
        Self {
            default_breaker: Arc::new(CircuitBreaker::new(config)),
            endpoint_breakers: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Get or create circuit breaker for a specific endpoint
    async fn get_breaker(&self, endpoint: &str) -> Arc<CircuitBreaker> {
        // Check if endpoint-specific breaker exists
        {
            let breakers = self.endpoint_breakers.read().await;
            if let Some(breaker) = breakers.get(endpoint) {
                return breaker.clone();
            }
        }

        // Create new breaker for this endpoint
        let config = CircuitBreakerConfig {
            failure_threshold: 5,
            success_threshold: 2,
            reset_timeout: Duration::from_secs(60),
            timeout: Duration::from_secs(30),
        };
        let new_breaker = Arc::new(CircuitBreaker::new(config));

        {
            let mut breakers = self.endpoint_breakers.write().await;
            breakers.insert(endpoint.to_string(), new_breaker.clone());
        }

        new_breaker
    }

    /// Check if request is allowed
    async fn allow_request(&self, endpoint: &str) -> bool {
        let breaker = self.get_breaker(endpoint).await;
        breaker.allow_request().await
    }

    /// Record success for an endpoint
    async fn record_success(&self, endpoint: &str) {
        let breaker = self.get_breaker(endpoint).await;
        breaker.record_success().await;
    }

    /// Record failure for an endpoint
    async fn record_failure(&self, endpoint: &str) {
        let breaker = self.get_breaker(endpoint).await;
        breaker.record_failure().await;
    }

    /// Get circuit breaker state for an endpoint
    pub async fn get_state(&self, endpoint: &str) -> CircuitBreakerState {
        let breaker = self.get_breaker(endpoint).await;
        breaker.get_state().await
    }
}

impl Default for CircuitBreakerManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Circuit breaker middleware
///
/// This middleware wraps requests with circuit breaker protection.
/// When a service endpoint experiences too many failures, the circuit
/// breaker opens and blocks requests until the service recovers.
pub async fn circuit_breaker_middleware(
    request: Request,
    next: Next,
) -> Response {
    // Extract endpoint path for circuit breaker identification
    let path = request.uri().path().to_string();
    let endpoint = normalize_endpoint(&path);

    // Get circuit breaker manager from extensions
    let manager = request
        .extensions()
        .get::<Arc<CircuitBreakerManager>>()
        .cloned();

    if let Some(manager) = manager {
        // Check if request is allowed
        if !manager.allow_request(&endpoint).await {
            warn!(
                "Circuit breaker is OPEN for endpoint: {}. Request blocked.",
                endpoint
            );
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "error": "Service temporarily unavailable",
                    "message": "Circuit breaker is open. Please try again later.",
                    "endpoint": endpoint,
                })),
            )
                .into_response();
        }

        // Execute the request
        let response = next.run(request).await;

        // Record result based on status code
        let status = response.status();
        if status.is_success() {
            manager.record_success(&endpoint).await;
        } else if status.is_server_error() {
            manager.record_failure(&endpoint).await;
            error!(
                "Request to {} failed with status {}. Circuit breaker recorded failure.",
                endpoint, status
            );
        }

        response
    } else {
        // No circuit breaker manager configured, proceed normally
        next.run(request).await
    }
}

/// Normalize endpoint path for circuit breaker grouping
///
/// Groups similar endpoints together (e.g., /api/v1/memories/:id -> /api/v1/memories/*)
fn normalize_endpoint(path: &str) -> String {
    use regex::Regex;
    
    // Remove UUIDs and IDs from path
    let uuid_pattern = Regex::new(r"/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}")
        .unwrap_or_else(|_| {
            // Fallback if regex compilation fails - use a pattern that matches nothing
            Regex::new("(?!)").unwrap()
        });
    let normalized = uuid_pattern.replace_all(path, "/*");

    // Replace numeric IDs with wildcard
    let numeric_pattern = Regex::new(r"/\d+")
        .unwrap_or_else(|_| Regex::new("(?!)").unwrap());
    let normalized = numeric_pattern.replace_all(&normalized, "/*");

    // Replace path parameters with wildcard
    let param_pattern = Regex::new(r"/:[^/]+")
        .unwrap_or_else(|_| Regex::new("(?!)").unwrap());
    let normalized = param_pattern.replace_all(&normalized, "/*");

    normalized.to_string()
}

// Tests are in separate file: circuit_breaker_tests.rs
