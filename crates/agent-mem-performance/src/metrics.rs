//! Performance metrics collection and monitoring
//!
//! This module provides comprehensive metrics collection for monitoring
//! AgentMem performance and identifying bottlenecks.

use agent_mem_traits::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Performance metrics collector
pub struct MetricsCollector {
    enabled: bool,
    metrics: Arc<RwLock<PerformanceMetrics>>,
    start_time: Instant,
}

/// Performance metrics data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub request_count: u64,
    pub error_count: u64,
    pub average_response_time_ms: f64,
    pub throughput_requests_per_second: f64,
    pub memory_usage_bytes: usize,
    pub cache_hit_rate: f64,
    pub active_connections: usize,
    pub custom_metrics: HashMap<String, f64>,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            request_count: 0,
            error_count: 0,
            average_response_time_ms: 0.0,
            throughput_requests_per_second: 0.0,
            memory_usage_bytes: 0,
            cache_hit_rate: 0.0,
            active_connections: 0,
            custom_metrics: HashMap::new(),
        }
    }
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(enabled: bool) -> Result<Self> {
        let collector = Self {
            enabled,
            metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            start_time: Instant::now(),
        };

        if enabled {
            collector.initialize_metrics()?;
            info!("Metrics collector initialized");
        }

        Ok(collector)
    }

    /// Record a request
    pub async fn record_request(&self, duration: Duration, success: bool) {
        if !self.enabled {
            return;
        }

        let mut metrics = self.metrics.write().await;
        metrics.request_count += 1;

        if !success {
            metrics.error_count += 1;
        }

        // Update average response time
        let duration_ms = duration.as_millis() as f64;
        metrics.average_response_time_ms =
            (metrics.average_response_time_ms * (metrics.request_count - 1) as f64 + duration_ms)
                / metrics.request_count as f64;

        // Update throughput
        let elapsed_seconds = self.start_time.elapsed().as_secs_f64();
        if elapsed_seconds > 0.0 {
            metrics.throughput_requests_per_second = metrics.request_count as f64 / elapsed_seconds;
        }

        #[cfg(feature = "metrics")]
        {
            // Simplified metrics implementation
            debug!(
                "Request recorded: duration={}ms, success={}",
                duration_ms, success
            );
        }
    }

    /// Update memory usage
    pub async fn update_memory_usage(&self, bytes: usize) {
        if !self.enabled {
            return;
        }

        let mut metrics = self.metrics.write().await;
        metrics.memory_usage_bytes = bytes;

        #[cfg(feature = "metrics")]
        {
            debug!("Memory usage updated: {} bytes", bytes);
        }
    }

    /// Update cache hit rate
    pub async fn update_cache_hit_rate(&self, hit_rate: f64) {
        if !self.enabled {
            return;
        }

        let mut metrics = self.metrics.write().await;
        metrics.cache_hit_rate = hit_rate;

        #[cfg(feature = "metrics")]
        {
            debug!("Cache hit rate updated: {}", hit_rate);
        }
    }

    /// Update active connections
    pub async fn update_active_connections(&self, count: usize) {
        if !self.enabled {
            return;
        }

        let mut metrics = self.metrics.write().await;
        metrics.active_connections = count;

        #[cfg(feature = "metrics")]
        {
            debug!("Active connections updated: {}", count);
        }
    }

    /// Record custom metric
    pub async fn record_custom_metric(&self, name: &str, value: f64) {
        if !self.enabled {
            return;
        }

        let mut metrics = self.metrics.write().await;
        metrics.custom_metrics.insert(name.to_string(), value);

        #[cfg(feature = "metrics")]
        {
            debug!("Custom metric '{}' updated: {}", name, value);
        }
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> Result<PerformanceMetrics> {
        Ok(self.metrics.read().await.clone())
    }

    /// Reset all metrics
    pub async fn reset(&self) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        *metrics = PerformanceMetrics::default();
        info!("Metrics reset");
        Ok(())
    }

    /// Shutdown the metrics collector
    pub async fn shutdown(&self) -> Result<()> {
        info!("Metrics collector shutdown");
        Ok(())
    }

    #[cfg(feature = "metrics")]
    fn initialize_metrics(&self) -> Result<()> {
        // Simplified metrics initialization
        info!("Metrics system initialized");
        Ok(())
    }

    #[cfg(not(feature = "metrics"))]
    fn initialize_metrics(&self) -> Result<()> {
        Ok(())
    }
}

/// Metrics middleware for timing operations
pub struct MetricsTimer {
    start_time: Instant,
    collector: Arc<MetricsCollector>,
}

impl MetricsTimer {
    /// Start a new timer
    pub fn start(collector: Arc<MetricsCollector>) -> Self {
        Self {
            start_time: Instant::now(),
            collector,
        }
    }

    /// Finish timing and record the result
    pub async fn finish(self, success: bool) {
        let duration = self.start_time.elapsed();
        self.collector.record_request(duration, success).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_metrics_collector_creation() {
        let collector = MetricsCollector::new(true);
        assert!(collector.is_ok());
    }

    #[tokio::test]
    async fn test_record_request() {
        let collector = MetricsCollector::new(true).unwrap();
        collector
            .record_request(Duration::from_millis(100), true)
            .await;

        let metrics = collector.get_metrics().await.unwrap();
        assert_eq!(metrics.request_count, 1);
        assert_eq!(metrics.error_count, 0);
    }

    #[tokio::test]
    async fn test_record_error() {
        let collector = MetricsCollector::new(true).unwrap();
        collector
            .record_request(Duration::from_millis(100), false)
            .await;

        let metrics = collector.get_metrics().await.unwrap();
        assert_eq!(metrics.request_count, 1);
        assert_eq!(metrics.error_count, 1);
    }

    #[tokio::test]
    async fn test_custom_metrics() {
        let collector = MetricsCollector::new(true).unwrap();
        collector.record_custom_metric("test_metric", 42.0).await;

        let metrics = collector.get_metrics().await.unwrap();
        assert_eq!(metrics.custom_metrics.get("test_metric"), Some(&42.0));
    }

    #[tokio::test]
    async fn test_metrics_timer() {
        let collector = Arc::new(MetricsCollector::new(true).unwrap());
        let timer = MetricsTimer::start(Arc::clone(&collector));

        tokio::time::sleep(Duration::from_millis(10)).await;
        timer.finish(true).await;

        let metrics = collector.get_metrics().await.unwrap();
        assert_eq!(metrics.request_count, 1);
        assert!(metrics.average_response_time_ms >= 10.0);
    }
}
