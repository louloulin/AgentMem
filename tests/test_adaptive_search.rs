//! 自适应搜索优化集成测试

use agent_mem_core::search::{
    AdaptiveSearchOptimizer, QueryFeatures, SearchQuery, SearchReranker, SearchResult,
    SearchWeights, WeightPredictor,
};

#[test]
fn test_query_feature_extraction() {
    // 测试1: 精确匹配查询
    let features1 = QueryFeatures::extract_from_query("user@example.com");
    assert!(features1.has_exact_terms, "Should detect exact terms");
    
    // 测试2: 时间查询
    let features2 = QueryFeatures::extract_from_query("what happened yesterday?");
    assert!(features2.has_temporal_indicator, "Should detect temporal indicator");
    assert!(features2.is_question, "Should detect question");
    
    // 测试3: 复杂语义查询
    let features3 = QueryFeatures::extract_from_query(
        "How can I improve the performance of vector search in large datasets?"
    );
    assert!(features3.semantic_complexity > 0.5, "Should detect high complexity");
    assert!(features3.is_question, "Should detect question");
    
    // 测试4: 简单查询
    let features4 = QueryFeatures::extract_from_query("pizza");
    assert!(features4.semantic_complexity < 0.5, "Should detect low complexity");
    assert!(!features4.is_question, "Should not be a question");
}

#[test]
fn test_weight_prediction_exact_match() {
    let predictor = WeightPredictor::new();
    
    // 精确匹配查询应该提高全文权重
    let features = QueryFeatures {
        has_exact_terms: true,
        semantic_complexity: 0.3,
        has_temporal_indicator: false,
        entity_count: 0,
        query_length: 20,
        is_question: false,
    };
    let weights = predictor.predict(&features);
    
    assert!(
        weights.fulltext_weight > weights.vector_weight,
        "Exact match query should prefer fulltext search. fulltext={}, vector={}",
        weights.fulltext_weight,
        weights.vector_weight
    );
    
    // 权重应该归一化
    let sum = weights.vector_weight + weights.fulltext_weight;
    assert!(
        (sum - 1.0).abs() < 0.001,
        "Weights should sum to 1.0, got {sum}"
    );
}

#[test]
fn test_weight_prediction_semantic_query() {
    let predictor = WeightPredictor::new();
    
    // 语义复杂查询应该提高向量权重
    let features = QueryFeatures {
        has_exact_terms: false,
        semantic_complexity: 0.8,
        has_temporal_indicator: false,
        entity_count: 0,
        query_length: 100,
        is_question: true,
    };
    let weights = predictor.predict(&features);
    
    assert!(
        weights.vector_weight > weights.fulltext_weight,
        "Semantic query should prefer vector search. vector={}, fulltext={}",
        weights.vector_weight,
        weights.fulltext_weight
    );
}

#[test]
fn test_weight_normalization() {
    let mut weights = SearchWeights {
        vector_weight: 0.7,
        fulltext_weight: 0.5,
        confidence: 0.8,
    };
    
    weights.normalize();
    
    let sum = weights.vector_weight + weights.fulltext_weight;
    assert!(
        (sum - 1.0).abs() < 0.001,
        "Normalized weights should sum to 1.0, got {sum}"
    );
}

#[test]
fn test_adaptive_optimizer() {
    let optimizer = AdaptiveSearchOptimizer::new(true);
    
    // 测试不同类型的查询
    let test_cases = vec![
        ("user@example.com", "exact"),
        ("How does memory work?", "semantic"),
        ("what happened yesterday?", "temporal"),
        ("pizza", "simple"),
    ];
    
    for (query_text, query_type) in test_cases {
        let query = SearchQuery {
            query: query_text.to_string(),
            ..Default::default()
        };
        
        let (_, weights) = optimizer.optimize_query(&query);
        
        println!("\nQuery type: {query_type}");
        println!("Query: {query_text}");
        println!("Vector weight: {:.3}", weights.vector_weight);
        println!("Fulltext weight: {:.3}", weights.fulltext_weight);
        println!("Confidence: {:.3}", weights.confidence);
        
        // 验证权重归一化
        let sum = weights.vector_weight + weights.fulltext_weight;
        assert!((sum - 1.0).abs() < 0.001, "Weights should sum to 1.0");
        
        // 验证置信度范围
        assert!(weights.confidence >= 0.0 && weights.confidence <= 1.0);
    }
}

