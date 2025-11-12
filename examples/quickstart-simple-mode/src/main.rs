//! 简单模式示例
//!
//! 演示如何禁用智能功能，直接存储原始内容
//!
//! # 运行方式
//!
//! ```bash
//! # 设置环境变量（任选其一）
//! export OPENAI_API_KEY=sk-...
//! # 或
//! export ANTHROPIC_API_KEY=sk-...
//!
//! # 运行示例
//! cargo run --example quickstart-simple-mode
//! ```
//!
//! # 功能演示
//!
//! 1. **简单模式**: 禁用智能功能（`infer: false`）
//! 2. **直接存储**: 不进行事实提取、去重、冲突解决
//! 3. **快速添加**: 跳过 LLM 调用，直接存储原始内容
//! 4. **适用场景**: 日志记录、原始数据存储、性能敏感场景

use agent_mem::{AddMemoryOptions, Memory};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 AgentMem 简单模式示例\n");

    // ========================================
    // 步骤 1: 初始化
    // ========================================
    println!("📦 步骤 1: 初始化");
    println!("   - 零配置初始化\n");

    let mem = Memory::new().await?;
    println!("✅ 初始化成功！\n");

    // ========================================
    // 步骤 2: 使用简单模式添加记忆
    // ========================================
    println!("📝 步骤 2: 使用简单模式添加记忆");
    println!("   - 禁用智能功能（infer: false）");
    println!("   - 直接存储原始内容");
    println!("   - 跳过事实提取、去重、冲突解决\n");

    // 创建简单模式选项
    let simple_options = AddMemoryOptions {
        infer: false, // 禁用智能功能
        ..Default::default()
    };

    println!("添加记忆 1: 'Raw content without processing'");
    let result1 = mem
        .add_with_options("Raw content without processing".to_string(), simple_options.clone())
        .await?;
    println!("✅ 添加成功: {:?}\n", result1);

    println!("添加记忆 2: 'Another raw message'");
    let result2 = mem
        .add_with_options("Another raw message".to_string(), simple_options.clone())
        .await?;
    println!("✅ 添加成功: {:?}\n", result2);

    println!("添加记忆 3: 'Log entry: User logged in at 10:30 AM'");
    let result3 = mem
        .add_with_options(
            "Log entry: User logged in at 10:30 AM".to_string(),
            simple_options.clone(),
        )
        .await?;
    println!("✅ 添加成功: {:?}\n", result3);

    // ========================================
    // 步骤 3: 对比智能模式
    // ========================================
    println!("🔄 步骤 3: 对比智能模式");
    println!("   - 使用默认选项（infer: true）\n");

    println!("添加记忆 4: 'I love pizza'（智能模式）");
    let result4 = mem.add("I love pizza").await?;
    println!("✅ 添加成功: {:?}\n", result4);

    // ========================================
    // 步骤 4: 搜索记忆
    // ========================================
    println!("🔍 步骤 4: 搜索记忆");

    let query = "raw";
    println!("搜索查询: '{}'", query);

    let results = mem.search(query).await?;
    println!("✅ 搜索成功，找到 {} 条记忆:\n", results.len());

    for (i, result) in results.iter().enumerate() {
        let score_str = result.score.map(|s| format!("{:.2}", s)).unwrap_or_else(|| "N/A".to_string());
        println!("  {}. {} (相关性: {})", i + 1, result.content, score_str);
    }

    // ========================================
    // 步骤 5: 获取所有记忆
    // ========================================
    println!("\n📚 步骤 5: 获取所有记忆");

    use agent_mem::GetAllOptions;
    let all_memories = mem.get_all(GetAllOptions::default()).await?;
    println!("✅ 共有 {} 条记忆:\n", all_memories.len());

    for (i, memory) in all_memories.iter().enumerate() {
        println!("  {}. {}", i + 1, memory.content);
    }

    // ========================================
    // 总结
    // ========================================
    println!("\n🎉 简单模式示例完成！");
    println!("\n📝 简单模式 vs 智能模式:");
    println!("\n简单模式（infer: false）:");
    println!("   ✅ 直接存储原始内容");
    println!("   ✅ 跳过 LLM 调用，性能更快");
    println!("   ✅ 不进行事实提取、去重、冲突解决");
    println!("   ✅ 适用场景：日志记录、原始数据存储、性能敏感场景");

    println!("\n智能模式（infer: true，默认）:");
    println!("   ✅ 自动事实提取");
    println!("   ✅ 智能去重");
    println!("   ✅ 冲突解决");
    println!("   ✅ 适用场景：对话记忆、知识管理、智能助手");

    println!("\n💡 提示:");
    println!("   - 默认使用智能模式（infer: true）");
    println!("   - 如需性能优先，使用简单模式（infer: false）");
    println!("   - 可以混合使用两种模式");

    Ok(())
}

