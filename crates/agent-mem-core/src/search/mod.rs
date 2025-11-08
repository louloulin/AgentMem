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
pub mod adaptive_threshold;
/// Week 5-6: Adaptive router with Thompson Sampling
pub mod adaptive_router;
/// Week 5-6: Adaptive search engine (complete integration)
pub mod adaptive_search_engine;
pub mod bm25;
#[cfg(feature = "redis-cache")]
pub mod cached_vector_search;
#[cfg(feature = "postgres")]
pub mod enhanced_hybrid;
pub mod enhanced_hybrid_v2;
#[cfg(feature = "postgres")]
pub mod fulltext_search;
pub mod fuzzy;
#[cfg(feature = "postgres")]
pub mod hybrid;
pub mod learning;
pub mod query_classifier;
pub mod query_optimizer;
pub mod ranker;
pub mod vector_search;

pub use adaptive::{
    AdaptiveSearchOptimizer, QueryFeatures, SearchReranker, SearchWeights, WeightPredictor,
};
pub use adaptive_threshold::{AdaptiveThresholdCalculator, AdaptiveThresholdConfig, ThresholdCalculation};
pub use bm25::{BM25Params, BM25SearchEngine};
pub use enhanced_hybrid_v2::{
    EnhancedHybridSearchEngine as EnhancedHybridSearchEngineV2,
    EnhancedHybridConfig,
    EnhancedSearchResult,
    EnhancedSearchStats,
};
pub use query_classifier::{
    QueryClassifier,
    QueryType,
    SearchStrategy as QuerySearchStrategy,
    QueryFeatures as QueryClassifierFeatures,
};
#[cfg(feature = "redis-cache")]
pub use cached_vector_search::{CachedVectorSearchConfig, CachedVectorSearchEngine};
#[cfg(feature = "postgres")]
pub use enhanced_hybrid::EnhancedHybridSearchEngine;
#[cfg(feature = "postgres")]
pub use fulltext_search::FullTextSearchEngine;
pub use fuzzy::{FuzzyMatchEngine, FuzzyMatchParams};
#[cfg(feature = "postgres")]
pub use hybrid::{HybridSearchConfig, HybridSearchEngine, HybridSearchResult};
pub use learning::{
    FeedbackRecord, LearningConfig, LearningEngine, OptimizationReport, PatternImprovement,
    QueryPattern,
};
pub use query_optimizer::{IndexStatistics, QueryOptimizer, ResultReranker};
pub use ranker::{RRFRanker, SearchResultRanker};
pub use vector_search::{
    build_hybrid_vector_search_sql, build_vector_search_sql, VectorDistanceOperator,
    VectorSearchEngine,
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
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            query: String::new(),
            limit: 10,
            threshold: Some(0.3),  // 🔧 降低阈值以支持商品ID等精确查询
            vector_weight: 0.7,
            fulltext_weight: 0.3,
            filters: None,
        }
    }
}

/// 搜索过滤条件
#[derive(Debug, Clone, Serialize, Deserialize)]
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
mod tests {
    use super::*;

    #[test]
    fn test_search_query_default() {
        let query = SearchQuery::default();
        assert_eq!(query.limit, 10);
        assert_eq!(query.threshold, Some(0.3));  // 🔧 更新测试以匹配新的默认阈值
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