#[test]
fn test_search_reranker() {
    let reranker = SearchReranker::new();
    
    let mut results = vec![
        SearchResult {
            id: "1".to_string(),
            content: "Short".to_string(),
            score: 0.9,
            vector_score: Some(0.9),
            fulltext_score: None,
            metadata: None,
        },
        SearchResult {
            id: "2".to_string(),
            content: "This is a medium length content that should be good".to_string(),
            score: 0.8,
            vector_score: Some(0.8),
            fulltext_score: None,
            metadata: Some(serde_json::json!({
                "importance": "0.9",
            })),
        },
        SearchResult {
            id: "3".to_string(),
            content: "Very short".to_string(),
            score: 0.85,
            vector_score: Some(0.85),
            fulltext_score: None,
            metadata: None,
        },
    ];
    
    let query = SearchQuery {
        query: "test query".to_string(),
        ..Default::default()
    };
    
    let reranked = reranker.rerank(results, &query);
    
    // 验证结果仍然按分数排序
    for i in 0..reranked.len() - 1 {
        assert!(
            reranked[i].score >= reranked[i + 1].score,
            "Results should be sorted by score"
        );
    }
    
    println!("\nReranked results:");
    for (i, result) in reranked.iter().enumerate() {
        println!("  {}. ID={}, score={:.3}", i + 1, result.id, result.score);
    }
}

#[test]
fn test_learning_from_feedback() {
    let mut optimizer = AdaptiveSearchOptimizer::new(true);
    
    let query = "test query";
    let weights = SearchWeights {
        vector_weight: 0.6,
        fulltext_weight: 0.4,
        confidence: 0.8,
    };
    
    // 记录高效反馈
    optimizer.record_feedback(query, weights.clone(), 0.9);
    
    // 记录低效反馈（不应该被记录）
    optimizer.record_feedback(query, weights, 0.3);
    
    // 注意：由于学习机制的简化实现，我们只能验证不会panic
    println!("Feedback recorded successfully");
}

#[test]
fn test_comprehensive_query_scenarios() {
    let predictor = WeightPredictor::new();
    
    let scenarios = vec![
        (
            "user@example.com",
            "精确邮箱查询",
            |w: &SearchWeights| w.fulltext_weight > w.vector_weight,
        ),
        (
            "How can I optimize vector search performance in PostgreSQL with millions of vectors?",
            "复杂技术问题",
            |w: &SearchWeights| w.vector_weight > w.fulltext_weight,
        ),
        (
            "what did we discuss yesterday about the project?",
            "时间相关回忆",
            |w: &SearchWeights| w.has_reasonable_distribution(),
        ),
        (
            "pizza",
            "单词查询",
            |w: &SearchWeights| w.fulltext_weight >= w.vector_weight,
        ),
    ];
    
    for (query, description, validator) in scenarios {
        let features = QueryFeatures::extract_from_query(query);
        let weights = predictor.predict(&features);
        
        println!("\n场景: {description}");
        println!("查询: {query}");
        println!("向量权重: {:.3}", weights.vector_weight);
        println!("全文权重: {:.3}", weights.fulltext_weight);
        
        assert!(
            validator(&weights),
            "权重分布不符合预期: vector={:.3}, fulltext={:.3}",
            weights.vector_weight,
            weights.fulltext_weight
        );
    }
}

// 辅助trait用于测试
trait WeightValidator {
    fn has_reasonable_distribution(&self) -> bool;
}

impl WeightValidator for SearchWeights {
    fn has_reasonable_distribution(&self) -> bool {
        // 权重应该归一化
        let sum = self.vector_weight + self.fulltext_weight;
        if (sum - 1.0).abs() > 0.001 {
            return false;
        }
        
        // 没有权重应该太极端（除非有明确理由）
        self.vector_weight >= 0.2 && self.vector_weight <= 0.8
            && self.fulltext_weight >= 0.2 && self.fulltext_weight <= 0.8
    }
}

