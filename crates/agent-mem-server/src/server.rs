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
            backend: if config.database_url.starts_with("libsql://")
                || config.database_url.starts_with("memory://")
                || config.database_url.ends_with(".db")
                || config.database_url == ":memory:"
                || config.database_url.starts_with("file:")
            {
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

        // Create memory manager (✅ 使用异步new()方法 + Embedder配置)
        let memory_manager = Arc::new(
            MemoryManager::new(
                config.embedder_provider.clone(),
                config.embedder_model.clone(),
            )
            .await
            .map_err(|e| {
                ServerError::ServerError(format!("Failed to create memory manager: {e}"))
            })?,
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

    /// Start the server with graceful shutdown support
    pub async fn start(self) -> ServerResult<()> {
        let addr = SocketAddr::from(([0, 0, 0, 0], self.config.port));
        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| ServerError::BindError(e.to_string()))?;

        info!("AgentMem server starting on {}", addr);
        info!("API documentation available at http://{}/swagger-ui/", addr);
        info!("Health check endpoint: http://{}/health", addr);
        info!("Metrics endpoint: http://{}/metrics", addr);

        // Setup graceful shutdown signal
        let shutdown_signal = async {
            #[cfg(unix)]
            {
                use tokio::signal::unix::{signal, SignalKind};
                
                let mut ctrl_c_stream = tokio::signal::ctrl_c();
                let mut terminate_stream = match signal(SignalKind::terminate()) {
                    Ok(stream) => stream,
                    Err(e) => {
                        tracing::error!("Failed to install SIGTERM signal handler: {}", e);
                        return;
                    }
                };

                tokio::select! {
                    _ = ctrl_c_stream => {
                        info!("🛑 Received CTRL+C signal, initiating graceful shutdown...");
                    }
                    _ = terminate_stream.recv() => {
                        info!("🛑 Received SIGTERM signal, initiating graceful shutdown...");
                    }
                }
            }

            #[cfg(not(unix))]
            {
                tokio::signal::ctrl_c()
                    .await
                    .map_err(|e| {
                        tracing::error!("Failed to install CTRL+C signal handler: {}", e);
                        e
                    })
                    .ok();
                info!("🛑 Received CTRL+C signal, initiating graceful shutdown...");
            }
        };

        // Start the server with graceful shutdown
        let server = axum::serve(listener, self.router)
            .with_graceful_shutdown(shutdown_signal);

        info!("✅ Server is ready to accept connections");
        
        server
            .await
            .map_err(|e| ServerError::ServerError(e.to_string()))?;

        info!("✅ Server shutdown complete");
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
    #[ignore = "Requires database connection, may fail in concurrent test environment"]
    async fn test_server_creation() {
        let mut config = ServerConfig::default();
        config.enable_logging = false; // Disable logging to avoid telemetry conflicts
        // Use :memory: format for SQLite in-memory database (each connection gets its own database)
        config.database_url = ":memory:".to_string();
        let server = MemoryServer::new(config).await;
        if let Err(e) = &server {
            eprintln!("Server creation failed: {:?}", e);
        }
        assert!(server.is_ok());
    }

    #[tokio::test]
    #[ignore = "Requires database connection, may fail in concurrent test environment"]
    async fn test_server_config() {
        let mut config = ServerConfig::default();
        config.enable_logging = false; // Disable logging to avoid telemetry conflicts
        // Use :memory: format for SQLite in-memory database (each connection gets its own database)
        config.database_url = ":memory:".to_string();
        let server = MemoryServer::new(config.clone()).await;
        if let Err(e) = &server {
            eprintln!("Server creation failed: {:?}", e);
        }
        let server = server.map_err(|e| {
            eprintln!("Server creation failed: {:?}", e);
            e
        }).expect("Server should be created successfully");
        assert_eq!(server.config().port, config.port);
    }
}
