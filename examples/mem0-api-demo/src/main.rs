//! # mem0 API 兼容性演示
//!
//! 演示 AgentMem 的 mem0 兼容 API，包括：
//! - add() - 添加记忆
//! - search() - 搜索记忆
//! - get() - 获取单个记忆
//! - get_all() - 获取所有记忆
//! - update() - 更新记忆
//! - delete() - 删除记忆
//! - delete_all() - 删除所有记忆

use agent_mem::{AddMemoryOptions, DeleteAllOptions, GetAllOptions, Memory, SearchOptions};
use anyhow::Result;
use colored::*;
use std::collections::HashMap;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt().with_env_filter("info").init();

    println!("{}", "🧠 mem0 API 兼容性演示".bright_blue().bold());
    println!("{}", "=".repeat(60).bright_blue());
    println!();

    // 1. 初始化 Memory（零配置）
    println!("{}", "1️⃣  初始化 Memory".bright_green().bold());
    let mem = Memory::new().await?;
    println!("   ✓ Memory 初始化成功");
    println!();

    // 2. 添加记忆（基础模式）
    println!("{}", "2️⃣  添加记忆（基础模式）".bright_green().bold());
    let result1 = mem.add("I love pizza").await?;
    println!("   ✓ 添加成功: {} 个记忆事件", result1.results.len());
    for event in &result1.results {
        println!("     - ID: {}", event.id);
        println!("     - 内容: {}", event.memory);
        println!("     - 事件: {}", event.event);
    }
    println!();

    // 3. 添加记忆（带选项）
    println!("{}", "3️⃣  添加记忆（带选项）".bright_green().bold());
    let mut metadata = HashMap::new();
    metadata.insert("source".to_string(), "demo".to_string());
    metadata.insert("importance".to_string(), "0.8".to_string());

    let options = AddMemoryOptions {
        user_id: Some("alice".to_string()),
        agent_id: Some("assistant-1".to_string()),
        infer: true, // 启用智能推理
        memory_type: Some("semantic_memory".to_string()),
        metadata,
        ..Default::default()
    };
    let result2 = mem
        .add_with_options("I prefer morning meetings", options)
        .await?;
    println!("   ✓ 添加成功: {} 个记忆事件", result2.results.len());
    for event in &result2.results {
        println!("     - ID: {}", event.id);
        println!("     - 内容: {}", event.memory);
        println!("     - 事件: {}", event.event);
        if let Some(actor) = &event.actor_id {
            println!("     - Actor: {}", actor);
        }
    }
    println!();

    // 4. 搜索记忆
    println!("{}", "4️⃣  搜索记忆".bright_green().bold());
    let search_options = SearchOptions {
        user_id: Some("alice".to_string()),
        limit: Some(10),
        threshold: Some(0.5),
        ..Default::default()
    };
    let search_results = mem
        .search_with_options("What do you know about me?", search_options)
        .await?;
    println!("   ✓ 找到 {} 条记忆", search_results.len());
    for (i, item) in search_results.iter().enumerate() {
        println!("     {}. {}", i + 1, item.content);
        println!("        重要性: {:.2}", item.importance);
    }
    println!();

    // 5. 获取所有记忆
    println!("{}", "5️⃣  获取所有记忆".bright_green().bold());
    let get_all_options = GetAllOptions {
        user_id: Some("alice".to_string()),
        limit: Some(100),
        ..Default::default()
    };
    let all_memories = mem.get_all(get_all_options).await?;
    println!("   ✓ 总共 {} 条记忆", all_memories.len());
    println!();

    // 6. 获取单个记忆（演示错误处理）
    println!("{}", "6️⃣  获取单个记忆".bright_green().bold());
    match mem.get("non-existent-id").await {
        Ok(memory) => {
            println!("   ✓ 获取成功: {}", memory.content);
        }
        Err(e) => {
            println!("   ⚠️  预期的错误: {}", e.to_string().yellow());
        }
    }
    println!();

    // 7. 更新记忆（演示错误处理）
    println!("{}", "7️⃣  更新记忆".bright_green().bold());
    let mut update_data = HashMap::new();
    update_data.insert(
        "content".to_string(),
        serde_json::json!("I love pizza and pasta"),
    );
    match mem.update("non-existent-id", update_data).await {
        Ok(updated) => {
            println!("   ✓ 更新成功: {}", updated.content);
        }
        Err(e) => {
            println!("   ⚠️  预期的错误: {}", e.to_string().yellow());
        }
    }
    println!();

    // 8. 删除记忆（演示错误处理）
    println!("{}", "8️⃣  删除记忆".bright_green().bold());
    match mem.delete("non-existent-id").await {
        Ok(_) => {
            println!("   ✓ 删除成功");
        }
        Err(e) => {
            println!("   ⚠️  预期的错误: {}", e.to_string().yellow());
        }
    }
    println!();

    // 9. 删除所有记忆
    println!("{}", "9️⃣  删除所有记忆".bright_green().bold());
    let delete_all_options = DeleteAllOptions {
        user_id: Some("alice".to_string()),
        ..Default::default()
    };
    let deleted_count = mem.delete_all(delete_all_options).await?;
    println!("   ✓ 删除了 {} 条记忆", deleted_count);
    println!();

    // 总结
    println!("{}", "✅ 演示完成！".bright_green().bold());
    println!();
    println!("{}", "📊 mem0 API 兼容性总结:".bright_blue().bold());
    println!("   ✓ add() - 添加记忆（支持 infer 参数）");
    println!("   ✓ search() - 搜索记忆（支持过滤和阈值）");
    println!("   ✓ get() - 获取单个记忆");
    println!("   ✓ get_all() - 获取所有记忆（支持过滤和限制）");
    println!("   ✓ update() - 更新记忆");
    println!("   ✓ delete() - 删除记忆");
    println!("   ✓ delete_all() - 删除所有记忆");
    println!();
    println!("{}", "🎯 下一步:".bright_yellow().bold());
    println!("   1. 实现 orchestrator 中的 TODO 方法");
    println!("   2. 集成 core 模块的 Agents");
    println!("   3. 添加智能推理功能（事实提取、去重）");
    println!("   4. 添加关系提取（图存储）");
    println!();

    Ok(())
}
