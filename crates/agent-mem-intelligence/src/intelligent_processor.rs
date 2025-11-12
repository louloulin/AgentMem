//! 智能记忆处理器
//!
//! 整合事实提取和决策引擎，提供完整的智能记忆处理能力
//!
//! Mem5 增强功能：
//! - 完整的智能处理流水线
//! - 多组件协调工作
//! - 端到端处理能力
//! - 性能监控和优化

use crate::conflict_resolution::{ConflictDetection, ConflictResolver, ConflictResolverConfig};
use crate::decision_engine::{
    DecisionContext, DecisionResult, EnhancedDecisionEngine, ExistingMemory, MemoryDecision,
    MemoryDecisionEngine,
};
use crate::fact_extraction::{AdvancedFactExtractor, ExtractedFact, FactExtractor, StructuredFact};
use crate::importance_evaluator::{
    ImportanceEvaluation, ImportanceEvaluator, ImportanceEvaluatorConfig,
};
use agent_mem_traits::{MemoryV4 as Memory, LLMConfig, MemoryItem, MemoryType, Message, Result, Session};
use agent_mem_llm::{factory::RealLLMFactory, LLMProvider};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

// MockLLMProvider 已移除，使用真实的 LLM 提供商

/// 智能处理结果（增强版本）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligentProcessingResult {
    pub extracted_facts: Vec<ExtractedFact>,
    pub memory_decisions: Vec<MemoryDecision>,
    pub conflict_detections: Vec<ConflictDetection>,
    pub processing_stats: ProcessingStats,
    pub recommendations: Vec<String>,
    pub quality_metrics: QualityMetrics,
    pub processing_insights: ProcessingInsights,
}

/// 质量指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub average_fact_confidence: f32,
    pub average_decision_confidence: f32,
    pub conflict_rate: f32,
    pub fact_diversity_score: f32,
    pub processing_efficiency: f32,
}

/// 处理洞察
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingInsights {
    pub dominant_fact_categories: Vec<String>,
    pub memory_growth_prediction: f32,
    pub suggested_optimizations: Vec<String>,
    pub attention_areas: Vec<String>,
}

/// 处理统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingStats {
    pub total_messages: usize,
    pub facts_extracted: usize,
    pub decisions_made: usize,
    pub high_confidence_decisions: usize,
    pub processing_time_ms: u64,
}

/// 处理配置
#[derive(Debug, Clone)]
pub struct ProcessingConfig {
    pub similarity_threshold: f32,
    pub confidence_threshold: f32,
    pub max_facts_per_message: usize,
    pub enable_fact_validation: bool,
    pub enable_fact_merging: bool,
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.7,
            confidence_threshold: 0.5,
            max_facts_per_message: 10,
            enable_fact_validation: true,
            enable_fact_merging: true,
        }
    }
}

/// 智能记忆处理器
pub struct IntelligentMemoryProcessor {
    fact_extractor: FactExtractor,
    decision_engine: MemoryDecisionEngine,
    conflict_resolver: ConflictResolver,
    config: ProcessingConfig,
}

impl IntelligentMemoryProcessor {
    /// 创建新的智能处理器（异步版本）
    pub async fn new(api_key: String) -> Result<Self> {
        let llm_config = LLMConfig {
            provider: "openai".to_string(),
            model: "gpt-3.5-turbo".to_string(),
            api_key: Some(api_key),
            base_url: None,
            temperature: Some(0.7),
            max_tokens: Some(2000),
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            response_format: None,
        };

        let llm = RealLLMFactory::create_with_fallback(&llm_config).await?;
        let fact_extractor = FactExtractor::new(llm.clone());
        let decision_engine = MemoryDecisionEngine::new(llm.clone());
        let conflict_resolver =
            ConflictResolver::new(llm.clone(), ConflictResolverConfig::default());
        let config = ProcessingConfig::default();

        Ok(Self {
            fact_extractor,
            decision_engine,
            conflict_resolver,
            config,
        })
    }

