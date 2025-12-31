//! Performance Optimization Tests - Phase 2 & 3
//!
//! 测试：
//! - Phase 2: 综合评分系统（relevance + importance + recency）
//! - Phase 3: HCAM极简Prompt构建
//! - 性能指标验证
//! - Task 2.1.4: 多层缓存性能测试

use agent_mem_core::cache::multi_layer::MultiLayerCache;
use agent_mem_traits::{
    abstractions::Memory,
    Result,
};
use std::sync::Arc;

/// Task 2.1.4 测试：多层缓存性能测试
#[tokio::test]
async fn test_multi_layer_cache_performance() -> Result<()> {
    let cache = MultiLayerCache::new();

    // 测试L1缓存性能
    let start = std::time::Instant::now();
    for i in 0..1000 {
        let key = format!("test_key_{i}");
        let memory = Memory::new(
            format!("agent_{i}"),
            Some(format!("user_{i}")),
            "episodic",
            "test content",
            0.8,
        );
        cache.set_memories(key.clone(), vec![memory]);
        cache.get_memories(&key);
    }
    let l1_duration = start.elapsed();

    // 测试L2缓存性能
    let start = std::time::Instant::now();
    for i in 0..1000 {
        let key = format!("test_prompt_{i}");
        cache.set_llm_response(key.clone(), format!("response_{i}"));
        cache.get_llm_response(&key);
    }
    let l2_duration = start.elapsed();

    // 测试L3缓存性能
    let start = std::time::Instant::now();
    for i in 0..1000 {
        let key = format!("test_text_{i}");
        let embedding = vec![0.1; 384]; // 模拟384维嵌入
        cache.set_embedding(key.clone(), embedding);
        cache.get_embedding(&key);
    }
    let l3_duration = start.elapsed();

    // 获取缓存指标
    let metrics = cache.metrics();

    // 验证性能指标
    assert!(l1_duration.as_millis() < 100, "L1缓存操作应在100ms内完成");
    assert!(l2_duration.as_millis() < 100, "L2缓存操作应在100ms内完成");
    assert!(l3_duration.as_millis() < 200, "L3缓存操作应在200ms内完成");

    // 验证缓存命中率
    assert!(metrics.l1_hits > 900, "L1缓存命中率应>90%");
    assert!(metrics.l2_hits > 900, "L2缓存命中率应>90%");
    assert!(metrics.l3_hits > 900, "L3缓存命中率应>90%");

    println!(
        "L1缓存: {}次操作，耗时{:?}，命中率{:.2}%",
        metrics.l1_hits + metrics.l1_misses,
        l1_duration,
        (metrics.l1_hits as f64 / (metrics.l1_hits + metrics.l1_misses) as f64) * 100.0
    );
    println!(
        "L2缓存: {}次操作，耗时{:?}，命中率{:.2}%",
        metrics.l2_hits + metrics.l2_misses,
        l2_duration,
        (metrics.l2_hits as f64 / (metrics.l2_hits + metrics.l2_misses) as f64) * 100.0
    );
    println!(
        "L3缓存: {}次操作，耗时{:?}，命中率{:.2}%",
        metrics.l3_hits + metrics.l3_misses,
        l3_duration,
        (metrics.l3_hits as f64 / (metrics.l3_hits + metrics.l3_misses) as f64) * 100.0
    );

    Ok(())
}

/// Task 2.1.4 测试：缓存预热性能
#[tokio::test]
async fn test_cache_warming_performance() -> Result<()> {
    let cache = MultiLayerCache::new();

    // 准备测试数据
    let common_queries: Vec<String> = (0..100).map(|i| format!("常见查询 {i}")).collect();
    let common_texts: Vec<String> = (0..100).map(|i| format!("常见文本 {i}")).collect();

    // 测试预热性能
    let start = std::time::Instant::now();
    let warming_stats = cache.warm_cache(common_queries, common_texts).await?;
    let warming_duration = start.elapsed();

    // 验证预热性能
    assert!(warming_duration.as_millis() < 1000, "缓存预热应在1秒内完成");
    assert_eq!(warming_stats.total_warmings, 1, "应执行1次预热");
    assert_eq!(warming_stats.total_items_warmed, 200, "应预热200个项目");

    // 获取预热统计
    let _stats = cache.get_warming_stats();

    println!(
        "缓存预热: {}个项目，耗时{:?}，平均每个项目{:?}ms",
        warming_stats.total_items_warmed,
        warming_duration,
        warming_duration.as_millis() as f64 / warming_stats.total_items_warmed as f64
    );

    Ok(())
}

/// Task 2.1.4 测试：缓存并发性能
#[tokio::test]
async fn test_cache_concurrent_performance() -> Result<()> {
    let cache = Arc::new(MultiLayerCache::new());
    let mut handles = vec![];

    // 创建100个并发任务
    for i in 0..100 {
        let cache_clone = cache.clone();
        let handle = tokio::spawn(async move {
            let key = format!("concurrent_key_{i}");

            // L1缓存操作
            let memory = Memory::new(
                format!("agent_{i}"),
                Some(format!("user_{i}")),
                "episodic",
                "test content",
                0.8,
            );
            cache_clone.set_memories(format!("{key}_mem"), vec![memory]);
            let _mem_result = cache_clone.get_memories(&format!("{key}_mem"));

            // L2缓存操作
            cache_clone.set_llm_response(format!("{key}_llm"), format!("response_{i}"));
            let _llm_result = cache_clone.get_llm_response(&format!("{key}_llm"));

            // L3缓存操作
            let embedding = vec![0.1; 384];
            cache_clone.set_embedding(format!("{key}_emb"), embedding);
            let _emb_result = cache_clone.get_embedding(&format!("{key}_emb"));
        });
        handles.push(handle);
    }

    // 等待所有任务完成
    let start = std::time::Instant::now();
    for handle in handles {
        handle.await.unwrap();
    }
    let concurrent_duration = start.elapsed();

    // 验证并发性能
    assert!(
        concurrent_duration.as_millis() < 500,
        "100个并发操作应在500ms内完成"
    );

    let _metrics = cache.metrics();
    println!(
        "并发测试: 300次操作，耗时{:?}，平均每次{:?}ms",
        concurrent_duration,
        concurrent_duration.as_millis() as f64 / 300.0
    );

    Ok(())
}
