//! Integration tests for User Repository
//!
//! These tests verify the user repository functionality with a real database.

#![cfg(feature = "postgres")]

use agent_mem_core::storage::user_repository::{UserAuth, UserRepository};
use sqlx::PgPool;

/// Helper function to create a test database pool
async fn create_test_pool() -> PgPool {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://agentmem:password@localhost:5432/agentmem_test".to_string()
    });

    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

/// Helper function to setup test database schema
async fn setup_test_schema(pool: &PgPool) {
    // Create users table if not exists
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            organization_id TEXT NOT NULL,
            email TEXT NOT NULL,
            password_hash TEXT NOT NULL,
            name TEXT NOT NULL,
            roles TEXT[] NOT NULL DEFAULT '{}',
            status TEXT NOT NULL DEFAULT 'active',
            timezone TEXT NOT NULL DEFAULT 'UTC',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
            created_by_id TEXT,
            last_updated_by_id TEXT,
            UNIQUE(email, organization_id)
        )
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create users table");
}

/// Helper function to cleanup test data
async fn cleanup_test_data(pool: &PgPool, email: &str) {
    sqlx::query("DELETE FROM users WHERE email = $1")
        .bind(email)
        .execute(pool)
        .await
        .ok();
}

#[tokio::test]
#[ignore] // Requires database
async fn test_user_repository_create() {
    let pool = create_test_pool().await;
    setup_test_schema(&pool).await;

    let repo = UserRepository::new(pool.clone());
    let test_email = "test_create@example.com";

    // Cleanup before test
    cleanup_test_data(&pool, test_email).await;

    // Create user
    let user = repo
        .create(
            "org123",
            test_email,
            "hashed_password",
            "Test User",
            vec!["user".to_string()],
            None,
        )
        .await
        .expect("Failed to create user");

    assert_eq!(user.email, test_email);
    assert_eq!(user.name, "Test User");
    assert_eq!(user.organization_id, "org123");
    assert_eq!(user.roles, vec!["user".to_string()]);
    assert_eq!(user.status, "active");
    assert!(!user.is_deleted);

    // Cleanup after test
    cleanup_test_data(&pool, test_email).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_user_repository_find_by_email() {
    let pool = create_test_pool().await;
    setup_test_schema(&pool).await;

    let repo = UserRepository::new(pool.clone());
    let test_email = "test_find@example.com";

    // Cleanup before test
    cleanup_test_data(&pool, test_email).await;

    // Create user
    let created_user = repo
        .create(
            "org123",
            test_email,
            "hashed_password",
            "Test User",
            vec!["user".to_string()],
            None,
        )
        .await
        .expect("Failed to create user");

    // Find by email
    let found_user = repo
        .find_by_email(test_email)
        .await
        .expect("Failed to find user")
        .expect("User not found");

    assert_eq!(found_user.id, created_user.id);
    assert_eq!(found_user.email, test_email);

    // Cleanup after test
    cleanup_test_data(&pool, test_email).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_user_repository_update() {
    let pool = create_test_pool().await;
    setup_test_schema(&pool).await;

    let repo = UserRepository::new(pool.clone());
    let test_email = "test_update@example.com";

    // Cleanup before test
    cleanup_test_data(&pool, test_email).await;

    // Create user
    let user = repo
        .create(
            "org123",
            test_email,
            "hashed_password",
            "Test User",
            vec!["user".to_string()],
            None,
        )
        .await
        .expect("Failed to create user");

    // Update user
    let updated_user = repo
        .update(
            &user.id,
            Some("Updated Name"),
            None,
            Some(vec!["admin".to_string()]),
            None,
            None,
        )
        .await
        .expect("Failed to update user");

    assert_eq!(updated_user.name, "Updated Name");
    assert_eq!(updated_user.roles, vec!["admin".to_string()]);

    // Cleanup after test
    cleanup_test_data(&pool, test_email).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_user_repository_update_password() {
    let pool = create_test_pool().await;
    setup_test_schema(&pool).await;

    let repo = UserRepository::new(pool.clone());
    let test_email = "test_password@example.com";

    // Cleanup before test
    cleanup_test_data(&pool, test_email).await;

    // Create user
    let user = repo
        .create(
            "org123",
            test_email,
            "old_password_hash",
            "Test User",
            vec!["user".to_string()],
            None,
        )
        .await
        .expect("Failed to create user");

    // Update password
    repo.update_password(&user.id, "new_password_hash", None)
        .await
        .expect("Failed to update password");

    // Verify password was updated
    let updated_user = repo
        .find_by_id(&user.id)
        .await
        .expect("Failed to find user")
        .expect("User not found");

    assert_eq!(updated_user.password_hash, "new_password_hash");

    // Cleanup after test
    cleanup_test_data(&pool, test_email).await;
}

#[tokio::test]
#[ignore] // Requires database
async fn test_user_repository_soft_delete() {
    let pool = create_test_pool().await;
    setup_test_schema(&pool).await;

    let repo = UserRepository::new(pool.clone());
    let test_email = "test_delete@example.com";

    // Cleanup before test
    cleanup_test_data(&pool, test_email).await;

    // Create user
    let user = repo
        .create(
            "org123",
            test_email,
            "hashed_password",
            "Test User",
            vec!["user".to_string()],
            None,
        )
        .await
        .expect("Failed to create user");

    // Soft delete
    repo.delete(&user.id, None)
        .await
        .expect("Failed to delete user");

    // Verify user is soft deleted (not found in normal queries)
    let found_user = repo
        .find_by_id(&user.id)
        .await
        .expect("Failed to query user");

    assert!(
        found_user.is_none(),
        "Soft deleted user should not be found"
    );

    // Cleanup after test
    repo.hard_delete(&user.id)
        .await
        .expect("Failed to hard delete user");
}

#[tokio::test]
#[ignore] // Requires database
async fn test_user_repository_email_exists() {
    let pool = create_test_pool().await;
    setup_test_schema(&pool).await;

    let repo = UserRepository::new(pool.clone());
    let test_email = "test_exists@example.com";

    // Cleanup before test
    cleanup_test_data(&pool, test_email).await;

    // Check email doesn't exist
    let exists = repo
        .email_exists(test_email, "org123")
        .await
        .expect("Failed to check email");
    assert!(!exists);

    // Create user
    repo.create(
        "org123",
        test_email,
        "hashed_password",
        "Test User",
        vec!["user".to_string()],
        None,
    )
    .await
    .expect("Failed to create user");

    // Check email exists
    let exists = repo
        .email_exists(test_email, "org123")
        .await
        .expect("Failed to check email");
    assert!(exists);

    // Cleanup after test
    cleanup_test_data(&pool, test_email).await;
}
