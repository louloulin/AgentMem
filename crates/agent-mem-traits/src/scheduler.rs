//! Memory Scheduler Traits
//!
//! 记忆调度器 trait，用于从候选记忆中选择最相关的记忆。
//! 参考 MemOS (ACL 2025) 的记忆调度算法设计。
//!
//! # 核心概念
//!
//! ## MemoryScheduler
//!
//! 负责从大量候选记忆中选择最相关的 top-k 个记忆。考虑因素：
//! - **相关性（Relevance）**: 与查询的语义相似度
//! - **重要性（Importance）**: 记忆的重要程度
//! - **时效性（Recency）**: 记忆的新鲜度（时间衰减）
//!
//! # 示例
//!
//! ```rust,ignore
//! use agent_mem_traits::{MemoryScheduler, Memory, ScheduleContext};
//!
//! async fn example(scheduler: &dyn MemoryScheduler) -> Result<Vec<Memory>> {
//!     let query = "What did I work on yesterday?";
//!     let candidates = fetch_candidates().await?;
//!
//!     // 调度器会自动选择最相关的 top-10 记忆
//!     let selected = scheduler.select_memories(query, candidates, 10).await?;
//!
//!     Ok(selected)
//! }
//! ```
//!
//! # 参考文献
//!
//! - MemOS: A Memory OS for AI System (ACL 2025)
//! - AgentMem 2.6 发展路线图

