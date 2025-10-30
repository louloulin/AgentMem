//! 记忆重要性评估模块

use agent_mem_traits::{AgentMemError, MemoryType, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 重要性评估结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportanceResult {
    /// 最终重要性分数 (0.0 - 1.0)
    pub importance_score: f32,
    /// 各因子分数
    pub factor_scores: HashMap<String, f32>,
    /// 各因子权重
    pub factor_weights: HashMap<String, f32>,
    /// 评估时间
    pub evaluated_at: DateTime<Utc>,
    /// 评估原因
    pub reasons: Vec<String>,
}

/// 重要性评估配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportanceConfig {
    /// 访问频率权重
    pub frequency_weight: f32,
    /// 时间衰减权重
    pub recency_weight: f32,
    /// 内容长度权重
    pub content_weight: f32,
    /// 记忆类型权重
    pub type_weight: f32,
    /// 关联性权重
    pub association_weight: f32,
    /// 用户交互权重
    pub interaction_weight: f32,
    /// 时间衰减因子（天）
    pub decay_factor: f32,
    /// 最小重要性分数
    pub min_importance: f32,
    /// 最大重要性分数
    pub max_importance: f32,
}

impl Default for ImportanceConfig {
    fn default() -> Self {
        Self {
            frequency_weight: 0.2,
            recency_weight: 0.2,
            content_weight: 0.15,
            type_weight: 0.15,
            association_weight: 0.15,
            interaction_weight: 0.15,
            decay_factor: 30.0, // 30天衰减因子
            min_importance: 0.0,
            max_importance: 1.0,
        }
    }
}

/// 记忆信息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    /// 记忆ID
    pub id: String,
    /// 记忆内容
    pub content: String,
    /// 记忆类型
    pub memory_type: MemoryType,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后访问时间
    pub last_accessed: DateTime<Utc>,
    /// 访问次数
    pub access_count: u32,
    /// 用户交互次数
    pub interaction_count: u32,
    /// 嵌入向量
    pub embedding: Option<Vec<f32>>,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

/// 重要性评估器
pub struct ImportanceEvaluator {
    config: ImportanceConfig,
}

impl ImportanceEvaluator {
    /// 创建新的重要性评估器
    pub fn new(config: ImportanceConfig) -> Self {
        Self { config }
    }

    /// 使用默认配置创建
    pub fn default() -> Self {
        Self::new(ImportanceConfig::default())
    }

    /// 评估记忆重要性
    pub fn evaluate_importance(
        &self,
        memory: &MemoryInfo,
        context_memories: &[MemoryInfo],
    ) -> Result<ImportanceResult> {
        let current_time = Utc::now();
        let mut factor_scores = HashMap::new();
        let mut reasons = Vec::new();

        // 1. 计算访问频率分数
        let frequency_score = self.calculate_frequency_score(memory);
        factor_scores.insert("frequency".to_string(), frequency_score);
        if frequency_score > 0.7 {
            reasons.push("High access frequency".to_string());
        }

        // 2. 计算时间新近性分数
        let recency_score = self.calculate_recency_score(memory, current_time);
        factor_scores.insert("recency".to_string(), recency_score);
        if recency_score > 0.8 {
            reasons.push("Recently accessed".to_string());
        }

        // 3. 计算内容丰富度分数
        let content_score = self.calculate_content_score(memory);
        factor_scores.insert("content".to_string(), content_score);
        if content_score > 0.6 {
            reasons.push("Rich content".to_string());
        }

        // 4. 计算记忆类型分数
        let type_score = self.calculate_type_score(memory);
        factor_scores.insert("type".to_string(), type_score);

        // 5. 计算关联性分数
        let association_score = self.calculate_association_score(memory, context_memories)?;
        factor_scores.insert("association".to_string(), association_score);
        if association_score > 0.7 {
            reasons.push("High association with other memories".to_string());
        }

        // 6. 计算用户交互分数
        let interaction_score = self.calculate_interaction_score(memory);
        factor_scores.insert("interaction".to_string(), interaction_score);
        if interaction_score > 0.5 {
            reasons.push("Significant user interaction".to_string());
        }

        // 计算加权总分
        let mut factor_weights = HashMap::new();
        factor_weights.insert("frequency".to_string(), self.config.frequency_weight);
        factor_weights.insert("recency".to_string(), self.config.recency_weight);
        factor_weights.insert("content".to_string(), self.config.content_weight);
        factor_weights.insert("type".to_string(), self.config.type_weight);
        factor_weights.insert("association".to_string(), self.config.association_weight);
        factor_weights.insert("interaction".to_string(), self.config.interaction_weight);

        let importance_score = frequency_score * self.config.frequency_weight
            + recency_score * self.config.recency_weight
            + content_score * self.config.content_weight
            + type_score * self.config.type_weight
            + association_score * self.config.association_weight
            + interaction_score * self.config.interaction_weight;

        // 应用边界限制
        let final_score = importance_score
            .max(self.config.min_importance)
            .min(self.config.max_importance);

        Ok(ImportanceResult {
            importance_score: final_score,
            factor_scores,
            factor_weights,
            evaluated_at: current_time,
            reasons,
        })
    }

