//! 搜索失败分析演示
//!
//! 本示例演示为什么某些搜索查询返回 0 结果，以及如何改进

#[path = "shared/simple_memory_adapter.rs"]
mod simple_memory_adapter;
use agent_mem_traits::Result;
use simple_memory_adapter::SimpleMemory;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 AgentMem 搜索失败分析演示");
    println!("{}", "=".repeat(70));

    // 1. 创建 SimpleMemory
    println!("\n📦 1. 初始化 SimpleMemory...");
    let memory = SimpleMemory::new().await?;
    println!("   ✅ SimpleMemory 创建成功");

    // 2. 添加测试数据
    println!("\n📝 2. 添加测试数据...");

    let test_data = vec![
        (
            "[struct] SimpleMemory in simple_memory.rs",
            "Simplified Memory API (Mem0-style)",
        ),
        (
            "[struct] HierarchicalMemoryManager in hierarchy.rs",
            "Manages hierarchical memory structure",
        ),
        (
            "[function] add_memory in manager.rs",
            "Add a new memory with intelligent processing",
        ),
        (
            "[function] search_memories in manager.rs",
            "Search memories using text or vector query",
        ),
    ];

    for (i, (content, description)) in test_data.iter().enumerate() {
        let mut metadata = HashMap::new();
        metadata.insert("description".to_string(), description.to_string());
        metadata.insert("index".to_string(), i.to_string());

        memory.add_with_metadata(*content, Some(metadata)).await?;
        println!("   [{}] 添加: {}", i + 1, content);
    }

    println!("\n   ✅ 添加了 {} 条测试数据", test_data.len());

    // 3. 搜索测试
    println!("\n🔍 3. 搜索测试...");
    println!("{}", "-".repeat(70));

    let test_queries = vec![
        ("SimpleMemory", "单词匹配 - 应该找到"),
        ("SimpleMemory 实现", "多词查询 - 可能找不到"),
        ("MemoryManager", "部分匹配 - 应该找到"),
        ("memory", "通用词 - 应该找到多个"),
        ("实现", "中文词 - 可能找不到"),
        ("search", "英文词 - 应该找到"),
    ];

    for (i, (query, description)) in test_queries.iter().enumerate() {
        println!("\n   查询 {}: \"{}\"", i + 1, query);
        println!("   描述: {}", description);

        let results = memory.search(*query).await?;

        println!("   📊 找到 {} 条结果", results.len());

        if !results.is_empty() {
            println!("   🎯 结果:");
            for (j, result) in results.iter().take(3).enumerate() {
                let first_line = result.content.lines().next().unwrap_or("Unknown");
                println!("      {}. {}", j + 1, first_line);
            }
        } else {
            println!("   ℹ️  未找到相关结果");
        }

        // 分析为什么找到或找不到
        analyze_search_result(query, &results, &test_data);
    }

    // 4. 总结
    println!("\n{}", "=".repeat(70));
    println!("📊 搜索行为分析总结");
    println!("\n✅ 能找到的情况:");
    println!("   1. 查询词是内容的子串 (如 'SimpleMemory')");
    println!("   2. 查询词在内容中完整出现 (如 'memory')");
    println!("   3. 查询词是较长词的一部分 (如 'Manager' 在 'MemoryManager')");

    println!("\n❌ 找不到的情况:");
    println!("   1. 多词查询，整体不在内容中 (如 'SimpleMemory 实现')");
    println!("   2. 查询词不在索引内容中 (如 '实现')");
    println!("   3. 同义词查询 (如 'implementation' vs '实现')");

    println!("\n💡 当前搜索算法:");
    println!("   - 使用简单的字符串包含匹配 (contains)");
    println!("   - 不支持语义搜索");
    println!("   - 不支持向量嵌入");
    println!("   - 不支持同义词");

    println!("\n🚀 改进建议:");
    println!("   1. 短期: 使用单词级别匹配");
    println!("   2. 中期: 集成向量嵌入模型");
    println!("   3. 长期: 实现混合搜索 (文本 + 向量)");

    Ok(())
}

/// 分析搜索结果
fn analyze_search_result(
    query: &str,
    results: &[agent_mem_traits::MemoryItem],
    test_data: &[(&str, &str)],
) {
    let query_lower = query.to_lowercase();

    println!("\n   🔬 分析:");

    if results.is_empty() {
        println!("      原因: 查询字符串 '{}' 不是任何内容的子串", query);

        // 检查是否有部分匹配
        let words: Vec<&str> = query_lower.split_whitespace().collect();
        if words.len() > 1 {
            println!("      查询包含 {} 个词: {:?}", words.len(), words);

            for (content, _) in test_data {
                let content_lower = content.to_lowercase();
                let matched_words: Vec<&&str> = words
                    .iter()
                    .filter(|word| content_lower.contains(*word))
                    .collect();

                if !matched_words.is_empty() {
                    println!(
                        "      在 '{}' 中找到部分匹配: {:?}",
                        content.lines().next().unwrap_or(content),
                        matched_words
                    );
                }
            }
        }
    } else {
        println!("      原因: 查询字符串 '{}' 是以下内容的子串:", query);
        for result in results.iter().take(3) {
            let first_line = result.content.lines().next().unwrap_or("Unknown");
            println!("         - {}", first_line);
        }
    }
}
