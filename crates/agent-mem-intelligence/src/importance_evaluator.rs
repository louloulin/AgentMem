//! 重要性评估器
//!
//! 提供动态重要性评估和置信度计算功能，包括：
//! - 内容重要性分析
//! - 上下文相关性评估
//! - 时间衰减计算
//! - 用户行为分析

use crate::fact_extraction::{EntityType, RelationType, StructuredFact};
use agent_mem_traits::{MemoryV4 as Memory, Result};
use agent_mem_llm::LLMProvider;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

/// 重要性评估结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportanceEvaluation {
    /// 记忆ID
    pub memory_id: String,
    /// 重要性分数 (0.0-1.0)
    pub importance_score: f32,
    /// 置信度 (0.0-1.0)
    pub confidence: f32,
    /// 评估因子
    pub factors: ImportanceFactors,
    /// 评估时间
    pub evaluated_at: chrono::DateTime<chrono::Utc>,
    /// 评估原因
    pub reasoning: String,
}

/// 重要性评估因子
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportanceFactors {
    /// 内容复杂度分数
    pub content_complexity: f32,
    /// 实体重要性分数
    pub entity_importance: f32,
    /// 关系重要性分数
    pub relation_importance: f32,
    /// 时间相关性分数
    pub temporal_relevance: f32,
    /// 用户交互分数
    pub user_interaction: f32,
    /// 上下文相关性分数
    pub contextual_relevance: f32,
    /// 情感强度分数
    pub emotional_intensity: f32,
}

impl Default for ImportanceFactors {
    fn default() -> Self {
        Self {
            content_complexity: 0.0,
            entity_importance: 0.0,
            relation_importance: 0.0,
            temporal_relevance: 0.0,
            user_interaction: 0.0,
            contextual_relevance: 0.0,
            emotional_intensity: 0.0,
        }
    }
}

/// 重要性评估器配置
#[derive(Debug, Clone)]
pub struct ImportanceEvaluatorConfig {
    /// 内容复杂度权重
    pub content_complexity_weight: f32,
    /// 实体重要性权重
    pub entity_importance_weight: f32,
    /// 关系重要性权重
    pub relation_importance_weight: f32,
    /// 时间相关性权重
    pub temporal_relevance_weight: f32,
    /// 用户交互权重
    pub user_interaction_weight: f32,
    /// 上下文相关性权重
    pub contextual_relevance_weight: f32,
    /// 情感强度权重
    pub emotional_intensity_weight: f32,
    /// 时间衰减因子
    pub time_decay_factor: f32,
}

impl Default for ImportanceEvaluatorConfig {
    fn default() -> Self {
        Self {
            content_complexity_weight: 0.15,
            entity_importance_weight: 0.20,
            relation_importance_weight: 0.20,
            temporal_relevance_weight: 0.10,
            user_interaction_weight: 0.15,
            contextual_relevance_weight: 0.15,
            emotional_intensity_weight: 0.05,
            time_decay_factor: 0.95, // 每天衰减5%
        }
    }
}

/// 重要性评估器
pub struct ImportanceEvaluator {
    llm: Arc<dyn LLMProvider + Send + Sync>,
    config: ImportanceEvaluatorConfig,
}

impl ImportanceEvaluator {
    /// 创建新的重要性评估器
    pub fn new(llm: Arc<dyn LLMProvider + Send + Sync>, config: ImportanceEvaluatorConfig) -> Self {
        Self { llm, config }
    }

    /// 评估记忆重要性
    pub async fn evaluate_importance(
        &self,
        memory: &Memory,
        facts: &[StructuredFact],
        context_memories: &[Memory],
    ) -> Result<ImportanceEvaluation> {
        info!("开始评估记忆重要性: {}", memory.id);

        // 计算各个评估因子
        let factors = self
            .calculate_importance_factors(memory, facts, context_memories)
            .await?;

        // 计算综合重要性分数
        let importance_score = self.calculate_weighted_score(&factors);

        // 评估置信度
        let confidence = self.calculate_confidence(&factors);

        // 生成评估原因
        let reasoning = self.generate_reasoning(&factors, importance_score).await?;

        Ok(ImportanceEvaluation {
            memory_id: memory.id.as_str().to_string(),
            importance_score,
            confidence,
            factors,
            evaluated_at: chrono::Utc::now(),
            reasoning,
        })
    }

    /// 批量评估重要性
    pub async fn evaluate_batch_importance(
        &self,
        memories: &[Memory],
        facts_map: &HashMap<String, Vec<StructuredFact>>,
        context_memories: &[Memory],
    ) -> Result<Vec<ImportanceEvaluation>> {
        info!("开始批量评估 {} 个记忆的重要性", memories.len());

        let mut evaluations = Vec::new();

        for memory in memories {
            let facts = facts_map.get(memory.id.as_str()).cloned().unwrap_or_default();
            let evaluation = self
                .evaluate_importance(memory, &facts, context_memories)
                .await?;
            evaluations.push(evaluation);
        }

        info!("完成批量重要性评估");
        Ok(evaluations)
    }