    /// 使用自定义配置创建处理器（异步版本）
    pub async fn with_config(api_key: String, config: ProcessingConfig) -> Result<Self> {
        let llm_config = LLMConfig {
            provider: "openai".to_string(),
            model: "gpt-3.5-turbo".to_string(),
            api_key: Some(api_key),
            base_url: None,
            temperature: Some(0.7),
            max_tokens: Some(2000),
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            response_format: None,
        };

        let llm = RealLLMFactory::create_with_fallback(&llm_config).await?;
        let fact_extractor = FactExtractor::new(llm.clone());
        let decision_engine = MemoryDecisionEngine::new(llm.clone());
        let conflict_resolver =
            ConflictResolver::new(llm.clone(), ConflictResolverConfig::default());

        Ok(Self {
            fact_extractor,
            decision_engine,
            conflict_resolver,
            config,
        })
    }

    /// 处理消息并生成智能记忆操作
    pub async fn process_messages(
        &self,
        messages: &[Message],
        existing_memories: &[ExistingMemory],
    ) -> Result<IntelligentProcessingResult> {
        let start_time = std::time::Instant::now();

        // 1. 提取事实
        let mut extracted_facts = self.fact_extractor.extract_facts_internal(messages).await?;

        // 2. 验证事实（如果启用）
        if self.config.enable_fact_validation {
            extracted_facts = self.fact_extractor.validate_facts(extracted_facts);
        }

        // 3. 合并相似事实（如果启用）
        if self.config.enable_fact_merging {
            extracted_facts = self.fact_extractor.merge_similar_facts(extracted_facts);
        }

        // 4. 限制事实数量
        if extracted_facts.len() > self.config.max_facts_per_message {
            extracted_facts.truncate(self.config.max_facts_per_message);
        }

        // 5. 冲突检测
        let new_memories: Vec<MemoryItem> = extracted_facts
            .iter()
            .map(|fact| MemoryItem {
                id: uuid::Uuid::new_v4().to_string(),
                content: fact.content.clone(),
                hash: None,
                metadata: fact
                    .metadata
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
                score: Some(fact.confidence),
                created_at: chrono::Utc::now(),
                updated_at: None,
                session: Session::default(),
                memory_type: MemoryType::Episodic,
                entities: fact
                    .entities
                    .iter()
                    .map(|e| agent_mem_traits::Entity {
                        id: e.id.clone(),
                        name: e.name.clone(),
                        entity_type: format!("{:?}", e.entity_type),
                        attributes: e
                            .attributes
                            .iter()
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect(),
                    })
                    .collect(),
                relations: vec![], // 简化处理，暂时为空
                agent_id: "default".to_string(),
                user_id: None,
                importance: fact.confidence,
                embedding: None,
                last_accessed_at: chrono::Utc::now(),
                access_count: 0,
                expires_at: None,
                version: 1,
            })
            .collect();

        // 转换 ExistingMemory 到 MemoryItem
        let existing_memory_items: Vec<MemoryItem> = existing_memories
            .iter()
            .map(|mem| MemoryItem {
                id: mem.id.clone(),
                content: mem.content.clone(),
                hash: None,
                metadata: mem
                    .metadata
                    .iter()
                    .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                    .collect(),
                score: Some(mem.importance),
                created_at: chrono::DateTime::parse_from_rfc3339(&mem.created_at)
                    .unwrap_or_else(|_| chrono::Utc::now().into())
                    .with_timezone(&chrono::Utc),
                updated_at: mem.updated_at.as_ref().and_then(|s| {
                    chrono::DateTime::parse_from_rfc3339(s)
                        .ok()
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                }),
                session: Session::default(),
                memory_type: MemoryType::Episodic,
                entities: vec![],
                relations: vec![],
                agent_id: "default".to_string(),
                user_id: None,
                importance: mem.importance,
                embedding: None,
                last_accessed_at: chrono::Utc::now(),
                access_count: 0,
                expires_at: None,
                version: 1,
            })
            .collect();

        // Convert MemoryItem to MemoryV4 for conflict detection
        use agent_mem_traits::MemoryV4;

        let new_memories_v4: Vec<MemoryV4> = new_memories
            .iter()
            .map(|item| MemoryV4::from_legacy_item(item))
            .collect();

        let existing_memories_v4: Vec<MemoryV4> = existing_memory_items
            .iter()
            .map(|item| MemoryV4::from_legacy_item(item))
            .collect();

        let conflict_detections = self
            .conflict_resolver
            .detect_conflicts(&new_memories_v4, &existing_memories_v4)
            .await?;

        // 6. 生成记忆决策
        let memory_decisions = self
            .decision_engine
            .make_decisions(&extracted_facts, existing_memories)
            .await?;

        // 7. 生成推荐
        let recommendations = self.generate_recommendations(&extracted_facts, &memory_decisions);

        // 8. 计算质量指标
        let quality_metrics = self.calculate_quality_metrics(
            &extracted_facts,
            &memory_decisions,
            &conflict_detections,
        );

        // 9. 生成处理洞察
        let processing_insights = self.generate_processing_insights(
            &extracted_facts,
            &memory_decisions,
            existing_memories,
        );

        // 7. 计算统计信息
        let processing_time = start_time.elapsed();
        let high_confidence_decisions = memory_decisions
            .iter()
            .filter(|d| d.confidence > 0.8)
            .count();

        let processing_stats = ProcessingStats {
            total_messages: messages.len(),
            facts_extracted: extracted_facts.len(),
            decisions_made: memory_decisions.len(),
            high_confidence_decisions,
            processing_time_ms: processing_time.as_millis() as u64,
        };

        Ok(IntelligentProcessingResult {
            extracted_facts,
            memory_decisions,
            conflict_detections,
            processing_stats,
            recommendations,
            quality_metrics,
            processing_insights,
        })
    }

