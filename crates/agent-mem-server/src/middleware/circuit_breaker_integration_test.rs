//! Circuit Breaker Integration Tests
//!
//! Integration tests for circuit breaker middleware with actual HTTP requests

#[cfg(test)]
mod integration_tests {
    use crate::middleware::circuit_breaker::{circuit_breaker_middleware, CircuitBreakerManager};
    use axum::{
        body::Body,
        extract::Request,
        http::{Method, StatusCode, Uri},
        middleware::Next,
        response::Response,
    };
    use std::sync::Arc;
    
    // Note: Integration tests require actual HTTP server setup
    // These tests are disabled for now as they need more complex setup
    // Unit tests in circuit_breaker_tests.rs provide sufficient coverage

    /// Helper function to create a test request
    fn create_test_request(path: &str) -> Request {
        let uri = Uri::try_from(path).expect("valid URI");
        Request::builder()
            .method(Method::GET)
            .uri(uri)
            .body(Body::empty())
            .expect("valid request")
    }

    /// Helper function to create a test response
    fn create_test_response(status: StatusCode) -> Response {
        Response::builder()
            .status(status)
            .body(Body::empty())
            .expect("valid response")
    }

    #[tokio::test]
    async fn test_circuit_breaker_middleware_with_success() {
        let manager = Arc::new(CircuitBreakerManager::new());
        let mut request = create_test_request("/api/v1/memories/123");

        // Add manager to request extensions
        request.extensions_mut().insert(manager.clone());

        // Create a next handler that returns success
        let next = Next::new(|_req: Request| async move {
            create_test_response(StatusCode::OK)
        });

        // Execute middleware
        let response = circuit_breaker_middleware(request, next).await;

        // Should succeed
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_circuit_breaker_middleware_blocks_when_open() {
        let manager = Arc::new(CircuitBreakerManager::new());
        
        // Trip the circuit breaker
        for _ in 0..5 {
            manager.record_failure("/api/v1/memories/*").await;
        }

        let mut request = create_test_request("/api/v1/memories/123");
        request.extensions_mut().insert(manager.clone());

        let next = Next::new(|_req: Request| async move {
            create_test_response(StatusCode::OK)
        });

        // Execute middleware
        let response = circuit_breaker_middleware(request, next).await;

        // Should be blocked with 503
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_circuit_breaker_middleware_records_failure() {
        let manager = Arc::new(CircuitBreakerManager::new());
        let mut request = create_test_request("/api/v1/memories/123");
        request.extensions_mut().insert(manager.clone());

        // Create a next handler that returns server error
        let next = Next::new(|_req: Request| async move {
            create_test_response(StatusCode::INTERNAL_SERVER_ERROR)
        });

        // Execute middleware
        let response = circuit_breaker_middleware(request, next).await;

        // Should return the error
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        // Circuit breaker should have recorded the failure
        // After 5 failures, it should be open
        for _ in 0..4 {
            let mut request = create_test_request("/api/v1/memories/456");
            request.extensions_mut().insert(manager.clone());
            let next = Next::new(|_req: Request| async move {
                create_test_response(StatusCode::INTERNAL_SERVER_ERROR)
            });
            let _ = circuit_breaker_middleware(request, next).await;
        }

        // Now circuit breaker should be open
        let mut request = create_test_request("/api/v1/memories/789");
        request.extensions_mut().insert(manager.clone());
        let next = Next::new(|_req: Request| async move {
            create_test_response(StatusCode::OK)
        });
        let response = circuit_breaker_middleware(request, next).await;

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_circuit_breaker_middleware_without_manager() {
        // Test that middleware works even without circuit breaker manager
        let request = create_test_request("/api/v1/memories");

        let next = Next::new(|_req: Request| async move {
            create_test_response(StatusCode::OK)
        });

        // Execute middleware without manager in extensions
        let response = circuit_breaker_middleware(request, next).await;

        // Should proceed normally
        assert_eq!(response.status(), StatusCode::OK);
    }
}
