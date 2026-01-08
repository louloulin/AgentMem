//! AgentMem 2.6 Integration Test
//!
//! 集成测试验证 P0-P2 功能的端到端工作流程
//!
//! 测试范围：
//! - P0: Memory Scheduler (记忆调度)
//! - P1: 8 Advanced Capabilities (8种高级能力)
//! - P2: Performance Optimization (性能优化)
//!
//! 📅 Created: 2025-01-08
//! 🎯 Purpose: End-to-end integration validation

use agent_mem_core::Memory;
use agent_mem_traits::{
    scheduler::{MemoryScheduler, ScheduleConfig},
    TimeDecayModel,
};
use std::sync::Arc;

/// Helper function to create test memories
fn create_test_memories() -> Vec<Memory> {
    let mut memories = Vec::new();

    for i in 0..10 {
        let importance = 0.5 + (i as f64 * 0.05);

        let memory = Memory::builder()
            .with_content(format!("Test memory content {}", i))
            .with_attribute("importance", importance)
            .with_attribute("category", "test")
            .build();

        memories.push(memory);
    }

    memories
}

/// Test P0: MemoryScheduler basic functionality
#[tokio::test]
async fn test_p0_memory_scheduler_basic() {
    let time_decay = TimeDecayModel::new(0.1);
    let scheduler = Arc::new(time_decay);
    let memories = create_test_memories();

    let config = ScheduleConfig::default();
    let query = "test query";

    let result = scheduler
        .select_memories(query, memories.clone(), 5, &config)
        .await;

    assert!(result.is_ok(), "Scheduler should succeed");

    let selected = result.unwrap();
    assert_eq!(selected.len(), 5, "Should select 5 memories");
}

/// Test P0: MemoryScheduler with time decay
#[tokio::test]
async fn test_p0_memory_scheduler_time_decay() {
    let time_decay = TimeDecayModel::new(0.1);
    let scheduler = Arc::new(time_decay);
    let memories = create_test_memories();

    let mut config = ScheduleConfig::default();
    config.enable_time_decay = true;
    config.time_decay_lambda = 0.1;

    let result = scheduler
        .select_memories("test", memories, 3, &config)
        .await;

    assert!(result.is_ok(), "Scheduler with time decay should succeed");
}

/// Test P0-P1: Scheduler with importance scoring
#[tokio::test]
async fn test_p0_p1_scheduler_importance() {
    let time_decay = TimeDecayModel::new(0.1);
    let scheduler = Arc::new(time_decay);

    let mut memories = create_test_memories();
    // Add a high importance memory
    let important_memory = Memory::builder()
        .with_content("Important information")
        .with_attribute("importance", 0.95)
        .with_attribute("category", "critical")
        .build();
    memories.push(important_memory);

    let mut config = ScheduleConfig::default();
    config.importance_weight = 0.5; // Increase importance weight

    let result = scheduler
        .select_memories("important", memories, 3, &config)
        .await;

    assert!(result.is_ok());

    let selected = result.unwrap();
    // The important memory should be ranked high
    assert!(selected.iter().any(|m| {
        m.attributes()
            .get(&"importance".into())
            .and_then(|v| v.as_number())
            .map_or(false, |v| v > 0.9)
    }));
}

/// Test P2: Performance optimization - context compression preparation
#[tokio::test]
async fn test_p2_context_compressor_config() {
    use agent_mem_core::llm_optimizer::{
        ContextCompressor, ContextCompressorConfig,
    };

    let config = ContextCompressorConfig::default();
    assert_eq!(config.max_context_tokens, 3000);
    assert_eq!(config.target_compression_ratio, 0.7);
    assert_eq!(config.importance_threshold, 0.7);

    let compressor = ContextCompressor::new(config);
    assert!(compressor.compress_context("", &[]).await.is_ok());
}

