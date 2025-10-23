//! Integration test for Prometheus metrics

use agent_mem_observability::metrics::MetricsRegistry;
use std::sync::Arc;

#[tokio::test]
async fn test_metrics_registry_creation() {
    let metrics = Arc::new(MetricsRegistry::new());
    let collector = metrics.collector();
    
    // Test recording requests
    collector.record_request("GET", "/test", 200).await;
    collector.record_request("POST", "/api/v1/memories", 201).await;
    
    // Test recording duration
    collector.record_request_duration("GET", "/test", 0.123).await;
    
    // Test recording errors
    collector.record_error("client_error").await;
    
    // Test setting memory usage
    collector.set_memory_usage(1024 * 1024 * 100).await; // 100MB
    
    // Test recording tool execution
    collector.record_tool_execution("search", 0.456).await;
    
    // Verify registry exists
    let registry = metrics.registry();
    assert!(!registry.gather().is_empty());
}

#[tokio::test]
async fn test_prometheus_text_format() {
    let metrics = Arc::new(MetricsRegistry::new());
    let collector = metrics.collector();
    
    // Add some metrics
    collector.record_request("GET", "/health", 200).await;
    collector.record_request_duration("GET", "/health", 0.001).await;
    
    // Use the gather method from MetricsRegistry
    let output = metrics.gather();
    
    // Verify output contains expected metrics
    assert!(output.contains("agentmem_requests_total"));
    assert!(output.contains("agentmem_request_duration_seconds"));
}

