//! Memory Scheduler Implementation
//!
//! 默认的记忆调度器实现，基于以下因素选择记忆：
//! - 查询相关性（从搜索引擎获取）
//! - 记忆重要性（从 ImportanceScorer 获取）
//! - 时间新鲜度（基于指数衰减模型）
//!
//! # 调度分数计算
//!
//! ```text
//! schedule_score = α * relevance + β * importance + γ * recency
//!
//! 其中：
//! - relevance: 搜索相关性分数（0-1）
//! - importance: 记忆重要性分数（0-1）
//! - recency: 时间新鲜度分数（0-1）
//! - α, β, γ: 可配置的权重系数
//! ```
//!
//! # 时间衰减模型
//!
//! ```text
//! recency = exp(-λ * age_in_days)
//!
//! 其中 λ 是衰减率（默认 0.1，即每天衰减 10%）
//! ```
//!
//! # 参考文献
//!
//! - MemOS: A Memory OS for AI System (ACL 2025)
//! - AgentMem 2.6 发展路线图

pub mod time_decay;

use agent_mem_traits::{
    AgentMemError, Memory, MemoryScheduler, Result, ScheduleConfig, ScheduleContext,
};
use std::collections::HashMap;
use std::sync::Arc;
use time_decay::TimeDecayModel;
use tracing::{debug, instrument};

pub use time_decay::ExponentialDecayModel;

/// 默认的记忆调度器
///
/// 综合考虑相关性、重要性和时效性来选择记忆。
pub struct DefaultMemoryScheduler {
    /// 调度器配置
    config: ScheduleConfig,

    /// 时间衰减模型
    time_decay_model: Arc<dyn TimeDecayModel>,

    /// 记忆重要性缓存（可选）
    importance_cache: Arc<parking_lot::RwLock<HashMap<String, f64>>>,
}

impl DefaultMemoryScheduler {
    /// 创建新的调度器
    ///
    /// # 参数
    ///
    /// - `config`: 调度器配置
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// use agent_mem_core::scheduler::{DefaultMemoryScheduler, ExponentialDecayModel};
    /// use agent_mem_traits::ScheduleConfig;
    ///
    /// let scheduler = DefaultMemoryScheduler::new(
    ///     ScheduleConfig::balanced(),
    ///     ExponentialDecayModel::new(0.1)
    /// );
    /// ```
    pub fn new(config: ScheduleConfig, time_decay_model: impl TimeDecayModel + 'static) -> Self {
        config.validate().expect("Invalid scheduler config");

        Self {
            config,
            time_decay_model: Arc::new(time_decay_model),
            importance_cache: Arc::new(parking_lot::RwLock::new(HashMap::new())),
        }
    }

    /// 创建默认调度器（推荐配置）
    pub fn default_config() -> Self {
        Self::new(ScheduleConfig::default(), ExponentialDecayModel::default())
    }

    /// 提取记忆的重要性分数
    ///
    /// 从记忆的 metadata 中提取 importance 字段。
    fn extract_importance(&self, memory: &Memory) -> f64 {
        // 尝试从 system.importance 获取
        if let Some(value) = memory
            .attributes
            .get(&agent_mem_traits::AttributeKey::system("importance"))
        {
            if let agent_mem_traits::AttributeValue::Number(score) = value {
                return *score;
            }
        }

        // 默认重要性（中等）
        0.5
    }

    /// 提取记忆的创建时间戳
    ///
    /// 从记忆的 metadata 中提取 created_at 字段。
    fn extract_created_at(&self, memory: &Memory) -> Option<i64> {
        // 从 metadata.created_at 获取
        let timestamp = memory.metadata.created_at.timestamp();
        return Some(timestamp);

        // 尝试从 attributes 获取
        if let Some(value) = memory
            .attributes
            .get(&agent_mem_traits::AttributeKey::system("created_at"))
        {
            match value {
                agent_mem_traits::AttributeValue::Number(ts) => Some(*ts as i64),
                agent_mem_traits::AttributeValue::String(s) => {
                    // 尝试解析 ISO 8601 格式
                    chrono::DateTime::parse_from_rfc3339(s)
                        .ok()
                        .map(|dt| dt.timestamp())
                }
                _ => None,
            }
        } else {
            None
        }
    }

    /// 计算时间新鲜度分数
    ///
    /// 基于时间衰减模型计算记忆的新鲜度（0-1 之间）。
    fn calculate_recency(&self, memory: &Memory, current_timestamp: i64) -> f64 {
        if let Some(created_at) = self.extract_created_at(memory) {
            let age_seconds = current_timestamp - created_at;
            let age_days = age_seconds as f64 / (24.0 * 3600.0);

            // 使用时间衰减模型
            self.time_decay_model.decay_score(age_days)
        } else {
            // 如果没有创建时间，返回中等新鲜度
            0.5
        }
    }

