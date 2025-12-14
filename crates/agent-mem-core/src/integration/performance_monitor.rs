use serde::{Deserialize, Serialize};
/// 性能监控器
///
/// 负责监控系统性能指标，提供实时性能数据和性能分析功能
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tokio::time::interval;

use super::{SystemIntegrationManager, SystemMetrics};
use agent_mem_traits::{AgentMemError, Result};

/// 性能监控配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMonitorConfig {
    /// 监控间隔 (秒)
    pub monitor_interval_seconds: u64,
    /// 指标保留时间 (小时)
    pub metrics_retention_hours: u64,
    /// 采样率 (0.0-1.0)
    pub sampling_rate: f64,
    /// 启用详细监控
    pub enable_detailed_monitoring: bool,
}

/// 性能指标数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDataPoint {
    /// 时间戳
    pub timestamp: u64,
    /// 指标值
    pub value: f64,
    /// 标签
    pub labels: HashMap<String, String>,
}

/// 性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetric {
    /// 指标名称
    pub name: String,
    /// 指标类型
    pub metric_type: MetricType,
    /// 数据点
    pub data_points: VecDeque<MetricDataPoint>,
    /// 统计信息
    pub statistics: MetricStatistics,
}

/// 指标类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    /// 计数器
    Counter,
    /// 仪表盘
    Gauge,
    /// 直方图
    Histogram,
    /// 摘要
    Summary,
}

/// 指标统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricStatistics {
    /// 最小值
    pub min: f64,
    /// 最大值
    pub max: f64,
    /// 平均值
    pub avg: f64,
    /// 标准差
    pub std_dev: f64,
    /// 第50百分位
    pub p50: f64,
    /// 第95百分位
    pub p95: f64,
    /// 第99百分位
    pub p99: f64,
}

/// 性能监控器
pub struct PerformanceMonitor {
    /// 配置
    config: PerformanceMonitorConfig,
    /// 性能指标
    metrics: Arc<RwLock<HashMap<String, PerformanceMetric>>>,
    /// 是否正在运行
    is_running: Arc<RwLock<bool>>,
    /// 系统指标
    system_metrics: Arc<RwLock<SystemMetrics>>,
}

impl PerformanceMonitor {
    /// 创建新的性能监控器
    pub fn new(config: PerformanceMonitorConfig) -> Self {
        Self {
            config,
            metrics: Arc::new(RwLock::new(HashMap::new())),
            is_running: Arc::new(RwLock::new(false)),
            system_metrics: Arc::new(RwLock::new(SystemMetrics::default())),
        }
    }

