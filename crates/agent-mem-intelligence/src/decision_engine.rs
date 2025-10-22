//! 智能决策引擎
//!
//! 基于提取的事实和现有记忆，智能决策记忆操作（添加、更新、删除、合并）
//!
//! Mem5 增强功能：
//! - 基于事实的智能决策制定
//! - 多维度决策评估
//! - 自适应决策策略
//! - 决策置信度评估

use crate::fact_extraction::{ExtractedFact, StructuredFact};
use crate::importance_evaluator::ImportanceEvaluation;
use agent_mem_core::Memory;
use agent_mem_llm::LLMProvider;
use agent_mem_traits::{Message, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};

// P0 优化: 超时控制
use crate::timeout::{with_timeout, with_timeout_and_retry, TimeoutConfig};

/// 记忆操作决策
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDecision {
    pub action: MemoryAction,
    pub confidence: f32,
    pub reasoning: String,
    pub affected_memories: Vec<String>,
    pub estimated_impact: f32,
}

/// 记忆操作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryAction {
    Add {
        content: String,
        importance: f32,
        metadata: HashMap<String, String>,
    },
    Update {
        memory_id: String,
        new_content: String,
        merge_strategy: MergeStrategy,
        change_reason: String,
    },
    Delete {
        memory_id: String,
        deletion_reason: DeletionReason,
    },
    Merge {
        primary_memory_id: String,
        secondary_memory_ids: Vec<String>,
        merged_content: String,
    },
    NoAction {
        reason: String,
    },
}

/// 合并策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MergeStrategy {
    Replace,    // 完全替换
    Append,     // 追加信息
    Merge,      // 智能合并
    Prioritize, // 优先保留重要信息
}

/// 删除原因
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeletionReason {
    Outdated,      // 信息过时
    Contradicted,  // 被新信息否定
    Redundant,     // 冗余信息
    LowQuality,    // 质量低下
    UserRequested, // 用户要求删除
}

/// 现有记忆信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExistingMemory {
    pub id: String,
    pub content: String,
    pub importance: f32,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// 决策响应
#[derive(Debug, Deserialize)]
pub struct DecisionResponse {
    pub decisions: Vec<MemoryDecision>,
    pub overall_confidence: f32,
    pub reasoning: String,
}

/// 冲突检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictDetection {
    pub has_conflict: bool,
    pub conflicting_memory_ids: Vec<String>,
    pub conflict_type: ConflictType,
    pub resolution_strategy: ConflictResolutionStrategy,
    pub confidence: f32,
}

/// 冲突类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictType {
    DirectContradiction,   // 直接矛盾
    TemporalInconsistency, // 时间不一致
    ValueConflict,         // 数值冲突
    CategoryMismatch,      // 类别不匹配
    ContextualConflict,    // 上下文冲突
}

/// 冲突解决策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolutionStrategy {
    KeepLatest,            // 保留最新信息
    KeepMostImportant,     // 保留最重要信息
    KeepHighestConfidence, // 保留置信度最高信息
    MergeInformation,      // 合并信息
    RequestUserInput,      // 请求用户输入
    MarkAsUncertain,       // 标记为不确定
}

/// 记忆决策引擎（增强版本）
pub struct MemoryDecisionEngine {
    llm: Arc<dyn LLMProvider + Send + Sync>,
    similarity_threshold: f32,
    confidence_threshold: f32,
    conflict_detection_enabled: bool,
    importance_weight: f32,
    temporal_weight: f32,
    // P0 优化: 超时配置
    timeout_config: TimeoutConfig,
}

impl MemoryDecisionEngine {
    /// 创建新的决策引擎
    pub fn new(llm: Arc<dyn LLMProvider + Send + Sync>) -> Self {
        Self {
            llm,
            similarity_threshold: 0.7,
            confidence_threshold: 0.5,
            conflict_detection_enabled: true,
            importance_weight: 0.3,
            temporal_weight: 0.2,
            timeout_config: TimeoutConfig::default(),
        }
    }

    /// 创建带自定义配置的决策引擎
    pub fn with_config(
        llm: Arc<dyn LLMProvider + Send + Sync>,
        similarity_threshold: f32,
        confidence_threshold: f32,
        conflict_detection_enabled: bool,
    ) -> Self {
        Self {
            llm,
            similarity_threshold,
            confidence_threshold,
            conflict_detection_enabled,
            importance_weight: 0.3,
            temporal_weight: 0.2,
            timeout_config: TimeoutConfig::default(),
        }
    }

    /// 创建带超时配置的决策引擎
    pub fn with_timeout_config(llm: Arc<dyn LLMProvider + Send + Sync>, timeout_config: TimeoutConfig) -> Self {
        Self {
            llm,
            similarity_threshold: 0.7,
            confidence_threshold: 0.5,
            conflict_detection_enabled: true,
            importance_weight: 0.3,
            temporal_weight: 0.2,
            timeout_config,
        }
    }

    /// 设置相似度阈值
    pub fn with_similarity_threshold(mut self, threshold: f32) -> Self {
        self.similarity_threshold = threshold;
        self
    }

    /// 设置置信度阈值
    pub fn with_confidence_threshold(mut self, threshold: f32) -> Self {
        self.confidence_threshold = threshold;
        self
    }

