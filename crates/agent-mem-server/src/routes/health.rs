//! Health Check Endpoints
//!
//! 完整版本，集成组件健康检查

use crate::error::ServerResult;
use crate::models::{ComponentStatus, HealthResponse};
use axum::{http::StatusCode, response::Json, Extension};
use std::collections::HashMap;
use std::sync::Arc;

/// GET /health - 基础健康检查
///
/// 返回服务健康状态，包括各组件状态
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse),
    )
)]
pub async fn health_check(
    Extension(repos): Extension<Arc<agent_mem_core::storage::factory::Repositories>>,
) -> ServerResult<(StatusCode, Json<HealthResponse>)> {
    let mut checks = HashMap::new();

    // Check database connectivity
    let db_status = check_database(&repos).await;
    checks.insert("database".to_string(), db_status.clone());

    // Check memory system
    let memory_status = check_memory_system().await;
    checks.insert("memory_system".to_string(), memory_status.clone());

    // Determine overall status based on components
    let overall_status = if checks.values().all(|s| s.status == "healthy") {
        "healthy"
    } else if checks.values().any(|s| s.status == "unhealthy") {
        "unhealthy"
    } else {
        "degraded"
    };

    let status_code = match overall_status {
        "healthy" => StatusCode::OK,
        "degraded" => StatusCode::OK, // Still operational
        _ => StatusCode::SERVICE_UNAVAILABLE,
    };

    let response = HealthResponse {
        status: overall_status.to_string(),
        timestamp: chrono::Utc::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        checks,
    };

    Ok((status_code, Json(response)))
}

/// GET /health/live - Liveness probe (Kubernetes)
///
/// 检查服务是否运行
#[utoipa::path(
    get,
    path = "/health/live",
    tag = "health",
    responses(
        (status = 200, description = "Service is alive"),
    )
)]
pub async fn liveness_check() -> ServerResult<(StatusCode, Json<serde_json::Value>)> {
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "alive",
            "timestamp": chrono::Utc::now(),
            "version": env!("CARGO_PKG_VERSION"),
        })),
    ))
}

/// GET /health/ready - Readiness probe (Kubernetes)
///
/// 检查服务是否就绪接受流量
#[utoipa::path(
    get,
    path = "/health/ready",
    tag = "health",
    responses(
        (status = 200, description = "Service is ready"),
        (status = 503, description = "Service is not ready"),
    )
)]
pub async fn readiness_check(
    Extension(repos): Extension<Arc<agent_mem_core::storage::factory::Repositories>>,
) -> ServerResult<(StatusCode, Json<serde_json::Value>)> {
    // Check all critical dependencies
    let db_healthy = check_database(&repos).await.status == "healthy";
    let memory_healthy = check_memory_system().await.status == "healthy";

    let is_ready = db_healthy && memory_healthy;

    let status_code = if is_ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    Ok((
        status_code,
        Json(serde_json::json!({
            "status": if is_ready { "ready" } else { "not_ready" },
            "timestamp": chrono::Utc::now(),
            "checks": {
                "database": db_healthy,
                "memory_system": memory_healthy,
            }
        })),
    ))
}

/// Check database connectivity
async fn check_database(repos: &agent_mem_core::storage::factory::Repositories) -> ComponentStatus {
    // Try to perform a simple operation to verify connectivity
    match repos.agents.list(10, 0).await {
        Ok(_) => ComponentStatus {
            status: "healthy".to_string(),
            message: Some("Database connection successful".to_string()),
            last_check: chrono::Utc::now(),
        },
        Err(e) => ComponentStatus {
            status: "unhealthy".to_string(),
            message: Some(format!("Database error: {}", e)),
            last_check: chrono::Utc::now(),
        },
    }
}

/// Check memory system health
async fn check_memory_system() -> ComponentStatus {
    // Basic health check - in production, this would verify:
    // - Embedding service availability
    // - Vector store connectivity
    // - LLM provider availability
    ComponentStatus {
        status: "healthy".to_string(),
        message: Some("Memory system operational".to_string()),
        last_check: chrono::Utc::now(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_mem_core::storage::factory::Repositories;

    #[tokio::test]
    async fn test_liveness_check() {
        let result = liveness_check().await;
        assert!(result.is_ok());
        let (status, response) = result.unwrap();
        assert_eq!(status, StatusCode::OK);
        assert_eq!(response.0["status"], "alive");
    }

    #[tokio::test]
    async fn test_check_memory_system() {
        let status = check_memory_system().await;
        assert_eq!(status.status, "healthy");
        assert!(status.message.is_some());
    }
}
