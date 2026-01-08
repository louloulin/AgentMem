//! Integration Test for Enhanced Hybrid Search
//!
//! 测试整个混合检索系统的集成

#[cfg(test)]
mod tests {
    use crate::search::{
        EnhancedHybridSearchEngineV2,
        EnhancedHybridConfig,
        QueryClassifier,
        QueryType,
        AdaptiveThresholdCalculator,
        SearchResult,
    };
    use std::sync::Arc;
    
    // 模拟向量搜索器
    struct MockVectorSearcher;
    
    #[async_trait::async_trait]
    impl super::super::enhanced_hybrid_v2::VectorSearcher for MockVectorSearcher {
        async fn search(&self, query: &str, limit: usize, threshold: f32) 
            -> agent_mem_traits::Result<Vec<SearchResult>> {
            // 模拟返回一些结果
            Ok(vec![
                SearchResult {
                    id: "vec1".to_string(),
                    content: format!("Vector result for: {}", query),
                    score: 0.9,
                    vector_score: Some(0.9),
                    fulltext_score: None,
                    metadata: None,
                },
                SearchResult {
                    id: "vec2".to_string(),
                    content: format!("Another vector result: {}", query),
                    score: 0.7,
                    vector_score: Some(0.7),
                    fulltext_score: None,
                    metadata: None,
                },
            ])
        }
    }
    
    // 模拟BM25搜索器
    struct MockBM25Searcher;
    
    #[async_trait::async_trait]
    impl super::super::enhanced_hybrid_v2::BM25Searcher for MockBM25Searcher {
        async fn search(&self, query: &str, limit: usize) 
            -> agent_mem_traits::Result<Vec<SearchResult>> {
            Ok(vec![
                SearchResult {
                    id: "bm25_1".to_string(),
                    content: format!("BM25 result for: {}", query),
                    score: 0.8,
                    vector_score: None,
                    fulltext_score: Some(0.8),
                    metadata: None,
                },
            ])
        }
    }
    
    // 模拟精确匹配器
    struct MockExactMatcher;
    
    #[async_trait::async_trait]
    impl super::super::enhanced_hybrid_v2::ExactMatcher for MockExactMatcher {
        async fn match_exact(&self, query: &str, limit: usize) 
            -> agent_mem_traits::Result<Vec<SearchResult>> {
            // 只对特定格式的查询返回结果
            if query.starts_with("P") && query.len() == 7 {
                Ok(vec![
                    SearchResult {
                        id: query.to_string(),
                        content: format!("Exact match: {}", query),
                        score: 1.0,
                        vector_score: None,
                        fulltext_score: None,
                        metadata: None,
                    },
                ])
            } else {
                Ok(vec![])
            }
        }
    }
    
    #[tokio::test]
    async fn test_enhanced_hybrid_search_exact_id() -> anyhow::Result<()> {
        let config = EnhancedHybridConfig::default();
        let engine = EnhancedHybridSearchEngineV2::new(config)
            .with_vector_searcher(Arc::new(MockVectorSearcher))
            .with_bm25_searcher(Arc::new(MockBM25Searcher))
            .with_exact_matcher(Arc::new(MockExactMatcher));
        
        let result = engine.search("P000001", 10).await?;
        
        // 应该分类为ExactId
        assert_eq!(result.query_type, QueryType::ExactId);
        
        // 应该使用精确匹配，返回1个结果
        assert_eq!(result.results.len(), 1);
        assert_eq!(result.results[0].id, "P000001");
        
        // 精确匹配应该很快
        assert!(result.stats.total_time_ms < 100);
    }
    
    #[tokio::test]
    async fn test_enhanced_hybrid_search_natural_language() {
        let config = EnhancedHybridConfig::default();
        let engine = EnhancedHybridSearchEngineV2::new(config)
            .with_vector_searcher(Arc::new(MockVectorSearcher))
            .with_bm25_searcher(Arc::new(MockBM25Searcher));
        
        let result = engine.search("推荐一款手机", 10).await?;
        
        // 应该分类为NaturalLanguage
        assert_eq!(result.query_type, QueryType::NaturalLanguage);
        
        // 应该融合向量和BM25结果
        assert!(result.results.len() > 0);
        assert!(result.stats.vector_results_count > 0);
        assert!(result.stats.bm25_results_count > 0);
    }
    
    #[tokio::test]
    async fn test_enhanced_hybrid_search_semantic() {
        let config = EnhancedHybridConfig::default();
        let engine = EnhancedHybridSearchEngineV2::new(config)
            .with_vector_searcher(Arc::new(MockVectorSearcher))
            .with_bm25_searcher(Arc::new(MockBM25Searcher));
        
        let result = engine.search(
            "What is the meaning of life, the universe, and everything?",
            10
        ).await?;
        
        // 应该分类为Semantic
        assert_eq!(result.query_type, QueryType::Semantic);
        
        // 应该主要使用向量搜索
        assert!(result.strategy.vector_weight > 0.7);
    }
    
    #[tokio::test]
    async fn test_adaptive_threshold() {
        let config = EnhancedHybridConfig {
            enable_adaptive_threshold: true,
            ..Default::default()
        Ok(())
        };
        let engine = EnhancedHybridSearchEngineV2::new(config)
            .with_vector_searcher(Arc::new(MockVectorSearcher));
        
        let short_query = engine.search("AI", 10).await?;
        let long_query = engine.search(
            "What is artificial intelligence and how does it work?",
            10
        ).await?;
        
        // 短查询应该有更低的阈值
        assert!(short_query.stats.threshold_used < long_query.stats.threshold_used);
    }
    
    #[tokio::test]
    async fn test_metrics_collection() -> anyhow::Result<()> {
        let config = EnhancedHybridConfig {
            enable_metrics: true,
            ..Default::default()
        Ok(())
        };
        let engine = EnhancedHybridSearchEngineV2::new(config)
            .with_vector_searcher(Arc::new(MockVectorSearcher));
        
        // 执行几次查询
        engine.search("test1", 10).await?;
        engine.search("test2", 10).await?;
        engine.search("test3", 10).await?;
        
        let metrics = engine.get_metrics().await;
        assert_eq!(metrics.total_queries, 3);
        assert!(metrics.avg_latency_ms > 0.0);
    }
    
    #[tokio::test]
    async fn test_parallel_search() -> anyhow::Result<()> {
        let config = EnhancedHybridConfig {
            enable_parallel: true,
            ..Default::default()
        Ok(())
        };
        let engine = EnhancedHybridSearchEngineV2::new(config)
            .with_vector_searcher(Arc::new(MockVectorSearcher))
            .with_bm25_searcher(Arc::new(MockBM25Searcher));
        
        let result = engine.search("test query", 10).await?;
        
        // 并行搜索应该比顺序搜索快
        assert!(result.stats.vector_search_time_ms > 0);
        assert!(result.stats.bm25_search_time_ms > 0);
    }
}

