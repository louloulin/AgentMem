//! Adaptive Search Router - Multi-Armed Bandit (Thompson Sampling)
//! Week 5-6: 自适应路由器，基于Thompson Sampling算法动态选择最优搜索策略

use super::{QueryFeatures, SearchQuery, SearchWeights};
use crate::config::AgentMemConfig;
use anyhow::Result;
use chrono::{DateTime, Utc};
use rand::{distributions::Distribution, Rng};
use rand_distr::Beta;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 搜索策略ID
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum StrategyId {
    /// 向量搜索为主 (0.9/0.1)
    VectorHeavy,
    /// 均衡搜索 (0.7/0.3) - 默认
    Balanced,
    /// 全文搜索为主 (0.3/0.7)
    FulltextHeavy,
    /// 极端向量 (1.0/0.0)
    VectorOnly,
    /// 极端全文 (0.0/1.0)
    FulltextOnly,
}

impl StrategyId {
    pub fn all() -> Vec<Self> {
        vec![
            Self::VectorHeavy,
            Self::Balanced,
            Self::FulltextHeavy,
            Self::VectorOnly,
            Self::FulltextOnly,
        ]
    }

    pub fn to_weights(&self) -> SearchWeights {
        match self {
            Self::VectorHeavy => SearchWeights {
                vector_weight: 0.9,
                fulltext_weight: 0.1,
                confidence: 0.8,
            },
            Self::Balanced => SearchWeights {
                vector_weight: 0.7,
                fulltext_weight: 0.3,
                confidence: 0.9, // 最平衡，置信度高
            },
            Self::FulltextHeavy => SearchWeights {
                vector_weight: 0.3,
                fulltext_weight: 0.7,
                confidence: 0.8,
            },
            Self::VectorOnly => SearchWeights {
                vector_weight: 1.0,
                fulltext_weight: 0.0,
                confidence: 0.7, // 极端策略，置信度稍低
            },
            Self::FulltextOnly => SearchWeights {
                vector_weight: 0.0,
                fulltext_weight: 1.0,
                confidence: 0.7,
            },
        }
    }
}

/// Thompson Sampling 贝塔分布参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThompsonSamplingArm {
    /// 成功次数 (alpha参数)
    pub alpha: f64,
    /// 失败次数 (beta参数)
    pub beta: f64,
    /// 总尝试次数
    pub total_tries: usize,
    /// 最后更新时间
    pub last_updated: DateTime<Utc>,
}

impl Default for ThompsonSamplingArm {
    fn default() -> Self {
        Self {
            alpha: 1.0, // 先验：均匀分布
            beta: 1.0,
            total_tries: 0,
            last_updated: Utc::now(),
        }
    }
}

impl ThompsonSamplingArm {
    /// 采样当前臂的期望收益
    pub fn sample(&self) -> f64 {
        // 尝试创建 Beta 分布
        let beta_dist = match Beta::new(self.alpha, self.beta) {
            Ok(dist) => dist,
            Err(_) => {
                // 如果参数无效，使用默认的Beta分布（alpha=1, beta=1，即均匀分布）
                // Beta(1,1) 总是有效的，因为 alpha > 0 且 beta > 0
                // 如果仍然失败（理论上不应该），使用均匀分布的近似值
                match Beta::new(1.0, 1.0) {
                    Ok(dist) => dist,
                    Err(_) => {
                        // 如果 Beta(1,1) 也失败（理论上不应该），直接返回随机值
                        return rand::thread_rng().gen_range(0.0..1.0);
                    }
                }
            }
        };
        let mut rng = rand::thread_rng();
        beta_dist.sample(&mut rng)
    }

    /// 更新臂的参数（基于观察到的reward）
    pub fn update(&mut self, reward: f64) {
        // reward ∈ [0, 1]，1表示完全成功，0表示完全失败
        self.alpha += reward;
        self.beta += 1.0 - reward;
        self.total_tries += 1;
        self.last_updated = Utc::now();
    }

    /// 获取期望成功率 (alpha / (alpha + beta))
    pub fn expected_rate(&self) -> f64 {
        self.alpha / (self.alpha + self.beta)
    }
}

