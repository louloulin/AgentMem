//! AgentMem 2.6 功能验证测试 - 简化版
//!
//! 验证 P0-P2 核心功能的实现
//!
//! 📅 Created: 2025-01-08
//! 🎯 Purpose: 验证核心功能已实现

// P0: 验证 Scheduler trait 存在并可用
#[test]
fn test_p0_scheduler_trait_exists() {
    use agent_mem_traits::scheduler::ScheduleConfig;

    let config = ScheduleConfig::default();
    assert!(config.relevance_weight > 0.0);
    assert!(config.importance_weight > 0.0);
    assert!(config.recency_weight > 0.0);
}

// P0: 验证时间衰减模型
#[test]
fn test_p0_time_decay_model() {
    use agent_mem_core::scheduler::ExponentialDecayModel;
    use agent_mem_core::scheduler::TimeDecayModel;

    let model = ExponentialDecayModel::new(0.1);

    // 测试衰减计算
    let score_now = model.decay_score(0.0);
    assert!((score_now - 1.0).abs() < 0.01, "Current memory should have score ~1.0");

    let score_old = model.decay_score(10.0);
    assert!(score_old < score_now, "Older memory should have lower score");
    assert!(score_old > 0.0, "Score should be positive");
}

// P1: 验证 Memory V4 存在并可用
#[test]
fn test_p1_memory_v4_exists() {
    use agent_mem_core::Memory;

    // 验证可以创建 Memory
    let memory = Memory::new(
        "test_agent",
        Some("test_user".to_string()),
        "test",
        "Test content",
        0.5,
    );

    assert_eq!(memory.agent_id(), "test_agent");
    assert_eq!(memory.content(), "Test content");
}

// P1: 验证 Memory V4 属性系统
#[test]
fn test_p1_memory_v4_attributes() {
    use agent_mem_core::Memory;
    use agent_mem_traits::AttributeKey;

    let memory = Memory::new(
        "test_agent",
        None,
        "test",
        "Test content",
        0.5,
    );

    // 验证可以访问属性
    let attrs = memory.attributes();
    assert!(!attrs.is_empty(), "Should have system attributes");
}

// P2: 验证 ContextCompressorConfig
#[test]
fn test_p2_context_compressor_config() {
    use agent_mem_core::llm_optimizer::ContextCompressorConfig;

    let config = ContextCompressorConfig::default();

    assert_eq!(config.max_context_tokens, 3000);
    assert_eq!(config.target_compression_ratio, 0.7);
    assert_eq!(config.importance_threshold, 0.7);
}

// P2: 验证 MultiLevelCacheConfig
#[test]
fn test_p2_multilevel_cache_config() {
    use agent_mem_core::llm_optimizer::{CacheLevelConfig, MultiLevelCacheConfig};

    let l1_config = CacheLevelConfig {
        max_entries: 100,
        ttl_seconds: 300,
    };

    let config = MultiLevelCacheConfig {
        l1: Some(l1_config),
        l2: None,
        l3: None,
    };

    assert!(config.l1.is_some());
}

// 集成测试: P0-P2 功能协同
#[test]
fn test_p0_p1_p2_integration() {
    use agent_mem_core::Memory;
    use agent_mem_core::llm_optimizer::ContextCompressorConfig;
    use agent_mem_traits::scheduler::ScheduleConfig;

    // P1: 创建记忆
    let memory = Memory::new(
        "test_agent",
        None,
        "test",
        "Integration test",
        0.8,
    );

    assert!(!memory.content().is_empty());

    // P0: 验证调度配置
    let config = ScheduleConfig::default();
    assert!(config.relevance_weight > 0.0);

    // P2: 验证压缩配置
    let compressor_config = ContextCompressorConfig::default();
    assert!(compressor_config.target_compression_ratio > 0.0);
}
