//! Network capability integration tests

use agent_mem_plugins::capabilities::network::{HttpMethod, HttpRequest, NetworkCapability};

#[test]
fn test_network_capability_basic() {
    let network = NetworkCapability::new(true, 100);

    let request = HttpRequest {
        url: "https://api.example.com/user/123".to_string(),
        method: HttpMethod::GET,
        headers: std::collections::HashMap::new(),
        body: None,
        timeout_ms: Some(5000),
    };

    let response = network.http_request(request).unwrap();
    assert_eq!(response.status, 200);
    assert!(response.body.contains("Test User"));
}

#[test]
fn test_network_multiple_requests() {
    let network = NetworkCapability::new(true, 100);

    // Make multiple requests
    for i in 0..10 {
        let request = HttpRequest {
            url: format!("https://api.example.com/user/{i}"),
            method: HttpMethod::GET,
            headers: std::collections::HashMap::new(),
            body: None,
            timeout_ms: Some(5000),
        };

        let response = network.http_request(request).unwrap();
        assert_eq!(response.status, 200);
    }

    assert_eq!(network.get_request_count(), 10);
}

#[test]
fn test_network_post_with_body() {
    let network = NetworkCapability::new(true, 100);

    let mut headers = std::collections::HashMap::new();
    headers.insert("content-type".to_string(), "application/json".to_string());

    let request = HttpRequest {
        url: "https://api.example.com/data".to_string(),
        method: HttpMethod::POST,
        headers,
        body: Some(r#"{"name": "test", "value": 123}"#.to_string()),
        timeout_ms: Some(5000),
    };

    let response = network.http_request(request).unwrap();
    assert_eq!(response.status, 200);
}

#[test]
fn test_network_error_handling() {
    let network = NetworkCapability::new(true, 100);

    // Test 404 error
    let request = HttpRequest {
        url: "https://api.example.com/error".to_string(),
        method: HttpMethod::GET,
        headers: std::collections::HashMap::new(),
        body: None,
        timeout_ms: Some(5000),
    };

    let response = network.http_request(request).unwrap();
    assert_eq!(response.status, 404);
    assert!(response.body.contains("Not found"));

    // Test timeout error
    let timeout_request = HttpRequest {
        url: "https://api.example.com/timeout".to_string(),
        method: HttpMethod::GET,
        headers: std::collections::HashMap::new(),
        body: None,
        timeout_ms: Some(1000),
    };

    let result = network.http_request(timeout_request);
    assert!(result.is_err());
}

#[test]
fn test_network_request_limit_enforcement() {
    let network = NetworkCapability::new(true, 5);

    // Make 5 requests (should succeed)
    for i in 0..5 {
        let request = HttpRequest {
            url: format!("https://api.example.com/test/{i}"),
            method: HttpMethod::GET,
            headers: std::collections::HashMap::new(),
            body: None,
            timeout_ms: Some(5000),
        };

        let result = network.http_request(request);
        assert!(result.is_ok(), "Request {i} should succeed");
    }

    // 6th request should fail
    let request = HttpRequest {
        url: "https://api.example.com/test/6".to_string(),
        method: HttpMethod::GET,
        headers: std::collections::HashMap::new(),
        body: None,
        timeout_ms: Some(5000),
    };

    let result = network.http_request(request);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("request limit exceeded"));
}

#[test]
fn test_network_reset_counter() {
    let network = NetworkCapability::new(true, 100);

    // Make some requests
    for i in 0..5 {
        let request = HttpRequest {
            url: format!("https://api.example.com/test/{i}"),
            method: HttpMethod::GET,
            headers: std::collections::HashMap::new(),
            body: None,
            timeout_ms: Some(5000),
        };

        network.http_request(request).unwrap();
    }

    assert_eq!(network.get_request_count(), 5);

    // Reset counter
    network.reset_request_count();
    assert_eq!(network.get_request_count(), 0);

    // Should be able to make requests again
    let request = HttpRequest {
        url: "https://api.example.com/test/new".to_string(),
        method: HttpMethod::GET,
        headers: std::collections::HashMap::new(),
        body: None,
        timeout_ms: Some(5000),
    };

    network.http_request(request).unwrap();
    assert_eq!(network.get_request_count(), 1);
}

#[test]
fn test_network_different_http_methods() {
    let network = NetworkCapability::new(true, 100);

    let methods = vec![
        HttpMethod::GET,
        HttpMethod::POST,
        HttpMethod::PUT,
        HttpMethod::DELETE,
        HttpMethod::PATCH,
    ];

    for method in methods {
        let request = HttpRequest {
            url: "https://api.example.com/test".to_string(),
            method: method.clone(),
            headers: std::collections::HashMap::new(),
            body: None,
            timeout_ms: Some(5000),
        };

        let result = network.http_request(request);
        assert!(result.is_ok(), "Method {method:?} should succeed");
    }
}
