//! AgentOrchestrator 单元测试
//!
//! 测试 MemoryIntegrator, MemoryExtractor, ToolIntegrator 等模块的单元功能

use agent_mem_core::{
    engine::{MemoryEngine, MemoryEngineConfig},
    orchestrator::memory_integration::{MemoryIntegrator, MemoryIntegratorConfig},
    Memory, MemoryType,
};
use agent_mem_traits::Session;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

// 辅助函数：创建测试用的 Memory
fn create_test_memory(content: &str, memory_type: MemoryType, score: Option<f32>) -> Memory {
    // importance 和 score 保持一致
    let importance = score.unwrap_or(0.5);
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
        importance,
        embedding: None,
        last_accessed_at: Utc::now(),
        access_count: 0,
        expires_at: None,
        version: 1,
    }
}

#[tokio::test]
async fn test_memory_integrator_format_memories() {
    // 1. 创建 MemoryEngine 和 MemoryIntegrator
    let memory_engine = Arc::new(MemoryEngine::new(MemoryEngineConfig::default()));
    let config = MemoryIntegratorConfig::default();
    let integrator = MemoryIntegrator::new(memory_engine, config);

    // 2. 创建测试记忆
    let memories = vec![
        create_test_memory("User likes coffee", MemoryType::Semantic, Some(0.9)),
        create_test_memory("User met John yesterday", MemoryType::Episodic, Some(0.8)),
    ];

    // 3. 格式化记忆
    let formatted = integrator.inject_memories_to_prompt(&memories);

    // 4. 验证格式化结果
    assert!(formatted.contains("Semantic"), "Should contain memory type");
    assert!(
        formatted.contains("coffee"),
        "Should contain memory content"
    );
    assert!(formatted.contains("Episodic"), "Should contain memory type");
    assert!(formatted.contains("John"), "Should contain memory content");

    println!("✅ test_memory_integrator_format_memories passed");
}

#[tokio::test]
async fn test_memory_integrator_filter_by_relevance() {
    // 1. 创建 MemoryEngine 和 MemoryIntegrator
    let memory_engine = Arc::new(MemoryEngine::new(MemoryEngineConfig::default()));
    let config = MemoryIntegratorConfig {
        relevance_threshold: 0.7,
        ..Default::default()
    };
    let integrator = MemoryIntegrator::new(memory_engine, config);

    // 2. 创建测试记忆（不同的相关性分数）
    let memories = vec![
        create_test_memory("High relevance", MemoryType::Semantic, Some(0.9)),
        create_test_memory("Low relevance", MemoryType::Semantic, Some(0.5)),
        create_test_memory("Medium relevance", MemoryType::Semantic, Some(0.75)),
    ];

    // 3. 过滤记忆
    let filtered = integrator.filter_by_relevance(memories);

    // 4. 验证过滤结果（只保留 score >= 0.7 的记忆）
    assert_eq!(
        filtered.len(),
        2,
        "Should keep 2 memories with score >= 0.7"
    );
    assert!(
        filtered.iter().all(|m| m.score.unwrap_or(0.0) >= 0.7),
        "All filtered memories should have score >= 0.7"
    );

    println!("✅ test_memory_integrator_filter_by_relevance passed");
}

#[tokio::test]
async fn test_memory_integrator_sort_memories() {
    // 1. 创建 MemoryEngine 和 MemoryIntegrator
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
    let sorted = integrator.sort_memories(memories);

    // 4. 验证排序结果（按分数降序）
    assert_eq!(sorted.len(), 3, "Should have 3 memories");
    assert_eq!(
        sorted[0].content, "High score",
        "First should be highest score"
    );
    assert_eq!(
        sorted[1].content, "Medium score",
        "Second should be medium score"
    );
    assert_eq!(
        sorted[2].content, "Low score",
        "Third should be lowest score"
    );

    println!("✅ test_memory_integrator_sort_memories passed");
}

#[tokio::test]
async fn test_memory_integrator_empty_memories() {
    // 1. 创建 MemoryEngine 和 MemoryIntegrator
    let memory_engine = Arc::new(MemoryEngine::new(MemoryEngineConfig::default()));
    let config = MemoryIntegratorConfig::default();
    let integrator = MemoryIntegrator::new(memory_engine, config);

    // 2. 测试空记忆列表
    let memories: Vec<Memory> = vec![];

    // 3. 格式化空记忆
    let formatted = integrator.inject_memories_to_prompt(&memories);

    // 4. 验证结果
    assert!(
        formatted.is_empty() || formatted.contains("No memories"),
        "Should handle empty memories"
    );

    // 5. 过滤空记忆
    let filtered = integrator.filter_by_relevance(memories.clone());
    assert_eq!(filtered.len(), 0, "Should return empty list");

    // 6. 排序空记忆
    let sorted = integrator.sort_memories(memories);
    assert_eq!(sorted.len(), 0, "Should return empty list");

    println!("✅ test_memory_integrator_empty_memories passed");
}

#[tokio::test]
async fn test_memory_integrator_no_score() {
    // 1. 创建 MemoryEngine 和 MemoryIntegrator
    let memory_engine = Arc::new(MemoryEngine::new(MemoryEngineConfig::default()));
    let config = MemoryIntegratorConfig {
        relevance_threshold: 0.7,
        ..Default::default()
    };
    let integrator = MemoryIntegrator::new(memory_engine, config);

    // 2. 创建没有分数的记忆
    let memories = vec![create_test_memory(
        "No score memory",
        MemoryType::Semantic,
        None,
    )];

    // 3. 过滤记忆（没有分数的记忆应该被过滤掉）
    let filtered = integrator.filter_by_relevance(memories);

    // 4. 验证结果
    assert_eq!(
        filtered.len(),
        0,
        "Memories without score should be filtered out"
    );

    println!("✅ test_memory_integrator_no_score passed");
}

#[tokio::test]
async fn test_memory_integrator_config() {
    // 1. 测试默认配置
    let default_config = MemoryIntegratorConfig::default();
    assert_eq!(
        default_config.relevance_threshold, 0.5,
        "Default threshold should be 0.5"
    );
    assert_eq!(
        default_config.max_memories, 10,
        "Default max memories should be 10"
    );

    // 2. 测试自定义配置
    let custom_config = MemoryIntegratorConfig {
        relevance_threshold: 0.8,
        max_memories: 20,
        include_timestamp: true,
        sort_by_importance: true,
    };
    assert_eq!(custom_config.relevance_threshold, 0.8);
    assert_eq!(custom_config.max_memories, 20);

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
        let memory = create_test_memory(
            &format!("Test {memory_type:?} memory"),
            memory_type.clone(),
            Some(0.8),
        );
        assert_eq!(memory.memory_type, memory_type);
    }

    println!("✅ test_memory_types passed");
}
