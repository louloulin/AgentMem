//! 性能优化模块测试

use agent_mem_core::performance::{
    BatchConfig, BatchProcessor, BenchmarkConfig, CacheKey, PerformanceBenchmark,
    PerformanceConfig, PerformanceManager, QueryCache, QueryCacheConfig,
};
use agent_mem_traits::Result;

#[tokio::test]
async fn test_query_cache_basic() {
    let config = QueryCacheConfig::default();
    let cache = QueryCache::new(config);
    
    // 测试缓存设置和获取
    let key = CacheKey::new("test_query", &"param1");
    let value = vec![1, 2, 3, 4, 5];
    
    cache.put(key.clone(), value.clone()).await.unwrap();
    
    let cached: Option<Vec<i32>> = cache.get(&key).await;
    assert_eq!(cached, Some(value));
    
    // 测试统计
    let stats = cache.get_stats().await;
    assert_eq!(stats.total_requests, 1);
    assert_eq!(stats.cache_hits, 1);
}

#[tokio::test]
async fn test_query_cache_miss() {
    let config = QueryCacheConfig::default();
    let cache = QueryCache::new(config);
    
    let key = CacheKey::new("test_query", &"param1");
    let cached: Option<Vec<i32>> = cache.get(&key).await;
    
    assert_eq!(cached, None);
    
    let stats = cache.get_stats().await;
    assert_eq!(stats.cache_misses, 1);
}

#[tokio::test]
async fn test_query_cache_cleanup() {
    let config = QueryCacheConfig {
        max_entries: 10,
        default_ttl_seconds: 1, // 1 秒过期
        enable_lru: true,
        enable_warmup: false,
        warmup_batch_size: 100,
    };
    let cache = QueryCache::new(config);
    
    // 添加一些缓存项
    for i in 0..5 {
        let key = CacheKey::new("test_query", &i);
        cache.put(key, vec![i]).await.unwrap();
    }
    
    // 等待过期
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    // 清理过期项
    cache.cleanup_expired().await.unwrap();
    
    let stats = cache.get_stats().await;
    assert_eq!(stats.expired_entries, 5);
}

#[tokio::test]
async fn test_batch_processor_basic() {
    let config = BatchConfig::default();
    let processor = BatchProcessor::new(config);
    
    let items = vec![1, 2, 3, 4, 5];
    let results = processor
        .batch_execute(items, |x| async move { Ok(x * 2) })
        .await
        .unwrap();
    
    assert_eq!(results, vec![2, 4, 6, 8, 10]);
    
    let stats = processor.get_stats().await;
    assert_eq!(stats.total_batches, 1);
    assert_eq!(stats.total_items, 5);
}

#[tokio::test]
async fn test_batch_processor_large_batch() {
    let config = BatchConfig {
        batch_size: 10,
        max_concurrency: 5,
        batch_timeout_seconds: 30,
        enable_auto_batching: true,
        auto_batch_delay_ms: 100,
    };
    let processor = BatchProcessor::new(config);
    
    let items: Vec<i32> = (0..100).collect();
    let results = processor
        .batch_execute(items, |x| async move { Ok(x * 2) })
        .await
        .unwrap();
    
    assert_eq!(results.len(), 100);
    assert_eq!(results[0], 0);
    assert_eq!(results[99], 198);
    
    let stats = processor.get_stats().await;
    assert_eq!(stats.total_items, 100);
}

#[tokio::test]
async fn test_batch_vector_search() {
    let config = BatchConfig::default();
    let processor = BatchProcessor::new(config);
    
    let query_vectors = vec![
        vec![1.0, 2.0, 3.0],
        vec![4.0, 5.0, 6.0],
        vec![7.0, 8.0, 9.0],
    ];
    
    let results: Vec<f32> = processor
        .batch_vector_search(query_vectors, |v| async move {
            Ok(v.iter().sum::<f32>())
        })
        .await
        .unwrap();
    
    assert_eq!(results, vec![6.0, 15.0, 24.0]);
}