    /// 计算访问频率分数
    fn calculate_frequency_score(&self, memory: &MemoryInfo) -> f32 {
        if memory.access_count == 0 {
            return 0.0;
        }

        // 使用对数函数避免频率过高时的饱和
        let log_frequency = (memory.access_count as f32).ln();
        let normalized_score = log_frequency / 5.0; // 调整为更合理的基数
        normalized_score.min(1.0)
    }

    /// 计算时间新近性分数
    fn calculate_recency_score(&self, memory: &MemoryInfo, current_time: DateTime<Utc>) -> f32 {
        let time_diff = current_time.signed_duration_since(memory.last_accessed);
        let days_since_access = time_diff.num_days() as f32;

        // 使用指数衰减函数
        (-days_since_access / self.config.decay_factor).exp()
    }

    /// 计算内容丰富度分数
    fn calculate_content_score(&self, memory: &MemoryInfo) -> f32 {
        let content_length = memory.content.len() as f32;
        let word_count = memory.content.split_whitespace().count() as f32;

        // 基于内容长度和词汇数量的综合评分
        let length_score = (content_length / 1000.0).min(1.0); // 1000字符为满分
        let word_score = (word_count / 100.0).min(1.0); // 100词为满分

        (length_score + word_score) / 2.0
    }

    /// 计算记忆类型分数
    fn calculate_type_score(&self, memory: &MemoryInfo) -> f32 {
        match memory.memory_type {
            MemoryType::Factual => 0.8,    // 事实记忆重要性较高
            MemoryType::Episodic => 0.7,   // 情节记忆中等重要
            MemoryType::Procedural => 0.9, // 程序记忆非常重要
            MemoryType::Semantic => 0.8,   // 语义记忆重要性较高
            MemoryType::Working => 0.5,    // 工作记忆重要性较低
            MemoryType::Core => 1.0,       // 核心记忆最重要
            MemoryType::Resource => 0.6,   // 资源记忆中等重要
            MemoryType::Knowledge => 0.85, // 知识记忆重要性高
            MemoryType::Contextual => 0.7, // 上下文记忆中等重要
        }
    }

    /// 计算关联性分数
    fn calculate_association_score(
        &self,
        memory: &MemoryInfo,
        context_memories: &[MemoryInfo],
    ) -> Result<f32> {
        if let Some(ref embedding) = memory.embedding {
            let mut total_similarity = 0.0;
            let mut count = 0;

            for context_memory in context_memories {
                if context_memory.id != memory.id {
                    if let Some(ref context_embedding) = context_memory.embedding {
                        let similarity = self.cosine_similarity(embedding, context_embedding)?;
                        total_similarity += similarity;
                        count += 1;
                    }
                }
            }

            if count > 0 {
                let avg_similarity = total_similarity / count as f32;
                Ok(avg_similarity)
            } else {
                Ok(0.0)
            }
        } else {
            Ok(0.0)
        }
    }

    /// 计算用户交互分数
    fn calculate_interaction_score(&self, memory: &MemoryInfo) -> f32 {
        if memory.interaction_count == 0 {
            return 0.0;
        }

        // 基于交互次数的评分
        
        (memory.interaction_count as f32 / 10.0).min(1.0)
    }

    /// 计算余弦相似度
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> Result<f32> {
        if a.len() != b.len() {
            return Err(AgentMemError::validation_error(
                "Vector dimensions must match",
            ));
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return Ok(0.0);
        }

        Ok(dot_product / (norm_a * norm_b))
    }

    /// 批量评估重要性
    pub fn batch_evaluate(&self, memories: &[MemoryInfo]) -> Result<Vec<ImportanceResult>> {
        let mut results = Vec::new();

        for memory in memories {
            let result = self.evaluate_importance(memory, memories)?;
            results.push(result);
        }

        Ok(results)
    }

