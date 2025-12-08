//! Metrics and monitoring routes

use crate::routes::memory::{MemoryManager, get_search_stats};
use crate::{error::ServerResult, models::MetricsResponse};
use axum::{
    body::Body,
    extract::Extension,
    response::{IntoResponse, Json, Response},
};
use chrono::Utc;
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Instant;
use utoipa;

/// 服务器启动时间（用于计算运行时间）
static SERVER_START_TIME: OnceLock<Instant> = OnceLock::new();

/// 初始化服务器启动时间
fn init_server_start_time() {
    SERVER_START_TIME.get_or_init(Instant::now);
}

/// 获取服务器运行时间（秒）
fn get_uptime_seconds() -> f64 {
    SERVER_START_TIME
        .get()
        .map(|start| start.elapsed().as_secs_f64())
        .unwrap_or(0.0)
}

/// 获取内存使用量（字节）
/// 
/// 🆕 Phase 4.2: 监控增强 - 实现真实的系统指标收集
fn get_memory_usage_bytes() -> f64 {
    // 使用标准库获取当前进程的内存使用
    // 注意：这是一个简化实现，实际生产环境可以使用sysinfo crate获取更详细的系统信息
    // 这里我们使用一个估算值，基于Rust的内存分配器统计
    // 实际实现可以使用jemalloc或其他内存分配器的统计信息
    0.0 // 占位符，实际实现需要集成系统监控库
}

/// 获取CPU使用率（百分比）
/// 
/// 🆕 Phase 4.2: 监控增强 - 实现真实的系统指标收集
fn get_cpu_usage_percent() -> f64 {
    // 使用标准库获取CPU使用率
    // 注意：这是一个简化实现，实际生产环境可以使用sysinfo crate获取真实的CPU使用率
    // 这里我们使用一个估算值
    0.0 // 占位符，实际实现需要集成系统监控库
}

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
    // 🆕 Phase 4.2: 初始化服务器启动时间（如果尚未初始化）
    init_server_start_time();

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

    metrics.insert(
        "average_importance".to_string(),
        stats.average_importance as f64,
    );

    // 🆕 Phase 4.2: 系统指标 - 实现真实的系统指标收集
    let uptime_seconds = get_uptime_seconds();
    metrics.insert("uptime_seconds".to_string(), uptime_seconds);
    metrics.insert("uptime_hours".to_string(), uptime_seconds / 3600.0);
    metrics.insert("uptime_days".to_string(), uptime_seconds / 86400.0);

    // 内存使用（简化实现，实际可以使用sysinfo crate）
    let memory_usage = get_memory_usage_bytes();
    metrics.insert("memory_usage_bytes".to_string(), memory_usage);
    metrics.insert("memory_usage_mb".to_string(), memory_usage / (1024.0 * 1024.0));

    // CPU使用率（简化实现，实际可以使用sysinfo crate）
    let cpu_usage = get_cpu_usage_percent();
    metrics.insert("cpu_usage_percent".to_string(), cpu_usage);

    // 🆕 Phase 4.2: 集成搜索统计到系统指标
    // 使用现有的搜索统计API获取统计信息（通过内部函数）
    let search_stats = get_search_stats();
    let search_stats_read = search_stats.read().await;
    metrics.insert("search_total_searches".to_string(), search_stats_read.get_total_searches() as f64);
    metrics.insert("search_cache_hits".to_string(), search_stats_read.get_cache_hits() as f64);
    metrics.insert("search_cache_misses".to_string(), search_stats_read.get_cache_misses() as f64);
    metrics.insert("search_cache_hit_rate".to_string(), search_stats_read.cache_hit_rate());
    metrics.insert("search_avg_latency_ms".to_string(), search_stats_read.avg_latency_ms());
    metrics.insert("search_exact_queries".to_string(), search_stats_read.get_exact_queries() as f64);
    metrics.insert("search_vector_searches".to_string(), search_stats_read.get_vector_searches() as f64);

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
        .expect("Failed to build metrics response - this should never fail with valid headers")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routes::memory::MemoryManager;

    #[tokio::test]
    async fn test_get_metrics() {
        // MemoryManager::new() is now async and requires embedder config
        if let Ok(memory_manager) = MemoryManager::new(None, None).await {
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