    /// 计算重要性评估因子
    async fn calculate_importance_factors(
        &self,
        memory: &Memory,
        facts: &[StructuredFact],
        context_memories: &[Memory],
    ) -> Result<ImportanceFactors> {
        let mut factors = ImportanceFactors::default();

        // 1. 内容复杂度分析
        let content_text = memory.content.as_text().unwrap_or("");
        factors.content_complexity = self.analyze_content_complexity(content_text).await?;

        // 2. 实体重要性分析
        factors.entity_importance = self.analyze_entity_importance(facts);

        // 3. 关系重要性分析
        factors.relation_importance = self.analyze_relation_importance(facts);

        // 4. 时间相关性分析
        factors.temporal_relevance = self.analyze_temporal_relevance(memory);

        // 5. 用户交互分析
        factors.user_interaction = self.analyze_user_interaction(memory);

        // 6. 上下文相关性分析
        factors.contextual_relevance = self
            .analyze_contextual_relevance(memory, context_memories)
            .await?;

        // 7. 情感强度分析
        let content_text = memory.content.as_text().unwrap_or("");
        factors.emotional_intensity = self.analyze_emotional_intensity(content_text).await?;

        Ok(factors)
    }

    /// 分析内容复杂度
    async fn analyze_content_complexity(&self, content: &str) -> Result<f32> {
        // 基于多个指标计算内容复杂度
        let length_score = (content.len() as f32 / 1000.0).min(1.0);
        let word_count = content.split_whitespace().count() as f32;
        let word_score = (word_count / 100.0).min(1.0);

        // 计算句子复杂度
        let sentence_count = content.matches(['。', '.', '!', '?']).count() as f32;
        let avg_sentence_length = if sentence_count > 0.0 {
            word_count / sentence_count
        } else {
            word_count
        };
        let sentence_complexity = (avg_sentence_length / 20.0).min(1.0);

        // 综合复杂度分数
        let complexity = (length_score + word_score + sentence_complexity) / 3.0;
        Ok(complexity)
    }

    /// 分析实体重要性
    fn analyze_entity_importance(&self, facts: &[StructuredFact]) -> f32 {
        if facts.is_empty() {
            return 0.0;
        }

        let mut total_importance = 0.0;
        let mut entity_count = 0;

        for fact in facts {
            for entity in &fact.entities {
                let entity_weight = match entity.entity_type {
                    EntityType::Person => 0.9,
                    EntityType::Organization => 0.8,
                    EntityType::Location => 0.6,
                    EntityType::Product => 0.7,
                    EntityType::Concept => 0.4,
                    EntityType::Date => 0.3,
                    EntityType::Time => 0.3,
                    EntityType::Number => 0.2,
                    EntityType::Money => 0.8,
                    EntityType::Percentage => 0.4,
                    EntityType::Email => 0.5,
                    EntityType::Phone => 0.5,
                    EntityType::Url => 0.3,
                    EntityType::Event => 0.7,
                    EntityType::Object => 0.3,
                    EntityType::Skill => 0.6,
                    EntityType::Language => 0.4,
                    EntityType::Technology => 0.5,
                    EntityType::Other(_) => 0.2,
                };

                total_importance += entity_weight * entity.confidence;
                entity_count += 1;
            }
        }

        if entity_count > 0 {
            total_importance / entity_count as f32
        } else {
            0.0
        }
    }

    /// 分析关系重要性
    fn analyze_relation_importance(&self, facts: &[StructuredFact]) -> f32 {
        if facts.is_empty() {
            return 0.0;
        }

        let mut total_importance = 0.0;
        let mut relation_count = 0;

        for fact in facts {
            for relation in &fact.relations {
                let relation_weight = match relation.relation_type {
                    RelationType::FamilyOf => 0.9,
                    RelationType::WorksAt => 0.8,
                    RelationType::Likes | RelationType::Dislikes => 0.7,
                    RelationType::FriendOf => 0.6,
                    RelationType::HasProperty => 0.5,
                    RelationType::LocatedAt => 0.4,
                    RelationType::ParticipatesIn => 0.6,
                    RelationType::OccursAt => 0.5,
                    RelationType::Causes => 0.8,
                    RelationType::Other(_) => 0.3,
                };

                total_importance += relation_weight * relation.confidence;
                relation_count += 1;
            }
        }

        if relation_count > 0 {
            total_importance / relation_count as f32
        } else {
            0.0
        }
    }

    /// 分析时间相关性
    fn analyze_temporal_relevance(&self, memory: &Memory) -> f32 {
        let now = chrono::Utc::now();
        let age_days = (now - memory.metadata.created_at).num_days() as f32;

        // 应用时间衰减

        // 最近的记忆更重要
        self.config.time_decay_factor.powf(age_days)
    }

