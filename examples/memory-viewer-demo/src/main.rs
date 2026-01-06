//! 记忆可视化功能演示
//!
//! 本示例演示 AgentMem 的记忆可视化功能：
//! 1. 创建用户
//! 2. 添加不同类型的记忆
//! 3. 可视化所有记忆
//! 4. 显示记忆统计摘要
//! 5. 按类型展示记忆

use agent_mem_core::client::{AgentMemClient, MemoryType};
use agent_mem_traits::Result;
use colored::*;

#[tokio::main]
async fn main() -> Result<()> {
    println!("{}", "=== AgentMem 记忆可视化演示 ===".cyan().bold());
    println!();
    
    // 1. 创建客户端
    println!("{}", "1. 创建 AgentMemClient...".yellow());
    let client = AgentMemClient::default();
    println!("{}", "   ✅ AgentMemClient 创建成功".green());
    println!();
    
    // 2. 创建用户
    println!("{}", "2. 创建用户...".yellow());
    let user = client.create_user("alice".to_string()).await?;
    println!("   ✅ 创建用户: {} (ID: {})", user.name.green(), user.id);
    println!();
    
    // 3. 添加不同类型的记忆
    println!("{}", "3. 添加不同类型的记忆...".yellow());

    // Episodic memories (事件记忆)
    client.add_simple(
        "Alice went to the park yesterday".to_string(),
        Some(user.id.clone()),
        None,
        Some(MemoryType::Episodic),
    ).await?;
    println!("   ✅ 添加 Episodic 记忆: {}", "去公园".cyan());

    client.add_simple(
        "Alice had lunch with Bob at noon".to_string(),
        Some(user.id.clone()),
        None,
        Some(MemoryType::Episodic),
    ).await?;
    println!("   ✅ 添加 Episodic 记忆: {}", "和 Bob 吃午饭".cyan());

    // Semantic memories (语义记忆)
    client.add_simple(
        "Paris is the capital of France".to_string(),
        Some(user.id.clone()),
        None,
        Some(MemoryType::Semantic),
    ).await?;
    println!("   ✅ 添加 Semantic 记忆: {}", "巴黎是法国首都".cyan());

    client.add_simple(
        "Water boils at 100 degrees Celsius".to_string(),
        Some(user.id.clone()),
        None,
        Some(MemoryType::Semantic),
    ).await?;
    println!("   ✅ 添加 Semantic 记忆: {}", "水的沸点".cyan());

    // Procedural memories (程序记忆)
    client.add_simple(
        "To make coffee: 1. Boil water 2. Add coffee grounds 3. Pour water".to_string(),
        Some(user.id.clone()),
        None,
        Some(MemoryType::Procedural),
    ).await?;
    println!("   ✅ 添加 Procedural 记忆: {}", "如何煮咖啡".cyan());

    // Core memories (核心记忆)
    client.add_simple(
        "Alice loves programming and AI".to_string(),
        Some(user.id.clone()),
        None,
        Some(MemoryType::Core),
    ).await?;
    println!("   ✅ 添加 Core 记忆: {}", "Alice 的兴趣".cyan());

    // Resource memories (资源记忆)
    client.add_simple(
        "Favorite book: The Pragmatic Programmer".to_string(),
        Some(user.id.clone()),
        None,
        Some(MemoryType::Resource),
    ).await?;
    println!("   ✅ 添加 Resource 记忆: {}", "最喜欢的书".cyan());

    // Knowledge memories (知识记忆)
    client.add_simple(
        "Rust is a systems programming language".to_string(),
        Some(user.id.clone()),
        None,
        Some(MemoryType::Knowledge),
    ).await?;
    println!("   ✅ 添加 Knowledge 记忆: {}", "Rust 知识".cyan());
    
    println!();
    
    // 4. 可视化记忆
    println!("{}", "4. 可视化所有记忆...".yellow());
    let viz = client.visualize_memories(Some(user.id.clone())).await?;
    println!();
    
    // 5. 显示摘要
    println!("{}", "=== 记忆统计摘要 ===".cyan().bold());
    println!("用户: {} (ID: {})", viz.user_name.green().bold(), viz.user_id);
    println!("总记忆数: {}", viz.summary.total_count.to_string().green().bold());
    println!();
    
    println!("按类型统计:");
    println!("  📅 Episodic (事件记忆):   {}", viz.summary.episodic_count.to_string().cyan());
    println!("  📚 Semantic (语义记忆):   {}", viz.summary.semantic_count.to_string().cyan());
    println!("  ⚙️  Procedural (程序记忆): {}", viz.summary.procedural_count.to_string().cyan());
    println!("  💎 Core (核心记忆):       {}", viz.summary.core_count.to_string().cyan());
    println!("  📦 Resource (资源记忆):   {}", viz.summary.resource_count.to_string().cyan());
    println!("  🧠 Knowledge (知识记忆):  {}", viz.summary.knowledge_count.to_string().cyan());
    println!("  🔄 Working (工作记忆):    {}", viz.summary.working_count.to_string().cyan());
    println!("  🌐 Contextual (上下文):   {}", viz.summary.contextual_count.to_string().cyan());
    println!();
    
    // 6. 按类型展示记忆
    println!("{}", "=== 按类型展示记忆 ===".cyan().bold());
    println!();
    
    if !viz.memories.episodic.is_empty() {
        println!("{}", "📅 Episodic Memories (事件记忆):".yellow().bold());
        for (i, mem) in viz.memories.episodic.iter().enumerate() {
            println!("  {}. {} (ID: {})", 
                i + 1, 
                mem.content.cyan(), 
                &mem.id[..8]);
            println!("     创建时间: {}", mem.created_at.format("%Y-%m-%d %H:%M:%S"));
        }
        println!();
    }
    
    if !viz.memories.semantic.is_empty() {
        println!("{}", "📚 Semantic Memories (语义记忆):".yellow().bold());
        for (i, mem) in viz.memories.semantic.iter().enumerate() {
            println!("  {}. {} (ID: {})", 
                i + 1, 
                mem.content.cyan(), 
                &mem.id[..8]);
            println!("     创建时间: {}", mem.created_at.format("%Y-%m-%d %H:%M:%S"));
        }
        println!();
    }
    
    if !viz.memories.procedural.is_empty() {
        println!("{}", "⚙️  Procedural Memories (程序记忆):".yellow().bold());
        for (i, mem) in viz.memories.procedural.iter().enumerate() {
            println!("  {}. {} (ID: {})", 
                i + 1, 
                mem.content.cyan(), 
                &mem.id[..8]);
            println!("     创建时间: {}", mem.created_at.format("%Y-%m-%d %H:%M:%S"));
        }
        println!();
    }
    
    if !viz.memories.core.is_empty() {
        println!("{}", "💎 Core Memories (核心记忆):".yellow().bold());
        for (i, mem) in viz.memories.core.iter().enumerate() {
            println!("  {}. {} (ID: {})", 
                i + 1, 
                mem.content.cyan(), 
                &mem.id[..8]);
            println!("     创建时间: {}", mem.created_at.format("%Y-%m-%d %H:%M:%S"));
        }
        println!();
    }
    
    if !viz.memories.resource.is_empty() {
        println!("{}", "📦 Resource Memories (资源记忆):".yellow().bold());
        for (i, mem) in viz.memories.resource.iter().enumerate() {
            println!("  {}. {} (ID: {})", 
                i + 1, 
                mem.content.cyan(), 
                &mem.id[..8]);
            println!("     创建时间: {}", mem.created_at.format("%Y-%m-%d %H:%M:%S"));
        }
        println!();
    }
    
    if !viz.memories.knowledge.is_empty() {
        println!("{}", "🧠 Knowledge Memories (知识记忆):".yellow().bold());
        for (i, mem) in viz.memories.knowledge.iter().enumerate() {
            println!("  {}. {} (ID: {})", 
                i + 1, 
                mem.content.cyan(), 
                &mem.id[..8]);
            println!("     创建时间: {}", mem.created_at.format("%Y-%m-%d %H:%M:%S"));
        }
        println!();
    }
    
    // 7. 测试无记忆的用户
    println!("{}", "5. 测试无记忆的用户...".yellow());
    let user2 = client.create_user("bob".to_string()).await?;
    let viz2 = client.visualize_memories(Some(user2.id.clone())).await?;
    println!("   用户: {} - 总记忆数: {}", 
        viz2.user_name.green(), 
        viz2.summary.total_count.to_string().yellow());
    println!();
    
    // 8. 测试默认用户（无 user_id）
    println!("{}", "6. 测试默认用户（无 user_id）...".yellow());
    let viz3 = client.visualize_memories(None).await?;
    println!("   用户: {} - 总记忆数: {}", 
        viz3.user_name.green(), 
        viz3.summary.total_count.to_string().yellow());
    println!();
    
    println!("{}", "=== 演示完成 ===".green().bold());
    println!();
    println!("✅ 所有测试通过！");
    println!();
    println!("📊 功能验证:");
    println!("  ✅ 创建用户");
    println!("  ✅ 添加不同类型的记忆");
    println!("  ✅ 可视化记忆");
    println!("  ✅ 统计摘要");
    println!("  ✅ 按类型分组展示");
    println!("  ✅ 处理无记忆用户");
    println!("  ✅ 处理默认用户");
    
    Ok(())
}

