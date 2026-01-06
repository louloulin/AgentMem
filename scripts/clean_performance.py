#!/usr/bin/env python3
"""
清理性能监控模块，只保留性能监控相关的代码
"""

def clean_performance_module():
    """清理性能监控模块"""
    
    performance_content = """// Performance 模块 - 性能监控和诊断
// 从 lib.rs 自动拆分生成

use std::collections::HashMap;
use std::sync::{Arc, RwLock, Mutex};
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use serde::{Deserialize, Serialize};
use crate::core::*;

// 性能指标结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_io: NetworkIO,
    pub database_stats: DatabaseStats,
    pub query_performance: QueryPerformance,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkIO {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub total_queries: u64,
    pub successful_queries: u64,
    pub failed_queries: u64,
    pub avg_query_time_ms: f64,
    pub cache_hit_rate: f64,
    pub active_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPerformance {
    pub total_queries: u64,
    pub avg_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub throughput_qps: f64,
}

// 性能监控器
pub struct PerformanceMonitor {
    metrics: Arc<RwLock<PerformanceMetrics>>,
    query_times: Arc<Mutex<Vec<f64>>>,
    start_time: Instant,
    query_count: AtomicU64,
    error_count: AtomicU64,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            query_times: Arc::new(Mutex::new(Vec::new())),
            start_time: Instant::now(),
            query_count: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
        }
    }

    pub fn record_query(&self, duration_ms: f64, success: bool) {
        self.query_count.fetch_add(1, Ordering::Relaxed);
        
        if !success {
            self.error_count.fetch_add(1, Ordering::Relaxed);
        }

        // 记录查询时间
        if let Ok(mut times) = self.query_times.lock() {
            times.push(duration_ms);
            
            // 保持最近1000个查询的记录
            if times.len() > 1000 {
                times.remove(0);
            }
        }
    }

    pub fn get_metrics(&self) -> PerformanceMetrics {
        let mut metrics = self.metrics.read().unwrap().clone();
        
        // 更新查询性能统计
        if let Ok(times) = self.query_times.lock() {
            if !times.is_empty() {
                let total_queries = self.query_count.load(Ordering::Relaxed);
                let avg_latency = times.iter().sum::<f64>() / times.len() as f64;
                
                let mut sorted_times = times.clone();
                sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
                
                let p95_index = (sorted_times.len() as f64 * 0.95) as usize;
                let p99_index = (sorted_times.len() as f64 * 0.99) as usize;
                
                let p95_latency = sorted_times.get(p95_index).copied().unwrap_or(0.0);
                let p99_latency = sorted_times.get(p99_index).copied().unwrap_or(0.0);
                
                let elapsed_secs = self.start_time.elapsed().as_secs_f64();
                let throughput = if elapsed_secs > 0.0 {
                    total_queries as f64 / elapsed_secs
                } else {
                    0.0
                };

                metrics.query_performance = QueryPerformance {
                    total_queries,
                    avg_latency_ms: avg_latency,
                    p95_latency_ms: p95_latency,
                    p99_latency_ms: p99_latency,
                    throughput_qps: throughput,
                };
            }
        }

        metrics.timestamp = chrono::Utc::now().timestamp();
        metrics
    }

    pub fn update_system_metrics(&self) {
        // 这里可以集成系统监控库来获取实际的系统指标
        // 为了简化，我们使用模拟数据
        
        if let Ok(mut metrics) = self.metrics.write() {
            metrics.cpu_usage = self.get_cpu_usage();
            metrics.memory_usage = self.get_memory_usage();
            metrics.disk_usage = self.get_disk_usage();
            metrics.network_io = self.get_network_io();
            metrics.database_stats = self.get_database_stats();
        }
    }

    fn get_cpu_usage(&self) -> f64 {
        // 模拟CPU使用率
        rand::random::<f64>() * 100.0
    }

    fn get_memory_usage(&self) -> f64 {
        // 模拟内存使用率
        rand::random::<f64>() * 100.0
    }

    fn get_disk_usage(&self) -> f64 {
        // 模拟磁盘使用率
        rand::random::<f64>() * 100.0
    }

    fn get_network_io(&self) -> NetworkIO {
        NetworkIO {
            bytes_sent: rand::random::<u64>() % 1000000,
            bytes_received: rand::random::<u64>() % 1000000,
            packets_sent: rand::random::<u64>() % 10000,
            packets_received: rand::random::<u64>() % 10000,
        }
    }

    fn get_database_stats(&self) -> DatabaseStats {
        let total_queries = self.query_count.load(Ordering::Relaxed);
        let failed_queries = self.error_count.load(Ordering::Relaxed);
        let successful_queries = total_queries - failed_queries;

        DatabaseStats {
            total_queries,
            successful_queries,
            failed_queries,
            avg_query_time_ms: 0.0, // 将在get_metrics中计算
            cache_hit_rate: rand::random::<f64>(),
            active_connections: rand::random::<u32>() % 100,
        }
    }

    pub fn get_error_rate(&self) -> f64 {
        let total = self.query_count.load(Ordering::Relaxed);
        let errors = self.error_count.load(Ordering::Relaxed);
        
        if total > 0 {
            errors as f64 / total as f64
        } else {
            0.0
        }
    }

    pub fn reset_metrics(&self) {
        self.query_count.store(0, Ordering::Relaxed);
        self.error_count.store(0, Ordering::Relaxed);
        
        if let Ok(mut times) = self.query_times.lock() {
            times.clear();
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            disk_usage: 0.0,
            network_io: NetworkIO {
                bytes_sent: 0,
                bytes_received: 0,
                packets_sent: 0,
                packets_received: 0,
            },
            database_stats: DatabaseStats {
                total_queries: 0,
                successful_queries: 0,
                failed_queries: 0,
                avg_query_time_ms: 0.0,
                cache_hit_rate: 0.0,
                active_connections: 0,
            },
            query_performance: QueryPerformance {
                total_queries: 0,
                avg_latency_ms: 0.0,
                p95_latency_ms: 0.0,
                p99_latency_ms: 0.0,
                throughput_qps: 0.0,
            },
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

// 性能诊断器
pub struct PerformanceDiagnostics {
    monitor: Arc<PerformanceMonitor>,
    thresholds: PerformanceThresholds,
}

#[derive(Debug, Clone)]
pub struct PerformanceThresholds {
    pub max_cpu_usage: f64,
    pub max_memory_usage: f64,
    pub max_query_latency_ms: f64,
    pub min_cache_hit_rate: f64,
    pub max_error_rate: f64,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            max_cpu_usage: 80.0,
            max_memory_usage: 85.0,
            max_query_latency_ms: 1000.0,
            min_cache_hit_rate: 0.8,
            max_error_rate: 0.05,
        }
    }
}

impl PerformanceDiagnostics {
    pub fn new(monitor: Arc<PerformanceMonitor>) -> Self {
        Self {
            monitor,
            thresholds: PerformanceThresholds::default(),
        }
    }

    pub fn diagnose(&self) -> Vec<PerformanceIssue> {
        let metrics = self.monitor.get_metrics();
        let mut issues = Vec::new();

        // 检查CPU使用率
        if metrics.cpu_usage > self.thresholds.max_cpu_usage {
            issues.push(PerformanceIssue {
                severity: IssueSeverity::Warning,
                category: IssueCategory::System,
                description: format!("High CPU usage: {:.1}%", metrics.cpu_usage),
                recommendation: "Consider optimizing queries or scaling resources".to_string(),
            });
        }

        // 检查内存使用率
        if metrics.memory_usage > self.thresholds.max_memory_usage {
            issues.push(PerformanceIssue {
                severity: IssueSeverity::Warning,
                category: IssueCategory::System,
                description: format!("High memory usage: {:.1}%", metrics.memory_usage),
                recommendation: "Consider increasing memory or optimizing data structures".to_string(),
            });
        }

        // 检查查询延迟
        if metrics.query_performance.avg_latency_ms > self.thresholds.max_query_latency_ms {
            issues.push(PerformanceIssue {
                severity: IssueSeverity::Critical,
                category: IssueCategory::Database,
                description: format!("High query latency: {:.1}ms", metrics.query_performance.avg_latency_ms),
                recommendation: "Optimize queries, add indexes, or scale database".to_string(),
            });
        }

        // 检查缓存命中率
        if metrics.database_stats.cache_hit_rate < self.thresholds.min_cache_hit_rate {
            issues.push(PerformanceIssue {
                severity: IssueSeverity::Warning,
                category: IssueCategory::Database,
                description: format!("Low cache hit rate: {:.1}%", metrics.database_stats.cache_hit_rate * 100.0),
                recommendation: "Increase cache size or optimize cache strategy".to_string(),
            });
        }

        // 检查错误率
        let error_rate = self.monitor.get_error_rate();
        if error_rate > self.thresholds.max_error_rate {
            issues.push(PerformanceIssue {
                severity: IssueSeverity::Critical,
                category: IssueCategory::Application,
                description: format!("High error rate: {:.1}%", error_rate * 100.0),
                recommendation: "Investigate and fix application errors".to_string(),
            });
        }

        issues
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceIssue {
    pub severity: IssueSeverity,
    pub category: IssueCategory,
    pub description: String,
    pub recommendation: String,
}

#[derive(Debug, Clone)]
pub enum IssueSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone)]
pub enum IssueCategory {
    System,
    Database,
    Network,
    Application,
}
"""
    
    with open('src/performance.rs', 'w', encoding='utf-8') as f:
        f.write(performance_content)
    
    print("Performance模块已清理完成")

if __name__ == "__main__":
    clean_performance_module()
