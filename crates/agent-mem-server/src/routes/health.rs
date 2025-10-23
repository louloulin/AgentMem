//! Health Check Endpoints
//!
//! 简化版本，与当前架构兼容

use axum::{http::StatusCode, response::Json};
use crate::error::ServerResult;
use crate::models::HealthResponse;

/// GET /health - 健康检查
///
/// 返回服务健康状态
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse),
    )
)]
pub async fn health_check() -> ServerResult<(StatusCode, Json<HealthResponse>)> {
    let response = HealthResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        checks: std::collections::HashMap::new(), // Empty checks for simplified version
    };

    Ok((StatusCode::OK, Json(response)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check() {
        let result = health_check().await;
        assert!(result.is_ok());
        let (_status, response) = result.unwrap();
        assert_eq!(response.0.status, "healthy");
    }
}
