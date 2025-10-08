//! Repository Factory
//!
//! Provides a factory pattern for creating repository instances based on database configuration.
//! Supports both LibSQL (embedded) and PostgreSQL (enterprise) backends.

use agent_mem_config::{DatabaseBackend, DatabaseConfig};
use agent_mem_traits::Result;
use std::sync::Arc;

use crate::storage::traits::*;

#[cfg(feature = "libsql")]
use crate::storage::libsql::{
    create_libsql_pool, run_migrations, LibSqlAgentRepository, LibSqlMessageRepository,
    LibSqlOrganizationRepository, LibSqlToolRepository, LibSqlUserRepository,
};

#[cfg(feature = "postgres")]
use crate::storage::user_repository::UserRepository as PgUserRepository;

/// Container for all repository trait objects
///
/// This struct holds Arc-wrapped trait objects for all repositories,
/// allowing them to be shared across threads and used polymorphically.
#[derive(Clone)]
pub struct Repositories {
    /// User repository
    pub users: Arc<dyn UserRepositoryTrait>,

    /// Organization repository
    pub organizations: Arc<dyn OrganizationRepositoryTrait>,

    /// Agent repository
    pub agents: Arc<dyn AgentRepositoryTrait>,

    /// Message repository
    pub messages: Arc<dyn MessageRepositoryTrait>,

    /// Tool repository
    pub tools: Arc<dyn ToolRepositoryTrait>,

    // TODO: Add other repositories as they are implemented
    // pub api_keys: Arc<dyn ApiKeyRepositoryTrait>,
    // pub memories: Arc<dyn MemoryRepositoryTrait>,
    // pub blocks: Arc<dyn BlockRepositoryTrait>,
}

/// Factory for creating repository instances
pub struct RepositoryFactory;

