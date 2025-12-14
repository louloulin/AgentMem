//! 缓存性能监控系统
//!
//! 提供实时的缓存性能指标收集、分析和报告功能

use super::{CacheLevel, CacheStats};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// 性能指标快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    /// 快照时间戳
    pub timestamp: u64,

    /// L1 缓存统计
    pub l1_stats: Option<CacheStats>,

    /// L2 缓存统计
    pub l2_stats: Option<CacheStats>,

    /// 总体统计
    pub combined_stats: CacheStats,

    /// 平均响应时间 (毫秒)
    pub avg_response_time_ms: f64,

    /// P50 响应时间 (毫秒)
    pub p50_response_time_ms: f64,

    /// P95 响应时间 (毫秒)
    pub p95_response_time_ms: f64,

    /// P99 响应时间 (毫秒)
    pub p99_response_time_ms: f64,

    /// 每秒请求数
    pub requests_per_second: f64,
}

/// 响应时间记录
#[derive(Debug, Clone)]
struct ResponseTimeRecord {
    timestamp: Instant,
    duration_ms: f64,
    cache_level: Option<CacheLevel>,
    hit: bool,
}

/// 缓存监控配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    /// 启用监控
    pub enabled: bool,

    /// 快照间隔（秒）
    pub snapshot_interval_secs: u64,

    /// 保留快照数量
    pub max_snapshots: usize,

    /// 响应时间记录窗口大小
    pub response_time_window: usize,

    /// 慢查询阈值（毫秒）
    pub slow_query_threshold_ms: f64,

    /// 启用慢查询日志
    pub enable_slow_query_log: bool,

    /// 启用报警
    pub enable_alerts: bool,

    /// 命中率报警阈值
    pub hit_rate_alert_threshold: f64,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            snapshot_interval_secs: 60, // 每分钟一次快照
            max_snapshots: 1440,        // 保留24小时数据
            response_time_window: 1000, // 最近1000次请求
            slow_query_threshold_ms: 100.0,
            enable_slow_query_log: true,
            enable_alerts: true,
            hit_rate_alert_threshold: 50.0, // 命中率低于50%报警
        }
    }
}

/// 缓存性能监控器
pub struct CacheMonitor {
    /// 配置
    config: MonitorConfig,

    /// 性能快照历史
    snapshots: Arc<RwLock<VecDeque<PerformanceSnapshot>>>,

    /// 响应时间记录
    response_times: Arc<RwLock<VecDeque<ResponseTimeRecord>>>,

    /// 慢查询计数
    slow_query_count: Arc<RwLock<u64>>,

    /// 最后快照时间
    last_snapshot: Arc<RwLock<Instant>>,
}