    /// 处理单个消息
    pub async fn process_single_message(
        &self,
        message: &Message,
        existing_memories: &[ExistingMemory],
    ) -> Result<IntelligentProcessingResult> {
        self.process_messages(&[message.clone()], existing_memories)
            .await
    }

    /// 分析现有记忆并提供优化建议
    pub async fn analyze_memory_health(
        &self,
        existing_memories: &[ExistingMemory],
    ) -> Result<MemoryHealthReport> {
        let mut report = MemoryHealthReport::default();

        // 分析记忆质量
        for memory in existing_memories {
            if memory.importance < 0.3 {
                report.low_importance_memories.push(memory.id.as_str().to_string());
            }

            let content_len = memory.content.len();
            if content_len < 20 {
                report.short_memories.push(memory.id.as_str().to_string());
            }
        }

        // 检测重复记忆
        for (i, memory1) in existing_memories.iter().enumerate() {
            for memory2 in existing_memories.iter().skip(i + 1) {
                let content1 = &memory1.content;
                let content2 = &memory2.content;
                let similarity = self
                    .decision_engine
                    .calculate_content_similarity(content1, content2);

                if similarity > 0.8 {
                    report
                        .duplicate_memories
                        .push((memory1.id.as_str().to_string(), memory2.id.as_str().to_string()));
                }
            }
        }

        // 生成优化建议
        if !report.low_importance_memories.is_empty() {
            report.suggestions.push(format!(
                "Consider removing {} low-importance memories",
                report.low_importance_memories.len()
            ));
        }

        if !report.duplicate_memories.is_empty() {
            report.suggestions.push(format!(
                "Found {} pairs of duplicate memories that could be merged",
                report.duplicate_memories.len()
            ));
        }

        Ok(report)
    }

