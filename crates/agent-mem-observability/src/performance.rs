//! Performance analysis and monitoring
//!
//! This module provides performance tracking and bottleneck identification.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Performance analyzer
pub struct PerformanceAnalyzer {
    operations: Arc<RwLock<HashMap<String, OperationStats>>>,
}

/// Operation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationStats {
    /// Operation name
    pub name: String,
    /// Total executions
    pub total_executions: u64,
    /// Total duration
    pub total_duration_ms: f64,
    /// Average duration
    pub avg_duration_ms: f64,
    /// Min duration
    pub min_duration_ms: f64,
    /// Max duration
    pub max_duration_ms: f64,
    /// P50 duration
    pub p50_duration_ms: f64,
    /// P95 duration
    pub p95_duration_ms: f64,
    /// P99 duration
    pub p99_duration_ms: f64,
    /// Recent durations (for percentile calculation)
    recent_durations: Vec<f64>,
}

impl OperationStats {
    fn new(name: String) -> Self {
        Self {
            name,
            total_executions: 0,
            total_duration_ms: 0.0,
            avg_duration_ms: 0.0,
            min_duration_ms: f64::MAX,
            max_duration_ms: 0.0,
            p50_duration_ms: 0.0,
            p95_duration_ms: 0.0,
            p99_duration_ms: 0.0,
            recent_durations: Vec::new(),
        }
    }

    fn record(&mut self, duration_ms: f64) {
        self.total_executions += 1;
        self.total_duration_ms += duration_ms;
        self.avg_duration_ms = self.total_duration_ms / self.total_executions as f64;
        self.min_duration_ms = self.min_duration_ms.min(duration_ms);
        self.max_duration_ms = self.max_duration_ms.max(duration_ms);

        // Keep last 1000 durations for percentile calculation
        self.recent_durations.push(duration_ms);
        if self.recent_durations.len() > 1000 {
            self.recent_durations.remove(0);
        }

        // Calculate percentiles
        let mut sorted = self.recent_durations.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        if !sorted.is_empty() {
            self.p50_duration_ms = sorted[sorted.len() * 50 / 100];
            self.p95_duration_ms = sorted[sorted.len() * 95 / 100];
            self.p99_duration_ms = sorted[sorted.len() * 99 / 100];
        }
    }
}

impl PerformanceAnalyzer {
    /// Create a new performance analyzer
    pub fn new() -> Self {
        Self {
            operations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start tracking an operation
    pub fn start_operation(&self, name: impl Into<String>) -> OperationTracker {
        OperationTracker {
            name: name.into(),
            start: Instant::now(),
            analyzer: self.operations.clone(),
        }
    }

    /// Record an operation duration
    pub async fn record_operation(&self, name: &str, duration: Duration) {
        let duration_ms = duration.as_secs_f64() * 1000.0;
        let mut operations = self.operations.write().await;

        operations
            .entry(name.to_string())
            .or_insert_with(|| OperationStats::new(name.to_string()))
            .record(duration_ms);
    }

    /// Get statistics for an operation
    pub async fn get_stats(&self, name: &str) -> Option<OperationStats> {
        let operations = self.operations.read().await;
        operations.get(name).cloned()
    }

    /// Get all statistics
    pub async fn get_all_stats(&self) -> Vec<OperationStats> {
        let operations = self.operations.read().await;
        operations.values().cloned().collect()
    }

    /// Identify slow operations (> threshold)
    pub async fn identify_slow_operations(&self, threshold_ms: f64) -> Vec<OperationStats> {
        let operations = self.operations.read().await;
        operations
            .values()
            .filter(|stats| stats.avg_duration_ms > threshold_ms)
            .cloned()
            .collect()
    }

    /// Get performance report
    pub async fn get_report(&self) -> PerformanceReport {
        let operations = self.operations.read().await;

        let total_operations: u64 = operations.values().map(|s| s.total_executions).sum();
        let avg_duration: f64 = if !operations.is_empty() {
            operations.values().map(|s| s.avg_duration_ms).sum::<f64>() / operations.len() as f64
        } else {
            0.0
        };

        let slowest = operations
            .values()
            .max_by(|a, b| a.avg_duration_ms.partial_cmp(&b.avg_duration_ms).unwrap())
            .cloned();

        PerformanceReport {
            total_operations,
            avg_duration_ms: avg_duration,
            slowest_operation: slowest,
            operations: operations.values().cloned().collect(),
        }
    }
}

impl Default for PerformanceAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Operation tracker (RAII pattern)
pub struct OperationTracker {
    name: String,
    start: Instant,
    analyzer: Arc<RwLock<HashMap<String, OperationStats>>>,
}

impl Drop for OperationTracker {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        let duration_ms = duration.as_secs_f64() * 1000.0;
        let name = self.name.clone();
        let analyzer = self.analyzer.clone();

        // Record in background
        tokio::spawn(async move {
            let mut operations = analyzer.write().await;
            operations
                .entry(name.clone())
                .or_insert_with(|| OperationStats::new(name))
                .record(duration_ms);
        });
    }
}

/// Performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    /// Total operations
    pub total_operations: u64,
    /// Average duration
    pub avg_duration_ms: f64,
    /// Slowest operation
    pub slowest_operation: Option<OperationStats>,
    /// All operations
    pub operations: Vec<OperationStats>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_performance_analyzer() {
        let analyzer = PerformanceAnalyzer::new();

        // Record some operations
        analyzer
            .record_operation("test_op", Duration::from_millis(10))
            .await;
        analyzer
            .record_operation("test_op", Duration::from_millis(20))
            .await;
        analyzer
            .record_operation("test_op", Duration::from_millis(15))
            .await;

        let stats = analyzer.get_stats("test_op").await.unwrap();
        assert_eq!(stats.total_executions, 3);
        assert!(stats.avg_duration_ms > 0.0);
        assert_eq!(stats.min_duration_ms, 10.0);
        assert_eq!(stats.max_duration_ms, 20.0);
    }

    #[tokio::test]
    async fn test_operation_tracker() {
        let analyzer = PerformanceAnalyzer::new();

        {
            let _tracker = analyzer.start_operation("tracked_op");
            sleep(Duration::from_millis(10)).await;
        }

        // Wait for background task
        sleep(Duration::from_millis(50)).await;

        let stats = analyzer.get_stats("tracked_op").await.unwrap();
        assert_eq!(stats.total_executions, 1);
        assert!(stats.avg_duration_ms >= 10.0);
    }

    #[tokio::test]
    async fn test_identify_slow_operations() {
        let analyzer = PerformanceAnalyzer::new();

        analyzer
            .record_operation("fast_op", Duration::from_millis(5))
            .await;
        analyzer
            .record_operation("slow_op", Duration::from_millis(100))
            .await;

        let slow_ops = analyzer.identify_slow_operations(50.0).await;
        assert_eq!(slow_ops.len(), 1);
        assert_eq!(slow_ops[0].name, "slow_op");
    }

    #[tokio::test]
    async fn test_performance_report() {
        let analyzer = PerformanceAnalyzer::new();

        analyzer
            .record_operation("op1", Duration::from_millis(10))
            .await;
        analyzer
            .record_operation("op2", Duration::from_millis(20))
            .await;

        let report = analyzer.get_report().await;
        assert_eq!(report.total_operations, 2);
        assert!(report.avg_duration_ms > 0.0);
        assert!(report.slowest_operation.is_some());
    }
}
