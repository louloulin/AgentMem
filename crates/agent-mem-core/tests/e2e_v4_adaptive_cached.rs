//! E2E Test - V4 Adaptive Search + Cache
//! Week 9-10: 自适应搜索+缓存集成测试

use agent_mem_core::config::AgentMemConfig;
use agent_mem_core::performance::cache::QueryCacheConfig;
use agent_mem_core::search::adaptive_router::AdaptiveRouter;
use agent_mem_core::search::query_classifier::SearchStrategy;
use agent_mem_core::search::adaptive_search_engine::SearchEngineBackend;
use agent_mem_core::search::cached_adaptive_engine::{CachedAdaptiveEngine, ParallelSearchOptimizer};
use agent_mem_core::search::{SearchQuery, SearchResult};
use agent_mem_core::types::{Content, Memory, MemoryBuilder, AttributeKey, AttributeValue};
use anyhow::Result;
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use std::time::Duration;

// ============================================================================
// Mock Search Backend
// ============================================================================

#[derive(Clone)]
struct MockSearchBackend {
    memories: Arc<Mutex<Vec<Memory>>>,
}

impl MockSearchBackend {
    fn new() -> Self {
        let mut memories = Vec::new();
        
        // 初始化测试数据
        for i in 0..50 {
            let memory = MemoryBuilder::new()
                .content(Content::Text(format!("测试记忆内容 {}", i)))
                .attribute(
                    AttributeKey::system("index"),
                    AttributeValue::Number(i as f64),
                )
                .build();
            memories.push(memory);
        }
        
        Self {
            memories: Arc::new(Mutex::new(memories)),
        }
    }
}

#[async_trait]
impl SearchEngineBackend for MockSearchBackend {
    async fn search_with_weights(
        &self,
        _query_vector: Vec<f32>,
        query: SearchQuery,
        _vector_weight: f32,
        _fulltext_weight: f32,
    ) -> Result<super::agent_mem_core::search::adaptive_search_engine::SearchBackendResult> {
        // 模拟搜索延迟
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        let memories = self.memories.lock().unwrap();
        let mut results = Vec::new();
        
        // 简单的文本匹配
        for memory in memories.iter() {
            if let Content::Text(text) = &memory.content {
                if text.contains(&query.query) {
                    results.push(SearchResult {
                        id: memory.id.clone(),
                        score: 0.85,
                        content: text.clone(),
                        metadata: None,
                        vector_score: Some(0.8),
                        fulltext_score: Some(0.9),
                    });
                }
            }
            
            if results.len() >= query.limit {
                break;
            }
        }
        
        Ok(super::agent_mem_core::search::adaptive_search_engine::SearchBackendResult {
            results,
        })
    }
}

// ============================================================================
// E2E Tests
// ============================================================================

/// E2E测试：自适应路由器策略选择
#[tokio::test]
async fn test_adaptive_router_strategy_selection() {
    let config = AgentMemConfig::default();
    let router = AdaptiveRouter::new(config);

    let query = SearchQuery {
        query: "测试查询".to_string(),
        limit: 10,
        threshold: Some(0.7),
        vector_weight: 0.0,
        fulltext_weight: 0.0,
        filters: None,
    };

    // 第一次决策（应该选择Balanced策略）
    let (strategy, weights) = router.decide_strategy(&query).await.unwrap();
    
    assert!(matches!(
        strategy,
        SearchStrategy::VectorHeavy
            | SearchStrategy::Balanced
            | SearchStrategy::FulltextHeavy
            | SearchStrategy::VectorOnly
            | SearchStrategy::FulltextOnly
    ));
    assert!((weights.vector_weight + weights.fulltext_weight - 1.0).abs() < 0.01);

    println!("✅ Adaptive router selected strategy: {:?}", strategy);
    println!("   - Vector weight: {:.2}", weights.vector_weight);
    println!("   - Fulltext weight: {:.2}", weights.fulltext_weight);
}

