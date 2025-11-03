//! AgentMem REST API Server
//!
//! Enterprise-grade REST API server for AgentMem memory management platform.
//! Provides HTTP endpoints for all memory operations with authentication,
//! multi-tenancy, and comprehensive monitoring.

pub mod auth;
pub mod config;
pub mod error;
pub mod middleware;
pub mod models;
pub mod orchestrator_factory;
pub mod rbac;
pub mod routes;
pub mod server;
pub mod sse;
pub mod telemetry;
pub mod websocket;

pub use config::ServerConfig;
pub use error::{ServerError, ServerResult};
pub use server::MemoryServer;

/// Re-export commonly used types
pub use models::{
    BatchRequest, BatchResponse, HealthResponse, MemoryRequest, MemoryResponse, MetricsResponse,
    SearchRequest, SearchResponse,
};

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_server_creation() {
        let mut config = ServerConfig::default();
        config.database_url = ":memory:".to_string(); // Use LibSql in-memory database for testing
        let server = MemoryServer::new(config).await;
        assert!(server.is_ok());
    }
}
