//! Integration tests for Repository Factory
//!
//! Tests the factory pattern for creating repository instances based on database configuration.

#[cfg(feature = "libsql")]
mod libsql_factory_tests {
    use agent_mem_config::{DatabaseBackend, DatabaseConfig, PoolConfig};
    use agent_mem_core::storage::factory::RepositoryFactory;
    use agent_mem_core::storage::models::User;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_create_libsql_repositories() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let config = DatabaseConfig {
            backend: DatabaseBackend::LibSql,
            url: db_path.to_str().unwrap().to_string(),
            pool: PoolConfig::default(),
            auto_migrate: true,
            log_queries: false,
            slow_query_threshold_ms: 1000,
        };

        let repos = RepositoryFactory::create_repositories(&config).await;
        assert!(
            repos.is_ok(),
            "Failed to create LibSQL repositories: {:?}",
            repos.err()
        );

        let repos = repos.unwrap();

        // Test that we can use the user repository
        let users = repos.users.list(10, 0).await;
        assert!(users.is_ok(), "Failed to list users: {:?}", users.err());
    }

    #[tokio::test]
    async fn test_libsql_auto_migrate() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_migrate.db");

        let config = DatabaseConfig {
            backend: DatabaseBackend::LibSql,
            url: db_path.to_str().unwrap().to_string(),
            pool: PoolConfig::default(),
            auto_migrate: true,
            log_queries: false,
            slow_query_threshold_ms: 1000,
        };

        let repos = RepositoryFactory::create_repositories(&config)
            .await
            .unwrap();

        // Verify migrations ran by checking we can list users (table exists)
        let result = repos.users.list(10, 0).await;
        assert!(
            result.is_ok(),
            "Failed to list users after migration: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_libsql_no_auto_migrate() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_no_migrate.db");

        let config = DatabaseConfig {
            backend: DatabaseBackend::LibSql,
            url: db_path.to_str().unwrap().to_string(),
            pool: PoolConfig::default(),
            auto_migrate: false,
            log_queries: false,
            slow_query_threshold_ms: 1000,
        };

        let repos = RepositoryFactory::create_repositories(&config)
            .await
            .unwrap();

        // Without migrations, listing users should fail (table doesn't exist)
        let result = repos.users.list(10, 0).await;
        assert!(
            result.is_err(),
            "Expected error without migrations, but got success"
        );
    }

    #[tokio::test]
    async fn test_factory_crud_operations() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_crud.db");

        let config = DatabaseConfig {
            backend: DatabaseBackend::LibSql,
            url: db_path.to_str().unwrap().to_string(),
            pool: PoolConfig::default(),
            auto_migrate: true,
            log_queries: false,
            slow_query_threshold_ms: 1000,
        };

        let repos = RepositoryFactory::create_repositories(&config)
            .await
            .unwrap();

        // Test basic repository operations (list)
        let users = repos.users.list(10, 0).await.unwrap();
        assert_eq!(users.len(), 0, "Should start with no users");

        // Note: Full CRUD operations would require creating an organization first
        // due to foreign key constraints. This test verifies the factory creates
        // working repository instances.
    }

    #[tokio::test]
    async fn test_factory_multiple_instances() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_multi.db");

        let config = DatabaseConfig {
            backend: DatabaseBackend::LibSql,
            url: db_path.to_str().unwrap().to_string(),
            pool: PoolConfig::default(),
            auto_migrate: true,
            log_queries: false,
            slow_query_threshold_ms: 1000,
        };

        // Create first instance
        let repos1 = RepositoryFactory::create_repositories(&config)
            .await
            .unwrap();

        // Create second instance (should share the same database)
        let repos2 = RepositoryFactory::create_repositories(&config)
            .await
            .unwrap();

        // Verify both instances can access the same database
        let users1 = repos1.users.list(10, 0).await.unwrap();
        let users2 = repos2.users.list(10, 0).await.unwrap();
        assert_eq!(
            users1.len(),
            users2.len(),
            "Both instances should see the same data"
        );
    }
}

#[cfg(not(feature = "libsql"))]
mod libsql_feature_disabled_tests {
    use agent_mem_config::{DatabaseBackend, DatabaseConfig, PoolConfig};
    use agent_mem_core::storage::factory::RepositoryFactory;

    #[tokio::test]
    async fn test_libsql_feature_disabled() {
        let config = DatabaseConfig {
            backend: DatabaseBackend::LibSql,
            url: "test.db".to_string(),
            pool: PoolConfig::default(),
            auto_migrate: false,
            log_queries: false,
            slow_query_threshold_ms: 1000,
        };

        let result = RepositoryFactory::create_repositories(&config).await;
        assert!(result.is_err());
        let err_msg = result.err().unwrap().to_string();
        assert!(err_msg.contains("LibSQL support is not enabled"));
    }
}

#[cfg(not(feature = "postgres"))]
mod postgres_feature_disabled_tests {
    use agent_mem_config::{DatabaseBackend, DatabaseConfig, PoolConfig};
    use agent_mem_core::storage::factory::RepositoryFactory;

    #[tokio::test]
    async fn test_postgres_feature_disabled() {
        let config = DatabaseConfig {
            backend: DatabaseBackend::Postgres,
            url: "postgresql://localhost/test".to_string(),
            pool: PoolConfig::default(),
            auto_migrate: false,
            log_queries: false,
            slow_query_threshold_ms: 1000,
        };

        let result = RepositoryFactory::create_repositories(&config).await;
        assert!(result.is_err());
        let err_msg = result.err().unwrap().to_string();
        assert!(err_msg.contains("PostgreSQL support is not enabled"));
    }
}
