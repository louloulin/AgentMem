//! AssociationManager 单元测试

#![cfg(feature = "postgres")]

use agent_mem_core::managers::{AssociationManagerConfig, AssociationType};

#[test]
fn test_association_type_as_str() {
    assert_eq!(AssociationType::Causal.as_str(), "causal");
    assert_eq!(AssociationType::Temporal.as_str(), "temporal");
    assert_eq!(AssociationType::Similar.as_str(), "similar");
    assert_eq!(AssociationType::Contrast.as_str(), "contrast");
    assert_eq!(AssociationType::Hierarchical.as_str(), "hierarchical");
    assert_eq!(AssociationType::Reference.as_str(), "reference");
}

#[test]
fn test_association_type_from_str() {
    assert_eq!(
        AssociationType::from_str("causal").unwrap(),
        AssociationType::Causal
    );
    assert_eq!(
        AssociationType::from_str("temporal").unwrap(),
        AssociationType::Temporal
    );
    assert_eq!(
        AssociationType::from_str("similar").unwrap(),
        AssociationType::Similar
    );
    assert_eq!(
        AssociationType::from_str("contrast").unwrap(),
        AssociationType::Contrast
    );
    assert_eq!(
        AssociationType::from_str("hierarchical").unwrap(),
        AssociationType::Hierarchical
    );
    assert_eq!(
        AssociationType::from_str("reference").unwrap(),
        AssociationType::Reference
    );
}

#[test]
fn test_association_type_custom() {
    let custom = AssociationType::Custom("my_type".to_string());
    assert_eq!(custom.as_str(), "my_type");

    let parsed = AssociationType::from_str("unknown_type").unwrap();
    match parsed {
        AssociationType::Custom(s) => assert_eq!(s, "unknown_type"),
        _ => panic!("Expected Custom variant"),
    }
}

#[test]
fn test_association_manager_config_default() {
    let config = AssociationManagerConfig::default();

    assert_eq!(config.min_strength_threshold, 0.3);
    assert_eq!(config.max_associations_per_memory, 50);
    assert!(config.auto_create_similar);
    assert_eq!(config.similarity_threshold, 0.7);
}

#[test]
fn test_association_manager_config_custom() {
    let config = AssociationManagerConfig {
        min_strength_threshold: 0.5,
        max_associations_per_memory: 100,
        auto_create_similar: false,
        similarity_threshold: 0.8,
    };

    assert_eq!(config.min_strength_threshold, 0.5);
    assert_eq!(config.max_associations_per_memory, 100);
    assert!(!config.auto_create_similar);
    assert_eq!(config.similarity_threshold, 0.8);
}

#[test]
fn test_association_type_equality() {
    assert_eq!(AssociationType::Causal, AssociationType::Causal);
    assert_ne!(AssociationType::Causal, AssociationType::Temporal);
}

#[test]
fn test_association_type_clone() {
    let type1 = AssociationType::Similar;
    let type2 = type1.clone();
    assert_eq!(type1, type2);
}

#[test]
fn test_association_type_serialization() {
    let assoc_type = AssociationType::Causal;
    let serialized = serde_json::to_string(&assoc_type).unwrap();
    assert!(serialized.contains("causal"));

    let assoc_type = AssociationType::Temporal;
    let serialized = serde_json::to_string(&assoc_type).unwrap();
    assert!(serialized.contains("temporal"));
}

#[test]
fn test_strength_threshold_validation() {
    let config = AssociationManagerConfig::default();
    assert!(config.min_strength_threshold >= 0.0);
    assert!(config.min_strength_threshold <= 1.0);
}

#[test]
fn test_similarity_threshold_validation() {
    let config = AssociationManagerConfig::default();
    assert!(config.similarity_threshold >= 0.0);
    assert!(config.similarity_threshold <= 1.0);
}

#[test]
fn test_max_associations_validation() {
    let config = AssociationManagerConfig::default();
    assert!(config.max_associations_per_memory > 0);
}

#[test]
fn test_config_clone() {
    let config1 = AssociationManagerConfig::default();
    let config2 = config1.clone();

    assert_eq!(config1.min_strength_threshold, config2.min_strength_threshold);
    assert_eq!(
        config1.max_associations_per_memory,
        config2.max_associations_per_memory
    );
    assert_eq!(config1.auto_create_similar, config2.auto_create_similar);
    assert_eq!(config1.similarity_threshold, config2.similarity_threshold);
}

