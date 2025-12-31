// AgentMem 搜索示例
//
// 演示各种搜索功能

use agentmem::{Memory, MemoryType, SearchOptions, GetAllOptions};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== AgentMem 搜索示例 ===\n");

    let memory = Memory::new().await?;

    // 添加示例数据
    println!("1. 添加示例数据...");
    let data = vec![
        "用户喜欢科幻电影，尤其是《星际穿越》和《银翼杀手》",
        "用户住在上海，喜欢在周末去咖啡厅",
        "用户是软件工程师，主要使用 Rust 和 Python",
        "用户最近开始学习机器学习",
        "用户喜欢吃辣，最喜欢的菜是麻婆豆腐",
    ];

    for item in &data {
        memory.add(*item).await?;
        println!("   ✓ {}", &item[..30.min(item.len())]);
    }

    println!("\n2. 简单搜索...");
    let results = memory.search("用户的兴趣爱好").await?;
    println!("   查询: \"用户的兴趣爱好\"");
    println!("   结果: {} 条\n", results.len());

    for (i, result) in results.iter().take(3).enumerate() {
        println!("   {}. {} ({:.2})", i + 1, result.content, result.score.unwrap_or(0.0));
    }

    println!("\n3. 带阈值搜索...");
    let results = memory.searchWithOptions(
        SearchOptions::builder()
            .query("编程")
            .threshold(0.7)
            .build()
    ).await?;
    println!("   阈值 > 0.7: {} 条结果", results.len());

    println!("\n4. 带类型过滤搜索...");
    let results = memory.searchWithOptions(
        SearchOptions::builder()
            .query("上海")
            .memory_type(Some(MemoryType::SEMANTIC))
            .build()
    ).await?;
    println!("   SEMANTIC 类型: {} 条结果", results.len());

    println!("\n5. 限制结果数量...");
    let results = memory.searchWithOptions(
        SearchOptions::builder()
            .query("用户")
            .limit(2)
            .build()
    ).await?;
    println!("   最多 2 条: {} 条结果", results.len());

    println!("\n=== 示例完成 ===");

    Ok(())
}