    /// 计算调度分数
    ///
    /// 综合相关性、重要性和新鲜度计算最终分数。
    fn compute_schedule_score(
        &self,
        relevance: f64,
        importance: f64,
        recency: f64,
    ) -> f64 {
        let config = &self.config;

        // 加权求和
        let score = config.relevance_weight * relevance
            + config.importance_weight * importance
            + config.recency_weight * recency;

        debug!(
            "Schedule score: relevance={:.2}, importance={:.2}, recency={:.2}, final={:.2}",
            relevance, importance, recency, score
        );

        score
    }
}

#[async_trait::async_trait]
impl MemoryScheduler for DefaultMemoryScheduler {
    #[instrument(skip(self, candidates))]
    async fn select_memories(
        &self,
        query: &str,
        candidates: Vec<Memory>,
        top_k: usize,
    ) -> Result<Vec<Memory>> {
        debug!(
            "Selecting top-{} memories from {} candidates for query: {}",
            top_k,
            candidates.len(),
            query
        );

        if candidates.is_empty() {
            return Ok(vec![]);
        }

        let current_timestamp = chrono::Utc::now().timestamp();

        // 为每个候选记忆计算调度分数
        let mut scored_memories = futures::future::join_all(candidates.into_iter().map(|memory| {
            let scheduler = self;
            async move {
                let relevance = 0.5; // TODO: 从搜索引擎获取
                let importance = scheduler.extract_importance(&memory);
                let recency = scheduler.calculate_recency(&memory, current_timestamp);
                let score = scheduler.compute_schedule_score(relevance, importance, recency);

                (memory, score)
            }
        }))
        .await;

        // 按分数降序排序
        scored_memories.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 过滤低于阈值的结果
        let min_score = self.config.min_score;
        scored_memories.retain(|(_, score)| *score >= min_score);

        // 取 top-k
        let selected: Vec<Memory> = scored_memories
            .into_iter()
            .take(top_k)
            .map(|(memory, _)| memory)
            .collect();

        debug!("Selected {} memories", selected.len());

        Ok(selected)
    }

    #[instrument(skip(self, memory, context))]
    async fn schedule_score(
        &self,
        memory: &Memory,
        _query: &str,
        context: &ScheduleContext,
    ) -> Result<f64> {
        let relevance = context.relevance_score;
        let importance = self.extract_importance(memory);
        let recency = self.calculate_recency(memory, context.current_timestamp);

        let score = self.compute_schedule_score(relevance, importance, recency);

        Ok(score)
    }

    fn config(&self) -> ScheduleConfig {
        self.config.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_mem_traits::{
        AttributeKey, AttributeValue, Content, MemoryBuilder, Metadata,
    };

    fn create_test_memory(importance: f64, days_ago: f64) -> Memory {
        let created_at = (chrono::Utc::now() - chrono::Duration::days(days_ago as i64)).timestamp();

        MemoryBuilder::new()
            .content(Content::Text(format!("Test memory from {} days ago", days_ago)))
            .build()
            .with_attribute(
                AttributeKey::system("importance"),
                AttributeValue::Number(importance as f64),
            )
    }

    #[tokio::test]
    async fn test_select_memories() {
        let scheduler = DefaultMemoryScheduler::default_config();

        // 创建测试记忆
        let candidates = vec![
            create_test_memory(0.9, 1.0),  // 高重要性，新
            create_test_memory(0.5, 10.0), // 中重要性，旧
            create_test_memory(0.8, 5.0),  // 高重要性，中等时间
        ];

        // 选择 top-2
        let selected = scheduler
            .select_memories("test query", candidates, 2)
            .await
            .unwrap();

        assert_eq!(selected.len(), 2);
        // 高重要性的记忆应该被选中
    }

    #[test]
    fn test_extract_importance() {
        let scheduler = DefaultMemoryScheduler::default_config();

        let memory = create_test_memory(0.75, 1.0);
        let importance = scheduler.extract_importance(&memory);

        assert_eq!(importance, 0.75);
    }

    #[test]
    fn test_calculate_recency() {
        let scheduler = DefaultMemoryScheduler::default_config();
        let current_timestamp = chrono::Utc::now().timestamp();

        // 新记忆
        let recent_memory = create_test_memory(0.5, 0.1);
        let recent_recency = scheduler.calculate_recency(&recent_memory, current_timestamp);
        assert!(recent_recency > 0.9);

        // 旧记忆
        let old_memory = create_test_memory(0.5, 100.0);
        let old_recency = scheduler.calculate_recency(&old_memory, current_timestamp);
        assert!(old_recency < 0.1);
    }

    #[tokio::test]
    async fn test_schedule_score() {
        let scheduler = DefaultMemoryScheduler::default_config();

        let memory = create_test_memory(0.8, 1.0);
        let context = ScheduleContext::new(0.7);

        let score = scheduler
            .schedule_score(&memory, "test query", &context)
            .await
            .unwrap();

        assert!(score >= 0.0 && score <= 1.0);
        // 分数应该在合理范围内
        assert!(score > 0.5, "Score should be > 0.5 for high-quality memory");
    }
}