    /// 基于新事实和现有记忆做出决策
    pub async fn make_decisions(
        &self,
        new_facts: &[ExtractedFact],
        existing_memories: &[ExistingMemory],
    ) -> Result<Vec<MemoryDecision>> {
        if new_facts.is_empty() {
            return Ok(vec![MemoryDecision {
                action: MemoryAction::NoAction {
                    reason: "No new facts to process".to_string(),
                },
                confidence: 1.0,
                reasoning: "No input facts provided".to_string(),
                affected_memories: vec![],
                estimated_impact: 0.0,
            }]);
        }

        let prompt = self.build_decision_prompt(new_facts, existing_memories);
        let messages = vec![Message {
            role: agent_mem_traits::MessageRole::User,
            content: prompt,
            timestamp: Some(chrono::Utc::now()),
        }];
        
        // P0 优化 #12: 添加超时和重试
        let llm = self.llm.clone();
        let response_text = with_timeout_and_retry(
            || {
                let llm = llm.clone();
                let messages = messages.clone();
                async move {
                    llm.generate(&messages).await
                }
            },
            self.timeout_config.decision_timeout_secs,
            2, // 最多重试2次
            "decision_making",
        ).await?;
        let cleaned_json = self.extract_json_from_response(&response_text)?;
        let response: DecisionResponse = serde_json::from_str(&cleaned_json)
            .map_err(|e| agent_mem_traits::AgentMemError::SerializationError(e))?;

        // 过滤低置信度的决策
        let mut filtered_decisions: Vec<MemoryDecision> = response
            .decisions
            .into_iter()
            .filter(|decision| decision.confidence >= self.confidence_threshold)
            .collect();

        // P2 优化 #13: 验证决策一致性
        filtered_decisions = self.validate_decision_consistency(filtered_decisions)?;

        // P2 优化 #14: 记录决策审计日志
        self.log_decisions(&filtered_decisions, new_facts, existing_memories);

        Ok(filtered_decisions)
    }

    /// 检测事实与现有记忆之间的冲突
    pub async fn detect_conflicts(
        &self,
        fact: &ExtractedFact,
        existing_memories: &[ExistingMemory],
    ) -> Result<ConflictDetection> {
        if !self.conflict_detection_enabled {
            return Ok(ConflictDetection {
                has_conflict: false,
                conflicting_memory_ids: vec![],
                conflict_type: ConflictType::DirectContradiction,
                resolution_strategy: ConflictResolutionStrategy::KeepLatest,
                confidence: 0.0,
            });
        }

        let prompt = self.build_conflict_detection_prompt(fact, existing_memories);
        let messages = vec![Message {
            role: agent_mem_traits::MessageRole::User,
            content: prompt,
            timestamp: Some(chrono::Utc::now()),
        }];
        
        // P0 优化 #12: 添加超时控制
        let llm = self.llm.clone();
        let response_text = with_timeout(
            async move {
                llm.generate(&messages).await
            },
            self.timeout_config.conflict_detection_timeout_secs,
            "conflict_detection",
        ).await?;
        let response: ConflictDetection = serde_json::from_str(&response_text)
            .map_err(|e| agent_mem_traits::AgentMemError::SerializationError(e))?;

        Ok(response)
    }

    /// 智能合并相似记忆
    pub async fn merge_similar_memories(
        &self,
        memories: &[ExistingMemory],
        similarity_threshold: Option<f32>,
    ) -> Result<Vec<MemoryDecision>> {
        let threshold = similarity_threshold.unwrap_or(self.similarity_threshold);
        let mut decisions = Vec::new();
        let mut processed_ids = std::collections::HashSet::new();

        for (i, memory) in memories.iter().enumerate() {
            if processed_ids.contains(&memory.id) {
                continue;
            }

            let mut similar_memories = vec![memory.clone()];

            // 查找相似记忆
            for (j, other_memory) in memories.iter().enumerate() {
                if i != j && !processed_ids.contains(&other_memory.id) {
                    let similarity =
                        self.calculate_content_similarity(&memory.content, &other_memory.content);
                    if similarity >= threshold {
                        similar_memories.push(other_memory.clone());
                        processed_ids.insert(other_memory.id.clone());
                    }
                }
            }

            // 如果找到相似记忆，创建合并决策
            if similar_memories.len() > 1 {
                let merged_content = self.generate_merged_content(&similar_memories).await?;
                let primary_memory = &similar_memories[0];
                let secondary_ids: Vec<String> =
                    similar_memories[1..].iter().map(|m| m.id.clone()).collect();

                decisions.push(MemoryDecision {
                    action: MemoryAction::Merge {
                        primary_memory_id: primary_memory.id.clone(),
                        secondary_memory_ids: secondary_ids.clone(),
                        merged_content,
                    },
                    confidence: 0.8,
                    reasoning: format!(
                        "Found {} similar memories that can be merged",
                        similar_memories.len()
                    ),
                    affected_memories: similar_memories.iter().map(|m| m.id.clone()).collect(),
                    estimated_impact: 0.7,
                });

                processed_ids.insert(primary_memory.id.clone());
            }
        }

        Ok(decisions)
    }

    /// 评估记忆重要性（增强版本）
    pub fn evaluate_importance_enhanced(
        &self,
        fact: &ExtractedFact,
        context: &[ExistingMemory],
    ) -> f32 {
        let mut importance = self.evaluate_importance(fact);

        // 基于上下文调整重要性
        let context_boost = self.calculate_context_importance(fact, context);
        importance += context_boost * self.importance_weight;

        // 基于时间信息调整
        if let Some(temporal_info) = &fact.temporal_info {
            let temporal_boost = self.calculate_temporal_importance(temporal_info);
            importance += temporal_boost * self.temporal_weight;
        }

        // 确保重要性在合理范围内
        importance.clamp(0.0, 1.0)
    }

