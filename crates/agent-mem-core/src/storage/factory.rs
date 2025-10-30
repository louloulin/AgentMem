//! Repository Factory
//!
//! Provides a factory pattern for creating repository instances based on database configuration.
//! Supports both LibSQL (embedded) and PostgreSQL (enterprise) backends.
//!
//! This module provides two factory implementations:
//! 1. `RepositoryFactory`: Legacy factory using `DatabaseConfig`
//! 2. `StorageFactory`: New factory using `DeploymentMode` (Task 3.2)

use agent_mem_config::{
    database::{DatabaseBackend, DatabaseConfig},
    DeploymentMode, EmbeddedModeConfig, ServerModeConfig,
};
use agent_mem_traits::Result;
use std::sync::Arc;

use crate::storage::traits::*;

#[cfg(feature = "libsql")]
use crate::storage::libsql::{
    create_libsql_pool, run_migrations, LibSqlAgentRepository, LibSqlApiKeyRepository,
    LibSqlAssociationRepository, LibSqlBlockRepository, LibSqlMemoryRepository,
    LibSqlMessageRepository, LibSqlOrganizationRepository, LibSqlToolRepository,
    LibSqlUserRepository,
};

// Note: PostgreSQL repository implementations are being refactored.
// The factory will return a clear error until Pg repositories implement the traits.

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

    /// API Key repository
    pub api_keys: Arc<dyn ApiKeyRepositoryTrait>,

    /// Memory repository
    pub memories: Arc<dyn MemoryRepositoryTrait>,

    /// Block repository
    pub blocks: Arc<dyn BlockRepositoryTrait>,

    /// Association repository
    pub associations: Arc<dyn AssociationRepositoryTrait>,
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
            .map_err(|e| AgentMemError::StorageError(format!("Failed to create LibSQL connection: {e}")))?;

        // Run migrations if auto_migrate is enabled
        if config.auto_migrate {
            run_migrations(conn.clone())
                .await
                .map_err(|e| AgentMemError::StorageError(format!("Failed to run migrations: {e}")))?;
        }

        // Create repository instances
        Ok(Repositories {
            users: Arc::new(LibSqlUserRepository::new(conn.clone())),
            organizations: Arc::new(LibSqlOrganizationRepository::new(conn.clone())),
            agents: Arc::new(LibSqlAgentRepository::new(conn.clone())),
            messages: Arc::new(LibSqlMessageRepository::new(conn.clone())),
            tools: Arc::new(LibSqlToolRepository::new(conn.clone())),
            api_keys: Arc::new(LibSqlApiKeyRepository::new(conn.clone())),
            memories: Arc::new(LibSqlMemoryRepository::new(conn.clone())),
            blocks: Arc::new(LibSqlBlockRepository::new(conn.clone())),
            associations: Arc::new(LibSqlAssociationRepository::new(conn.clone())),
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
            // Use our internal Rust-based migrations module
            crate::storage::migrations::run_migrations(&pool)
                .await
                .map_err(|e| AgentMemError::StorageError(format!("Failed to run migrations: {}", e)))?;
        }

        // PostgreSQL repositories are under refactor and not yet implementing traits.
        // Return a clear error to callers for now.
        Err(AgentMemError::ConfigError(
            "PostgreSQL repositories are not yet implemented for the new trait-based factory.".to_string(),
        ))
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

        // Verify migrations ran by checking we can create an organization first
        use crate::storage::models::{Organization, User};
        
        // Create organization first (required for foreign key constraint)
        let org = Organization::new("Test Org".to_string());
        let org_result = repos.organizations.create(&org).await;
        assert!(org_result.is_ok(), "Failed to create organization after migration: {:?}", org_result.err());
        
        // Now create user with the organization ID
        let user = User::new(
            org.id.clone(),
            "Test User".to_string(),
            "test@example.com".to_string(),
            "password_hash".to_string(),
            "UTC".to_string()
        );
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
        let user = User::new(
            "org-123".to_string(),
            "Test User".to_string(),
            "test@example.com".to_string(),
            "password_hash".to_string(),
            "UTC".to_string()
        );
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

    #[tokio::test]
    #[cfg(feature = "postgres")]
    async fn test_postgres_factory_returns_error_until_implemented() {
        let config = DatabaseConfig {
            backend: DatabaseBackend::Postgres,
            url: "postgresql://localhost/agentmem".to_string(),
            pool: PoolConfig::default(),
            auto_migrate: false,
            log_queries: false,
            slow_query_threshold_ms: 1000,
        };

        let result = RepositoryFactory::create_repositories(&config).await;
        assert!(result.is_err());
        let msg = result.err().unwrap().to_string();
        assert!(msg.contains("PostgreSQL repositories are not yet implemented"));
    }
}

// ============================================================================
// StorageFactory - New factory using DeploymentMode (Task 3.2)
// ============================================================================

