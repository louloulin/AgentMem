//! Telemetry and monitoring setup

use crate::{
    config::ServerConfig,
    error::{ServerError, ServerResult},
};
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::time::Instant;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Setup telemetry and logging
pub fn setup_telemetry(config: &ServerConfig) -> ServerResult<()> {
    if !config.enable_logging {
        return Ok(());
    }

    // Check if tracing is already initialized
    if tracing::dispatcher::has_been_set() {
        tracing::info!("Tracing already initialized, skipping setup");
        return Ok(());
    }

    // Create environment filter
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.log_level));

    // Setup tracing subscriber
    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .try_init()
        .map_err(|e| ServerError::telemetry_error(format!("Failed to setup tracing: {e}")))?;

    tracing::info!("Telemetry initialized with log level: {}", config.log_level);

    Ok(())
}

/// Metrics collector for tracking request and operation metrics
#[derive(Clone)]
pub struct MetricsCollector {
    inner: Arc<RwLock<MetricsInner>>,
}

struct MetricsInner {
    request_count: u64,
    error_count: u64,
    total_duration_ms: u64,
    memory_operations: u64,
    memory_errors: u64,
    operation_durations: HashMap<String, u64>,
}

impl Default for MetricsInner {
    fn default() -> Self {
        Self {
            request_count: 0,
            error_count: 0,
            total_duration_ms: 0,
            memory_operations: 0,
            memory_errors: 0,
            operation_durations: HashMap::new(),
        }
    }
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(MetricsInner::default())),
        }
    }

    /// Record an HTTP request
    pub fn record_request(&self, _method: &str, _path: &str, status: u16, duration_ms: u64) {
        let mut inner = self.inner.write().unwrap();
        inner.request_count += 1;
        if status >= 400 {
            inner.error_count += 1;
        }
        inner.total_duration_ms += duration_ms;
    }

    /// Record a memory operation
    pub fn record_memory_operation(&self, operation: &str, success: bool, duration_ms: u64) {
        let mut inner = self.inner.write().unwrap();
        inner.memory_operations += 1;
        if !success {
            inner.memory_errors += 1;
        }
        inner.operation_durations.insert(operation.to_string(), duration_ms);
    }

    /// Get all metrics as a HashMap
    pub fn get_metrics(&self) -> HashMap<String, f64> {
        let inner = self.inner.read().unwrap();
        let mut metrics = HashMap::new();
        
        metrics.insert("requests.total".to_string(), inner.request_count as f64);
        metrics.insert("requests.errors".to_string(), inner.error_count as f64);
        metrics.insert("memory.operations.total".to_string(), inner.memory_operations as f64);
        metrics.insert("memory.operations.errors".to_string(), inner.memory_errors as f64);
        
        if inner.request_count > 0 {
            metrics.insert("requests.avg_duration_ms".to_string(), 
                inner.total_duration_ms as f64 / inner.request_count as f64);
            metrics.insert("requests.error_rate".to_string(),
                inner.error_count as f64 / inner.request_count as f64);
        }
        
        metrics
    }

    /// Get request count
    pub fn request_count(&self) -> u64 {
        self.inner.read().unwrap().request_count
    }

    /// Get error count
    pub fn error_count(&self) -> u64 {
        self.inner.read().unwrap().error_count
    }

    /// Get memory operations count
    pub fn memory_operations(&self) -> u64 {
        self.inner.read().unwrap().memory_operations
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector_creation() {
        let collector = MetricsCollector::new();
        let metrics = collector.get_metrics();
        assert_eq!(metrics.get("requests.total"), Some(&0.0));
        assert_eq!(metrics.get("memory.operations.total"), Some(&0.0));
    }

    #[test]
    fn test_record_request() {
        let collector = MetricsCollector::new();
        collector.record_request("GET", "/api/test", 200, 50);
        
        assert_eq!(collector.request_count(), 1);
        let metrics = collector.get_metrics();
        assert_eq!(metrics.get("requests.total"), Some(&1.0));
    }

    #[test]
    fn test_record_error_request() {
        let collector = MetricsCollector::new();
        collector.record_request("GET", "/api/test", 500, 100);
        
        assert_eq!(collector.request_count(), 1);
        assert_eq!(collector.error_count(), 1);
        let metrics = collector.get_metrics();
        assert_eq!(metrics.get("requests.errors"), Some(&1.0));
    }

    #[test]
    fn test_record_memory_operation() {
        let collector = MetricsCollector::new();
        collector.record_memory_operation("search", true, 25);
        collector.record_memory_operation("store", false, 10);
        
        assert_eq!(collector.memory_operations(), 2);
        let metrics = collector.get_metrics();
        assert_eq!(metrics.get("memory.operations.total"), Some(&2.0));
        assert_eq!(metrics.get("memory.operations.errors"), Some(&1.0));
    }

    #[test]
    fn test_average_duration_calculation() {
        let collector = MetricsCollector::new();
        collector.record_request("GET", "/api/test1", 200, 100);
        collector.record_request("GET", "/api/test2", 200, 200);
        
        let metrics = collector.get_metrics();
        assert_eq!(metrics.get("requests.avg_duration_ms"), Some(&150.0));
    }

    #[test]
    fn test_error_rate_calculation() {
        let collector = MetricsCollector::new();
        collector.record_request("GET", "/api/test1", 200, 50);
        collector.record_request("GET", "/api/test2", 500, 50);
        
        let metrics = collector.get_metrics();
        assert_eq!(metrics.get("requests.error_rate"), Some(&0.5));
    }

    #[test]
    fn test_telemetry_setup_disabled() {
        let mut config = ServerConfig::default();
        config.enable_logging = false;

        let result = setup_telemetry(&config);
        assert!(result.is_ok());
    }
}

