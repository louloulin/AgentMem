//! Telemetry and monitoring utilities

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Performance metrics for operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub operation: String,
    pub duration_ms: u64,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl PerformanceMetrics {
    pub fn new(operation: &str) -> Self {
        Self {
            operation: operation.to_string(),
            duration_ms: 0,
            timestamp: Utc::now(),
            success: true,
            metadata: HashMap::new(),
        }
    }

    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration_ms = duration.as_millis() as u64;
        self
    }

    pub fn with_success(mut self, success: bool) -> Self {
        self.success = success;
        self
    }

    pub fn with_metadata(mut self, key: &str, value: serde_json::Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }
}

/// Timer for measuring operation duration
pub struct Timer {
    start: Instant,
    operation: String,
}

impl Timer {
    pub fn new(operation: &str) -> Self {
        Self {
            start: Instant::now(),
            operation: operation.to_string(),
        }
    }

    pub fn finish(self) -> PerformanceMetrics {
        let duration = self.start.elapsed();
        PerformanceMetrics::new(&self.operation).with_duration(duration)
    }

    pub fn finish_with_result<T>(
        self,
        result: &Result<T, impl std::error::Error>,
    ) -> PerformanceMetrics {
        let duration = self.start.elapsed();
        PerformanceMetrics::new(&self.operation)
            .with_duration(duration)
            .with_success(result.is_ok())
    }
}

/// Usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsageStats {
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub total_duration_ms: u64,
    pub average_duration_ms: f64,
    pub operations_by_type: HashMap<String, u64>,
}

impl UsageStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_operation(&mut self, metrics: &PerformanceMetrics) {
        self.total_operations += 1;
        self.total_duration_ms += metrics.duration_ms;

        if metrics.success {
            self.successful_operations += 1;
        } else {
            self.failed_operations += 1;
        }

        *self
            .operations_by_type
            .entry(metrics.operation.clone())
            .or_insert(0) += 1;

        // Update average
        self.average_duration_ms = self.total_duration_ms as f64 / self.total_operations as f64;
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_operations == 0 {
            0.0
        } else {
            self.successful_operations as f64 / self.total_operations as f64
        }
    }
}

/// Process telemetry filters (inspired by mem0)
pub fn process_telemetry_filters(
    filters: &HashMap<String, String>,
) -> (Vec<String>, HashMap<String, String>) {
    let mut encoded_ids = HashMap::new();
    let filter_keys: Vec<String> = filters.keys().cloned().collect();

    for (key, value) in filters {
        if ["user_id", "agent_id", "run_id"].contains(&key.as_str()) {
            // Simple hash encoding for privacy
            let encoded = crate::hash::short_hash(value);
            encoded_ids.insert(key.clone(), encoded);
        }
    }

    (filter_keys, encoded_ids)
}

/// Log performance metrics
pub fn log_performance(metrics: &PerformanceMetrics) {
    tracing::info!(
        operation = %metrics.operation,
        duration_ms = metrics.duration_ms,
        success = metrics.success,
        "Operation completed"
    );
}

/// Log usage statistics
pub fn log_usage_stats(stats: &UsageStats) {
    tracing::info!(
        total_operations = stats.total_operations,
        success_rate = stats.success_rate(),
        average_duration_ms = stats.average_duration_ms,
        "Usage statistics"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_performance_metrics() {
        let metrics = PerformanceMetrics::new("test_operation")
            .with_duration(Duration::from_millis(100))
            .with_success(true)
            .with_metadata(
                "test_key",
                serde_json::Value::String("test_value".to_string()),
            );

        assert_eq!(metrics.operation, "test_operation");
        assert_eq!(metrics.duration_ms, 100);
        assert!(metrics.success);
        assert!(metrics.metadata.contains_key("test_key"));
    }

    #[test]
    fn test_timer() {
        let timer = Timer::new("test_timer");
        thread::sleep(Duration::from_millis(10));
        let metrics = timer.finish();

        assert_eq!(metrics.operation, "test_timer");
        assert!(metrics.duration_ms >= 10);
    }

    #[test]
    fn test_usage_stats() {
        let mut stats = UsageStats::new();

        let metrics1 = PerformanceMetrics::new("op1")
            .with_duration(Duration::from_millis(100))
            .with_success(true);

        let metrics2 = PerformanceMetrics::new("op2")
            .with_duration(Duration::from_millis(200))
            .with_success(false);

        stats.record_operation(&metrics1);
        stats.record_operation(&metrics2);

        assert_eq!(stats.total_operations, 2);
        assert_eq!(stats.successful_operations, 1);
        assert_eq!(stats.failed_operations, 1);
        assert_eq!(stats.success_rate(), 0.5);
        assert_eq!(stats.average_duration_ms, 150.0);
    }

    #[test]
    fn test_process_telemetry_filters() {
        let mut filters = HashMap::new();
        filters.insert("user_id".to_string(), "user123".to_string());
        filters.insert("other_key".to_string(), "other_value".to_string());

        let (filter_keys, encoded_ids) = process_telemetry_filters(&filters);

        assert_eq!(filter_keys.len(), 2);
        assert!(encoded_ids.contains_key("user_id"));
        assert!(!encoded_ids.contains_key("other_key"));
    }
}
