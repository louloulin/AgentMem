//! System Metrics Demo
//!
//! 系统指标监控演示
//!
//! 本示例演示如何使用 AgentMem 的监控系统收集和导出系统指标：
//! - CPU 使用率监控
//! - 内存使用率监控
//! - Prometheus 指标导出
//!
//! ## 运行方式
//!
//! ```bash
//! cargo run --package system-metrics-demo
//! ```
//!
//! 然后访问 http://localhost:9090/metrics 查看 Prometheus 指标

use agent_mem_observability::metrics::{MetricsRegistry, SystemMetricsMonitor};
use std::time::Duration;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    info!("🚀 启动系统指标监控演示");

    // 创建指标注册表
    let registry = MetricsRegistry::new();
    let collector = registry.collector();

    info!("📊 创建系统指标监控器");

    // 创建系统监控器（每 2 秒收集一次）
    let monitor = SystemMetricsMonitor::new(collector.clone(), Duration::from_secs(2));

    // 启动监控
    let monitor_handle = monitor.start();
    info!("✅ 系统监控已启动（每 2 秒收集一次）");

    // 启动 Prometheus 指标服务器
    info!("🌐 启动 Prometheus 指标服务器: http://localhost:9090/metrics");
    let metrics_server = tokio::spawn(async move {
        if let Err(e) =
            agent_mem_observability::metrics::start_metrics_server(registry.registry(), 9090).await
        {
            eprintln!("指标服务器错误: {e}");
        }
    });

    // 模拟一些应用活动
    info!("🔄 模拟应用活动...");
    for i in 1..=10 {
        tokio::time::sleep(Duration::from_secs(3)).await;

        // 记录一些请求指标
        collector.record_request("GET", "/api/health", 200).await;
        collector
            .record_request_duration("GET", "/api/health", 0.005)
            .await;

        // 设置活动连接数
        collector.set_active_connections(i * 5).await;

        // 设置应用内存使用（模拟）
        collector
            .set_memory_usage(1024 * 1024 * (100 + i * 10) as u64)
            .await;

        info!("📈 已记录第 {} 次请求指标", i);
    }

    info!("✅ 演示完成！");
    info!("💡 提示：");
    info!("   1. 访问 http://localhost:9090/metrics 查看所有指标");
    info!("   2. 查找以下指标：");
    info!("      - agentmem_cpu_usage_percent: CPU 使用率");
    info!("      - agentmem_system_memory_total_bytes: 系统总内存");
    info!("      - agentmem_system_memory_used_bytes: 已使用内存");
    info!("      - agentmem_system_memory_available_bytes: 可用内存");
    info!("      - agentmem_requests_total: 请求总数");
    info!("      - agentmem_active_connections: 活动连接数");
    info!("   3. 按 Ctrl+C 停止服务器");

    // 等待用户中断
    tokio::signal::ctrl_c().await?;

    info!("🛑 正在停止...");
    monitor_handle.abort();
    metrics_server.abort();

    Ok(())
}
