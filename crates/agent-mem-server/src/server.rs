//! Main server implementation

// ✅ 使用memory::MemoryManager（基于agent-mem统一API）
use crate::routes::memory::MemoryManager;
use crate::{
    config::ServerConfig,
    error::{ServerError, ServerResult},
    routes::create_router,
    telemetry::setup_telemetry,
};
use agent_mem_config::{DatabaseBackend, DatabaseConfig};
use agent_mem_core::storage::factory::{Repositories, RepositoryFactory};
use agent_mem_observability::metrics::MetricsRegistry;
use axum::Router;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

/// Main memory server
pub struct MemoryServer {
    config: ServerConfig,
    memory_manager: Arc<MemoryManager>,
    metrics_registry: Arc<MetricsRegistry>,
    repositories: Repositories,
    router: Router,
}

impl MemoryServer {
    /// Create a new memory server
    pub async fn new(config: ServerConfig) -> ServerResult<Self> {
        // Setup telemetry
        setup_telemetry(&config)?;

        // Create database configuration from server config
        let db_config = DatabaseConfig {
            backend: if config.database_url.starts_with("libsql://") || config.database_url.ends_with(".db") {
                DatabaseBackend::LibSql
            } else {
                DatabaseBackend::Postgres
            },
            url: config.database_url.clone(),
            pool: Default::default(),
            auto_migrate: true,
            log_queries: false,
            slow_query_threshold_ms: 1000,
        };

        info!("Initializing database backend: {:?}", db_config.backend);

        // Create repositories using factory
        let repositories = RepositoryFactory::create_repositories(&db_config)
            .await
            .map_err(|e| ServerError::ServerError(format!("Failed to create repositories: {e}")))?;

        info!("Database repositories initialized");

        // Create memory manager (✅ 使用异步new()方法)
        let memory_manager = Arc::new(
            MemoryManager::new()
                .await
                .map_err(|e| ServerError::ServerError(format!("Failed to create memory manager: {e}")))?
        );
        info!("Memory manager initialized (using agent-mem unified API)");

        // Create metrics registry
        let metrics_registry = Arc::new(MetricsRegistry::new());
        info!("Metrics registry initialized");

        // Create router with all routes and middleware
        let router = create_router(
            memory_manager.clone(),
            metrics_registry.clone(),
            repositories.clone(),
        )
        .await?;

        info!("Memory server initialized successfully");

        Ok(Self {
            config,
            memory_manager,
            metrics_registry,
            repositories,
            router,
        })
    }

    /// Start the server
    pub async fn start(self) -> ServerResult<()> {
        let addr = SocketAddr::from(([0, 0, 0, 0], self.config.port));
        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| ServerError::BindError(e.to_string()))?;

        info!("AgentMem server starting on {}", addr);
        info!("API documentation available at http://{}/swagger-ui/", addr);
        info!("Health check endpoint: http://{}/health", addr);
        info!("Metrics endpoint: http://{}/metrics", addr);

        // Start the server
        axum::serve(listener, self.router)
            .await
            .map_err(|e| ServerError::ServerError(e.to_string()))?;

        Ok(())
    }

    /// Get server configuration
    pub fn config(&self) -> &ServerConfig {
        &self.config
    }

    /// Get memory manager reference
    pub fn memory_manager(&self) -> Arc<MemoryManager> {
        self.memory_manager.clone()
    }

    /// Graceful shutdown
    pub async fn shutdown(&self) -> ServerResult<()> {
        info!("Shutting down AgentMem server gracefully...");
        // Perform cleanup operations here
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_server_creation() {
        let mut config = ServerConfig::default();
        config.enable_logging = false; // Disable logging to avoid telemetry conflicts
        let server = MemoryServer::new(config).await;
        assert!(server.is_ok());
    }

    #[tokio::test]
    async fn test_server_config() {
        let mut config = ServerConfig::default();
        config.enable_logging = false; // Disable logging to avoid telemetry conflicts
        let server = MemoryServer::new(config.clone()).await.unwrap();
        assert_eq!(server.config().port, config.port);
    }
}
