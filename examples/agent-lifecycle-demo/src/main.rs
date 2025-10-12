//! Agent 生命周期演示
//! 
//! 本示例演示 AgentMem 中所有 Agent 的 initialize() 和 shutdown() 方法。
//! 
//! 功能:
//! - 创建 5 个不同类型的 Agent
//! - 调用 initialize() 方法初始化 Agent
//! - 执行简单任务
//! - 调用 shutdown() 方法关闭 Agent
//! 
//! 运行方式:
//! ```bash
//! cargo run --example agent-lifecycle-demo
//! ```

use agent_mem_core::agents::{
    core_agent::CoreAgent,
    episodic_agent::EpisodicAgent,
    semantic_agent::SemanticAgent,
    procedural_agent::ProceduralAgent,
    working_agent::WorkingAgent,
    MemoryAgent,
};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    println!("\n╔══════════════════════════════════════════════════════════════════════╗");
    println!("║          AgentMem Agent 生命周期演示                                  ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝\n");

    // 1. 测试 CoreAgent
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("1️⃣  测试 CoreAgent (核心记忆 Agent)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let mut core_agent = CoreAgent::new("core-agent-demo".to_string());
    
    println!("📌 调用 initialize()...");
    core_agent.initialize().await?;
    
    println!("✅ CoreAgent 初始化完成\n");
    
    println!("📌 调用 shutdown()...");
    core_agent.shutdown().await?;
    
    println!("✅ CoreAgent 关闭完成\n");

    // 2. 测试 EpisodicAgent
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("2️⃣  测试 EpisodicAgent (情景记忆 Agent)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let mut episodic_agent = EpisodicAgent::new("episodic-agent-demo".to_string());

    println!("📌 调用 initialize()...");
    episodic_agent.initialize().await?;

    println!("✅ EpisodicAgent 初始化完成\n");

    println!("📌 调用 shutdown()...");
    episodic_agent.shutdown().await?;

    println!("✅ EpisodicAgent 关闭完成\n");

    // 3. 测试 SemanticAgent
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("3️⃣  测试 SemanticAgent (语义记忆 Agent)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let mut semantic_agent = SemanticAgent::new("semantic-agent-demo".to_string());

    println!("📌 调用 initialize()...");
    semantic_agent.initialize().await?;

    println!("✅ SemanticAgent 初始化完成\n");

    println!("📌 调用 shutdown()...");
    semantic_agent.shutdown().await?;

    println!("✅ SemanticAgent 关闭完成\n");

    // 4. 测试 ProceduralAgent
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("4️⃣  测试 ProceduralAgent (程序记忆 Agent)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let mut procedural_agent = ProceduralAgent::new("procedural-agent-demo".to_string());

    println!("📌 调用 initialize()...");
    procedural_agent.initialize().await?;

    println!("✅ ProceduralAgent 初始化完成\n");

    println!("📌 调用 shutdown()...");
    procedural_agent.shutdown().await?;

    println!("✅ ProceduralAgent 关闭完成\n");

    // 5. 测试 WorkingAgent
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("5️⃣  测试 WorkingAgent (工作记忆 Agent)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let mut working_agent = WorkingAgent::new("working-agent-demo".to_string());
    
    println!("📌 调用 initialize()...");
    working_agent.initialize().await?;
    
    println!("✅ WorkingAgent 初始化完成\n");
    
    println!("📌 调用 shutdown()...");
    working_agent.shutdown().await?;
    
    println!("✅ WorkingAgent 关闭完成\n");

    // 总结
    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║          🎉 所有 Agent 生命周期测试完成！                             ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝\n");

    println!("✅ 测试结果:");
    println!("   - CoreAgent: 初始化 ✅ | 关闭 ✅");
    println!("   - EpisodicAgent: 初始化 ✅ | 关闭 ✅");
    println!("   - SemanticAgent: 初始化 ✅ | 关闭 ✅");
    println!("   - ProceduralAgent: 初始化 ✅ | 关闭 ✅");
    println!("   - WorkingAgent: 初始化 ✅ | 关闭 ✅\n");

    println!("📝 说明:");
    println!("   - 所有 Agent 都未配置存储后端，因此以只读模式运行");
    println!("   - 如需测试存储集成，请配置相应的存储后端（PostgreSQL, LibSQL 等）");
    println!("   - 详细日志请查看上方输出\n");

    Ok(())
}

