//! ContextSynthesizer - 上下文合成器
//!
//! 参考 MIRIX 的多源记忆融合机制，实现智能上下文合成和冲突解决

use crate::retrieval::{RetrievalRequest, RetrievedMemory};
use agent_mem_traits::{AgentMemError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 冲突解决策略
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConflictResolution {
    /// 保留最新的记忆
    KeepLatest,
    /// 保留最相关的记忆
    KeepMostRelevant,
    /// 保留置信度最高的记忆
    KeepHighestConfidence,
    /// 合并冲突的记忆
    Merge,
    /// 标记冲突但保留所有记忆
    MarkConflict,
    /// 用户手动解决
    ManualResolution,
}

impl ConflictResolution {
    /// 获取策略描述
    pub fn description(&self) -> &'static str {
        match self {
            ConflictResolution::KeepLatest => "保留时间最新的记忆项",
            ConflictResolution::KeepMostRelevant => "保留相关性分数最高的记忆项",
            ConflictResolution::KeepHighestConfidence => "保留置信度最高的记忆项",
            ConflictResolution::Merge => "智能合并冲突的记忆项",
            ConflictResolution::MarkConflict => "标记冲突但保留所有记忆项",
            ConflictResolution::ManualResolution => "标记为需要用户手动解决",
        }
    }
}

/// 合成结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesisResult {
    /// 合成的记忆项
    pub synthesized_memories: Vec<SynthesizedMemory>,
    /// 检测到的冲突
    pub detected_conflicts: Vec<MemoryConflict>,
    /// 合成摘要
    pub synthesis_summary: String,
    /// 置信度分数
    pub confidence_score: f32,
    /// 相关性排序
    pub relevance_ranking: Vec<String>, // 记忆ID列表，按相关性排序
    /// 合成时间（毫秒）
    pub synthesis_time_ms: u64,
    /// 使用的合成策略
    pub synthesis_strategies: Vec<SynthesisStrategy>,
}

/// 合成的记忆项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesizedMemory {
    /// 合成记忆ID
    pub id: String,
    /// 原始记忆ID列表
    pub source_memory_ids: Vec<String>,
    /// 合成内容
    pub synthesized_content: String,
    /// 合成置信度
    pub synthesis_confidence: f32,
    /// 合成类型
    pub synthesis_type: SynthesisType,
    /// 关键信息提取
    pub key_insights: Vec<String>,
    /// 元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 合成类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SynthesisType {
    /// 信息聚合
    Aggregation,
    /// 时序整合
    TemporalIntegration,
    /// 主题融合
    TopicFusion,
    /// 上下文增强
    ContextEnhancement,
    /// 冲突解决
    ConflictResolution,
}

/// 记忆冲突
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConflict {
    /// 冲突ID
    pub conflict_id: String,
    /// 冲突的记忆ID列表
    pub conflicting_memory_ids: Vec<String>,
    /// 冲突类型
    pub conflict_type: ConflictType,
    /// 冲突描述
    pub description: String,
    /// 冲突严重程度 (0.0-1.0)
    pub severity: f32,
    /// 建议的解决策略
    pub suggested_resolution: ConflictResolution,
    /// 解决状态
    pub resolution_status: ResolutionStatus,
}

/// 冲突类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConflictType {
    /// 内容矛盾
    ContentContradiction,
    /// 时间冲突
    TemporalConflict,
    /// 事实不一致
    FactualInconsistency,
    /// 重复信息
    DuplicateInformation,
    /// 版本冲突
    VersionConflict,
}

/// 解决状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionStatus {
    /// 未解决
    Unresolved,
    /// 自动解决
    AutoResolved,
    /// 等待用户解决
    PendingUserResolution,
    /// 已解决
    Resolved,
}

/// 合成策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SynthesisStrategy {
    /// 基于相关性的合成
    RelevanceBased,
    /// 基于时间的合成
    TimeBased,
    /// 基于主题的合成
    TopicBased,
    /// 基于上下文的合成
    ContextBased,
    /// 智能摘要合成
    IntelligentSummarization,
}

