//! MCP Logging 功能演示
//!
//! 演示如何使用 MCP Logging 功能进行日志管理

use agent_mem_tools::mcp::{
    LoggingManager, LogEntry, LogLevel, LogFilter, LoggingConfig,
};
use futures::StreamExt;
use tracing::Level;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化 tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    println!("=== MCP Logging 功能演示 ===\n");

    // 演示 1: 日志级别管理
    demo_log_levels().await?;

    // 演示 2: 日志记录和查询
    demo_logging().await?;

    // 演示 3: 日志过滤
    demo_filtering().await?;

    // 演示 4: 日志统计
    demo_stats().await?;

    // 演示 5: 日志流式传输
    demo_streaming().await?;

    println!("\n=== 演示完成 ===");
    Ok(())
}

/// 演示 1: 日志级别管理
async fn demo_log_levels() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- 演示 1: 日志级别管理 ---");

    // 日志级别顺序
    println!("✓ 日志级别顺序（从低到高）:");
    println!("  Trace < Debug < Info < Warn < Error");

    // 日志级别转换
    println!("\n✓ 日志级别字符串转换:");
    let levels = vec!["trace", "debug", "info", "warn", "error"];
    for level_str in levels {
        if let Some(level) = LogLevel::from_str(level_str) {
            println!("  \"{}\" -> {:?}", level_str, level);
        }
    }

    // 默认日志级别
    let default_level = LogLevel::default();
    println!("\n✓ 默认日志级别: {:?}", default_level);

    println!();
    Ok(())
}

/// 演示 2: 日志记录和查询
async fn demo_logging() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- 演示 2: 日志记录和查询 ---");

    let manager = LoggingManager::new(LoggingConfig::default());
    println!("✓ 创建日志管理器");
    println!("  启用状态: {}", manager.is_enabled());

    // 记录不同级别的日志
    println!("\n✓ 记录不同级别的日志:");
    
    manager.log(
        LogEntry::new(LogLevel::Info, "应用启动", "app")
            .with_operation("startup")
            .with_tag("system")
    ).await?;
    println!("  [INFO] 应用启动");

    manager.log(
        LogEntry::new(LogLevel::Debug, "加载配置文件", "config")
            .with_operation("load")
            .with_field("file", serde_json::json!("config.toml"))
    ).await?;
    println!("  [DEBUG] 加载配置文件");

    manager.log(
        LogEntry::new(LogLevel::Warn, "连接超时，正在重试", "network")
            .with_operation("connect")
            .with_user_id("user1")
    ).await?;
    println!("  [WARN] 连接超时，正在重试");

    manager.log(
        LogEntry::new(LogLevel::Error, "数据库连接失败", "database")
            .with_operation("connect")
            .with_field("error", serde_json::json!("connection refused"))
    ).await?;
    println!("  [ERROR] 数据库连接失败");

    // 查询所有日志
    let all_logs = manager.get_logs(None, None).await;
    println!("\n✓ 查询所有日志:");
    println!("  总数: {}", all_logs.len());

    // 查询特定级别的日志
    let warn_logs = manager.query_logs(
        Some(LogLevel::Warn),
        None,
        None,
        None,
        None,
    ).await;
    println!("\n✓ 查询 Warn 及以上级别的日志:");
    println!("  数量: {}", warn_logs.len());
    for log in &warn_logs {
        println!("  - [{:?}] {}: {}", log.level, log.component, log.message);
    }

    // 查询特定组件的日志
    let network_logs = manager.query_logs(
        None,
        Some("network"),
        None,
        None,
        None,
    ).await;
    println!("\n✓ 查询 network 组件的日志:");
    println!("  数量: {}", network_logs.len());

    println!();
    Ok(())
}

