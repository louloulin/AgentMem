//! 性能监控和优化模块
//!
//! 提供存储性能监控、指标收集和优化建议功能

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info};

/// 性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// 操作类型
    pub operation: String,
    /// 执行时间（毫秒）
    pub duration_ms: f64,
    /// 成功标志
    pub success: bool,
    /// 数据大小（字节）
    pub data_size_bytes: Option<usize>,
    /// 记录数量
    pub record_count: Option<usize>,
    /// 时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// 额外元数据
    pub metadata: HashMap<String, String>,
}

/// 性能统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    /// 操作类型
    pub operation: String,
    /// 总请求数
    pub total_requests: u64,
    /// 成功请求数
    pub successful_requests: u64,
    /// 失败请求数
    pub failed_requests: u64,
    /// 成功率
    pub success_rate: f64,
    /// 平均响应时间（毫秒）
    pub avg_duration_ms: f64,
    /// 最小响应时间（毫秒）
    pub min_duration_ms: f64,
    /// 最大响应时间（毫秒）
    pub max_duration_ms: f64,
    /// P50响应时间（毫秒）
    pub p50_duration_ms: f64,
    /// P95响应时间（毫秒）
    pub p95_duration_ms: f64,
    /// P99响应时间（毫秒）
    pub p99_duration_ms: f64,
    /// 吞吐量（请求/秒）
    pub throughput_rps: f64,
    /// 平均数据大小（字节）
    pub avg_data_size_bytes: f64,
    /// 总数据传输量（字节）
    pub total_data_bytes: u64,
}

/// 性能监控器
pub struct PerformanceMonitor {
    /// 指标历史记录
    metrics_history: Arc<RwLock<Vec<PerformanceMetrics>>>,
    /// 最大历史记录数
    max_history_size: usize,
    /// 统计缓存
    stats_cache: Arc<RwLock<HashMap<String, PerformanceStats>>>,
    /// 缓存过期时间
    cache_ttl: Duration,
    /// 上次缓存更新时间
    last_cache_update: Arc<RwLock<Instant>>,
}

