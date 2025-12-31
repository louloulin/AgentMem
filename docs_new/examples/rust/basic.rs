// AgentMem 基础示例
//
// 演示最基本的使用方法

use agentmem::{Memory, MemoryType, GetAllOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== AgentMem 基础示例 ===\n");

    // 1. 创建实例
    println!("1. 创建 Memory 实例...");
    let memory = Memory::new().await?;
    println!("   ✓ 初始化成功\n");

    // 2. 添加记忆
    println!("2. 添加记忆...");
    let result1 = memory.add("用户喜欢咖啡").await?;
    println!("   ✓ 添加: \"{}\"", result1.id);

    let result2 = memory.add("用户住在上海").await?;
    println!("   ✓ 添加: \"{}\"", result2.id);

    let result3 = memory.add("用户是软件工程师").await?;
    println!("   ✓ 添加: \"{}\"\n", result3.id);

    // 3. 搜索记忆
    println!("3. 搜索记忆...");
    let query = "用户的爱好和职业";
    let results = memory.search(query).await?;

    println!("   查询: \"{}\"", query);
    println!("   找到 {} 条结果:\n", results.len());

    for (i, result) in results.iter().enumerate() {
        let score = result.score.unwrap_or(0.0);
        println!("   {}. {} (相似度: {:.2})", i + 1, result.content, score);
    }

    // 4. 获取所有记忆
    println!("\n4. 获取所有记忆...");
    let all_memories = memory.get_all(GetAllOptions::default()).await?;

    println!("   总共 {} 条记忆:\n", all_memories.len());
    for (i, mem) in all_memories.iter().enumerate() {
        println!("   {}. {}", i + 1, mem.content);
    }

    // 5. 清理
    println!("\n5. 清理...");
    memory.delete_all(Default::default()).await?;
    println!("   ✓ 已清理所有记忆");

    println!("\n=== 示例完成 ===");

    Ok(())
}