    /// 计算质量指标
    fn calculate_quality_metrics(
        &self,
        facts: &[ExtractedFact],
        decisions: &[MemoryDecision],
        conflicts: &[ConflictDetection],
    ) -> QualityMetrics {
        let average_fact_confidence = if facts.is_empty() {
            0.0
        } else {
            facts.iter().map(|f| f.confidence).sum::<f32>() / facts.len() as f32
        };

        let average_decision_confidence = if decisions.is_empty() {
            0.0
        } else {
            decisions.iter().map(|d| d.confidence).sum::<f32>() / decisions.len() as f32
        };

        let conflict_rate = if facts.is_empty() {
            0.0
        } else {
            conflicts.len() as f32 / facts.len() as f32
        };

        // 计算事实多样性分数（基于不同类别的数量）
        let unique_categories: std::collections::HashSet<_> = facts
            .iter()
            .map(|f| std::mem::discriminant(&f.category))
            .collect();
        let fact_diversity_score = if facts.is_empty() {
            0.0
        } else {
            unique_categories.len() as f32 / 15.0 // 15是总类别数
        };

        // 处理效率（基于事实数量和决策数量的比率）
        let processing_efficiency = if facts.is_empty() {
            1.0
        } else {
            decisions.len() as f32 / facts.len() as f32
        };

        QualityMetrics {
            average_fact_confidence,
            average_decision_confidence,
            conflict_rate,
            fact_diversity_score,
            processing_efficiency,
        }
    }

    /// 生成处理洞察
    fn generate_processing_insights(
        &self,
        facts: &[ExtractedFact],
        decisions: &[MemoryDecision],
        existing_memories: &[ExistingMemory],
    ) -> ProcessingInsights {
        // 统计主要事实类别
        let mut category_counts = std::collections::HashMap::new();
        for fact in facts {
            let category_name = format!("{:?}", fact.category);
            *category_counts.entry(category_name).or_insert(0) += 1;
        }

        let mut dominant_fact_categories: Vec<_> = category_counts.into_iter().collect();
        dominant_fact_categories.sort_by(|a, b| b.1.cmp(&a.1));
        let dominant_fact_categories: Vec<String> = dominant_fact_categories
            .into_iter()
            .take(3)
            .map(|(category, _)| category)
            .collect();

        // 预测记忆增长
        let memory_growth_prediction = if existing_memories.is_empty() {
            facts.len() as f32
        } else {
            let growth_rate = facts.len() as f32 / existing_memories.len() as f32;
            growth_rate.min(2.0) // 限制最大增长率
        };

        // 生成优化建议
        let mut suggested_optimizations = Vec::new();
        if facts.len() > 10 {
            suggested_optimizations.push("Consider increasing batch processing size".to_string());
        }
        if decisions.iter().any(|d| d.confidence < 0.5) {
            suggested_optimizations.push("Review low-confidence decisions".to_string());
        }

        // 生成注意区域
        let mut attention_areas = Vec::new();
        if facts
            .iter()
            .any(|f| matches!(f.category, crate::fact_extraction::FactCategory::Personal))
        {
            attention_areas
                .push("Personal information detected - ensure privacy compliance".to_string());
        }
        if facts
            .iter()
            .any(|f| matches!(f.category, crate::fact_extraction::FactCategory::Financial))
        {
            attention_areas
                .push("Financial information detected - ensure security measures".to_string());
        }

        ProcessingInsights {
            dominant_fact_categories,
            memory_growth_prediction,
            suggested_optimizations,
            attention_areas,
        }
    }

    /// 生成处理推荐
    fn generate_recommendations(
        &self,
        facts: &[ExtractedFact],
        decisions: &[MemoryDecision],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // 基于事实数量的推荐
        if facts.len() > 5 {
            recommendations.push(
                "High number of facts extracted. Consider breaking down conversations into smaller segments.".to_string()
            );
        }

        // 基于决策置信度的推荐
        let low_confidence_decisions = decisions.iter().filter(|d| d.confidence < 0.6).count();
        if low_confidence_decisions > 0 {
            recommendations.push(format!(
                "{low_confidence_decisions} decisions have low confidence. Manual review recommended."
            ));
        }

        // 基于事实类别的推荐
        let personal_facts = facts
            .iter()
            .filter(|f| matches!(f.category, crate::fact_extraction::FactCategory::Personal))
            .count();
        if personal_facts > 0 {
            recommendations.push("Personal facts detected. Ensure privacy compliance.".to_string());
        }

        recommendations
    }

