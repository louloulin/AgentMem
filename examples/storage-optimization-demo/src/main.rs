//! 存储优化系统演示程序
//!
//! 演示 Phase 4.1 存储优化功能，包括：
//! - 多维索引和查询优化
//! - 向量压缩和量化
//! - 智能数据分片和路由
//! - 多级缓存和预热机制
//! - 对象池和内存复用

use agent_mem_compat::storage_optimization::{
    StorageOptimizationConfig, StorageOptimizationManager,
};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    info!("🚀 启动存储优化系统演示");

    // 创建存储优化管理器
    let config = StorageOptimizationConfig::default();
    let storage_manager = StorageOptimizationManager::new(config).await?;
    info!("✅ 存储优化管理器创建成功");

    // 演示索引优化功能
    info!("🔍 演示索引优化功能");
    demo_index_optimization(&storage_manager).await?;

    // 演示压缩功能
    info!("🗜️ 演示数据压缩功能");
    demo_compression(&storage_manager).await?;

    // 演示分片路由功能
    info!("🔀 演示分片路由功能");
    demo_sharding(&storage_manager).await?;

    // 演示缓存功能
    info!("💾 演示多级缓存功能");
    demo_caching(&storage_manager).await?;

    // 演示内存池功能
    info!("🧠 演示内存池管理功能");
    demo_memory_pool(&storage_manager).await?;

    // 启动存储优化系统
    info!("🔄 启动存储优化系统");
    storage_manager.start().await?;

    // 获取系统统计信息
    info!("📊 获取存储优化统计信息");
    let stats = storage_manager.get_optimization_stats().await?;
    display_optimization_stats(&stats);

    // 运行系统一段时间
    info!("⏱️  运行存储优化系统 30 秒...");
    sleep(Duration::from_secs(30)).await;

    // 停止系统
    info!("⏹️  停止存储优化系统");
    storage_manager.stop().await?;

    info!("🎉 存储优化系统演示完成！");
    Ok(())
}

async fn demo_index_optimization(
    manager: &StorageOptimizationManager,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("优化查询计划...");

    let queries = vec![
        "SELECT * FROM memories WHERE user_id = 'user123'",
        "SELECT * FROM memories WHERE importance > 0.8",
        "SELECT * FROM memories WHERE content LIKE '%AI%'",
    ];

    for query in queries {
        let plan = manager.optimize_query(query).await?;
        info!(
            "✅ 查询优化完成: {} -> 计划ID: {}, 预估成本: {:.2}",
            query, plan.plan_id, plan.estimated_cost
        );
        info!("   📋 执行步骤: {} 步", plan.execution_steps.len());
        for (i, step) in plan.execution_steps.iter().enumerate() {
            info!(
                "     {}. {:?} (预估行数: {}, 成本: {:.2})",
                i + 1,
                step.step_type,
                step.estimated_rows,
                step.estimated_cost
            );
        }
    }

    Ok(())
}

async fn demo_compression(
    manager: &StorageOptimizationManager,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("测试数据压缩...");

    let test_data = b"This is a test data for compression. It contains some repetitive patterns that should compress well. This is a test data for compression.";

    info!("原始数据大小: {} 字节", test_data.len());

    let compressed = manager.compress_data(test_data).await?;
    info!(
        "✅ 压缩完成: {} 字节 -> {} 字节",
        test_data.len(),
        compressed.len()
    );

    let decompressed = manager.decompress_data(&compressed).await?;
    info!("✅ 解压完成: {} 字节", decompressed.len());

    if test_data == decompressed.as_slice() {
        info!("✅ 数据完整性验证通过");
    } else {
        warn!("⚠️  数据完整性验证失败");
    }

    Ok(())
}

async fn demo_sharding(
    manager: &StorageOptimizationManager,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("测试分片路由...");

    let test_keys = vec![
        "user_123_memory_001",
        "user_456_memory_002",
        "user_789_memory_003",
        "user_abc_memory_004",
        "user_def_memory_005",
    ];

    for key in test_keys {
        let shard = manager.get_shard_route(key).await?;
        info!("✅ 路由完成: {} -> {}", key, shard);
    }

    Ok(())
}

async fn demo_caching(
    manager: &StorageOptimizationManager,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("测试多级缓存...");

    let test_data = b"cached_data_example";
    let cache_key = "test_cache_key";

    // 缓存到 L1
    manager.cache_data(cache_key, test_data, "L1").await?;
    info!("✅ 数据已缓存到 L1: {}", cache_key);

    // 缓存到 L2
    manager.cache_data(cache_key, test_data, "L2").await?;
    info!("✅ 数据已缓存到 L2: {}", cache_key);

    // 尝试获取缓存数据
    let cached_data = manager.get_cached_data(cache_key).await?;
    match cached_data {
        Some(data) => info!("✅ 缓存命中: {} 字节", data.len()),
        None => info!("❌ 缓存未命中"),
    }

    Ok(())
}

