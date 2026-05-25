//! 混合搜索模块
//!
//! 提供向量搜索 + 全文搜索的混合搜索系统，包括：
//! - 向量语义搜索
//! - 全文关键词搜索
//! - BM25 算法搜索
//! - 模糊匹配搜索
//! - RRF (Reciprocal Rank Fusion) 融合算法
//! - 搜索结果重排序
//! - 搜索性能优化

pub mod adaptive;
/// Week 5-6: Adaptive router with Thompson Sampling
pub mod adaptive_router;
/// Week 5-6: Adaptive search engine (complete integration)
pub mod adaptive_search_engine;
pub mod adaptive_threshold;
pub mod bm25;
/// Week 7-8: Cached adaptive engine with parallel search
pub mod cached_adaptive_engine;
// Removed: cached_vector_search - functionality integrated into enhanced_hybrid_v2
// Removed: enhanced_hybrid - replaced by enhanced_hybrid_v2
pub mod enhanced_hybrid_v2;
/// 外部重排序器（Cohere等）
pub mod external_reranker;
#[cfg(feature = "postgres")]
pub mod fulltext_search;
pub mod fuzzy;
pub mod hnsw_optimizer;
#[cfg(feature = "postgres")]
pub mod hybrid;
pub mod learning;
/// 元数据过滤系统（阶段2实现）
pub mod metadata_filter;
pub mod query_classifier;
pub mod query_optimizer;
pub mod ranker;
pub mod reranker;
pub mod vector_search;
/// Week 11-13: Enhanced search with category/resource awareness
pub mod category_recall;
pub mod resource_recall;
pub mod sufficiency_check;
pub mod enhanced_v4;

pub use adaptive::{
    AdaptiveSearchOptimizer, QueryFeatures, SearchReranker, SearchWeights, WeightPredictor,
};
pub use adaptive_threshold::{
    AdaptiveThresholdCalculator, AdaptiveThresholdConfig, ThresholdCalculation,
};
pub use bm25::{BM25Params, BM25SearchEngine};
// Removed: CachedVectorSearchEngine - functionality integrated into EnhancedHybridSearchEngineV2
// Removed: EnhancedHybridSearchEngine - replaced by EnhancedHybridSearchEngineV2
pub use enhanced_hybrid_v2::{
    EnhancedHybridConfig, EnhancedHybridSearchEngine as EnhancedHybridSearchEngineV2,
    EnhancedSearchResult, EnhancedSearchStats,
};
pub use external_reranker::{
    InternalReranker, LLMReranker, LLMRerankerConfig, Reranker, RerankerFactory,
    RerankCacheStats, CachedReranker,
};
#[cfg(feature = "postgres")]
pub use fulltext_search::FullTextSearchEngine;
pub use fuzzy::{FuzzyMatchEngine, FuzzyMatchParams};
#[cfg(feature = "postgres")]
pub use hybrid::{HybridSearchConfig, HybridSearchEngine, HybridSearchResult};
pub use learning::{
    FeedbackRecord, LearningConfig, LearningEngine, OptimizationReport, PatternImprovement,
    QueryPattern,
};
pub use query_classifier::{
    QueryClassifier, QueryFeatures as QueryClassifierFeatures, QueryType,
    SearchStrategy as QuerySearchStrategy,
};
pub use query_optimizer::{IndexStatistics, QueryOptimizer, ResultReranker};
pub use ranker::{RRFRanker, SearchResultRanker};
pub use vector_search::{
    build_hybrid_vector_search_sql, build_vector_search_sql, VectorDistanceOperator,
    VectorSearchEngine,
};
// Week 11-13: Enhanced search exports
pub use category_recall::{
    CategoryFilter, CategoryRecallConfig, CategoryRecallEngine, CategoryRecallResult,
    CategorySearchResult, CategoryScope, InMemoryCategoryRecall,
};
pub use resource_recall::{
    ResourceContext, ResourceRecallConfig, ResourceRecallEngine, ResourceRecallResult,
    ResourceType, InMemoryResourceRecall,
};
pub use sufficiency_check::{
    SufficiencyAction, SufficiencyCheckResult, SufficiencyCheckType, SufficiencyChecker,
    SufficiencyConfig, SufficiencyContext, RuleBasedSufficiencyChecker,
};
pub use enhanced_v4::{
    EnhancedQueryType, EnhancedSearchV4, EnhancedSearchV4Config, EnhancedSearchV4Result,
    EnhancedSearchV4Stats,
};

use agent_mem_traits::{
    AttributeValue, ComparisonOperator, Constraint, Query, QueryIntent,
};
pub use metadata_filter::{
    FilterOperator, FilterValue, LogicalOperator, MetadataFilter, MetadataFilterSystem,
};
use serde::{Deserialize, Serialize};

/// 搜索查询
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    /// 查询文本
    pub query: String,
    /// 最大结果数
    pub limit: usize,
    /// 最小相似度阈值 (0.0 - 1.0)
    pub threshold: Option<f32>,
    /// 向量搜索权重 (0.0 - 1.0)
    pub vector_weight: f32,
    /// 全文搜索权重 (0.0 - 1.0)
    pub fulltext_weight: f32,
    /// 过滤条件
    pub filters: Option<SearchFilters>,
    /// 元数据过滤条件（阶段2：高级过滤）
    pub metadata_filters: Option<LogicalOperator>,
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            query: String::new(),
            limit: 10,
            threshold: Some(0.3), // 🔧 降低阈值以支持商品ID等精确查询
            vector_weight: 0.7,
            fulltext_weight: 0.3,
            filters: None,
            metadata_filters: None,
        }
    }
}