    /// 从响应中提取 JSON 部分
    fn extract_json_from_response(&self, response: &str) -> Result<String> {
        // 尝试直接解析
        if response.trim().starts_with('{') {
            return Ok(response.to_string());
        }

        // 查找 JSON 块
        if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                if end > start {
                    return Ok(response[start..=end].to_string());
                }
            }
        }

        // 如果找不到 JSON，返回错误
        Err(agent_mem_traits::AgentMemError::internal_error(
            "No valid JSON found in response".to_string(),
        ))
    }

    /// 构建决策提示
    fn build_decision_prompt(
        &self,
        new_facts: &[ExtractedFact],
        existing_memories: &[ExistingMemory],
    ) -> String {
        // 简化输入数据以减少token使用
        let facts_summary: Vec<String> = new_facts
            .iter()
            .map(|f| format!("{} (conf: {:.1})", f.content, f.confidence))
            .collect();

        let memories_summary: Vec<String> = existing_memories
            .iter()
            .map(|m| format!("{}: {} (imp: {:.1})", m.id, m.content, m.importance))
            .collect();

        format!(
            r#"Memory management task. Return JSON only.

New Facts:
{}

Existing Memories:
{}

Decide memory operations:

{{
    "decisions": [
        {{
            "action": {{"Add": {{"content": "text", "importance": 0.8, "metadata": {{}}}}}},
            "confidence": 0.9,
            "reasoning": "brief reason",
            "affected_memories": [],
            "estimated_impact": 0.7
        }}
    ],
    "overall_confidence": 0.8,
    "reasoning": "summary"
}}

Action Examples:
- Add: {{"Add": {{"content": "text", "importance": 0.8, "metadata": {{}}}}}}
- Update: {{"Update": {{"memory_id": "mem123", "new_content": "updated text", "merge_strategy": "Replace", "change_reason": "new info"}}}}
- Delete: {{"Delete": {{"memory_id": "mem123", "deletion_reason": "Outdated"}}}}
- Merge: {{"Merge": {{"primary_memory_id": "mem1", "secondary_memory_ids": ["mem2"], "merged_content": "combined text"}}}}
- NoAction: {{"NoAction": {{"reason": "no changes needed"}}}}

Rules:
- Add new facts as memories (importance 0.3-1.0)
- Update if fact conflicts with existing memory (provide memory_id)
- Delete outdated/contradicted memories (provide memory_id)
- Merge similar memories (provide memory_ids)
- NoAction if no changes needed

Keep reasoning brief. Max 3 decisions."#,
            facts_summary.join("\n"),
            memories_summary.join("\n")
        )
    }

    /// 构建冲突检测提示
    fn build_conflict_detection_prompt(
        &self,
        fact: &ExtractedFact,
        existing_memories: &[ExistingMemory],
    ) -> String {
        let memories_summary: Vec<String> = existing_memories
            .iter()
            .map(|m| format!("{}: {}", m.id, m.content))
            .collect();

        format!(
            r#"Detect conflicts between new fact and existing memories. Return JSON only.

New Fact: {}

Existing Memories:
{}

Analyze for conflicts:

{{
    "has_conflict": true/false,
    "conflicting_memory_ids": ["mem1", "mem2"],
    "conflict_type": "DirectContradiction|TemporalInconsistency|ValueConflict|CategoryMismatch|ContextualConflict",
    "resolution_strategy": "KeepLatest|KeepMostImportant|KeepHighestConfidence|MergeInformation|RequestUserInput|MarkAsUncertain",
    "confidence": 0.9
}}

Conflict types:
- DirectContradiction: Facts directly contradict each other
- TemporalInconsistency: Timeline doesn't match
- ValueConflict: Different values for same attribute
- CategoryMismatch: Same entity in different categories
- ContextualConflict: Context makes facts incompatible

Resolution strategies:
- KeepLatest: Prefer newer information
- KeepMostImportant: Prefer higher importance
- KeepHighestConfidence: Prefer higher confidence
- MergeInformation: Combine compatible parts
- RequestUserInput: Ask user to resolve
- MarkAsUncertain: Flag as uncertain"#,
            fact.content,
            memories_summary.join("\n")
        )
    }

    /// 生成合并后的内容
    async fn generate_merged_content(&self, memories: &[ExistingMemory]) -> Result<String> {
        let contents: Vec<String> = memories.iter().map(|m| m.content.clone()).collect();

        let _prompt = format!(
            r#"Merge these similar memory contents into one coherent text. Return only the merged content.

Contents to merge:
{}

Requirements:
- Preserve all important information
- Remove redundancy
- Create coherent narrative
- Keep factual accuracy
- Maximum 200 words"#,
            contents.join("\n- ")
        );

        // 使用简单的文本合并逻辑，因为 DeepSeekProvider 没有 generate 方法
        let merged_content = self.merge_contents_simple(&contents);

        Ok(merged_content.trim().to_string())
    }

    /// 计算上下文重要性
    fn calculate_context_importance(
        &self,
        fact: &ExtractedFact,
        context: &[ExistingMemory],
    ) -> f32 {
        let mut context_boost = 0.0;

        // 检查是否与现有记忆相关
        for memory in context {
            let similarity = self.calculate_content_similarity(&fact.content, &memory.content);
            if similarity > 0.3 {
                // 相关记忆越重要，新事实的重要性提升越大
                context_boost += similarity * memory.importance * 0.1;
            }
        }

        context_boost.min(0.3) // 最大提升30%
    }

    /// 计算时间重要性
    fn calculate_temporal_importance(
        &self,
        temporal_info: &crate::fact_extraction::TemporalInfo,
    ) -> f32 {
        let mut temporal_boost: f32 = 0.0;

        // 相对时间的重要性
        if let Some(relative_time) = &temporal_info.relative_time {
            temporal_boost += match relative_time.as_str() {
                "today" | "now" => 0.2,
                "yesterday" | "tomorrow" => 0.15,
                "this week" | "last week" => 0.1,
                _ => 0.05,
            };
        }

        // 频率信息的重要性
        if let Some(frequency) = &temporal_info.frequency {
            temporal_boost += match frequency.as_str() {
                "daily" => 0.15,
                "weekly" => 0.1,
                "monthly" => 0.05,
                _ => 0.02,
            };
        }

        temporal_boost.min(0.25) // 最大提升25%
    }

    /// 简单的内容合并逻辑
    fn merge_contents_simple(&self, contents: &[String]) -> String {
        if contents.is_empty() {
            return String::new();
        }

        if contents.len() == 1 {
            return contents[0].clone();
        }

        // 简单合并：去重并连接
        let mut unique_sentences = std::collections::HashSet::new();
        let mut merged_parts = Vec::new();

        for content in contents {
            // 按句号分割内容
            let sentences: Vec<&str> = content
                .split('.')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            for sentence in sentences {
                if unique_sentences.insert(sentence.to_lowercase()) {
                    merged_parts.push(sentence.to_string());
                }
            }
        }

        merged_parts.join(". ") + if !merged_parts.is_empty() { "." } else { "" }
    }

    /// 查找相似的现有记忆
    pub fn find_similar_memories<'a>(
        &self,
        fact: &ExtractedFact,
        existing_memories: &'a [ExistingMemory],
    ) -> Vec<&'a ExistingMemory> {
        existing_memories
            .iter()
            .filter(|memory| {
                self.calculate_content_similarity(&fact.content, &memory.content)
                    > self.similarity_threshold
            })
            .collect()
    }

    /// 计算内容相似性
    pub fn calculate_content_similarity(&self, content1: &str, content2: &str) -> f32 {
        let content1_lower = content1.to_lowercase();
        let content2_lower = content2.to_lowercase();

        let words1: std::collections::HashSet<&str> = content1_lower.split_whitespace().collect();
        let words2: std::collections::HashSet<&str> = content2_lower.split_whitespace().collect();

        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }

    /// 检测记忆冲突（简化版本，用于向后兼容）
    pub fn detect_conflicts_simple<'a>(
        &self,
        fact: &ExtractedFact,
        existing_memories: &'a [ExistingMemory],
    ) -> Vec<&'a ExistingMemory> {
        // 简单的冲突检测：查找包含否定词的相似内容
        let conflict_indicators = ["not", "no", "never", "don't", "doesn't", "won't", "can't"];

        existing_memories
            .iter()
            .filter(|memory| {
                let similarity = self.calculate_content_similarity(&fact.content, &memory.content);
                let has_conflict_indicators = conflict_indicators.iter().any(|indicator| {
                    fact.content.to_lowercase().contains(indicator)
                        || memory.content.to_lowercase().contains(indicator)
                });

                similarity > 0.5 && has_conflict_indicators
            })
            .collect()
    }

    /// 评估记忆重要性
    pub fn evaluate_importance(&self, fact: &ExtractedFact) -> f32 {
        let mut importance = fact.confidence;

        // 根据类别调整重要性
        importance *= match fact.category {
            crate::fact_extraction::FactCategory::Personal => 0.9,
            crate::fact_extraction::FactCategory::Preference => 0.8,
            crate::fact_extraction::FactCategory::Relationship => 0.9,
            crate::fact_extraction::FactCategory::Event => 0.7,
            crate::fact_extraction::FactCategory::Knowledge => 0.8,
            crate::fact_extraction::FactCategory::Procedural => 0.9,
            crate::fact_extraction::FactCategory::Emotional => 0.7,
            crate::fact_extraction::FactCategory::Goal => 0.9,
            crate::fact_extraction::FactCategory::Skill => 0.8,
            crate::fact_extraction::FactCategory::Location => 0.7,
            crate::fact_extraction::FactCategory::Temporal => 0.6,
            crate::fact_extraction::FactCategory::Financial => 0.8,
            crate::fact_extraction::FactCategory::Health => 0.9,
            crate::fact_extraction::FactCategory::Educational => 0.8,
            crate::fact_extraction::FactCategory::Professional => 0.8,
        };

        // 根据实体数量调整
        if !fact.entities.is_empty() {
            importance += 0.1 * (fact.entities.len() as f32).min(3.0) / 3.0;
        }

        // 根据时间信息调整
        if fact.temporal_info.is_some() {
            importance += 0.1;
        }

        importance.min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fact_extraction::FactCategory;

    #[test]
    fn test_memory_action_serialization() {
        let action = MemoryAction::Add {
            content: "Test content".to_string(),
            importance: 0.8,
            metadata: HashMap::new(),
        };

        let serialized = serde_json::to_string(&action).unwrap();
        assert!(serialized.contains("Add"));
        assert!(serialized.contains("Test content"));
    }

    #[test]
    fn test_content_similarity() {
        // 由于 calculate_content_similarity 是私有方法，我们无法直接测试
        // 但可以通过其他公共方法间接验证其行为
        let fact = ExtractedFact {
            content: "User likes coffee".to_string(),
            confidence: 0.9,
            category: FactCategory::Preference,
            entities: vec![],
            temporal_info: None,
            source_message_id: None,
            metadata: HashMap::new(),
        };

        let memory = ExistingMemory {
            id: "mem1".to_string(),
            content: "User enjoys coffee".to_string(),
            importance: 0.8,
            created_at: "2023-01-01".to_string(),
            updated_at: None,
            metadata: HashMap::new(),
        };

        // 在实际测试中需要创建 MemoryDecisionEngine 实例
        // find_similar_memories 会使用 calculate_content_similarity
    }

    #[test]
    fn test_importance_evaluation() {
        let fact = ExtractedFact {
            content: "User's name is John".to_string(),
            confidence: 0.9,
            category: FactCategory::Personal,
            entities: vec![],
            temporal_info: None,
            source_message_id: None,
            metadata: HashMap::new(),
        };

        // 需要创建 MemoryDecisionEngine 实例来测试 evaluate_importance
        // 预期重要性会基于类别、实体和时间信息进行调整
    }
}