impl PerformanceMonitor {
    /// 创建新的性能监控器
    pub fn new(max_history_size: usize, cache_ttl: Duration) -> Self {
        Self {
            metrics_history: Arc::new(RwLock::new(Vec::new())),
            max_history_size,
            stats_cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl,
            last_cache_update: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// 记录性能指标
    pub async fn record_metric(&self, metric: PerformanceMetrics) {
        let mut history = self.metrics_history.write().await;

        // 添加新指标
        history.push(metric);

        // 限制历史记录大小
        if history.len() > self.max_history_size {
            history.remove(0);
        }

        // 清除统计缓存以强制重新计算
        let mut cache = self.stats_cache.write().await;
        cache.clear();

        debug!(
            "Recorded performance metric, history size: {}",
            history.len()
        );
    }

    /// 开始计时
    pub fn start_timer(&self) -> PerformanceTimer {
        PerformanceTimer::new()
    }

    /// 获取操作统计
    pub async fn get_stats(&self, operation: &str) -> Option<PerformanceStats> {
        // 检查缓存是否有效
        let last_update = *self.last_cache_update.read().await;
        if last_update.elapsed() < self.cache_ttl {
            let cache = self.stats_cache.read().await;
            if let Some(stats) = cache.get(operation) {
                return Some(stats.clone());
            }
        }

        // 重新计算统计
        self.calculate_stats(operation).await
    }

    /// 获取所有操作的统计
    pub async fn get_all_stats(&self) -> HashMap<String, PerformanceStats> {
        let history = self.metrics_history.read().await;
        let mut operations = std::collections::HashSet::new();

        // 收集所有操作类型
        for metric in history.iter() {
            operations.insert(metric.operation.clone());
        }

        let mut all_stats = HashMap::new();
        for operation in operations {
            if let Some(stats) = self.calculate_stats(&operation).await {
                all_stats.insert(operation, stats);
            }
        }

        all_stats
    }

    /// 计算指定操作的统计信息
    async fn calculate_stats(&self, operation: &str) -> Option<PerformanceStats> {
        let history = self.metrics_history.read().await;

        // 过滤指定操作的指标
        let operation_metrics: Vec<_> = history
            .iter()
            .filter(|m| m.operation == operation)
            .collect();

        if operation_metrics.is_empty() {
            return None;
        }

        let total_requests = operation_metrics.len() as u64;
        let successful_requests = operation_metrics.iter().filter(|m| m.success).count() as u64;
        let failed_requests = total_requests - successful_requests;
        let success_rate = successful_requests as f64 / total_requests as f64;

        // 计算响应时间统计
        let mut durations: Vec<f64> = operation_metrics.iter().map(|m| m.duration_ms).collect();
        durations.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let avg_duration_ms = durations.iter().sum::<f64>() / durations.len() as f64;
        let min_duration_ms = durations.first().copied().unwrap_or(0.0);
        let max_duration_ms = durations.last().copied().unwrap_or(0.0);

        // 计算百分位数
        let p50_duration_ms = percentile(&durations, 0.5);
        let p95_duration_ms = percentile(&durations, 0.95);
        let p99_duration_ms = percentile(&durations, 0.99);

        // 计算吞吐量
        let time_span = if operation_metrics.len() > 1 {
            let first_time = operation_metrics.first().unwrap().timestamp;
            let last_time = operation_metrics.last().unwrap().timestamp;
            (last_time - first_time).num_seconds() as f64
        } else {
            1.0
        };
        let throughput_rps = if time_span > 0.0 {
            total_requests as f64 / time_span
        } else {
            0.0
        };

        // 计算数据大小统计
        let data_sizes: Vec<usize> = operation_metrics
            .iter()
            .filter_map(|m| m.data_size_bytes)
            .collect();

        let avg_data_size_bytes = if !data_sizes.is_empty() {
            data_sizes.iter().sum::<usize>() as f64 / data_sizes.len() as f64
        } else {
            0.0
        };

        let total_data_bytes = data_sizes.iter().sum::<usize>() as u64;

        let stats = PerformanceStats {
            operation: operation.to_string(),
            total_requests,
            successful_requests,
            failed_requests,
            success_rate,
            avg_duration_ms,
            min_duration_ms,
            max_duration_ms,
            p50_duration_ms,
            p95_duration_ms,
            p99_duration_ms,
            throughput_rps,
            avg_data_size_bytes,
            total_data_bytes,
        };

        // 更新缓存
        let mut cache = self.stats_cache.write().await;
        cache.insert(operation.to_string(), stats.clone());
        *self.last_cache_update.write().await = Instant::now();

        Some(stats)
    }

    /// 清除历史记录
    pub async fn clear_history(&self) {
        let mut history = self.metrics_history.write().await;
        history.clear();

        let mut cache = self.stats_cache.write().await;
        cache.clear();

        info!("Performance history cleared");
    }

    /// 获取历史记录数量
    pub async fn history_size(&self) -> usize {
        let history = self.metrics_history.read().await;
        history.len()
    }

    /// 生成性能报告
    pub async fn generate_report(&self) -> PerformanceReport {
        let all_stats = self.get_all_stats().await;
        let history_size = self.history_size().await;

        PerformanceReport {
            timestamp: chrono::Utc::now(),
            total_metrics: history_size,
            operation_stats: all_stats,
        }
    }
}

/// 性能计时器
pub struct PerformanceTimer {
    start_time: Instant,
}

impl PerformanceTimer {
    fn new() -> Self {
        Self {
            start_time: Instant::now(),
        }
    }

    /// 完成计时并创建指标
    pub fn finish(
        self,
        operation: String,
        success: bool,
        data_size_bytes: Option<usize>,
        record_count: Option<usize>,
        metadata: HashMap<String, String>,
    ) -> PerformanceMetrics {
        let duration = self.start_time.elapsed();

        PerformanceMetrics {
            operation,
            duration_ms: duration.as_secs_f64() * 1000.0,
            success,
            data_size_bytes,
            record_count,
            timestamp: chrono::Utc::now(),
            metadata,
        }
    }
}

/// 性能报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    /// 报告生成时间
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// 总指标数量
    pub total_metrics: usize,
    /// 各操作统计
    pub operation_stats: HashMap<String, PerformanceStats>,
}

/// 计算百分位数
fn percentile(sorted_values: &[f64], p: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }

    let index = (p * (sorted_values.len() - 1) as f64).round() as usize;
    sorted_values.get(index).copied().unwrap_or(0.0)
}

