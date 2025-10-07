//! KnowledgeGraphManager 单元测试

use agent_mem_core::managers::{EntityType, KnowledgeGraphConfig, RelationType};

#[test]
fn test_entity_type_as_str() {
    assert_eq!(EntityType::Person.as_str(), "person");
    assert_eq!(EntityType::Organization.as_str(), "organization");
    assert_eq!(EntityType::Location.as_str(), "location");
    assert_eq!(EntityType::Event.as_str(), "event");
    assert_eq!(EntityType::Concept.as_str(), "concept");
    assert_eq!(EntityType::Object.as_str(), "object");
}

#[test]
fn test_entity_type_from_str() {
    assert_eq!(
        EntityType::from_str("person").unwrap(),
        EntityType::Person
    );
    assert_eq!(
        EntityType::from_str("organization").unwrap(),
        EntityType::Organization
    );
    assert_eq!(
        EntityType::from_str("location").unwrap(),
        EntityType::Location
    );
    assert_eq!(EntityType::from_str("event").unwrap(), EntityType::Event);
    assert_eq!(
        EntityType::from_str("concept").unwrap(),
        EntityType::Concept
    );
    assert_eq!(EntityType::from_str("object").unwrap(), EntityType::Object);
}

#[test]
fn test_entity_type_custom() {
    let custom = EntityType::Custom("my_entity".to_string());
    assert_eq!(custom.as_str(), "my_entity");

    let parsed = EntityType::from_str("unknown_type").unwrap();
    match parsed {
        EntityType::Custom(s) => assert_eq!(s, "unknown_type"),
        _ => panic!("Expected Custom variant"),
    }
}

#[test]
fn test_relation_type_as_str() {
    assert_eq!(RelationType::IsA.as_str(), "is_a");
    assert_eq!(RelationType::PartOf.as_str(), "part_of");
    assert_eq!(RelationType::RelatedTo.as_str(), "related_to");
    assert_eq!(RelationType::CausedBy.as_str(), "caused_by");
    assert_eq!(RelationType::Leads.as_str(), "leads");
    assert_eq!(RelationType::SimilarTo.as_str(), "similar_to");
    assert_eq!(RelationType::OppositeOf.as_str(), "opposite_of");
    assert_eq!(RelationType::LocatedIn.as_str(), "located_in");
    assert_eq!(RelationType::WorksFor.as_str(), "works_for");
    assert_eq!(RelationType::Owns.as_str(), "owns");
}

#[test]
fn test_relation_type_custom() {
    let custom = RelationType::Custom("my_relation".to_string());
    assert_eq!(custom.as_str(), "my_relation");
}

#[test]
fn test_knowledge_graph_config_default() {
    let config = KnowledgeGraphConfig::default();

    assert_eq!(config.min_confidence, 0.5);
    assert_eq!(config.max_path_length, 5);
    assert!(config.auto_extract);
}

#[test]
fn test_knowledge_graph_config_custom() {
    let config = KnowledgeGraphConfig {
        min_confidence: 0.7,
        max_path_length: 10,
        auto_extract: false,
    };

    assert_eq!(config.min_confidence, 0.7);
    assert_eq!(config.max_path_length, 10);
    assert!(!config.auto_extract);
}

#[test]
fn test_entity_type_equality() {
    assert_eq!(EntityType::Person, EntityType::Person);
    assert_ne!(EntityType::Person, EntityType::Organization);
}

#[test]
fn test_relation_type_equality() {
    assert_eq!(RelationType::IsA, RelationType::IsA);
    assert_ne!(RelationType::IsA, RelationType::PartOf);
}

#[test]
fn test_entity_type_clone() {
    let type1 = EntityType::Person;
    let type2 = type1.clone();
    assert_eq!(type1, type2);
}

#[test]
fn test_relation_type_clone() {
    let type1 = RelationType::IsA;
    let type2 = type1.clone();
    assert_eq!(type1, type2);
}

#[test]
fn test_entity_type_serialization() {
    let entity_type = EntityType::Person;
    let serialized = serde_json::to_string(&entity_type).unwrap();
    assert!(serialized.contains("person"));
}

#[test]
fn test_relation_type_serialization() {
    let relation_type = RelationType::IsA;
    let serialized = serde_json::to_string(&relation_type).unwrap();
    assert!(serialized.contains("is_a"));
}

#[test]
fn test_confidence_threshold_validation() {
    let config = KnowledgeGraphConfig::default();
    assert!(config.min_confidence >= 0.0);
    assert!(config.min_confidence <= 1.0);
}

#[test]
fn test_max_path_length_validation() {
    let config = KnowledgeGraphConfig::default();
    assert!(config.max_path_length > 0);
}

#[test]
fn test_config_clone() {
    let config1 = KnowledgeGraphConfig::default();
    let config2 = config1.clone();

    assert_eq!(config1.min_confidence, config2.min_confidence);
    assert_eq!(config1.max_path_length, config2.max_path_length);
    assert_eq!(config1.auto_extract, config2.auto_extract);
}

