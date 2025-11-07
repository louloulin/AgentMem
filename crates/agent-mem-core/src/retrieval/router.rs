//! RetrievalRouter - 智能检索路由器
//!
//! 参考 MIRIX 的多策略检索机制，实现基于主题的智能路由

use crate::retrieval::{ExtractedTopic, RetrievalRequest};
use crate::types::MemoryType;
use agent_mem_traits::{AgentMemError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 检索策略
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RetrievalStrategy {
    /// 向量嵌入检索
    Embedding,
    /// BM25 全文检索
    BM25,
    /// 字符串匹配检索
    StringMatch,
    /// 模糊匹配检索
    FuzzyMatch,
    /// 混合检索策略
    Hybrid,
    /// 语义图检索
    SemanticGraph,
    /// 时序检索
    Temporal,
    /// 上下文感知检索
    ContextAware,
}

impl RetrievalStrategy {
    /// 获取策略描述
    pub fn description(&self) -> &'static str {
        match self {
            RetrievalStrategy::Embedding => "基于向量嵌入的语义相似度检索",
            RetrievalStrategy::BM25 => "基于 BM25 算法的全文检索",
            RetrievalStrategy::StringMatch => "基于字符串包含的精确匹配检索",
            RetrievalStrategy::FuzzyMatch => "基于编辑距离的模糊匹配检索",
            RetrievalStrategy::Hybrid => "结合多种策略的混合检索",
            RetrievalStrategy::SemanticGraph => "基于知识图谱的语义关系检索",
            RetrievalStrategy::Temporal => "基于时间序列的时序检索",
            RetrievalStrategy::ContextAware => "基于上下文感知的智能检索",
        }
    }

    /// 获取策略权重
    pub fn weight(&self) -> f32 {
        match self {
            RetrievalStrategy::Embedding => 0.9,
            RetrievalStrategy::BM25 => 0.8,
            RetrievalStrategy::StringMatch => 0.6,
            RetrievalStrategy::FuzzyMatch => 0.7,
            RetrievalStrategy::Hybrid => 1.0,
            RetrievalStrategy::SemanticGraph => 0.85,
            RetrievalStrategy::Temporal => 0.75,
            RetrievalStrategy::ContextAware => 0.95,
        }
    }
}

/// 路由决策
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteDecision {
    /// 选择的检索策略
    pub selected_strategies: Vec<RetrievalStrategy>,
    /// 目标记忆类型
    pub target_memory_types: Vec<MemoryType>,
    /// 策略权重分配
    pub strategy_weights: HashMap<RetrievalStrategy, f32>,
    /// 决策置信度
    pub confidence: f32,
    /// 决策原因
    pub reasoning: Vec<String>,
    /// 预估性能指标
    pub estimated_performance: PerformanceEstimate,
}

/// 性能预估
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceEstimate {
    /// 预估响应时间（毫秒）
    pub estimated_response_time_ms: u64,
    /// 预估准确率
    pub estimated_accuracy: f32,
    /// 预估召回率
    pub estimated_recall: f32,
    /// 预估资源消耗
    pub estimated_resource_usage: f32,
}

/// 路由结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingResult {
    /// 路由决策
    pub decision: RouteDecision,
    /// 路由时间（毫秒）
    pub routing_time_ms: u64,
    /// 路由成功标志
    pub success: bool,
    /// 错误信息（如果有）
    pub error_message: Option<String>,
}

/// 检索路由器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalRouterConfig {
    /// 默认检索策略
    pub default_strategies: Vec<RetrievalStrategy>,
    /// 策略选择阈值
    pub strategy_selection_threshold: f32,
    /// 是否启用自适应路由
    pub enable_adaptive_routing: bool,
    /// 是否启用性能监控
    pub enable_performance_monitoring: bool,
    /// 最大并发检索数
    pub max_concurrent_retrievals: usize,
    /// 路由超时时间（秒）
    pub routing_timeout_seconds: u64,
    /// 主题-策略映射规则
    pub topic_strategy_mapping: HashMap<String, Vec<RetrievalStrategy>>,
    /// 记忆类型-策略映射规则
    pub memory_type_strategy_mapping: HashMap<MemoryType, Vec<RetrievalStrategy>>,
}

