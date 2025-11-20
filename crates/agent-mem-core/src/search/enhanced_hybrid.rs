//! 增强的混合搜索引擎
//!
//! 在原有混合搜索基础上增加自适应权重调整、结果重排序和机器学习

use super::adaptive::{AdaptiveSearchOptimizer, QueryFeatures, SearchReranker, SearchWeights};
use super::hybrid::HybridSearchEngine;
use super::learning::{LearningConfig, LearningEngine, OptimizationReport, QueryPattern};
use super::{SearchQuery, SearchResult};
use agent_mem_traits::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 增强混合搜索引擎
pub struct EnhancedHybridSearchEngine {
    /// 原有的混合搜索引擎
    base_engine: Arc<HybridSearchEngine>,

    /// 自适应优化器
    optimizer: Arc<RwLock<AdaptiveSearchOptimizer>>,

    /// 重排序器
    reranker: Arc<SearchReranker>,

    /// 学习引擎（可选）
    learning_engine: Option<Arc<LearningEngine>>,

    /// 是否启用自适应权重
    enable_adaptive_weights: bool,

    /// 是否启用重排序
    enable_reranking: bool,

    /// 是否启用机器学习
    enable_learning: bool,
}

impl EnhancedHybridSearchEngine {
    /// 创建新的增强混合搜索引擎（基础版本，不启用学习）
    pub fn new(
        base_engine: Arc<HybridSearchEngine>,
        enable_adaptive_weights: bool,
        enable_reranking: bool,
    ) -> Self {
        Self {
            base_engine,
            optimizer: Arc::new(RwLock::new(AdaptiveSearchOptimizer::default())),
            reranker: Arc::new(SearchReranker::default()),
            learning_engine: None,
            enable_adaptive_weights,
            enable_reranking,
            enable_learning: false,
        }
    }

    /// 创建带学习功能的增强搜索引擎
    pub fn with_learning(
        base_engine: Arc<HybridSearchEngine>,
        enable_adaptive_weights: bool,
        enable_reranking: bool,
        learning_config: Option<LearningConfig>,
    ) -> Self {
        let learning_engine = Arc::new(LearningEngine::new(learning_config.unwrap_or_default()));

        Self {
            base_engine,
            optimizer: Arc::new(RwLock::new(AdaptiveSearchOptimizer::default())),
            reranker: Arc::new(SearchReranker::default()),
            learning_engine: Some(learning_engine),
            enable_adaptive_weights,
            enable_reranking,
            enable_learning: true,
        }
    }

    /// 创建带学习和持久化功能的增强搜索引擎
    #[cfg(feature = "libsql")]
    pub async fn with_learning_and_persistence(
        base_engine: Arc<HybridSearchEngine>,
        enable_adaptive_weights: bool,
        enable_reranking: bool,
        learning_config: Option<LearningConfig>,
        repository: Arc<dyn crate::storage::libsql::LearningRepositoryTrait>,
    ) -> Result<Self> {
        let learning_engine = Arc::new(LearningEngine::with_persistence(
            learning_config.unwrap_or_default(),
            repository,
        ));

        // 从存储加载历史数据
        learning_engine.load_from_storage().await?;

        Ok(Self {
            base_engine,
            optimizer: Arc::new(RwLock::new(AdaptiveSearchOptimizer::default())),
            reranker: Arc::new(SearchReranker::default()),
            learning_engine: Some(learning_engine),
            enable_adaptive_weights,
            enable_reranking,
            enable_learning: true,
        })
    }

    /// 执行增强搜索
    pub async fn search(
        &self,
        query_vector: Vec<f32>,
        query: SearchQuery,
    ) -> Result<Vec<SearchResult>> {
        // 步骤1: 确定搜索权重
        let weights = self.determine_weights(&query).await;

        // 应用权重到查询
        let mut optimized_query = query.clone();
        optimized_query.vector_weight = weights.vector_weight;
        optimized_query.fulltext_weight = weights.fulltext_weight;

        // 步骤2: 执行基础混合搜索（使用确定的权重）
        let result = self
            .base_engine
            .search_with_weights(
                query_vector,
                optimized_query.clone(),
                weights.vector_weight,
                weights.fulltext_weight,
            )
            .await?;

        let mut results = result.results;

        // 步骤3: 结果重排序（如果启用）
        if self.enable_reranking {
            results = self.reranker.rerank(results, &optimized_query);
        }

        Ok(results)
    }

    /// 确定搜索权重（综合规则和学习）
    async fn determine_weights(&self, query: &SearchQuery) -> SearchWeights {
        let features = QueryFeatures::extract_from_query(&query.query);

        // 优先使用学习到的权重
        if self.enable_learning {
            if let Some(learning_engine) = &self.learning_engine {
                if let Some(learned_weights) =
                    learning_engine.get_recommended_weights(&features).await
                {
                    return learned_weights;
                }
            }
        }

        // 其次使用规则预测的权重
        if self.enable_adaptive_weights {
            let optimizer = self.optimizer.read().await;
            let (_optimized_query, predicted_weights) = optimizer.optimize_query(query);
            return predicted_weights;
        }

        // 最后使用默认权重
        SearchWeights {
            vector_weight: query.vector_weight,
            fulltext_weight: query.fulltext_weight,
            confidence: 0.5,
        }
    }