impl RepositoryFactory {
    /// Create all repositories based on the provided configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Database configuration specifying backend and connection details
    ///
    /// # Returns
    ///
    /// A `Repositories` struct containing all repository instances
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Database connection fails
    /// - Migrations fail (if auto_migrate is enabled)
    /// - Required feature is not enabled (e.g., postgres feature for PostgreSQL)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use agent_mem_config::DatabaseConfig;
    /// use agent_mem_core::storage::factory::RepositoryFactory;
    ///
    /// # async fn example() -> agent_mem_traits::Result<()> {
    /// let config = DatabaseConfig::from_env();
    /// let repos = RepositoryFactory::create_repositories(&config).await?;
    /// 
    /// // Use repositories
    /// let users = repos.users.list(10, 0).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_repositories(config: &DatabaseConfig) -> Result<Repositories> {
        match config.backend {
            DatabaseBackend::LibSql => {
                Self::create_libsql_repositories(config).await
            }
            DatabaseBackend::Postgres => {
                Self::create_postgres_repositories(config).await
            }
        }
    }

    /// Create LibSQL-backed repositories
    #[cfg(feature = "libsql")]
    async fn create_libsql_repositories(config: &DatabaseConfig) -> Result<Repositories> {
        use agent_mem_traits::AgentMemError;

        // Create connection pool
        let conn = create_libsql_pool(&config.url)
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to create LibSQL connection: {}", e)))?;

        // Run migrations if auto_migrate is enabled
        if config.auto_migrate {
            run_migrations(conn.clone())
                .await
                .map_err(|e| AgentMemError::StorageError(format!("Failed to run migrations: {}", e)))?;
        }

        // Create repository instances
        Ok(Repositories {
            users: Arc::new(LibSqlUserRepository::new(conn.clone())),
            organizations: Arc::new(LibSqlOrganizationRepository::new(conn.clone())),
            agents: Arc::new(LibSqlAgentRepository::new(conn.clone())),
            messages: Arc::new(LibSqlMessageRepository::new(conn.clone())),
            tools: Arc::new(LibSqlToolRepository::new(conn.clone())),
            // TODO: Add other repositories as they are implemented
        })
    }

    /// Create LibSQL-backed repositories (fallback when feature is disabled)
    #[cfg(not(feature = "libsql"))]
    async fn create_libsql_repositories(_config: &DatabaseConfig) -> Result<Repositories> {
        use agent_mem_traits::AgentMemError;
        Err(AgentMemError::ConfigError(
            "LibSQL support is not enabled. Enable the 'libsql' feature to use LibSQL backend.".to_string()
        ))
    }

    /// Create PostgreSQL-backed repositories
    #[cfg(feature = "postgres")]
    async fn create_postgres_repositories(config: &DatabaseConfig) -> Result<Repositories> {
        use agent_mem_traits::AgentMemError;
        use sqlx::postgres::PgPoolOptions;

        // Create connection pool
        let pool = PgPoolOptions::new()
            .max_connections(config.pool.max_connections)
            .min_connections(config.pool.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(config.pool.acquire_timeout_seconds))
            .idle_timeout(Some(std::time::Duration::from_secs(config.pool.idle_timeout_seconds)))
            .max_lifetime(Some(std::time::Duration::from_secs(config.pool.max_lifetime_seconds)))
            .connect(&config.url)
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to create PostgreSQL pool: {}", e)))?;

        // Run migrations if auto_migrate is enabled
        if config.auto_migrate {
            sqlx::migrate!("./migrations")
                .run(&pool)
                .await
                .map_err(|e| AgentMemError::StorageError(format!("Failed to run migrations: {}", e)))?;
        }

        // Create repository instances
        Ok(Repositories {
            users: Arc::new(PgUserRepository::new(pool.clone())),
            // TODO: Add other repositories as they are implemented
        })
    }

    /// Create PostgreSQL-backed repositories (fallback when feature is disabled)
    #[cfg(not(feature = "postgres"))]
    async fn create_postgres_repositories(_config: &DatabaseConfig) -> Result<Repositories> {
        use agent_mem_traits::AgentMemError;
        Err(AgentMemError::ConfigError(
            "PostgreSQL support is not enabled. Enable the 'postgres' feature to use PostgreSQL backend.".to_string()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_mem_config::{DatabaseBackend, DatabaseConfig, PoolConfig};

    #[tokio::test]
    #[cfg(feature = "libsql")]
    async fn test_create_libsql_repositories() {
        use tempfile::TempDir;

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
        assert!(repos.is_ok(), "Failed to create LibSQL repositories: {:?}", repos.err());

        let repos = repos.unwrap();
        
        // Test that we can use the user repository
        let users = repos.users.list(10, 0).await;
        assert!(users.is_ok(), "Failed to list users: {:?}", users.err());
    }

    #[tokio::test]
    #[cfg(feature = "libsql")]
    async fn test_libsql_auto_migrate() {
        use tempfile::TempDir;

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

        let repos = RepositoryFactory::create_repositories(&config).await.unwrap();

        // Verify migrations ran by checking we can create a user
        use crate::storage::models::User;
        let user = User::new("org-123".to_string(), "Test User".to_string(), "UTC".to_string());
        let result = repos.users.create(&user).await;
        assert!(result.is_ok(), "Failed to create user after migration: {:?}", result.err());
    }

    #[tokio::test]
    #[cfg(feature = "libsql")]
    async fn test_libsql_no_auto_migrate() {
        use tempfile::TempDir;

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

        let repos = RepositoryFactory::create_repositories(&config).await.unwrap();

        // Without migrations, creating a user should fail
        use crate::storage::models::User;
        let user = User::new("org-123".to_string(), "Test User".to_string(), "UTC".to_string());
        let result = repos.users.create(&user).await;
        assert!(result.is_err(), "Expected error without migrations, but got success");
    }

    #[tokio::test]
    #[cfg(not(feature = "libsql"))]
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
        assert!(result.unwrap_err().to_string().contains("LibSQL support is not enabled"));
    }

    #[tokio::test]
    #[cfg(not(feature = "postgres"))]
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