/// 上下文合成器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSynthesizerConfig {
    /// 最大合成记忆数量
    pub max_synthesis_memories: usize,
    /// 冲突检测阈值
    pub conflict_detection_threshold: f32,
    /// 默认冲突解决策略
    pub default_conflict_resolution: ConflictResolution,
    /// 是否启用智能摘要
    pub enable_intelligent_summarization: bool,
    /// 是否启用冲突检测
    pub enable_conflict_detection: bool,
    /// 相关性权重
    pub relevance_weight: f32,
    /// 时间权重
    pub temporal_weight: f32,
    /// 主题权重
    pub topic_weight: f32,
    /// 最小置信度阈值
    pub min_confidence_threshold: f32,
    /// 合成超时时间（秒）
    pub synthesis_timeout_seconds: u64,
}

impl Default for ContextSynthesizerConfig {
    fn default() -> Self {
        Self {
            max_synthesis_memories: 5,
            conflict_detection_threshold: 0.7,
            default_conflict_resolution: ConflictResolution::KeepMostRelevant,
            enable_intelligent_summarization: true,
            enable_conflict_detection: true,
            relevance_weight: 0.4,
            temporal_weight: 0.3,
            topic_weight: 0.3,
            min_confidence_threshold: 0.5,
            synthesis_timeout_seconds: 30,
        }
    }
}

/// 合成器统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesizerStats {
    /// 总合成次数
    pub total_syntheses: u64,
    /// 成功合成次数
    pub successful_syntheses: u64,
    /// 平均合成时间（毫秒）
    pub avg_synthesis_time_ms: f64,
    /// 检测到的冲突总数
    pub total_conflicts_detected: u64,
    /// 自动解决的冲突数
    pub auto_resolved_conflicts: u64,
    /// 按类型统计的冲突
    pub conflicts_by_type: HashMap<ConflictType, u64>,
    /// 平均合成置信度
    pub avg_synthesis_confidence: f32,
}

/// 上下文合成器
///
/// 多源记忆融合和智能上下文合成系统
pub struct ContextSynthesizer {
    /// 配置
    config: ContextSynthesizerConfig,
    /// 统计信息
    stats: Arc<RwLock<SynthesizerStats>>,
    /// 冲突解决历史
    conflict_history: Arc<RwLock<HashMap<String, MemoryConflict>>>,
    /// 合成缓存
    synthesis_cache: Arc<RwLock<HashMap<String, SynthesisResult>>>,
}