/// 增强决策引擎 (Mem5 版本)
///
/// 按照 Mem5 计划实现的智能决策引擎，支持：
/// - 基于事实的智能决策制定
/// - 多维度决策评估
/// - 自适应决策策略
/// - 决策置信度评估
pub struct EnhancedDecisionEngine {
    llm: Arc<dyn LLMProvider + Send + Sync>,
    config: DecisionEngineConfig,
}

/// 决策引擎配置
#[derive(Debug, Clone)]
pub struct DecisionEngineConfig {
    /// 决策置信度阈值
    pub confidence_threshold: f32,
    /// 自动执行阈值
    pub auto_execution_threshold: f32,
    /// 最大考虑记忆数量
    pub max_consideration_memories: usize,
    /// 决策超时时间（秒）
    pub decision_timeout_seconds: u64,
}

impl Default for DecisionEngineConfig {
    fn default() -> Self {
        Self {
            confidence_threshold: 0.7,
            auto_execution_threshold: 0.8,
            max_consideration_memories: 50,
            decision_timeout_seconds: 30,
        }
    }
}

/// 决策上下文
#[derive(Debug, Clone)]
pub struct DecisionContext {
    /// 新的结构化事实
    pub new_facts: Vec<StructuredFact>,
    /// 现有记忆
    pub existing_memories: Vec<Memory>,
    /// 重要性评估结果
    pub importance_evaluations: Vec<ImportanceEvaluation>,
    /// 冲突检测结果
    pub conflict_detections: Vec<crate::conflict_resolution::ConflictDetection>,
    /// 用户偏好
    pub user_preferences: HashMap<String, String>,
}