/// 演示 3: 日志过滤
async fn demo_filtering() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- 演示 3: 日志过滤 ---");

    let manager = LoggingManager::new(LoggingConfig::default());

    // 添加过滤器：只记录 Warn 及以上级别的日志
    let filter = LogFilter::new("warn-filter")
        .with_min_level(LogLevel::Warn);
    manager.add_filter(filter).await;
    println!("✓ 添加过滤器: 只记录 Warn 及以上级别");

    // 记录不同级别的日志
    println!("\n✓ 记录日志:");
    
    manager.log(
        LogEntry::new(LogLevel::Info, "这条日志会被过滤", "test")
    ).await?;
    println!("  [INFO] 这条日志会被过滤 (不会被记录)");

    manager.log(
        LogEntry::new(LogLevel::Warn, "这条日志会被记录", "test")
    ).await?;
    println!("  [WARN] 这条日志会被记录");

    manager.log(
        LogEntry::new(LogLevel::Error, "这条日志也会被记录", "test")
    ).await?;
    println!("  [ERROR] 这条日志也会被记录");

    // 查看实际记录的日志
    let logs = manager.get_logs(None, None).await;
    println!("\n✓ 实际记录的日志数量: {}", logs.len());
    for log in &logs {
        println!("  - [{:?}] {}", log.level, log.message);
    }

    // 移除过滤器
    manager.remove_filter("warn-filter").await;
    println!("\n✓ 移除过滤器");

    // 添加组件过滤器
    let comp_filter = LogFilter::new("mcp-filter")
        .with_component("mcp");
    manager.add_filter(comp_filter).await;
    println!("✓ 添加组件过滤器: 只记录 mcp 组件");

    manager.log(
        LogEntry::new(LogLevel::Info, "MCP 服务器启动", "mcp-server")
    ).await?;
    println!("  [INFO] MCP 服务器启动 (会被记录)");

    manager.log(
        LogEntry::new(LogLevel::Info, "其他组件日志", "other")
    ).await?;
    println!("  [INFO] 其他组件日志 (不会被记录)");

    // 获取所有过滤器
    let filters = manager.get_filters().await;
    println!("\n✓ 当前过滤器数量: {}", filters.len());

    println!();
    Ok(())
}

/// 演示 4: 日志统计
async fn demo_stats() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- 演示 4: 日志统计 ---");

    let manager = LoggingManager::new(LoggingConfig::default());

    // 记录多条日志
    println!("✓ 记录多条日志...");
    for i in 0..5 {
        manager.log(
            LogEntry::new(LogLevel::Info, format!("Info 日志 {}", i), "app")
        ).await?;
    }
    for i in 0..3 {
        manager.log(
            LogEntry::new(LogLevel::Warn, format!("Warn 日志 {}", i), "network")
        ).await?;
    }
    for i in 0..2 {
        manager.log(
            LogEntry::new(LogLevel::Error, format!("Error 日志 {}", i), "database")
        ).await?;
    }

    // 获取统计信息
    let stats = manager.get_stats().await;
    println!("\n✓ 日志统计信息:");
    println!("  总日志数: {}", stats.total_logs);
    println!("  Trace 级别: {}", stats.trace_count);
    println!("  Debug 级别: {}", stats.debug_count);
    println!("  Info 级别: {}", stats.info_count);
    println!("  Warn 级别: {}", stats.warn_count);
    println!("  Error 级别: {}", stats.error_count);

    println!("\n✓ 各组件日志数:");
    for (component, count) in &stats.components {
        println!("  {}: {}", component, count);
    }

    // 清空日志
    manager.clear().await;
    println!("\n✓ 清空日志");
    let logs_after_clear = manager.get_logs(None, None).await;
    println!("  清空后日志数: {}", logs_after_clear.len());

    println!();
    Ok(())
}

/// 演示 5: 日志流式传输
async fn demo_streaming() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- 演示 5: 日志流式传输 ---");

    let config = LoggingConfig {
        enabled: true,
        default_level: LogLevel::Info,
        max_buffer_size: 1000,
        enable_streaming: true,
        integrate_tracing: true,
    };
    let manager = LoggingManager::new(config);
    println!("✓ 创建启用流式传输的日志管理器");

    // 订阅日志流
    let mut stream = manager.subscribe().await?;
    println!("✓ 订阅日志流");

    // 在后台任务中记录日志
    let manager_clone = manager.clone();
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        for i in 0..3 {
            let _ = manager_clone.log(
                LogEntry::new(LogLevel::Info, format!("流式日志 {}", i), "stream-test")
            ).await;
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    });

    // 接收流式日志
    println!("\n✓ 接收流式日志:");
    let mut count = 0;
    while let Some(entry) = stream.next().await {
        println!("  - [{:?}] {}: {}", entry.level, entry.component, entry.message);
        count += 1;
        if count >= 3 {
            break;
        }
    }

    println!();
    Ok(())
}