impl ContextSynthesizer {
    /// 创建新的上下文合成器
    pub async fn new(config: ContextSynthesizerConfig) -> Result<Self> {
        let stats = SynthesizerStats {
            total_syntheses: 0,
            successful_syntheses: 0,
            avg_synthesis_time_ms: 0.0,
            total_conflicts_detected: 0,
            auto_resolved_conflicts: 0,
            conflicts_by_type: HashMap::new(),
            avg_synthesis_confidence: 0.0,
        };

        Ok(Self {
            config,
            stats: Arc::new(RwLock::new(stats)),
            conflict_history: Arc::new(RwLock::new(HashMap::new())),
            synthesis_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// 合成上下文
    pub async fn synthesize_context(
        &self,
        memories: &[RetrievedMemory],
        request: &RetrievalRequest,
    ) -> Result<SynthesisResult> {
        let start_time = std::time::Instant::now();

        // 更新统计信息
        {
            let mut stats = self.stats.write().await;
            stats.total_syntheses += 1;
        }

        // 检查缓存
        let cache_key = self.generate_synthesis_cache_key(memories, request);
        if let Some(cached_result) = self.check_synthesis_cache(&cache_key).await {
            return Ok(cached_result);
        }

        // 1. 冲突检测
        let detected_conflicts = if self.config.enable_conflict_detection {
            self.detect_conflicts(memories).await?
        } else {
            Vec::new()
        };

        // 2. 相关性排序
        let relevance_ranking = self.rank_by_relevance(memories, request).await?;

        // 3. 执行合成
        let synthesized_memories = self
            .perform_synthesis(memories, &detected_conflicts)
            .await?;

        // 4. 生成摘要
        let synthesis_summary = if self.config.enable_intelligent_summarization {
            self.generate_intelligent_summary(&synthesized_memories, request)
                .await?
        } else {
            self.generate_simple_summary(&synthesized_memories)
        };

        // 5. 计算置信度
        let confidence_score =
            self.calculate_synthesis_confidence(&synthesized_memories, &detected_conflicts);

        let synthesis_time_ms = start_time.elapsed().as_millis() as u64;

        let result = SynthesisResult {
            synthesized_memories,
            detected_conflicts,
            synthesis_summary,
            confidence_score,
            relevance_ranking,
            synthesis_time_ms,
            synthesis_strategies: vec![
                SynthesisStrategy::RelevanceBased,
                SynthesisStrategy::ContextBased,
            ],
        };

        // 缓存结果
        self.cache_synthesis_result(&cache_key, &result).await;

        // 更新统计信息
        self.update_synthesis_stats(&result).await;

        Ok(result)
    }

    /// 检测记忆冲突
    async fn detect_conflicts(&self, memories: &[RetrievedMemory]) -> Result<Vec<MemoryConflict>> {
        let mut conflicts = Vec::new();

        // 简化的冲突检测逻辑
        for (i, memory1) in memories.iter().enumerate() {
            for (j, memory2) in memories.iter().enumerate().skip(i + 1) {
                if let Some(conflict) = self.check_memory_pair_conflict(memory1, memory2).await? {
                    conflicts.push(conflict);
                }
            }
        }

        Ok(conflicts)
    }

    /// 检查两个记忆项之间的冲突
    async fn check_memory_pair_conflict(
        &self,
        memory1: &RetrievedMemory,
        memory2: &RetrievedMemory,
    ) -> Result<Option<MemoryConflict>> {
        // 简化的冲突检测
        let content_similarity =
            self.calculate_content_similarity(&memory1.content, &memory2.content);

        if content_similarity > self.config.conflict_detection_threshold {
            let conflict = MemoryConflict {
                conflict_id: format!("conflict_{}_{}", memory1.id, memory2.id),
                conflicting_memory_ids: vec![memory1.id.clone(), memory2.id.clone()],
                conflict_type: ConflictType::DuplicateInformation,
                description: "检测到重复或相似的记忆内容".to_string(),
                severity: content_similarity,
                suggested_resolution: self.config.default_conflict_resolution.clone(),
                resolution_status: ResolutionStatus::Unresolved,
            };
            return Ok(Some(conflict));
        }

        Ok(None)
    }

    /// 计算内容相似度
    fn calculate_content_similarity(&self, content1: &str, content2: &str) -> f32 {
        // 简化的相似度计算
        let words1: std::collections::HashSet<&str> = content1.split_whitespace().collect();
        let words2: std::collections::HashSet<&str> = content2.split_whitespace().collect();

        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }

    /// 按相关性排序
    async fn rank_by_relevance(
        &self,
        memories: &[RetrievedMemory],
        _request: &RetrievalRequest,
    ) -> Result<Vec<String>> {
        let mut ranked_memories: Vec<_> = memories.iter().collect();
        ranked_memories.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(ranked_memories.into_iter().map(|m| m.id.clone()).collect())
    }

    /// 执行合成
    async fn perform_synthesis(
        &self,
        memories: &[RetrievedMemory],
        conflicts: &[MemoryConflict],
    ) -> Result<Vec<SynthesizedMemory>> {
        let mut synthesized = Vec::new();

        // 简化的合成逻辑
        if memories.len() <= self.config.max_synthesis_memories {
            // 如果记忆数量不多，直接聚合
            let aggregated = self.aggregate_memories(memories).await?;
            synthesized.push(aggregated);
        } else {
            // 如果记忆数量较多，按主题分组合成
            let grouped_memories = self.group_memories_by_topic(memories).await?;
            for (topic, group_memories) in grouped_memories {
                let topic_synthesis = self.synthesize_topic_group(&topic, &group_memories).await?;
                synthesized.push(topic_synthesis);
            }
        }

        // 处理冲突
        for conflict in conflicts {
            if let Some(resolved_memory) = self.resolve_conflict(conflict, memories).await? {
                synthesized.push(resolved_memory);
            }
        }

        Ok(synthesized)
    }

    /// 聚合记忆
    async fn aggregate_memories(&self, memories: &[RetrievedMemory]) -> Result<SynthesizedMemory> {
        let combined_content = memories
            .iter()
            .map(|m| m.content.as_str())
            .collect::<Vec<_>>()
            .join("\n\n");

        let avg_confidence =
            memories.iter().map(|m| m.relevance_score).sum::<f32>() / memories.len() as f32;

        Ok(SynthesizedMemory {
            id: format!("synthesized_{}", uuid::Uuid::new_v4()),
            source_memory_ids: memories.iter().map(|m| m.id.clone()).collect(),
            synthesized_content: combined_content,
            synthesis_confidence: avg_confidence,
            synthesis_type: SynthesisType::Aggregation,
            key_insights: self.extract_key_insights(memories).await?,
            metadata: HashMap::new(),
        })
    }

    /// 按主题分组记忆
    async fn group_memories_by_topic(
        &self,
        memories: &[RetrievedMemory],
    ) -> Result<HashMap<String, Vec<RetrievedMemory>>> {
        let mut groups = HashMap::new();

        // 简化的主题分组
        for memory in memories {
            let topic = memory.memory_type.to_string(); // 使用记忆类型作为主题
            groups
                .entry(topic)
                .or_insert_with(Vec::new)
                .push(memory.clone());
        }

        Ok(groups)
    }

    /// 合成主题组
    async fn synthesize_topic_group(
        &self,
        topic: &str,
        memories: &[RetrievedMemory],
    ) -> Result<SynthesizedMemory> {
        let combined_content = format!(
            "主题: {}\n\n{}",
            topic,
            memories
                .iter()
                .map(|m| m.content.as_str())
                .collect::<Vec<_>>()
                .join("\n\n")
        );

        let avg_confidence =
            memories.iter().map(|m| m.relevance_score).sum::<f32>() / memories.len() as f32;

        Ok(SynthesizedMemory {
            id: format!("topic_synthesis_{}", uuid::Uuid::new_v4()),
            source_memory_ids: memories.iter().map(|m| m.id.clone()).collect(),
            synthesized_content: combined_content,
            synthesis_confidence: avg_confidence,
            synthesis_type: SynthesisType::TopicFusion,
            key_insights: self.extract_key_insights(memories).await?,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert(
                    "topic".to_string(),
                    serde_json::Value::String(topic.to_string()),
                );
                meta
            },
        })
    }

    /// 解决冲突
    async fn resolve_conflict(
        &self,
        conflict: &MemoryConflict,
        memories: &[RetrievedMemory],
    ) -> Result<Option<SynthesizedMemory>> {
        match conflict.suggested_resolution {
            ConflictResolution::KeepMostRelevant => {
                // 找到最相关的记忆
                let conflicting_memories: Vec<_> = memories
                    .iter()
                    .filter(|m| conflict.conflicting_memory_ids.contains(&m.id))
                    .collect();

                if let Some(most_relevant) = conflicting_memories.iter().max_by(|a, b| {
                    a.relevance_score
                        .partial_cmp(&b.relevance_score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }) {
                    return Ok(Some(SynthesizedMemory {
                        id: format!("conflict_resolved_{}", conflict.conflict_id),
                        source_memory_ids: vec![most_relevant.id.clone()],
                        synthesized_content: most_relevant.content.clone(),
                        synthesis_confidence: most_relevant.relevance_score,
                        synthesis_type: SynthesisType::ConflictResolution,
                        key_insights: vec!["冲突已通过保留最相关记忆解决".to_string()],
                        metadata: HashMap::new(),
                    }));
                }
            }
            ConflictResolution::Merge => {
                // 合并冲突的记忆
                let conflicting_memories: Vec<_> = memories
                    .iter()
                    .filter(|m| conflict.conflicting_memory_ids.contains(&m.id))
                    .collect();

                if !conflicting_memories.is_empty() {
                    return Ok(Some(
                        self.aggregate_memories(
                            &conflicting_memories
                                .into_iter()
                                .cloned()
                                .collect::<Vec<_>>(),
                        )
                        .await?,
                    ));
                }
            }
            _ => {
                // 其他解决策略暂不实现
            }
        }

        Ok(None)
    }

    /// 提取关键洞察
    async fn extract_key_insights(&self, memories: &[RetrievedMemory]) -> Result<Vec<String>> {
        let mut insights = Vec::new();

        // 简化的洞察提取
        insights.push(format!("合成了 {} 个记忆项", memories.len()));

        let memory_types: std::collections::HashSet<_> =
            memories.iter().map(|m| &m.memory_type).collect();
        insights.push(format!("涉及 {} 种记忆类型", memory_types.len()));

        let avg_relevance =
            memories.iter().map(|m| m.relevance_score).sum::<f32>() / memories.len() as f32;
        insights.push(format!("平均相关性分数: {avg_relevance:.2}"));

        Ok(insights)
    }

    /// 生成智能摘要
    async fn generate_intelligent_summary(
        &self,
        synthesized_memories: &[SynthesizedMemory],
        _request: &RetrievalRequest,
    ) -> Result<String> {
        // 简化的智能摘要生成
        // 在实际实现中，这里会调用 LLM 生成摘要

        let total_memories = synthesized_memories.len();
        let avg_confidence = synthesized_memories
            .iter()
            .map(|m| m.synthesis_confidence)
            .sum::<f32>()
            / total_memories as f32;

        Ok(format!(
            "成功合成了 {total_memories} 个记忆项，平均置信度 {avg_confidence:.2}。合成结果包含了多个来源的信息，经过冲突检测和解决处理。"
        ))
    }

    /// 生成简单摘要
    fn generate_simple_summary(&self, synthesized_memories: &[SynthesizedMemory]) -> String {
        format!("合成了 {} 个记忆项", synthesized_memories.len())
    }

    /// 计算合成置信度
    fn calculate_synthesis_confidence(
        &self,
        synthesized_memories: &[SynthesizedMemory],
        conflicts: &[MemoryConflict],
    ) -> f32 {
        if synthesized_memories.is_empty() {
            return 0.0;
        }

        let avg_synthesis_confidence = synthesized_memories
            .iter()
            .map(|m| m.synthesis_confidence)
            .sum::<f32>()
            / synthesized_memories.len() as f32;

        // 冲突会降低置信度
        let conflict_penalty = conflicts.len() as f32 * 0.1;

        (avg_synthesis_confidence - conflict_penalty)
            .max(0.0)
            .min(1.0)
    }

    /// 生成合成缓存键
    fn generate_synthesis_cache_key(
        &self,
        memories: &[RetrievedMemory],
        request: &RetrievalRequest,
    ) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        for memory in memories {
            memory.id.hash(&mut hasher);
        }
        request.query.hash(&mut hasher);

        format!("synthesis_{}", hasher.finish())
    }

    /// 检查合成缓存
    async fn check_synthesis_cache(&self, cache_key: &str) -> Option<SynthesisResult> {
        let cache = self.synthesis_cache.read().await;
        cache.get(cache_key).cloned()
    }

    /// 缓存合成结果
    async fn cache_synthesis_result(&self, cache_key: &str, result: &SynthesisResult) {
        let mut cache = self.synthesis_cache.write().await;
        cache.insert(cache_key.to_string(), result.clone());
    }

    /// 更新合成统计信息
    async fn update_synthesis_stats(&self, result: &SynthesisResult) {
        let mut stats = self.stats.write().await;
        stats.successful_syntheses += 1;

        // 更新平均合成时间
        let total_time = stats.avg_synthesis_time_ms * (stats.successful_syntheses - 1) as f64
            + result.synthesis_time_ms as f64;
        stats.avg_synthesis_time_ms = total_time / stats.successful_syntheses as f64;

        // 更新冲突统计
        stats.total_conflicts_detected += result.detected_conflicts.len() as u64;
        for conflict in &result.detected_conflicts {
            *stats
                .conflicts_by_type
                .entry(conflict.conflict_type.clone())
                .or_insert(0) += 1;
            if matches!(conflict.resolution_status, ResolutionStatus::AutoResolved) {
                stats.auto_resolved_conflicts += 1;
            }
        }

        // 更新平均合成置信度
        let total_confidence = stats.avg_synthesis_confidence
            * (stats.successful_syntheses - 1) as f32
            + result.confidence_score;
        stats.avg_synthesis_confidence = total_confidence / stats.successful_syntheses as f32;
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> Result<serde_json::Value> {
        let stats = self.stats.read().await;
        serde_json::to_value(&*stats).map_err(|e| {
            AgentMemError::ProcessingError(format!("Failed to serialize stats: {e}"))
        })
    }
}