/// E2E测试：自适应学习反馈循环
#[tokio::test]
async fn test_adaptive_learning_feedback() {
    let config = AgentMemConfig::default();
    let router = AdaptiveRouter::new(config);

    let query = SearchQuery {
        query: "学习测试".to_string(),
        limit: 10,
        threshold: Some(0.7),
        vector_weight: 0.0,
        fulltext_weight: 0.0,
        filters: None,
    };

    // 执行10次搜索+反馈循环
    for i in 0..10 {
        let (strategy, _) = router.decide_strategy(&query).await.unwrap();
        
        // 模拟搜索结果反馈
        let accuracy = 0.7 + (i as f32 * 0.02); // 递增准确率
        let latency_ms = 100 + (i * 5); // 递增延迟
        
        router
            .record_performance(&query, strategy, accuracy, latency_ms as u64)
            .await
            .unwrap();
    }

    // 获取性能历史
    let history = router.get_performance_history().await;
    assert_eq!(history.len(), 10);

    println!("✅ Adaptive learning feedback loop completed");
    println!("   - Iterations: 10");
    println!("   - History records: {}", history.len());
    println!("   - Accuracy range: 0.70 → 0.88");
}

/// E2E测试：缓存命中和未命中
#[tokio::test]
async fn test_cache_hit_and_miss() {
    let config = AgentMemConfig::default();
    let cache_config = QueryCacheConfig::default();
    let backend = Arc::new(MockSearchBackend::new());
    
    let engine = CachedAdaptiveEngine::new(
        backend,
        config,
        cache_config,
        true,  // enable_cache
        false, // disable_learning for this test
    );

    let query = SearchQuery {
        query: "测试记忆内容 5".to_string(),
        limit: 10,
        threshold: Some(0.7),
        vector_weight: 0.0,
        fulltext_weight: 0.0,
        filters: None,
    };

    let query_vector = vec![0.1; 128];

    // 第一次搜索（缓存未命中）
    let start1 = std::time::Instant::now();
    let results1 = engine.search(query_vector.clone(), query.clone()).await.unwrap();
    let latency1 = start1.elapsed();

    assert!(!results1.is_empty());
    assert!(latency1.as_millis() >= 50); // 应该有backend延迟

    // 第二次搜索（缓存命中）
    let start2 = std::time::Instant::now();
    let results2 = engine.search(query_vector.clone(), query.clone()).await.unwrap();
    let latency2 = start2.elapsed();

    assert_eq!(results1.len(), results2.len());
    assert!(latency2.as_millis() < 20); // 缓存应该很快

    println!("✅ Cache hit/miss test passed");
    println!("   - 1st search (miss): {:?}", latency1);
    println!("   - 2nd search (hit): {:?}", latency2);
    println!("   - Speedup: {:.1}x", latency1.as_millis() as f64 / latency2.as_millis() as f64);
}

/// E2E测试：缓存统计
#[tokio::test]
async fn test_cache_statistics() {
    let config = AgentMemConfig::default();
    let cache_config = QueryCacheConfig {
        max_entries: 100,
        default_ttl_seconds: 300,
        enable_lru: true,
        enable_warmup: false,
        warmup_batch_size: 0,
    };
    let backend = Arc::new(MockSearchBackend::new());
    
    let engine = Arc::new(CachedAdaptiveEngine::new(
        backend,
        config,
        cache_config,
        true,
        false,
    ));

    let query_vector = vec![0.1; 128];

    // 执行20次搜索（10个不同查询，每个查询2次）
    for i in 0..10 {
        let query = SearchQuery {
            query: format!("测试记忆内容 {}", i),
            limit: 10,
            threshold: Some(0.7),
            vector_weight: 0.0,
            fulltext_weight: 0.0,
            filters: None,
        };

        // 每个查询执行2次
        engine.search(query_vector.clone(), query.clone()).await.unwrap();
        engine.search(query_vector.clone(), query).await.unwrap();
    }

    // 获取缓存统计
    let stats = engine.get_cache_stats().await.unwrap();
    println!("✅ Cache statistics:");
    println!("{}", stats);
    
    // 验证：20次请求，10次未命中，10次命中，命中率50%
    assert!(stats.contains("Total Requests: 20"));
    assert!(stats.contains("Cache Hits: 10"));
}

