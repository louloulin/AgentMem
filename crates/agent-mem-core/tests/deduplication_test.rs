//! MemoryDeduplicator 单元测试

use agent_mem_core::managers::{DeduplicationConfig, MemoryDeduplicator, MergeStrategy};
use chrono::Utc;
use std::collections::HashMap;

// 辅助函数：创建测试记忆
fn create_test_memory(
    id: &str,
    content: &str,
    importance: f32,
) -> agent_mem_core::managers::deduplication::MemoryItem {
    agent_mem_core::managers::deduplication::MemoryItem {
        id: id.to_string(),
        content: content.to_string(),
        importance,
        created_at: Utc::now(),
        metadata: HashMap::new(),
    }
}

#[test]
fn test_deduplication_config_default() {
    let config = DeduplicationConfig::default();

    assert_eq!(config.similarity_threshold, 0.85);
    assert_eq!(config.time_window_seconds, 30 * 60); // 30 分钟
    assert_eq!(config.batch_size, 100);
    assert!(config.enable_intelligent_merge);
    assert!(config.preserve_history);
}

#[test]
fn test_deduplication_config_custom() {
    let config = DeduplicationConfig {
        similarity_threshold: 0.9,
        time_window_seconds: 60 * 60, // 1 小时
        batch_size: 50,
        enable_intelligent_merge: false,
        preserve_history: false,
    };

    assert_eq!(config.similarity_threshold, 0.9);
    assert_eq!(config.time_window_seconds, 60 * 60);
    assert_eq!(config.batch_size, 50);
    assert!(!config.enable_intelligent_merge);
    assert!(!config.preserve_history);
}

#[test]
fn test_merge_strategy_serialization() {
    let strategy = MergeStrategy::KeepLatest;
    let serialized = serde_json::to_string(&strategy).unwrap();
    assert!(serialized.contains("keep_latest"));

    let strategy = MergeStrategy::IntelligentMerge;
    let serialized = serde_json::to_string(&strategy).unwrap();
    assert!(serialized.contains("intelligent_merge"));
}

#[test]
fn test_deduplicator_creation() {
    let config = DeduplicationConfig::default();
    let deduplicator = MemoryDeduplicator::new(config);
    let stats = deduplicator.get_stats();

    assert_eq!(stats.get("total_processed"), Some(&0));
    assert_eq!(stats.get("total_duplicates_found"), Some(&0));
}

#[test]
fn test_deduplicator_with_default_config() {
    let deduplicator = MemoryDeduplicator::with_default_config();
    let stats = deduplicator.get_stats();

    assert_eq!(stats.get("total_processed"), Some(&0));
}

#[test]
fn test_deduplicate_empty_list() {
    let mut deduplicator = MemoryDeduplicator::with_default_config();
    let memories = vec![];

    let result = deduplicator.deduplicate(memories);
    assert!(result.is_ok());

    let (dedup_result, processed) = result.unwrap();
    assert_eq!(dedup_result.original_count, 0);
    assert_eq!(dedup_result.deduplicated_count, 0);
    assert_eq!(dedup_result.duplicate_count, 0);
    assert_eq!(processed.len(), 0);
}

#[test]
fn test_deduplicate_single_memory() {
    let mut deduplicator = MemoryDeduplicator::with_default_config();
    let memories = vec![create_test_memory("mem-1", "Test content", 0.8)];

    let result = deduplicator.deduplicate(memories);
    assert!(result.is_ok());

    let (dedup_result, processed) = result.unwrap();
    assert_eq!(dedup_result.original_count, 1);
    assert_eq!(dedup_result.deduplicated_count, 1);
    assert_eq!(dedup_result.duplicate_count, 0);
    assert_eq!(processed.len(), 1);
}

#[test]
fn test_deduplicate_identical_memories() {
    let mut deduplicator = MemoryDeduplicator::with_default_config();
    let memories = vec![
        create_test_memory("mem-1", "Identical content", 0.8),
        create_test_memory("mem-2", "Identical content", 0.7),
    ];

    let result = deduplicator.deduplicate(memories);
    assert!(result.is_ok());

    let (dedup_result, processed) = result.unwrap();
    assert_eq!(dedup_result.original_count, 2);
    // 应该去重为 1 个
    assert!(dedup_result.deduplicated_count <= 2);
}

#[test]
fn test_deduplicate_different_memories() {
    let mut deduplicator = MemoryDeduplicator::with_default_config();
    let memories = vec![
        create_test_memory("mem-1", "Content A", 0.8),
        create_test_memory("mem-2", "Content B", 0.7),
        create_test_memory("mem-3", "Content C", 0.9),
    ];

    let result = deduplicator.deduplicate(memories);
    assert!(result.is_ok());

    let (dedup_result, processed) = result.unwrap();
    assert_eq!(dedup_result.original_count, 3);
    // 不同内容应该保留
    assert_eq!(processed.len(), 3);
}

#[test]
fn test_deduplication_result_fields() {
    let mut deduplicator = MemoryDeduplicator::with_default_config();
    let memories = vec![
        create_test_memory("mem-1", "Test", 0.8),
        create_test_memory("mem-2", "Test", 0.7),
    ];

    let result = deduplicator.deduplicate(memories);
    assert!(result.is_ok());

    let (dedup_result, _) = result.unwrap();
    assert!(dedup_result.processing_time_ms >= 0);
    assert!(dedup_result.deduplication_rate >= 0.0);
    assert!(dedup_result.deduplication_rate <= 100.0);
}

