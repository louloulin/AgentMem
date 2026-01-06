//! Integration tests for LibSQL backend
//!
//! Tests the complete LibSQL implementation including:
//! - Connection management
//! - Migrations
//! - Basic CRUD operations

#[cfg(feature = "libsql")]
mod libsql_tests {
    use agent_mem_core::storage::libsql::{
        connection::create_libsql_pool, migrations::run_migrations,
    };
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_libsql_connection_and_migrations() {
        // Create a temporary directory for the test database
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let db_path_str = db_path.to_str().expect("Invalid path");

        // Create connection pool
        let conn = create_libsql_pool(db_path_str)
            .await
            .expect("Failed to create connection pool");

        // Run migrations
        run_migrations(conn.clone())
            .await
            .expect("Failed to run migrations");

        // Verify migrations table exists
        let conn_guard = conn.lock().await;
        let mut rows = conn_guard
            .query(
                "SELECT COUNT(*) as count FROM _migrations",
                libsql::params![],
            )
            .await
            .expect("Failed to query migrations table");

        if let Some(row) = rows.next().await.expect("Failed to get row") {
            let count: i64 = row.get(0).expect("Failed to get count");
            assert_eq!(count, 10, "Expected 10 migrations to be applied");
        } else {
            panic!("No rows returned from migrations table");
        }

        // Verify organizations table exists
        let mut rows = conn_guard
            .query(
                "SELECT name FROM sqlite_master WHERE type='table' AND name='organizations'",
                libsql::params![],
            )
            .await
            .expect("Failed to query sqlite_master");

        assert!(
            rows.next().await.expect("Failed to get row").is_some(),
            "organizations table should exist"
        );

        // Verify users table exists
        let mut rows = conn_guard
            .query(
                "SELECT name FROM sqlite_master WHERE type='table' AND name='users'",
                libsql::params![],
            )
            .await
            .expect("Failed to query sqlite_master");

        assert!(
            rows.next().await.expect("Failed to get row").is_some(),
            "users table should exist"
        );

        // Verify agents table exists
        let mut rows = conn_guard
            .query(
                "SELECT name FROM sqlite_master WHERE type='table' AND name='agents'",
                libsql::params![],
            )
            .await
            .expect("Failed to query sqlite_master");

        assert!(
            rows.next().await.expect("Failed to get row").is_some(),
            "agents table should exist"
        );

        println!("✅ LibSQL connection and migrations test passed!");
    }

    #[tokio::test]
    async fn test_libsql_idempotent_migrations() {
        // Create a temporary directory for the test database
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test_idempotent.db");
        let db_path_str = db_path.to_str().expect("Invalid path");

        // Create connection pool
        let conn = create_libsql_pool(db_path_str)
            .await
            .expect("Failed to create connection pool");

        // Run migrations first time
        run_migrations(conn.clone())
            .await
            .expect("Failed to run migrations first time");

        // Run migrations second time (should be idempotent)
        run_migrations(conn.clone())
            .await
            .expect("Failed to run migrations second time");

        // Verify still only 10 migrations
        let conn_guard = conn.lock().await;
        let mut rows = conn_guard
            .query(
                "SELECT COUNT(*) as count FROM _migrations",
                libsql::params![],
            )
            .await
            .expect("Failed to query migrations table");

        if let Some(row) = rows.next().await.expect("Failed to get row") {
            let count: i64 = row.get(0).expect("Failed to get count");
            assert_eq!(count, 10, "Expected 10 migrations (idempotent)");
        } else {
            panic!("No rows returned from migrations table");
        }

        println!("✅ LibSQL idempotent migrations test passed!");
    }

    #[tokio::test]
    async fn test_libsql_basic_crud() {
        // Create a temporary directory for the test database
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test_crud.db");
        let db_path_str = db_path.to_str().expect("Invalid path");

        // Create connection pool
        let conn = create_libsql_pool(db_path_str)
            .await
            .expect("Failed to create connection pool");

        // Run migrations
        run_migrations(conn.clone())
            .await
            .expect("Failed to run migrations");

        let conn_guard = conn.lock().await;

        // Insert an organization
        let org_id = "org-test-123";
        let org_name = "Test Organization";
        let now = chrono::Utc::now().timestamp();

        conn_guard
            .execute(
                "INSERT INTO organizations (id, name, created_at, updated_at, is_deleted) VALUES (?, ?, ?, ?, ?)",
                libsql::params![org_id, org_name, now, now, 0],
            )
            .await
            .expect("Failed to insert organization");

        // Query the organization
        let mut rows = conn_guard
            .query(
                "SELECT id, name FROM organizations WHERE id = ?",
                libsql::params![org_id],
            )
            .await
            .expect("Failed to query organization");

        if let Some(row) = rows.next().await.expect("Failed to get row") {
            let id: String = row.get(0).expect("Failed to get id");
            let name: String = row.get(1).expect("Failed to get name");
            assert_eq!(id, org_id);
            assert_eq!(name, org_name);
        } else {
            panic!("Organization not found");
        }

        // Update the organization
        let new_name = "Updated Organization";
        conn_guard
            .execute(
                "UPDATE organizations SET name = ?, updated_at = ? WHERE id = ?",
                libsql::params![new_name, chrono::Utc::now().timestamp(), org_id],
            )
            .await
            .expect("Failed to update organization");

        // Verify update
        let mut rows = conn_guard
            .query(
                "SELECT name FROM organizations WHERE id = ?",
                libsql::params![org_id],
            )
            .await
            .expect("Failed to query organization");

        if let Some(row) = rows.next().await.expect("Failed to get row") {
            let name: String = row.get(0).expect("Failed to get name");
            assert_eq!(name, new_name);
        } else {
            panic!("Organization not found after update");
        }

        // Soft delete the organization
        conn_guard
            .execute(
                "UPDATE organizations SET is_deleted = ? WHERE id = ?",
                libsql::params![1, org_id],
            )
            .await
            .expect("Failed to soft delete organization");

        // Verify soft delete
        let mut rows = conn_guard
            .query(
                "SELECT is_deleted FROM organizations WHERE id = ?",
                libsql::params![org_id],
            )
            .await
            .expect("Failed to query organization");

        if let Some(row) = rows.next().await.expect("Failed to get row") {
            let is_deleted: i64 = row.get(0).expect("Failed to get is_deleted");
            assert_eq!(is_deleted, 1);
        } else {
            panic!("Organization not found after soft delete");
        }

        println!("✅ LibSQL basic CRUD test passed!");
    }
}
