//! 自适应搜索性能基准测试
//!
//! 对比固定权重 vs 自适应权重的性能和准确性

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use agent_mem_core::search::{
    AdaptiveSearchOptimizer, QueryFeatures, SearchQuery, SearchReranker, WeightPredictor,
};

/// 测试查询特征提取性能
fn bench_feature_extraction(c: &mut Criterion) {
    let queries = vec![
        "user@example.com",
        "How does the memory system work?",
        "what happened yesterday?",
        "Find conversations with @John about Python programming",
        "simple",
        "Explain how to optimize vector search performance in large-scale systems",
    ];
    
    c.bench_function("feature_extraction", |b| {
        b.iter(|| {
            for query in &queries {
                let _features = QueryFeatures::extract_from_query(black_box(query));
            }
        });
    });
}

/// 测试权重预测性能
fn bench_weight_prediction(c: &mut Criterion) {
    let predictor = WeightPredictor::new();
    
    let test_features = vec![
        QueryFeatures {
            has_exact_terms: true,
            semantic_complexity: 0.3,
            has_temporal_indicator: false,
            entity_count: 0,
            query_length: 20,
            is_question: false,
        },
        QueryFeatures {
            has_exact_terms: false,
            semantic_complexity: 0.8,
            has_temporal_indicator: false,
            entity_count: 0,
            query_length: 100,
            is_question: true,
        },
        QueryFeatures {
            has_exact_terms: false,
            semantic_complexity: 0.5,
            has_temporal_indicator: true,
            entity_count: 2,
            query_length: 50,
            is_question: true,
        },
    ];
    
    c.bench_function("weight_prediction", |b| {
        b.iter(|| {
            for features in &test_features {
                let _weights = predictor.predict(black_box(features));
            }
        });
    });
}

/// 测试端到端优化流程
fn bench_end_to_end_optimization(c: &mut Criterion) {
    let optimizer = AdaptiveSearchOptimizer::new(true);
    
    let queries = vec![
        SearchQuery {
            query: "user@example.com".to_string(),
            limit: 10,
            ..Default::default()
        },
        SearchQuery {
            query: "How does machine learning improve search accuracy?".to_string(),
            limit: 10,
            ..Default::default()
        },
        SearchQuery {
            query: "yesterday's meeting notes".to_string(),
            limit: 10,
            ..Default::default()
        },
    ];
    
    c.bench_function("end_to_end_optimization", |b| {
        b.iter(|| {
            for query in &queries {
                let (_optimized, _weights) = optimizer.optimize_query(black_box(query));
            }
        });
    });
}

/// 测试结果重排序性能
fn bench_reranking(c: &mut Criterion) {
    use agent_mem_core::search::SearchResult;
    
    let reranker = SearchReranker::new();
    
    // 创建不同大小的结果集
    let sizes = vec![10, 50, 100];
    
    for size in sizes {
        let mut results: Vec<SearchResult> = (0..size)
            .map(|i| SearchResult {
                id: format!("id_{}", i),
                content: format!("Content {} with some text to make it realistic", i),
                score: 0.9 - (i as f32 * 0.001),
                vector_score: Some(0.85),
                fulltext_score: Some(0.75),
                metadata: if i % 3 == 0 {
                    Some(serde_json::json!({
                        "importance": 0.8,
                        "created_at": 1700000000 + i * 1000,
                    }))
                } else {
                    None
                },
            })
            .collect();
        
        let query = SearchQuery {
            query: "test query".to_string(),
            ..Default::default()
        };
        
        c.bench_with_input(
            BenchmarkId::new("reranking", size),
            &size,
            |b, _| {
                b.iter(|| {
                    let _reranked = reranker.rerank(black_box(results.clone()), black_box(&query));
                });
            },
        );
    }
}

/// 对比不同查询类型的优化效果
fn bench_query_types_comparison(c: &mut Criterion) {
    let optimizer = AdaptiveSearchOptimizer::new(true);
    
    let query_types = vec![
        ("exact_match", "user@example.com"),
        ("semantic", "How does artificial intelligence improve search quality and relevance?"),
        ("temporal", "what happened yesterday in our discussion?"),
        ("simple", "pizza"),
        ("complex", "Analyze the performance characteristics of distributed vector databases"),
    ];
    
    let mut group = c.benchmark_group("query_types");
    
    for (type_name, query_text) in query_types {
        let query = SearchQuery {
            query: query_text.to_string(),
            ..Default::default()
        };
        
        group.bench_with_input(
            BenchmarkId::from_parameter(type_name),
            &query,
            |b, q| {
                b.iter(|| {
                    let (_optimized, _weights) = optimizer.optimize_query(black_box(q));
                });
            },
        );
    }
    
    group.finish();
}

/// 测试权重归一化性能
fn bench_weight_normalization(c: &mut Criterion) {
    use agent_mem_core::search::SearchWeights;
    
    c.bench_function("weight_normalization", |b| {
        b.iter(|| {
            for i in 0..100 {
                let mut weights = SearchWeights {
                    vector_weight: 0.5 + (i as f32 * 0.01),
                    fulltext_weight: 0.5 - (i as f32 * 0.01),
                    confidence: 0.8,
                };
                weights.normalize();
                black_box(weights);
            }
        });
    });
}

criterion_group!(
    benches,
    bench_feature_extraction,
    bench_weight_prediction,
    bench_end_to_end_optimization,
    bench_reranking,
    bench_query_types_comparison,
    bench_weight_normalization,
);

criterion_main!(benches);