// ============================================================================
// Alert System
// ============================================================================

/// Alert level for monitoring alerts
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlertLevel {
    Info,
    Warning,
    Error,
    Critical,
}

impl std::fmt::Display for AlertLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertLevel::Info => write!(f, "INFO"),
            AlertLevel::Warning => write!(f, "WARNING"),
            AlertLevel::Error => write!(f, "ERROR"),
            AlertLevel::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// A monitoring alert
#[derive(Debug, Clone)]
pub struct Alert {
    pub level: AlertLevel,
    pub message: String,
    pub metric: String,
    pub value: f64,
    pub threshold: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Alert {
    pub fn new(level: AlertLevel, message: &str, metric: &str, value: f64, threshold: f64) -> Self {
        Self {
            level,
            message: message.to_string(),
            metric: metric.to_string(),
            value,
            threshold,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Alert configuration for monitoring thresholds
#[derive(Debug, Clone)]
pub struct AlertConfig {
    pub error_rate_threshold: f64,
    pub error_count_threshold: u64,
    pub latency_threshold_ms: u64,
    pub memory_operations_error_rate_threshold: f64,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            error_rate_threshold: 0.05,        // 5% error rate
            error_count_threshold: 100,        // 100 errors
            latency_threshold_ms: 1000,        // 1 second
            memory_operations_error_rate_threshold: 0.1, // 10% error rate
        }
    }
}

/// Alert manager for checking metrics and generating alerts
#[derive(Clone)]
pub struct AlertManager {
    config: AlertConfig,
    metrics: MetricsCollector,
}

impl AlertManager {
    pub fn new(config: AlertConfig, metrics: MetricsCollector) -> Self {
        Self { config, metrics }
    }

    /// Check metrics and generate any triggered alerts
    pub fn check_alerts(&self) -> Vec<Alert> {
        let mut alerts = Vec::new();
        let metrics = self.metrics.get_metrics();

        // Check error rate
        if let Some(&error_rate) = metrics.get("requests.error_rate") {
            if error_rate > self.config.error_rate_threshold {
                alerts.push(Alert::new(
                    if error_rate > 0.1 {
                        AlertLevel::Critical
                    } else {
                        AlertLevel::Warning
                    },
                    &format!("Error rate {}% exceeds threshold {}%", 
                        (error_rate * 100.0) as i32, 
                        (self.config.error_rate_threshold * 100.0) as i32),
                    "requests.error_rate",
                    error_rate,
                    self.config.error_rate_threshold,
                ));
            }
        }

        // Check error count
        if let Some(&error_count) = metrics.get("requests.errors") {
            if error_count as f64 > self.config.error_count_threshold as f64 {
                alerts.push(Alert::new(
                    AlertLevel::Warning,
                    &format!("Error count {} exceeds threshold {}", 
                        error_count as i32, 
                        self.config.error_count_threshold),
                    "requests.errors",
                    error_count,
                    self.config.error_count_threshold as f64,
                ));
            }
        }

        // Check average latency
        if let Some(&avg_duration) = metrics.get("requests.avg_duration_ms") {
            if avg_duration > self.config.latency_threshold_ms as f64 {
                alerts.push(Alert::new(
                    AlertLevel::Warning,
                    &format!("Average latency {}ms exceeds threshold {}ms", 
                        avg_duration as i32, 
                        self.config.latency_threshold_ms),
                    "requests.avg_duration_ms",
                    avg_duration,
                    self.config.latency_threshold_ms as f64,
                ));
            }
        }

        // Check memory operations error rate
        let memory_ops = metrics.get("memory.operations.total").copied().unwrap_or(0.0);
        let memory_errors = metrics.get("memory.operations.errors").copied().unwrap_or(0.0);
        if memory_ops > 0.0 {
            let memory_error_rate = memory_errors / memory_ops;
            if memory_error_rate > self.config.memory_operations_error_rate_threshold {
                alerts.push(Alert::new(
                    AlertLevel::Error,
                    &format!("Memory operations error rate {}% exceeds threshold {}%", 
                        (memory_error_rate * 100.0) as i32, 
                        (self.config.memory_operations_error_rate_threshold * 100.0) as i32),
                    "memory.operations.error_rate",
                    memory_error_rate,
                    self.config.memory_operations_error_rate_threshold,
                ));
            }
        }

        alerts
    }
}

#[cfg(test)]
mod alert_tests {
    use super::*;

    #[test]
    fn test_alert_creation() {
        let alert = Alert::new(
            AlertLevel::Warning,
            "Test alert",
            "test_metric",
            0.1,
            0.05,
        );
        assert_eq!(alert.level, AlertLevel::Warning);
        assert_eq!(alert.metric, "test_metric");
    }

    #[test]
    fn test_alert_level_display() {
        assert_eq!(format!("{}", AlertLevel::Info), "INFO");
        assert_eq!(format!("{}", AlertLevel::Warning), "WARNING");
        assert_eq!(format!("{}", AlertLevel::Error), "ERROR");
        assert_eq!(format!("{}", AlertLevel::Critical), "CRITICAL");
    }

    #[test]
    fn test_alert_config_default() {
        let config = AlertConfig::default();
        assert_eq!(config.error_rate_threshold, 0.05);
        assert_eq!(config.error_count_threshold, 100);
        assert_eq!(config.latency_threshold_ms, 1000);
    }

    #[test]
    fn test_alert_manager_no_alerts() {
        let metrics = MetricsCollector::new();
        let config = AlertConfig::default();
        let manager = AlertManager::new(config, metrics);
        
        let alerts = manager.check_alerts();
        assert!(alerts.is_empty());
    }

    #[test]
    fn test_alert_manager_high_error_rate() {
        let metrics = MetricsCollector::new();
        metrics.record_request("GET", "/api/test", 500, 100);
        metrics.record_request("GET", "/api/test", 500, 100);
        metrics.record_request("GET", "/api/test", 500, 100);
        metrics.record_request("GET", "/api/test", 200, 100);
        
        let config = AlertConfig::default();
        let manager = AlertManager::new(config, metrics);
        
        let alerts = manager.check_alerts();
        // 75% error rate should trigger an alert (above 5% threshold)
        assert!(!alerts.is_empty());
    }
}