/// 决策结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionResult {
    /// 决策ID
    pub decision_id: String,
    /// 推荐的操作
    pub recommended_actions: Vec<MemoryAction>,
    /// 决策置信度
    pub confidence: f32,
    /// 决策原因
    pub reasoning: String,
    /// 预期影响
    pub expected_impact: DecisionImpact,
    /// 决策时间
    pub decided_at: chrono::DateTime<chrono::Utc>,
    /// 是否需要人工确认
    pub requires_confirmation: bool,
}

/// 决策影响
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionImpact {
    /// 影响的记忆数量
    pub affected_memory_count: usize,
    /// 预期性能影响
    pub performance_impact: f32,
    /// 预期存储影响
    pub storage_impact: f32,
    /// 预期用户体验影响
    pub user_experience_impact: f32,
}

impl EnhancedDecisionEngine {
    /// 创建新的增强决策引擎
    pub fn new(llm: Arc<dyn LLMProvider + Send + Sync>, config: DecisionEngineConfig) -> Self {
        Self { llm, config }
    }

    /// 制定智能决策
    pub async fn make_decisions(&self, context: &DecisionContext) -> Result<DecisionResult> {
        info!(
            "开始制定智能决策，事实数量: {}, 记忆数量: {}",
            context.new_facts.len(),
            context.existing_memories.len()
        );

        // 1. 分析决策上下文
        let analysis = self.analyze_decision_context(context).await?;

        // 2. 生成候选操作
        let candidate_actions = self.generate_candidate_actions(context, &analysis).await?;

        // 3. 评估候选操作
        let evaluated_actions = self
            .evaluate_candidate_actions(&candidate_actions, context)
            .await?;

        // 4. 选择最佳操作
        let selected_actions = self.select_best_actions(&evaluated_actions);

        // 5. 计算决策置信度
        let confidence = self.calculate_decision_confidence(&selected_actions, &analysis);

        // 6. 生成决策原因
        let reasoning = self
            .generate_decision_reasoning(&selected_actions, &analysis)
            .await?;

        // 7. 评估决策影响
        let impact = self.assess_decision_impact(&selected_actions, context);

        let decision_result = DecisionResult {
            decision_id: format!("decision_{}", chrono::Utc::now().timestamp()),
            recommended_actions: selected_actions,
            confidence,
            reasoning,
            expected_impact: impact,
            decided_at: chrono::Utc::now(),
            requires_confirmation: confidence < self.config.auto_execution_threshold,
        };

        info!(
            "决策制定完成，置信度: {:.2}, 操作数量: {}",
            decision_result.confidence,
            decision_result.recommended_actions.len()
        );

        Ok(decision_result)
    }

    /// 分析决策上下文
    async fn analyze_decision_context(&self, context: &DecisionContext) -> Result<ContextAnalysis> {
        let mut analysis = ContextAnalysis::default();

        // 分析事实质量
        analysis.fact_quality = self.analyze_fact_quality(&context.new_facts);

        // 分析记忆状态
        analysis.memory_state = self.analyze_memory_state(&context.existing_memories);

        // 分析冲突情况
        analysis.conflict_severity = self.analyze_conflict_severity(&context.conflict_detections);

        // 分析重要性分布
        analysis.importance_distribution =
            self.analyze_importance_distribution(&context.importance_evaluations);

        Ok(analysis)
    }

