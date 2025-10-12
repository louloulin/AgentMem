//! AgentMem 性能优化演示
//!
//! 演示查询缓存、批量处理、性能优化器和基准测试功能

use agent_mem_core::performance::{
    BatchConfig, BatchProcessor, BenchmarkConfig, CacheKey, PerformanceBenchmark,
    PerformanceConfig, PerformanceManager, QueryCache, QueryCacheConfig,
};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();
    
    info!("=== AgentMem 性能优化演示 ===\n");
    
    // 演示 1: 查询缓存
    demo_query_cache().await?;
    
    // 演示 2: 批量处理
    demo_batch_processing().await?;
    
    // 演示 3: 性能基准测试
    demo_performance_benchmark().await?;
    
    // 演示 4: 性能管理器
    demo_performance_manager().await?;
    
    info!("\n=== 演示完成 ===");
    
    Ok(())
}

/// 演示查询缓存功能
async fn demo_query_cache() -> Result<(), Box<dyn std::error::Error>> {
    info!("--- 1. 查询缓存演示 ---");
    
    let config = QueryCacheConfig {
        max_entries: 1000,
        default_ttl_seconds: 300,
        enable_lru: true,
        enable_warmup: false,
        warmup_batch_size: 100,
    };
    
    let cache = QueryCache::new(config);
    info!("✓ 创建查询缓存（最大 1000 条目，TTL 300 秒）");
    
    // 添加一些缓存项
    for i in 0..100 {
        let key = CacheKey::new("vector_search", &format!("query_{}", i));
        let value: Vec<i32> = (0..10).map(|j| i * 10 + j).collect();
        cache.put(key, value).await?;
    }
    info!("✓ 添加 100 个缓存项");
    
    // 测试缓存命中
    let mut hits = 0;
    let mut misses = 0;
    
    for i in 0..150 {
        let key = CacheKey::new("vector_search", &format!("query_{}", i));
        let result: Option<Vec<i32>> = cache.get(&key).await;
        
        if result.is_some() {
            hits += 1;
        } else {
            misses += 1;
        }
    }
    
    info!("✓ 缓存命中: {}, 未命中: {}", hits, misses);
    
    // 获取统计信息
    let stats = cache.get_stats().await;
    info!("✓ 缓存统计:");
    info!("  - 总请求数: {}", stats.total_requests);
    info!("  - 缓存命中数: {}", stats.cache_hits);
    info!("  - 缓存未命中数: {}", stats.cache_misses);
    info!("  - 命中率: {:.2}%", stats.hit_rate() * 100.0);
    info!("  - 当前条目数: {}\n", stats.total_entries);
    
    Ok(())
}

/// 演示批量处理功能
async fn demo_batch_processing() -> Result<(), Box<dyn std::error::Error>> {
    info!("--- 2. 批量处理演示 ---");
    
    let config = BatchConfig {
        batch_size: 50,
        max_concurrency: 10,
        batch_timeout_seconds: 30,
        enable_auto_batching: true,
        auto_batch_delay_ms: 100,
    };
    
    let processor = BatchProcessor::new(config);
    info!("✓ 创建批量处理器（批量大小 50，最大并发 10）");
    
    // 批量向量搜索
    let query_vectors: Vec<Vec<f32>> = (0..200)
        .map(|i| (0..128).map(|j| (i + j) as f32 / 1000.0).collect())
        .collect();
    
    info!("✓ 准备 200 个查询向量");
    
    let start = std::time::Instant::now();
    let results: Vec<f32> = processor
        .batch_vector_search(query_vectors, |v| async move {
            // 模拟向量搜索
            Ok(v.iter().sum::<f32>())
        })
        .await?;
    
    let duration = start.elapsed();
    
    info!("✓ 批量向量搜索完成");
    info!("  - 处理 {} 个查询", results.len());
    info!("  - 总耗时: {:.2} ms", duration.as_millis());
    info!("  - 平均耗时: {:.2} ms/查询", duration.as_millis() as f64 / results.len() as f64);
    info!("  - 吞吐量: {:.2} 查询/秒", results.len() as f64 / duration.as_secs_f64());
    
    // 批量图查询
    let node_ids: Vec<String> = (0..100).map(|i| format!("node_{}", i)).collect();
    
    let graph_results: Vec<usize> = processor
        .batch_graph_query(node_ids, |id| async move {
            // 模拟图查询
            Ok(id.len())
        })
        .await?;
    
    info!("✓ 批量图查询完成（处理 {} 个节点）", graph_results.len());
    
    // 获取统计信息
    let stats = processor.get_stats().await;
    info!("✓ 批量处理统计:");
    info!("  - 总批次数: {}", stats.total_batches);
    info!("  - 成功批次数: {}", stats.successful_batches);
    info!("  - 总处理项目数: {}", stats.total_items);
    info!("  - 平均处理时间: {:.2} ms", stats.average_processing_time_ms());
    info!("  - 成功率: {:.2}%\n", stats.success_rate() * 100.0);
    
    Ok(())
}

