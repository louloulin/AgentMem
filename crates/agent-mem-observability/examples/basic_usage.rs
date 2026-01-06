//! Basic usage example for agent-mem-observability

use agent_mem_observability::{
    init_observability, HealthCheck, MetricsRegistry, ObservabilityConfig, PerformanceAnalyzer,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 AgentMem Observability Example\n");

    // 1. Initialize observability
    println!("1️⃣  Initializing observability...");
    let config = ObservabilityConfig {
        service_name: "agentmem-example".to_string(),
        otlp_endpoint: None, // Set to Some("http://localhost:4317") to enable
        enable_metrics: true,
        metrics_port: 9090,
        log_level: "info".to_string(),
        json_logging: false,
    };

    init_observability(config.clone()).await?;
    println!("   ✅ Observability initialized\n");

    // 2. Structured logging
    println!("2️⃣  Testing structured logging...");
    info!("Application started");
    info!(user_id = "user123", action = "login", "User logged in");
    warn!(retry_count = 3, "Retrying operation");
    error!(error_code = "E001", "An error occurred");
    println!("   ✅ Logs emitted\n");

    // 3. Metrics collection
    println!("3️⃣  Testing metrics collection...");
    let metrics_registry = MetricsRegistry::new();
    let collector = metrics_registry.collector();

    // Record some metrics
    collector.record_request("GET", "/api/users", 200).await;
    collector.record_request("POST", "/api/users", 201).await;
    collector.record_request("GET", "/api/users", 404).await;
    collector.record_error("validation_error").await;
    collector.set_active_connections(10).await;
    collector.set_memory_usage(1024 * 1024 * 100).await; // 100 MB
    collector
        .record_request_duration("GET", "/api/users", 0.05)
        .await;
    collector.record_tool_execution("calculator", 0.001).await;

    println!("   ✅ Metrics recorded");
    println!("\n   📊 Metrics (Prometheus format):");
    println!("   {}", "-".repeat(60));
    let metrics_text = metrics_registry.gather();
    for line in metrics_text.lines().take(20) {
        if !line.starts_with('#') && !line.is_empty() {
            println!("   {line}");
        }
    }
    println!("   {}", "-".repeat(60));
    println!();

    // 4. Performance analysis
    println!("4️⃣  Testing performance analysis...");
    let perf_analyzer = PerformanceAnalyzer::new();

    // Simulate some operations
    for i in 0..5 {
        let _tracker = perf_analyzer.start_operation("database_query");
        sleep(Duration::from_millis(10 + i * 5)).await;
    }

    for i in 0..3 {
        let _tracker = perf_analyzer.start_operation("api_call");
        sleep(Duration::from_millis(50 + i * 10)).await;
    }

    // Wait for trackers to complete
    sleep(Duration::from_millis(100)).await;

    let report = perf_analyzer.get_report().await;
    println!("   ✅ Performance data collected");
    println!("\n   📈 Performance Report:");
    println!("   {}", "-".repeat(60));
    println!("   Total operations: {}", report.total_operations);
    println!("   Average duration: {:.2}ms", report.avg_duration_ms);
    if let Some(slowest) = report.slowest_operation {
        println!(
            "   Slowest operation: {} ({:.2}ms)",
            slowest.name, slowest.avg_duration_ms
        );
    }
    println!("\n   Operation Details:");
    for op in report.operations {
        println!("   - {}", op.name);
        println!("     Executions: {}", op.total_executions);
        println!(
            "     Avg: {:.2}ms, Min: {:.2}ms, Max: {:.2}ms",
            op.avg_duration_ms, op.min_duration_ms, op.max_duration_ms
        );
        println!(
            "     P50: {:.2}ms, P95: {:.2}ms, P99: {:.2}ms",
            op.p50_duration_ms, op.p95_duration_ms, op.p99_duration_ms
        );
    }
    println!("   {}", "-".repeat(60));
    println!();

    // 5. Health checks
    println!("5️⃣  Testing health checks...");
    let health_check = Arc::new(HealthCheck::new());

    // Register components
    health_check.register_component("database").await;
    health_check.register_component("cache").await;
    health_check.register_component("vector_store").await;

    // Check liveness
    let liveness = health_check.liveness().await;
    println!("   ✅ Liveness check: {}", liveness.status);
    println!("      Version: {}", liveness.version);
    println!("      Uptime: {}s", liveness.uptime_seconds);

    // Check readiness
    let readiness = health_check.readiness().await;
    println!("   ✅ Readiness check: {}", readiness.status);
    println!("      Components: {}", readiness.components.len());
    for (name, component) in readiness.components {
        println!("      - {}: {}", name, component.status);
    }
    println!();

    // 6. Tracing (if enabled)
    println!("6️⃣  Tracing:");
    if config.otlp_endpoint.is_some() {
        println!("   ✅ Tracing enabled");
        println!("      Traces will be exported to configured endpoint");
    } else {
        println!("   ℹ️  Tracing not configured");
        println!("      Set otlp_endpoint to enable");
    }
    println!();

    // Summary
    println!("✨ Summary:");
    println!("   - Structured logging: ✅");
    println!("   - Metrics collection: ✅");
    println!("   - Performance analysis: ✅");
    println!("   - Health checks: ✅");
    println!(
        "   - Distributed tracing: {}",
        if config.otlp_endpoint.is_some() {
            "✅"
        } else {
            "⚠️  (not configured)"
        }
    );
    println!();

    println!("🎉 All observability features working correctly!");
    println!();
    println!("💡 Next steps:");
    println!("   - Start metrics server: http://localhost:9090/metrics");
    println!("   - Start health server: http://localhost:8080/health/live");
    println!("   - Configure Jaeger/OTLP for distributed tracing");
    println!("   - Set up Grafana dashboards for visualization");

    Ok(())
}
