//! 压测统计模块

use hdrhistogram::Histogram;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressTestStats {
    // 基本统计
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub duration_seconds: f64,
    
    // 吞吐量
    pub throughput: f64, // ops/sec
    
    // 延迟统计 (毫秒)
    pub latency_min: f64,
    pub latency_max: f64,
    pub latency_mean: f64,
    pub latency_p50: f64,
    pub latency_p90: f64,
    pub latency_p95: f64,
    pub latency_p99: f64,
    pub latency_p999: f64,
    
    // 资源使用
    pub avg_cpu_usage: f64,
    pub peak_cpu_usage: f64,
    pub avg_memory_mb: f64,
    pub peak_memory_mb: f64,
    
    // 错误统计
    pub error_rate: f64,
    pub errors_by_type: Vec<(String, u64)>,
    
    // 时间戳
    pub start_time: String,
    pub end_time: String,
}

impl Default for StressTestStats {
    fn default() -> Self {
        Self {
            total_operations: 0,
            successful_operations: 0,
            failed_operations: 0,
            duration_seconds: 0.0,
            throughput: 0.0,
            latency_min: 0.0,
            latency_max: 0.0,
            latency_mean: 0.0,
            latency_p50: 0.0,
            latency_p90: 0.0,
            latency_p95: 0.0,
            latency_p99: 0.0,
            latency_p999: 0.0,
            avg_cpu_usage: 0.0,
            peak_cpu_usage: 0.0,
            avg_memory_mb: 0.0,
            peak_memory_mb: 0.0,
            error_rate: 0.0,
            errors_by_type: Vec::new(),
            start_time: String::new(),
            end_time: String::new(),
        }
    }
}

pub struct StatsCollector {
    start_time: Instant,
    histogram: Arc<RwLock<Histogram<u64>>>,
    total_ops: Arc<RwLock<u64>>,
    successful_ops: Arc<RwLock<u64>>,
    failed_ops: Arc<RwLock<u64>>,
    cpu_samples: Arc<RwLock<Vec<f64>>>,
    memory_samples: Arc<RwLock<Vec<f64>>>,
}

impl StatsCollector {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            histogram: Arc::new(RwLock::new(
                Histogram::<u64>::new_with_bounds(1, 60_000, 3).unwrap()
            )),
            total_ops: Arc::new(RwLock::new(0)),
            successful_ops: Arc::new(RwLock::new(0)),
            failed_ops: Arc::new(RwLock::new(0)),
            cpu_samples: Arc::new(RwLock::new(Vec::new())),
            memory_samples: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn record_operation(&self, duration: Duration, success: bool) {
        let mut total = self.total_ops.write().await;
        *total += 1;

        if success {
            let mut successful = self.successful_ops.write().await;
            *successful += 1;
        } else {
            let mut failed = self.failed_ops.write().await;
            *failed += 1;
        }

        let mut hist = self.histogram.write().await;
        let _ = hist.record(duration.as_millis() as u64);
    }

    pub async fn record_system_stats(&self, cpu_usage: f64, memory_mb: f64) {
        let mut cpu = self.cpu_samples.write().await;
        cpu.push(cpu_usage);

        let mut mem = self.memory_samples.write().await;
        mem.push(memory_mb);
    }

    pub async fn get_stats(&self) -> StressTestStats {
        let total = *self.total_ops.read().await;
        let successful = *self.successful_ops.read().await;
        let failed = *self.failed_ops.read().await;
        let duration = self.start_time.elapsed().as_secs_f64();

        let hist = self.histogram.read().await;
        let cpu_samples = self.cpu_samples.read().await;
        let memory_samples = self.memory_samples.read().await;

        let throughput = if duration > 0.0 {
            total as f64 / duration
        } else {
            0.0
        };

        let error_rate = if total > 0 {
            failed as f64 / total as f64
        } else {
            0.0
        };

        let avg_cpu = if !cpu_samples.is_empty() {
            cpu_samples.iter().sum::<f64>() / cpu_samples.len() as f64
        } else {
            0.0
        };

        let peak_cpu = cpu_samples.iter().cloned().fold(0.0, f64::max);

        let avg_memory = if !memory_samples.is_empty() {
            memory_samples.iter().sum::<f64>() / memory_samples.len() as f64
        } else {
            0.0
        };

        let peak_memory = memory_samples.iter().cloned().fold(0.0, f64::max);

        StressTestStats {
            total_operations: total,
            successful_operations: successful,
            failed_operations: failed,
            duration_seconds: duration,
            throughput,
            latency_min: hist.min() as f64,
            latency_max: hist.max() as f64,
            latency_mean: hist.mean(),
            latency_p50: hist.value_at_quantile(0.50) as f64,
            latency_p90: hist.value_at_quantile(0.90) as f64,
            latency_p95: hist.value_at_quantile(0.95) as f64,
            latency_p99: hist.value_at_quantile(0.99) as f64,
            latency_p999: hist.value_at_quantile(0.999) as f64,
            avg_cpu_usage: avg_cpu,
            peak_cpu_usage: peak_cpu,
            avg_memory_mb: avg_memory,
            peak_memory_mb: peak_memory,
            error_rate,
            errors_by_type: Vec::new(),
            start_time: chrono::Utc::now().to_rfc3339(),
            end_time: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub async fn reset(&self) {
        *self.total_ops.write().await = 0;
        *self.successful_ops.write().await = 0;
        *self.failed_ops.write().await = 0;
        self.histogram.write().await.reset();
        self.cpu_samples.write().await.clear();
        self.memory_samples.write().await.clear();
    }
}

impl Default for StatsCollector {
    fn default() -> Self {
        Self::new()
    }
}

