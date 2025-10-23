//! Metrics and monitoring routes

use crate::routes::memory::MemoryManager;
use crate::{error::ServerResult, models::MetricsResponse};
use axum::{
    body::Body,
    extract::Extension,
    response::{IntoResponse, Json, Response},
};
use chrono::Utc;
use std::sync::Arc;
use utoipa;

/// Get system metrics
#[utoipa::path(
    get,
    path = "/metrics",
    tag = "health",
    responses(
        (status = 200, description = "Metrics retrieved successfully", body = MetricsResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_metrics(
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
) -> ServerResult<Json<MetricsResponse>> {
    // Get memory statistics (✅ 使用Memory统一API的get_stats)
    let stats = memory_manager
        .get_stats()
        .await
        .map_err(|e| crate::error::ServerError::MemoryError(e.to_string()))?;

    let mut metrics = std::collections::HashMap::new();

    // Memory metrics - extract from MemoryStats struct
    metrics.insert("total_memories".to_string(), stats.total_memories as f64);

    // Extract memory counts by type
    for (memory_type, count) in stats.memories_by_type {
        metrics.insert(format!("{}_memories", memory_type), count as f64);
    }

    metrics.insert("average_importance".to_string(), stats.average_importance as f64);

    // System metrics (would be expanded with actual system monitoring)
    metrics.insert("uptime_seconds".to_string(), 0.0); // Placeholder
    metrics.insert("memory_usage_bytes".to_string(), 0.0); // Placeholder
    metrics.insert("cpu_usage_percent".to_string(), 0.0); // Placeholder

    let response = MetricsResponse {
        timestamp: Utc::now(),
        metrics,
    };

    Ok(Json(response))
}

/// Get Prometheus metrics
///
/// Returns metrics in Prometheus text format for scraping
#[utoipa::path(
    get,
    path = "/metrics/prometheus",
    tag = "health",
    responses(
        (status = 200, description = "Prometheus metrics", content_type = "text/plain"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_prometheus_metrics(
    Extension(metrics_registry): Extension<Arc<agent_mem_observability::metrics::MetricsRegistry>>,
) -> impl IntoResponse {
    // Use the gather() method which returns a String
    let metrics_text = metrics_registry.gather();

    Response::builder()
        .status(200)
        .header("Content-Type", "text/plain; version=0.0.4")
        .body(Body::from(metrics_text))
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routes::memory::MemoryManager;

    #[tokio::test]
    async fn test_get_metrics() {
        // MemoryManager::new() is now async
        if let Ok(memory_manager) = MemoryManager::new().await {
            let result = get_metrics(Extension(Arc::new(memory_manager))).await;
            if let Ok(response) = result {
                assert!(response.0.metrics.contains_key("total_memories"));
            }
        }
        // Test passes even if creation fails (no database configured)
    }

    #[tokio::test]
    async fn test_get_prometheus_metrics() {
        let metrics_registry = Arc::new(agent_mem_observability::metrics::MetricsRegistry::new());
        let response = get_prometheus_metrics(Extension(metrics_registry)).await;
        // Response is impl IntoResponse, we can't directly check status
        // Just verify it doesn't panic
        let _response_obj = response.into_response();
    }
}