/// 性能历史记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRecord {
    pub strategy_id: StrategyId,
    pub query_pattern: String,
    pub accuracy: f32,   // 准确率 [0, 1]
    pub latency_ms: u64, // 延迟（毫秒）
    pub reward: f64,     // 综合奖励 [0, 1]
    pub timestamp: DateTime<Utc>,
}

/// 性能历史
pub struct PerformanceHistory {
    records: Vec<PerformanceRecord>,
    max_size: usize,
}

impl PerformanceHistory {
    pub fn new(max_size: usize) -> Self {
        Self {
            records: Vec::new(),
            max_size,
        }
    }

    pub fn record(&mut self, record: PerformanceRecord) {
        self.records.push(record);
        if self.records.len() > self.max_size {
            self.records.remove(0);
        }
    }

    pub fn get_pattern_stats(&self, pattern: &str) -> Option<PatternStats> {
        let pattern_records: Vec<_> = self
            .records
            .iter()
            .filter(|r| r.query_pattern == pattern)
            .collect();

        if pattern_records.is_empty() {
            return None;
        }

        let total = pattern_records.len();
        let avg_accuracy = pattern_records.iter().map(|r| r.accuracy).sum::<f32>() / total as f32;
        let avg_latency = pattern_records.iter().map(|r| r.latency_ms).sum::<u64>() / total as u64;

        Some(PatternStats {
            total_queries: total,
            avg_accuracy,
            avg_latency_ms: avg_latency,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PatternStats {
    pub total_queries: usize,
    pub avg_accuracy: f32,
    pub avg_latency_ms: u64,
}

/// 自适应路由器（核心）
pub struct AdaptiveRouter {
    /// 配置
    config: AgentMemConfig,
    /// Thompson Sampling 多臂老虎机
    bandit: Arc<RwLock<HashMap<StrategyId, ThompsonSamplingArm>>>,
    /// 性能历史
    performance_history: Arc<RwLock<PerformanceHistory>>,
    /// 探索率（ε-greedy中的ε）
    exploration_rate: f64,
}

impl AdaptiveRouter {
    pub fn new(config: AgentMemConfig) -> Self {
        let mut bandit = HashMap::new();
        for strategy_id in StrategyId::all() {
            bandit.insert(strategy_id, ThompsonSamplingArm::default());
        }

        Self {
            config,
            bandit: Arc::new(RwLock::new(bandit)),
            performance_history: Arc::new(RwLock::new(PerformanceHistory::new(10000))),
            exploration_rate: 0.1, // 10%探索，90%利用
        }
    }

    /// 决策：选择最优策略（Thompson Sampling）
    pub async fn decide_strategy(
        &self,
        query: &SearchQuery,
    ) -> Result<(StrategyId, SearchWeights)> {
        let features = QueryFeatures::extract_from_query(&query.query);

        // 探索 vs 利用
        let should_explore = rand::random::<f64>() < self.exploration_rate;

        let strategy_id = if should_explore {
            // 探索：随机选择
            let all_strategies = StrategyId::all();
            let idx = rand::random::<usize>() % all_strategies.len();
            all_strategies[idx]
        } else {
            // 利用：Thompson Sampling选择
            self.thompson_sampling_select().await?
        };

        let weights = strategy_id.to_weights();
        Ok((strategy_id, weights))
    }

    /// Thompson Sampling 选择臂
    async fn thompson_sampling_select(&self) -> Result<StrategyId> {
        let bandit = self.bandit.read().await;

        // 采样所有臂
        let mut samples: Vec<(StrategyId, f64)> = Vec::new();
        for (strategy_id, arm) in bandit.iter() {
            let sample = arm.sample();
            samples.push((*strategy_id, sample));
        }

        // 选择采样值最大的臂
        samples.sort_by(|a, b| b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal));
        Ok(samples[0].0)
    }

    /// 记录性能反馈并更新Bandit
    pub async fn record_performance(
        &self,
        query: &SearchQuery,
        strategy_id: StrategyId,
        accuracy: f32,
        latency_ms: u64,
    ) -> Result<()> {
        // 计算综合奖励
        let reward = self.calculate_reward(accuracy, latency_ms);

        // 更新 Thompson Sampling
        let mut bandit = self.bandit.write().await;
        if let Some(arm) = bandit.get_mut(&strategy_id) {
            arm.update(reward);
        }

        // 记录历史
        let features = QueryFeatures::extract_from_query(&query.query);
        let pattern = self.classify_query_pattern(&features);

        let mut history = self.performance_history.write().await;
        history.record(PerformanceRecord {
            strategy_id,
            query_pattern: pattern,
            accuracy,
            latency_ms,
            reward,
            timestamp: Utc::now(),
        });

        Ok(())
    }

    /// 计算奖励（综合准确率和延迟）
    fn calculate_reward(&self, accuracy: f32, latency_ms: u64) -> f64 {
        // 准确率权重 0.7，延迟权重 0.3
        let accuracy_score = accuracy as f64;

        // 延迟归一化：假设100ms为优秀，500ms为可接受
        let latency_score = if latency_ms < 100 {
            1.0
        } else if latency_ms < 500 {
            1.0 - (latency_ms - 100) as f64 / 400.0
        } else {
            0.0
        };

        0.7 * accuracy_score + 0.3 * latency_score
    }

    /// 分类查询模式
    fn classify_query_pattern(&self, features: &QueryFeatures) -> String {
        if features.has_exact_terms {
            "exact_match".to_string()
        } else if features.has_temporal_indicator {
            "temporal_query".to_string()
        } else if features.is_question && features.semantic_complexity > 0.6 {
            "semantic_question".to_string()
        } else if features.semantic_complexity < 0.3 {
            "simple_keyword".to_string()
        } else {
            "complex_semantic".to_string()
        }
    }

    /// 获取策略统计信息
    pub async fn get_strategy_stats(&self) -> HashMap<StrategyId, ThompsonSamplingArm> {
        self.bandit.read().await.clone()
    }

    /// 获取模式统计信息
    pub async fn get_pattern_stats(&self, pattern: &str) -> Option<PatternStats> {
        self.performance_history
            .read()
            .await
            .get_pattern_stats(pattern)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thompson_sampling_arm() {
        let mut arm = ThompsonSamplingArm::default();

        // 初始期望成功率应该是 0.5 (1/(1+1))
        assert!((arm.expected_rate() - 0.5).abs() < 0.01);

        // 更新成功
        arm.update(1.0);
        // 期望成功率应该增加: 2/(2+1) = 0.666...
        assert!(arm.expected_rate() > 0.6);

        // 更新失败
        arm.update(0.0);
        // 期望成功率应该下降: 2/(2+2) = 0.5
        assert!((arm.expected_rate() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_strategy_weights() {
        let balanced = StrategyId::Balanced;
        let weights = balanced.to_weights();
        assert_eq!(weights.vector_weight, 0.7);
        assert_eq!(weights.fulltext_weight, 0.3);

        let vector_heavy = StrategyId::VectorHeavy;
        let weights = vector_heavy.to_weights();
        assert_eq!(weights.vector_weight, 0.9);
        assert_eq!(weights.fulltext_weight, 0.1);
    }

    #[test]
    fn test_reward_calculation() {
        let config = AgentMemConfig::default();
        let router = AdaptiveRouter::new(config);

        // 高准确率，低延迟
        let reward1 = router.calculate_reward(0.9, 50);
        assert!(reward1 > 0.8); // 应该很高

        // 低准确率，高延迟
        let reward2 = router.calculate_reward(0.3, 600);
        assert!(reward2 < 0.3); // 应该很低

        // 高准确率，高延迟
        let reward3 = router.calculate_reward(0.9, 600);
        assert!(reward3 > 0.6 && reward3 < 0.7); // 中等偏高
    }

    #[tokio::test]
    async fn test_adaptive_router() -> anyhow::Result<()> {
        let config = AgentMemConfig::default();
        let router = AdaptiveRouter::new(config);

        let query = SearchQuery {
            query: "test query".to_string(),
            limit: 10,
            threshold: Some(0.0),
            vector_weight: 0.7,
            fulltext_weight: 0.3,
            filters: None,
            metadata_filters: None,
        Ok(())
        };

        // 决策
        let (strategy_id, weights) = router.decide_strategy(&query).await?;
        assert!(weights.vector_weight + weights.fulltext_weight > 0.0);

        // 记录反馈
        router
            .record_performance(&query, strategy_id, 0.85, 120)
            .await
            .unwrap();

        // 检查统计
        let stats = router.get_strategy_stats().await;
        assert!(stats.contains_key(&strategy_id));
        assert_eq!(stats[&strategy_id].total_tries, 1);
    }
}
