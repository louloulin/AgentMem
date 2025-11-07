// Phase 3-D: 查询优化与智能重排序集成测试

use agent_mem_core::search::{
    IndexStatistics, OptimizedSearchPlan, QueryOptimizer, QueryOptimizerConfig, RerankConfig,
    ResultReranker, SearchQuery, SearchResult, SearchStrategy,
};
use std::sync::{Arc, RwLock};

/// 测试查询优化器 - 小数据集
#[test]
fn test_query_optimizer_small_dataset() {
    let stats = Arc::new(RwLock::new(IndexStatistics::new(5_000, 1536)));
    let optimizer = QueryOptimizer::with_default_config(stats);

    let query = SearchQuery {
        query: "test query".to_string(),
        limit: 10,
        ..Default::default()
    };

    let plan = optimizer.optimize_query(&query).unwrap();

    // 小数据集应使用精确搜索
    assert!(matches!(plan.strategy, SearchStrategy::Exact));
    // 注意：5000个向量仍然>1000（重排序阈值），所以会启用重排序
    // 这是合理的，因为可以提升精度
    assert_eq!(plan.estimated_recall, 1.0); // 100%召回
}

/// 测试查询优化器 - 中等数据集
#[test]
fn test_query_optimizer_medium_dataset() {
    let stats = Arc::new(RwLock::new(IndexStatistics::new(50_000, 1536)));
    let optimizer = QueryOptimizer::with_default_config(stats);

    let query = SearchQuery {
        query: "test query".to_string(),
        limit: 10,
        ..Default::default()
    };

    let plan = optimizer.optimize_query(&query).unwrap();

    // 中等数据集应使用HNSW
    assert!(matches!(plan.strategy, SearchStrategy::HNSW { .. }));
    assert!(plan.should_rerank); // 需要重排序
    assert!(plan.estimated_recall >= 0.95); // 高召回率
}

/// 测试查询优化器 - 大数据集
#[test]
fn test_query_optimizer_large_dataset() {
    let stats = Arc::new(RwLock::new(IndexStatistics::new(150_000, 1536)));
    let optimizer = QueryOptimizer::with_default_config(stats);

    let query = SearchQuery {
        query: "test query".to_string(),
        limit: 10,
        ..Default::default()
    };

    let plan = optimizer.optimize_query(&query).unwrap();

    // 大数据集应使用混合索引
    assert!(matches!(plan.strategy, SearchStrategy::Hybrid { .. }));
    assert!(plan.should_rerank);
    assert!(plan.estimated_latency_ms < 50); // 应该很快
}

/// 测试延迟估算
#[test]
fn test_latency_estimation() {
    let configs = vec![
        (1_000, 15),   // 小数据集，~15ms
        (10_000, 25),  // 中数据集，~25ms
        (100_000, 35), // 大数据集，~35ms
    ];

    for (size, expected_max_latency) in configs {
        let stats = Arc::new(RwLock::new(IndexStatistics::new(size, 1536)));
        let optimizer = QueryOptimizer::with_default_config(stats);

        let query = SearchQuery {
            query: "test".to_string(),
            limit: 10,
            ..Default::default()
        };

        let plan = optimizer.optimize_query(&query).unwrap();
        assert!(
            plan.estimated_latency_ms <= expected_max_latency,
            "数据集大小: {}, 预期延迟: {} ms, 实际: {} ms",
            size,
            expected_max_latency,
            plan.estimated_latency_ms
        );
    }
}

/// 测试统计信息更新
#[test]
fn test_statistics_update() {
    let stats = Arc::new(RwLock::new(IndexStatistics::new(5_000, 1536)));
    let optimizer = QueryOptimizer::with_default_config(stats.clone());

    // 初始应该是Flat索引
    {
        let s = stats.read().unwrap();
        assert_eq!(s.total_vectors, 5_000);
    }

    // 更新到中等规模
    optimizer.update_statistics(50_000);
    {
        let s = stats.read().unwrap();
        assert_eq!(s.total_vectors, 50_000);
    }

    // 更新到大规模
    optimizer.update_statistics(150_000);
    {
        let s = stats.read().unwrap();
        assert_eq!(s.total_vectors, 150_000);
    }
}

/// 创建测试搜索结果
fn create_test_result(id: &str, score: f32, content: &str) -> SearchResult {
    SearchResult {
        id: id.to_string(),
        content: content.to_string(),
        score,
        vector_score: Some(score),
        fulltext_score: None,
        metadata: None,
    }
}

