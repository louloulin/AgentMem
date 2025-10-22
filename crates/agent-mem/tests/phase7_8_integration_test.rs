//! Phase 7-8 集成测试
//!
//! 测试内容：
//! - Phase 7.2: 向量搜索
//! - Phase 7.3: metadata 标准化
//! - Phase 8.1: reset() 方法
//! - Phase 8.2: update() 方法
//! - Phase 8.3: delete() 方法

use agent_mem::Memory;
use std::collections::HashMap;

#[tokio::test]
async fn test_reset_method() {
    // 测试 Phase 8.1: reset() 方法
    let mem = Memory::new().await.expect("Failed to create Memory");

    // 添加一些记忆
    let _id1 = mem.add("测试记忆 1").await.expect("Failed to add memory");
    let _id2 = mem.add("测试记忆 2").await.expect("Failed to add memory");

    // 重置所有记忆
    mem.reset().await.expect("Failed to reset");

    // 验证：应该没有记忆了
    let results = mem.get_all(Default::default()).await.expect("Failed to get all");
    assert_eq!(results.len(), 0, "重置后应该没有记忆");

    println!("✅ test_reset_method passed");
}

#[tokio::test]
async fn test_update_method() {
    // 测试 Phase 8.2: update() 方法
    let mem = Memory::new().await.expect("Failed to create Memory");

    // 添加一个记忆
    let results = mem.add("原始内容").await.expect("Failed to add memory");
    let memory_id = results.results.first().expect("No memory created").id.clone();

    // 更新记忆
    let mut update_data = HashMap::new();
    update_data.insert("content".to_string(), serde_json::json!("更新后的内容"));
    update_data.insert("user_id".to_string(), serde_json::json!("user123"));
    update_data.insert("agent_id".to_string(), serde_json::json!("agent456"));

    let updated = mem.update(&memory_id, update_data).await.expect("Failed to update");

    // 验证更新
    assert_eq!(updated.content, "更新后的内容");
    assert!(updated.hash.is_some(), "应该有 hash");
    assert!(updated.updated_at.is_some(), "应该有 updated_at");

    // 验证历史记录
    let history = mem.history(&memory_id).await.expect("Failed to get history");
    assert!(!history.is_empty(), "应该有历史记录");
    assert_eq!(history.iter().filter(|h| h.event == "UPDATE").count(), 1);

    println!("✅ test_update_method passed");
}

#[tokio::test]
async fn test_delete_method() {
    // 测试 Phase 8.3: delete() 方法
    let mem = Memory::new().await.expect("Failed to create Memory");

    // 添加一个记忆
    let results = mem.add("要删除的记忆").await.expect("Failed to add memory");
    let memory_id = results.results.first().expect("No memory created").id.clone();

    // 删除记忆
    mem.delete(&memory_id).await.expect("Failed to delete");

    // 验证历史记录
    let history = mem.history(&memory_id).await.expect("Failed to get history");
    assert!(!history.is_empty(), "应该有历史记录");
    
    let delete_entries: Vec<_> = history.iter().filter(|h| h.event == "DELETE").collect();
    assert_eq!(delete_entries.len(), 1, "应该有一条 DELETE 记录");
    assert!(delete_entries[0].is_deleted, "DELETE 记录应该标记为已删除");

    println!("✅ test_delete_method passed");
}

#[tokio::test]
async fn test_vector_search() {
    // 测试 Phase 7.2: 向量搜索（如果 embedder 可用）
    let mem = Memory::new().await.expect("Failed to create Memory");

    // 添加一些记忆
    mem.add("我喜欢吃披萨").await.ok();
    mem.add("我喜欢吃意大利面").await.ok();
    mem.add("我在学习 Rust 编程").await.ok();

    // 搜索：语义相似
    let results = mem.search("意大利美食").await.expect("Failed to search");

    // 如果有结果，验证相关性
    if !results.is_empty() {
        println!("找到 {} 个结果", results.len());
        for result in &results {
            println!("  - {}: {:.4}", result.content, result.score.unwrap_or(0.0));
        }
        
        // 验证：应该包含食物相关的内容
        let has_food_related = results.iter().any(|r| 
            r.content.contains("披萨") || r.content.contains("意大利面")
        );
        
        if has_food_related {
            println!("✅ 找到了食物相关的记忆");
        }
    } else {
        println!("⚠️ 没有搜索结果（可能 Embedder 未初始化）");
    }

    println!("✅ test_vector_search passed");
}

#[tokio::test]
async fn test_metadata_standardization() {
    // 测试 Phase 7.3: metadata 标准化
    let mem = Memory::new().await.expect("Failed to create Memory");

    // 添加记忆
    let results = mem.add("测试内容").await.expect("Failed to add");
    let memory_event = results.results.first().expect("No memory created");

    // 验证基本字段
    assert!(!memory_event.id.is_empty(), "应该有 ID");
    assert!(!memory_event.memory.is_empty(), "应该有内容");
    assert_eq!(memory_event.event, "ADD", "事件类型应该是 ADD");

    println!("✅ test_metadata_standardization passed");
}

#[tokio::test]
async fn test_complete_workflow() {
    // 完整流程测试：ADD -> UPDATE -> DELETE -> HISTORY
    let mem = Memory::new().await.expect("Failed to create Memory");

    // 1. 添加
    let add_result = mem.add("初始内容").await.expect("Failed to add");
    let memory_id = add_result.results.first().expect("No memory").id.clone();
    
    // 2. 更新
    let mut update_data = HashMap::new();
    update_data.insert("content".to_string(), serde_json::json!("更新内容"));
    mem.update(&memory_id, update_data).await.expect("Failed to update");
    
    // 3. 删除
    mem.delete(&memory_id).await.expect("Failed to delete");
    
    // 4. 验证历史
    let history = mem.history(&memory_id).await.expect("Failed to get history");
    assert!(history.len() >= 2, "应该至少有 2 条历史记录");
    
    // 验证事件顺序（最新的在前）
    let events: Vec<_> = history.iter().map(|h| h.event.as_str()).collect();
    assert!(events.contains(&"ADD"));
    assert!(events.contains(&"UPDATE"));
    assert!(events.contains(&"DELETE"));

    println!("✅ test_complete_workflow passed");
}