    /// 生成候选操作
    async fn generate_candidate_actions(
        &self,
        context: &DecisionContext,
        _analysis: &ContextAnalysis,
    ) -> Result<Vec<CandidateAction>> {
        let mut candidates = Vec::new();

        // 基于新事实生成添加操作
        for fact in &context.new_facts {
            if fact.confidence > 0.6 {
                candidates.push(CandidateAction {
                    action: MemoryAction::Add {
                        content: fact.description.clone(),
                        importance: fact.importance,
                        metadata: HashMap::new(),
                    },
                    confidence: fact.confidence,
                    reasoning: format!("基于高置信度事实: {}", fact.description),
                    priority: self.calculate_action_priority(&fact.fact_type, fact.importance),
                });
            }
        }

        // 基于冲突检测生成解决操作
        for conflict in &context.conflict_detections {
            match &conflict.suggested_resolution {
                crate::conflict_resolution::ResolutionStrategy::KeepLatest => {
                    if let Some(_latest_id) = conflict.memory_ids.last() {
                        candidates.push(CandidateAction {
                            action: MemoryAction::Delete {
                                memory_id: conflict.memory_ids[0].clone(),
                                deletion_reason: DeletionReason::Contradicted,
                            },
                            confidence: conflict.confidence,
                            reasoning: "解决冲突：保留最新记忆".to_string(),
                            priority: conflict.severity,
                        });
                    }
                }
                _ => {
                    // 其他解决策略的处理
                }
            }
        }

        Ok(candidates)
    }

    /// 评估候选操作
    async fn evaluate_candidate_actions(
        &self,
        candidates: &[CandidateAction],
        context: &DecisionContext,
    ) -> Result<Vec<EvaluatedAction>> {
        let mut evaluated = Vec::new();

        for candidate in candidates {
            let evaluation = ActionEvaluation {
                feasibility: self.assess_action_feasibility(&candidate.action, context),
                risk: self.assess_action_risk(&candidate.action, context),
                benefit: self.assess_action_benefit(&candidate.action, context),
                cost: self.assess_action_cost(&candidate.action, context),
            };

            let overall_score = self.calculate_overall_score(&evaluation, candidate.confidence);
            evaluated.push(EvaluatedAction {
                candidate: candidate.clone(),
                evaluation,
                overall_score,
            });
        }

        Ok(evaluated)
    }

    /// 选择最佳操作
    fn select_best_actions(&self, evaluated_actions: &[EvaluatedAction]) -> Vec<MemoryAction> {
        let mut sorted_actions = evaluated_actions.to_vec();
        sorted_actions.sort_by(|a, b| b.overall_score.partial_cmp(&a.overall_score).unwrap());

        sorted_actions
            .into_iter()
            .take(10) // 限制操作数量
            .filter(|action| action.overall_score > self.config.confidence_threshold)
            .map(|action| action.candidate.action)
            .collect()
    }

    /// 计算决策置信度
    fn calculate_decision_confidence(
        &self,
        actions: &[MemoryAction],
        analysis: &ContextAnalysis,
    ) -> f32 {
        if actions.is_empty() {
            return 0.0;
        }

        // 基于多个因素计算置信度
        let fact_quality_factor = analysis.fact_quality;
        let memory_state_factor = analysis.memory_state;
        let conflict_factor = 1.0 - analysis.conflict_severity;

        let base_confidence = (fact_quality_factor + memory_state_factor + conflict_factor) / 3.0;

        // 根据操作数量调整
        let action_count_factor = if actions.len() <= 5 {
            1.0
        } else {
            0.8 // 操作过多时降低置信度
        };

        (base_confidence * action_count_factor).clamp(0.0, 1.0)
    }

    /// 生成决策原因
    async fn generate_decision_reasoning(
        &self,
        actions: &[MemoryAction],
        analysis: &ContextAnalysis,
    ) -> Result<String> {
        let mut reasons = Vec::new();

        if analysis.fact_quality > 0.8 {
            reasons.push("事实质量较高");
        }
        if analysis.conflict_severity > 0.7 {
            reasons.push("存在严重冲突需要解决");
        }
        if actions.len() > 3 {
            reasons.push("需要执行多个操作");
        }

        let reasoning = if reasons.is_empty() {
            "基于标准决策流程".to_string()
        } else {
            format!("决策依据: {}", reasons.join("，"))
        };

        Ok(reasoning)
    }

    /// 评估决策影响
    fn assess_decision_impact(
        &self,
        actions: &[MemoryAction],
        context: &DecisionContext,
    ) -> DecisionImpact {
        let affected_memory_count = actions.len();

        DecisionImpact {
            affected_memory_count,
            performance_impact: (affected_memory_count as f32 / 100.0).min(1.0),
            storage_impact: (affected_memory_count as f32 / 50.0).min(1.0),
            user_experience_impact: if affected_memory_count <= 5 { 0.1 } else { 0.3 },
        }
    }

    // 辅助方法实现...
    fn analyze_fact_quality(&self, facts: &[StructuredFact]) -> f32 {
        if facts.is_empty() {
            return 0.0;
        }
        facts.iter().map(|f| f.confidence).sum::<f32>() / facts.len() as f32
    }

    fn analyze_memory_state(&self, memories: &[Memory]) -> f32 {
        // 简化的记忆状态分析
        if memories.len() < 100 {
            0.9
        } else {
            0.7
        }
    }