impl Default for RetrievalRouterConfig {
    fn default() -> Self {
        let mut topic_strategy_mapping = HashMap::new();
        topic_strategy_mapping.insert(
            "technical".to_string(),
            vec![
                RetrievalStrategy::Embedding,
                RetrievalStrategy::SemanticGraph,
            ],
        );
        topic_strategy_mapping.insert(
            "business".to_string(),
            vec![RetrievalStrategy::BM25, RetrievalStrategy::ContextAware],
        );
        topic_strategy_mapping.insert(
            "personal".to_string(),
            vec![RetrievalStrategy::Temporal, RetrievalStrategy::Embedding],
        );

        let mut memory_type_strategy_mapping = HashMap::new();
        memory_type_strategy_mapping.insert(
            MemoryType::Episodic,
            vec![RetrievalStrategy::Temporal, RetrievalStrategy::ContextAware],
        );
        memory_type_strategy_mapping.insert(
            MemoryType::Semantic,
            vec![
                RetrievalStrategy::Embedding,
                RetrievalStrategy::SemanticGraph,
            ],
        );
        memory_type_strategy_mapping.insert(
            MemoryType::Procedural,
            vec![RetrievalStrategy::BM25, RetrievalStrategy::StringMatch],
        );
        memory_type_strategy_mapping.insert(
            MemoryType::Working,
            vec![RetrievalStrategy::ContextAware, RetrievalStrategy::Temporal],
        );

        Self {
            default_strategies: vec![RetrievalStrategy::Embedding, RetrievalStrategy::BM25],
            strategy_selection_threshold: 0.6,
            enable_adaptive_routing: true,
            enable_performance_monitoring: true,
            max_concurrent_retrievals: 5,
            routing_timeout_seconds: 10,
            topic_strategy_mapping,
            memory_type_strategy_mapping,
        }
    }
}

/// 路由器统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterStats {
    /// 总路由次数
    pub total_routes: u64,
    /// 成功路由次数
    pub successful_routes: u64,
    /// 平均路由时间（毫秒）
    pub avg_routing_time_ms: f64,
    /// 策略使用统计
    pub strategy_usage_stats: HashMap<RetrievalStrategy, u64>,
    /// 记忆类型路由统计
    pub memory_type_routing_stats: HashMap<MemoryType, u64>,
    /// 平均决策置信度
    pub avg_decision_confidence: f32,
}

/// 检索路由器
///
/// 基于主题和上下文的智能检索路由系统
pub struct RetrievalRouter {
    /// 配置
    config: RetrievalRouterConfig,
    /// 统计信息
    stats: Arc<RwLock<RouterStats>>,
    /// 性能历史记录
    performance_history: Arc<RwLock<HashMap<RetrievalStrategy, Vec<PerformanceRecord>>>>,
    /// 自适应权重
    adaptive_weights: Arc<RwLock<HashMap<RetrievalStrategy, f32>>>,
}

/// 性能记录
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PerformanceRecord {
    /// 响应时间（毫秒）
    pub response_time_ms: u64,
    /// 准确率
    pub accuracy: f32,
    /// 召回率
    pub recall: f32,
    /// 记录时间
    pub timestamp: std::time::SystemTime,
}

impl RetrievalRouter {
    /// 创建新的检索路由器
    pub async fn new(config: RetrievalRouterConfig) -> Result<Self> {
        let stats = RouterStats {
            total_routes: 0,
            successful_routes: 0,
            avg_routing_time_ms: 0.0,
            strategy_usage_stats: HashMap::new(),
            memory_type_routing_stats: HashMap::new(),
            avg_decision_confidence: 0.0,
        };

        let mut adaptive_weights = HashMap::new();
        for strategy in &config.default_strategies {
            adaptive_weights.insert(strategy.clone(), strategy.weight());
        }

        Ok(Self {
            config,
            stats: Arc::new(RwLock::new(stats)),
            performance_history: Arc::new(RwLock::new(HashMap::new())),
            adaptive_weights: Arc::new(RwLock::new(adaptive_weights)),
        })
    }

