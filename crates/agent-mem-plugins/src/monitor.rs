//! Plugin execution monitoring
//!
//! Provides metrics and monitoring for plugin execution.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Execution metrics for a plugin
#[derive(Debug, Clone)]
pub struct ExecutionMetrics {
    /// Plugin ID
    pub plugin_id: String,

    /// Total number of calls
    pub total_calls: u64,

    /// Total number of successful calls
    pub successful_calls: u64,

    /// Total number of failed calls
    pub failed_calls: u64,

    /// Total execution time
    pub total_execution_time: Duration,

    /// Average execution time
    pub avg_execution_time: Duration,

    /// Min execution time
    pub min_execution_time: Option<Duration>,

    /// Max execution time
    pub max_execution_time: Option<Duration>,

    /// Last execution time
    pub last_execution_time: Option<Instant>,

    /// Last error (if any)
    pub last_error: Option<String>,
}

impl ExecutionMetrics {
    pub fn new(plugin_id: String) -> Self {
        Self {
            plugin_id,
            total_calls: 0,
            successful_calls: 0,
            failed_calls: 0,
            total_execution_time: Duration::from_secs(0),
            avg_execution_time: Duration::from_secs(0),
            min_execution_time: None,
            max_execution_time: None,
            last_execution_time: None,
            last_error: None,
        }
    }

    /// Record a successful execution
    pub fn record_success(&mut self, duration: Duration) {
        self.total_calls += 1;
        self.successful_calls += 1;
        self.record_duration(duration);
    }

    /// Record a failed execution
    pub fn record_failure(&mut self, duration: Duration, error: String) {
        self.total_calls += 1;
        self.failed_calls += 1;
        self.record_duration(duration);
        self.last_error = Some(error);
    }

    /// Record execution duration
    fn record_duration(&mut self, duration: Duration) {
        self.total_execution_time += duration;
        self.last_execution_time = Some(Instant::now());

        // Update average
        self.avg_execution_time =
            self.total_execution_time / (self.total_calls as u32).max(1);

        // Update min
        self.min_execution_time = Some(match self.min_execution_time {
            Some(min) if min < duration => min,
            _ => duration,
        });

        // Update max
        self.max_execution_time = Some(match self.max_execution_time {
            Some(max) if max > duration => max,
            _ => duration,
        });
    }

    /// Get success rate (0.0 to 1.0)
    pub fn success_rate(&self) -> f64 {
        if self.total_calls == 0 {
            return 0.0;
        }
        self.successful_calls as f64 / self.total_calls as f64
    }

    /// Get failure rate (0.0 to 1.0)
    pub fn failure_rate(&self) -> f64 {
        if self.total_calls == 0 {
            return 0.0;
        }
        self.failed_calls as f64 / self.total_calls as f64
    }
}

/// Plugin monitor
pub struct PluginMonitor {
    metrics: Arc<RwLock<HashMap<String, ExecutionMetrics>>>,
}

impl PluginMonitor {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start monitoring a plugin execution
    pub fn start_execution(&self, plugin_id: &str) -> ExecutionTracker {
        // Ensure metrics entry exists
        {
            let mut metrics = self.metrics.write().unwrap();
            metrics
                .entry(plugin_id.to_string())
                .or_insert_with(|| ExecutionMetrics::new(plugin_id.to_string()));
        }

        ExecutionTracker {
            plugin_id: plugin_id.to_string(),
            start_time: Instant::now(),
            monitor: self.clone(),
        }
    }

    /// Get metrics for a specific plugin
    pub fn get_metrics(&self, plugin_id: &str) -> Option<ExecutionMetrics> {
        let metrics = self.metrics.read().unwrap();
        metrics.get(plugin_id).cloned()
    }

    /// Get all metrics
    pub fn get_all_metrics(&self) -> HashMap<String, ExecutionMetrics> {
        let metrics = self.metrics.read().unwrap();
        metrics.clone()
    }

    /// Reset metrics for a specific plugin
    pub fn reset_metrics(&self, plugin_id: &str) {
        let mut metrics = self.metrics.write().unwrap();
        metrics.remove(plugin_id);
    }

    /// Reset all metrics
    pub fn reset_all_metrics(&self) {
        let mut metrics = self.metrics.write().unwrap();
        metrics.clear();
    }

    /// Record successful execution (internal)
    fn record_success(&self, plugin_id: &str, duration: Duration) {
        let mut metrics = self.metrics.write().unwrap();
        if let Some(metric) = metrics.get_mut(plugin_id) {
            metric.record_success(duration);
        }
    }

    /// Record failed execution (internal)
    fn record_failure(&self, plugin_id: &str, duration: Duration, error: String) {
        let mut metrics = self.metrics.write().unwrap();
        if let Some(metric) = metrics.get_mut(plugin_id) {
            metric.record_failure(duration, error);
        }
    }
}

impl Clone for PluginMonitor {
    fn clone(&self) -> Self {
        Self {
            metrics: self.metrics.clone(),
        }
    }
}

impl Default for PluginMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Execution tracker for monitoring a single plugin execution
pub struct ExecutionTracker {
    plugin_id: String,
    start_time: Instant,
    monitor: PluginMonitor,
}

impl ExecutionTracker {
    /// Complete the execution successfully
    pub fn complete(self) {
        let duration = self.start_time.elapsed();
        self.monitor.record_success(&self.plugin_id, duration);
    }

