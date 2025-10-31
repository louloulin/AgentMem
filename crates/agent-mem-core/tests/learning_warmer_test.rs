//! 学习驱动的缓存预热测试

#[cfg(test)]
mod warmer_tests {
    use agent_mem_core::cache::{LearningBasedCacheWarmer, LearningWarmingConfig};
    use agent_mem_core::search::{
        LearningConfig, LearningEngine, QueryFeatures, SearchWeights,
    };
    use std::sync::Arc;
    use std::time::Duration;
    
    async fn create_test_learning_engine() -> Arc<LearningEngine> {
        let config = LearningConfig {
            min_samples: 5,
            ..Default::default()
        };
        Arc::new(LearningEngine::new(config))
    }
    
    #[tokio::test]
    async fn test_warmer_creation() {
        let learning_engine = create_test_learning_engine().await;
        let config = LearningWarmingConfig::default();
        
        let warmer = LearningBasedCacheWarmer::new(learning_engine, config);
        
        // 验证统计初始化
        let stats = warmer.get_stats().await;
        assert_eq!(stats.total_warmings, 0);
        assert_eq!(stats.total_items_warmed, 0);
    }
    
    #[tokio::test]
    async fn test_warmer_with_learning_data() {
        let learning_engine = create_test_learning_engine().await;
        
        // 模拟一些学习数据
        for _ in 0..15 {
            let features = QueryFeatures {
                has_exact_terms: true,
                semantic_complexity: 0.3,
                has_temporal_indicator: false,
                entity_count: 2,
                query_length: 30,
                is_question: false,
            };
            
            let weights = SearchWeights {
                vector_weight: 0.3,
                fulltext_weight: 0.7,
                confidence: 0.9,
            };
            
            learning_engine
                .record_feedback(features, weights, 0.85, None)
                .await;
        }
        
        // 创建预热器
        let config = LearningWarmingConfig {
            top_patterns: 5,
            min_query_count: 10,
            min_effectiveness: 0.7,
            ..Default::default()
        };
        
        let warmer = LearningBasedCacheWarmer::new(learning_engine.clone(), config);
        
        // 模拟预热（使用简单的计数器）
        let warmed_count = Arc::new(tokio::sync::RwLock::new(0_u64));
        let warmed_count_clone = warmed_count.clone();
        
        let stats = warmer
            .warm_cache_with_engine(move |_features| {
                let count = warmed_count_clone.clone();
                async move {
                    let mut c = count.write().await;
                    *c += 1;
                    Ok(())
                }
            })
            .await
            .unwrap();
        
        // 验证预热结果
        let final_count = *warmed_count.read().await;
        assert!(final_count > 0, "Should have warmed some queries");
        assert_eq!(stats.total_warmings, 1);
        assert_eq!(stats.total_items_warmed, final_count);
    }
    
    #[tokio::test]
    async fn test_warmer_config() {
        let config = LearningWarmingConfig {
            top_patterns: 10,
            queries_per_pattern: 5,
            min_query_count: 20,
            min_effectiveness: 0.8,
            warming_interval: Duration::from_secs(600),
            warm_on_startup: false,
        };
        
        assert_eq!(config.top_patterns, 10);
        assert_eq!(config.queries_per_pattern, 5);
        assert_eq!(config.min_query_count, 20);
        assert_eq!(config.min_effectiveness, 0.8);
    }
    
    #[tokio::test]
    async fn test_warmer_stats_reset() {
        let learning_engine = create_test_learning_engine().await;
        let warmer = LearningBasedCacheWarmer::new(
            learning_engine,
            LearningWarmingConfig::default(),
        );
        
        // 重置统计
        warmer.reset_stats().await;
        
        let stats = warmer.get_stats().await;
        assert_eq!(stats.total_warmings, 0);
    }
}

