//! 情景记忆管理器测试

use agent_mem_core::managers::{EpisodicEvent, EpisodicMemoryManager, EpisodicQuery};
use chrono::Utc;
use serde_json::json;

/// 创建测试用的情景事件
fn create_test_event(event_type: &str, summary: &str) -> EpisodicEvent {
    EpisodicEvent {
        id: format!("ep-{}", uuid::Uuid::new_v4()),
        organization_id: "org-test".to_string(),
        user_id: "user-test".to_string(),
        agent_id: "agent-test".to_string(),
        occurred_at: Utc::now(),
        event_type: event_type.to_string(),
        actor: Some("test-actor".to_string()),
        summary: summary.to_string(),
        details: Some("Test event details".to_string()),
        importance_score: 0.7,
        metadata: json!({
            "test": true,
            "category": "test"
        }),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

#[tokio::test]
#[ignore] // 需要数据库连接
async fn test_create_episodic_event() {
    // 这个测试需要真实的数据库连接
    // 在 CI/CD 环境中应该设置 DATABASE_URL
    
    if std::env::var("DATABASE_URL").is_err() {
        println!("Skipping test: DATABASE_URL not set");
        return;
    }

    let pool = sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to database");

    let manager = EpisodicMemoryManager::new(std::sync::Arc::new(pool));

    let event = create_test_event("conversation", "Test conversation event");

    let result = manager.create_event(event.clone()).await;
    assert!(result.is_ok(), "Failed to create event: {:?}", result.err());

    let created_event = result.unwrap();
    assert_eq!(created_event.id, event.id);
    assert_eq!(created_event.summary, event.summary);
}

#[tokio::test]
#[ignore] // 需要数据库连接
async fn test_query_episodic_events() {
    if std::env::var("DATABASE_URL").is_err() {
        println!("Skipping test: DATABASE_URL not set");
        return;
    }

    let pool = sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to database");

    let manager = EpisodicMemoryManager::new(std::sync::Arc::new(pool));

    // 创建测试事件
    let event1 = create_test_event("conversation", "First conversation");
    let event2 = create_test_event("action", "User action");
    let event3 = create_test_event("observation", "System observation");

    let _ = manager.create_event(event1).await;
    let _ = manager.create_event(event2).await;
    let _ = manager.create_event(event3).await;

    // 查询所有事件
    let query = EpisodicQuery {
        start_time: None,
        end_time: None,
        event_type: None,
        min_importance: None,
        limit: Some(10),
    };

    let result = manager.query_events("user-test", query).await;
    assert!(result.is_ok(), "Failed to query events: {:?}", result.err());

    let events = result.unwrap();
    assert!(events.len() >= 3, "Should have at least 3 events");
}

#[tokio::test]
#[ignore] // 需要数据库连接
async fn test_query_by_event_type() {
    if std::env::var("DATABASE_URL").is_err() {
        println!("Skipping test: DATABASE_URL not set");
        return;
    }

    let pool = sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to database");

    let manager = EpisodicMemoryManager::new(std::sync::Arc::new(pool));

    // 查询特定类型的事件
    let query = EpisodicQuery {
        start_time: None,
        end_time: None,
        event_type: Some("conversation".to_string()),
        min_importance: None,
        limit: Some(10),
    };

    let result = manager.query_events("user-test", query).await;
    assert!(result.is_ok(), "Failed to query events: {:?}", result.err());

    let events = result.unwrap();
    for event in events {
        assert_eq!(event.event_type, "conversation");
    }
}

#[tokio::test]
#[ignore] // 需要数据库连接
async fn test_update_importance() {
    if std::env::var("DATABASE_URL").is_err() {
        println!("Skipping test: DATABASE_URL not set");
        return;
    }

    let pool = sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to database");

    let manager = EpisodicMemoryManager::new(std::sync::Arc::new(pool));

    let event = create_test_event("conversation", "Important conversation");
    let created = manager.create_event(event.clone()).await.unwrap();

    // 更新重要性评分
    let result = manager
        .update_importance(&created.id, "user-test", 0.95)
        .await;
    assert!(result.is_ok(), "Failed to update importance: {:?}", result.err());
    assert!(result.unwrap(), "Should have updated the event");

    // 验证更新
    let updated = manager.get_event(&created.id, "user-test").await.unwrap();
    assert!(updated.is_some());
    assert_eq!(updated.unwrap().importance_score, 0.95);
}

#[tokio::test]
#[ignore] // 需要数据库连接
async fn test_delete_event() {
    if std::env::var("DATABASE_URL").is_err() {
        println!("Skipping test: DATABASE_URL not set");
        return;
    }

    let pool = sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to database");

    let manager = EpisodicMemoryManager::new(std::sync::Arc::new(pool));

    let event = create_test_event("conversation", "To be deleted");
    let created = manager.create_event(event.clone()).await.unwrap();

    // 删除事件
    let result = manager.delete_event(&created.id, "user-test").await;
    assert!(result.is_ok(), "Failed to delete event: {:?}", result.err());
    assert!(result.unwrap(), "Should have deleted the event");

    // 验证删除
    let deleted = manager.get_event(&created.id, "user-test").await.unwrap();
    assert!(deleted.is_none(), "Event should be deleted");
}

#[test]
fn test_episodic_event_serialization() {
    let event = create_test_event("conversation", "Test event");

    // 测试序列化
    let json = serde_json::to_string(&event).expect("Failed to serialize");
    assert!(json.contains("Test event"));

    // 测试反序列化
    let deserialized: EpisodicEvent =
        serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(deserialized.summary, event.summary);
    assert_eq!(deserialized.event_type, event.event_type);
}

#[test]
fn test_episodic_query_builder() {
    let query = EpisodicQuery {
        start_time: Some(Utc::now()),
        end_time: Some(Utc::now()),
        event_type: Some("conversation".to_string()),
        min_importance: Some(0.5),
        limit: Some(10),
    };

    // 测试序列化
    let json = serde_json::to_string(&query).expect("Failed to serialize");
    assert!(json.contains("conversation"));

    // 测试反序列化
    let deserialized: EpisodicQuery =
        serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(deserialized.event_type, query.event_type);
    assert_eq!(deserialized.limit, query.limit);
}

#[test]
fn test_importance_score_validation() {
    let mut event = create_test_event("conversation", "Test event");

    // 测试有效的重要性评分
    event.importance_score = 0.0;
    assert!(event.importance_score >= 0.0 && event.importance_score <= 1.0);

    event.importance_score = 0.5;
    assert!(event.importance_score >= 0.0 && event.importance_score <= 1.0);

    event.importance_score = 1.0;
    assert!(event.importance_score >= 0.0 && event.importance_score <= 1.0);
}

#[test]
fn test_event_metadata() {
    let event = create_test_event("conversation", "Test event");

    // 验证元数据
    assert!(event.metadata.is_object());
    assert_eq!(event.metadata["test"], true);
    assert_eq!(event.metadata["category"], "test");
}

