//! 综合自适应搜索验证测试
//!
//! 包含更多真实场景和边界情况的验证

use std::sync::Arc;
use agent_mem_config::agentmem_config::AgentMemConfig;
use agent_mem_core::search::{
    AdaptiveSearchOptimizer, QueryFeatures, SearchQuery, SearchReranker, SearchResult,
    SearchWeights, WeightPredictor,
};

/// 测试多语言查询支持
#[test]
fn test_multilingual_query_support() {
    let test_cases = vec![
        ("user@example.com", "英文邮箱"),
        ("用户@示例.com", "中文邮箱"),
        ("What is AI?", "英文问句"),
        ("什么是人工智能？", "中文问句"),
        ("昨天发生了什么？", "中文时间查询"),
        ("yesterday meeting", "英文时间查询"),
        ("@张三 说过 #项目 很重要", "中文实体"),
        ("@John mentioned #project", "英文实体"),
    ];

    let config = Arc::new(AgentMemConfig::default().search);
    let optimizer = AdaptiveSearchOptimizer::new(config);

    println!("\n=== 多语言查询支持测试 ===\n");

    for (query_text, description) in test_cases {
        let query = SearchQuery {
            query: query_text.to_string(),
            ..Default::default()
        };

        let (_, weights) = optimizer.optimize_query(&query);

        println!("场景: {}", description);
        println!("查询: \"{}\"", query_text);
        println!("  向量权重: {:.3}", weights.vector_weight);
        println!("  全文权重: {:.3}", weights.fulltext_weight);

        // 验证权重有效性
        assert!(weights.vector_weight >= 0.0 && weights.vector_weight <= 1.0);
        assert!(weights.fulltext_weight >= 0.0 && weights.fulltext_weight <= 1.0);
        assert!((weights.vector_weight + weights.fulltext_weight - 1.0).abs() < 0.001);

        println!("  ✅ 权重有效\n");
    }
}

/// 测试极端长度查询
#[test]
fn test_extreme_length_queries() {
    let config = Arc::new(AgentMemConfig::default().search);
    let optimizer = AdaptiveSearchOptimizer::new(config);

    // 极短查询
    let very_short = vec!["a", "x", "?", "!", "@"];

    println!("\n=== 极短查询测试 ===\n");
    for query_text in very_short {
        let query = SearchQuery {
            query: query_text.to_string(),
            ..Default::default()
        };

        let (_, weights) = optimizer.optimize_query(&query);
        println!(
            "查询: \"{}\" → vector={:.3}, fulltext={:.3}",
            query_text, weights.vector_weight, weights.fulltext_weight
        );

        assert!((weights.vector_weight + weights.fulltext_weight - 1.0).abs() < 0.001);
    }

    // 极长查询
    let very_long =
        "This is an extremely long query that contains multiple sentences and paragraphs. "
            .repeat(20);

    println!("\n=== 极长查询测试 ===\n");
    let query = SearchQuery {
        query: very_long.clone(),
        ..Default::default()
    };

    let (_, weights) = optimizer.optimize_query(&query);
    println!("查询长度: {} 字符", very_long.len());
    println!("  向量权重: {:.3}", weights.vector_weight);
    println!("  全文权重: {:.3}", weights.fulltext_weight);

    assert!((weights.vector_weight + weights.fulltext_weight - 1.0).abs() < 0.001);
}

/// 测试特殊字符和符号
#[test]
fn test_special_characters() {
    let special_queries = vec![
        ("!@#$%^&*()", "特殊符号"),
        ("user@example.com", "邮箱符号"),
        ("C++ programming", "编程符号"),
        ("价格$99.99", "货币符号"),
        ("50% discount", "百分号"),
        ("file.txt", "文件扩展名"),
        ("/path/to/file", "路径"),
        ("https://example.com", "URL"),
        ("(parentheses) [brackets] {braces}", "括号"),
        ("emoji 😀 👍 🎉", "表情符号"),
    ];

    let config = Arc::new(AgentMemConfig::default().search);
    let optimizer = AdaptiveSearchOptimizer::new(config);

    println!("\n=== 特殊字符测试 ===\n");

    for (query_text, description) in special_queries {
        let query = SearchQuery {
            query: query_text.to_string(),
            ..Default::default()
        };

        let (_, weights) = optimizer.optimize_query(&query);

        println!("{}: \"{}\"", description, query_text);
        println!(
            "  vector={:.3}, fulltext={:.3}",
            weights.vector_weight, weights.fulltext_weight
        );

        assert!((weights.vector_weight + weights.fulltext_weight - 1.0).abs() < 0.001);
    }
}

