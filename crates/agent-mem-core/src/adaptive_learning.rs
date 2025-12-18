//! Adaptive Learning Mechanism
//!
//! Phase 5.1: 自适应学习机制
//! - 学习策略优化
//! - 自适应参数调整
//! - 在线学习支持
//!
//! 参考Mem0和自适应学习理论，实现持续学习和优化

use agent_mem_traits::{Result, AgentMemError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// 自适应学习配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveLearningConfig {
    /// 启用自适应学习
    pub enable_learning: bool,
    /// 学习率 (0.0-1.0)
    pub learning_rate: f64,
    /// 最小样本数（少于此数量不进行学习）
    pub min_samples: usize,
    /// 衰减因子（旧数据的权重衰减）
    pub decay_factor: f64,
    /// 在线学习更新间隔（秒）
    pub online_update_interval: u64,
    /// 性能评估窗口（小时）
    pub evaluation_window_hours: u64,
    /// 性能阈值（低于此值触发调整）
    pub performance_threshold: f64,
}

impl Default for AdaptiveLearningConfig {
    fn default() -> Self {
        Self {
            enable_learning: true,
            learning_rate: 0.1,
            min_samples: 50,
            decay_factor: 0.95,
            online_update_interval: 3600, // 1小时
            evaluation_window_hours: 24,
            performance_threshold: 0.7,
        }
    }
}

/// 学习策略
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LearningStrategy {
    /// 保守策略 - 缓慢调整，优先稳定性
    Conservative,
    /// 平衡策略 - 平衡学习和稳定性
    Balanced,
    /// 激进策略 - 快速调整，优先性能
    Aggressive,
    /// 自适应策略 - 根据性能自动选择
    Adaptive,
}

/// 性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// 准确率
    pub accuracy: f64,
    /// 延迟（毫秒）
    pub latency_ms: u64,
    /// 吞吐量（ops/s）
    pub throughput: f64,
    /// 用户满意度
    pub user_satisfaction: f64,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
}

/// 参数调整记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterAdjustment {
    /// 参数名称
    pub parameter_name: String,
    /// 旧值
    pub old_value: f64,
    /// 新值
    pub new_value: f64,
    /// 调整原因
    pub reason: String,
    /// 调整时间
    pub adjusted_at: DateTime<Utc>,
    /// 预期效果
    pub expected_improvement: f64,
}

/// 学习统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningStatistics {
    /// 总学习次数
    pub total_learning_cycles: usize,
    /// 参数调整次数
    pub parameter_adjustments: usize,
    /// 平均性能提升
    pub avg_performance_improvement: f64,
    /// 当前策略
    pub current_strategy: LearningStrategy,
    /// 最后更新时间
    pub last_updated: DateTime<Utc>,
}

/// 自适应学习引擎
pub struct AdaptiveLearningEngine {
    config: AdaptiveLearningConfig,
    /// 当前学习策略
    current_strategy: Arc<RwLock<LearningStrategy>>,
    /// 性能历史记录
    performance_history: Arc<RwLock<VecDeque<PerformanceMetrics>>>,
    /// 参数调整历史
    adjustment_history: Arc<RwLock<VecDeque<ParameterAdjustment>>>,
    /// 参数当前值
    current_parameters: Arc<RwLock<HashMap<String, f64>>>,
    /// 学习统计
    statistics: Arc<RwLock<LearningStatistics>>,
}

impl AdaptiveLearningEngine {
    /// 创建新的自适应学习引擎
    pub fn new(config: AdaptiveLearningConfig) -> Self {
        let mut default_params = HashMap::new();
        default_params.insert("vector_weight".to_string(), 0.7);
        default_params.insert("fulltext_weight".to_string(), 0.3);
        default_params.insert("rerank_threshold".to_string(), 0.5);
        default_params.insert("cache_ttl".to_string(), 3600.0);

        Self {
            config,
            current_strategy: Arc::new(RwLock::new(LearningStrategy::Balanced)),
            performance_history: Arc::new(RwLock::new(VecDeque::new())),
            adjustment_history: Arc::new(RwLock::new(VecDeque::new())),
            current_parameters: Arc::new(RwLock::new(default_params)),
            statistics: Arc::new(RwLock::new(LearningStatistics {
                total_learning_cycles: 0,
                parameter_adjustments: 0,
                avg_performance_improvement: 0.0,
                current_strategy: LearningStrategy::Balanced,
                last_updated: Utc::now(),
            })),
        }
    }

    /// 使用默认配置创建
    pub fn with_defaults() -> Self {
        Self::new(AdaptiveLearningConfig::default())
    }

    /// 记录性能指标（在线学习）
    pub async fn record_performance(&self, metrics: PerformanceMetrics) -> Result<()> {
        if !self.config.enable_learning {
            return Ok(());
        }

        let mut history = self.performance_history.write().await;
        history.push_back(metrics.clone());

        // 限制历史记录大小
        if history.len() > 1000 {
            history.pop_front();
        }

        // 如果样本数足够，触发学习
        if history.len() >= self.config.min_samples {
            drop(history);
            self.trigger_learning().await?;
        }

        Ok(())
    }

