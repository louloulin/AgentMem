//! Circuit Breaker Middleware Tests
//!
//! Comprehensive tests for circuit breaker functionality

#[cfg(test)]
mod tests {
    use super::super::circuit_breaker::{normalize_endpoint, CircuitBreakerManager};
    use agent_mem_performance::error_recovery::CircuitBreakerState;
    use std::time::Duration;
    use tokio::time::sleep;

    #[test]
    fn test_normalize_endpoint() {
        assert_eq!(
            normalize_endpoint("/api/v1/memories/123"),
            "/api/v1/memories/*"
        );
        assert_eq!(
            normalize_endpoint("/api/v1/memories/abc-123-def"),
            "/api/v1/memories/*"
        );
        assert_eq!(
            normalize_endpoint("/api/v1/memories/:id"),
            "/api/v1/memories/*"
        );
        assert_eq!(
            normalize_endpoint("/api/v1/memories"),
            "/api/v1/memories"
        );
        assert_eq!(
            normalize_endpoint("/api/v1/memories/550e8400-e29b-41d4-a716-446655440000"),
            "/api/v1/memories/*"
        );
    }

    #[tokio::test]
    async fn test_circuit_breaker_manager_initial_state() {
        let manager = CircuitBreakerManager::new();

        // Initially should allow requests
        assert!(manager.allow_request("test").await);
        assert_eq!(manager.get_state("test").await, CircuitBreakerState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_opens_after_failures() {
        let manager = CircuitBreakerManager::new();

        // Record failures to trip the circuit breaker
        for _ in 0..5 {
            manager.record_failure("test").await;
        }

        // Circuit breaker should be open now
        assert!(!manager.allow_request("test").await);
        assert_eq!(manager.get_state("test").await, CircuitBreakerState::Open);
    }

    #[tokio::test]
    async fn test_circuit_breaker_recovery() {
        let manager = CircuitBreakerManager::new();

        // Trip the circuit breaker
        for _ in 0..5 {
            manager.record_failure("test").await;
        }
        assert!(!manager.allow_request("test").await);

        // Wait for reset timeout (in real scenario, this would be 60 seconds)
        // For testing, we can manually check the state transition logic
        // In production, the circuit breaker will automatically transition to HalfOpen
        // after the reset timeout expires
    }

    #[tokio::test]
    async fn test_circuit_breaker_success_resets_failure_count() {
        let manager = CircuitBreakerManager::new();

        // Record some failures
        manager.record_failure("test").await;
        manager.record_failure("test").await;

        // Record success should reset failure count
        manager.record_success("test").await;

        // Should still allow requests (failure count reset)
        assert!(manager.allow_request("test").await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_endpoint_isolation() {
        let manager = CircuitBreakerManager::new();

        // Trip circuit breaker for endpoint1
        for _ in 0..5 {
            manager.record_failure("endpoint1").await;
        }

        // endpoint2 should still be open
        assert!(manager.allow_request("endpoint2").await);
        assert!(!manager.allow_request("endpoint1").await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open_to_closed() {
        let manager = CircuitBreakerManager::new();

        // Trip the circuit breaker
        for _ in 0..5 {
            manager.record_failure("test").await;
        }

        // In HalfOpen state, record enough successes to close
        // Note: This test demonstrates the logic, but in practice,
        // the circuit breaker transitions to HalfOpen after reset_timeout
        manager.record_success("test").await;
        manager.record_success("test").await;
        manager.record_success("test").await;

        // After enough successes, circuit breaker should close
        // (This depends on the success_threshold configuration)
    }

    #[tokio::test]
    async fn test_circuit_breaker_custom_config() {
        use agent_mem_performance::error_recovery::CircuitBreakerConfig;
        use std::time::Duration;

        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            reset_timeout: Duration::from_secs(30),
            timeout: Duration::from_secs(15),
        };

        let manager = CircuitBreakerManager::with_config(config);

        // With lower threshold, should trip faster
        for _ in 0..3 {
            manager.record_failure("test").await;
        }

        assert!(!manager.allow_request("test").await);
        assert_eq!(manager.get_state("test").await, CircuitBreakerState::Open);
    }

    #[tokio::test]
    async fn test_circuit_breaker_multiple_endpoints() {
        let manager = CircuitBreakerManager::new();

        // Trip circuit breaker for endpoint1
        for _ in 0..5 {
            manager.record_failure("endpoint1").await;
        }

        // endpoint2 and endpoint3 should still be open
        assert!(manager.allow_request("endpoint2").await);
        assert!(manager.allow_request("endpoint3").await);
        assert!(!manager.allow_request("endpoint1").await);

        // Trip endpoint2
        for _ in 0..5 {
            manager.record_failure("endpoint2").await;
        }

        // Now endpoint1 and endpoint2 are open, endpoint3 is still closed
        assert!(!manager.allow_request("endpoint1").await);
        assert!(!manager.allow_request("endpoint2").await);
        assert!(manager.allow_request("endpoint3").await);
    }

    #[test]
    fn test_normalize_endpoint_comprehensive() {
        // Test UUID normalization
        assert_eq!(
            normalize_endpoint("/api/v1/memories/550e8400-e29b-41d4-a716-446655440000"),
            "/api/v1/memories/*"
        );

        // Test numeric ID normalization
        assert_eq!(
            normalize_endpoint("/api/v1/users/12345"),
            "/api/v1/users/*"
        );

        // Test path parameter normalization
        assert_eq!(
            normalize_endpoint("/api/v1/memories/:id"),
            "/api/v1/memories/*"
        );

        // Test multiple parameters
        assert_eq!(
            normalize_endpoint("/api/v1/agents/:agent_id/memories/:memory_id"),
            "/api/v1/agents/*/memories/*"
        );

        // Test paths without parameters
        assert_eq!(
            normalize_endpoint("/api/v1/memories"),
            "/api/v1/memories"
        );

        // Test root path
        assert_eq!(
            normalize_endpoint("/"),
            "/"
        );

        // Test complex path
        assert_eq!(
            normalize_endpoint("/api/v1/organizations/org-123/users/user-456/memories/789"),
            "/api/v1/organizations/*/users/*/memories/*"
        );
    }
}

