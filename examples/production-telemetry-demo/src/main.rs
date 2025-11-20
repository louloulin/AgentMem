//! Production-grade telemetry system demonstration
//!
//! This example demonstrates the comprehensive telemetry and monitoring
//! capabilities of AgentMem, including:
//! - Prometheus metrics collection
//! - Jaeger distributed tracing
//! - Structured logging
//! - Health monitoring
//! - Performance analytics

use agent_mem_core::{
    engine::{MemoryEngine, MemoryEngineConfig},
    hierarchy::MemoryScope,
};
use agent_mem_performance::telemetry::{ProductionTelemetryConfig, ProductionTelemetrySystem};
use agent_mem_traits::{Content, MemoryItem, MemoryType, MemoryV4, Session};

use anyhow::Result;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{error, info, warn};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize structured logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .json()
        .try_init()
        .ok(); // Ignore error if already initialized

    info!("🚀 Starting Production Telemetry Demo");

    // Create production telemetry configuration
    let telemetry_config = ProductionTelemetryConfig {
        enabled: true,
        prometheus_enabled: true,
        jaeger_enabled: true,
        jaeger_endpoint: Some("http://localhost:14268/api/traces".to_string()),
        prometheus_port: Some(9090),
        service_name: "agentmem-demo".to_string(),
    };

    // Initialize production telemetry system
    let telemetry = ProductionTelemetrySystem::new(telemetry_config).await?;
    info!("✅ Production telemetry system initialized");

    // Create memory engine for testing
    let engine_config = MemoryEngineConfig::default();

    let engine = MemoryEngine::new(engine_config);
    info!("✅ Memory engine initialized");

    // Demonstrate telemetry capabilities
    demonstrate_memory_operations(&telemetry, &engine).await?;
    demonstrate_performance_monitoring(&telemetry).await?;
    demonstrate_health_monitoring(&telemetry).await?;
    demonstrate_error_tracking(&telemetry).await?;

    info!("🎯 Production telemetry demo completed successfully");
    Ok(())
}

/// Demonstrate memory operations with telemetry tracking
async fn demonstrate_memory_operations(
    telemetry: &ProductionTelemetrySystem,
    engine: &MemoryEngine,
) -> Result<()> {
    info!("📊 Demonstrating memory operations with telemetry");

    for i in 0..10 {
        let start = Instant::now();
        let user_id = format!("user_{}", i % 3); // Simulate 3 different users

        // Create a test memory
        let session = Session {
            id: format!("session_{i}"),
            user_id: Some(user_id.clone()),
            agent_id: Some("demo_agent".to_string()),
            run_id: None,
            actor_id: None,
            created_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        let memory_item = MemoryItem {
            id: Uuid::new_v4().to_string(),
            content: format!("This is test message {i} for telemetry demonstration"),
            hash: None,
            metadata: HashMap::new(),
            score: None,
            created_at: chrono::Utc::now(),
            updated_at: None,
            session,
            memory_type: MemoryType::Episodic,
            entities: Vec::new(),
            relations: Vec::new(),
            agent_id: "demo_agent".to_string(),
            user_id: Some(user_id.clone()),
            importance: 0.5,
            embedding: None,
            last_accessed_at: chrono::Utc::now(),
            access_count: 0,
            expires_at: None,
            version: 1,
        };

        // Convert to MemoryV4 and add memory with telemetry tracking
        let memory = MemoryV4::from_legacy_item(&memory_item);
        let memory_result = engine.add_memory(memory).await;

        let duration = start.elapsed();
        let success = memory_result.is_ok();

        // Track the operation
        telemetry
            .track_memory_operation("add_memory", Some(&user_id), duration, success)
            .await;

        if let Err(e) = memory_result {
            error!("Failed to add memory: {}", e);
        } else {
            info!("✅ Memory {} added successfully in {:?}", i, duration);
        }

        // Small delay to simulate realistic usage
        sleep(Duration::from_millis(100)).await;
    }

    // Demonstrate search operations
    for i in 0..5 {
        let start = Instant::now();
        let user_id = format!("user_{}", i % 3);

        let search_result = engine
            .search_memories(
                &format!("test message {i}"),
                Some(MemoryScope::Agent(user_id.clone())),
                Some(10),
            )
            .await;

        let duration = start.elapsed();
        let success = search_result.is_ok();

        telemetry
            .track_memory_operation("search_memories", Some(&user_id), duration, success)
            .await;

        if let Ok(results) = search_result {
            info!(
                "🔍 Search {} found {} results in {:?}",
                i,
                results.len(),
                duration
            );
        }

        sleep(Duration::from_millis(50)).await;
    }

    Ok(())
}

/// Demonstrate performance monitoring
async fn demonstrate_performance_monitoring(telemetry: &ProductionTelemetrySystem) -> Result<()> {
    info!("📈 Demonstrating performance monitoring");

    // Collect system metrics
    let metrics = telemetry.collect_system_metrics().await;

    info!("📊 System Metrics:");
    info!("  Total Requests: {}", metrics.total_requests);
    info!("  Error Rate: {:.2}%", metrics.error_rate * 100.0);
    info!(
        "  Avg Response Time: {:.2}ms",
        metrics.average_response_time_ms
    );
    info!("  Throughput: {:.2} RPS", metrics.throughput_rps);
    info!("  Memory Usage: {} bytes", metrics.memory_usage_bytes);
    info!("  Cache Hit Rate: {:.2}%", metrics.cache_hit_rate * 100.0);
    info!("  Uptime: {:.2}s", metrics.uptime_seconds);

    Ok(())
}

/// Demonstrate health monitoring
async fn demonstrate_health_monitoring(telemetry: &ProductionTelemetrySystem) -> Result<()> {
    info!("🏥 Demonstrating health monitoring");

    let health_status = telemetry.get_health_status().await;

    info!("🩺 Health Status:");
    info!("  Status: {}", health_status.status);
    info!("  Timestamp: {}", health_status.timestamp);

    if let Some(metrics) = health_status.metrics {
        info!("  Health Metrics:");
        info!("    Error Rate: {:.2}%", metrics.error_rate * 100.0);
        info!(
            "    Response Time: {:.2}ms",
            metrics.average_response_time_ms
        );
        info!("    Throughput: {:.2} RPS", metrics.throughput_rps);
    }

    Ok(())
}

/// Demonstrate error tracking
async fn demonstrate_error_tracking(telemetry: &ProductionTelemetrySystem) -> Result<()> {
    info!("🚨 Demonstrating error tracking");

    // Simulate some errors
    for i in 0..3 {
        let start = Instant::now();
        let user_id = format!("error_user_{i}");

        // Simulate a failed operation
        let duration = Duration::from_millis(50 + i * 10);
        sleep(duration).await;

        telemetry
            .track_memory_operation(
                "simulated_error",
                Some(&user_id),
                start.elapsed(),
                false, // Mark as failed
            )
            .await;

        warn!("⚠️ Simulated error {} tracked", i);
    }

    // Check health status after errors
    let health_status = telemetry.get_health_status().await;
    info!("🩺 Health status after errors: {}", health_status.status);

    Ok(())
}
