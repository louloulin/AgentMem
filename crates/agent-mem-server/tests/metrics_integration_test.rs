//! Integration test for Prometheus metrics

use agent_mem_observability::metrics::MetricsRegistry;
use std::sync::Arc;

#[tokio::test]
async fn test_metrics_registry_creation() {
    let metrics = Arc::new(MetricsRegistry::new());
    
    // Test incrementing requests
    metrics.increment_requests("GET", "/test", "200").await;
    metrics.increment_requests("POST", "/api/v1/memories", "201").await;
    
    // Test recording duration
    metrics.record_request_duration("GET", "/test", 0.123).await;
    
    // Test incrementing errors
    metrics.increment_errors("client_error").await;
    
    // Test setting memory usage
    metrics.set_memory_usage(1024 * 1024 * 100).await; // 100MB
    
    // Test recording tool execution
    metrics.record_tool_execution("search", 0.456).await;
    
    // Verify registry exists
    let registry = metrics.registry();
    assert!(!registry.gather().is_empty());
}

#[tokio::test]
async fn test_prometheus_text_format() {
    use prometheus::Encoder;
    
    let metrics = Arc::new(MetricsRegistry::new());
    
    // Add some metrics
    metrics.increment_requests("GET", "/health", "200").await;
    metrics.record_request_duration("GET", "/health", 0.001).await;
    
    // Encode to Prometheus text format
    let encoder = prometheus::TextEncoder::new();
    let metric_families = metrics.registry().gather();
    
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    
    let output = String::from_utf8(buffer).unwrap();
    
    // Verify output contains expected metrics
    assert!(output.contains("agentmem_requests_total"));
    assert!(output.contains("agentmem_request_duration_seconds"));
}

