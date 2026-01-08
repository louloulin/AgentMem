//! AgentMem 2.6 功能验证测试
//!
//! 验证 P0-P2 核心功能的实现和可用性
//!
//! 📅 Created: 2025-01-08
//! 🎯 Purpose: 验证核心功能已实现并可工作

use agent_mem_core::Memory;
use agent_mem_traits::{
    scheduler::{MemoryScheduler, ScheduleConfig},
    TimeDecayModel,
};
use std::sync::Arc;

/// 验证 P0: MemoryScheduler trait 已实现
#[tokio::test]
async fn verify_p0_scheduler_exists() {
    let time_decay = TimeDecayModel::new(0.1);
    let scheduler = Arc::new(time_decay);

    // 创建测试记忆
    let memory = Memory::builder()
        .with_content("Test content")
        .with_attribute("importance", 0.8)
        .build();

    let memories = vec![memory];
    let config = ScheduleConfig::default();

    // 验证 scheduler 可以调用
    let result = scheduler
        .select_memories("test", memories, 1, &config)
        .await;

    assert!(result.is_ok(), "P0 Scheduler should work");
    assert!(!result.unwrap().is_empty(), "Should return memories");
}

/// 验证 P1: Memory V4 的开放属性系统
#[test]
fn verify_p1_memory_v4_attributes() {
    // 验证可以添加自定义属性
    let memory = Memory::builder()
        .with_content("Test")
        .with_attribute("custom_field", "custom_value")
        .with_attribute("numeric", 42.0)
        .with_attribute("boolean", true)
        .build();

    // 验证属性可访问
    let attrs = memory.attributes();
    assert!(attrs.contains_key(&"custom_field".into()));
    assert!(attrs.contains_key(&"numeric".into()));
    assert!(attrs.contains_key(&"boolean".into()));
}

/// 验证 P2: ContextCompressor 已实现
#[test]
fn verify_p2_context_compressor_exists() {
    use agent_mem_core::llm_optimizer::ContextCompressorConfig;

    let config = ContextCompressorConfig::default();

    // 验证配置正确
    assert_eq!(config.max_context_tokens, 3000);
    assert_eq!(config.target_compression_ratio, 0.7);
    assert_eq!(config.importance_threshold, 0.7);
}

/// 验证 P2: MultiLevelCache 已实现
#[test]
fn verify_p2_multilevel_cache_exists() {
    use agent_mem_core::llm_optimizer::MultiLevelCacheConfig;

    let config = MultiLevelCacheConfig::default();

    // 验证默认配置
    assert!(config.l1.is_some() || config.l2.is_some() || config.l3.is_some());
}

/// 验证核心功能集成
#[tokio::test]
async fn verify_p0_p1_p2_integration() {
    // P0: 创建 scheduler
    let time_decay = TimeDecayModel::new(0.1);
    let scheduler = Arc::new(time_decay);

    // P1: 创建带有开放属性的记忆
    let memories: Vec<Memory> = (0..5)
        .map(|i| {
            Memory::builder()
                .with_content(format!("Memory {}", i))
                .with_attribute("importance", 0.5 + (i as f64 * 0.1))
                .with_attribute("category", "test")
                .build()
        })
        .collect();

    // P0: 使用调度器
    let config = ScheduleConfig::default();
    let result = scheduler
        .select_memories("query", memories, 3, &config)
        .await;

    assert!(result.is_ok(), "Integration should work");

    let selected = result.unwrap();
    assert!(selected.len() <= 3, "Should limit to top 3");

    // P2: 验证可以应用压缩配置
    use agent_mem_core::llm_optimizer::ContextCompressorConfig;
    let compressor_config = ContextCompressorConfig::default();
    assert!(compressor_config.target_compression_ratio > 0.0);
}
