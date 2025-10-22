//! Health Check Endpoints
//!
//! Phase 10 MVP P0-2: 健康检查端点实现
//!
//! 提供三个级别的健康检查：
//! - /health: 完整健康检查（包含组件状态）
//! - /health/live: 存活检查（服务是否运行）
//! - /health/ready: 就绪检查（是否可处理请求）

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::server::AppState;

/// 健康检查响应
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    /// 整体状态: "healthy", "degraded", "unhealthy"
    pub status: String,
    /// 服务版本
    pub version: String,
    /// 检查时间戳
    pub timestamp: String,
    /// 各组件健康状态
    pub components: ComponentsHealth,
}

/// 组件健康状态
#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentsHealth {
    /// Memory API状态
    pub memory_api: ComponentStatus,
    /// 向量存储状态
    pub vector_store: ComponentStatus,
    /// 历史数据库状态
    pub history_db: ComponentStatus,
}

/// 单个组件状态
#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentStatus {
    /// 状态: "healthy", "degraded", "unhealthy"
    pub status: String,
    /// 响应延迟（毫秒）
    pub latency_ms: Option<u64>,
    /// 状态消息
    pub message: Option<String>,
}

/// 简单健康检查响应
#[derive(Debug, Serialize)]
pub struct SimpleHealthResponse {
    pub status: String,
}

/// GET /health - 完整健康检查
///
/// 返回详细的组件健康状态
async fn health_check(
    State(state): State<Arc<AppState>>,
) -> Result<Json<HealthResponse>, StatusCode> {
    use chrono::Utc;
    
    // 检查各个组件（并行）
    let (memory_status, vector_status, db_status) = tokio::join!(
        check_memory_api(&state),
        check_vector_store(&state),
        check_history_db(&state),
    );
    
    // 确定整体状态
    let overall_status = if memory_status.status == "healthy"
        && vector_status.status == "healthy"
        && db_status.status == "healthy"
    {
        "healthy"
    } else if memory_status.status == "unhealthy"
        || vector_status.status == "unhealthy"
        || db_status.status == "unhealthy"
    {
        "unhealthy"
    } else {
        "degraded"
    }
    .to_string();
    
    Ok(Json(HealthResponse {
        status: overall_status,
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: Utc::now().to_rfc3339(),
        components: ComponentsHealth {
            memory_api: memory_status,
            vector_store: vector_status,
            history_db: db_status,
        },
    }))
}

/// GET /health/live - 存活检查
///
/// 快速检查服务是否运行（用于k8s liveness probe）
async fn liveness_check() -> Json<SimpleHealthResponse> {
    Json(SimpleHealthResponse {
        status: "alive".to_string(),
    })
}

/// GET /health/ready - 就绪检查
///
/// 检查服务是否就绪处理请求（用于k8s readiness probe）
async fn readiness_check(
    State(state): State<Arc<AppState>>,
) -> Result<Json<SimpleHealthResponse>, StatusCode> {
    // 快速检查Memory API是否可用
    let start = std::time::Instant::now();
    
    // 尝试获取stats（轻量级操作）
    match state.memory.get_stats().await {
        Ok(_) => {
            let latency = start.elapsed().as_millis();
            if latency < 1000 {
                // 响应时间小于1秒，认为就绪
                Ok(Json(SimpleHealthResponse {
                    status: "ready".to_string(),
                }))
            } else {
                // 响应太慢，暂不就绪
                Err(StatusCode::SERVICE_UNAVAILABLE)
            }
        }
        Err(_) => Err(StatusCode::SERVICE_UNAVAILABLE),
    }
}

/// 检查Memory API健康状态
async fn check_memory_api(state: &AppState) -> ComponentStatus {
    let start = std::time::Instant::now();
    
    match state.memory.get_stats().await {
        Ok(_stats) => {
            let latency = start.elapsed().as_millis() as u64;
            ComponentStatus {
                status: "healthy".to_string(),
                latency_ms: Some(latency),
                message: None,
            }
        }
        Err(e) => ComponentStatus {
            status: "unhealthy".to_string(),
            latency_ms: None,
            message: Some(format!("Memory API error: {}", e)),
        },
    }
}

/// 检查向量存储健康状态
async fn check_vector_store(_state: &AppState) -> ComponentStatus {
    // 简化检查：假设可用
    // TODO: 实际检查向量存储连接
    ComponentStatus {
        status: "healthy".to_string(),
        latency_ms: Some(1),
        message: Some("Using in-memory vector store".to_string()),
    }
}

/// 检查历史数据库健康状态
async fn check_history_db(_state: &AppState) -> ComponentStatus {
    // 简化检查：假设SQLite可用
    // TODO: 实际检查history.db文件
    let history_path = "./data/history.db";
    
    if std::path::Path::new(history_path).exists() {
        ComponentStatus {
            status: "healthy".to_string(),
            latency_ms: Some(1),
            message: Some(format!("History DB: {}", history_path)),
        }
    } else {
        ComponentStatus {
            status: "degraded".to_string(),
            latency_ms: None,
            message: Some("History DB not initialized".to_string()),
        }
    }
}

/// 创建健康检查路由器
pub fn health_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/health", get(health_check))
        .route("/health/live", get(liveness_check))
        .route("/health/ready", get(readiness_check))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_response_serialization() {
        let response = HealthResponse {
            status: "healthy".to_string(),
            version: "1.0.0".to_string(),
            timestamp: "2025-10-22T10:00:00Z".to_string(),
            components: ComponentsHealth {
                memory_api: ComponentStatus {
                    status: "healthy".to_string(),
                    latency_ms: Some(5),
                    message: None,
                },
                vector_store: ComponentStatus {
                    status: "healthy".to_string(),
                    latency_ms: Some(2),
                    message: None,
                },
                history_db: ComponentStatus {
                    status: "healthy".to_string(),
                    latency_ms: Some(1),
                    message: None,
                },
            },
        };
        
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("healthy"));
        assert!(json.contains("memory_api"));
    }
}
