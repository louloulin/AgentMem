//! AgentOrchestrator 简化单元测试
//!
//! 测试 MemoryIntegrator 的核心功能

use agent_mem_core::{
    orchestrator::memory_integration::{MemoryIntegrator, MemoryIntegratorConfig},
    engine::{MemoryEngine, MemoryEngineConfig},
    Memory, MemoryType,
};
use agent_mem_traits::Session;
use chrono::Utc;
use std::sync::Arc;
use std::collections::HashMap;
use uuid::Uuid;

// 辅助函数：创建测试用的 Memory
fn create_test_memory(content: &str, memory_type: MemoryType, score: Option<f32>) -> Memory {
    Memory {
        id: Uuid::new_v4().to_string(),
        content: content.to_string(),
        hash: None,
        metadata: HashMap::new(),
        score,
        created_at: Utc::now(),
        updated_at: Some(Utc::now()),
        session: Session::new(),
        memory_type,
        entities: vec![],
        relations: vec![],
        agent_id: "test-agent".to_string(),
        user_id: Some("test-user".to_string()),
        importance: score.unwrap_or(0.5), // 使用 score 作为 importance
        embedding: None,
        last_accessed_at: Utc::now(),
        access_count: 0,
        expires_at: None,
        version: 1,
    }
}

#[tokio::test]
async fn test_memory_integrator_inject_memories() {
    // 1. 创建 MemoryEngine 和 MemoryIntegrator
    let memory_engine = Arc::new(MemoryEngine::new(MemoryEngineConfig::default()));
    let config = MemoryIntegratorConfig::default();
    let integrator = MemoryIntegrator::new(memory_engine, config);

    // 2. 创建测试记忆
    let memories = vec![
        create_test_memory("User likes coffee", MemoryType::Semantic, Some(0.9)),
        create_test_memory("User met John yesterday", MemoryType::Episodic, Some(0.8)),
    ];

    // 3. 注入记忆到 prompt
    let formatted = integrator.inject_memories_to_prompt(&memories);

    // 4. 验证格式化结果
    assert!(formatted.contains("Semantic"), "Should contain memory type");
    assert!(formatted.contains("coffee"), "Should contain memory content");
    assert!(formatted.contains("Episodic"), "Should contain memory type");
    assert!(formatted.contains("John"), "Should contain memory content");

    println!("✅ test_memory_integrator_inject_memories passed");
}

#[tokio::test]
async fn test_memory_integrator_filter_by_relevance() {
    // 1. 创建 MemoryIntegrator
    let memory_engine = Arc::new(MemoryEngine::new(MemoryEngineConfig::default()));
    let config = MemoryIntegratorConfig {
        max_memories: 10,
        relevance_threshold: 0.7,
        include_timestamp: true,
        sort_by_importance: true,
    };
    let integrator = MemoryIntegrator::new(memory_engine, config);

    // 2. 创建测试记忆（不同的相关性分数）
    let memories = vec![
        create_test_memory("High relevance", MemoryType::Semantic, Some(0.9)),
        create_test_memory("Medium relevance", MemoryType::Semantic, Some(0.75)),
        create_test_memory("Low relevance", MemoryType::Semantic, Some(0.5)),
    ];

    // 3. 过滤记忆（相关性阈值 0.7）
    let filtered = integrator.filter_by_relevance(memories.clone());

    // 4. 验证过滤结果
    assert_eq!(filtered.len(), 2, "Should filter out low relevance memories");
    assert!(filtered[0].importance >= 0.7, "All filtered memories should have importance >= 0.7");
    assert!(filtered[1].importance >= 0.7, "All filtered memories should have importance >= 0.7");

    println!("✅ test_memory_integrator_filter_by_relevance passed");
}