#[test]
fn test_all_entity_types() {
    let types = vec![
        EntityType::Person,
        EntityType::Organization,
        EntityType::Location,
        EntityType::Event,
        EntityType::Concept,
        EntityType::Object,
    ];

    for entity_type in types {
        let str_repr = entity_type.as_str();
        assert!(!str_repr.is_empty());
        assert!(str_repr.chars().all(|c| c.is_ascii_lowercase()));
    }
}

#[test]
fn test_all_relation_types() {
    let types = vec![
        RelationType::IsA,
        RelationType::PartOf,
        RelationType::RelatedTo,
        RelationType::CausedBy,
        RelationType::Leads,
        RelationType::SimilarTo,
        RelationType::OppositeOf,
        RelationType::LocatedIn,
        RelationType::WorksFor,
        RelationType::Owns,
    ];

    for relation_type in types {
        let str_repr = relation_type.as_str();
        assert!(!str_repr.is_empty());
    }
}

#[test]
fn test_entity_type_roundtrip() {
    let types = vec![
        EntityType::Person,
        EntityType::Organization,
        EntityType::Location,
        EntityType::Event,
        EntityType::Concept,
        EntityType::Object,
    ];

    for original in types {
        let str_repr = original.as_str();
        let parsed = EntityType::from_str(str_repr).unwrap();
        assert_eq!(original, parsed);
    }
}

#[test]
fn test_auto_extract_flag() {
    let config = KnowledgeGraphConfig {
        auto_extract: true,
        ..Default::default()
    };
    assert!(config.auto_extract);

    let config = KnowledgeGraphConfig {
        auto_extract: false,
        ..Default::default()
    };
    assert!(!config.auto_extract);
}

#[test]
fn test_entity_type_debug() {
    let entity_type = EntityType::Person;
    let debug_str = format!("{:?}", entity_type);
    assert!(debug_str.contains("Person"));
}

#[test]
fn test_relation_type_debug() {
    let relation_type = RelationType::IsA;
    let debug_str = format!("{:?}", relation_type);
    assert!(debug_str.contains("IsA"));
}

#[test]
fn test_custom_entity_type_equality() {
    let custom1 = EntityType::Custom("type1".to_string());
    let custom2 = EntityType::Custom("type1".to_string());
    let custom3 = EntityType::Custom("type2".to_string());

    assert_eq!(custom1, custom2);
    assert_ne!(custom1, custom3);
}

#[test]
fn test_custom_relation_type_equality() {
    let custom1 = RelationType::Custom("type1".to_string());
    let custom2 = RelationType::Custom("type1".to_string());
    let custom3 = RelationType::Custom("type2".to_string());

    assert_eq!(custom1, custom2);
    assert_ne!(custom1, custom3);
}

#[test]
fn test_confidence_range() {
    let valid_confidences = vec![0.0, 0.3, 0.5, 0.7, 1.0];
    for confidence in valid_confidences {
        assert!(confidence >= 0.0 && confidence <= 1.0);
    }
}

#[test]
fn test_entity_type_pattern_matching() {
    let entity_type = EntityType::Person;
    match entity_type {
        EntityType::Person => assert!(true),
        _ => panic!("Pattern matching failed"),
    }
}

#[test]
fn test_relation_type_pattern_matching() {
    let relation_type = RelationType::IsA;
    match relation_type {
        RelationType::IsA => assert!(true),
        _ => panic!("Pattern matching failed"),
    }
}

#[test]
fn test_config_builder_pattern() {
    let config = KnowledgeGraphConfig {
        min_confidence: 0.6,
        max_path_length: 8,
        auto_extract: true,
    };

    assert_eq!(config.min_confidence, 0.6);
    assert_eq!(config.max_path_length, 8);
    assert!(config.auto_extract);
}

#[test]
fn test_entity_type_coverage() {
    let all_types = vec![
        EntityType::Person,
        EntityType::Organization,
        EntityType::Location,
        EntityType::Event,
        EntityType::Concept,
        EntityType::Object,
        EntityType::Custom("test".to_string()),
    ];

    for entity_type in all_types {
        let str_repr = entity_type.as_str();
        assert!(!str_repr.is_empty());
    }
}

#[test]
fn test_relation_type_coverage() {
    let all_types = vec![
        RelationType::IsA,
        RelationType::PartOf,
        RelationType::RelatedTo,
        RelationType::CausedBy,
        RelationType::Leads,
        RelationType::SimilarTo,
        RelationType::OppositeOf,
        RelationType::LocatedIn,
        RelationType::WorksFor,
        RelationType::Owns,
        RelationType::Custom("test".to_string()),
    ];

    for relation_type in all_types {
        let str_repr = relation_type.as_str();
        assert!(!str_repr.is_empty());
    }
}

// 注意：数据库集成测试需要实际的数据库连接
// 这些测试应该在集成测试中进行，而不是单元测试

