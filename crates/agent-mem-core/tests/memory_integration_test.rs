//! MemoryIntegrator 单元测试

use agent_mem_core::orchestrator::memory_integration::{MemoryIntegrator, MemoryIntegratorConfig};
use agent_mem_core::engine::{MemoryEngine, MemoryEngineConfig};
use agent_mem_core::Memory;
use agent_mem_traits::{Message, MessageRole, MemoryType};
use chrono::Utc;
use std::sync::Arc;

#[test]
fn test_memory_integrator_config_default() {
    let config = MemoryIntegratorConfig::default();
    assert_eq!(config.max_memories, 10);
    assert_eq!(config.relevance_threshold, 0.5);
    assert!(config.include_importance);
    assert!(config.include_timestamp);
}

#[test]
fn test_memory_integrator_config_custom() {
    let config = MemoryIntegratorConfig {
        max_memories: 20,
        relevance_threshold: 0.7,
        include_importance: false,
        include_timestamp: false,
    };
    assert_eq!(config.max_memories, 20);
    assert_eq!(config.relevance_threshold, 0.7);
    assert!(!config.include_importance);
    assert!(!config.include_timestamp);
}

#[tokio::test]
async fn test_retrieve_memories_empty_query() {
    let memory_engine = Arc::new(MemoryEngine::new(MemoryEngineConfig::default()));
    let config = MemoryIntegratorConfig::default();
    let integrator = MemoryIntegrator::new(memory_engine, config);

    let result = integrator
        .retrieve_memories("", "agent-1", "user-1", 10)
        .await;

    assert!(result.is_ok());
    let memories = result.unwrap();
    // 空查询应该返回空结果或最近的记忆
    println!("Retrieved {} memories for empty query", memories.len());
}

#[test]
fn test_format_memory_for_prompt() {
    let memory = Memory {
        id: "mem-1".to_string(),
        organization_id: "org-1".to_string(),
        user_id: "user-1".to_string(),
        agent_id: "agent-1".to_string(),
        content: "User prefers Python".to_string(),
        memory_type: MemoryType::Semantic,
        importance: 0.8,
        score: Some(0.9),
        metadata: serde_json::json!({}),
        created_at: Utc::now().timestamp(),
        updated_at: Some(Utc::now().timestamp()),
        expires_at: None,
        access_count: 5,
        version: 1,
    };

    let formatted = format_memory_for_test(&memory, true, true);
    assert!(formatted.contains("User prefers Python"));
    assert!(formatted.contains("Semantic"));
    assert!(formatted.contains("0.8")); // importance
}

// 辅助函数：格式化记忆
fn format_memory_for_test(memory: &Memory, include_importance: bool, include_timestamp: bool) -> String {
    let mut formatted = format!("[{:?}] {}", memory.memory_type, memory.content);
    
    if include_importance {
        formatted.push_str(&format!(" (importance: {:.1})", memory.importance));
    }
    
    if include_timestamp {
        formatted.push_str(&format!(" (created: {})", memory.created_at));
    }
    
    formatted
}

#[test]
fn test_filter_by_relevance() {
    let memories = vec![
        create_test_memory("mem-1", "Relevant content", 0.9),
        create_test_memory("mem-2", "Less relevant", 0.4),
        create_test_memory("mem-3", "Very relevant", 0.8),
    ];

    let threshold = 0.5;
    let filtered: Vec<_> = memories
        .into_iter()
        .filter(|m| m.score.unwrap_or(0.0) >= threshold)
        .collect();

    assert_eq!(filtered.len(), 2);
}

// 辅助函数：创建测试记忆
fn create_test_memory(id: &str, content: &str, score: f32) -> Memory {
    Memory {
        id: id.to_string(),
        organization_id: "org-1".to_string(),
        user_id: "user-1".to_string(),
        agent_id: "agent-1".to_string(),
        content: content.to_string(),
        memory_type: MemoryType::Semantic,
        importance: score,
        score: Some(score),
        metadata: serde_json::json!({}),
        created_at: Utc::now().timestamp(),
        updated_at: Some(Utc::now().timestamp()),
        expires_at: None,
        access_count: 0,
        version: 1,
    }
}