#[tokio::test]
async fn test_memory_integrator_sort_memories() {
    // 1. 创建 MemoryIntegrator
    let memory_engine = Arc::new(MemoryEngine::new(MemoryEngineConfig::default()));
    let config = MemoryIntegratorConfig::default();
    let integrator = MemoryIntegrator::new(memory_engine, config);

    // 2. 创建测试记忆（不同的分数）
    let memories = vec![
        create_test_memory("Low score", MemoryType::Semantic, Some(0.5)),
        create_test_memory("High score", MemoryType::Semantic, Some(0.9)),
        create_test_memory("Medium score", MemoryType::Semantic, Some(0.7)),
    ];

    // 3. 排序记忆
    let sorted = integrator.sort_memories(memories.clone());

    // 4. 验证排序结果（应该按 importance 降序）
    assert_eq!(sorted.len(), 3, "Should have all memories");
    assert!(sorted[0].importance >= sorted[1].importance, "Should be sorted by importance descending");
    assert!(sorted[1].importance >= sorted[2].importance, "Should be sorted by importance descending");
    assert_eq!(sorted[0].content, "High score", "Highest importance should be first");
    assert_eq!(sorted[2].content, "Low score", "Lowest importance should be last");

    println!("✅ test_memory_integrator_sort_memories passed");
}

#[tokio::test]
async fn test_memory_integrator_empty_memories() {
    // 1. 创建 MemoryIntegrator
    let memory_engine = Arc::new(MemoryEngine::new(MemoryEngineConfig::default()));
    let config = MemoryIntegratorConfig::default();
    let integrator = MemoryIntegrator::new(memory_engine, config);

    // 2. 测试空记忆列表
    let memories: Vec<Memory> = vec![];

    // 3. 注入空记忆
    let formatted = integrator.inject_memories_to_prompt(&memories);

    // 4. 验证结果（应该返回空字符串或默认消息）
    assert!(formatted.is_empty() || formatted.contains("No memories"), "Should handle empty memories gracefully");

    println!("✅ test_memory_integrator_empty_memories passed");
}

#[tokio::test]
async fn test_memory_integrator_no_score() {
    // 1. 创建 MemoryIntegrator
    let memory_engine = Arc::new(MemoryEngine::new(MemoryEngineConfig::default()));
    let config = MemoryIntegratorConfig {
        max_memories: 10,
        relevance_threshold: 0.7,
        include_timestamp: true,
        sort_by_importance: true,
    };
    let integrator = MemoryIntegrator::new(memory_engine, config);

    // 2. 创建没有分数的记忆
    let memories = vec![
        create_test_memory("No score memory", MemoryType::Semantic, None),
        create_test_memory("With score memory", MemoryType::Semantic, Some(0.8)),
    ];

    // 3. 过滤记忆
    let filtered = integrator.filter_by_relevance(memories.clone());

    // 4. 验证结果（importance < 0.7 的记忆应该被过滤掉）
    assert_eq!(filtered.len(), 1, "Should filter out memories with low importance");
    assert!(filtered[0].importance >= 0.7, "Filtered memories should have importance >= 0.7");

    println!("✅ test_memory_integrator_no_score passed");
}

#[tokio::test]
async fn test_memory_integrator_config() {
    // 1. 测试默认配置
    let default_config = MemoryIntegratorConfig::default();
    assert_eq!(default_config.max_memories, 10, "Default max memories should be 10");
    assert_eq!(default_config.relevance_threshold, 0.5, "Default threshold should be 0.5");
    assert!(default_config.include_timestamp, "Default should include timestamp");
    assert!(default_config.sort_by_importance, "Default should sort by importance");

    // 2. 测试自定义配置
    let custom_config = MemoryIntegratorConfig {
        relevance_threshold: 0.8,
        max_memories: 20,
        include_timestamp: false,
        sort_by_importance: false,
    };
    assert_eq!(custom_config.relevance_threshold, 0.8);
    assert_eq!(custom_config.max_memories, 20);
    assert!(!custom_config.include_timestamp);
    assert!(!custom_config.sort_by_importance);

    println!("✅ test_memory_integrator_config passed");
}

#[tokio::test]
async fn test_memory_types() {
    // 测试所有记忆类型
    let memory_types = vec![
        MemoryType::Episodic,
        MemoryType::Semantic,
        MemoryType::Procedural,
        MemoryType::Working,
        MemoryType::Core,
        MemoryType::Resource,
        MemoryType::Knowledge,
        MemoryType::Contextual,
    ];

    for memory_type in memory_types {
        let memory = create_test_memory("Test content", memory_type.clone(), Some(0.8));
        assert_eq!(memory.memory_type, memory_type, "Memory type should match");
    }

    println!("✅ test_memory_types passed");
}