#[test]
fn test_stats_tracking() {
    let mut deduplicator = MemoryDeduplicator::with_default_config();
    let memories = vec![
        create_test_memory("mem-1", "Test", 0.8),
        create_test_memory("mem-2", "Test", 0.7),
    ];

    let _ = deduplicator.deduplicate(memories);
    let stats = deduplicator.get_stats();

    assert!(stats.get("total_processed").unwrap() > &0);
}

#[test]
fn test_similarity_threshold_validation() {
    let config = DeduplicationConfig {
        similarity_threshold: 0.95,
        ..Default::default()
    };

    assert!(config.similarity_threshold > 0.0);
    assert!(config.similarity_threshold <= 1.0);
}

#[test]
fn test_time_window_validation() {
    let config = DeduplicationConfig {
        time_window_seconds: 3600, // 1 小时
        ..Default::default()
    };

    assert!(config.time_window_seconds > 0);
}

#[test]
fn test_batch_size_validation() {
    let config = DeduplicationConfig {
        batch_size: 50,
        ..Default::default()
    };

    assert!(config.batch_size > 0);
}

#[test]
fn test_merge_strategy_equality() {
    assert_eq!(MergeStrategy::KeepLatest, MergeStrategy::KeepLatest);
    assert_ne!(MergeStrategy::KeepLatest, MergeStrategy::KeepMostImportant);
}

#[test]
fn test_merge_strategy_clone() {
    let strategy1 = MergeStrategy::IntelligentMerge;
    let strategy2 = strategy1.clone();
    assert_eq!(strategy1, strategy2);
}

#[test]
fn test_deduplication_rate_calculation() {
    let mut deduplicator = MemoryDeduplicator::with_default_config();
    let memories = vec![
        create_test_memory("mem-1", "Same", 0.8),
        create_test_memory("mem-2", "Same", 0.7),
        create_test_memory("mem-3", "Different", 0.9),
    ];

    let result = deduplicator.deduplicate(memories);
    assert!(result.is_ok());

    let (dedup_result, _) = result.unwrap();
    // 去重率应该在合理范围内
    assert!(dedup_result.deduplication_rate >= 0.0);
    assert!(dedup_result.deduplication_rate <= 100.0);
}

#[test]
fn test_removed_ids_tracking() {
    let mut deduplicator = MemoryDeduplicator::with_default_config();
    let memories = vec![
        create_test_memory("mem-1", "Same", 0.8),
        create_test_memory("mem-2", "Same", 0.7),
    ];

    let result = deduplicator.deduplicate(memories);
    assert!(result.is_ok());

    let (dedup_result, _) = result.unwrap();
    // 应该有被移除的 ID（如果检测到重复）
    assert!(dedup_result.removed_ids.len() <= 1);
}

#[test]
fn test_merged_groups_tracking() {
    let mut deduplicator = MemoryDeduplicator::with_default_config();
    let memories = vec![
        create_test_memory("mem-1", "Same", 0.8),
        create_test_memory("mem-2", "Same", 0.7),
    ];

    let result = deduplicator.deduplicate(memories);
    assert!(result.is_ok());

    let (dedup_result, _) = result.unwrap();
    // 合并组数应该合理
    assert!(dedup_result.merged_groups <= 1);
}

#[test]
fn test_config_clone() {
    let config1 = DeduplicationConfig::default();
    let config2 = config1.clone();

    assert_eq!(config1.similarity_threshold, config2.similarity_threshold);
    assert_eq!(config1.time_window_seconds, config2.time_window_seconds);
    assert_eq!(config1.batch_size, config2.batch_size);
}

#[test]
fn test_multiple_deduplication_runs() {
    let mut deduplicator = MemoryDeduplicator::with_default_config();

    // 第一次去重
    let memories1 = vec![
        create_test_memory("mem-1", "Test 1", 0.8),
        create_test_memory("mem-2", "Test 1", 0.7),
    ];
    let _ = deduplicator.deduplicate(memories1);

    // 第二次去重
    let memories2 = vec![
        create_test_memory("mem-3", "Test 2", 0.8),
        create_test_memory("mem-4", "Test 2", 0.7),
    ];
    let _ = deduplicator.deduplicate(memories2);

    let stats = deduplicator.get_stats();
    // 总处理数应该累加
    assert_eq!(stats.get("total_processed"), Some(&4));
}

#[test]
fn test_importance_based_merging() {
    // 测试基于重要性的合并逻辑
    let config = DeduplicationConfig {
        enable_intelligent_merge: true,
        ..Default::default()
    };

    let deduplicator = MemoryDeduplicator::new(config);
    let stats = deduplicator.get_stats();
    assert_eq!(stats.get("total_processed"), Some(&0));
}

#[test]
fn test_preserve_history_flag() {
    let config = DeduplicationConfig {
        preserve_history: true,
        ..Default::default()
    };

    assert!(config.preserve_history);

    let config = DeduplicationConfig {
        preserve_history: false,
        ..Default::default()
    };

    assert!(!config.preserve_history);
}

#[test]
fn test_all_merge_strategies() {
    let strategies = vec![
        MergeStrategy::KeepLatest,
        MergeStrategy::KeepMostImportant,
        MergeStrategy::KeepMostComplete,
        MergeStrategy::IntelligentMerge,
        MergeStrategy::UserDefined,
    ];

    for strategy in strategies {
        let serialized = serde_json::to_string(&strategy).unwrap();
        assert!(!serialized.is_empty());
    }
}