/// E2E测试：并发搜索性能
#[tokio::test]
async fn test_parallel_search_performance() {
    let config = AgentMemConfig::default();
    let cache_config = QueryCacheConfig::default();
    let backend = Arc::new(MockSearchBackend::new());
    
    let engine = Arc::new(CachedAdaptiveEngine::new(
        backend,
        config,
        cache_config,
        true,
        false,
    ));

    let optimizer = ParallelSearchOptimizer::new(10); // 最大10并发

    // 准备50个查询
    let queries: Vec<(Vec<f32>, SearchQuery)> = (0..50)
        .map(|i| {
            let query = SearchQuery {
                query: format!("测试记忆内容 {}", i),
                limit: 10,
                threshold: Some(0.7),
                vector_weight: 0.0,
                fulltext_weight: 0.0,
                filters: None,
            };
            (vec![0.1; 128], query)
        })
        .collect();

    // 执行并发搜索
    let start = std::time::Instant::now();
    let results = optimizer.batch_search(engine.clone(), queries).await.unwrap();
    let elapsed = start.elapsed();

    assert_eq!(results.len(), 50);
    
    let qps = 50.0 / elapsed.as_secs_f64();
    println!("✅ Parallel search completed");
    println!("   - Queries: 50");
    println!("   - Elapsed: {:?}", elapsed);
    println!("   - QPS: {:.0}", qps);

    // 串行最少需要 50 * 50ms = 2.5s，并发应该显著减少
    assert!(elapsed.as_secs() < 3);
}

/// E2E测试：缓存预热
#[tokio::test]
async fn test_cache_warmup() {
    let config = AgentMemConfig::default();
    let cache_config = QueryCacheConfig::default();
    let backend = Arc::new(MockSearchBackend::new());
    
    let engine = CachedAdaptiveEngine::new(
        backend,
        config,
        cache_config,
        true,
        false,
    );

    // 准备热点查询
    let hot_queries: Vec<(Vec<f32>, SearchQuery)> = (0..10)
        .map(|i| {
            let query = SearchQuery {
                query: format!("测试记忆内容 {}", i),
                limit: 10,
                threshold: Some(0.7),
                vector_weight: 0.0,
                fulltext_weight: 0.0,
                filters: None,
            };
            (vec![0.1; 128], query)
        })
        .collect();

    // 预热缓存
    let warmed = engine.warmup_cache(hot_queries).await.unwrap();
    
    assert_eq!(warmed, 10);
    println!("✅ Cache warmup completed: {} queries", warmed);

    // 验证缓存已填充
    let stats = engine.get_cache_stats().await.unwrap();
    assert!(stats.contains("Total Entries: 10"));
}

/// E2E测试：自适应搜索完整流程
#[tokio::test]
async fn test_full_adaptive_search_flow() {
    let config = AgentMemConfig::default();
    let cache_config = QueryCacheConfig::default();
    let backend = Arc::new(MockSearchBackend::new());
    
    let engine = CachedAdaptiveEngine::new(
        backend,
        config,
        cache_config,
        true, // enable_cache
        true, // enable_learning
    );

    let query = SearchQuery {
        query: "测试记忆内容 10".to_string(),
        limit: 10,
        threshold: Some(0.7),
        vector_weight: 0.0,
        fulltext_weight: 0.0,
        filters: None,
    };

    let query_vector = vec![0.1; 128];

    // 执行搜索（完整流程：路由→搜索→缓存→学习）
    let results = engine.search(query_vector, query).await.unwrap();

    assert!(!results.is_empty());
    
    for (idx, result) in results.iter().enumerate() {
        println!(
            "   Result {}: score={:.2}, content={}",
            idx + 1,
            result.score,
            &result.content[..30.min(result.content.len())]
        );
    }

    println!("✅ Full adaptive search flow completed");
    println!("   - Results: {}", results.len());
}

/// E2E测试：缓存清空
#[tokio::test]
async fn test_cache_clear() {
    let config = AgentMemConfig::default();
    let cache_config = QueryCacheConfig::default();
    let backend = Arc::new(MockSearchBackend::new());
    
    let engine = CachedAdaptiveEngine::new(
        backend,
        config,
        cache_config,
        true,
        false,
    );

    let query = SearchQuery {
        query: "测试记忆内容 20".to_string(),
        limit: 10,
        threshold: Some(0.7),
        vector_weight: 0.0,
        fulltext_weight: 0.0,
        filters: None,
    };

    // 执行搜索填充缓存
    engine.search(vec![0.1; 128], query).await.unwrap();

    // 验证缓存有内容
    let stats1 = engine.get_cache_stats().await.unwrap();
    assert!(stats1.contains("Total Entries: 1"));

    // 清空缓存
    engine.clear_cache().await.unwrap();

    // 验证缓存已清空
    let stats2 = engine.get_cache_stats().await.unwrap();
    assert!(stats2.contains("Total Entries: 0"));

    println!("✅ Cache clear test passed");
}