    /// 记录搜索反馈（用于持续优化和学习）
    pub async fn record_feedback(
        &self,
        query: &str,
        weights: SearchWeights,
        user_satisfaction: f32,
    ) {
        // 1. 记录到优化器（保持向后兼容）
        let mut optimizer = self.optimizer.write().await;
        optimizer.record_feedback(query, weights, user_satisfaction);
        drop(optimizer);

        // 2. 如果启用学习，记录到学习引擎
        if self.enable_learning {
            if let Some(learning_engine) = &self.learning_engine {
                let features = QueryFeatures::extract_from_query(query);
                learning_engine
                    .record_feedback(
                        features,
                        weights,
                        user_satisfaction,
                        None, // user_id可以后续添加
                    )
                    .await;
            }
        }
    }

    /// 获取优化报告（从学习引擎）
    pub async fn get_optimization_report(&self) -> Option<OptimizationReport> {
        if let Some(learning_engine) = &self.learning_engine {
            Some(learning_engine.optimize().await)
        } else {
            None
        }
    }

    /// 获取学习统计信息
    pub async fn get_learning_stats(
        &self,
    ) -> Option<HashMap<QueryPattern, super::learning::PatternStatistics>> {
        if let Some(learning_engine) = &self.learning_engine {
            Some(learning_engine.get_all_statistics().await)
        } else {
            None
        }
    }

    /// 检查学习功能是否启用
    pub fn is_learning_enabled(&self) -> bool {
        self.enable_learning
    }
}

// ============================================================================
// SearchEngine Trait 实现 (V4)
// ============================================================================

use agent_mem_traits::{Query, QueryIntent, QueryIntentType, SearchEngine};
use async_trait::async_trait;

#[async_trait]
impl SearchEngine for EnhancedHybridSearchEngine {
    /// 执行搜索查询（V4 Query 接口）
    async fn search(&self, query: &Query) -> Result<Vec<agent_mem_traits::SearchResultV4>> {
        // 1. 提取查询向量和文本
        let (query_vector, query_text) = match &query.intent {
            QueryIntent::Hybrid { intents, .. } => {
                // 从混合查询中提取向量和文本
                let vector = intents.iter().find_map(|intent| {
                    if let QueryIntent::Vector { embedding } = intent {
                        Some(embedding.clone())
                    } else {
                        None
                    }
                });

                let text = intents
                    .iter()
                    .find_map(|intent| {
                        if let QueryIntent::NaturalLanguage { text, .. } = intent {
                            Some(text.clone())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default();

                let vector = vector.ok_or_else(|| {
                    agent_mem_traits::AgentMemError::validation_error(
                        "Hybrid query must contain at least one Vector intent",
                    )
                })?;

                (vector, text)
            }
            QueryIntent::Vector { embedding } => {
                // 纯向量查询
                (embedding.clone(), String::new())
            }
            QueryIntent::NaturalLanguage { text, .. } => {
                // 纯文本查询需要先生成 embedding
                return Err(agent_mem_traits::AgentMemError::validation_error(
                    "EnhancedHybridSearchEngine requires pre-computed embedding for vector search. Use QueryIntent::Hybrid with both Vector and NaturalLanguage intents."
                ));
            }
            _ => {
                return Err(agent_mem_traits::AgentMemError::validation_error(format!(
                    "Unsupported query intent for EnhancedHybridSearchEngine: {:?}",
                    query.intent
                )));
            }
        };

        // 2. 转换 Query V4 到 SearchQuery
        let mut search_query = SearchQuery::from_query_v4(query);
        // 确保查询文本被设置
        if !query_text.is_empty() {
            search_query.query = query_text;
        }

        // 3. 执行增强混合搜索（使用现有的 search 方法）
        let results = self.search(query_vector, search_query).await?;

        // 4. 转换 SearchResult 到 SearchResultV4
        let v4_results = results
            .into_iter()
            .map(|r| agent_mem_traits::SearchResultV4 {
                id: r.id,
                content: r.content,
                score: r.score,
                vector_score: r.vector_score,
                fulltext_score: r.fulltext_score,
                metadata: r.metadata,
            })
            .collect();

        Ok(v4_results)
    }

    /// 获取引擎名称
    fn name(&self) -> &str {
        "EnhancedHybridSearchEngine"
    }

    /// 获取支持的查询意图类型
    fn supported_intents(&self) -> Vec<QueryIntentType> {
        vec![
            QueryIntentType::Hybrid, // 主要支持混合查询
            QueryIntentType::Vector, // 也支持纯向量查询
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 注意：这些测试需要mock HybridSearchEngine
    // 实际测试将在集成测试中进行

    #[test]
    fn test_enhanced_engine_creation() {
        // 测试基础版本创建
        // let base_engine = Arc::new(/* mock engine */);
        // let engine = EnhancedHybridSearchEngine::new(base_engine, true, true);
        // assert!(!engine.is_learning_enabled());
    }

    #[test]
    fn test_enhanced_engine_with_learning() {
        // 测试带学习功能的版本创建
        // let base_engine = Arc::new(/* mock engine */);
        // let engine = EnhancedHybridSearchEngine::with_learning(base_engine, true, true, None);
        // assert!(engine.is_learning_enabled());
    }
}