    /// 启动性能监控
    pub async fn start(&self, system_manager: Arc<SystemIntegrationManager>) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Err(AgentMemError::PerformanceMonitorAlreadyRunning);
        }
        *is_running = true;
        drop(is_running);

        let config = self.config.clone();
        let metrics = Arc::clone(&self.metrics);
        let system_metrics = Arc::clone(&self.system_metrics);
        let is_running = Arc::clone(&self.is_running);

        // 启动监控任务
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(config.monitor_interval_seconds));

            while *is_running.read().await {
                interval.tick().await;

                // 收集性能指标
                if let Err(e) =
                    Self::collect_metrics(&system_manager, &config, &metrics, &system_metrics).await
                {
                    eprintln!("性能指标收集失败: {e}");
                }
            }
        });

        Ok(())
    }

    /// 停止性能监控
    pub async fn stop(&self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        Ok(())
    }

    /// 记录指标
    pub async fn record_metric(
        &self,
        name: &str,
        value: f64,
        labels: HashMap<String, String>,
    ) -> Result<()> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let data_point = MetricDataPoint {
            timestamp,
            value,
            labels,
        };

        let mut metrics = self.metrics.write().await;
        let metric = metrics
            .entry(name.to_string())
            .or_insert_with(|| PerformanceMetric {
                name: name.to_string(),
                metric_type: MetricType::Gauge,
                data_points: VecDeque::new(),
                statistics: MetricStatistics::default(),
            });

        metric.data_points.push_back(data_point);

        // 清理过期数据
        let retention_seconds = self.config.metrics_retention_hours * 3600;
        let cutoff_time = timestamp.saturating_sub(retention_seconds);

        while let Some(front) = metric.data_points.front() {
            if front.timestamp < cutoff_time {
                metric.data_points.pop_front();
            } else {
                break;
            }
        }

        // 更新统计信息
        self.update_metric_statistics(metric);

        Ok(())
    }

    /// 获取指标
    pub async fn get_metric(&self, name: &str) -> Option<PerformanceMetric> {
        let metrics = self.metrics.read().await;
        metrics.get(name).cloned()
    }

    /// 获取所有指标
    pub async fn get_all_metrics(&self) -> HashMap<String, PerformanceMetric> {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }

    /// 获取系统指标
    pub async fn get_system_metrics(&self) -> SystemMetrics {
        let metrics = self.system_metrics.read().await;
        metrics.clone()
    }

    /// 获取指标统计信息
    pub async fn get_metric_statistics(&self, name: &str) -> Option<MetricStatistics> {
        let metrics = self.metrics.read().await;
        metrics.get(name).map(|m| m.statistics.clone())
    }

    /// 查询指标数据
    pub async fn query_metrics(
        &self,
        name: &str,
        start_time: Option<u64>,
        end_time: Option<u64>,
        labels: Option<HashMap<String, String>>,
    ) -> Vec<MetricDataPoint> {
        let metrics = self.metrics.read().await;

        if let Some(metric) = metrics.get(name) {
            let mut results = Vec::new();

            for data_point in &metric.data_points {
                // 时间范围过滤
                if let Some(start) = start_time {
                    if data_point.timestamp < start {
                        continue;
                    }
                }

                if let Some(end) = end_time {
                    if data_point.timestamp > end {
                        continue;
                    }
                }

                // 标签过滤
                if let Some(ref filter_labels) = labels {
                    let mut matches = true;
                    for (key, value) in filter_labels {
                        if data_point.labels.get(key) != Some(value) {
                            matches = false;
                            break;
                        }
                    }
                    if !matches {
                        continue;
                    }
                }

                results.push(data_point.clone());
            }

            results
        } else {
            Vec::new()
        }
    }

    /// 私有方法：收集性能指标
    async fn collect_metrics(
        system_manager: &SystemIntegrationManager,
        config: &PerformanceMonitorConfig,
        metrics: &Arc<RwLock<HashMap<String, PerformanceMetric>>>,
        system_metrics: &Arc<RwLock<SystemMetrics>>,
    ) -> Result<()> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // 收集系统级指标
        let mut labels = HashMap::new();
        labels.insert("component".to_string(), "system".to_string());

        // 内存使用情况
        let memory_usage = Self::get_memory_usage().await;
        Self::record_metric_internal(
            metrics,
            "memory_usage_mb",
            memory_usage,
            labels.clone(),
            timestamp,
            config.metrics_retention_hours * 3600,
        )
        .await;

        // CPU使用情况
        let cpu_usage = Self::get_cpu_usage().await;
        Self::record_metric_internal(
            metrics,
            "cpu_usage_percent",
            cpu_usage,
            labels.clone(),
            timestamp,
            config.metrics_retention_hours * 3600,
        )
        .await;

        // 收集组件级指标
        if config.enable_detailed_monitoring {
            Self::collect_component_metrics(
                system_manager,
                metrics,
                timestamp,
                config.metrics_retention_hours * 3600,
            )
            .await?;
        }

        // 更新系统指标
        let mut sys_metrics = system_metrics.write().await;
        sys_metrics.memory_usage_mb = memory_usage;
        sys_metrics.cpu_usage_percent = cpu_usage;

        Ok(())
    }

    /// 收集组件级指标
    async fn collect_component_metrics(
        system_manager: &SystemIntegrationManager,
        metrics: &Arc<RwLock<HashMap<String, PerformanceMetric>>>,
        timestamp: u64,
        retention_seconds: u64,
    ) -> Result<()> {
        // 收集各组件的性能指标
        let components = vec![
            "core_memory",
            "resource_memory",
            "knowledge_vault",
            "contextual_memory",
        ];

        for component_name in components {
            let mut labels = HashMap::new();
            labels.insert("component".to_string(), component_name.to_string());

            // 这里可以添加具体的组件指标收集逻辑
            Self::record_metric_internal(
                metrics,
                &format!("{component_name}_operations_total"),
                1.0, // 示例值
                labels.clone(),
                timestamp,
                retention_seconds,
            )
            .await;
        }

        Ok(())
    }

    /// 内部记录指标方法
    async fn record_metric_internal(
        metrics: &Arc<RwLock<HashMap<String, PerformanceMetric>>>,
        name: &str,
        value: f64,
        labels: HashMap<String, String>,
        timestamp: u64,
        retention_seconds: u64,
    ) {
        let data_point = MetricDataPoint {
            timestamp,
            value,
            labels,
        };

        let mut metrics_map = metrics.write().await;
        let metric = metrics_map
            .entry(name.to_string())
            .or_insert_with(|| PerformanceMetric {
                name: name.to_string(),
                metric_type: MetricType::Gauge,
                data_points: VecDeque::new(),
                statistics: MetricStatistics::default(),
            });

        metric.data_points.push_back(data_point);

        // 清理过期数据
        let cutoff_time = timestamp.saturating_sub(retention_seconds);
        while let Some(front) = metric.data_points.front() {
            if front.timestamp < cutoff_time {
                metric.data_points.pop_front();
            } else {
                break;
            }
        }
    }

    /// 更新指标统计信息
    fn update_metric_statistics(&self, metric: &mut PerformanceMetric) {
        if metric.data_points.is_empty() {
            return;
        }

        let values: Vec<f64> = metric.data_points.iter().map(|dp| dp.value).collect();
        let mut sorted_values = values.clone();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let len = values.len() as f64;
        let sum: f64 = values.iter().sum();
        let avg = sum / len;

        let variance: f64 = values.iter().map(|v| (v - avg).powi(2)).sum::<f64>() / len;
        let std_dev = variance.sqrt();

        metric.statistics = MetricStatistics {
            min: sorted_values[0],
            max: sorted_values[sorted_values.len() - 1],
            avg,
            std_dev,
            p50: Self::percentile(&sorted_values, 0.5),
            p95: Self::percentile(&sorted_values, 0.95),
            p99: Self::percentile(&sorted_values, 0.99),
        };
    }

    /// 计算百分位数
    fn percentile(sorted_values: &[f64], percentile: f64) -> f64 {
        if sorted_values.is_empty() {
            return 0.0;
        }

        let index = (percentile * (sorted_values.len() - 1) as f64).round() as usize;
        sorted_values[index.min(sorted_values.len() - 1)]
    }

    /// 获取内存使用情况
    async fn get_memory_usage() -> f64 {
        // 简化的内存使用情况获取
        // 在实际实现中，可以使用系统调用获取真实的内存使用情况
        100.0 // 示例值 (MB)
    }

    /// 获取CPU使用情况
    async fn get_cpu_usage() -> f64 {
        // 简化的CPU使用情况获取
        // 在实际实现中，可以使用系统调用获取真实的CPU使用情况
        15.0 // 示例值 (%)
    }
}

impl Default for PerformanceMonitorConfig {
    fn default() -> Self {
        Self {
            monitor_interval_seconds: 30,
            metrics_retention_hours: 24,
            sampling_rate: 1.0,
            enable_detailed_monitoring: true,
        }
    }
}

impl Default for MetricStatistics {
    fn default() -> Self {
        Self {
            min: 0.0,
            max: 0.0,
            avg: 0.0,
            std_dev: 0.0,
            p50: 0.0,
            p95: 0.0,
            p99: 0.0,
        }
    }
}
