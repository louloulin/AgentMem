//! Prometheus metrics collection
//!
//! This module provides Prometheus metrics collection and export.

use crate::error::{ObservabilityError, ObservabilityResult};
use prometheus::{CounterVec, Gauge, HistogramOpts, HistogramVec, Opts, Registry};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Duration;

/// Metrics registry
pub struct MetricsRegistry {
    registry: Arc<Registry>,
    collectors: Arc<RwLock<MetricsCollectors>>,
}

/// Collection of all metrics
struct MetricsCollectors {
    // Counters
    requests_total: CounterVec,
    errors_total: CounterVec,

    // Gauges
    active_connections: Gauge,
    memory_usage_bytes: Gauge,
    cpu_usage_percent: Gauge,
    system_memory_total_bytes: Gauge,
    system_memory_used_bytes: Gauge,
    system_memory_available_bytes: Gauge,

    // Histograms
    request_duration_seconds: HistogramVec,
    tool_execution_duration_seconds: HistogramVec,
}

impl MetricsRegistry {
    /// Create a new metrics registry
    pub fn new() -> Self {
        let registry = Registry::new();

        // Create collectors
        let collectors = MetricsCollectors {
            requests_total: CounterVec::new(
                Opts::new("agentmem_requests_total", "Total number of requests"),
                &["method", "endpoint", "status"],
            )
            .unwrap(),

            errors_total: CounterVec::new(
                Opts::new("agentmem_errors_total", "Total number of errors"),
                &["error_type"],
            )
            .unwrap(),

            active_connections: Gauge::new(
                "agentmem_active_connections",
                "Number of active connections",
            )
            .unwrap(),

            memory_usage_bytes: Gauge::new("agentmem_memory_usage_bytes", "Memory usage in bytes")
                .unwrap(),

            cpu_usage_percent: Gauge::new(
                "agentmem_cpu_usage_percent",
                "CPU usage percentage (0-100)",
            )
            .unwrap(),

            system_memory_total_bytes: Gauge::new(
                "agentmem_system_memory_total_bytes",
                "Total system memory in bytes",
            )
            .unwrap(),

            system_memory_used_bytes: Gauge::new(
                "agentmem_system_memory_used_bytes",
                "Used system memory in bytes",
            )
            .unwrap(),

            system_memory_available_bytes: Gauge::new(
                "agentmem_system_memory_available_bytes",
                "Available system memory in bytes",
            )
            .unwrap(),

            request_duration_seconds: HistogramVec::new(
                HistogramOpts::new(
                    "agentmem_request_duration_seconds",
                    "Request duration in seconds",
                )
                .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0]),
                &["method", "endpoint"],
            )
            .unwrap(),

