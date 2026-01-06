use agent_mem_core::hierarchy::{DefaultHierarchyManager, HierarchyConfig, HierarchyManager};
use agent_mem_core::types::{Memory, MemoryType, MemoryLevel, MemoryScope};
use agent_mem_traits::Session;
use chrono::Utc;
use std::collections::HashMap;

/// 创建测试用的Memory实例
fn create_test_memory(id: &str, content: &str, score: f32) -> Memory {
    Memory {
        id: id.to_string(),
        content: content.to_string(),
        hash: None,
        metadata: HashMap::new(),
        score: Some(score),
        created_at: Utc::now(),
        updated_at: Some(Utc::now()),
        session: Session::new(),
        memory_type: MemoryType::Episodic,
        entities: Vec::new(),
        relations: Vec::new(),
        agent_id: "test_agent".to_string(),
        user_id: Some("test_user".to_string()),
        importance: score as f64,
        embedding: None,
        last_accessed_at: Utc::now(),
        access_count: 0,
        expires_at: None,
        version: 1,
    }
}

#[tokio::test]
async fn test_comprehensive_hierarchy_functionality() {
    let config = HierarchyConfig::default();
    let manager = DefaultHierarchyManager::new(config);

    // 测试不同重要性级别的内存
    let strategic_memory = create_test_memory("strategic_1", "Strategic decision about company direction", 0.9);
    let tactical_memory = create_test_memory("tactical_1", "Tactical plan for Q1", 0.7);
    let operational_memory = create_test_memory("operational_1", "Daily task completion", 0.5);
    let contextual_memory = create_test_memory("contextual_1", "Random conversation note", 0.2);

    // 添加内存到层次结构
    let strategic_hier = manager.add_memory(strategic_memory).await.unwrap();
    let tactical_hier = manager.add_memory(tactical_memory).await.unwrap();
    let operational_hier = manager.add_memory(operational_memory).await.unwrap();
    let contextual_hier = manager.add_memory(contextual_memory).await.unwrap();

    // 验证层次分配
    assert_eq!(strategic_hier.level, MemoryLevel::Strategic);
    assert_eq!(tactical_hier.level, MemoryLevel::Tactical);
    assert_eq!(operational_hier.level, MemoryLevel::Operational);
    assert_eq!(contextual_hier.level, MemoryLevel::Contextual);

    // 测试按层次获取内存
    let strategic_memories = manager.get_memories_at_level(MemoryLevel::Strategic).await.unwrap();
    let tactical_memories = manager.get_memories_at_level(MemoryLevel::Tactical).await.unwrap();
    let operational_memories = manager.get_memories_at_level(MemoryLevel::Operational).await.unwrap();
    let contextual_memories = manager.get_memories_at_level(MemoryLevel::Contextual).await.unwrap();

    assert_eq!(strategic_memories.len(), 1);
    assert_eq!(tactical_memories.len(), 1);
    assert_eq!(operational_memories.len(), 1);
    assert_eq!(contextual_memories.len(), 1);

    // 测试搜索功能
    let search_results = manager.search_memories("decision", None, None).await.unwrap();
    assert_eq!(search_results.len(), 1);
    assert_eq!(search_results[0].memory.id, "strategic_1");

    let plan_results = manager.search_memories("plan", None, None).await.unwrap();
    assert_eq!(plan_results.len(), 1);
    assert_eq!(plan_results[0].memory.id, "tactical_1");

    // 测试搜索限制
    let limited_results = manager.search_memories("task", None, Some(1)).await.unwrap();
    assert!(limited_results.len() <= 1);

    // 测试层次统计
    let stats = manager.get_hierarchy_stats().await.unwrap();
    assert_eq!(stats.memories_by_level.get(&MemoryLevel::Strategic), Some(&1));
    assert_eq!(stats.memories_by_level.get(&MemoryLevel::Tactical), Some(&1));
    assert_eq!(stats.memories_by_level.get(&MemoryLevel::Operational), Some(&1));
    assert_eq!(stats.memories_by_level.get(&MemoryLevel::Contextual), Some(&1));

    // 验证平均重要性计算
    assert!(stats.avg_importance_by_level.get(&MemoryLevel::Strategic).unwrap() > &0.8);
    assert!(stats.avg_importance_by_level.get(&MemoryLevel::Tactical).unwrap() > &0.6);
    assert!(stats.avg_importance_by_level.get(&MemoryLevel::Operational).unwrap() > &0.4);
    assert!(stats.avg_importance_by_level.get(&MemoryLevel::Contextual).unwrap() < &0.4);

    // 测试内存更新
    let mut updated_strategic = strategic_hier.clone();
    updated_strategic.memory.content = "Updated strategic decision".to_string();
    let updated_result = manager.update_memory(updated_strategic).await.unwrap();
    assert_eq!(updated_result.memory.content, "Updated strategic decision");

    // 验证更新后的内存可以检索
    let retrieved = manager.get_memory("strategic_1").await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().memory.content, "Updated strategic decision");

    // 测试内存删除
    let removed = manager.remove_memory("contextual_1").await.unwrap();
    assert!(removed);

    // 验证删除后的内存不存在
    let deleted_memory = manager.get_memory("contextual_1").await.unwrap();
    assert!(deleted_memory.is_none());

    // 验证删除后的统计更新
    let updated_stats = manager.get_hierarchy_stats().await.unwrap();
    assert_eq!(updated_stats.memories_by_level.get(&MemoryLevel::Contextual), Some(&0));

    println!("✅ 所有层次化内存管理功能测试通过！");
}

#[tokio::test]
async fn test_concurrent_access() {
    use tokio::task;
    
    let config = HierarchyConfig::default();
    let manager = std::sync::Arc::new(DefaultHierarchyManager::new(config));

    // 并发添加内存
    let mut handles = Vec::new();
    for i in 0..10 {
        let manager_clone = manager.clone();
        let handle = task::spawn(async move {
            let memory = create_test_memory(
                &format!("concurrent_{}", i),
                &format!("Concurrent memory {}", i),
                0.5 + (i as f32 * 0.05),
            );
            manager_clone.add_memory(memory).await
        });
        handles.push(handle);
    }

    // 等待所有任务完成
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }

    // 验证所有内存都被正确添加
    let stats = manager.get_hierarchy_stats().await.unwrap();
    let total_memories: usize = stats.memories_by_level.values().sum();
    assert_eq!(total_memories, 10);

    println!("✅ 并发访问测试通过！");
}

#[tokio::test]
async fn test_scope_filtering() {
    let config = HierarchyConfig::default();
    let manager = DefaultHierarchyManager::new(config);

    // 创建不同作用域的内存
    let mut global_memory = create_test_memory("global_1", "Global knowledge", 0.8);
    global_memory.metadata.insert("scope".to_string(), serde_json::json!("Global"));

    let mut agent_memory = create_test_memory("agent_1", "Agent specific knowledge", 0.7);
    agent_memory.metadata.insert("scope".to_string(), serde_json::json!({"Agent": "test_agent"}));

    manager.add_memory(global_memory).await.unwrap();
    manager.add_memory(agent_memory).await.unwrap();

    // 测试全局搜索
    let all_results = manager.search_memories("knowledge", None, None).await.unwrap();
    assert_eq!(all_results.len(), 2);

    // 测试作用域过滤搜索
    let global_results = manager.search_memories("knowledge", Some(MemoryScope::Global), None).await.unwrap();
    assert_eq!(global_results.len(), 1);
    assert_eq!(global_results[0].memory.id, "global_1");

    println!("✅ 作用域过滤测试通过！");
}