/// Storage factory for creating repository instances based on deployment mode
///
/// This is the new factory implementation that uses `DeploymentMode` configuration
/// introduced in Task 3.1. It provides a unified interface for creating storage
/// instances in both embedded and server modes.
///
/// # Examples
///
/// ## Embedded Mode
///
/// ```no_run
/// use agent_mem_config::DeploymentMode;
/// use agent_mem_core::storage::factory::StorageFactory;
///
/// # async fn example() -> agent_mem_traits::Result<()> {
/// let mode = DeploymentMode::embedded("./data");
/// let repos = StorageFactory::create(mode).await?;
/// # Ok(())
/// # }
/// ```
///
/// ## Server Mode with pgvector
///
/// ```no_run
/// use agent_mem_config::DeploymentMode;
/// use agent_mem_core::storage::factory::StorageFactory;
///
/// # async fn example() -> agent_mem_traits::Result<()> {
/// let mode = DeploymentMode::server_with_pgvector(
///     "postgresql://localhost:5432/agentmem".to_string()
/// );
/// let repos = StorageFactory::create(mode).await?;
/// # Ok(())
/// # }
/// ```
pub struct StorageFactory;

impl StorageFactory {
    /// Create storage repositories based on deployment mode
    ///
    /// # Arguments
    ///
    /// * `mode` - Deployment mode configuration (Embedded or Server)
    ///
    /// # Returns
    ///
    /// A `Repositories` struct containing all repository instances
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Database connection fails
    /// - Migrations fail
    /// - Required feature is not enabled
    /// - Vector store initialization fails
    pub async fn create(mode: DeploymentMode) -> Result<Repositories> {
        match mode {
            DeploymentMode::Embedded(config) => {
                Self::create_embedded(config).await
            }
            DeploymentMode::Server(config) => {
                Self::create_server(config).await
            }
        }
    }

    /// Create embedded mode storage (LibSQL + LanceDB)
    ///
    /// # Arguments
    ///
    /// * `config` - Embedded mode configuration
    ///
    /// # Returns
    ///
    /// Repositories configured for embedded deployment
    #[cfg(feature = "libsql")]
    async fn create_embedded(config: EmbeddedModeConfig) -> Result<Repositories> {
        use agent_mem_traits::AgentMemError;

        // 1. Create LibSQL connection
        let conn = create_libsql_pool(&config.database_path.to_string_lossy())
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to create LibSQL connection: {e}")))?;

        // 2. Run migrations
        run_migrations(conn.clone())
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to run migrations: {e}")))?;

        // 3. Create repository instances
        Ok(Repositories {
            users: Arc::new(LibSqlUserRepository::new(conn.clone())),
            organizations: Arc::new(LibSqlOrganizationRepository::new(conn.clone())),
            agents: Arc::new(LibSqlAgentRepository::new(conn.clone())),
            messages: Arc::new(LibSqlMessageRepository::new(conn.clone())),
            tools: Arc::new(LibSqlToolRepository::new(conn.clone())),
            api_keys: Arc::new(LibSqlApiKeyRepository::new(conn.clone())),
            memories: Arc::new(LibSqlMemoryRepository::new(conn.clone())),
            blocks: Arc::new(LibSqlBlockRepository::new(conn.clone())),
            associations: Arc::new(LibSqlAssociationRepository::new(conn.clone())),
        })
    }

    /// Create embedded mode storage (fallback when feature is disabled)
    #[cfg(not(feature = "libsql"))]
    async fn create_embedded(_config: EmbeddedModeConfig) -> Result<Repositories> {
        use agent_mem_traits::AgentMemError;
        Err(AgentMemError::ConfigError(
            "LibSQL support is not enabled. Enable the 'libsql' feature to use embedded mode.".to_string()
        ))
    }

    /// Create server mode storage (PostgreSQL + Vector Services)
    ///
    /// # Arguments
    ///
    /// * `config` - Server mode configuration
    ///
    /// # Returns
    ///
    /// Repositories configured for server deployment
    #[cfg(feature = "postgres")]
    async fn create_server(config: ServerModeConfig) -> Result<Repositories> {
        use agent_mem_traits::AgentMemError;
        use sqlx::postgres::PgPoolOptions;

        // 1. Create PostgreSQL connection pool
        let pool = PgPoolOptions::new()
            .max_connections(config.pool_config.max_connections)
            .min_connections(config.pool_config.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(config.pool_config.connect_timeout_seconds))
            .idle_timeout(Some(std::time::Duration::from_secs(config.pool_config.idle_timeout_seconds)))
            .max_lifetime(Some(std::time::Duration::from_secs(config.pool_config.max_lifetime_seconds)))
            .connect(&config.database_url)
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to create PostgreSQL pool: {}", e)))?;

        // 2. Run migrations
        crate::storage::migrations::run_migrations(&pool)
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to run migrations: {}", e)))?;

        // 3. PostgreSQL repositories are under refactor
        // Return a clear error until they implement the new traits
        Err(AgentMemError::ConfigError(
            "PostgreSQL repositories are not yet implemented for the new trait-based factory.".to_string(),
        ))
    }

    /// Create server mode storage (fallback when feature is disabled)
    #[cfg(not(feature = "postgres"))]
    async fn create_server(_config: ServerModeConfig) -> Result<Repositories> {
        use agent_mem_traits::AgentMemError;
        Err(AgentMemError::ConfigError(
            "PostgreSQL support is not enabled. Enable the 'postgres' feature to use server mode.".to_string()
        ))
    }
}