    /// Complete the execution with an error
    pub fn fail(self, error: String) {
        let duration = self.start_time.elapsed();
        self.monitor.record_failure(&self.plugin_id, duration, error);
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_execution_metrics_creation() {
        let metrics = ExecutionMetrics::new("test-plugin".to_string());
        assert_eq!(metrics.plugin_id, "test-plugin");
        assert_eq!(metrics.total_calls, 0);
        assert_eq!(metrics.successful_calls, 0);
        assert_eq!(metrics.failed_calls, 0);
    }

    #[test]
    fn test_record_success() {
        let mut metrics = ExecutionMetrics::new("test-plugin".to_string());

        metrics.record_success(Duration::from_millis(100));

        assert_eq!(metrics.total_calls, 1);
        assert_eq!(metrics.successful_calls, 1);
        assert_eq!(metrics.failed_calls, 0);
        assert!(metrics.min_execution_time.is_some());
        assert!(metrics.max_execution_time.is_some());
    }

    #[test]
    fn test_record_failure() {
        let mut metrics = ExecutionMetrics::new("test-plugin".to_string());

        metrics.record_failure(Duration::from_millis(100), "Test error".to_string());

        assert_eq!(metrics.total_calls, 1);
        assert_eq!(metrics.successful_calls, 0);
        assert_eq!(metrics.failed_calls, 1);
        assert_eq!(metrics.last_error, Some("Test error".to_string()));
    }

    #[test]
    fn test_success_rate() {
        let mut metrics = ExecutionMetrics::new("test-plugin".to_string());

        metrics.record_success(Duration::from_millis(100));
        metrics.record_success(Duration::from_millis(100));
        metrics.record_failure(Duration::from_millis(100), "Error".to_string());

        let success_rate = metrics.success_rate();
        assert!((success_rate - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_failure_rate() {
        let mut metrics = ExecutionMetrics::new("test-plugin".to_string());

        metrics.record_success(Duration::from_millis(100));
        metrics.record_failure(Duration::from_millis(100), "Error".to_string());

        let failure_rate = metrics.failure_rate();
        assert_eq!(failure_rate, 0.5);
    }

    #[test]
    fn test_plugin_monitor_basic() {
        let monitor = PluginMonitor::new();

        let tracker = monitor.start_execution("test-plugin");
        thread::sleep(Duration::from_millis(10));
        tracker.complete();

        let metrics = monitor.get_metrics("test-plugin").unwrap();
        assert_eq!(metrics.total_calls, 1);
        assert_eq!(metrics.successful_calls, 1);
    }

    #[test]
    fn test_plugin_monitor_failure() {
        let monitor = PluginMonitor::new();

        let tracker = monitor.start_execution("test-plugin");
        tracker.fail("Test error".to_string());

        let metrics = monitor.get_metrics("test-plugin").unwrap();
        assert_eq!(metrics.total_calls, 1);
        assert_eq!(metrics.failed_calls, 1);
        assert_eq!(metrics.last_error, Some("Test error".to_string()));
    }

    #[test]
    fn test_plugin_monitor_multiple_plugins() {
        let monitor = PluginMonitor::new();

        // Plugin 1
        let tracker1 = monitor.start_execution("plugin-1");
        tracker1.complete();

        // Plugin 2
        let tracker2 = monitor.start_execution("plugin-2");
        tracker2.complete();

        // Plugin 1 again
        let tracker3 = monitor.start_execution("plugin-1");
        tracker3.complete();

        let all_metrics = monitor.get_all_metrics();
        assert_eq!(all_metrics.len(), 2);

        let metrics1 = monitor.get_metrics("plugin-1").unwrap();
        assert_eq!(metrics1.total_calls, 2);

        let metrics2 = monitor.get_metrics("plugin-2").unwrap();
        assert_eq!(metrics2.total_calls, 1);
    }

    #[test]
    fn test_reset_metrics() {
        let monitor = PluginMonitor::new();

        let tracker = monitor.start_execution("test-plugin");
        tracker.complete();

        assert!(monitor.get_metrics("test-plugin").is_some());

        monitor.reset_metrics("test-plugin");
        assert!(monitor.get_metrics("test-plugin").is_none());
    }

    #[test]
    fn test_reset_all_metrics() {
        let monitor = PluginMonitor::new();

        let tracker1 = monitor.start_execution("plugin-1");
        tracker1.complete();

        let tracker2 = monitor.start_execution("plugin-2");
        tracker2.complete();

        assert_eq!(monitor.get_all_metrics().len(), 2);

        monitor.reset_all_metrics();
        assert_eq!(monitor.get_all_metrics().len(), 0);
    }

    #[test]
    fn test_min_max_execution_time() {
        let mut metrics = ExecutionMetrics::new("test-plugin".to_string());

        metrics.record_success(Duration::from_millis(100));
        metrics.record_success(Duration::from_millis(50));
        metrics.record_success(Duration::from_millis(200));

        assert_eq!(
            metrics.min_execution_time,
            Some(Duration::from_millis(50))
        );
        assert_eq!(
            metrics.max_execution_time,
            Some(Duration::from_millis(200))
        );
    }

    #[test]
    fn test_average_execution_time() {
        let mut metrics = ExecutionMetrics::new("test-plugin".to_string());

        metrics.record_success(Duration::from_millis(100));
        metrics.record_success(Duration::from_millis(200));
        metrics.record_success(Duration::from_millis(300));

        // Average should be 200ms
        assert_eq!(metrics.avg_execution_time, Duration::from_millis(200));
    }
}