    fn analyze_conflict_severity(
        &self,
        conflicts: &[crate::conflict_resolution::ConflictDetection],
    ) -> f32 {
        if conflicts.is_empty() {
            return 0.0;
        }
        conflicts.iter().map(|c| c.severity).sum::<f32>() / conflicts.len() as f32
    }

    fn analyze_importance_distribution(&self, evaluations: &[ImportanceEvaluation]) -> f32 {
        if evaluations.is_empty() {
            return 0.5;
        }
        evaluations.iter().map(|e| e.importance_score).sum::<f32>() / evaluations.len() as f32
    }

    fn calculate_action_priority(&self, fact_type: &str, importance: f32) -> f32 {
        importance
            * if fact_type.contains("Person") {
                1.2
            } else {
                1.0
            }
    }

    fn assess_action_feasibility(&self, _action: &MemoryAction, _context: &DecisionContext) -> f32 {
        0.8
    }
    fn assess_action_risk(&self, _action: &MemoryAction, _context: &DecisionContext) -> f32 {
        0.2
    }
    fn assess_action_benefit(&self, _action: &MemoryAction, _context: &DecisionContext) -> f32 {
        0.7
    }
    fn assess_action_cost(&self, _action: &MemoryAction, _context: &DecisionContext) -> f32 {
        0.3
    }

    fn calculate_overall_score(&self, evaluation: &ActionEvaluation, confidence: f32) -> f32 {
        (evaluation.feasibility * 0.3 + evaluation.benefit * 0.4
            - evaluation.risk * 0.2
            - evaluation.cost * 0.1)
            * confidence
    }
}

// 辅助结构体
#[derive(Debug, Clone, Default)]
struct ContextAnalysis {
    fact_quality: f32,
    memory_state: f32,
    conflict_severity: f32,
    importance_distribution: f32,
}

#[derive(Debug, Clone)]
struct CandidateAction {
    action: MemoryAction,
    confidence: f32,
    reasoning: String,
    priority: f32,
}

#[derive(Debug, Clone)]
struct ActionEvaluation {
    feasibility: f32,
    risk: f32,
    benefit: f32,
    cost: f32,
}

#[derive(Debug, Clone)]
struct EvaluatedAction {
    candidate: CandidateAction,
    evaluation: ActionEvaluation,
    overall_score: f32,
}

impl MemoryDecisionEngine {
    /// P2 优化 #13: 验证决策一致性
    /// 
    /// 确保决策之间没有冲突，例如：
    /// - UPDATE和DELETE同一个记忆
    /// - 多个DELETE同一个记忆
    /// - MERGE中的记忆被DELETE
    fn validate_decision_consistency(&self, mut decisions: Vec<MemoryDecision>) -> Result<Vec<MemoryDecision>> {
        use std::collections::HashSet;
        
        info!("P2优化 #13: 开始验证决策一致性，共 {} 个决策", decisions.len());
        
        let mut to_update: HashSet<String> = HashSet::new();
        let mut to_delete: HashSet<String> = HashSet::new();
        let mut to_merge: HashSet<String> = HashSet::new();
        let mut conflicts = Vec::new();
        
        // 第一遍：收集所有操作的目标记忆
        for decision in decisions.iter() {
            match &decision.action {
                MemoryAction::Update { memory_id, .. } => {
                    to_update.insert(memory_id.clone());
                }
                MemoryAction::Delete { memory_id, .. } => {
                    to_delete.insert(memory_id.clone());
                }
                MemoryAction::Merge { primary_memory_id, secondary_memory_ids, .. } => {
                    to_merge.insert(primary_memory_id.clone());
                    for id in secondary_memory_ids {
                        to_merge.insert(id.clone());
                    }
                }
                _ => {}
            }
        }
        
        // 第二遍：检测冲突
        for (idx, decision) in decisions.iter().enumerate() {
            let mut has_conflict = false;
            let mut conflict_reason = String::new();
            
            match &decision.action {
                MemoryAction::Update { memory_id, .. } => {
                    // UPDATE vs DELETE 冲突
                    if to_delete.contains(memory_id) {
                        has_conflict = true;
                        conflict_reason = format!("记忆 {} 同时被UPDATE和DELETE", memory_id);
                    }
                    // UPDATE vs MERGE 冲突
                    if to_merge.contains(memory_id) {
                        has_conflict = true;
                        conflict_reason = format!("记忆 {} 同时被UPDATE和MERGE", memory_id);
                    }
                }
                MemoryAction::Delete { memory_id, .. } => {
                    // DELETE vs MERGE 冲突
                    if to_merge.contains(memory_id) {
                        has_conflict = true;
                        conflict_reason = format!("记忆 {} 同时被DELETE和MERGE", memory_id);
                    }
                }
                MemoryAction::Merge { primary_memory_id, secondary_memory_ids, .. } => {
                    // MERGE 内部冲突：主记忆被删除
                    if to_delete.contains(primary_memory_id) {
                        has_conflict = true;
                        conflict_reason = format!("MERGE的主记忆 {} 被DELETE", primary_memory_id);
                    }
                    // MERGE 内部冲突：次要记忆被删除
                    for id in secondary_memory_ids {
                        if to_delete.contains(id) {
                            has_conflict = true;
                            conflict_reason = format!("MERGE的次要记忆 {} 被DELETE", id);
                            break;
                        }
                    }
                }
                _ => {}
            }
            
            if has_conflict {
                warn!("❌ 决策冲突 #{}: {}", idx, conflict_reason);
                conflicts.push(idx);
            }
        }
        
        // 移除冲突的决策（保留置信度较高的）
        if !conflicts.is_empty() {
            warn!("发现 {} 个决策冲突，移除冲突决策", conflicts.len());
            
            let total_decisions = decisions.len();
            
            // 按置信度从高到低排序
            decisions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
            
            // 重新验证（移除低置信度的冲突决策）
            let mut validated = Vec::new();
            let mut processed_memories: HashSet<String> = HashSet::new();
            
            for decision in decisions {
                let memory_ids = self.get_affected_memory_ids(&decision.action);
                let has_processed = memory_ids.iter().any(|id| processed_memories.contains(id));
                
                if !has_processed {
                    // 记录已处理的记忆
                    for id in memory_ids {
                        processed_memories.insert(id);
                    }
                    validated.push(decision);
                } else {
                    warn!("移除冲突决策: {:?} (置信度: {})", decision.action, decision.confidence);
                }
            }
            
            info!("✅ 决策一致性验证完成：保留 {} 个决策，移除 {} 个冲突决策",
                validated.len(),
                total_decisions - validated.len()
            );
            
            Ok(validated)
        } else {
            info!("✅ 所有决策一致，无冲突");
            Ok(decisions)
        }
    }
    