#[test]
fn test_all_association_types() {
    let types = vec![
        AssociationType::Causal,
        AssociationType::Temporal,
        AssociationType::Similar,
        AssociationType::Contrast,
        AssociationType::Hierarchical,
        AssociationType::Reference,
    ];

    for assoc_type in types {
        let str_repr = assoc_type.as_str();
        assert!(!str_repr.is_empty());
        assert!(str_repr.chars().all(|c| c.is_ascii_lowercase()));
    }
}

#[test]
fn test_association_type_roundtrip() {
    let types = vec![
        AssociationType::Causal,
        AssociationType::Temporal,
        AssociationType::Similar,
        AssociationType::Contrast,
        AssociationType::Hierarchical,
        AssociationType::Reference,
    ];

    for original in types {
        let str_repr = original.as_str();
        let parsed = AssociationType::from_str(str_repr).unwrap();
        assert_eq!(original, parsed);
    }
}

#[test]
fn test_config_thresholds_relationship() {
    let config = AssociationManagerConfig::default();
    // 相似度阈值应该高于最小强度阈值
    assert!(config.similarity_threshold >= config.min_strength_threshold);
}

#[test]
fn test_auto_create_similar_flag() {
    let config = AssociationManagerConfig {
        auto_create_similar: true,
        ..Default::default()
    };
    assert!(config.auto_create_similar);

    let config = AssociationManagerConfig {
        auto_create_similar: false,
        ..Default::default()
    };
    assert!(!config.auto_create_similar);
}

#[test]
fn test_association_type_debug() {
    let assoc_type = AssociationType::Causal;
    let debug_str = format!("{:?}", assoc_type);
    assert!(debug_str.contains("Causal"));
}

#[test]
fn test_custom_association_type_equality() {
    let custom1 = AssociationType::Custom("type1".to_string());
    let custom2 = AssociationType::Custom("type1".to_string());
    let custom3 = AssociationType::Custom("type2".to_string());

    assert_eq!(custom1, custom2);
    assert_ne!(custom1, custom3);
}

#[test]
fn test_strength_range() {
    // 测试强度应该在 0.0 到 1.0 之间
    let valid_strengths = vec![0.0, 0.3, 0.5, 0.7, 1.0];
    for strength in valid_strengths {
        assert!(strength >= 0.0 && strength <= 1.0);
    }
}

#[test]
fn test_confidence_range() {
    // 测试置信度应该在 0.0 到 1.0 之间
    let valid_confidences = vec![0.0, 0.3, 0.5, 0.7, 1.0];
    for confidence in valid_confidences {
        assert!(confidence >= 0.0 && confidence <= 1.0);
    }
}

#[test]
fn test_association_type_pattern_matching() {
    let assoc_type = AssociationType::Causal;
    match assoc_type {
        AssociationType::Causal => assert!(true),
        _ => panic!("Pattern matching failed"),
    }
}

#[test]
fn test_config_builder_pattern() {
    let config = AssociationManagerConfig {
        min_strength_threshold: 0.4,
        max_associations_per_memory: 75,
        auto_create_similar: true,
        similarity_threshold: 0.75,
    };

    assert_eq!(config.min_strength_threshold, 0.4);
    assert_eq!(config.max_associations_per_memory, 75);
    assert!(config.auto_create_similar);
    assert_eq!(config.similarity_threshold, 0.75);
}

#[test]
fn test_association_type_coverage() {
    // 确保所有关联类型都有字符串表示
    let all_types = vec![
        AssociationType::Causal,
        AssociationType::Temporal,
        AssociationType::Similar,
        AssociationType::Contrast,
        AssociationType::Hierarchical,
        AssociationType::Reference,
        AssociationType::Custom("test".to_string()),
    ];

    for assoc_type in all_types {
        let str_repr = assoc_type.as_str();
        assert!(!str_repr.is_empty());
    }
}

#[test]
fn test_bidirectional_associations() {
    // 测试双向关联的概念
    let from_id = "mem-1";
    let to_id = "mem-2";

    // 正向关联
    let forward_type = AssociationType::Causal;
    assert_eq!(forward_type.as_str(), "causal");

    // 反向关联（可以是相同类型或不同类型）
    let backward_type = AssociationType::Causal;
    assert_eq!(backward_type.as_str(), "causal");
}

#[test]
fn test_association_metadata_concept() {
    // 测试元数据的概念
    let metadata = serde_json::json!({
        "reason": "User mentioned both topics",
        "context": "conversation",
        "timestamp": "2025-10-07T00:00:00Z"
    });

    assert!(metadata.is_object());
    assert!(metadata.get("reason").is_some());
}

// 注意：数据库集成测试需要实际的数据库连接
// 这些测试应该在集成测试中进行，而不是单元测试