/// 带性能监控的存储包装器
pub struct MonitoredVectorStore {
    /// 底层存储
    inner: Arc<dyn agent_mem_traits::VectorStore + Send + Sync>,
    /// 性能监控器
    monitor: Arc<PerformanceMonitor>,
}

impl MonitoredVectorStore {
    /// 创建新的带性能监控的存储
    pub fn new(
        inner: Arc<dyn agent_mem_traits::VectorStore + Send + Sync>,
        monitor: Arc<PerformanceMonitor>,
    ) -> Self {
        Self { inner, monitor }
    }

    /// 获取性能监控器
    pub fn monitor(&self) -> &Arc<PerformanceMonitor> {
        &self.monitor
    }
}

#[async_trait::async_trait]
impl agent_mem_traits::VectorStore for MonitoredVectorStore {
    async fn add_vectors(
        &self,
        vectors: Vec<agent_mem_traits::VectorData>,
    ) -> agent_mem_traits::Result<Vec<String>> {
        let timer = self.monitor.start_timer();
        let data_size = vectors.iter().map(|v| v.vector.len() * 4).sum(); // 假设f32
        let record_count = vectors.len();

        let result = self.inner.add_vectors(vectors).await;

        let metric = timer.finish(
            "add_vectors".to_string(),
            result.is_ok(),
            Some(data_size),
            Some(record_count),
            HashMap::new(),
        );

        self.monitor.record_metric(metric).await;
        result
    }

    async fn search_vectors(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        threshold: Option<f32>,
    ) -> agent_mem_traits::Result<Vec<agent_mem_traits::VectorSearchResult>> {
        let timer = self.monitor.start_timer();
        let data_size = query_vector.len() * 4; // f32 size

        let result = self
            .inner
            .search_vectors(query_vector, limit, threshold)
            .await;

        let record_count = result.as_ref().map(|r| r.len()).unwrap_or(0);
        let metric = timer.finish(
            "search_vectors".to_string(),
            result.is_ok(),
            Some(data_size),
            Some(record_count),
            HashMap::new(),
        );

        self.monitor.record_metric(metric).await;
        result
    }

    async fn delete_vectors(&self, ids: Vec<String>) -> agent_mem_traits::Result<()> {
        let timer = self.monitor.start_timer();
        let record_count = ids.len();

        let result = self.inner.delete_vectors(ids).await;

        let metric = timer.finish(
            "delete_vectors".to_string(),
            result.is_ok(),
            None,
            Some(record_count),
            HashMap::new(),
        );

        self.monitor.record_metric(metric).await;
        result
    }

    async fn update_vectors(
        &self,
        vectors: Vec<agent_mem_traits::VectorData>,
    ) -> agent_mem_traits::Result<()> {
        let timer = self.monitor.start_timer();
        let data_size = vectors.iter().map(|v| v.vector.len() * 4).sum();
        let record_count = vectors.len();

        let result = self.inner.update_vectors(vectors).await;

        let metric = timer.finish(
            "update_vectors".to_string(),
            result.is_ok(),
            Some(data_size),
            Some(record_count),
            HashMap::new(),
        );

        self.monitor.record_metric(metric).await;
        result
    }

    async fn get_vector(
        &self,
        id: &str,
    ) -> agent_mem_traits::Result<Option<agent_mem_traits::VectorData>> {
        let timer = self.monitor.start_timer();

        let result = self.inner.get_vector(id).await;

        let data_size = result
            .as_ref()
            .ok()
            .and_then(|opt| opt.as_ref().map(|v| v.vector.len() * 4));
        let metric = timer.finish(
            "get_vector".to_string(),
            result.is_ok(),
            data_size,
            Some(1),
            HashMap::new(),
        );

        self.monitor.record_metric(metric).await;
        result
    }

    async fn count_vectors(&self) -> agent_mem_traits::Result<usize> {
        let timer = self.monitor.start_timer();

        let result = self.inner.count_vectors().await;

        let metric = timer.finish(
            "count_vectors".to_string(),
            result.is_ok(),
            None,
            None,
            HashMap::new(),
        );

        self.monitor.record_metric(metric).await;
        result
    }

