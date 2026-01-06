//! AgentMem 统一 API 演示
//!
//! 这个示例演示了新的统一 Memory API 的使用方法

use agent_mem::Memory;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();
    
    info!("=== AgentMem 统一 API 演示 ===\n");
    
    // 示例 1: 零配置模式
    info!("示例 1: 零配置模式");
    demo_zero_config().await?;
    
    // 示例 2: Builder 模式
    info!("\n示例 2: Builder 模式");
    demo_builder_pattern().await?;
    
    // 示例 3: 添加和搜索记忆
    info!("\n示例 3: 添加和搜索记忆");
    demo_add_and_search().await?;
    
    info!("\n=== 演示完成 ===");
    
    Ok(())
}

/// 示例 1: 零配置模式
async fn demo_zero_config() -> Result<(), Box<dyn std::error::Error>> {
    info!("创建 Memory 实例（零配置）...");
    
    let mem = Memory::new().await?;
    
    info!("✅ Memory 实例创建成功！");
    info!("   - 自动配置完成");
    info!("   - 使用默认存储后端");
    
    Ok(())
}

/// 示例 2: Builder 模式
async fn demo_builder_pattern() -> Result<(), Box<dyn std::error::Error>> {
    info!("使用 Builder 模式创建 Memory 实例...");
    
    let mem = Memory::builder()
        .with_storage("libsql://unified_api_demo.db")
        .with_user("alice")
        .with_agent("demo-agent")
        .build()
        .await?;
    
    info!("✅ Memory 实例创建成功！");
    info!("   - 存储: libsql://unified_api_demo.db");
    info!("   - 用户: alice");
    info!("   - Agent: demo-agent");
    
    Ok(())
}

/// 示例 3: 添加和搜索记忆
async fn demo_add_and_search() -> Result<(), Box<dyn std::error::Error>> {
    info!("创建 Memory 实例...");
    let mem = Memory::new().await?;
    
    // 添加记忆
    info!("添加记忆...");
    let id1 = mem.add("I love pizza").await?;
    info!("✅ 记忆已添加: {}", id1);
    
    let id2 = mem.add("I work at Google").await?;
    info!("✅ 记忆已添加: {}", id2);
    
    let id3 = mem.add("My favorite color is blue").await?;
    info!("✅ 记忆已添加: {}", id3);
    
    // 搜索记忆
    info!("\n搜索记忆: 'What do you know about me?'");
    let results = mem.search("What do you know about me?").await?;
    info!("找到 {} 条记忆", results.len());
    
    // 获取所有记忆
    info!("\n获取所有记忆...");
    let all_memories = mem.get_all().await?;
    info!("总共 {} 条记忆", all_memories.len());
    
    // 获取统计信息
    info!("\n获取统计信息...");
    let stats = mem.get_stats().await?;
    info!("统计信息:");
    info!("  - 总记忆数: {}", stats.total_memories);
    info!("  - 平均重要性: {:.2}", stats.average_importance);
    
    Ok(())
}