impl SearchQuery {
    /// 从 Query V4 转换到 SearchQuery（向后兼容）
    pub fn from_query_v4(query: &Query) -> Self {
        // 提取查询文本
        let query_text = match &query.intent {
            QueryIntent::NaturalLanguage { text, .. } => text.clone(),
            QueryIntent::Vector { .. } => String::new(), // 向量查询没有文本
            QueryIntent::Structured { .. } => String::new(), // 结构化查询需要特殊处理
            QueryIntent::Hybrid { intents, .. } => {
                // 从混合查询中提取第一个自然语言查询
                intents
                    .iter()
                    .find_map(|intent| {
                        if let QueryIntent::NaturalLanguage { text, .. } = intent {
                            Some(text.clone())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default()
            }
        };

        // 提取限制
        let limit = query
            .constraints
            .iter()
            .find_map(|c| {
                if let Constraint::Attribute {
                    key,
                    operator,
                    value,
                } = c
                {
                    if key.name == "limit" && matches!(operator, ComparisonOperator::Equals) {
                        if let AttributeValue::Number(n) = value {
                            return Some(*n as usize);
                        }
                    }
                }
                None
            })
            .unwrap_or(10);

        // 提取阈值
        let threshold = query.constraints.iter().find_map(|c| {
            if let Constraint::Attribute {
                key,
                operator,
                value,
            } = c
            {
                if key.name == "threshold" && matches!(operator, ComparisonOperator::GreaterOrEqual)
                {
                    if let AttributeValue::Number(n) = value {
                        return Some(*n as f32);
                    }
                }
            }
            None
        });

        // 提取过滤条件
        let filters = Self::extract_filters(query);

        Self {
            query: query_text,
            limit,
            threshold,
            vector_weight: 0.7, // 默认权重
            fulltext_weight: 0.3,
            filters,
            metadata_filters: None,
        }
    }

    /// 从 Query V4 约束中提取过滤条件
    fn extract_filters(query: &Query) -> Option<SearchFilters> {
        let mut user_id = None;
        let mut agent_id = None;
        let mut organization_id = None;
        let mut start_time = None;
        let mut end_time = None;
        let mut tags = None;

        for constraint in &query.constraints {
            match constraint {
                Constraint::Attribute {
                    key,
                    operator,
                    value,
                } => {
                    if matches!(operator, ComparisonOperator::Equals) {
                        match key.name.as_str() {
                            "user_id" => {
                                if let AttributeValue::String(s) = value {
                                    user_id = Some(s.clone());
                                }
                            }
                            "agent_id" => {
                                if let AttributeValue::String(s) = value {
                                    agent_id = Some(s.clone());
                                }
                            }
                            "organization_id" => {
                                if let AttributeValue::String(s) = value {
                                    organization_id = Some(s.clone());
                                }
                            }
                            "tags" => {
                                if let AttributeValue::List(arr) = value {
                                    tags = Some(
                                        arr.iter()
                                            .filter_map(|v| {
                                                if let AttributeValue::String(s) = v {
                                                    Some(s.clone())
                                                } else {
                                                    None
                                                }
                                            })
                                            .collect(),
                                    );
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Constraint::Temporal { time_range } => {
                    start_time = time_range.start;
                    end_time = time_range.end;
                }
                _ => {}
            }
        }

        if user_id.is_some()
            || agent_id.is_some()
            || organization_id.is_some()
            || start_time.is_some()
            || end_time.is_some()
            || tags.is_some()
        {
            Some(SearchFilters {
                user_id,
                organization_id,
                agent_id,
                start_time,
                end_time,
                tags,
            })
        } else {
            None
        }
    }
}

/// 搜索过滤条件
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchFilters {
    /// 用户 ID 过滤
    pub user_id: Option<String>,
    /// 组织 ID 过滤
    pub organization_id: Option<String>,
    /// Agent ID 过滤
    pub agent_id: Option<String>,
    /// 时间范围过滤 (开始时间)
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    /// 时间范围过滤 (结束时间)
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    /// 标签过滤
    pub tags: Option<Vec<String>>,
}

/// 搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// 记忆 ID
    pub id: String,
    /// 记忆内容
    pub content: String,
    /// 相似度分数 (0.0 - 1.0)
    pub score: f32,
    /// 向量搜索分数
    pub vector_score: Option<f32>,
    /// 全文搜索分数
    pub fulltext_score: Option<f32>,
    /// 元数据
    pub metadata: Option<serde_json::Value>,
}

/// 搜索统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchStats {
    /// 总搜索时间 (毫秒)
    pub total_time_ms: u64,
    /// 向量搜索时间 (毫秒)
    pub vector_search_time_ms: u64,
    /// 全文搜索时间 (毫秒)
    pub fulltext_search_time_ms: u64,
    /// 融合时间 (毫秒)
    pub fusion_time_ms: u64,
    /// 向量搜索结果数
    pub vector_results_count: usize,
    /// 全文搜索结果数
    pub fulltext_results_count: usize,
    /// 最终结果数
    pub final_results_count: usize,
}

#[cfg(test)]
#[cfg(feature = "inline_tests")]
mod tests {
    use super::*;

    #[test]
    fn test_search_query_default() {
        let query = SearchQuery::default();
        assert_eq!(query.limit, 10);
        assert_eq!(query.threshold, Some(0.3)); // 🔧 更新测试以匹配新的默认阈值
        assert_eq!(query.vector_weight, 0.7);
        assert_eq!(query.fulltext_weight, 0.3);
    }

    #[test]
    fn test_search_result() {
        let result = SearchResult {
            id: "test-id".to_string(),
            content: "test content".to_string(),
            score: 0.9,
            vector_score: Some(0.85),
            fulltext_score: Some(0.95),
            metadata: None,
        };
        assert_eq!(result.id, "test-id");
        assert_eq!(result.score, 0.9);
    }
}