    /// 获取处理器配置
    pub fn get_config(&self) -> &ProcessingConfig {
        &self.config
    }

    /// 更新处理器配置
    pub fn update_config(&mut self, config: ProcessingConfig) {
        self.config = config;
    }
}

/// 记忆健康报告
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MemoryHealthReport {
    pub total_memories: usize,
    pub low_importance_memories: Vec<String>,
    pub short_memories: Vec<String>,
    pub duplicate_memories: Vec<(String, String)>,
    pub suggestions: Vec<String>,
    pub overall_health_score: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fact_extraction::FactCategory;

    #[test]
    fn test_processing_config_default() {
        let config = ProcessingConfig::default();
        assert_eq!(config.similarity_threshold, 0.7);
        assert_eq!(config.confidence_threshold, 0.5);
        assert_eq!(config.max_facts_per_message, 10);
        assert!(config.enable_fact_validation);
        assert!(config.enable_fact_merging);
    }

    #[test]
    fn test_processing_stats_creation() {
        let stats = ProcessingStats {
            total_messages: 5,
            facts_extracted: 3,
            decisions_made: 2,
            high_confidence_decisions: 1,
            processing_time_ms: 150,
        };

        assert_eq!(stats.total_messages, 5);
        assert_eq!(stats.facts_extracted, 3);
        assert_eq!(stats.processing_time_ms, 150);
    }

    #[test]
    fn test_memory_health_report_default() {
        let report = MemoryHealthReport::default();
        assert_eq!(report.total_memories, 0);
        assert!(report.low_importance_memories.is_empty());
        assert!(report.suggestions.is_empty());
    }

    #[tokio::test]
    #[ignore] // Requires LLM provider access
    async fn test_recommendation_generation() {
        // 创建测试数据
        let facts = vec![
            ExtractedFact {
                content: "User's name is John".to_string(),
                confidence: 0.9,
                category: FactCategory::Personal,
                entities: vec![],
                temporal_info: None,
                source_message_id: None,
                metadata: std::collections::HashMap::new(),
            },
            ExtractedFact {
                content: "User lives in New York".to_string(),
                confidence: 0.8,
                category: FactCategory::Personal,
                entities: vec![],
                temporal_info: None,
                source_message_id: None,
                metadata: std::collections::HashMap::new(),
            },
        ];

        let decisions = vec![MemoryDecision {
            action: crate::decision_engine::MemoryAction::Add {
                content: "Test".to_string(),
                importance: 0.8,
                metadata: HashMap::new(),
            },
            confidence: 0.5, // 低置信度
            reasoning: "Test".to_string(),
            affected_memories: vec![],
            estimated_impact: 0.5,
        }];

        // 创建处理器实例进行测试
        let processor = IntelligentMemoryProcessor::new("test-key".to_string())
            .await
            .unwrap();
        let recommendations = processor.generate_recommendations(&facts, &decisions);

        // 验证推荐生成
        assert!(!recommendations.is_empty());

        // 应该包含关于低置信度决策的推荐
        assert!(recommendations.iter().any(|r| r.contains("low confidence")));

        // 应该包含关于个人信息的推荐
        assert!(recommendations
            .iter()
            .any(|r| r.contains("Personal facts") || r.contains("privacy")));
    }
}

/// 增强智能处理器 (Mem5 版本)
///
/// 按照 Mem5 计划实现的完整智能处理流水线，整合：
/// - 高级事实提取器
/// - 智能决策引擎
/// - 冲突解决系统
/// - 重要性评估器
pub struct EnhancedIntelligentProcessor {
    fact_extractor: AdvancedFactExtractor,
    decision_engine: EnhancedDecisionEngine,
    conflict_resolver: ConflictResolver,
    importance_evaluator: ImportanceEvaluator,
    config: ProcessorConfig,
}