#[cfg(test)]
mod storage_factory_tests {
    use super::*;
    use agent_mem_config::DeploymentMode;

    #[tokio::test]
    #[cfg(feature = "libsql")]
    async fn test_storage_factory_embedded_mode() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let data_root = temp_dir.path();

        let mode = DeploymentMode::embedded(data_root);
        let repos = StorageFactory::create(mode).await;

        assert!(repos.is_ok(), "Failed to create embedded storage: {:?}", repos.err());

        let repos = repos.unwrap();

        // Test that we can use the repositories
        let users = repos.users.list(10, 0).await;
        assert!(users.is_ok(), "Failed to list users: {:?}", users.err());
    }

    #[tokio::test]
    #[cfg(feature = "libsql")]
    async fn test_storage_factory_embedded_with_custom_config() {
        use tempfile::TempDir;
        use agent_mem_config::EmbeddedModeConfig;
        use std::path::PathBuf;

        let temp_dir = TempDir::new().unwrap();

        let config = EmbeddedModeConfig {
            database_path: temp_dir.path().join("custom.db"),
            vector_path: temp_dir.path().join("vectors"),
            vector_dimension: 768,
            enable_wal: false,
            cache_size_kb: 5120,
        };

        let mode = DeploymentMode::Embedded(config);
        let repos = StorageFactory::create(mode).await;

        assert!(repos.is_ok(), "Failed to create embedded storage with custom config: {:?}", repos.err());
    }

    #[tokio::test]
    #[cfg(feature = "libsql")]
    async fn test_storage_factory_embedded_creates_tables() {
        use tempfile::TempDir;
        use crate::storage::models::{Organization, User};

        let temp_dir = TempDir::new().unwrap();
        let mode = DeploymentMode::embedded(temp_dir.path());
        let repos = StorageFactory::create(mode).await.unwrap();

        // First create an organization (required for foreign key)
        let org = Organization::new("Test Org".to_string());
        repos.organizations.create(&org).await.unwrap();

        // Then create a user
        let user = User::new(
            org.id.clone(),
            "Test User".to_string(),
            "test@example.com".to_string(),
            "password_hash".to_string(),
            "UTC".to_string()
        );

        let result = repos.users.create(&user).await;
        assert!(result.is_ok(), "Failed to create user: {:?}", result.err());
    }

    #[tokio::test]
    #[cfg(not(feature = "libsql"))]
    async fn test_storage_factory_embedded_feature_disabled() {
        use agent_mem_config::EmbeddedModeConfig;
        use std::path::PathBuf;

        let config = EmbeddedModeConfig {
            database_path: PathBuf::from("./data/test.db"),
            vector_path: PathBuf::from("./data/vectors"),
            vector_dimension: 1536,
            enable_wal: true,
            cache_size_kb: 10240,
        };

        let mode = DeploymentMode::Embedded(config);
        let result = StorageFactory::create(mode).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("LibSQL support is not enabled"));
    }

    #[tokio::test]
    #[cfg(not(feature = "postgres"))]
    async fn test_storage_factory_server_feature_disabled() {
        let mode = DeploymentMode::server_with_pgvector(
            "postgresql://localhost:5432/test".to_string()
        );

        let result = StorageFactory::create(mode).await;

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("PostgreSQL support is not enabled"));
        }
    }

    #[tokio::test]
    #[cfg(feature = "postgres")]
    async fn test_storage_factory_server_not_yet_implemented() {
        let mode = DeploymentMode::server_with_pgvector(
            "postgresql://localhost:5432/agentmem".to_string()
        );

        let result = StorageFactory::create(mode).await;

        assert!(result.is_err());
        let msg = result.err().unwrap().to_string();
        assert!(msg.contains("PostgreSQL repositories are not yet implemented"));
    }

    #[tokio::test]
    #[cfg(feature = "libsql")]
    async fn test_storage_factory_convenience_methods() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        // Test embedded() convenience method
        let mode = DeploymentMode::embedded(temp_dir.path());
        let repos = StorageFactory::create(mode).await;
        assert!(repos.is_ok(), "Failed with embedded() method");
    }

    #[tokio::test]
    #[cfg(feature = "libsql")]
    async fn test_storage_factory_all_repositories_available() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let mode = DeploymentMode::embedded(temp_dir.path());
        let repos = StorageFactory::create(mode).await.unwrap();

        // Verify key repositories with list() method are available
        assert!(repos.users.list(1, 0).await.is_ok());
        assert!(repos.organizations.list(1, 0).await.is_ok());
        assert!(repos.agents.list(1, 0).await.is_ok());
        assert!(repos.tools.list(1, 0).await.is_ok());
        assert!(repos.api_keys.list(1, 0).await.is_ok());
        assert!(repos.blocks.list(1, 0).await.is_ok());

        // Verify all repositories exist (even if they don't have list())
        let _ = &repos.memories;
        let _ = &repos.messages;
        let _ = &repos.associations;
    }
}