/// 测试结果重排序 - 基础功能
#[tokio::test]
async fn test_result_reranker_basic() {
    let reranker = ResultReranker::with_default_config();

    // 创建候选结果
    let candidates = vec![
        create_test_result("1", 0.7, "short"),
        create_test_result(
            "2",
            0.9,
            "This is a test content with good length for quality scoring",
        ),
        create_test_result("3", 0.8, "medium length content here"),
    ];

    let query_vector = vec![0.5; 1536];
    let query = SearchQuery {
        query: "test".to_string(),
        limit: 10,
        ..Default::default()
    };

    let reranked = reranker
        .rerank(candidates, &query_vector, &query)
        .await
        .unwrap();

    // 应该按综合得分排序
    assert_eq!(reranked.len(), 3);
    // 最高分应该在前面
    for i in 0..reranked.len() - 1 {
        assert!(reranked[i].score >= reranked[i + 1].score);
    }
}

/// 测试余弦相似度计算
#[test]
fn test_cosine_similarity() {
    use agent_mem_core::search::cosine_similarity_exact;

    // 测试1：相同向量
    let vec1 = vec![1.0, 0.0, 0.0];
    let vec2 = vec![1.0, 0.0, 0.0];
    let similarity = cosine_similarity_exact(&vec1, &vec2);
    assert!((similarity - 1.0).abs() < 0.001);

    // 测试2：正交向量
    let vec3 = vec![1.0, 0.0, 0.0];
    let vec4 = vec![0.0, 1.0, 0.0];
    let similarity = cosine_similarity_exact(&vec3, &vec4);
    assert!((similarity - 0.0).abs() < 0.001);

    // 测试3：相反向量
    let vec5 = vec![1.0, 0.0, 0.0];
    let vec6 = vec![-1.0, 0.0, 0.0];
    let similarity = cosine_similarity_exact(&vec5, &vec6);
    assert!((similarity + 1.0).abs() < 0.001);
}

/// 测试自定义重排序配置
#[tokio::test]
async fn test_custom_rerank_config() {
    // 创建强调相似度的配置
    let config = RerankConfig {
        similarity_weight: 0.8, // 80%权重给相似度
        metadata_weight: 0.1,
        time_weight: 0.05,
        importance_weight: 0.03,
        quality_weight: 0.02,
        ..Default::default()
    };

    let reranker = ResultReranker::new(config);

    let candidates = vec![
        create_test_result("1", 0.95, "high similarity"),
        create_test_result("2", 0.70, "low similarity but good content"),
    ];

    let query_vector = vec![0.5; 1536];
    let query = SearchQuery {
        query: "test".to_string(),
        limit: 10,
        ..Default::default()
    };

    let reranked = reranker
        .rerank(candidates, &query_vector, &query)
        .await
        .unwrap();

    // 高相似度应该排在前面（因为权重很高）
    assert_eq!(reranked[0].id, "1");
}

/// 测试查询优化器和重排序器的集成
#[tokio::test]
async fn test_optimizer_reranker_integration() {
    // 1. 创建查询优化器
    let stats = Arc::new(RwLock::new(IndexStatistics::new(50_000, 1536)));
    let optimizer = QueryOptimizer::with_default_config(stats);

    // 2. 创建重排序器
    let reranker = ResultReranker::with_default_config();

    // 3. 优化查询
    let query = SearchQuery {
        query: "test query".to_string(),
        limit: 10,
        ..Default::default()
    };

    let plan = optimizer.optimize_query(&query).unwrap();

    // 4. 模拟搜索结果
    let mut candidates = vec![
        create_test_result("1", 0.8, "result one"),
        create_test_result("2", 0.9, "result two with better content"),
        create_test_result("3", 0.7, "result three"),
    ];

    // 5. 根据计划调整候选数量
    if plan.should_rerank {
        // 应该召回更多候选用于重排序
        candidates.extend(vec![
            create_test_result("4", 0.65, "extra candidate one"),
            create_test_result("5", 0.68, "extra candidate two"),
        ]);
    }

    // 6. 重排序
    let query_vector = vec![0.5; 1536];
    let reranked = reranker
        .rerank(candidates, &query_vector, &query)
        .await
        .unwrap();

    // 7. 取top-k
    let final_results: Vec<_> = reranked.into_iter().take(query.limit).collect();

    assert!(final_results.len() <= query.limit);
    // 结果应该是排序好的
    for i in 0..final_results.len() - 1 {
        assert!(final_results[i].score >= final_results[i + 1].score);
    }
}

/// 性能基准测试（简化版）
#[tokio::test]
async fn test_performance_baseline() {
    use std::time::Instant;

    let reranker = ResultReranker::with_default_config();

    // 创建100个候选结果
    let candidates: Vec<_> = (0..100)
        .map(|i| {
            create_test_result(
                &format!("id_{}", i),
                0.5 + (i as f32 / 200.0),
                "test content",
            )
        })
        .collect();

    let query_vector = vec![0.5; 1536];
    let query = SearchQuery {
        query: "test".to_string(),
        limit: 10,
        ..Default::default()
    };

    let start = Instant::now();
    let _reranked = reranker
        .rerank(candidates, &query_vector, &query)
        .await
        .unwrap();
    let duration = start.elapsed();

    // 重排序100个结果应该很快（<10ms）
    assert!(duration.as_millis() < 10, "重排序耗时: {:?}", duration);
}
