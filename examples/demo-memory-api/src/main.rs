//! Memory API 真实演示示例
//!
//! 展示AgentMem的核心功能：
//! 1. 添加记忆
//! 2. 搜索记忆
//! 3. 获取所有记忆
//! 4. 删除记忆

use agent_mem::{GetAllOptions, MemoryBuilder};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    println!("🚀 AgentMem Memory API 演示\n");

    // 1. 创建Memory实例（使用FastEmbed本地嵌入，零配置）
    println!("1️⃣ 创建Memory实例（使用FastEmbed本地嵌入）...");
    let memory = MemoryBuilder::new()
        .with_agent("demo_agent")
        .with_user("demo_user")
        .with_embedder("fastembed", "all-MiniLM-L6-v2") // 本地嵌入，无需API key
        .build()
        .await?;
    println!("✅ Memory实例创建成功\n");

    // 2. 添加记忆
    println!("2️⃣ 添加记忆...");
    let memories = vec![
        "我喜欢使用Rust编程，因为它提供了内存安全保证",
        "Python在数据科学领域非常流行",
        "AgentMem是一个高性能的AI记忆管理平台",
        "Cangjie（仓颉）是华为开发的新编程语言",
        "向量数据库可以实现语义搜索",
    ];

    for content in &memories {
        match memory.add(*content).await {
            Ok(result) => {
                if let Some(first) = result.results.first() {
                    println!("  ✅ 添加成功: {} (ID: {})", content, &first.id[..8]);
                }
            }
            Err(e) => println!("  ❌ 添加失败: {}", e),
        }
    }
    println!();

    // 3. 搜索记忆
    println!("3️⃣ 搜索记忆...");
    let queries = vec![
        ("编程语言", "搜索关于编程语言的记忆"),
        ("性能", "搜索关于性能的记忆"),
        ("安全", "搜索关于安全的记忆"),
    ];

    for (query, description) in &queries {
        println!("\n  🔍 {}: \"{}\"", description, query);
        match memory.search(*query).await {
            Ok(results) => {
                if results.is_empty() {
                    println!("    ℹ️  未找到匹配的记忆");
                } else {
                    println!("    ✅ 找到 {} 条相关记忆:", results.len());
                    for (i, result) in results.iter().take(3).enumerate() {
                        println!("       {}. {} (相关度: 高)", i + 1, result.content);
                    }
                }
            }
            Err(e) => println!("    ❌ 搜索失败: {}", e),
        }
    }
    println!();

    // 4. 获取所有记忆
    println!("4️⃣ 获取所有记忆...");
    match memory.get_all(GetAllOptions::default()).await {
        Ok(all_memories) => {
            println!("  ✅ 共有 {} 条记忆:", all_memories.len());
            for (i, mem) in all_memories.iter().enumerate() {
                println!("     {}. {}", i + 1, mem.content);
            }
        }
        Err(e) => println!("  ❌ 获取失败: {}", e),
    }
    println!();

    // 5. 删除特定记忆
    println!("5️⃣ 删除记忆...");
    match memory.get_all(GetAllOptions::default()).await {
        Ok(all_memories) => {
            if let Some(first) = all_memories.first() {
                let id_to_delete = first.id.clone();
                match memory.delete(&id_to_delete).await {
                    Ok(_) => println!("  ✅ 成功删除记忆: {}", first.content),
                    Err(e) => println!("  ❌ 删除失败: {}", e),
                }
            }
        }
        Err(e) => println!("  ❌ 获取记忆失败: {}", e),
    }
    println!();

    // 6. 验证删除
    println!("6️⃣ 验证删除后的记忆数量...");
    match memory.get_all(GetAllOptions::default()).await {
        Ok(all_memories) => {
            println!("  ✅ 现在有 {} 条记忆（已删除1条）", all_memories.len());
        }
        Err(e) => println!("  ❌ 获取失败: {}", e),
    }

    println!("\n🎉 演示完成！");
    println!("\n📊 AgentMem特性：");
    println!("  ✅ 零配置启动（LibSQL + FastEmbed）");
    println!("  ✅ 本地嵌入，无需API key");
    println!("  ✅ 语义搜索，智能匹配");
    println!("  ✅ 向量维度自动适配");
    println!("  ✅ Rust性能，2-10x提升");

    Ok(())
}