    /// 执行检索路由
    pub async fn route_retrieval(
        &self,
        request: &RetrievalRequest,
        extracted_topics: &[ExtractedTopic],
    ) -> Result<RoutingResult> {
        let start_time = std::time::Instant::now();

        // 更新统计信息
        {
            let mut stats = self.stats.write().await;
            stats.total_routes += 1;
        }

        // 分析请求特征
        let request_features = self
            .analyze_request_features(request, extracted_topics)
            .await?;

        // 选择检索策略
        let selected_strategies = self.select_strategies(&request_features).await?;

        // 确定目标记忆类型
        let target_memory_types = self
            .determine_target_memory_types(request, extracted_topics)
            .await?;

        // 计算策略权重
        let strategy_weights = self
            .calculate_strategy_weights(&selected_strategies, &request_features)
            .await?;

        // 预估性能
        let estimated_performance = self.estimate_performance(&selected_strategies).await?;

        // 生成决策原因
        let reasoning =
            self.generate_reasoning(&selected_strategies, &target_memory_types, extracted_topics);

        // 计算决策置信度
        let confidence =
            self.calculate_decision_confidence(&selected_strategies, &request_features);

        let decision = RouteDecision {
            selected_strategies: selected_strategies.clone(),
            target_memory_types: target_memory_types.clone(),
            strategy_weights,
            confidence,
            reasoning,
            estimated_performance,
        };

        let routing_time_ms = start_time.elapsed().as_millis() as u64;

        // 更新统计信息
        self.update_routing_stats(
            &selected_strategies,
            &target_memory_types,
            routing_time_ms,
            confidence,
        )
        .await;

        Ok(RoutingResult {
            decision,
            routing_time_ms,
            success: true,
            error_message: None,
        })
    }

    /// 分析请求特征
    async fn analyze_request_features(
        &self,
        request: &RetrievalRequest,
        extracted_topics: &[ExtractedTopic],
    ) -> Result<RequestFeatures> {
        Ok(RequestFeatures {
            query_length: request.query.len(),
            has_context: request.context.is_some(),
            topic_count: extracted_topics.len(),
            primary_topic_category: extracted_topics.first().map(|t| t.category.clone()),
            has_preferred_strategy: request.preferred_strategy.is_some(),
            target_memory_types_specified: request.target_memory_types.is_some(),
        })
    }

    /// 选择检索策略
    async fn select_strategies(
        &self,
        features: &RequestFeatures,
    ) -> Result<Vec<RetrievalStrategy>> {
        let mut strategies = Vec::new();

        // 基于用户偏好
        if features.has_preferred_strategy {
            // 这里需要从 features 中获取实际的偏好策略
            // 简化处理，使用默认策略
        }

        // 基于主题类别选择策略
        if let Some(category) = &features.primary_topic_category {
            let category_key = format!("{category:?}").to_lowercase();
            if let Some(topic_strategies) = self.config.topic_strategy_mapping.get(&category_key) {
                strategies.extend(topic_strategies.clone());
            }
        }

        // 如果没有选择到策略，使用默认策略
        if strategies.is_empty() {
            strategies = self.config.default_strategies.clone();
        }

        // 去重并限制数量
        strategies.sort();
        strategies.dedup();
        strategies.truncate(3); // 最多选择3个策略

        Ok(strategies)
    }

    /// 确定目标记忆类型
    async fn determine_target_memory_types(
        &self,
        request: &RetrievalRequest,
        extracted_topics: &[ExtractedTopic],
    ) -> Result<Vec<MemoryType>> {
        // 如果请求中指定了目标类型，直接使用
        if let Some(target_types) = &request.target_memory_types {
            return Ok(target_types.clone());
        }

        // 基于主题推断记忆类型
        let mut memory_types = Vec::new();

        for topic in extracted_topics {
            match topic.category {
                crate::retrieval::TopicCategory::Technical => {
                    memory_types.extend(vec![MemoryType::Semantic, MemoryType::Procedural]);
                }
                crate::retrieval::TopicCategory::Personal => {
                    memory_types.extend(vec![MemoryType::Episodic, MemoryType::Core]);
                }
                crate::retrieval::TopicCategory::Business => {
                    memory_types.extend(vec![MemoryType::Working, MemoryType::Resource]);
                }
                _ => {
                    memory_types.push(MemoryType::Semantic);
                }
            }
        }

        // 去重
        memory_types.sort();
        memory_types.dedup();

        // 如果没有推断出类型，使用所有类型
        if memory_types.is_empty() {
            memory_types = vec![
                MemoryType::Episodic,
                MemoryType::Semantic,
                MemoryType::Procedural,
                MemoryType::Working,
            ];
        }

        Ok(memory_types)
    }