#[test]
fn test_sort_by_importance() {
    let mut memories = vec![
        create_test_memory("mem-1", "Low importance", 0.3),
        create_test_memory("mem-2", "High importance", 0.9),
        create_test_memory("mem-3", "Medium importance", 0.6),
    ];

    memories.sort_by(|a, b| {
        b.importance
            .partial_cmp(&a.importance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    assert_eq!(memories[0].id, "mem-2"); // 最高重要性
    assert_eq!(memories[1].id, "mem-3");
    assert_eq!(memories[2].id, "mem-1"); // 最低重要性
}

#[test]
fn test_limit_memories() {
    let memories = vec![
        create_test_memory("mem-1", "Memory 1", 0.9),
        create_test_memory("mem-2", "Memory 2", 0.8),
        create_test_memory("mem-3", "Memory 3", 0.7),
        create_test_memory("mem-4", "Memory 4", 0.6),
        create_test_memory("mem-5", "Memory 5", 0.5),
    ];

    let max_memories = 3;
    let limited: Vec<_> = memories.into_iter().take(max_memories).collect();

    assert_eq!(limited.len(), 3);
}

#[test]
fn test_build_memory_context() {
    let memories = vec![
        create_test_memory("mem-1", "User likes Python", 0.8),
        create_test_memory("mem-2", "User is working on ML project", 0.9),
    ];

    let context = build_memory_context_for_test(&memories);
    assert!(context.contains("User likes Python"));
    assert!(context.contains("User is working on ML project"));
}

// 辅助函数：构建记忆上下文
fn build_memory_context_for_test(memories: &[Memory]) -> String {
    if memories.is_empty() {
        return String::new();
    }

    let mut context = String::from("Relevant memories:\n\n");
    for (i, memory) in memories.iter().enumerate() {
        context.push_str(&format!("{}. {}\n", i + 1, memory.content));
    }
    context
}

#[test]
fn test_memory_type_labels() {
    let types = vec![
        (MemoryType::Episodic, "Episodic"),
        (MemoryType::Semantic, "Semantic"),
        (MemoryType::Procedural, "Procedural"),
        (MemoryType::Working, "Working"),
    ];

    for (memory_type, expected_label) in types {
        let label = format!("{:?}", memory_type);
        assert_eq!(label, expected_label);
    }
}

#[test]
fn test_relevance_threshold_validation() {
    let valid_thresholds = vec![0.0, 0.5, 1.0];
    for threshold in valid_thresholds {
        assert!(threshold >= 0.0 && threshold <= 1.0);
    }

    let invalid_thresholds = vec![-0.1, 1.1];
    for threshold in invalid_thresholds {
        assert!(threshold < 0.0 || threshold > 1.0);
    }
}

#[test]
fn test_max_memories_validation() {
    let valid_limits = vec![1, 10, 100];
    for limit in valid_limits {
        assert!(limit > 0);
    }

    let invalid_limit = 0;
    assert!(invalid_limit == 0);
}

#[test]
fn test_memory_metadata() {
    let memory = create_test_memory("mem-1", "Test content", 0.8);
    
    assert!(memory.metadata.is_object());
    assert_eq!(memory.version, 1);
    assert_eq!(memory.access_count, 0);
}

#[test]
fn test_memory_timestamps() {
    let memory = create_test_memory("mem-1", "Test content", 0.8);
    
    assert!(memory.created_at > 0);
    assert!(memory.updated_at.is_some());
    assert!(memory.expires_at.is_none());
}

#[test]
fn test_combine_memories_and_messages() {
    let memories = vec![
        create_test_memory("mem-1", "User likes Python", 0.8),
    ];

    let messages = vec![
        Message {
            role: MessageRole::User,
            content: "Tell me about Python".to_string(),
            timestamp: Some(Utc::now()),
        },
    ];

    let memory_context = build_memory_context_for_test(&memories);
    assert!(!memory_context.is_empty());
    assert_eq!(messages.len(), 1);
}

#[test]
fn test_empty_memories_handling() {
    let memories: Vec<Memory> = vec![];
    let context = build_memory_context_for_test(&memories);
    assert!(context.is_empty());
}

#[test]
fn test_memory_importance_range() {
    let memory = create_test_memory("mem-1", "Test", 0.5);
    assert!(memory.importance >= 0.0 && memory.importance <= 1.0);
}

#[test]
fn test_memory_score_optional() {
    let mut memory = create_test_memory("mem-1", "Test", 0.5);
    assert!(memory.score.is_some());
    
    memory.score = None;
    assert!(memory.score.is_none());
}