    /// 根据重要性排序记忆
    pub fn rank_memories(&self, memories: &[MemoryInfo]) -> Result<Vec<(usize, ImportanceResult)>> {
        let results = self.batch_evaluate(memories)?;

        let mut ranked: Vec<(usize, ImportanceResult)> = results.into_iter().enumerate().collect();

        // 按重要性分数降序排序
        ranked.sort_by(|(_, a), (_, b)| {
            b.importance_score
                .partial_cmp(&a.importance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(ranked)
    }

    /// 更新配置
    pub fn update_config(&mut self, config: ImportanceConfig) {
        self.config = config;
    }

    /// 获取当前配置
    pub fn get_config(&self) -> &ImportanceConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_memory(id: &str, content: &str, access_count: u32) -> MemoryInfo {
        MemoryInfo {
            id: id.to_string(),
            content: content.to_string(),
            memory_type: MemoryType::Factual,
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            access_count,
            interaction_count: 0,
            embedding: Some(vec![1.0, 0.0, 0.0]),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_importance_evaluator_creation() {
        let evaluator = ImportanceEvaluator::default();
        assert_eq!(evaluator.config.frequency_weight, 0.2);
        assert_eq!(evaluator.config.recency_weight, 0.2);
    }

    #[test]
    fn test_calculate_frequency_score() {
        let evaluator = ImportanceEvaluator::default();

        let memory_zero = create_test_memory("0", "test", 0);
        let memory_low = create_test_memory("1", "test", 2);
        let memory_high = create_test_memory("2", "test", 10);

        let score_zero = evaluator.calculate_frequency_score(&memory_zero);
        let score_low = evaluator.calculate_frequency_score(&memory_low);
        let score_high = evaluator.calculate_frequency_score(&memory_high);

        assert_eq!(score_zero, 0.0);
        assert!(score_high > score_low);
        assert!(score_low > 0.0);
        assert!(score_high <= 1.0);
    }

    #[test]
    fn test_calculate_content_score() {
        let evaluator = ImportanceEvaluator::default();

        let memory_short = create_test_memory("1", "short", 1);
        let memory_long = create_test_memory("2", &"long content ".repeat(50), 1);

        let score_short = evaluator.calculate_content_score(&memory_short);
        let score_long = evaluator.calculate_content_score(&memory_long);

        assert!(score_long > score_short);
    }

    #[test]
    fn test_calculate_type_score() {
        let evaluator = ImportanceEvaluator::default();

        let mut memory = create_test_memory("1", "test", 1);

        memory.memory_type = MemoryType::Procedural;
        let procedural_score = evaluator.calculate_type_score(&memory);

        memory.memory_type = MemoryType::Working;
        let working_score = evaluator.calculate_type_score(&memory);

        assert!(procedural_score > working_score);
    }

    #[test]
    fn test_evaluate_importance() {
        let evaluator = ImportanceEvaluator::default();
        let memory = create_test_memory("1", "test content", 5);
        let context = vec![create_test_memory("2", "other content", 3)];

        let result = evaluator.evaluate_importance(&memory, &context).unwrap();

        assert!(result.importance_score >= 0.0);
        assert!(result.importance_score <= 1.0);
        assert!(!result.factor_scores.is_empty());
        assert!(!result.factor_weights.is_empty());
    }

    #[test]
    fn test_batch_evaluate() {
        let evaluator = ImportanceEvaluator::default();
        let memories = vec![
            create_test_memory("1", "first memory", 5),
            create_test_memory("2", "second memory", 10),
            create_test_memory("3", "third memory", 2),
        ];

        let results = evaluator.batch_evaluate(&memories).unwrap();
        assert_eq!(results.len(), 3);

        for result in &results {
            assert!(result.importance_score >= 0.0);
            assert!(result.importance_score <= 1.0);
        }
    }

    #[test]
    fn test_rank_memories() {
        let evaluator = ImportanceEvaluator::default();
        let memories = vec![
            create_test_memory("1", "low importance", 1),
            create_test_memory("2", "high importance", 20),
            create_test_memory("3", "medium importance", 5),
        ];

        let ranked = evaluator.rank_memories(&memories).unwrap();
        assert_eq!(ranked.len(), 3);

        // 检查是否按重要性降序排列
        assert!(ranked[0].1.importance_score >= ranked[1].1.importance_score);
        assert!(ranked[1].1.importance_score >= ranked[2].1.importance_score);
    }

    #[test]
    fn test_cosine_similarity() {
        let evaluator = ImportanceEvaluator::default();

        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let similarity = evaluator.cosine_similarity(&a, &b).unwrap();
        assert!((similarity - 1.0).abs() < 1e-6);

        let c = vec![0.0, 1.0, 0.0];
        let similarity2 = evaluator.cosine_similarity(&a, &c).unwrap();
        assert!((similarity2 - 0.0).abs() < 1e-6);
    }
}