impl CacheMonitor {
    /// 创建新的监控器
    pub fn new(config: MonitorConfig) -> Self {
        info!("Creating cache monitor (enabled: {})", config.enabled);

        Self {
            config,
            snapshots: Arc::new(RwLock::new(VecDeque::with_capacity(1440))),
            response_times: Arc::new(RwLock::new(VecDeque::with_capacity(1000))),
            slow_query_count: Arc::new(RwLock::new(0)),
            last_snapshot: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// 记录缓存操作
    pub async fn record_operation(
        &self,
        duration: Duration,
        hit: bool,
        cache_level: Option<CacheLevel>,
    ) {
        if !self.config.enabled {
            return;
        }

        let duration_ms = duration.as_secs_f64() * 1000.0;

        // 检查是否为慢查询
        if duration_ms > self.config.slow_query_threshold_ms {
            let mut count = self.slow_query_count.write().await;
            *count += 1;

            if self.config.enable_slow_query_log {
                warn!(
                    "Slow cache query detected: {:.2}ms (threshold: {:.2}ms), hit: {}, level: {:?}",
                    duration_ms, self.config.slow_query_threshold_ms, hit, cache_level
                );
            }
        }

        // 记录响应时间
        let record = ResponseTimeRecord {
            timestamp: Instant::now(),
            duration_ms,
            cache_level,
            hit,
        };

        let mut times = self.response_times.write().await;
        times.push_back(record);

        // 限制窗口大小
        if times.len() > self.config.response_time_window {
            times.pop_front();
        }
    }

    /// 创建性能快照
    pub async fn create_snapshot(
        &self,
        l1_stats: Option<CacheStats>,
        l2_stats: Option<CacheStats>,
        combined_stats: CacheStats,
    ) -> PerformanceSnapshot {
        // 计算响应时间指标
        let times = self.response_times.read().await;
        let mut durations: Vec<f64> = times.iter().map(|r| r.duration_ms).collect();
        durations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let avg_response_time_ms = if !durations.is_empty() {
            durations.iter().sum::<f64>() / durations.len() as f64
        } else {
            0.0
        };

        let p50_response_time_ms = Self::percentile(&durations, 50.0);
        let p95_response_time_ms = Self::percentile(&durations, 95.0);
        let p99_response_time_ms = Self::percentile(&durations, 99.0);

        // 计算QPS
        let now = Instant::now();
        let last = *self.last_snapshot.read().await;
        let elapsed_secs = now.duration_since(last).as_secs_f64();
        let requests_per_second = if elapsed_secs > 0.0 {
            times.len() as f64 / elapsed_secs
        } else {
            0.0
        };

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        PerformanceSnapshot {
            timestamp,
            l1_stats,
            l2_stats,
            combined_stats,
            avg_response_time_ms,
            p50_response_time_ms,
            p95_response_time_ms,
            p99_response_time_ms,
            requests_per_second,
        }
    }

    /// 保存快照
    pub async fn save_snapshot(&self, snapshot: PerformanceSnapshot) {
        if !self.config.enabled {
            return;
        }

        // 检查命中率报警
        if self.config.enable_alerts {
            let hit_rate = snapshot.combined_stats.hit_rate();
            if hit_rate < self.config.hit_rate_alert_threshold {
                warn!(
                    "Cache hit rate alert: {:.2}% (threshold: {:.2}%)",
                    hit_rate, self.config.hit_rate_alert_threshold
                );
            }
        }

        let mut snapshots = self.snapshots.write().await;
        snapshots.push_back(snapshot);

        // 限制快照数量
        if snapshots.len() > self.config.max_snapshots {
            snapshots.pop_front();
        }

        // 更新最后快照时间
        *self.last_snapshot.write().await = Instant::now();

        debug!("Cache performance snapshot saved");
    }

    /// 获取最新快照
    pub async fn latest_snapshot(&self) -> Option<PerformanceSnapshot> {
        let snapshots = self.snapshots.read().await;
        snapshots.back().cloned()
    }

    /// 获取所有快照
    pub async fn all_snapshots(&self) -> Vec<PerformanceSnapshot> {
        let snapshots = self.snapshots.read().await;
        snapshots.iter().cloned().collect()
    }

    /// 获取慢查询数量
    pub async fn slow_query_count(&self) -> u64 {
        *self.slow_query_count.read().await
    }

    /// 重置慢查询计数
    pub async fn reset_slow_query_count(&self) {
        *self.slow_query_count.write().await = 0;
    }

    /// 生成性能报告
    pub async fn generate_report(&self) -> Option<PerformanceReport> {
        let snapshots = self.snapshots.read().await;

        if snapshots.is_empty() {
            return None;
        }

        // Safe unwrap: we already checked snapshots.is_empty() above
        let latest = snapshots.back().expect("snapshots should not be empty after is_empty() check");
        let earliest = snapshots.front().expect("snapshots should not be empty after is_empty() check");

        // 计算趋势
        let hit_rate_trend = latest.combined_stats.hit_rate() - earliest.combined_stats.hit_rate();

        let avg_response_time_trend = latest.avg_response_time_ms - earliest.avg_response_time_ms;

        // 计算平均值
        let avg_hit_rate = snapshots
            .iter()
            .map(|s| s.combined_stats.hit_rate())
            .sum::<f64>()
            / snapshots.len() as f64;

        let avg_qps =
            snapshots.iter().map(|s| s.requests_per_second).sum::<f64>() / snapshots.len() as f64;

        // 找到最佳和最差性能
        let mut sorted_by_hit_rate: Vec<_> = snapshots.iter().collect();
        sorted_by_hit_rate.sort_by(|a, b| {
            b.combined_stats
                .hit_rate()
                .partial_cmp(&a.combined_stats.hit_rate())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let best_hit_rate = sorted_by_hit_rate
            .first()
            .unwrap()
            .combined_stats
            .hit_rate();
        let worst_hit_rate = sorted_by_hit_rate.last().unwrap().combined_stats.hit_rate();

        Some(PerformanceReport {
            report_period_secs: (latest.timestamp - earliest.timestamp),
            total_snapshots: snapshots.len(),
            latest_snapshot: latest.clone(),
            avg_hit_rate,
            hit_rate_trend,
            best_hit_rate,
            worst_hit_rate,
            avg_qps,
            avg_response_time_ms: latest.avg_response_time_ms,
            avg_response_time_trend,
            slow_query_count: *self.slow_query_count.read().await,
            recommendations: Self::generate_recommendations(latest, hit_rate_trend),
        })
    }

    /// 计算百分位数
    fn percentile(sorted_values: &[f64], percentile: f64) -> f64 {
        if sorted_values.is_empty() {
            return 0.0;
        }

        let index = ((percentile / 100.0) * (sorted_values.len() - 1) as f64).round() as usize;
        sorted_values[index.min(sorted_values.len() - 1)]
    }

    /// 生成优化建议
    fn generate_recommendations(
        snapshot: &PerformanceSnapshot,
        hit_rate_trend: f64,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        let hit_rate = snapshot.combined_stats.hit_rate();

        // 命中率建议
        if hit_rate < 50.0 {
            recommendations.push(
                "⚠️  命中率过低 (<50%)，建议：1) 增加缓存容量 2) 优化缓存键设计 3) 启用缓存预热"
                    .to_string(),
            );
        } else if hit_rate < 70.0 {
            recommendations.push(
                "💡 命中率可以改进 (<70%)，建议：1) 分析访问模式 2) 调整TTL 3) 考虑预热热门数据"
                    .to_string(),
            );
        } else if hit_rate > 85.0 {
            recommendations.push("✅ 命中率优秀 (>85%)，缓存策略运行良好！".to_string());
        }

        // 趋势建议
        if hit_rate_trend < -5.0 {
            recommendations.push(
                "📉 命中率下降趋势明显，建议检查：1) 查询模式变化 2) 缓存失效策略 3) 数据热度分布"
                    .to_string(),
            );
        } else if hit_rate_trend > 5.0 {
            recommendations.push("📈 命中率提升趋势良好，当前优化策略有效！".to_string());
        }

        // 响应时间建议
        if snapshot.p99_response_time_ms > 100.0 {
            recommendations.push(
                "⚠️  P99响应时间过高 (>100ms)，建议：1) 优化缓存查询 2) 检查网络延迟 3) 考虑增加缓存层级"
                    .to_string()
            );
        }

        // QPS建议
        if snapshot.requests_per_second > 1000.0 {
            recommendations.push(
                "📊 高QPS场景 (>1000)，建议：1) 确保缓存容量充足 2) 监控内存使用 3) 考虑分布式缓存"
                    .to_string(),
            );
        }

        if recommendations.is_empty() {
            recommendations.push("✅ 缓存系统运行正常，无特殊建议".to_string());
        }

        recommendations
    }
}

/// 性能报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    /// 报告周期（秒）
    pub report_period_secs: u64,

    /// 快照总数
    pub total_snapshots: usize,

    /// 最新快照
    pub latest_snapshot: PerformanceSnapshot,

    /// 平均命中率
    pub avg_hit_rate: f64,

    /// 命中率趋势（正值表示上升）
    pub hit_rate_trend: f64,

    /// 最佳命中率
    pub best_hit_rate: f64,

    /// 最差命中率
    pub worst_hit_rate: f64,

    /// 平均QPS
    pub avg_qps: f64,

    /// 平均响应时间
    pub avg_response_time_ms: f64,

    /// 响应时间趋势
    pub avg_response_time_trend: f64,

    /// 慢查询数量
    pub slow_query_count: u64,

    /// 优化建议
    pub recommendations: Vec<String>,
}

impl PerformanceReport {
    /// 格式化为可读文本
    pub fn format_text(&self) -> String {
        let mut output = String::new();

        output.push_str("=== 缓存性能报告 ===\n\n");
        output.push_str(&format!(
            "报告周期: {} 秒 ({} 分钟)\n",
            self.report_period_secs,
            self.report_period_secs / 60
        ));
        output.push_str(&format!("快照数量: {}\n\n", self.total_snapshots));

        output.push_str("--- 命中率指标 ---\n");
        output.push_str(&format!(
            "当前命中率: {:.2}%\n",
            self.latest_snapshot.combined_stats.hit_rate()
        ));
        output.push_str(&format!("平均命中率: {:.2}%\n", self.avg_hit_rate));
        output.push_str(&format!("最佳命中率: {:.2}%\n", self.best_hit_rate));
        output.push_str(&format!("最差命中率: {:.2}%\n", self.worst_hit_rate));
        output.push_str(&format!("命中率趋势: {:+.2}%\n\n", self.hit_rate_trend));

        output.push_str("--- 响应时间指标 ---\n");
        output.push_str(&format!(
            "平均响应: {:.2}ms\n",
            self.latest_snapshot.avg_response_time_ms
        ));
        output.push_str(&format!(
            "P50响应: {:.2}ms\n",
            self.latest_snapshot.p50_response_time_ms
        ));
        output.push_str(&format!(
            "P95响应: {:.2}ms\n",
            self.latest_snapshot.p95_response_time_ms
        ));
        output.push_str(&format!(
            "P99响应: {:.2}ms\n",
            self.latest_snapshot.p99_response_time_ms
        ));
        output.push_str(&format!(
            "响应时间趋势: {:+.2}ms\n\n",
            self.avg_response_time_trend
        ));

        output.push_str("--- 吞吐量指标 ---\n");
        output.push_str(&format!(
            "当前QPS: {:.2}\n",
            self.latest_snapshot.requests_per_second
        ));
        output.push_str(&format!("平均QPS: {:.2}\n", self.avg_qps));
        output.push_str(&format!("慢查询数: {}\n\n", self.slow_query_count));

        output.push_str("--- 优化建议 ---\n");
        for (i, rec) in self.recommendations.iter().enumerate() {
            output.push_str(&format!("{}. {}\n", i + 1, rec));
        }

        output.push_str("\n=================\n");

        output
    }

    /// 格式化为JSON
    pub fn format_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_monitor_creation() {
        let config = MonitorConfig::default();
        let monitor = CacheMonitor::new(config.clone());

        assert_eq!(monitor.config.enabled, config.enabled);
        assert_eq!(monitor.slow_query_count().await, 0);
    }

    #[tokio::test]
    async fn test_record_operation() {
        let config = MonitorConfig::default();
        let monitor = CacheMonitor::new(config);

        // 记录几次操作
        monitor
            .record_operation(Duration::from_millis(10), true, Some(CacheLevel::L1))
            .await;

        monitor
            .record_operation(
                Duration::from_millis(150), // 慢查询
                false,
                Some(CacheLevel::L2),
            )
            .await;

        // 验证慢查询计数
        assert_eq!(monitor.slow_query_count().await, 1);
    }

    #[tokio::test]
    async fn test_snapshot_creation() {
        let config = MonitorConfig::default();
        let monitor = CacheMonitor::new(config);

        // 记录一些操作
        for _ in 0..10 {
            monitor
                .record_operation(Duration::from_millis(20), true, Some(CacheLevel::L1))
                .await;
        }

        let stats = CacheStats {
            total_gets: 10,
            hits: 8,
            misses: 2,
            total_sets: 5,
            evictions: 0,
            invalidations: 0,
            total_size_bytes: 1024,
            entry_count: 5,
        };

        let snapshot = monitor
            .create_snapshot(Some(stats.clone()), None, stats)
            .await;

        assert_eq!(snapshot.combined_stats.hit_rate(), 80.0);
        assert!(snapshot.avg_response_time_ms > 0.0);
    }
}