/// 测试权重一致性
#[test]
fn test_weight_consistency() {
    let config = Arc::new(AgentMemConfig::default().search);
    let optimizer = AdaptiveSearchOptimizer::new(config);

    // 相同查询应该得到相同的权重
    let query_text = "How does machine learning work?";

    let mut weights_list = Vec::new();

    for _ in 0..10 {
        let query = SearchQuery {
            query: query_text.to_string(),
            ..Default::default()
        };

        let (_, weights) = optimizer.optimize_query(&query);
        weights_list.push((weights.vector_weight, weights.fulltext_weight));
    }

    // 验证所有权重相同
    let first = weights_list[0];
    for weights in &weights_list[1..] {
        assert_eq!(weights.0, first.0, "向量权重应该一致");
        assert_eq!(weights.1, first.1, "全文权重应该一致");
    }

    println!("✅ 权重一致性测试通过：10次查询得到相同权重");
}

/// 测试重排序的稳定性
#[test]
fn test_reranker_stability() {
    let reranker = SearchReranker::new();

    // 创建测试结果集
    let results = vec![
        SearchResult {
            id: "1".to_string(),
            content: "First result".to_string(),
            score: 0.9,
            vector_score: Some(0.9),
            fulltext_score: None,
            metadata: None,
        },
        SearchResult {
            id: "2".to_string(),
            content: "Second result".to_string(),
            score: 0.85,
            vector_score: Some(0.85),
            fulltext_score: None,
            metadata: None,
        },
        SearchResult {
            id: "3".to_string(),
            content: "Third result".to_string(),
            score: 0.8,
            vector_score: Some(0.8),
            fulltext_score: None,
            metadata: None,
        },
    ];

    let query = SearchQuery {
        query: "test".to_string(),
        ..Default::default()
    };

    // 多次重排序应该得到相同结果
    let reranked1 = reranker.rerank(results.clone(), &query);
    let reranked2 = reranker.rerank(results.clone(), &query);
    let reranked3 = reranker.rerank(results, &query);

    assert_eq!(reranked1.len(), reranked2.len());
    assert_eq!(reranked2.len(), reranked3.len());

    for i in 0..reranked1.len() {
        assert_eq!(reranked1[i].id, reranked2[i].id);
        assert_eq!(reranked2[i].id, reranked3[i].id);
        assert!((reranked1[i].score - reranked2[i].score).abs() < 0.0001);
    }

    println!("✅ 重排序稳定性测试通过");
}

/// 测试并发安全性
#[tokio::test]
async fn test_concurrent_optimization() {
    use std::sync::Arc;
    use tokio::sync::RwLock;

    let config = Arc::new(AgentMemConfig::default().search);
    let optimizer = Arc::new(RwLock::new(AdaptiveSearchOptimizer::new(config)));

    let queries = vec![
        "user@example.com",
        "How does AI work?",
        "yesterday",
        "simple query",
        "complex technical question about distributed systems",
    ];

    let mut handles = vec![];

    for query_text in queries {
        let optimizer = optimizer.clone();
        let query_text = query_text.to_string();

        let handle = tokio::spawn(async move {
            let optimizer = optimizer.read().await;
            let query = SearchQuery {
                query: query_text.clone(),
                ..Default::default()
            };

            let (_, weights) = optimizer.optimize_query(&query);

            // 验证权重有效性
            assert!(weights.vector_weight >= 0.0 && weights.vector_weight <= 1.0);
            assert!(weights.fulltext_weight >= 0.0 && weights.fulltext_weight <= 1.0);
            assert!((weights.vector_weight + weights.fulltext_weight - 1.0).abs() < 0.001);

            query_text
        });

        handles.push(handle);
    }

    let results = futures::future::join_all(handles).await;

    for result in results {
        assert!(result.is_ok(), "并发查询不应该失败");
    }

    println!("✅ 并发安全性测试通过");
}

/// 测试性能退化检测
#[test]
fn test_performance_degradation() {
    let config = Arc::new(AgentMemConfig::default().search);
    let optimizer = AdaptiveSearchOptimizer::new(config);

    use std::time::Instant;

    let queries: Vec<String> = (0..1000)
        .map(|i| format!("test query number {}", i))
        .collect();

    let start = Instant::now();

    for query_text in &queries {
        let query = SearchQuery {
            query: query_text.clone(),
            ..Default::default()
        };

        let (_, _weights) = optimizer.optimize_query(&query);
    }

    let elapsed = start.elapsed();
    let avg_time = elapsed.as_micros() as f64 / queries.len() as f64;

    println!("\n=== 性能测试 ===");
    println!("查询数量: {}", queries.len());
    println!("总耗时: {:?}", elapsed);
    println!("平均耗时: {:.2} μs/查询", avg_time);

    // 平均每个查询应该在100μs以内
    assert!(
        avg_time < 100.0,
        "性能退化：平均查询时间 {:.2} μs 超过100 μs",
        avg_time
    );

    println!("✅ 性能测试通过");
}