use crate::{Memory, Result, AgentMemError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 记忆调度器
///
/// 负责从候选记忆中选择最相关的记忆。综合考虑：
/// 1. 查询相关性（从搜索引擎获取）
/// 2. 记忆重要性（从 ImportanceScorer 获取）
/// 3. 时间新鲜度（基于时间衰减模型）
#[async_trait]
pub trait MemoryScheduler: Send + Sync {
    /// 从候选记忆中选择最相关的 top-k 个
    ///
    /// # 参数
    ///
    /// - `query`: 用户查询
    /// - `candidates`: 候选记忆列表（已包含相关性分数）
    /// - `top_k`: 返回的记忆数量
    ///
    /// # 返回
    ///
    /// 按调度分数排序的 top-k 记忆
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// let selected = scheduler.select_memories(
    ///     "What did I work on?",
    ///     candidates,
    ///     10
    /// ).await?;
    /// ```
    async fn select_memories(
        &self,
        query: &str,
        candidates: Vec<Memory>,
        top_k: usize,
    ) -> Result<Vec<Memory>>;

    /// 计算单个记忆的调度分数
    ///
    /// # 参数
    ///
    /// - `memory`: 要评估的记忆
    /// - `query`: 用户查询
    /// - `context`: 调度上下文（包含相关性分数等）
    ///
    /// # 返回
    ///
    /// 调度分数（0-1 之间，越高越相关）
    async fn schedule_score(
        &self,
        memory: &Memory,
        query: &str,
        context: &ScheduleContext,
    ) -> Result<f64>;

    /// 获取调度器配置
    fn config(&self) -> ScheduleConfig;
}

/// 调度上下文
///
/// 包含调度所需的额外信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleContext {
    /// 查询相关性分数（从搜索引擎获取）
    pub relevance_score: f64,

    /// 当前时间戳（用于计算时间衰减）
    pub current_timestamp: i64,

    /// 额外的上下文信息
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ScheduleContext {
    /// 创建新的调度上下文
    pub fn new(relevance_score: f64) -> Self {
        Self {
            relevance_score,
            current_timestamp: chrono::Utc::now().timestamp(),
            metadata: HashMap::new(),
        }
    }

    /// 添加元数据
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// 调度器配置
///
/// 控制调度算法的参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleConfig {
    /// 相关性权重（0-1）
    pub relevance_weight: f64,

    /// 重要性权重（0-1）
    pub importance_weight: f64,

    /// 新鲜度权重（0-1）
    pub recency_weight: f64,

    /// 时间衰减率（lambda，用于指数衰减）
    /// 值越大，衰减越快
    pub decay_rate: f64,

    /// 最小调度分数阈值
    /// 低于此分数的记忆不会被返回
    pub min_score: f64,
}

impl Default for ScheduleConfig {
    fn default() -> Self {
        // 基于经验值的默认配置
        // 相关性最重要（0.5），重要性次之（0.3），新鲜度再次（0.2）
        Self {
            relevance_weight: 0.5,
            importance_weight: 0.3,
            recency_weight: 0.2,
            decay_rate: 0.1,  // 每天衰减 10%
            min_score: 0.1,   // 最低分数阈值
        }
    }
}

impl ScheduleConfig {
    /// 验证配置是否有效
    pub fn validate(&self) -> Result<()> {
        // 权重之和应该接近 1.0（允许 0.01 的误差）
        let total = self.relevance_weight + self.importance_weight + self.recency_weight;
        if (total - 1.0).abs() > 0.01 {
            return Err(AgentMemError::ValidationError(format!(
                "Weight sum must be 1.0, got {}",
                total
            )));
        }

        // 权重必须在 0-1 之间
        if !(0.0..=1.0).contains(&self.relevance_weight)
            || !(0.0..=1.0).contains(&self.importance_weight)
            || !(0.0..=1.0).contains(&self.recency_weight)
        {
            return Err(AgentMemError::ValidationError(
                "Weights must be between 0 and 1".to_string(),
            ));
        }

        // 衰减率必须为正
        if self.decay_rate <= 0.0 {
            return Err(AgentMemError::ValidationError(
                "Decay rate must be positive".to_string(),
            ));
        }

        // 最小分数必须在 0-1 之间
        if !(0.0..=1.0).contains(&self.min_score) {
            return Err(AgentMemError::ValidationError(
                "Min score must be between 0 and 1".to_string(),
            ));
        }

        Ok(())
    }

    /// 创建平衡配置（默认）
    pub fn balanced() -> Self {
        Self::default()
    }

    /// 创建相关性优先配置
    pub fn relevance_focused() -> Self {
        Self {
            relevance_weight: 0.7,
            importance_weight: 0.2,
            recency_weight: 0.1,
            ..Default::default()
        }
    }

    /// 创建重要性优先配置
    pub fn importance_focused() -> Self {
        Self {
            relevance_weight: 0.2,
            importance_weight: 0.7,
            recency_weight: 0.1,
            ..Default::default()
        }
    }

    /// 创建新鲜度优先配置
    pub fn recency_focused() -> Self {
        Self {
            relevance_weight: 0.2,
            importance_weight: 0.2,
            recency_weight: 0.6,
            decay_rate: 0.2,  // 更快的衰减
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schedule_config_validation() {
        // 有效配置
        let config = ScheduleConfig::default();
        assert!(config.validate().is_ok());

        // 权重之和不为 1
        let invalid_config = ScheduleConfig {
            relevance_weight: 0.8,
            importance_weight: 0.3,
            recency_weight: 0.2,
            ..Default::default()
        };
        assert!(invalid_config.validate().is_err());

        // 负权重
        let invalid_config = ScheduleConfig {
            relevance_weight: -0.1,
            ..Default::default()
        };
        assert!(invalid_config.validate().is_err());

        // 负衰减率
        let invalid_config = ScheduleConfig {
            decay_rate: -0.1,
            ..Default::default()
        };
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_schedule_config_presets() {
        // 测试各种预设配置
        let configs = vec![
            ScheduleConfig::balanced(),
            ScheduleConfig::relevance_focused(),
            ScheduleConfig::importance_focused(),
            ScheduleConfig::recency_focused(),
        ];

        for config in configs {
            assert!(config.validate().is_ok());
        }
    }

    #[test]
    fn test_schedule_context() {
        let context = ScheduleContext::new(0.8)
            .with_metadata("key".to_string(), serde_json::json!("value"));

        assert_eq!(context.relevance_score, 0.8);
        assert_eq!(context.metadata.len(), 1);
        assert!(context.metadata.contains_key("key"));
    }
}
