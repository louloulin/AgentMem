//! AgentMem REST API Server
//!
//! Enterprise-grade REST API server for AgentMem memory management platform.
//! Provides HTTP endpoints for all memory operations with authentication,
//! multi-tenancy, and comprehensive monitoring.

pub mod auth;
pub mod background_tasks; // 🔴 Phase 2: Forgetting background tasks
pub mod config;
pub mod error;
pub mod error_handler; // ✅ Phase 0.1: 统一错误处理模块
pub mod middleware;
pub mod models;
pub mod orchestrator_factory;
pub mod rbac;
pub mod routes;
pub mod server;
pub mod sse;
pub mod telemetry;
pub mod websocket;

// Re-export middleware for external use
pub use middleware::scope_middleware::{ScopeMiddlewareState, ScopeExt, extract_scope_from_request, check_scope_access, validate_access};

pub use config::ServerConfig;
pub use error::{ServerError, ServerResult};
pub use error_handler::{safe_expect, safe_unwrap, ErrorHandler, ErrorMonitor}; // ✅ Phase 0.1: 导出错误处理工具
pub use server::MemoryServer;

/// Re-export commonly used types
pub use models::{
    ApplyMigrationRequest, BatchRequest, BatchResponse, CancelProactiveTaskRequest,
    CategoryDescriptor, CategoryMetadataDescriptor, CategoryStatus, ExtractedEntity,
    ExtractedRelation, ExtractionRequest, ExtractionResult, HealthResponse, MemoryRequest,
    MemoryResponse, MetricsResponse, MigrationPlan, MigrationReport, MountResourceRequest,
    OperationStatus, PlanMigrationRequest, PlatformErrorCode, ProactiveTaskInfo,
    ResourceDescriptor, ResourceMetadataDescriptor, ResourceStatus, RollbackMigrationRequest,
    RunProactiveTaskRequest, SchedulerState, SchedulerStats, ScopeDescriptor,
    SearchCategoriesRequest, SearchRequest, SearchResponse,
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