/// Test P2: Multi-level cache configuration
#[tokio::test]
async fn test_p2_multilevel_cache_config() {
    use agent_mem_core::llm_optimizer::{
        MultiLevelCache, MultiLevelCacheConfig, CacheLevelConfig,
    };

    let l1_config = CacheLevelConfig {
        max_entries: 100,
        ttl_seconds: 300, // 5 minutes
    };

    let l2_config = CacheLevelConfig {
        max_entries: 1000,
        ttl_seconds: 1800, // 30 minutes
    };

    let l3_config = CacheLevelConfig {
        max_entries: 10000,
        ttl_seconds: 7200, // 2 hours
    };

    let config = MultiLevelCacheConfig {
        l1: Some(l1_config),
        l2: Some(l2_config),
        l3: Some(l3_config),
    };

    let cache = MultiLevelCache::new(config);
    assert!(cache.get("test_key").await.is_ok());
}

/// Integration Test: P0-P2 Combined Workflow
#[tokio::test]
async fn test_integration_p0_p1_p2_combined() {
    // Step 1: Create memories
    let memories = create_test_memories();

    // Step 2: Apply P0 scheduling
    let time_decay = TimeDecayModel::new(0.1);
    let scheduler = Arc::new(time_decay);
    let config = ScheduleConfig::default();

    let scheduled = scheduler
        .select_memories("test query", memories, 5, &config)
        .await
        .expect("Scheduling should succeed");

    assert!(!scheduled.is_empty(), "Should have scheduled memories");

    // Step 3: Verify P2 optimization can be applied
    use agent_mem_core::llm_optimizer::ContextCompressorConfig;

    let compressor_config = ContextCompressorConfig::default();
    assert!(compressor_config.target_compression_ratio > 0.0);

    // Verify the workflow completes successfully
    assert!(scheduled.len() <= 5, "Should limit to top 5 memories");
}

/// Test P1: Active Retrieval Preparation
#[tokio::test]
async fn test_p1_active_retrieval_preparation() {
    // This test prepares for active retrieval functionality
    let memories = create_test_memories();

    // Verify memories have the necessary attributes for active retrieval
    for memory in &memories {
        assert!(
            memory.content().len() > 0,
            "Memory should have content"
        );
    }
}

/// Test Memory V4: Open attribute system
#[test]
fn test_memory_v4_open_attributes() {
    // Test Memory V4's open attribute system
    let memory = Memory::builder()
        .with_content("Test content")
        .with_attribute("custom_field", "custom_value")
        .with_attribute("numeric_value", 42)
        .with_attribute("boolean_value", true)
        .build();

    // Verify custom attributes are accessible
    assert_eq!(
        memory
            .attributes()
            .get(&"custom_field".into())
            .and_then(|v| v.as_string()),
        Some("custom_value")
    );

    assert_eq!(
        memory
            .attributes()
            .get(&"numeric_value".into())
            .and_then(|v| v.as_number()),
        Some(42.0)
    );
}

/// Test Memory V4: Multimodal content support
#[test]
fn test_memory_v4_multimodal() {
    use agent_mem_core::MemoryContent;

    // Test text content
    let text_content = MemoryContent::Text("Hello, world!".to_string());
    assert!(matches!(text_content, MemoryContent::Text(_)));

    // Test structured content
    let structured = MemoryContent::Structured(serde_json::json!({
        "key": "value",
        "number": 42
    }));
    assert!(matches!(structured, MemoryContent::Structured(_)));
}

/// Benchmark: P0 Scheduler Performance
#[tokio::test]
async fn benchmark_p0_scheduler_performance() {
    use std::time::Instant;

    let time_decay = TimeDecayModel::new(0.1);
    let scheduler = Arc::new(time_decay);
    let memories = create_test_memories();

    let config = ScheduleConfig::default();

    let start = Instant::now();
    let result = scheduler
        .select_memories("test", memories, 5, &config)
        .await;

    let elapsed = start.elapsed();

    assert!(result.is_ok(), "Scheduling should succeed");
    assert!(
        elapsed.as_millis() < 100,
        "Scheduling should complete in < 100ms, took {}ms",
        elapsed.as_millis()
    );
}

/// Test P0-P2: Error handling
#[tokio::test]
async fn test_error_handling() {
    let time_decay = TimeDecayModel::new(0.1);
    let scheduler = Arc::new(time_decay);
    let empty_memories: Vec<Memory> = vec![];

    let config = ScheduleConfig::default();

    let result = scheduler
        .select_memories("test", empty_memories, 5, &config)
        .await;

    // Should handle empty memories gracefully
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}