    /// 计算策略权重
    async fn calculate_strategy_weights(
        &self,
        strategies: &[RetrievalStrategy],
        _features: &RequestFeatures,
    ) -> Result<HashMap<RetrievalStrategy, f32>> {
        let mut weights = HashMap::new();
        let adaptive_weights = self.adaptive_weights.read().await;

        let mut strategy_weights = Vec::new();
        for strategy in strategies {
            let weight = adaptive_weights
                .get(strategy)
                .copied()
                .unwrap_or_else(|| strategy.weight());
            strategy_weights.push((strategy, weight));
        }

        let total_weight: f32 = strategy_weights.iter().map(|(_, w)| *w).sum();

        for (strategy, weight) in strategy_weights {
            weights.insert(strategy.clone(), weight / total_weight);
        }

        Ok(weights)
    }

    /// 预估性能
    async fn estimate_performance(
        &self,
        strategies: &[RetrievalStrategy],
    ) -> Result<PerformanceEstimate> {
        let performance_history = self.performance_history.read().await;

        let mut total_response_time = 0u64;
        let mut total_accuracy = 0.0f32;
        let mut total_recall = 0.0f32;
        let mut count = 0;

        for strategy in strategies {
            if let Some(records) = performance_history.get(strategy) {
                if let Some(latest_record) = records.last() {
                    total_response_time += latest_record.response_time_ms;
                    total_accuracy += latest_record.accuracy;
                    total_recall += latest_record.recall;
                    count += 1;
                }
            }
        }

        if count == 0 {
            // 使用默认估值
            Ok(PerformanceEstimate {
                estimated_response_time_ms: 100,
                estimated_accuracy: 0.8,
                estimated_recall: 0.7,
                estimated_resource_usage: 0.5,
            })
        } else {
            Ok(PerformanceEstimate {
                estimated_response_time_ms: total_response_time / count as u64,
                estimated_accuracy: total_accuracy / count as f32,
                estimated_recall: total_recall / count as f32,
                estimated_resource_usage: 0.5, // 简化处理
            })
        }
    }

    /// 生成决策原因
    fn generate_reasoning(
        &self,
        strategies: &[RetrievalStrategy],
        memory_types: &[MemoryType],
        topics: &[ExtractedTopic],
    ) -> Vec<String> {
        let mut reasoning = Vec::new();

        reasoning.push(format!("选择了 {} 种检索策略", strategies.len()));
        reasoning.push(format!("目标记忆类型: {memory_types:?}"));

        if !topics.is_empty() {
            reasoning.push(format!("基于 {} 个提取的主题进行路由", topics.len()));
        }

        for strategy in strategies {
            reasoning.push(format!("策略 {:?}: {}", strategy, strategy.description()));
        }

        reasoning
    }

    /// 计算决策置信度
    fn calculate_decision_confidence(
        &self,
        strategies: &[RetrievalStrategy],
        _features: &RequestFeatures,
    ) -> f32 {
        if strategies.is_empty() {
            return 0.0;
        }

        let avg_weight: f32 =
            strategies.iter().map(|s| s.weight()).sum::<f32>() / strategies.len() as f32;
        avg_weight.min(1.0)
    }