    /// 触发学习过程
    async fn trigger_learning(&self) -> Result<()> {
        info!("触发自适应学习过程");

        let history = self.performance_history.read().await;
        if history.len() < self.config.min_samples {
            return Ok(());
        }

        // 评估当前性能
        let current_performance = self.evaluate_performance(&history).await?;

        // 如果性能低于阈值，触发参数调整
        if current_performance < self.config.performance_threshold {
            info!("性能低于阈值 ({:.2} < {:.2})，触发参数调整", 
                current_performance, self.config.performance_threshold);
            self.adjust_parameters().await?;
        }

        // 更新学习策略
        self.update_learning_strategy().await?;

        // 更新统计
        {
            let mut stats = self.statistics.write().await;
            stats.total_learning_cycles += 1;
            stats.last_updated = Utc::now();
        }

        Ok(())
    }

    /// 评估当前性能
    async fn evaluate_performance(
        &self,
        history: &VecDeque<PerformanceMetrics>,
    ) -> Result<f64> {
        if history.is_empty() {
            return Ok(0.0);
        }

        // 计算加权平均性能（最近的数据权重更高）
        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;

        for (i, metric) in history.iter().enumerate() {
            let weight = self.config.decay_factor.powi(i as i32);
            let performance = (metric.accuracy * 0.4 
                + (1.0 - metric.latency_ms as f64 / 1000.0) * 0.3
                + (metric.throughput / 1000.0) * 0.2
                + metric.user_satisfaction * 0.1).min(1.0);
            
            weighted_sum += performance * weight;
            total_weight += weight;
        }

        Ok(if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            0.0
        })
    }

    /// 调整参数
    async fn adjust_parameters(&self) -> Result<()> {
        let history = self.performance_history.read().await;
        let strategy = self.current_strategy.read().await.clone();
        drop(history);

        let mut params = self.current_parameters.write().await;
        let mut adjustments = self.adjustment_history.write().await;

        // 根据策略调整参数
        match strategy {
            LearningStrategy::Conservative => {
                // 保守策略：小幅调整
                self.adjust_parameter_conservative(&mut params, &mut adjustments, "vector_weight").await?;
            }
            LearningStrategy::Balanced => {
                // 平衡策略：中等调整
                self.adjust_parameter_balanced(&mut params, &mut adjustments, "vector_weight").await?;
                self.adjust_parameter_balanced(&mut params, &mut adjustments, "fulltext_weight").await?;
            }
            LearningStrategy::Aggressive => {
                // 激进策略：大幅调整
                self.adjust_parameter_aggressive(&mut params, &mut adjustments, "vector_weight").await?;
                self.adjust_parameter_aggressive(&mut params, &mut adjustments, "fulltext_weight").await?;
                self.adjust_parameter_aggressive(&mut params, &mut adjustments, "rerank_threshold").await?;
            }
            LearningStrategy::Adaptive => {
                // 自适应策略：根据性能自动选择
                let history = self.performance_history.read().await;
                let performance = self.evaluate_performance(&history).await?;
                drop(history);
                
                if performance < 0.5 {
                    // 性能很差，使用激进策略
                    self.adjust_parameter_aggressive(&mut params, &mut adjustments, "vector_weight").await?;
                } else if performance < 0.7 {
                    // 性能一般，使用平衡策略
                    self.adjust_parameter_balanced(&mut params, &mut adjustments, "vector_weight").await?;
                } else {
                    // 性能良好，使用保守策略
                    self.adjust_parameter_conservative(&mut params, &mut adjustments, "vector_weight").await?;
                }
            }
        }

        // 更新统计
        {
            let mut stats = self.statistics.write().await;
            stats.parameter_adjustments += 1;
        }

        Ok(())
    }

    /// 保守策略调整参数
    async fn adjust_parameter_conservative(
        &self,
        params: &mut HashMap<String, f64>,
        adjustments: &mut VecDeque<ParameterAdjustment>,
        param_name: &str,
    ) -> Result<()> {
        let current_value = params.get(param_name).copied().unwrap_or(0.5);
        let adjustment = self.config.learning_rate * 0.1; // 保守调整：10%的学习率
        let new_value = (current_value + adjustment).clamp(0.0, 1.0);

        if (new_value - current_value).abs() > 0.01 {
            adjustments.push_back(ParameterAdjustment {
                parameter_name: param_name.to_string(),
                old_value: current_value,
                new_value,
                reason: "Conservative strategy: small adjustment".to_string(),
                adjusted_at: Utc::now(),
                expected_improvement: 0.05,
            });

            params.insert(param_name.to_string(), new_value);
            info!("保守调整参数 {}: {:.3} -> {:.3}", param_name, current_value, new_value);
        }
        Ok(())
    }

    /// 平衡策略调整参数
    async fn adjust_parameter_balanced(
        &self,
        params: &mut HashMap<String, f64>,
        adjustments: &mut VecDeque<ParameterAdjustment>,
        param_name: &str,
    ) -> Result<()> {
        let current_value = params.get(param_name).copied().unwrap_or(0.5);
        let adjustment = self.config.learning_rate * 0.5; // 平衡调整：50%的学习率
        let new_value = (current_value + adjustment).clamp(0.0, 1.0);

        if (new_value - current_value).abs() > 0.01 {
            adjustments.push_back(ParameterAdjustment {
                parameter_name: param_name.to_string(),
                old_value: current_value,
                new_value,
                reason: "Balanced strategy: moderate adjustment".to_string(),
                adjusted_at: Utc::now(),
                expected_improvement: 0.1,
            });

            params.insert(param_name.to_string(), new_value);
            info!("平衡调整参数 {}: {:.3} -> {:.3}", param_name, current_value, new_value);
        }
        Ok(())
    }

    /// 激进策略调整参数
    async fn adjust_parameter_aggressive(
        &self,
        params: &mut HashMap<String, f64>,
        adjustments: &mut VecDeque<ParameterAdjustment>,
        param_name: &str,
    ) -> Result<()> {
        let current_value = params.get(param_name).copied().unwrap_or(0.5);
        let adjustment = self.config.learning_rate; // 激进调整：100%的学习率
        let new_value = (current_value + adjustment).clamp(0.0, 1.0);

        if (new_value - current_value).abs() > 0.01 {
            adjustments.push_back(ParameterAdjustment {
                parameter_name: param_name.to_string(),
                old_value: current_value,
                new_value,
                reason: "Aggressive strategy: large adjustment".to_string(),
                adjusted_at: Utc::now(),
                expected_improvement: 0.2,
            });

            params.insert(param_name.to_string(), new_value);
            info!("激进调整参数 {}: {:.3} -> {:.3}", param_name, current_value, new_value);
        }
        Ok(())
    }

    /// 更新学习策略
    async fn update_learning_strategy(&self) -> Result<()> {
        let history = self.performance_history.read().await;
        if history.len() < self.config.min_samples {
            return Ok(());
        }

        let performance = self.evaluate_performance(&history).await?;
        drop(history);

        let mut strategy = self.current_strategy.write().await;
        let old_strategy = strategy.clone();

        // 根据性能选择策略
        if performance < 0.5 {
            *strategy = LearningStrategy::Aggressive;
        } else if performance < 0.7 {
            *strategy = LearningStrategy::Balanced;
        } else {
            *strategy = LearningStrategy::Conservative;
        }

        if *strategy != old_strategy {
            info!("学习策略更新: {:?} -> {:?}", old_strategy, strategy);
        }

        // 更新统计
        {
            let mut stats = self.statistics.write().await;
            stats.current_strategy = strategy.clone();
        }

        Ok(())
    }

    /// 获取当前参数值
    pub async fn get_parameter(&self, param_name: &str) -> Option<f64> {
        let params = self.current_parameters.read().await;
        params.get(param_name).copied()
    }

    /// 设置参数值（手动调整）
    pub async fn set_parameter(&self, param_name: &str, value: f64) -> Result<()> {
        let mut params = self.current_parameters.write().await;
        let old_value = params.get(param_name).copied().unwrap_or(0.0);
        params.insert(param_name.to_string(), value.clamp(0.0, 1.0));

        // 记录调整
        let mut adjustments = self.adjustment_history.write().await;
        adjustments.push_back(ParameterAdjustment {
            parameter_name: param_name.to_string(),
            old_value,
            new_value: value,
            reason: "Manual adjustment".to_string(),
            adjusted_at: Utc::now(),
            expected_improvement: 0.0,
        });

        info!("手动设置参数 {}: {:.3} -> {:.3}", param_name, old_value, value);
        Ok(())
    }

    /// 获取学习统计
    pub async fn get_statistics(&self) -> LearningStatistics {
        self.statistics.read().await.clone()
    }

    /// 获取参数调整历史
    pub async fn get_adjustment_history(&self, limit: usize) -> Vec<ParameterAdjustment> {
        let adjustments = self.adjustment_history.read().await;
        adjustments.iter().rev().take(limit).cloned().collect()
    }

    /// 在线学习更新（定期调用）
    pub async fn online_learning_update(&self) -> Result<()> {
        if !self.config.enable_learning {
            return Ok(());
        }

        debug!("执行在线学习更新");

        // 评估性能并调整
        self.trigger_learning().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_adaptive_learning() {
        let engine = AdaptiveLearningEngine::with_defaults();

        // 记录性能指标
        let metrics = PerformanceMetrics {
            accuracy: 0.8,
            latency_ms: 100,
            throughput: 100.0,
            user_satisfaction: 0.9,
            timestamp: Utc::now(),
        };

        engine.record_performance(metrics).await.unwrap();

        // 获取统计
        let stats = engine.get_statistics().await;
        assert_eq!(stats.current_strategy, LearningStrategy::Balanced);
    }

    #[tokio::test]
    async fn test_parameter_adjustment() {
        let engine = AdaptiveLearningEngine::with_defaults();

        // 设置参数
        engine.set_parameter("vector_weight", 0.8).await.unwrap();

        // 获取参数
        let value = engine.get_parameter("vector_weight").await;
        assert_eq!(value, Some(0.8));
    }
}

