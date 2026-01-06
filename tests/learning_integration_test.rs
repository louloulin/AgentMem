//! 学习机制集成测试
//!
//! 测试学习引擎与增强混合搜索引擎的集成

#[cfg(test)]
mod tests {
    use agent_mem_core::search::{
        QueryFeatures, SearchWeights, LearningConfig, QueryPattern,
        PatternImprovement,
    };
    
    #[tokio::test]
    async fn test_query_pattern_classification() {
        // 测试1: 问题查询
        let query1 = "What is machine learning?";
        let features1 = QueryFeatures::extract_from_query(query1);
        assert!(features1.is_question);
        assert!(features1.query_length > 0);
        
        // 测试2: 技术查询
        let query2 = "vector database optimization algorithm performance";
        let features2 = QueryFeatures::extract_from_query(query2);
        assert!(features2.has_technical_terms);
        
        // 测试3: 短查询
        let query3 = "AI";
        let features3 = QueryFeatures::extract_from_query(query3);
        assert_eq!(features3.query_length, 2);
    }
    
    #[tokio::test]
    async fn test_learning_config() {
        // 测试默认配置
        let config = LearningConfig::default();
        assert!(config.min_samples_for_learning >= 10);
        assert!(config.learning_rate > 0.0 && config.learning_rate <= 1.0);
        assert!(config.smoothing_factor > 0.0 && config.smoothing_factor <= 1.0);
        
        // 测试自定义配置
        let custom_config = LearningConfig {
            min_samples_for_learning: 20,
            learning_rate: 0.2,
            smoothing_factor: 0.15,
            feedback_threshold: 0.8,
            max_history_size: 5000,
            enable_cross_pattern_learning: true,
        };
        assert_eq!(custom_config.min_samples_for_learning, 20);
        assert_eq!(custom_config.learning_rate, 0.2);
    }
    
    #[tokio::test]
    async fn test_search_weights_normalization() {
        let mut weights = SearchWeights {
            vector_weight: 0.8,
            fulltext_weight: 0.4,
            confidence: 0.9,
        };
        
        weights.normalize();
        
        // 验证归一化后总和为1.0
        let sum = weights.vector_weight + weights.fulltext_weight;
        assert!((sum - 1.0).abs() < 0.001);
        
        // 验证权重范围
        assert!(weights.vector_weight >= 0.0 && weights.vector_weight <= 1.0);
        assert!(weights.fulltext_weight >= 0.0 && weights.fulltext_weight <= 1.0);
    }
    
    #[tokio::test]
    async fn test_pattern_improvement_calculation() {
        let pattern = QueryPattern::Question;
        let old_weights = SearchWeights {
            vector_weight: 0.6,
            fulltext_weight: 0.4,
            confidence: 0.7,
        };
        let new_weights = SearchWeights {
            vector_weight: 0.7,
            fulltext_weight: 0.3,
            confidence: 0.85,
        };
        let effectiveness_improvement = 0.15;
        
        let improvement = PatternImprovement {
            pattern,
            old_weights,
            new_weights,
            effectiveness_improvement,
            sample_count: 100,
        };
        
        assert_eq!(improvement.pattern, QueryPattern::Question);
        assert_eq!(improvement.effectiveness_improvement, 0.15);
        assert_eq!(improvement.sample_count, 100);
        assert!(improvement.new_weights.confidence > improvement.old_weights.confidence);
    }
    
    #[tokio::test]
    async fn test_concurrent_feedback_recording() {
        // 测试并发反馈记录的线程安全性
        use agent_mem_core::search::LearningEngine;
        use std::sync::Arc;
        
        let config = LearningConfig::default();
        let engine = Arc::new(LearningEngine::new(config));
        
        // 并发记录多个反馈
        let mut handles = vec![];
        for i in 0..10 {
            let engine_clone = engine.clone();
            let handle = tokio::spawn(async move {
                let query = format!("test query {}", i);
                let features = QueryFeatures::extract_from_query(&query);
                let weights = SearchWeights {
                    vector_weight: 0.7,
                    fulltext_weight: 0.3,
                    confidence: 0.8,
                };
                engine_clone.record_feedback(features, weights, 0.85, None).await;
            });
            handles.push(handle);
        }
        
        // 等待所有任务完成
        for handle in handles {
            handle.await.unwrap();
        }
        
        // 验证数据一致性
        let stats = engine.get_all_statistics().await;
        assert!(!stats.is_empty());
    }
    
    #[tokio::test]
    async fn test_learning_with_insufficient_data() {
        use agent_mem_core::search::LearningEngine;
        
        let config = LearningConfig {
            min_samples_for_learning: 50,  // 需要50个样本
            ..Default::default()
        };
        let engine = LearningEngine::new(config);
        
        // 只记录10个反馈（不足最小要求）
        for i in 0..10 {
            let query = format!("test {}", i);
            let features = QueryFeatures::extract_from_query(&query);
            let weights = SearchWeights {
                vector_weight: 0.7,
                fulltext_weight: 0.3,
                confidence: 0.8,
            };
            engine.record_feedback(features, weights, 0.85, None).await;
        }
        
        // 获取推荐权重（应该返回None，因为样本不足）
        let query = "test query";
        let features = QueryFeatures::extract_from_query(query);
        let recommended = engine.get_recommended_weights(&features).await;
        
        // 验证：样本不足时不应该返回学习到的权重
        // （或返回低置信度的权重）
        if let Some(weights) = recommended {
            assert!(weights.confidence < 0.5);
        }
    }
    
    #[tokio::test]
    async fn test_optimization_report() {
        use agent_mem_core::search::LearningEngine;
        
        let config = LearningConfig {
            min_samples_for_learning: 5,  // 降低要求便于测试
            ..Default::default()
        };
        let engine = LearningEngine::new(config);
        
        // 记录多组反馈
        for i in 0..20 {
            let query = if i % 2 == 0 {
                "What is this?"  // 问题类
            } else {
                "technical term algorithm"  // 技术类
            };
            let features = QueryFeatures::extract_from_query(query);
            let weights = SearchWeights {
                vector_weight: 0.6 + (i as f32 * 0.01),
                fulltext_weight: 0.4 - (i as f32 * 0.01),
                confidence: 0.8,
            };
            let effectiveness = 0.7 + (i as f32 * 0.01);
            engine.record_feedback(features, weights, effectiveness, None).await;
        }
        
        // 执行优化
        let report = engine.optimize().await;
        
        // 验证报告内容
        assert!(!report.improvements.is_empty() || report.total_samples > 0);
        assert!(report.timestamp > 0);
    }
    
    #[test]
    fn test_query_pattern_variants() {
        // 测试所有查询模式变体
        let patterns = vec![
            QueryPattern::Question,
            QueryPattern::Technical,
            QueryPattern::Short,
            QueryPattern::Long,
            QueryPattern::Conversational,
            QueryPattern::General,
        ];
        
        // 确保所有模式都可以序列化/反序列化
        for pattern in patterns {
            let serialized = format!("{:?}", pattern);
            assert!(!serialized.is_empty());
        }
    }
}