    /// 更新路由统计信息
    async fn update_routing_stats(
        &self,
        strategies: &[RetrievalStrategy],
        memory_types: &[MemoryType],
        routing_time_ms: u64,
        confidence: f32,
    ) {
        let mut stats = self.stats.write().await;
        stats.successful_routes += 1;

        // 更新平均路由时间
        let total_time = stats.avg_routing_time_ms * (stats.successful_routes - 1) as f64
            + routing_time_ms as f64;
        stats.avg_routing_time_ms = total_time / stats.successful_routes as f64;

        // 更新策略使用统计
        for strategy in strategies {
            *stats
                .strategy_usage_stats
                .entry(strategy.clone())
                .or_insert(0) += 1;
        }

        // 更新记忆类型路由统计
        for memory_type in memory_types {
            *stats
                .memory_type_routing_stats
                .entry(*memory_type)
                .or_insert(0) += 1;
        }

        // 更新平均决策置信度
        let total_confidence =
            stats.avg_decision_confidence * (stats.successful_routes - 1) as f32 + confidence;
        stats.avg_decision_confidence = total_confidence / stats.successful_routes as f32;
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> Result<serde_json::Value> {
        let stats = self.stats.read().await;
        serde_json::to_value(&*stats)
            .map_err(|e| AgentMemError::ProcessingError(format!("Failed to serialize stats: {e}")))
    }
}

/// 请求特征
#[derive(Debug, Clone)]
struct RequestFeatures {
    pub query_length: usize,
    pub has_context: bool,
    pub topic_count: usize,
    pub primary_topic_category: Option<crate::retrieval::TopicCategory>,
    pub has_preferred_strategy: bool,
    pub target_memory_types_specified: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retrieval_strategy_description() {
        assert_eq!(
            RetrievalStrategy::Embedding.description(),
            "基于向量嵌入的语义相似度检索"
        );
        assert_eq!(
            RetrievalStrategy::BM25.description(),
            "基于 BM25 算法的全文检索"
        );
        assert_eq!(
            RetrievalStrategy::Hybrid.description(),
            "结合多种策略的混合检索"
        );
    }

    #[test]
    fn test_retrieval_strategy_weight() {
        assert_eq!(RetrievalStrategy::Embedding.weight(), 0.9);
        assert_eq!(RetrievalStrategy::BM25.weight(), 0.8);
        assert_eq!(RetrievalStrategy::Hybrid.weight(), 1.0);
        assert_eq!(RetrievalStrategy::ContextAware.weight(), 0.95);
    }

    #[test]
    fn test_retrieval_strategy_ordering() {
        let mut strategies = [
            RetrievalStrategy::BM25,
            RetrievalStrategy::Embedding,
            RetrievalStrategy::StringMatch,
        ];
        strategies.sort();

        // 验证排序后的顺序
        assert_eq!(strategies.len(), 3);
    }

    #[test]
    fn test_retrieval_strategy_equality() {
        assert_eq!(RetrievalStrategy::Embedding, RetrievalStrategy::Embedding);
        assert_ne!(RetrievalStrategy::Embedding, RetrievalStrategy::BM25);
    }

    #[test]
    fn test_retrieval_strategy_serialization() {
        let strategy = RetrievalStrategy::Hybrid;
        let json = serde_json::to_string(&strategy).unwrap();
        let deserialized: RetrievalStrategy = serde_json::from_str(&json).unwrap();

        assert_eq!(strategy, deserialized);
    }

    #[test]
    fn test_route_decision_creation() {
        let mut weights = HashMap::new();
        weights.insert(RetrievalStrategy::Embedding, 0.9);
        weights.insert(RetrievalStrategy::BM25, 0.8);

        let decision = RouteDecision {
            selected_strategies: vec![RetrievalStrategy::Embedding, RetrievalStrategy::BM25],
            target_memory_types: vec![MemoryType::Episodic, MemoryType::Semantic],
            strategy_weights: weights,
            confidence: 0.85,
            reasoning: vec!["High semantic similarity".to_string()],
            estimated_performance: PerformanceEstimate {
                estimated_response_time_ms: 100,
                estimated_accuracy: 0.85,
                estimated_recall: 0.9,
                estimated_resource_usage: 0.5,
            },
        };

        assert_eq!(decision.selected_strategies.len(), 2);
        assert_eq!(decision.confidence, 0.85);
        assert_eq!(decision.reasoning.len(), 1);
    }

