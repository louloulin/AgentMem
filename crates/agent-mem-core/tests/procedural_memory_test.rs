//! 程序记忆管理器测试

#![cfg(feature = "postgres")]

use agent_mem_core::managers::{ProceduralMemoryItem, ProceduralMemoryManager, ProceduralQuery};
use chrono::Utc;
use serde_json::json;

/// 创建测试用的程序记忆项
fn create_test_item(entry_type: &str, summary: &str) -> ProceduralMemoryItem {
    ProceduralMemoryItem {
        id: format!("proc-{}", uuid::Uuid::new_v4()),
        organization_id: "org-test".to_string(),
        user_id: "user-test".to_string(),
        agent_id: "agent-test".to_string(),
        entry_type: entry_type.to_string(),
        summary: summary.to_string(),
        steps: vec![
            "Step 1: Initialize".to_string(),
            "Step 2: Process".to_string(),
            "Step 3: Finalize".to_string(),
        ],
        tree_path: vec!["workflows".to_string(), "development".to_string()],
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
async fn test_create_procedural_item() {
    if std::env::var("DATABASE_URL").is_err() {
        println!("Skipping test: DATABASE_URL not set");
        return;
    }

    let pool = sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to database");

    let manager = ProceduralMemoryManager::new(std::sync::Arc::new(pool));

    let item = create_test_item("workflow", "Test workflow");

    let result = manager.create_item(item.clone()).await;
    assert!(result.is_ok(), "Failed to create item: {:?}", result.err());

    let created_item = result.unwrap();
    assert_eq!(created_item.id, item.id);
    assert_eq!(created_item.summary, item.summary);
    assert_eq!(created_item.steps.len(), 3);
}

#[tokio::test]
#[ignore] // 需要数据库连接
async fn test_query_procedural_items() {
    if std::env::var("DATABASE_URL").is_err() {
        println!("Skipping test: DATABASE_URL not set");
        return;
    }

    let pool = sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to database");

    let manager = ProceduralMemoryManager::new(std::sync::Arc::new(pool));

    // 创建测试项
    let item1 = create_test_item("workflow", "Deployment workflow");
    let item2 = create_test_item("guide", "Testing guide");
    let item3 = create_test_item("script", "Build script");

    let _ = manager.create_item(item1).await;
    let _ = manager.create_item(item2).await;
    let _ = manager.create_item(item3).await;

    // 查询所有项
    let query = ProceduralQuery {
        entry_type: None,
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
async fn test_get_items_by_type() {
    if std::env::var("DATABASE_URL").is_err() {
        println!("Skipping test: DATABASE_URL not set");
        return;
    }

    let pool = sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to database");

    let manager = ProceduralMemoryManager::new(std::sync::Arc::new(pool));

    // 获取特定类型的项
    let result = manager.get_items_by_type("user-test", "workflow", 10).await;
    assert!(
        result.is_ok(),
        "Failed to get items by type: {:?}",
        result.err()
    );

    let items = result.unwrap();
    for item in items {
        assert_eq!(item.entry_type, "workflow");
    }
}

#[tokio::test]
#[ignore] // 需要数据库连接
async fn test_search_by_summary() {
    if std::env::var("DATABASE_URL").is_err() {
        println!("Skipping test: DATABASE_URL not set");
        return;
    }

    let pool = sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to database");

    let manager = ProceduralMemoryManager::new(std::sync::Arc::new(pool));

    // 搜索包含 "workflow" 的项
    let result = manager.search_by_summary("user-test", "workflow", 10).await;
    assert!(result.is_ok(), "Failed to search items: {:?}", result.err());

    let items = result.unwrap();
    for item in items {
        assert!(item.summary.to_lowercase().contains("workflow"));
    }
}

#[tokio::test]
#[ignore] // 需要数据库连接
async fn test_update_procedural_item() {
    if std::env::var("DATABASE_URL").is_err() {
        println!("Skipping test: DATABASE_URL not set");
        return;
    }

    let pool = sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to database");

    let manager = ProceduralMemoryManager::new(std::sync::Arc::new(pool));

    let mut item = create_test_item("workflow", "Original workflow");
    let created = manager.create_item(item.clone()).await.unwrap();

    // 更新项
    item.summary = "Updated workflow".to_string();
    item.steps.push("Step 4: Verify".to_string());
    let result = manager.update_item(item.clone()).await;
    assert!(result.is_ok(), "Failed to update item: {:?}", result.err());
    assert!(result.unwrap(), "Should have updated the item");

    // 验证更新
    let updated = manager.get_item(&created.id, "user-test").await.unwrap();
    assert!(updated.is_some());
    let updated_item = updated.unwrap();
    assert_eq!(updated_item.summary, "Updated workflow");
    assert_eq!(updated_item.steps.len(), 4);
}

#[tokio::test]
#[ignore] // 需要数据库连接
async fn test_delete_procedural_item() {
    if std::env::var("DATABASE_URL").is_err() {
        println!("Skipping test: DATABASE_URL not set");
        return;
    }

    let pool = sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to database");

    let manager = ProceduralMemoryManager::new(std::sync::Arc::new(pool));

    let item = create_test_item("workflow", "To be deleted");
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
fn test_procedural_item_serialization() {
    let item = create_test_item("workflow", "Test workflow");

    // 测试序列化
    let json = serde_json::to_string(&item).expect("Failed to serialize");
    assert!(json.contains("Test workflow"));

    // 测试反序列化
    let deserialized: ProceduralMemoryItem =
        serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(deserialized.summary, item.summary);
    assert_eq!(deserialized.entry_type, item.entry_type);
    assert_eq!(deserialized.steps.len(), item.steps.len());
}

#[test]
fn test_procedural_query_builder() {
    let query = ProceduralQuery {
        entry_type: Some("workflow".to_string()),
        summary_query: Some("deployment".to_string()),
        tree_path_prefix: Some(vec!["workflows".to_string(), "development".to_string()]),
        limit: Some(10),
    };

    // 测试序列化
    let json = serde_json::to_string(&query).expect("Failed to serialize");
    assert!(json.contains("workflow"));

    // 测试反序列化
    let deserialized: ProceduralQuery = serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(deserialized.entry_type, query.entry_type);
    assert_eq!(deserialized.limit, query.limit);
}

#[test]
fn test_steps_structure() {
    let item = create_test_item("workflow", "Test");

    // 验证步骤列表
    assert_eq!(item.steps.len(), 3);
    assert_eq!(item.steps[0], "Step 1: Initialize");
    assert_eq!(item.steps[1], "Step 2: Process");
    assert_eq!(item.steps[2], "Step 3: Finalize");
}

#[test]
fn test_entry_types() {
    let workflow = create_test_item("workflow", "Test workflow");
    let guide = create_test_item("guide", "Test guide");
    let script = create_test_item("script", "Test script");

    assert_eq!(workflow.entry_type, "workflow");
    assert_eq!(guide.entry_type, "guide");
    assert_eq!(script.entry_type, "script");
}

#[test]
fn test_tree_path_structure() {
    let item = create_test_item("workflow", "Test");

    // 验证层级路径
    assert_eq!(item.tree_path.len(), 2);
    assert_eq!(item.tree_path[0], "workflows");
    assert_eq!(item.tree_path[1], "development");
}

#[test]
fn test_metadata_structure() {
    let item = create_test_item("workflow", "Test");

    // 验证元数据
    assert!(item.metadata.is_object());
    assert_eq!(item.metadata["test"], true);
    assert_eq!(item.metadata["category"], "test");
}
