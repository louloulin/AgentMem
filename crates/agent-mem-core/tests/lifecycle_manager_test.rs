//! LifecycleManager 单元测试

use agent_mem_core::managers::{
    LifecycleEventType, LifecycleManager, LifecycleManagerConfig, MemoryState,
};
use chrono::Utc;

#[test]
fn test_memory_state_as_str() {
    assert_eq!(MemoryState::Created.as_str(), "created");
    assert_eq!(MemoryState::Active.as_str(), "active");
    assert_eq!(MemoryState::Archived.as_str(), "archived");
    assert_eq!(MemoryState::Deprecated.as_str(), "deprecated");
    assert_eq!(MemoryState::Deleted.as_str(), "deleted");
}

#[test]
fn test_memory_state_from_str() {
    assert_eq!(
        MemoryState::from_str("created").unwrap(),
        MemoryState::Created
    );
    assert_eq!(
        MemoryState::from_str("active").unwrap(),
        MemoryState::Active
    );
    assert_eq!(
        MemoryState::from_str("archived").unwrap(),
        MemoryState::Archived
    );
    assert_eq!(
        MemoryState::from_str("deprecated").unwrap(),
        MemoryState::Deprecated
    );
    assert_eq!(
        MemoryState::from_str("deleted").unwrap(),
        MemoryState::Deleted
    );
}

#[test]
fn test_memory_state_from_str_invalid() {
    assert!(MemoryState::from_str("invalid").is_err());
}

#[test]
fn test_lifecycle_event_type_as_str() {
    assert_eq!(LifecycleEventType::Created.as_str(), "created");
    assert_eq!(LifecycleEventType::Accessed.as_str(), "accessed");
    assert_eq!(LifecycleEventType::Updated.as_str(), "updated");
    assert_eq!(LifecycleEventType::Archived.as_str(), "archived");
    assert_eq!(LifecycleEventType::Restored.as_str(), "restored");
    assert_eq!(LifecycleEventType::Deprecated.as_str(), "deprecated");
    assert_eq!(LifecycleEventType::Deleted.as_str(), "deleted");
    assert_eq!(
        LifecycleEventType::ImportanceChanged.as_str(),
        "importance_changed"
    );
    assert_eq!(
        LifecycleEventType::ExpirationSet.as_str(),
        "expiration_set"
    );
}

#[test]
fn test_lifecycle_manager_config_default() {
    let config = LifecycleManagerConfig::default();

    assert_eq!(config.auto_archive_age, Some(30 * 24 * 3600)); // 30 days
    assert_eq!(config.auto_delete_age, Some(365 * 24 * 3600)); // 1 year
    assert_eq!(config.archive_importance_threshold, 0.3);
    assert_eq!(config.delete_importance_threshold, 0.1);
    assert_eq!(config.working_memory_ttl, 24 * 3600); // 1 day
}

#[test]
fn test_lifecycle_manager_config_custom() {
    let config = LifecycleManagerConfig {
        auto_archive_age: Some(7 * 24 * 3600), // 7 days
        auto_delete_age: Some(90 * 24 * 3600), // 90 days
        archive_importance_threshold: 0.5,
        delete_importance_threshold: 0.2,
        working_memory_ttl: 12 * 3600, // 12 hours
    };

    assert_eq!(config.auto_archive_age, Some(7 * 24 * 3600));
    assert_eq!(config.auto_delete_age, Some(90 * 24 * 3600));
    assert_eq!(config.archive_importance_threshold, 0.5);
    assert_eq!(config.delete_importance_threshold, 0.2);
    assert_eq!(config.working_memory_ttl, 12 * 3600);
}

#[test]
fn test_memory_state_transitions() {
    // 测试状态转换逻辑
    let states = vec![
        MemoryState::Created,
        MemoryState::Active,
        MemoryState::Archived,
        MemoryState::Deprecated,
        MemoryState::Deleted,
    ];

    for state in states {
        let state_str = state.as_str();
        let parsed_state = MemoryState::from_str(state_str).unwrap();
        assert_eq!(state, parsed_state);
    }
}

#[test]
fn test_lifecycle_event_types() {
    let event_types = vec![
        LifecycleEventType::Created,
        LifecycleEventType::Accessed,
        LifecycleEventType::Updated,
        LifecycleEventType::Archived,
        LifecycleEventType::Restored,
        LifecycleEventType::Deprecated,
        LifecycleEventType::Deleted,
        LifecycleEventType::ImportanceChanged,
        LifecycleEventType::ExpirationSet,
    ];

    for event_type in event_types {
        let event_str = event_type.as_str();
        assert!(!event_str.is_empty());
    }
}

#[test]
fn test_archive_age_calculation() {
    let config = LifecycleManagerConfig::default();
    let archive_age = config.auto_archive_age.unwrap();

    // 30 天 = 30 * 24 * 3600 秒
    assert_eq!(archive_age, 2_592_000);
}