/// 演示性能基准测试
async fn demo_performance_benchmark() -> Result<(), Box<dyn std::error::Error>> {
    info!("--- 3. 性能基准测试演示 ---");
    
    let config = BenchmarkConfig {
        iterations: 1000,
        warmup_iterations: 100,
        vector_dimension: 384,
        dataset_size: 10000,
        verbose: false,
    };
    
    let mut benchmark = PerformanceBenchmark::new(config);
    info!("✓ 创建性能基准测试（1000 次迭代，100 次预热）");
    
    info!("✓ 运行基准测试...");
    let result = benchmark.run_all_benchmarks().await?;
    
    info!("✓ 基准测试完成\n");
    
    info!("向量搜索性能:");
    info!("  - 平均延迟: {:.2} μs", result.vector_search.avg_latency_us);
    info!("  - 最小延迟: {:.2} μs", result.vector_search.min_latency_us);
    info!("  - 最大延迟: {:.2} μs", result.vector_search.max_latency_us);
    info!("  - P50 延迟: {:.2} μs", result.vector_search.p50_latency_us);
    info!("  - P95 延迟: {:.2} μs", result.vector_search.p95_latency_us);
    info!("  - P99 延迟: {:.2} μs", result.vector_search.p99_latency_us);
    info!("  - 吞吐量: {:.2} ops/s", result.vector_search.throughput_ops);
    
    info!("\n图查询性能:");
    info!("  - 平均延迟: {:.2} μs", result.graph_query.avg_latency_us);
    info!("  - P95 延迟: {:.2} μs", result.graph_query.p95_latency_us);
    info!("  - 吞吐量: {:.2} ops/s", result.graph_query.throughput_ops);
    
    info!("\n缓存性能:");
    info!("  - 平均延迟: {:.2} μs", result.cache_performance.avg_latency_us);
    info!("  - P95 延迟: {:.2} μs", result.cache_performance.p95_latency_us);
    info!("  - 吞吐量: {:.2} ops/s", result.cache_performance.throughput_ops);
    
    info!("\n批量操作性能:");
    info!("  - 平均延迟: {:.2} μs", result.batch_operations.avg_latency_us);
    info!("  - P95 延迟: {:.2} μs", result.batch_operations.p95_latency_us);
    info!("  - 吞吐量: {:.2} ops/s\n", result.batch_operations.throughput_ops);
    
    // 性能对比
    let cache_speedup = result.vector_search.avg_latency_us / result.cache_performance.avg_latency_us;
    info!("性能提升:");
    info!("  - 缓存加速比: {:.2}x", cache_speedup);
    
    let batch_speedup = result.vector_search.avg_latency_us / result.batch_operations.avg_latency_us;
    info!("  - 批量处理加速比: {:.2}x\n", batch_speedup);
    
    Ok(())
}

/// 演示性能管理器
async fn demo_performance_manager() -> Result<(), Box<dyn std::error::Error>> {
    info!("--- 4. 性能管理器演示 ---");
    
    let config = PerformanceConfig {
        cache_config: QueryCacheConfig {
            max_entries: 1000,
            default_ttl_seconds: 300,
            enable_lru: true,
            enable_warmup: false,
            warmup_batch_size: 100,
        },
        batch_config: BatchConfig {
            batch_size: 50,
            max_concurrency: 10,
            batch_timeout_seconds: 30,
            enable_auto_batching: true,
            auto_batch_delay_ms: 100,
        },
        optimizer_config: Default::default(),
        enable_monitoring: false, // 禁用后台监控
        monitoring_interval_seconds: 60,
    };
    
    let manager = PerformanceManager::new(config);
    info!("✓ 创建性能管理器");
    
    // 启动管理器
    manager.start().await?;
    info!("✓ 启动性能管理器");
    
    // 获取组件
    let cache = manager.get_query_cache();
    let batch_processor = manager.get_batch_processor();
    let optimizer = manager.get_optimizer();
    
    // 使用缓存
    for i in 0..50 {
        let key = CacheKey::new("test", &i);
        cache.put(key, vec![i]).await?;
    }
    info!("✓ 添加 50 个缓存项");
    
    // 使用批处理
    let items: Vec<i32> = (0..100).collect();
    let _results = batch_processor
        .batch_execute(items, |x| async move { Ok(x * 2) })
        .await?;
    info!("✓ 批量处理 100 个项目");
    
    // 记录查询
    use agent_mem_core::performance::optimizer::QueryType;
    for _ in 0..20 {
        optimizer.record_query(QueryType::VectorSearch, 50).await;
    }
    for _ in 0..10 {
        optimizer.record_query(QueryType::GraphQuery, 100).await;
    }
    info!("✓ 记录 30 个查询");
    
    // 运行优化
    manager.optimize().await?;
    info!("✓ 运行性能优化");
    
    // 获取优化参数
    let vector_params = optimizer.get_vector_params().await;
    let graph_params = optimizer.get_graph_params().await;
    
    info!("✓ 优化参数:");
    info!("  - 向量相似度阈值: {:.2}", vector_params.similarity_threshold);
    info!("  - 向量最大结果数: {}", vector_params.max_results);
    info!("  - 图最大深度: {}", graph_params.max_depth);
    info!("  - 图最大节点数: {}", graph_params.max_nodes);
    
    // 获取综合统计
    let stats = manager.get_performance_stats().await;
    info!("✓ 性能统计:");
    info!("  - 缓存命中率: {:.2}%", stats.cache_stats.hit_rate() * 100.0);
    info!("  - 批处理成功率: {:.2}%", stats.batch_stats.success_rate() * 100.0);
    info!("  - 向量优化次数: {}", stats.optimizer_stats.vector_optimizations);
    info!("  - 图优化次数: {}", stats.optimizer_stats.graph_optimizations);
    
    // 停止管理器
    manager.stop().await?;
    info!("✓ 停止性能管理器\n");
    
    Ok(())
}