            tool_execution_duration_seconds: HistogramVec::new(
                HistogramOpts::new(
                    "agentmem_tool_execution_duration_seconds",
                    "Tool execution duration in seconds",
                )
                .buckets(vec![0.0001, 0.0005, 0.001, 0.005, 0.01, 0.05, 0.1]),
                &["tool_name"],
            )
            .unwrap(),
        };

        // Register collectors
        registry
            .register(Box::new(collectors.requests_total.clone()))
            .unwrap();
        registry
            .register(Box::new(collectors.errors_total.clone()))
            .unwrap();
        registry
            .register(Box::new(collectors.active_connections.clone()))
            .unwrap();
        registry
            .register(Box::new(collectors.memory_usage_bytes.clone()))
            .unwrap();
        registry
            .register(Box::new(collectors.cpu_usage_percent.clone()))
            .unwrap();
        registry
            .register(Box::new(collectors.system_memory_total_bytes.clone()))
            .unwrap();
        registry
            .register(Box::new(collectors.system_memory_used_bytes.clone()))
            .unwrap();
        registry
            .register(Box::new(collectors.system_memory_available_bytes.clone()))
            .unwrap();
        registry
            .register(Box::new(collectors.request_duration_seconds.clone()))
            .unwrap();
        registry
            .register(Box::new(collectors.tool_execution_duration_seconds.clone()))
            .unwrap();

        Self {
            registry: Arc::new(registry),
            collectors: Arc::new(RwLock::new(collectors)),
        }
    }

    /// Get the Prometheus registry
    pub fn registry(&self) -> Arc<Registry> {
        self.registry.clone()
    }

    /// Get metrics collector
    pub fn collector(&self) -> MetricsCollector {
        MetricsCollector {
            collectors: self.collectors.clone(),
        }
    }

    /// Gather metrics in Prometheus format
    pub fn gather(&self) -> String {
        use prometheus::Encoder;
        let encoder = prometheus::TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Metrics collector for recording metrics
#[derive(Clone)]
pub struct MetricsCollector {
    collectors: Arc<RwLock<MetricsCollectors>>,
}

impl MetricsCollector {
    /// Record a request
    pub async fn record_request(&self, method: &str, endpoint: &str, status: u16) {
        let collectors = self.collectors.read().await;
        collectors
            .requests_total
            .with_label_values(&[method, endpoint, &status.to_string()])
            .inc();
    }

    /// Record an error
    pub async fn record_error(&self, error_type: &str) {
        let collectors = self.collectors.read().await;
        collectors
            .errors_total
            .with_label_values(&[error_type])
            .inc();
    }

    /// Set active connections
    pub async fn set_active_connections(&self, count: i64) {
        let collectors = self.collectors.read().await;
        collectors.active_connections.set(count as f64);
    }

    /// Set memory usage
    pub async fn set_memory_usage(&self, bytes: u64) {
        let collectors = self.collectors.read().await;
        collectors.memory_usage_bytes.set(bytes as f64);
    }

    /// Record request duration
    pub async fn record_request_duration(&self, method: &str, endpoint: &str, duration_secs: f64) {
        let collectors = self.collectors.read().await;
        collectors
            .request_duration_seconds
            .with_label_values(&[method, endpoint])
            .observe(duration_secs);
    }

    /// Record tool execution duration
    pub async fn record_tool_execution(&self, tool_name: &str, duration_secs: f64) {
        let collectors = self.collectors.read().await;
        collectors
            .tool_execution_duration_seconds
            .with_label_values(&[tool_name])
            .observe(duration_secs);
    }

    /// Set CPU usage percentage (0-100)
    pub async fn set_cpu_usage(&self, percent: f64) {
        let collectors = self.collectors.read().await;
        collectors.cpu_usage_percent.set(percent);
    }

    /// Set system memory metrics
    pub async fn set_system_memory(&self, total_bytes: u64, used_bytes: u64, available_bytes: u64) {
        let collectors = self.collectors.read().await;
        collectors.system_memory_total_bytes.set(total_bytes as f64);
        collectors.system_memory_used_bytes.set(used_bytes as f64);
        collectors.system_memory_available_bytes.set(available_bytes as f64);
    }
}

/// Start metrics server
pub async fn start_metrics_server(registry: Arc<Registry>, port: u16) -> ObservabilityResult<()> {
    use axum::{routing::get, Router};
    use std::net::SocketAddr;

    let app = Router::new().route(
        "/metrics",
        get(move || async move {
            use prometheus::Encoder;
            let encoder = prometheus::TextEncoder::new();
            let metric_families = registry.gather();
            let mut buffer = Vec::new();
            encoder.encode(&metric_families, &mut buffer).unwrap();
            String::from_utf8(buffer).unwrap()
        }),
    );

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Metrics server listening on {}", addr);

    axum::serve(
        tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| ObservabilityError::MetricsInitFailed(e.to_string()))?,
        app,
    )
    .await
    .map_err(|e| ObservabilityError::MetricsInitFailed(e.to_string()))?;

    Ok(())
}

/// System metrics monitor
///
/// 系统指标监控器，定期收集 CPU 和内存使用率
pub struct SystemMetricsMonitor {
    collector: MetricsCollector,
    interval: Duration,
}