/// 处理器配置
#[derive(Debug, Clone)]
pub struct ProcessorConfig {
    /// 并行处理线程数
    pub parallel_threads: usize,
    /// 处理超时时间（秒）
    pub processing_timeout_seconds: u64,
    /// 启用性能监控
    pub enable_performance_monitoring: bool,
    /// 启用详细日志
    pub enable_detailed_logging: bool,
}

impl Default for ProcessorConfig {
    fn default() -> Self {
        Self {
            parallel_threads: 4,
            processing_timeout_seconds: 120,
            enable_performance_monitoring: true,
            enable_detailed_logging: false,
        }
    }
}

/// 增强处理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedProcessingResult {
    /// 处理ID
    pub processing_id: String,
    /// 提取的结构化事实
    pub structured_facts: Vec<StructuredFact>,
    /// 重要性评估结果
    pub importance_evaluations: Vec<ImportanceEvaluation>,
    /// 冲突检测结果
    pub conflict_detections: Vec<ConflictDetection>,
    /// 决策结果
    pub decision_result: DecisionResult,
    /// 处理统计
    pub processing_stats: EnhancedProcessingStats,
    /// 处理时间
    pub processed_at: chrono::DateTime<chrono::Utc>,
    /// 处理置信度
    pub overall_confidence: f32,
}

/// 增强处理统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedProcessingStats {
    /// 处理的消息数量
    pub messages_processed: usize,
    /// 提取的事实数量
    pub facts_extracted: usize,
    /// 检测的冲突数量
    pub conflicts_detected: usize,
    /// 生成的决策数量
    pub decisions_made: usize,
    /// 总处理时间（毫秒）
    pub total_processing_time_ms: u64,
    /// 各阶段处理时间
    pub stage_timings: HashMap<String, u64>,
    /// 性能指标
    pub performance_metrics: PerformanceMetrics,
}

/// 性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// 吞吐量（事实/秒）
    pub throughput_facts_per_second: f32,
    /// 平均响应时间（毫秒）
    pub average_response_time_ms: f32,
    /// 内存使用量（MB）
    pub memory_usage_mb: f32,
    /// CPU 使用率
    pub cpu_usage_percent: f32,
}

impl EnhancedIntelligentProcessor {
    /// 创建新的增强智能处理器
    pub fn new(llm: Arc<dyn LLMProvider + Send + Sync>, config: ProcessorConfig) -> Self {
        let fact_extractor = AdvancedFactExtractor::new(llm.clone());
        let decision_engine = EnhancedDecisionEngine::new(
            llm.clone(),
            crate::decision_engine::DecisionEngineConfig::default(),
        );
        let conflict_resolver =
            ConflictResolver::new(llm.clone(), ConflictResolverConfig::default());
        let importance_evaluator =
            ImportanceEvaluator::new(llm.clone(), ImportanceEvaluatorConfig::default());

        Self {
            fact_extractor,
            decision_engine,
            conflict_resolver,
            importance_evaluator,
            config,
        }
    }