    async fn clear(&self) -> agent_mem_traits::Result<()> {
        let timer = self.monitor.start_timer();

        let result = self.inner.clear().await;

        let metric = timer.finish(
            "clear".to_string(),
            result.is_ok(),
            None,
            None,
            HashMap::new(),
        );

        self.monitor.record_metric(metric).await;
        result
    }

    async fn search_with_filters(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        filters: &HashMap<String, serde_json::Value>,
        threshold: Option<f32>,
    ) -> agent_mem_traits::Result<Vec<agent_mem_traits::VectorSearchResult>> {
        let timer = self.monitor.start_timer();
        let data_size = query_vector.len() * 4;

        let result = self
            .inner
            .search_with_filters(query_vector, limit, filters, threshold)
            .await;

        let record_count = result.as_ref().map(|r| r.len()).unwrap_or(0);
        let metric = timer.finish(
            "search_with_filters".to_string(),
            result.is_ok(),
            Some(data_size),
            Some(record_count),
            HashMap::new(),
        );

        self.monitor.record_metric(metric).await;
        result
    }

    async fn health_check(&self) -> agent_mem_traits::Result<agent_mem_traits::HealthStatus> {
        let timer = self.monitor.start_timer();

        let result = self.inner.health_check().await;

        let metric = timer.finish(
            "health_check".to_string(),
            result.is_ok(),
            None,
            None,
            HashMap::new(),
        );

        self.monitor.record_metric(metric).await;
        result
    }

    async fn get_stats(&self) -> agent_mem_traits::Result<agent_mem_traits::VectorStoreStats> {
        let timer = self.monitor.start_timer();

        let result = self.inner.get_stats().await;

        let metric = timer.finish(
            "get_stats".to_string(),
            result.is_ok(),
            None,
            None,
            HashMap::new(),
        );

        self.monitor.record_metric(metric).await;
        result
    }

    async fn add_vectors_batch(
        &self,
        batches: Vec<Vec<agent_mem_traits::VectorData>>,
    ) -> agent_mem_traits::Result<Vec<Vec<String>>> {
        let timer = self.monitor.start_timer();
        let data_size: usize = batches.iter().flatten().map(|v| v.vector.len() * 4).sum();
        let record_count: usize = batches.iter().map(|b| b.len()).sum();

        let result = self.inner.add_vectors_batch(batches).await;

        let metric = timer.finish(
            "add_vectors_batch".to_string(),
            result.is_ok(),
            Some(data_size),
            Some(record_count),
            HashMap::new(),
        );

        self.monitor.record_metric(metric).await;
        result
    }

    async fn delete_vectors_batch(
        &self,
        id_batches: Vec<Vec<String>>,
    ) -> agent_mem_traits::Result<Vec<bool>> {
        let timer = self.monitor.start_timer();
        let record_count: usize = id_batches.iter().map(|b| b.len()).sum();

        let result = self.inner.delete_vectors_batch(id_batches).await;

        let metric = timer.finish(
            "delete_vectors_batch".to_string(),
            result.is_ok(),
            None,
            Some(record_count),
            HashMap::new(),
        );

        self.monitor.record_metric(metric).await;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_performance_monitor() {
        let monitor = PerformanceMonitor::new(1000, Duration::from_secs(60));

        // 记录一些测试指标
        let metric1 = PerformanceMetrics {
            operation: "search".to_string(),
            duration_ms: 100.0,
            success: true,
            data_size_bytes: Some(1024),
            record_count: Some(10),
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        monitor.record_metric(metric1).await;

        let metric2 = PerformanceMetrics {
            operation: "search".to_string(),
            duration_ms: 150.0,
            success: true,
            data_size_bytes: Some(2048),
            record_count: Some(20),
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        monitor.record_metric(metric2).await;

        // 获取统计信息
        let stats = monitor.get_stats("search").await.unwrap();
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.successful_requests, 2);
        assert_eq!(stats.success_rate, 1.0);
        assert_eq!(stats.avg_duration_ms, 125.0);
    }

    #[tokio::test]
    async fn test_performance_timer() {
        let timer = PerformanceTimer::new();
        sleep(Duration::from_millis(10)).await;

        let metric = timer.finish("test".to_string(), true, Some(100), Some(1), HashMap::new());

        assert!(metric.duration_ms >= 10.0);
        assert_eq!(metric.operation, "test");
        assert!(metric.success);
    }
}