impl SystemMetricsMonitor {
    /// Create a new system metrics monitor
    ///
    /// 创建新的系统指标监控器
    ///
    /// # Arguments
    ///
    /// * `collector` - Metrics collector
    /// * `interval` - Collection interval (default: 5 seconds)
    pub fn new(collector: MetricsCollector, interval: Duration) -> Self {
        Self { collector, interval }
    }

    /// Start monitoring system metrics
    ///
    /// 启动系统指标监控
    ///
    /// This will spawn a background task that periodically collects CPU and memory metrics.
    /// 这将启动一个后台任务，定期收集 CPU 和内存指标。
    pub fn start(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut sys = sysinfo::System::new_all();
            let mut interval = tokio::time::interval(self.interval);

            loop {
                interval.tick().await;

                // Refresh system information
                // 刷新系统信息
                sys.refresh_all();

                // Collect CPU usage
                // 收集 CPU 使用率
                let cpu_usage = sys.global_cpu_usage() as f64;
                self.collector.set_cpu_usage(cpu_usage).await;

                // Collect memory usage
                // 收集内存使用率
                let total_memory = sys.total_memory();
                let used_memory = sys.used_memory();
                let available_memory = sys.available_memory();

                self.collector
                    .set_system_memory(total_memory, used_memory, available_memory)
                    .await;

                tracing::debug!(
                    "System metrics: CPU={:.2}%, Memory={}/{} MB ({:.2}%)",
                    cpu_usage,
                    used_memory / 1024 / 1024,
                    total_memory / 1024 / 1024,
                    (used_memory as f64 / total_memory as f64) * 100.0
                );
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_registry() {
        let registry = MetricsRegistry::new();
        let collector = registry.collector();

        // Record some metrics
        collector.record_request("GET", "/api/test", 200).await;
        collector.record_error("test_error").await;
        collector.set_active_connections(5).await;
        collector.set_memory_usage(1024 * 1024).await;
        collector
            .record_request_duration("GET", "/api/test", 0.05)
            .await;
        collector.record_tool_execution("calculator", 0.001).await;

        // Gather metrics
        let metrics = registry.gather();
        assert!(metrics.contains("agentmem_requests_total"));
        assert!(metrics.contains("agentmem_errors_total"));
        assert!(metrics.contains("agentmem_active_connections"));
        assert!(metrics.contains("agentmem_memory_usage_bytes"));
    }

    #[tokio::test]
    async fn test_metrics_collector() {
        let registry = MetricsRegistry::new();
        let collector = registry.collector();

        collector.record_request("POST", "/api/create", 201).await;
        collector
            .record_request_duration("POST", "/api/create", 0.1)
            .await;

        let metrics = registry.gather();
        assert!(metrics.contains("POST"));
        assert!(metrics.contains("/api/create"));
    }

    #[tokio::test]
    async fn test_system_metrics() {
        let registry = MetricsRegistry::new();
        let collector = registry.collector();

        // Set CPU usage
        collector.set_cpu_usage(45.5).await;

        // Set memory metrics
        collector
            .set_system_memory(16_000_000_000, 8_000_000_000, 8_000_000_000)
            .await;

        let metrics = registry.gather();
        assert!(metrics.contains("agentmem_cpu_usage_percent"));
        assert!(metrics.contains("agentmem_system_memory_total_bytes"));
        assert!(metrics.contains("agentmem_system_memory_used_bytes"));
        assert!(metrics.contains("agentmem_system_memory_available_bytes"));
    }

    #[tokio::test]
    async fn test_system_metrics_monitor() {
        let registry = MetricsRegistry::new();
        let collector = registry.collector();

        // Create monitor with short interval for testing
        let monitor = SystemMetricsMonitor::new(collector.clone(), Duration::from_millis(100));

        // Start monitoring
        let handle = monitor.start();

        // Wait for a few collection cycles
        tokio::time::sleep(Duration::from_millis(350)).await;

        // Stop monitoring
        handle.abort();

        // Check that metrics were collected
        let metrics = registry.gather();
        assert!(metrics.contains("agentmem_cpu_usage_percent"));
        assert!(metrics.contains("agentmem_system_memory_total_bytes"));
    }
}