    #[test]
    fn test_route_decision_serialization() {
        let decision = RouteDecision {
            selected_strategies: vec![RetrievalStrategy::Embedding],
            target_memory_types: vec![MemoryType::Episodic],
            strategy_weights: HashMap::new(),
            confidence: 0.9,
            reasoning: vec!["Test reason".to_string()],
            estimated_performance: PerformanceEstimate {
                estimated_response_time_ms: 50,
                estimated_accuracy: 0.9,
                estimated_recall: 0.95,
                estimated_resource_usage: 0.3,
            },
        };

        let json = serde_json::to_string(&decision).unwrap();
        let deserialized: RouteDecision = serde_json::from_str(&json).unwrap();

        assert_eq!(
            decision.selected_strategies,
            deserialized.selected_strategies
        );
        assert_eq!(decision.confidence, deserialized.confidence);
    }

    #[test]
    fn test_performance_estimate_creation() {
        let estimate = PerformanceEstimate {
            estimated_response_time_ms: 200,
            estimated_accuracy: 0.82,
            estimated_recall: 0.88,
            estimated_resource_usage: 0.7,
        };

        assert_eq!(estimate.estimated_response_time_ms, 200);
        assert_eq!(estimate.estimated_recall, 0.88);
        assert_eq!(estimate.estimated_accuracy, 0.82);
        assert_eq!(estimate.estimated_resource_usage, 0.7);
    }

    #[test]
    fn test_performance_estimate_serialization() {
        let estimate = PerformanceEstimate {
            estimated_response_time_ms: 150,
            estimated_accuracy: 0.9,
            estimated_recall: 0.85,
            estimated_resource_usage: 0.6,
        };

        let json = serde_json::to_string(&estimate).unwrap();
        let deserialized: PerformanceEstimate = serde_json::from_str(&json).unwrap();

        assert_eq!(
            estimate.estimated_response_time_ms,
            deserialized.estimated_response_time_ms
        );
        assert_eq!(estimate.estimated_accuracy, deserialized.estimated_accuracy);
    }

    #[test]
    fn test_route_decision_with_empty_strategies() {
        let decision = RouteDecision {
            selected_strategies: vec![],
            target_memory_types: vec![],
            strategy_weights: HashMap::new(),
            confidence: 0.0,
            reasoning: vec![],
            estimated_performance: PerformanceEstimate {
                estimated_response_time_ms: 0,
                estimated_accuracy: 0.0,
                estimated_recall: 0.0,
                estimated_resource_usage: 0.0,
            },
        };

        assert!(decision.selected_strategies.is_empty());
        assert!(decision.target_memory_types.is_empty());
        assert_eq!(decision.confidence, 0.0);
    }

    #[test]
    fn test_route_decision_with_multiple_strategies() {
        let strategies = vec![
            RetrievalStrategy::Embedding,
            RetrievalStrategy::BM25,
            RetrievalStrategy::SemanticGraph,
            RetrievalStrategy::ContextAware,
        ];

        let decision = RouteDecision {
            selected_strategies: strategies.clone(),
            target_memory_types: vec![MemoryType::Episodic],
            strategy_weights: HashMap::new(),
            confidence: 0.75,
            reasoning: vec!["Multiple strategies for better coverage".to_string()],
            estimated_performance: PerformanceEstimate {
                estimated_response_time_ms: 300,
                estimated_accuracy: 0.88,
                estimated_recall: 0.92,
                estimated_resource_usage: 0.8,
            },
        };

        assert_eq!(decision.selected_strategies.len(), 4);
        assert!(decision
            .selected_strategies
            .contains(&RetrievalStrategy::Embedding));
    }

    #[test]
    fn test_strategy_weights_calculation() {
        let embedding_weight = RetrievalStrategy::Embedding.weight();
        let bm25_weight = RetrievalStrategy::BM25.weight();
        let hybrid_weight = RetrievalStrategy::Hybrid.weight();

        assert!(hybrid_weight >= embedding_weight);
        assert!(embedding_weight > bm25_weight);
    }
}