#[tokio::test]
async fn test_performance_benchmark() {
    let config = BenchmarkConfig {
        iterations: 100,
        warmup_iterations: 10,
        vector_dimension: 128,
        dataset_size: 1000,
        verbose: false,
    };
    
    let mut benchmark = PerformanceBenchmark::new(config);
    let result = benchmark.run_all_benchmarks().await.unwrap();
    
    // 验证所有基准测试都运行了
    assert!(result.vector_search.avg_latency_us > 0.0);
    assert!(result.graph_query.avg_latency_us > 0.0);
    assert!(result.cache_performance.avg_latency_us > 0.0);
    assert!(result.batch_operations.avg_latency_us > 0.0);
    
    // 验证吞吐量
    assert!(result.vector_search.throughput_ops > 0.0);
    assert!(result.cache_performance.throughput_ops > result.vector_search.throughput_ops);
}

#[tokio::test]
async fn test_performance_manager() {
    let config = PerformanceConfig {
        cache_config: QueryCacheConfig::default(),
        batch_config: BatchConfig::default(),
        optimizer_config: Default::default(),
        enable_monitoring: false, // 禁用监控以避免后台任务
        monitoring_interval_seconds: 60,
    };
    
    let manager = PerformanceManager::new(config);
    
    // 启动管理器
    manager.start().await.unwrap();
    
    // 获取组件
    let cache = manager.get_query_cache();
    let batch_processor = manager.get_batch_processor();
    let optimizer = manager.get_optimizer();
    
    // 测试缓存
    let key = CacheKey::new("test", &"param");
    cache.put(key.clone(), vec![1, 2, 3]).await.unwrap();
    let cached: Option<Vec<i32>> = cache.get(&key).await;
    assert_eq!(cached, Some(vec![1, 2, 3]));
    
    // 测试批处理
    let items = vec![1, 2, 3];
    let results = batch_processor
        .batch_execute(items, |x| async move { Ok(x * 2) })
        .await
        .unwrap();
    assert_eq!(results, vec![2, 4, 6]);
    
    // 测试优化器
    let vector_params = optimizer.get_vector_params().await;
    assert!(vector_params.similarity_threshold > 0.0);
    
    // 获取性能统计
    let stats = manager.get_performance_stats().await;
    assert_eq!(stats.cache_stats.cache_hits, 1);
    assert_eq!(stats.batch_stats.total_batches, 1);
    
    // 停止管理器
    manager.stop().await.unwrap();
}

#[tokio::test]
async fn test_performance_optimization() {
    let config = PerformanceConfig::default();
    let manager = PerformanceManager::new(config);
    
    // 运行优化
    manager.optimize().await.unwrap();
    
    // 验证优化器统计
    let optimizer = manager.get_optimizer();
    let stats = optimizer.get_stats().await;
    
    // 优化应该已经运行
    assert!(
        stats.vector_optimizations > 0
            || stats.graph_optimizations > 0
            || stats.index_optimizations > 0
    );
}

#[tokio::test]
async fn test_cache_hit_rate() {
    let config = QueryCacheConfig::default();
    let cache = QueryCache::new(config);
    
    // 添加一些缓存项
    for i in 0..10 {
        let key = CacheKey::new("test", &i);
        cache.put(key, vec![i]).await.unwrap();
    }
    
    // 访问缓存项（命中）
    for i in 0..10 {
        let key = CacheKey::new("test", &i);
        let _: Option<Vec<i32>> = cache.get(&key).await;
    }
    
    // 访问不存在的项（未命中）
    for i in 10..15 {
        let key = CacheKey::new("test", &i);
        let _: Option<Vec<i32>> = cache.get(&key).await;
    }
    
    let stats = cache.get_stats().await;
    let hit_rate = stats.hit_rate();
    
    // 命中率应该是 10/15 = 0.666...
    assert!((hit_rate - 0.666).abs() < 0.01);
}

#[tokio::test]
async fn test_batch_stats() {
    let config = BatchConfig::default();
    let processor = BatchProcessor::new(config);
    
    // 执行多个批次
    for _ in 0..5 {
        let items = vec![1, 2, 3, 4, 5];
        let _ = processor
            .batch_execute(items, |x| async move { Ok(x * 2) })
            .await
            .unwrap();
    }
    
    let stats = processor.get_stats().await;
    assert_eq!(stats.total_batches, 5);
    assert_eq!(stats.total_items, 25);
    assert_eq!(stats.successful_batches, 5);
    assert_eq!(stats.success_rate(), 1.0);
}