    /// 处理记忆添加 (Mem5 核心功能)
    pub async fn process_memory_addition(
        &self,
        messages: &[Message],
        existing_memories: &[Memory],
    ) -> Result<EnhancedProcessingResult> {
        let start_time = std::time::Instant::now();
        let processing_id = format!("proc_{}", chrono::Utc::now().timestamp());

        info!(
            "开始增强记忆处理，ID: {}, 消息数: {}, 现有记忆数: {}",
            processing_id,
            messages.len(),
            existing_memories.len()
        );

        let mut stage_timings = HashMap::new();

        // 1. 事实提取阶段
        let fact_start = std::time::Instant::now();
        let structured_facts = self
            .fact_extractor
            .extract_structured_facts(messages)
            .await?;
        stage_timings.insert(
            "fact_extraction".to_string(),
            fact_start.elapsed().as_millis() as u64,
        );

        info!(
            "事实提取完成，提取到 {} 个结构化事实",
            structured_facts.len()
        );

        // 2. 重要性评估阶段
        let importance_start = std::time::Instant::now();
        let mut importance_evaluations = Vec::new();
        let _facts_map: HashMap<String, Vec<StructuredFact>> = structured_facts
            .iter()
            .enumerate()
            .map(|(i, fact)| (format!("temp_memory_{i}"), vec![fact.clone()]))
            .collect();

        // 为每个事实创建临时记忆进行评估
        for (i, fact) in structured_facts.iter().enumerate() {
            let temp_memory_item = MemoryItem {
                id: format!("temp_memory_{i}"),
                content: fact.description.clone(),
                hash: None,
                metadata: HashMap::new(),
                score: Some(fact.confidence),
                created_at: chrono::Utc::now(),
                updated_at: Some(chrono::Utc::now()),
                session: agent_mem_traits::Session::new(),
                memory_type: agent_mem_traits::MemoryType::Episodic,
                entities: fact
                    .entities
                    .iter()
                    .map(|e| agent_mem_traits::Entity {
                        id: e.id.clone(),
                        name: e.name.clone(),
                        entity_type: format!("{:?}", e.entity_type),
                        attributes: HashMap::new(),
                    })
                    .collect(),
                relations: fact
                    .relations
                    .iter()
                    .map(|r| agent_mem_traits::Relation {
                        relation_type: r.predicate.clone(),
                        source: r.subject.clone(),
                        target: r.object.clone(),
                        confidence: r.confidence,
                    })
                    .collect(),
                agent_id: "demo_agent".to_string(),
                user_id: None,
                importance: fact.importance,
                embedding: None,
                last_accessed_at: chrono::Utc::now(),
                access_count: 0,
                expires_at: None,
                version: 1,
            };
            
            let temp_memory = agent_mem_core::storage::conversion::legacy_to_v4(&temp_memory_item);

            let evaluation = self
                .importance_evaluator
                .evaluate_importance(&temp_memory, &[fact.clone()], existing_memories)
                .await?;
            importance_evaluations.push(evaluation);
        }
        stage_timings.insert(
            "importance_evaluation".to_string(),
            importance_start.elapsed().as_millis() as u64,
        );

        info!(
            "重要性评估完成，评估了 {} 个记忆",
            importance_evaluations.len()
        );

        // 3. 冲突检测阶段
        let conflict_start = std::time::Instant::now();
        let temp_memory_items: Vec<MemoryItem> = structured_facts
            .iter()
            .enumerate()
            .map(|(i, fact)| MemoryItem {
                id: format!("temp_memory_{i}"),
                content: fact.description.clone(),
                hash: None,
                metadata: HashMap::new(),
                score: Some(fact.confidence),
                created_at: chrono::Utc::now(),
                updated_at: Some(chrono::Utc::now()),
                session: agent_mem_traits::Session::new(),
                memory_type: agent_mem_traits::MemoryType::Episodic,
                entities: fact
                    .entities
                    .iter()
                    .map(|e| agent_mem_traits::Entity {
                        id: e.id.clone(),
                        name: e.name.clone(),
                        entity_type: format!("{:?}", e.entity_type),
                        attributes: HashMap::new(),
                    })
                    .collect(),
                relations: fact
                    .relations
                    .iter()
                    .map(|r| agent_mem_traits::Relation {
                        relation_type: r.predicate.clone(),
                        source: r.subject.clone(),
                        target: r.object.clone(),
                        confidence: r.confidence,
                    })
                    .collect(),
                agent_id: "demo_agent".to_string(),
                user_id: None,
                importance: fact.importance,
                embedding: None,
                last_accessed_at: chrono::Utc::now(),
                access_count: 0,
                expires_at: None,
                version: 1,
            })
            .collect();
        
        let temp_memories: Vec<Memory> = temp_memory_items
            .iter()
            .map(|item| agent_mem_core::storage::conversion::legacy_to_v4(item))
            .collect();

        let conflict_detections = self
            .conflict_resolver
            .detect_conflicts(&temp_memories, existing_memories)
            .await?;
        stage_timings.insert(
            "conflict_detection".to_string(),
            conflict_start.elapsed().as_millis() as u64,
        );

        info!("冲突检测完成，检测到 {} 个冲突", conflict_detections.len());

        // 4. 决策制定阶段
        let decision_start = std::time::Instant::now();
        let decision_context = DecisionContext {
            new_facts: structured_facts.clone(),
            existing_memories: existing_memories.to_vec(),
            importance_evaluations: importance_evaluations.clone(),
            conflict_detections: conflict_detections.clone(),
            user_preferences: HashMap::new(),
        };

        let decision_result = self
            .decision_engine
            .make_decisions(&decision_context)
            .await?;
        stage_timings.insert(
            "decision_making".to_string(),
            decision_start.elapsed().as_millis() as u64,
        );

        info!(
            "决策制定完成，生成 {} 个推荐操作",
            decision_result.recommended_actions.len()
        );

        // 5. 计算处理统计和性能指标
        let total_time = start_time.elapsed();
        let processing_stats = self.calculate_processing_stats(
            messages.len(),
            &structured_facts,
            &conflict_detections,
            &decision_result,
            total_time,
            stage_timings,
        );

        // 6. 计算整体置信度
        let overall_confidence = self.calculate_overall_confidence(
            &structured_facts,
            &importance_evaluations,
            &decision_result,
        );

        let result = EnhancedProcessingResult {
            processing_id,
            structured_facts,
            importance_evaluations,
            conflict_detections,
            decision_result,
            processing_stats,
            processed_at: chrono::Utc::now(),
            overall_confidence,
        };

        info!(
            "增强记忆处理完成，整体置信度: {:.2}, 总耗时: {}ms",
            result.overall_confidence, result.processing_stats.total_processing_time_ms
        );

        Ok(result)
    }

