//! AgentMem 可观测性完整演示
//!
//! 本示例演示如何使用 AgentMem 的监控和可观测性功能：
//! - Prometheus 指标收集
//! - 分布式追踪（OpenTelemetry）
//! - 结构化日志
//! - 健康检查
//! - 性能分析

use agent_mem_observability::{
    init_observability, HealthCheck, HealthStatus, MetricsRegistry, ObservabilityConfig,
    PerformanceAnalyzer,
};
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, instrument, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== AgentMem 可观测性完整演示 ===\n");

    // 1. 初始化可观测性
    println!("--- 1. 初始化可观测性 ---");
    let config = ObservabilityConfig {
        service_name: "agentmem-demo".to_string(),
        otlp_endpoint: Some("http://localhost:4317".to_string()),
        enable_metrics: true,
        metrics_port: 9091,
        log_level: "info".to_string(),
        json_logging: false, // 使用人类可读格式便于演示
    };

    init_observability(config).await?;
    info!("✓ 可观测性系统已初始化");
    println!("✓ 可观测性系统已初始化");
    println!("  - Prometheus 指标: http://localhost:9091/metrics");
    println!("  - OTLP 追踪: http://localhost:4317");
    println!();

    // 2. 指标收集演示
    println!("--- 2. Prometheus 指标收集演示 ---");
    let metrics = MetricsRegistry::new();
    let collector = metrics.collector();

    // 记录一些请求
    for i in 0..10 {
        let status = if i % 5 == 0 { 500 } else { 200 };
        collector
            .record_request("GET", "/api/users", status)
            .await;
        collector
            .record_request_duration("GET", "/api/users", 0.05 + (i as f64 * 0.01))
            .await;
    }
    info!("✓ 记录了 10 个请求指标");
    println!("✓ 记录了 10 个请求指标");

    // 记录工具执行
    collector
        .record_tool_execution("calculator", 0.001)
        .await;
    collector
        .record_tool_execution("web_search", 0.5)
        .await;
    info!("✓ 记录了 2 个工具执行指标");
    println!("✓ 记录了 2 个工具执行指标");

    // 设置资源指标
    collector.set_memory_usage(1024 * 1024 * 512).await; // 512MB
    collector.set_active_connections(42i64).await;
    info!("✓ 设置了资源使用指标");
    println!("✓ 设置了资源使用指标");

    // 导出指标
    let metrics_output = metrics.gather();
    println!("\n指标示例（前 500 字符）:");
    println!("{}", &metrics_output[..metrics_output.len().min(500)]);
    println!("...\n");

    // 3. 分布式追踪演示
    println!("--- 3. 分布式追踪演示 ---");
    info!("开始分布式追踪演示");

    // 模拟一个请求处理流程
    process_user_request("user123").await?;
    info!("✓ 完成分布式追踪演示");
    println!("✓ 完成分布式追踪演示");
    println!("  - 查看追踪: http://localhost:16686");
    println!("  - 服务名称: agentmem-demo");
    println!();

    // 4. 结构化日志演示
    println!("--- 4. 结构化日志演示 ---");

    // 不同级别的日志
    info!(user_id = "user123", action = "login", "用户登录");
    warn!(
        user_id = "user456",
        attempt = 3,
        "用户登录失败次数过多"
    );
    error!(
        error_code = "DB_001",
        error_message = "数据库连接失败",
        "发生错误"
    );

    // 带有上下文的日志
    info!(
        method = "GET",
        endpoint = "/api/users",
        status_code = 200,
        duration_ms = 45.2,
        "请求完成"
    );

    println!("✓ 记录了多种类型的结构化日志");
    println!("  - 查看日志: http://localhost:5601 (Kibana)");
    println!();

    // 5. 健康检查演示
    println!("--- 5. 健康检查演示 ---");
    let health = Arc::new(HealthCheck::new());

    // 注册组件
    health.register_component("database").await;
    health.register_component("cache").await;
    health.register_component("vector_store").await;
    info!("✓ 注册了 3 个组件");
    println!("✓ 注册了 3 个组件");

    // 更新组件状态
    health
        .update_component("database", HealthStatus::Healthy, None)
        .await;
    health
        .update_component("cache", HealthStatus::Healthy, None)
        .await;
    health
        .update_component(
            "vector_store",
            HealthStatus::Degraded,
            Some("高延迟".to_string()),
        )
        .await;

    // 检查存活性
    let liveness = health.liveness().await;
    println!("\n存活性检查:");
    println!("  状态: {}", liveness.status);
    println!("  版本: {}", liveness.version);
    println!("  运行时间: {} 秒", liveness.uptime_seconds);

    // 检查就绪性
    let readiness = health.readiness().await;
    println!("\n就绪性检查:");
    println!("  状态: {}", readiness.status);
    println!("  组件数: {}", readiness.components.len());
    for (name, status) in &readiness.components {
        println!("  - {}: {}", name, status.status);
        if let Some(msg) = &status.message {
            println!("    消息: {}", msg);
        }
    }
    println!();

    // 6. 性能分析演示
    println!("--- 6. 性能分析演示 ---");
    let analyzer = PerformanceAnalyzer::new();

    // 模拟一些操作
    for i in 0..20 {
        let duration = Duration::from_millis(10 + (i * 5));
        analyzer
            .record_operation("database_query", duration)
            .await;
    }

    for i in 0..15 {
        let duration = Duration::from_millis(50 + (i * 10));
        analyzer.record_operation("vector_search", duration).await;
    }

    for i in 0..10 {
        let duration = Duration::from_millis(5 + (i * 2));
        analyzer.record_operation("cache_lookup", duration).await;
    }

    info!("✓ 记录了 45 个操作");
    println!("✓ 记录了 45 个操作");

    // 获取性能报告
    let report = analyzer.get_report().await;
    println!("\n性能报告:");
    println!("  总操作数: {}", report.total_operations);
    println!("  平均延迟: {:.2} ms", report.avg_duration_ms);
    if let Some(slowest) = &report.slowest_operation {
        println!("  最慢操作: {} ({:.2} ms)", slowest.name, slowest.max_duration_ms);
    }

    // 识别慢操作
    let slow_ops = analyzer.identify_slow_operations(100.0).await;
    if !slow_ops.is_empty() {
        println!("\n慢操作（> 100ms）:");
        for op in slow_ops.iter().take(5) {
            println!("  - {}: {:.2} ms", op.name, op.max_duration_ms);
        }
    }
    println!();

    // 7. 模拟持续运行
    println!("--- 7. 持续监控 ---");
    println!("演示将持续运行 30 秒，生成监控数据...");
    println!("\n访问以下 URL 查看实时数据:");
    println!("  - Prometheus: http://localhost:9090");
    println!("  - Grafana: http://localhost:3000");
    println!("  - Jaeger: http://localhost:16686");
    println!("  - Kibana: http://localhost:5601");
    println!("\n按 Ctrl+C 停止演示\n");

    // 持续生成数据
    for i in 0..30 {
        tokio::time::sleep(Duration::from_secs(1)).await;

        // 记录请求
        let status = if i % 10 == 0 { 500 } else { 200 };
        collector
            .record_request("GET", "/api/data", status)
            .await;
        collector
            .record_request_duration("GET", "/api/data", 0.1 + (i as f64 * 0.01))
            .await;

        // 记录工具执行
        if i % 3 == 0 {
            collector
                .record_tool_execution("data_processor", 0.05)
                .await;
        }

        // 更新资源指标
        let memory_usage = 1024 * 1024 * (500 + (i * 10));
        collector.set_memory_usage(memory_usage).await;
        collector.set_active_connections((40 + i) as i64).await;

        // 记录性能数据
        let duration = Duration::from_millis(50 + (i * 5));
        analyzer.record_operation("api_call", duration).await;

        // 日志
        if i % 5 == 0 {
            info!(
                iteration = i,
                memory_mb = memory_usage / 1024 / 1024,
                "监控数据已更新"
            );
        }

        print!(".");
        if (i + 1) % 10 == 0 {
            println!(" {} 秒", i + 1);
        }
    }

    println!("\n\n=== 演示完成 ===");
    println!("\n最终统计:");
    let final_report = analyzer.get_report().await;
    println!("  总操作数: {}", final_report.total_operations);
    println!("  平均延迟: {:.2} ms", final_report.avg_duration_ms);
    if let Some(slowest) = &final_report.slowest_operation {
        println!("  最慢操作: {} ({:.2} ms)", slowest.name, slowest.max_duration_ms);
    }

    println!("\n提示:");
    println!("  - 指标已导出到 Prometheus");
    println!("  - 追踪已发送到 Jaeger");
    println!("  - 日志已发送到 Logstash/Elasticsearch");
    println!("  - 使用 Grafana 查看统一仪表板");

    Ok(())
}

/// 模拟用户请求处理（带追踪）
#[instrument]
async fn process_user_request(user_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("开始处理用户请求");

    // 步骤 1: 验证用户
    validate_user(user_id).await?;

    // 步骤 2: 获取数据
    fetch_user_data(user_id).await?;

    // 步骤 3: 处理数据
    process_data(user_id).await?;

    info!("用户请求处理完成");
    Ok(())
}

#[instrument]
async fn validate_user(user_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("验证用户");
    tokio::time::sleep(Duration::from_millis(10)).await;
    Ok(())
}

#[instrument]
async fn fetch_user_data(user_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("获取用户数据");

    // 嵌套 span: 数据库查询
    query_database(user_id).await?;

    // 嵌套 span: 缓存查询
    query_cache(user_id).await?;

    Ok(())
}

#[instrument]
async fn query_database(user_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("查询数据库");
    tokio::time::sleep(Duration::from_millis(50)).await;
    Ok(())
}

#[instrument]
async fn query_cache(user_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("查询缓存");
    tokio::time::sleep(Duration::from_millis(5)).await;
    Ok(())
}

#[instrument]
async fn process_data(user_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("处理数据");
    tokio::time::sleep(Duration::from_millis(20)).await;
    Ok(())
}

