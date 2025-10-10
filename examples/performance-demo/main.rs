//! 性能优化和缓存演示
//!
//! 演示 AgentMem 的性能监控、缓存机制和优化功能

use agent_mem_storage::{
    cache::{CacheConfig, CachedVectorStore},
    performance::{MonitoredVectorStore, PerformanceMonitor},
    VectorStoreFactory,
};
use agent_mem_traits::{VectorData, VectorStoreConfig};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{sleep, Instant};
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    info!("AgentMem 性能优化和缓存演示");

    // 创建基础存储
    let base_config = VectorStoreConfig {
        provider: "memory".to_string(),
        dimension: Some(128),
        ..Default::default()
    };
    let base_store = VectorStoreFactory::create_vector_store(&base_config).await?;

    // 创建性能监控器
    let monitor = Arc::new(PerformanceMonitor::new(
        10000,                   // 最大历史记录数
        Duration::from_secs(60), // 缓存TTL
    ));

    // 创建带性能监控的存储
    let monitored_store = Arc::new(MonitoredVectorStore::new(base_store, monitor.clone()));

    // 创建缓存配置
    let cache_config = CacheConfig {
        max_entries: 1000,
        default_ttl_seconds: Some(300), // 5分钟TTL
        enable_lru: true,
        stats_interval_seconds: 60,
        enable_warmup: false,
        warmup_batch_size: 50,
    };

    // 创建带缓存的存储
    let cached_store_impl = Arc::new(CachedVectorStore::new(monitored_store, cache_config));
    let cached_store: Arc<dyn agent_mem_traits::VectorStore + Send + Sync> =
        cached_store_impl.clone();

    info!("=== 性能基准测试 ===");

    // 准备测试数据
    let mut test_vectors = Vec::new();
    for i in 0..1000 {
        let mut metadata = HashMap::new();
        metadata.insert("category".to_string(), format!("category_{}", i % 10));
        metadata.insert("index".to_string(), i.to_string());

        test_vectors.push(VectorData {
            id: format!("perf_test_{}", i),
            vector: (0..128).map(|j| (i + j) as f32 / 1000.0).collect(),
            metadata,
        });
    }

    // 测试1: 批量插入性能
    info!("测试1: 批量插入性能");
    let start_time = Instant::now();

    let batch_size = 100;
    for chunk in test_vectors.chunks(batch_size) {
        cached_store.add_vectors(chunk.to_vec()).await?;
    }

    let insert_duration = start_time.elapsed();
    info!(
        "插入 {} 条记录耗时: {:.2}ms",
        test_vectors.len(),
        insert_duration.as_millis()
    );
    info!(
        "插入速度: {:.2} 记录/秒",
        test_vectors.len() as f64 / insert_duration.as_secs_f64()
    );

    // 测试2: 搜索性能（冷缓存）
    info!("\n测试2: 搜索性能（冷缓存）");
    let query_vector = vec![0.5; 128];
    let search_start = Instant::now();

    let search_results = cached_store
        .search_vectors(query_vector.clone(), 10, Some(0.8))
        .await?;
    let cold_search_duration = search_start.elapsed();

    info!("冷缓存搜索耗时: {:.2}ms", cold_search_duration.as_millis());
    info!("找到 {} 条结果", search_results.len());

    // 测试3: 搜索性能（热缓存）
    info!("\n测试3: 搜索性能（热缓存）");
    let hot_search_start = Instant::now();

    let _cached_results = cached_store
        .search_vectors(query_vector.clone(), 10, Some(0.8))
        .await?;
    let hot_search_duration = hot_search_start.elapsed();

    info!("热缓存搜索耗时: {:.2}ms", hot_search_duration.as_millis());
    info!(
        "缓存加速比: {:.2}x",
        cold_search_duration.as_secs_f64() / hot_search_duration.as_secs_f64()
    );

    // 测试4: 随机访问性能
    info!("\n测试4: 随机访问性能");
    let random_access_start = Instant::now();

    for i in (0..100).step_by(10) {
        let id = format!("perf_test_{}", i);
        let _vector = cached_store.get_vector(&id).await?;
    }

    let random_access_duration = random_access_start.elapsed();
    info!(
        "随机访问 10 条记录耗时: {:.2}ms",
        random_access_duration.as_millis()
    );

    // 测试5: 重复访问（缓存命中）
    info!("\n测试5: 重复访问（缓存命中）");
    let repeat_access_start = Instant::now();

    for i in (0..100).step_by(10) {
        let id = format!("perf_test_{}", i);
        let _vector = cached_store.get_vector(&id).await?;
    }

    let repeat_access_duration = repeat_access_start.elapsed();
    info!(
        "重复访问 10 条记录耗时: {:.2}ms",
        repeat_access_duration.as_millis()
    );
    info!(
        "缓存加速比: {:.2}x",
        random_access_duration.as_secs_f64() / repeat_access_duration.as_secs_f64()
    );

    // 等待一段时间让性能指标收集完成
    sleep(Duration::from_millis(100)).await;

    // 显示性能统计
    info!("\n=== 性能统计报告 ===");
    let report = monitor.generate_report().await;

    info!("总指标数量: {}", report.total_metrics);

    for (operation, stats) in &report.operation_stats {
        info!("\n操作: {}", operation);
        info!("  总请求数: {}", stats.total_requests);
        info!("  成功率: {:.2}%", stats.success_rate * 100.0);
        info!("  平均响应时间: {:.2}ms", stats.avg_duration_ms);
        info!("  P50响应时间: {:.2}ms", stats.p50_duration_ms);
        info!("  P95响应时间: {:.2}ms", stats.p95_duration_ms);
        info!("  P99响应时间: {:.2}ms", stats.p99_duration_ms);
        info!("  吞吐量: {:.2} 请求/秒", stats.throughput_rps);

        if stats.avg_data_size_bytes > 0.0 {
            info!(
                "  平均数据大小: {:.2} KB",
                stats.avg_data_size_bytes / 1024.0
            );
            info!(
                "  总数据传输: {:.2} MB",
                stats.total_data_bytes as f64 / (1024.0 * 1024.0)
            );
        }
    }

    // 显示缓存统计
    info!("\n=== 缓存统计报告 ===");
    let (vector_cache_stats, search_cache_stats) =
        cached_store_impl.cache_manager().get_stats().await;

    info!("向量缓存:");
    info!("  总请求数: {}", vector_cache_stats.total_requests);
    info!("  缓存命中数: {}", vector_cache_stats.cache_hits);
    info!("  缓存命中率: {:.2}%", vector_cache_stats.hit_rate * 100.0);
    info!("  当前条目数: {}", vector_cache_stats.current_entries);
    info!(
        "  缓存使用率: {:.2}%",
        vector_cache_stats.usage_rate * 100.0
    );
    info!("  过期条目数: {}", vector_cache_stats.expired_entries);
    info!("  淘汰条目数: {}", vector_cache_stats.evicted_entries);

    info!("\n搜索结果缓存:");
    info!("  总请求数: {}", search_cache_stats.total_requests);
    info!("  缓存命中数: {}", search_cache_stats.cache_hits);
    info!("  缓存命中率: {:.2}%", search_cache_stats.hit_rate * 100.0);
    info!("  当前条目数: {}", search_cache_stats.current_entries);
    info!(
        "  缓存使用率: {:.2}%",
        search_cache_stats.usage_rate * 100.0
    );

    // 测试6: 缓存清理
    info!("\n测试6: 缓存清理");
    let (vector_expired, search_expired) =
        cached_store_impl.cache_manager().cleanup_expired().await;
    info!(
        "清理过期条目: 向量缓存 {} 条, 搜索缓存 {} 条",
        vector_expired, search_expired
    );

    // 性能优化建议
    info!("\n=== 性能优化建议 ===");

    let add_vectors_stats = report.operation_stats.get("add_vectors");
    let search_stats = report.operation_stats.get("search_vectors");

    if let Some(stats) = add_vectors_stats {
        if stats.avg_duration_ms > 100.0 {
            warn!(
                "建议: 插入操作平均耗时较高 ({:.2}ms)，考虑增加批次大小或使用异步插入",
                stats.avg_duration_ms
            );
        }

        if stats.p95_duration_ms > stats.avg_duration_ms * 2.0 {
            warn!("建议: 插入操作P95响应时间较高，可能存在性能瓶颈");
        }
    }

    if let Some(stats) = search_stats {
        if stats.avg_duration_ms > 50.0 {
            warn!(
                "建议: 搜索操作平均耗时较高 ({:.2}ms)，考虑优化索引或增加缓存",
                stats.avg_duration_ms
            );
        }
    }

    if vector_cache_stats.hit_rate < 0.5 {
        warn!(
            "建议: 向量缓存命中率较低 ({:.2}%)，考虑增加缓存大小或调整TTL",
            vector_cache_stats.hit_rate * 100.0
        );
    }

    if search_cache_stats.hit_rate < 0.3 {
        warn!(
            "建议: 搜索缓存命中率较低 ({:.2}%)，考虑优化查询模式或增加缓存大小",
            search_cache_stats.hit_rate * 100.0
        );
    }

    info!("\n性能优化和缓存演示完成");
    Ok(())
}
