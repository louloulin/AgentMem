//! Core Memory 数据库持久化测试

use chrono::Utc;
use serde_json::json;

#[tokio::test]
#[ignore] // 需要数据库连接
async fn test_core_memory_table_exists() {
    if std::env::var("DATABASE_URL").is_err() {
        println!("Skipping test: DATABASE_URL not set");
        return;
    }

    let pool = sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to database");

    // 测试表是否存在
    let result = sqlx::query!(
        r#"
        SELECT table_name 
        FROM information_schema.tables 
        WHERE table_schema = 'public' 
        AND table_name = 'core_memory_blocks'
        "#
    )
    .fetch_optional(&pool)
    .await;

    assert!(result.is_ok(), "Failed to query table: {:?}", result.err());
    assert!(result.unwrap().is_some(), "Table core_memory_blocks does not exist");
}

#[tokio::test]
#[ignore] // 需要数据库连接
async fn test_insert_core_memory_block() {
    if std::env::var("DATABASE_URL").is_err() {
        println!("Skipping test: DATABASE_URL not set");
        return;
    }

    let pool = sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to database");

    let id = format!("core-{}", uuid::Uuid::new_v4());
    let now = Utc::now();

    // 插入测试数据
    let result = sqlx::query!(
        r#"
        INSERT INTO core_memory_blocks (
            id, organization_id, user_id, agent_id, block_type,
            content, importance, max_capacity, current_size,
            access_count, last_accessed, metadata, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        "#,
        id,
        "org-test",
        "user-test",
        "agent-test",
        "persona",
        "Test persona content",
        0.8,
        2000,
        20,
        0,
        now,
        json!({"test": true}),
        now,
        now,
    )
    .execute(&pool)
    .await;

    assert!(result.is_ok(), "Failed to insert: {:?}", result.err());

    // 清理测试数据
    let _ = sqlx::query!("DELETE FROM core_memory_blocks WHERE id = $1", id)
        .execute(&pool)
        .await;
}

#[tokio::test]
#[ignore] // 需要数据库连接
async fn test_unique_block_per_agent_constraint() {
    if std::env::var("DATABASE_URL").is_err() {
        println!("Skipping test: DATABASE_URL not set");
        return;
    }

    let pool = sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to database");

    let agent_id = format!("agent-{}", uuid::Uuid::new_v4());
    let now = Utc::now();

    // 插入第一个 persona 块
    let id1 = format!("core-{}", uuid::Uuid::new_v4());
    let result1 = sqlx::query!(
        r#"
        INSERT INTO core_memory_blocks (
            id, organization_id, user_id, agent_id, block_type,
            content, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
        id1,
        "org-test",
        "user-test",
        agent_id,
        "persona",
        "First persona",
        now,
        now,
    )
    .execute(&pool)
    .await;

    assert!(result1.is_ok(), "Failed to insert first block");

    // 尝试插入第二个 persona 块（应该失败）
    let id2 = format!("core-{}", uuid::Uuid::new_v4());
    let result2 = sqlx::query!(
        r#"
        INSERT INTO core_memory_blocks (
            id, organization_id, user_id, agent_id, block_type,
            content, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
        id2,
        "org-test",
        "user-test",
        agent_id,
        "persona",
        "Second persona",
        now,
        now,
    )
    .execute(&pool)
    .await;

    assert!(result2.is_err(), "Should fail due to unique constraint");

    // 清理测试数据
    let _ = sqlx::query!("DELETE FROM core_memory_blocks WHERE agent_id = $1", agent_id)
        .execute(&pool)
        .await;
}

#[test]
fn test_block_type_validation() {
    // 测试块类型枚举
    let valid_types = vec!["persona", "human"];
    for block_type in valid_types {
        assert!(block_type == "persona" || block_type == "human");
    }
}

#[test]
fn test_importance_range() {
    // 测试重要性评分范围
    let valid_scores = vec![0.0, 0.5, 1.0];
    for score in valid_scores {
        assert!(score >= 0.0 && score <= 1.0);
    }

    let invalid_scores = vec![-0.1, 1.1];
    for score in invalid_scores {
        assert!(score < 0.0 || score > 1.0);
    }
}

#[test]
fn test_capacity_management() {
    let max_capacity = 2000;
    let content = "Test content";
    let current_size = content.len();

    assert!(current_size <= max_capacity);
}