    /// 计算处理统计
    fn calculate_processing_stats(
        &self,
        messages_count: usize,
        facts: &[StructuredFact],
        conflicts: &[crate::conflict_resolution::ConflictDetection],
        decision: &DecisionResult,
        total_time: std::time::Duration,
        stage_timings: HashMap<String, u64>,
    ) -> EnhancedProcessingStats {
        let total_ms = total_time.as_millis() as u64;
        let throughput = if total_ms > 0 {
            (facts.len() as f32 * 1000.0) / total_ms as f32
        } else {
            0.0
        };

        EnhancedProcessingStats {
            messages_processed: messages_count,
            facts_extracted: facts.len(),
            conflicts_detected: conflicts.len(),
            decisions_made: decision.recommended_actions.len(),
            total_processing_time_ms: total_ms,
            stage_timings,
            performance_metrics: PerformanceMetrics {
                throughput_facts_per_second: throughput,
                average_response_time_ms: total_ms as f32 / messages_count as f32,
                memory_usage_mb: 0.0,   // 简化实现
                cpu_usage_percent: 0.0, // 简化实现
            },
        }
    }

    /// 计算整体置信度
    fn calculate_overall_confidence(
        &self,
        facts: &[StructuredFact],
        evaluations: &[ImportanceEvaluation],
        decision: &DecisionResult,
    ) -> f32 {
        let fact_confidence = if facts.is_empty() {
            0.0
        } else {
            facts.iter().map(|f| f.confidence).sum::<f32>() / facts.len() as f32
        };

        let evaluation_confidence = if evaluations.is_empty() {
            0.0
        } else {
            evaluations.iter().map(|e| e.confidence).sum::<f32>() / evaluations.len() as f32
        };

        let decision_confidence = decision.confidence;

        (fact_confidence + evaluation_confidence + decision_confidence) / 3.0
    }
}