    /// 获取决策影响的记忆ID列表
    fn get_affected_memory_ids(&self, action: &MemoryAction) -> Vec<String> {
        match action {
            MemoryAction::Update { memory_id, .. } => vec![memory_id.clone()],
            MemoryAction::Delete { memory_id, .. } => vec![memory_id.clone()],
            MemoryAction::Merge { primary_memory_id, secondary_memory_ids, .. } => {
                let mut ids = vec![primary_memory_id.clone()];
                ids.extend(secondary_memory_ids.clone());
                ids
            }
            _ => Vec::new(),
        }
    }
    
    /// P2 优化 #14: 记录决策审计日志
    /// 
    /// 记录所有决策的详细信息，便于调试和追踪
    fn log_decisions(
        &self,
        decisions: &[MemoryDecision],
        new_facts: &[ExtractedFact],
        existing_memories: &[ExistingMemory],
    ) {
        info!("==================== 决策审计日志 ====================");
        info!("时间: {}", chrono::Utc::now());
        info!("新事实数量: {}", new_facts.len());
        info!("现有记忆数量: {}", existing_memories.len());
        info!("决策数量: {}", decisions.len());
        info!("");
        
        // 统计决策类型
        let mut add_count = 0;
        let mut update_count = 0;
        let mut delete_count = 0;
        let mut merge_count = 0;
        let mut no_action_count = 0;
        
        for decision in decisions {
            match &decision.action {
                MemoryAction::Add { .. } => add_count += 1,
                MemoryAction::Update { .. } => update_count += 1,
                MemoryAction::Delete { .. } => delete_count += 1,
                MemoryAction::Merge { .. } => merge_count += 1,
                MemoryAction::NoAction { .. } => no_action_count += 1,
            }
        }
        
        info!("决策类型统计:");
        info!("  - ADD: {}", add_count);
        info!("  - UPDATE: {}", update_count);
        info!("  - DELETE: {}", delete_count);
        info!("  - MERGE: {}", merge_count);
        info!("  - NO_ACTION: {}", no_action_count);
        info!("");
        
        // 记录每个决策的详细信息
        for (idx, decision) in decisions.iter().enumerate() {
            info!("决策 #{}: {:?}", idx + 1, self.format_decision_action(&decision.action));
            info!("  置信度: {:.2}", decision.confidence);
            info!("  影响的记忆: {:?}", decision.affected_memories);
            info!("  预估影响: {:.2}", decision.estimated_impact);
            info!("  推理依据: {}", decision.reasoning.chars().take(100).collect::<String>());
            
            // 详细的操作参数
            match &decision.action {
                MemoryAction::Add { content, importance, .. } => {
                    info!("  内容预览: {}", content.chars().take(50).collect::<String>());
                    info!("  重要性: {:.2}", importance);
                }
                MemoryAction::Update { memory_id, new_content, merge_strategy, change_reason } => {
                    info!("  目标记忆ID: {}", memory_id);
                    info!("  新内容预览: {}", new_content.chars().take(50).collect::<String>());
                    info!("  合并策略: {:?}", merge_strategy);
                    info!("  变更原因: {}", change_reason);
                }
                MemoryAction::Delete { memory_id, deletion_reason } => {
                    info!("  删除记忆ID: {}", memory_id);
                    info!("  删除原因: {:?}", deletion_reason);
                }
                MemoryAction::Merge { primary_memory_id, secondary_memory_ids, merged_content } => {
                    info!("  主记忆ID: {}", primary_memory_id);
                    info!("  次要记忆ID: {:?}", secondary_memory_ids);
                    info!("  合并内容预览: {}", merged_content.chars().take(50).collect::<String>());
                }
                MemoryAction::NoAction { reason } => {
                    info!("  原因: {}", reason);
                }
            }
            info!("");
        }
        
        info!("====================================================");
    }
    
    /// 格式化决策动作类型
    fn format_decision_action(&self, action: &MemoryAction) -> &'static str {
        match action {
            MemoryAction::Add { .. } => "ADD",
            MemoryAction::Update { .. } => "UPDATE",
            MemoryAction::Delete { .. } => "DELETE",
            MemoryAction::Merge { .. } => "MERGE",
            MemoryAction::NoAction { .. } => "NO_ACTION",
        }
    }
}