#[test]
fn test_delete_age_calculation() {
    let config = LifecycleManagerConfig::default();
    let delete_age = config.auto_delete_age.unwrap();

    // 1 年 = 365 * 24 * 3600 秒
    assert_eq!(delete_age, 31_536_000);
}

#[test]
fn test_importance_thresholds() {
    let config = LifecycleManagerConfig::default();

    assert!(config.archive_importance_threshold > 0.0);
    assert!(config.archive_importance_threshold < 1.0);
    assert!(config.delete_importance_threshold > 0.0);
    assert!(config.delete_importance_threshold < 1.0);
    assert!(config.delete_importance_threshold < config.archive_importance_threshold);
}

#[test]
fn test_working_memory_ttl() {
    let config = LifecycleManagerConfig::default();
    let ttl = config.working_memory_ttl;

    // 1 天 = 24 * 3600 秒
    assert_eq!(ttl, 86_400);
}

#[test]
fn test_memory_state_equality() {
    assert_eq!(MemoryState::Created, MemoryState::Created);
    assert_ne!(MemoryState::Created, MemoryState::Active);
    assert_ne!(MemoryState::Active, MemoryState::Archived);
}

#[test]
fn test_lifecycle_event_type_equality() {
    assert_eq!(LifecycleEventType::Created, LifecycleEventType::Created);
    assert_ne!(LifecycleEventType::Created, LifecycleEventType::Accessed);
}

#[test]
fn test_config_clone() {
    let config1 = LifecycleManagerConfig::default();
    let config2 = config1.clone();

    assert_eq!(config1.auto_archive_age, config2.auto_archive_age);
    assert_eq!(config1.auto_delete_age, config2.auto_delete_age);
    assert_eq!(
        config1.archive_importance_threshold,
        config2.archive_importance_threshold
    );
    assert_eq!(
        config1.delete_importance_threshold,
        config2.delete_importance_threshold
    );
    assert_eq!(config1.working_memory_ttl, config2.working_memory_ttl);
}

#[test]
fn test_memory_state_serialization() {
    let state = MemoryState::Active;
    let serialized = serde_json::to_string(&state).unwrap();
    assert!(serialized.contains("active"));
}

#[test]
fn test_lifecycle_event_type_serialization() {
    let event_type = LifecycleEventType::Created;
    let serialized = serde_json::to_string(&event_type).unwrap();
    assert!(serialized.contains("created"));
}

#[test]
fn test_config_validation() {
    let config = LifecycleManagerConfig {
        auto_archive_age: Some(7 * 24 * 3600),
        auto_delete_age: Some(30 * 24 * 3600),
        archive_importance_threshold: 0.3,
        delete_importance_threshold: 0.1,
        working_memory_ttl: 24 * 3600,
    };

    // 验证删除年龄大于归档年龄
    assert!(config.auto_delete_age.unwrap() > config.auto_archive_age.unwrap());

    // 验证删除阈值小于归档阈值
    assert!(config.delete_importance_threshold < config.archive_importance_threshold);
}

#[test]
fn test_optional_ages() {
    let config = LifecycleManagerConfig {
        auto_archive_age: None,
        auto_delete_age: None,
        archive_importance_threshold: 0.3,
        delete_importance_threshold: 0.1,
        working_memory_ttl: 24 * 3600,
    };

    assert!(config.auto_archive_age.is_none());
    assert!(config.auto_delete_age.is_none());
}

#[test]
fn test_state_progression() {
    // 测试典型的状态进展
    let progression = vec![
        MemoryState::Created,
        MemoryState::Active,
        MemoryState::Archived,
        MemoryState::Deleted,
    ];

    for (i, state) in progression.iter().enumerate() {
        assert_eq!(state.as_str(), MemoryState::from_str(state.as_str()).unwrap().as_str());
        println!("State {}: {:?}", i, state);
    }
}

#[test]
fn test_event_type_coverage() {
    // 确保所有事件类型都有字符串表示
    let all_types = vec![
        LifecycleEventType::Created,
        LifecycleEventType::Accessed,
        LifecycleEventType::Updated,
        LifecycleEventType::Archived,
        LifecycleEventType::Restored,
        LifecycleEventType::Deprecated,
        LifecycleEventType::Deleted,
        LifecycleEventType::ImportanceChanged,
        LifecycleEventType::ExpirationSet,
    ];

    for event_type in all_types {
        let str_repr = event_type.as_str();
        assert!(!str_repr.is_empty());
        assert!(str_repr.chars().all(|c| c.is_ascii_lowercase() || c == '_'));
    }
}

// 注意：数据库集成测试需要实际的数据库连接
// 这些测试应该在集成测试中进行，而不是单元测试