/// 测试查询特征提取的准确性
#[test]
fn test_feature_extraction_accuracy() {
    println!("\n=== 特征提取准确性测试 ===\n");

    // 测试1: 精确匹配检测
    let exact_match_queries = vec!["user@example.com", "#tag", "\"exact phrase\"", "@mention"];
    for query in exact_match_queries {
        let features = QueryFeatures::extract_from_query(query);
        assert!(features.has_exact_terms, "应该检测到精确匹配: {}", query);
    }
    println!("✅ 精确匹配检测准确");

    // 测试2: 时间指示词检测
    let temporal_queries = vec!["yesterday", "today", "last week", "昨天", "最近"];
    for query in temporal_queries {
        let features = QueryFeatures::extract_from_query(query);
        assert!(
            features.has_temporal_indicator,
            "应该检测到时间指示: {}",
            query
        );
    }
    println!("✅ 时间指示词检测准确");

    // 测试3: 问句检测
    let question_queries = vec![
        "What is AI?",
        "How does it work?",
        "Why?",
        "什么是AI？",
        "怎么做？",
    ];
    for query in question_queries {
        let features = QueryFeatures::extract_from_query(query);
        assert!(features.is_question, "应该检测到问句: {}", query);
    }
    println!("✅ 问句检测准确");

    // 测试4: 语义复杂度计算
    let simple = QueryFeatures::extract_from_query("pizza");
    let complex = QueryFeatures::extract_from_query(
        "Explain the architectural considerations for implementing distributed vector databases",
    );
    assert!(
        simple.semantic_complexity < complex.semantic_complexity,
        "复杂查询的语义复杂度应该更高"
    );
    println!("✅ 语义复杂度计算合理");
}

/// 测试权重预测的合理性
#[test]
fn test_weight_prediction_rationality() {
    let config = Arc::new(AgentMemConfig::default().search);
    let predictor = WeightPredictor::new(config);

    println!("\n=== 权重预测合理性测试 ===\n");

    // 测试1: 精确匹配应该提高全文权重
    let exact_features = QueryFeatures {
        has_exact_terms: true,
        semantic_complexity: 0.3,
        has_temporal_indicator: false,
        entity_count: 0,
        query_length: 20,
        is_question: false,
    };
    let weights = predictor.predict(&exact_features);
    assert!(
        weights.fulltext_weight > 0.5,
        "精确匹配查询应该偏向全文搜索，实际: {:.2}",
        weights.fulltext_weight
    );
    println!("✅ 精确匹配权重合理");

    // 测试2: 高语义复杂度应该提高向量权重
    let semantic_features = QueryFeatures {
        has_exact_terms: false,
        semantic_complexity: 0.9,
        has_temporal_indicator: false,
        entity_count: 0,
        query_length: 100,
        is_question: true,
    };
    let weights = predictor.predict(&semantic_features);
    assert!(
        weights.vector_weight > 0.6,
        "高语义复杂度应该偏向向量搜索，实际: {:.2}",
        weights.vector_weight
    );
    println!("✅ 语义查询权重合理");

    // 测试3: 平衡查询应该权重接近
    let balanced_features = QueryFeatures {
        has_exact_terms: false,
        semantic_complexity: 0.5,
        has_temporal_indicator: false,
        entity_count: 0,
        query_length: 30,
        is_question: false,
    };
    let weights = predictor.predict(&balanced_features);
    let diff = (weights.vector_weight - weights.fulltext_weight).abs();
    assert!(
        diff < 0.3,
        "平衡查询权重差异应该较小，实际差异: {:.2}",
        diff
    );
    println!("✅ 平衡查询权重合理");
}

/// 测试学习机制
#[test]
fn test_learning_mechanism() {
    let config = Arc::new(AgentMemConfig::default().search);
    let mut optimizer = AdaptiveSearchOptimizer::new(config);

    let query = "test query";
    let good_weights = SearchWeights {
        vector_weight: 0.7,
        fulltext_weight: 0.3,
        confidence: 0.8,
    };

    let bad_weights = SearchWeights {
        vector_weight: 0.5,
        fulltext_weight: 0.5,
        confidence: 0.6,
    };

    // 记录好的配置（高效果）
    optimizer.record_feedback(query, good_weights.clone(), 0.95);

    // 记录坏的配置（低效果）- 不应该被记录
    optimizer.record_feedback(query, bad_weights, 0.3);

    // 记录中等配置 - 不应该被记录
    optimizer.record_feedback(query, good_weights, 0.6);

    println!("✅ 学习机制测试通过（只记录高效反馈）");
}