async fn demo_memory_pool(
    manager: &StorageOptimizationManager,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("测试内存池管理...");

    // 分配向量对象
    let vector_objects = [manager.allocate_object("Vector").await?,
        manager.allocate_object("Vector").await?,
        manager.allocate_object("Vector").await?];
    info!("✅ 分配了 {} 个向量对象", vector_objects.len());

    // 分配内存对象
    let memory_objects = vec![
        manager.allocate_object("Memory").await?,
        manager.allocate_object("Memory").await?,
    ];
    info!("✅ 分配了 {} 个内存对象", memory_objects.len());

    // 释放部分对象
    for &obj_id in &vector_objects[0..2] {
        manager.deallocate_object("Vector", obj_id).await?;
    }
    info!("✅ 释放了 2 个向量对象");

    for &obj_id in &memory_objects {
        manager.deallocate_object("Memory", obj_id).await?;
    }
    info!("✅ 释放了 {} 个内存对象", memory_objects.len());

    Ok(())
}

fn display_optimization_stats(
    stats: &agent_mem_compat::storage_optimization::StorageOptimizationStats,
) {
    info!("📈 存储优化统计信息:");

    // 整体性能
    info!("🎯 整体性能评分:");
    info!(
        "   - 查询性能: {:.1}%",
        stats.overall_performance.query_performance_score
    );
    info!(
        "   - 存储效率: {:.1}%",
        stats.overall_performance.storage_efficiency_score
    );
    info!(
        "   - 缓存效率: {:.1}%",
        stats.overall_performance.cache_efficiency_score
    );
    info!(
        "   - 内存利用率: {:.1}%",
        stats.overall_performance.memory_utilization_score
    );
    info!(
        "   - 总体评分: {:.1}%",
        stats.overall_performance.overall_score
    );

    // 索引统计
    info!("📚 索引统计 ({} 个):", stats.index_stats.len());
    for index in &stats.index_stats {
        info!(
            "   - {}: {:?}, 大小: {:.1}MB, 命中率: {:.1}%, 查询时间: {:.2}ms",
            index.name,
            index.index_type,
            index.size_bytes as f64 / 1024.0 / 1024.0,
            index.hit_rate * 100.0,
            index.avg_query_time_ms
        );
    }

    // 压缩统计
    info!("🗜️ 压缩统计 ({} 个):", stats.compression_stats.len());
    for comp in &stats.compression_stats {
        info!(
            "   - {:?}: {:.1}MB -> {:.1}MB (压缩比: {:.1}%), 压缩时间: {:.2}ms",
            comp.algorithm,
            comp.original_size_bytes as f64 / 1024.0 / 1024.0,
            comp.compressed_size_bytes as f64 / 1024.0 / 1024.0,
            comp.compression_ratio * 100.0,
            comp.compression_time_ms
        );
    }

    // 分片统计
    info!("🔀 分片统计 ({} 个):", stats.sharding_stats.len());
    for shard in &stats.sharding_stats {
        info!(
            "   - {}: {:?}, 大小: {:.1}MB, 记录数: {}, 负载: {:.1}%, 副本: {}",
            shard.shard_id,
            shard.status,
            shard.data_size_bytes as f64 / 1024.0 / 1024.0,
            shard.record_count,
            shard.load_score * 100.0,
            shard.replicas.len()
        );
    }

    // 缓存统计
    info!("💾 缓存统计 ({} 层):", stats.cache_stats.len());
    for cache in &stats.cache_stats {
        info!(
            "   - {}: {:?}, 命中率: {:.1}%, 使用率: {:.1}%, 访问时间: {:.2}ms",
            cache.level_name,
            cache.cache_type,
            cache.hit_rate * 100.0,
            cache.usage_ratio * 100.0,
            cache.avg_access_time_ms
        );
    }

    // 内存池统计
    info!("🧠 内存池统计 ({} 个):", stats.memory_pool_stats.len());
    for pool in &stats.memory_pool_stats {
        info!(
            "   - {}: 池大小: {}, 使用率: {:.1}%, 池命中率: {:.1}%",
            pool.object_type,
            pool.pool_size,
            pool.usage_ratio * 100.0,
            pool.pool_hit_rate * 100.0
        );
    }
}
