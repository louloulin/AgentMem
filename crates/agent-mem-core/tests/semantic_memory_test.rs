//! 语义记忆管理器测试

use agent_mem_core::managers::{SemanticMemoryItem, SemanticMemoryManager, SemanticQuery};
use chrono::Utc;
use serde_json::json;

/// 创建测试用的语义记忆项
fn create_test_item(name: &str, summary: &str) -> SemanticMemoryItem {
    SemanticMemoryItem {
        id: format!("sem-{}", uuid::Uuid::new_v4()),
        organization_id: "org-test".to_string(),
        user_id: "user-test".to_string(),
        agent_id: "agent-test".to_string(),
        name: name.to_string(),
        summary: summary.to_string(),
        details: "Test semantic memory details".to_string(),
        source: Some("test-source".to_string()),
        tree_path: vec!["test".to_string(), "concepts".to_string()],
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
async fn test_create_semantic_item() {
    if std::env::var("DATABASE_URL").is_err() {
        println!("Skipping test: DATABASE_URL not set");
        return;
    }

    let pool = sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to database");

    let manager = SemanticMemoryManager::new(std::sync::Arc::new(pool));

    let item = create_test_item("Machine Learning", "A subset of AI");

    let result = manager.create_item(item.clone()).await;
    assert!(result.is_ok(), "Failed to create item: {:?}", result.err());

    let created_item = result.unwrap();
    assert_eq!(created_item.id, item.id);
    assert_eq!(created_item.name, item.name);
}

#[tokio::test]
#[ignore] // 需要数据库连接
async fn test_query_semantic_items() {
    if std::env::var("DATABASE_URL").is_err() {
        println!("Skipping test: DATABASE_URL not set");
        return;
    }

    let pool = sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to database");

    let manager = SemanticMemoryManager::new(std::sync::Arc::new(pool));

    // 创建测试项
    let item1 = create_test_item("Deep Learning", "Neural networks with multiple layers");
    let item2 = create_test_item("Natural Language Processing", "AI for understanding text");
    let item3 = create_test_item("Computer Vision", "AI for understanding images");

    let _ = manager.create_item(item1).await;
    let _ = manager.create_item(item2).await;
    let _ = manager.create_item(item3).await;

    // 查询所有项
    let query = SemanticQuery {
        name_query: None,
        summary_query: None,
        tree_path_prefix: None,
        limit: Some(10),
    };

    let result = manager.query_items("user-test", query).await;
    assert!(result.is_ok(), "Failed to query items: {:?}", result.err());

    let items = result.unwrap();
    assert!(items.len() >= 3, "Should have at least 3 items");
}

#[tokio::test]
#[ignore] // 需要数据库连接
async fn test_search_by_name() {
    if std::env::var("DATABASE_URL").is_err() {
        println!("Skipping test: DATABASE_URL not set");
        return;
    }

    let pool = sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to database");

    let manager = SemanticMemoryManager::new(std::sync::Arc::new(pool));

    // 搜索包含 "Learning" 的项
    let result = manager.search_by_name("user-test", "Learning", 10).await;
    assert!(result.is_ok(), "Failed to search items: {:?}", result.err());

    let items = result.unwrap();
    for item in items {
        assert!(item.name.contains("Learning"));
    }
}

#[tokio::test]
#[ignore] // 需要数据库连接
async fn test_get_items_by_tree_path() {
    if std::env::var("DATABASE_URL").is_err() {
        println!("Skipping test: DATABASE_URL not set");
        return;
    }

    let pool = sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to database");

    let manager = SemanticMemoryManager::new(std::sync::Arc::new(pool));

    // 获取特定层级路径下的项
    let tree_path = vec!["test".to_string(), "concepts".to_string()];
    let result = manager.get_items_by_tree_path("user-test", &tree_path).await;
    assert!(result.is_ok(), "Failed to get items by tree path: {:?}", result.err());

    let items = result.unwrap();
    for item in items {
        assert!(item.tree_path.starts_with(&tree_path));
    }
}

#[tokio::test]
#[ignore] // 需要数据库连接
async fn test_update_semantic_item() {
    if std::env::var("DATABASE_URL").is_err() {
        println!("Skipping test: DATABASE_URL not set");
        return;
    }

    let pool = sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to database");

    let manager = SemanticMemoryManager::new(std::sync::Arc::new(pool));

    let mut item = create_test_item("Test Concept", "Original summary");
    let created = manager.create_item(item.clone()).await.unwrap();

    // 更新项
    item.summary = "Updated summary".to_string();
    let result = manager.update_item(item.clone()).await;
    assert!(result.is_ok(), "Failed to update item: {:?}", result.err());
    assert!(result.unwrap(), "Should have updated the item");

    // 验证更新
    let updated = manager.get_item(&created.id, "user-test").await.unwrap();
    assert!(updated.is_some());
    assert_eq!(updated.unwrap().summary, "Updated summary");
}

#[tokio::test]
#[ignore] // 需要数据库连接
async fn test_delete_semantic_item() {
    if std::env::var("DATABASE_URL").is_err() {
        println!("Skipping test: DATABASE_URL not set");
        return;
    }

    let pool = sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to database");

    let manager = SemanticMemoryManager::new(std::sync::Arc::new(pool));

    let item = create_test_item("To be deleted", "This will be deleted");
    let created = manager.create_item(item.clone()).await.unwrap();

    // 删除项
    let result = manager.delete_item(&created.id, "user-test").await;
    assert!(result.is_ok(), "Failed to delete item: {:?}", result.err());
    assert!(result.unwrap(), "Should have deleted the item");

    // 验证删除
    let deleted = manager.get_item(&created.id, "user-test").await.unwrap();
    assert!(deleted.is_none(), "Item should be deleted");
}

#[test]
fn test_semantic_item_serialization() {
    let item = create_test_item("Test Concept", "Test summary");

    // 测试序列化
    let json = serde_json::to_string(&item).expect("Failed to serialize");
    assert!(json.contains("Test Concept"));

    // 测试反序列化
    let deserialized: SemanticMemoryItem =
        serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(deserialized.name, item.name);
    assert_eq!(deserialized.summary, item.summary);
}

#[test]
fn test_semantic_query_builder() {
    let query = SemanticQuery {
        name_query: Some("Machine Learning".to_string()),
        summary_query: Some("AI".to_string()),
        tree_path_prefix: Some(vec!["test".to_string(), "concepts".to_string()]),
        limit: Some(10),
    };

    // 测试序列化
    let json = serde_json::to_string(&query).expect("Failed to serialize");
    assert!(json.contains("Machine Learning"));

    // 测试反序列化
    let deserialized: SemanticQuery =
        serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(deserialized.name_query, query.name_query);
    assert_eq!(deserialized.limit, query.limit);
}

#[test]
fn test_tree_path_structure() {
    let item = create_test_item("Test", "Test");

    // 验证层级路径
    assert_eq!(item.tree_path.len(), 2);
    assert_eq!(item.tree_path[0], "test");
    assert_eq!(item.tree_path[1], "concepts");
}

#[test]
fn test_metadata_structure() {
    let item = create_test_item("Test", "Test");

    // 验证元数据
    assert!(item.metadata.is_object());
    assert_eq!(item.metadata["test"], true);
    assert_eq!(item.metadata["category"], "test");
}

#[test]
fn test_source_field() {
    let item = create_test_item("Test", "Test");

    // 验证来源字段
    assert!(item.source.is_some());
    assert_eq!(item.source.unwrap(), "test-source");
}