    /// 分析用户交互
    fn analyze_user_interaction(&self, memory: &Memory) -> f32 {
        // 基于记忆的访问次数、更新次数等计算用户交互分数
        // 这里使用简化的计算方式
        let base_score = 0.5;

        // 如果有用户ID，说明是用户特定的记忆，重要性更高
        if memory.user_id().is_some() {
            base_score + 0.3
        } else {
            base_score
        }
    }

    /// 分析上下文相关性
    async fn analyze_contextual_relevance(
        &self,
        memory: &Memory,
        context_memories: &[Memory],
    ) -> Result<f32> {
        if context_memories.is_empty() {
            return Ok(0.5);
        }

        // 计算与上下文记忆的相关性
        let mut total_relevance = 0.0;
        let mut count = 0;

        for context_memory in context_memories.iter().take(10) {
            // 限制比较数量
            let content1 = memory.content.as_text().unwrap_or("");
            let content2 = context_memory.content.as_text().unwrap_or("");
            let relevance = self.calculate_content_similarity(content1, content2);
            total_relevance += relevance;
            count += 1;
        }

        if count > 0 {
            Ok(total_relevance / count as f32)
        } else {
            Ok(0.5)
        }
    }

    /// 分析情感强度
    async fn analyze_emotional_intensity(&self, content: &str) -> Result<f32> {
        // 简化的情感强度分析
        let emotional_keywords = [
            "爱", "恨", "喜欢", "讨厌", "开心", "难过", "愤怒", "兴奋", "担心", "害怕", "惊讶",
            "失望", "满意", "不满", "感动", "震惊",
        ];

        let emotional_count = emotional_keywords
            .iter()
            .map(|&keyword| content.matches(keyword).count())
            .sum::<usize>();

        let word_count = content.split_whitespace().count();

        if word_count > 0 {
            let emotional_ratio = emotional_count as f32 / word_count as f32;
            Ok((emotional_ratio * 10.0).min(1.0))
        } else {
            Ok(0.0)
        }
    }

    /// 计算内容相似性
    fn calculate_content_similarity(&self, content1: &str, content2: &str) -> f32 {
        // 简化的相似性计算
        let words1: std::collections::HashSet<&str> = content1.split_whitespace().collect();
        let words2: std::collections::HashSet<&str> = content2.split_whitespace().collect();

        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();

        if union > 0 {
            intersection as f32 / union as f32
        } else {
            0.0
        }
    }

    /// 计算加权分数
    fn calculate_weighted_score(&self, factors: &ImportanceFactors) -> f32 {
        let score = factors.content_complexity * self.config.content_complexity_weight
            + factors.entity_importance * self.config.entity_importance_weight
            + factors.relation_importance * self.config.relation_importance_weight
            + factors.temporal_relevance * self.config.temporal_relevance_weight
            + factors.user_interaction * self.config.user_interaction_weight
            + factors.contextual_relevance * self.config.contextual_relevance_weight
            + factors.emotional_intensity * self.config.emotional_intensity_weight;

        score.clamp(0.0, 1.0)
    }

    /// 计算置信度
    fn calculate_confidence(&self, factors: &ImportanceFactors) -> f32 {
        // 基于各因子的一致性计算置信度
        let factor_values = [
            factors.content_complexity,
            factors.entity_importance,
            factors.relation_importance,
            factors.temporal_relevance,
            factors.user_interaction,
            factors.contextual_relevance,
            factors.emotional_intensity,
        ];

        let mean = factor_values.iter().sum::<f32>() / factor_values.len() as f32;
        let variance = factor_values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f32>()
            / factor_values.len() as f32;

        // 方差越小，置信度越高
        (1.0 - variance).clamp(0.0, 1.0)
    }

    /// 生成评估原因
    async fn generate_reasoning(&self, factors: &ImportanceFactors, score: f32) -> Result<String> {
        let mut reasons = Vec::new();

        if factors.content_complexity > 0.7 {
            reasons.push("内容复杂度较高");
        }
        if factors.entity_importance > 0.7 {
            reasons.push("包含重要实体");
        }
        if factors.relation_importance > 0.7 {
            reasons.push("包含重要关系");
        }
        if factors.temporal_relevance > 0.8 {
            reasons.push("时间相关性强");
        }
        if factors.user_interaction > 0.7 {
            reasons.push("用户交互频繁");
        }
        if factors.contextual_relevance > 0.7 {
            reasons.push("上下文相关性高");
        }
        if factors.emotional_intensity > 0.7 {
            reasons.push("情感强度较高");
        }

        let reasoning = if reasons.is_empty() {
            format!("综合评估分数: {score:.2}")
        } else {
            format!("{}，综合评估分数: {:.2}", reasons.join("，"), score)
        };

        Ok(reasoning)
    }
}
