//! MCP Resources 功能演示
//!
//! 本示例演示了 MCP (Model Context Protocol) Resources 功能的使用：
//! 1. 列出所有可用资源
//! 2. 读取资源内容
//! 3. 订阅资源变更
//! 4. 取消订阅
//! 5. 通知资源变更
//! 6. 缓存管理

use agent_mem_tools::mcp::{
    ResourceManager, ResourceChangeType, McpServer,
    McpReadResourceRequest, McpSubscribeResourceRequest,
};
use agent_mem_tools::mcp::server::McpServerConfig;
use agent_mem_tools::executor::ToolExecutor;
use std::sync::Arc;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("🚀 MCP Resources 功能演示");
    info!("{}", "=".repeat(60));

    // 演示 1: 创建资源管理器
    demo_resource_manager().await?;

    info!("");
    info!("{}", "=".repeat(60));

    // 演示 2: MCP 服务器集成
    demo_mcp_server().await?;

    info!("");
    info!("🎉 所有演示完成！");

    Ok(())
}

/// 演示资源管理器功能
async fn demo_resource_manager() -> anyhow::Result<()> {
    info!("📋 演示 1: 资源管理器功能");
    info!("{}", "-".repeat(60));

    // 创建资源管理器（缓存 TTL 为 300 秒）
    let manager = ResourceManager::new(300);

    // 1. 列出所有资源
    info!("1️⃣ 列出所有资源:");
    let resources = manager.list_resources().await?;
    for resource in &resources {
        info!("  - {} ({})", resource.name, resource.uri);
        if let Some(desc) = &resource.description {
            info!("    描述: {}", desc);
        }
        if let Some(mime) = &resource.mime_type {
            info!("    类型: {}", mime);
        }
    }
    info!("  总计: {} 个资源", resources.len());

    // 2. 读取资源内容
    info!("");
    info!("2️⃣ 读取资源内容:");
    let uri = "agentmem://memory/core";
    info!("  读取资源: {}", uri);
    let content = manager.read_resource(uri).await?;
    info!("  ✅ 成功读取资源");
    info!("  URI: {}", content.uri);
    if let Some(modified) = content.last_modified {
        info!("  最后修改: {}", modified);
    }

    // 3. 测试缓存
    info!("");
    info!("3️⃣ 测试资源缓存:");
    info!("  第一次读取（从存储）...");
    let start = std::time::Instant::now();
    let _ = manager.read_resource(uri).await?;
    let duration1 = start.elapsed();
    info!("  耗时: {:?}", duration1);

    info!("  第二次读取（从缓存）...");
    let start = std::time::Instant::now();
    let _ = manager.read_resource(uri).await?;
    let duration2 = start.elapsed();
    info!("  耗时: {:?}", duration2);
    info!("  ✅ 缓存加速: {:.2}x", duration1.as_nanos() as f64 / duration2.as_nanos() as f64);

    // 4. 订阅资源
    info!("");
    info!("4️⃣ 订阅资源变更:");
    let subscription = manager.subscribe_resource(uri).await?;
    info!("  ✅ 订阅成功");
    info!("  订阅 ID: {}", subscription.id);
    info!("  资源 URI: {}", subscription.uri);
    info!("  创建时间: {}", subscription.created_at);

    // 5. 通知资源变更
    info!("");
    info!("5️⃣ 通知资源变更:");
    manager.notify_resource_change(uri, ResourceChangeType::Updated).await?;
    info!("  ✅ 已通知资源更新");

    // 6. 列出所有订阅
    info!("");
    info!("6️⃣ 列出所有订阅:");
    let subscriptions = manager.list_subscriptions().await;
    info!("  总计: {} 个订阅", subscriptions.len());
    for sub in &subscriptions {
        info!("  - {} -> {}", sub.id, sub.uri);
    }

    // 7. 取消订阅
    info!("");
    info!("7️⃣ 取消订阅:");
    manager.unsubscribe_resource(&subscription.id).await?;
    info!("  ✅ 订阅已取消");

    // 8. 获取缓存统计
    info!("");
    info!("8️⃣ 缓存统计:");
    let stats = manager.get_cache_stats().await;
    info!("  缓存的资源数: {}", stats.total_cached);
    info!("  活跃订阅数: {}", stats.total_subscriptions);

    // 9. 清理过期缓存
    info!("");
    info!("9️⃣ 清理过期缓存:");
    let removed = manager.cleanup_expired_cache().await;
    info!("  清理了 {} 个过期缓存项", removed);

    Ok(())
}

/// 演示 MCP 服务器集成
async fn demo_mcp_server() -> anyhow::Result<()> {
    info!("📋 演示 2: MCP 服务器集成");
    info!("{}", "-".repeat(60));

    // 创建 MCP 服务器
    let config = McpServerConfig::default();

    let tool_executor = Arc::new(ToolExecutor::new());
    let server = McpServer::new(config, tool_executor);

    // 初始化服务器
    info!("1️⃣ 初始化 MCP 服务器:");
    server.initialize().await?;
    info!("  ✅ 服务器已初始化");

    // 获取服务器信息
    info!("");
    info!("2️⃣ 服务器信息:");
    let info_data = server.get_server_info();
    info!("  名称: {}", info_data.name);
    info!("  版本: {}", info_data.version);
    info!("  描述: {}", info_data.description);
    info!("  协议版本: {}", info_data.protocol_version);
    info!("  能力:");
    info!("    - Tools: {}", info_data.capabilities.tools);
    info!("    - Resources: {}", info_data.capabilities.resources);
    info!("    - Prompts: {}", info_data.capabilities.prompts);

    // 列出资源
    info!("");
    info!("3️⃣ 列出 MCP 资源:");
    let response = server.list_resources().await?;
    info!("  找到 {} 个资源:", response.resources.len());
    for resource in &response.resources {
        info!("    - {}", resource.name);
    }

    // 读取资源
    info!("");
    info!("4️⃣ 读取 MCP 资源:");
    let request = McpReadResourceRequest {
        uri: "agentmem://memory/core".to_string(),
    };
    let response = server.read_resource(request).await?;
    info!("  ✅ 成功读取 {} 个内容项", response.contents.len());

    // 订阅资源
    info!("");
    info!("5️⃣ 订阅 MCP 资源:");
    let request = McpSubscribeResourceRequest {
        uri: "agentmem://memory/semantic".to_string(),
    };
    let response = server.subscribe_resource(request).await?;
    info!("  ✅ 订阅成功");
    info!("  订阅 ID: {}", response.subscription_id);

    // 通知变更
    info!("");
    info!("6️⃣ 通知资源变更:");
    server.notify_resource_change(
        "agentmem://memory/semantic",
        ResourceChangeType::Updated,
    ).await?;
    info!("  ✅ 已通知变更");

    // 取消订阅
    info!("");
    info!("7️⃣ 取消订阅:");
    server.unsubscribe_resource(&response.subscription_id).await?;
    info!("  ✅ 订阅已取消");

    // 获取资源管理器统计
    info!("");
    info!("8️⃣ 资源管理器统计:");
    let stats = server.resource_manager().get_cache_stats().await;
    info!("  缓存的资源数: {}", stats.total_cached);
    info!("  活跃订阅数: {}", stats.total_subscriptions);

    Ok(())
}

