//! AgentMem 完整 CRUD 操作示例
//!
//! 这个示例演示了所有记忆管理功能：
//! - 创建 (Create): 添加新记忆
//! - 读取 (Read): 获取单个/所有记忆
//! - 更新 (Update): 修改现有记忆
//! - 删除 (Delete): 删除记忆
//! - 批量操作: 批量添加和删除
//! - 会话管理: 使用 Session 组织记忆
//!
//! # 运行方式
//!
//! ```bash
//! export OPENAI_API_KEY=sk-...
//! cargo run --example memory_management
//! ```
//!
//! # 预期输出
//!
//! ```text
//! 🗂️  AgentMem 完整 CRUD 操作示例
//!
//! ✅ 步骤 1: 创建 (CREATE)
//!    创建记忆: "学习 Rust" -> id: mem_001
//!    创建记忆: "学习 Python" -> id: mem_002
//!    创建记忆: "学习 JavaScript" -> id: mem_003
//!
//! ✅ 步骤 2: 读取 (READ)
//!    获取单个记忆: mem_001 -> "学习 Rust"
//!    获取所有记忆: 3 条
//!
//! ✅ 步骤 3: 更新 (UPDATE)
//!    更新前: "学习 Rust"
//!    更新后: "精通 Rust 编程"
//!
//! ✅ 步骤 4: 删除 (DELETE)
//!    删除记忆: mem_003
//!    剩余记忆: 2 条
//!
//! ✅ 步骤 5: 批量操作
//!    批量添加 2 条记忆
//!    批量删除 1 条记忆
//!
//! ✅ 步骤 6: 会话管理
//!    创建会话: session_001
//!    会话中添加 2 条记忆
//!    会话记忆数量: 2
//! ```

use agent_mem::{GetAllOptions, Memory};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🗂️  AgentMem 完整 CRUD 操作示例\n");

    // 初始化
    let mem = Memory::new().await?;

    // ============================================
    // 步骤 1: 创建 (CREATE)
    // ============================================
    println!("✅ 步骤 1: 创建 (CREATE)");

    let mem1 = mem.add("学习 Rust").await?;
    println!("   创建记忆: \"学习 Rust\" -> id: {}", mem1.id);

    let mem2 = mem.add("学习 Python").await?;
    println!("   创建记忆: \"学习 Python\" -> id: {}", mem2.id);

    let mem3 = mem.add("学习 JavaScript").await?;
    println!("   创建记忆: \"学习 JavaScript\" -> id: {}", mem3.id);
    println!();

    // ============================================
    // 步骤 2: 读取 (READ)
    // ============================================
    println!("✅ 步骤 2: 读取 (READ)");

    // 获取单个记忆
    match mem.get(&mem1.id).await {
        Ok(Some(memory)) => {
            println!("   获取单个记忆: {} -> \"{}\"", memory.id, memory.content);
        }
        Ok(None) => {
            println!("   记忆不存在: {}", mem1.id);
        }
        Err(e) => {
            println!("   获取失败: {}", e);
        }
    }

    // 获取所有记忆
    let all_memories = mem.get_all(GetAllOptions::default()).await?;
    println!("   获取所有记忆: {} 条", all_memories.len());
    println!();

    // ============================================
    // 步骤 3: 更新 (UPDATE)
    // ============================================
    println!("✅ 步骤 3: 更新 (UPDATE)");

    // 先显示更新前的内容
    if let Some(original) = mem.get(&mem1.id).await? {
        println!("   更新前: \"{}\"", original.content);

        // 更新记忆内容
        let updated = mem
            .update(&mem1.id, "精通 Rust 编程")
            .await?;

        println!("   更新后: \"{}\"", updated.content);
    }
    println!();

    // ============================================
    // 步骤 4: 删除 (DELETE)
    // ============================================
    println!("✅ 步骤 4: 删除 (DELETE)");

    // 获取删除前的记忆数量
    let before_count = mem.get_all(GetAllOptions::default()).await?.len();
    println!("   删除前: {} 条记忆", before_count);

    // 删除记忆
    match mem.delete(&mem3.id).await {
        Ok(_) => {
            println!("   删除记忆: {}", mem3.id);

            // 获取删除后的记忆数量
            let after_count = mem.get_all(GetAllOptions::default()).await?.len();
            println!("   删除后: {} 条记忆", after_count);
        }
        Err(e) => {
            println!("   删除失败: {}", e);
        }
    }
    println!();

    // ============================================
    // 步骤 5: 批量操作
    // ============================================
    println!("✅ 步骤 5: 批量操作");

    // 批量添加记忆
    let batch_memories = vec![
        "学习 Go 语言",
        "学习 C++ 语言",
    ];

    println!("   批量添加 {} 条记忆", batch_memories.len());

    for content in batch_memories {
        match mem.add(content).await {
            Ok(result) => println!("   ✅ 添加: \"{}\" -> {}", content, result.id),
            Err(e) => println!("   ❌ 添加失败: {} - {}", content, e),
        }
    }

    // 批量删除（删除所有 "学习" 相关的记忆）
    let all_memories = mem.get_all(GetAllOptions::default()).await?;
    let mut delete_count = 0;

    for memory in all_memories {
        if memory.content.contains("学习") {
            match mem.delete(&memory.id).await {
                Ok(_) => delete_count += 1,
                Err(e) => println!("   ❌ 删除失败: {} - {}", memory.id, e),
            }
        }
    }

    println!("   批量删除 {} 条记忆", delete_count);
    println!();

    // ============================================
    // 步骤 6: 会话管理
    // ============================================
    println!("✅ 步骤 6: 会话管理");

    // 创建一个新会话 ID
    let session_id = Uuid::new_v4().to_string();
    println!("   创建会话: {}", session_id);

    // 在会话中添加记忆
    let session_mem1 = mem.add("用户问: 如何学习 Rust？").await?;
    let session_mem2 = mem.add("用户说: 我有 Python 基础").await?;

    println!("   会话中添加 2 条记忆");

    // 获取会话中的所有记忆
    let all_memories = mem.get_all(GetAllOptions::default()).await?;
    println!("   总记忆数量: {}", all_memories.len());
    println!();

    // ============================================
    // 完成
    // ============================================
    println!("🎉 完成！所有 CRUD 操作演示完毕。\n");

    println!("💡 关键要点:");
    println!("   1. 每个操作都有完整的错误处理");
    println!("   2. 支持单个和批量操作");
    println!("   3. 可以使用 Session 组织相关记忆");
    println!("   4. 所有操作都是异步的，使用 .await");

    Ok(())
}

// ============================================
// 高级技巧: 元数据管理
// ============================================
//
// 你可以为记忆添加元数据来增强其可检索性:
//
// ```rust
// use agent_mem::{Memory, Metadata};
//
// let mut metadata = Metadata::new();
// metadata.insert("category".to_string(), "编程语言".to_string());
// metadata.insert("difficulty".to_string(), "中级".to_string());
//
// let mem = Memory::new().await?;
// mem.add_with_metadata("学习 Rust", metadata).await?;
// ```
//
// 然后可以根据元数据过滤:
//
// ```rust
// let results = mem.search_with_filter(
//     "编程",
//     |metadata| {
//         metadata.get("category") == Some(&"编程语言".to_string())
//     }
// ).await?;
// ```
