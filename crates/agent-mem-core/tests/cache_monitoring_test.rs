//! 缓存性能监控集成测试

use agent_mem_core::cache::{
    Cache, CacheLevel, CacheMonitor, CacheStats, MonitorConfig, MultiLevelCache,
    MultiLevelCacheConfig,
};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_monitor_basic_operations() {
    let config = MonitorConfig {
        enabled: true,
        snapshot_interval_secs: 1,
        max_snapshots: 10,
        response_time_window: 100,
        slow_query_threshold_ms: 50.0,
        enable_slow_query_log: true,
        enable_alerts: false,
        hit_rate_alert_threshold: 30.0,
    };

    let monitor = CacheMonitor::new(config);

    // 记录一些操作
    for i in 0..10 {
        let duration = Duration::from_millis(10 + i * 5);
        let hit = i % 3 != 0; // 部分命中
        let level = if hit { Some(CacheLevel::L1) } else { None };

        monitor.record_operation(duration, hit, level).await;
    }

    // 创建快照
    let stats = CacheStats {
        total_gets: 10,
        hits: 7,
        misses: 3,
        total_sets: 0,
        evictions: 0,
        invalidations: 0,
        total_size_bytes: 0,
        entry_count: 0,
    };

    let snapshot = monitor
        .create_snapshot(Some(stats.clone()), None, stats)
        .await;

    // 验证快照
    assert!(snapshot.avg_response_time_ms > 0.0);
    assert_eq!(snapshot.combined_stats.hit_rate(), 70.0);

    // 保存快照
    monitor.save_snapshot(snapshot).await;

    // 获取最新快照
    let latest = monitor.latest_snapshot().await;
    assert!(latest.is_some());
}

#[tokio::test]
async fn test_slow_query_detection() {
    let config = MonitorConfig {
        enabled: true,
        slow_query_threshold_ms: 50.0,
        enable_slow_query_log: true,
        ..Default::default()
    };

    let monitor = CacheMonitor::new(config);

    // 记录正常查询
    monitor
        .record_operation(Duration::from_millis(30), true, Some(CacheLevel::L1))
        .await;

    // 记录慢查询
    monitor
        .record_operation(Duration::from_millis(100), true, Some(CacheLevel::L2))
        .await;

    monitor
        .record_operation(Duration::from_millis(150), false, None)
        .await;

    // 验证慢查询计数
    assert_eq!(monitor.slow_query_count().await, 2);

    // 重置计数
    monitor.reset_slow_query_count().await;
    assert_eq!(monitor.slow_query_count().await, 0);
}

#[tokio::test]
async fn test_performance_report_generation() {
    let config = MonitorConfig {
        enabled: true,
        max_snapshots: 100,
        ..Default::default()
    };

    let monitor = CacheMonitor::new(config);

    // 创建多个快照，模拟一段时间的运行
    for i in 0..5 {
        // 模拟不同的性能表现
        let hit_rate = 60.0 + (i as f64 * 5.0);
        let total_gets = 100;
        let hits = (total_gets as f64 * hit_rate / 100.0) as u64;

        let stats = CacheStats {
            total_gets,
            hits,
            misses: total_gets - hits,
            total_sets: 50,
            evictions: i as u64,
            invalidations: 0,
            total_size_bytes: 1024 * 1024,
            entry_count: 100,
        };

        // 记录一些操作
        for _ in 0..10 {
            monitor
                .record_operation(Duration::from_millis(20), true, Some(CacheLevel::L1))
                .await;
        }

        let snapshot = monitor
            .create_snapshot(Some(stats.clone()), None, stats)
            .await;

        monitor.save_snapshot(snapshot).await;

        // 小延迟以产生不同的时间戳
        sleep(Duration::from_millis(100)).await;
    }

    // 生成报告
    let report = monitor.generate_report().await;
    assert!(report.is_some());

    let report = report.unwrap();
    assert_eq!(report.total_snapshots, 5);
    assert!(report.avg_hit_rate > 0.0);
    assert!(report.hit_rate_trend != 0.0); // 应该有趋势
    assert!(!report.recommendations.is_empty());

    // 验证报告格式化
    let text = report.format_text();
    assert!(text.contains("缓存性能报告"));
    assert!(text.contains("命中率"));

    let json = report.format_json();
    assert!(json.is_ok());
}

#[tokio::test]
async fn test_multi_level_cache_with_monitoring() {
    let config = MultiLevelCacheConfig {
        enable_l1: true,
        enable_l2: false,
        enable_monitoring: true,
        monitor_config: Some(MonitorConfig {
            enabled: true,
            slow_query_threshold_ms: 50.0,
            enable_alerts: false,
            ..Default::default()
        }),
        ..Default::default()
    };

    let cache = MultiLevelCache::new(config);

    // 执行一些缓存操作
    cache
        .set("key1".to_string(), b"value1".to_vec(), None)
        .await
        .unwrap();
    cache
        .set("key2".to_string(), b"value2".to_vec(), None)
        .await
        .unwrap();

    // 命中
    let _ = cache.get(&"key1".to_string()).await.unwrap();
    let _ = cache.get(&"key2".to_string()).await.unwrap();

    // 未命中
    let _ = cache.get(&"key3".to_string()).await.unwrap();

    // 获取性能快照
    let snapshot = cache.performance_snapshot().await.unwrap();
    assert!(snapshot.is_some());

    let snapshot = snapshot.unwrap();
    assert_eq!(snapshot.combined_stats.total_gets, 3);
    assert_eq!(snapshot.combined_stats.hits, 2);
    assert_eq!(snapshot.combined_stats.misses, 1);

    // 验证监控器可访问
    let monitor = cache.monitor();
    assert!(monitor.is_some());
}

#[tokio::test]
async fn test_monitor_recommendations() {
    let config = MonitorConfig {
        enabled: true,
        hit_rate_alert_threshold: 70.0,
        ..Default::default()
    };

    let monitor = CacheMonitor::new(config);

    // 场景1: 低命中率
    let low_hit_stats = CacheStats {
        total_gets: 100,
        hits: 40,
        misses: 60,
        total_sets: 50,
        evictions: 0,
        invalidations: 0,
        total_size_bytes: 1024,
        entry_count: 50,
    };

    let snapshot1 = monitor
        .create_snapshot(Some(low_hit_stats.clone()), None, low_hit_stats)
        .await;
    monitor.save_snapshot(snapshot1).await;

    // 场景2: 高命中率
    let high_hit_stats = CacheStats {
        total_gets: 100,
        hits: 90,
        misses: 10,
        total_sets: 50,
        evictions: 0,
        invalidations: 0,
        total_size_bytes: 1024,
        entry_count: 50,
    };

    let snapshot2 = monitor
        .create_snapshot(Some(high_hit_stats.clone()), None, high_hit_stats)
        .await;
    monitor.save_snapshot(snapshot2).await;

    sleep(Duration::from_millis(100)).await;

    // 生成报告并检查建议
    let report = monitor.generate_report().await.unwrap();

    // 应该包含关于命中率提升的建议
    assert!(!report.recommendations.is_empty());

    // 验证趋势计算
    assert!(report.hit_rate_trend > 0.0); // 从低到高
}

#[tokio::test]
async fn test_percentile_calculations() {
    let config = MonitorConfig::default();
    let monitor = CacheMonitor::new(config);

    // 记录一系列不同的响应时间
    let durations = vec![10, 20, 30, 40, 50, 60, 70, 80, 90, 100];

    for duration_ms in durations {
        monitor
            .record_operation(
                Duration::from_millis(duration_ms),
                true,
                Some(CacheLevel::L1),
            )
            .await;
    }

    let stats = CacheStats {
        total_gets: 10,
        hits: 10,
        misses: 0,
        total_sets: 0,
        evictions: 0,
        invalidations: 0,
        total_size_bytes: 0,
        entry_count: 0,
    };

    let snapshot = monitor
        .create_snapshot(Some(stats.clone()), None, stats)
        .await;

    // 验证百分位数
    assert!(snapshot.p50_response_time_ms > 0.0);
    assert!(snapshot.p95_response_time_ms > snapshot.p50_response_time_ms);
    assert!(snapshot.p99_response_time_ms >= snapshot.p95_response_time_ms);
    assert!(snapshot.avg_response_time_ms > 0.0);
}

#[tokio::test]
async fn test_monitoring_can_be_disabled() {
    let config = MultiLevelCacheConfig {
        enable_l1: true,
        enable_l2: false,
        enable_monitoring: false, // 禁用监控
        ..Default::default()
    };

    let cache = MultiLevelCache::new(config);

    // 执行操作
    cache
        .set("key1".to_string(), b"value1".to_vec(), None)
        .await
        .unwrap();
    let _ = cache.get(&"key1".to_string()).await.unwrap();

    // 监控器应该不存在
    assert!(cache.monitor().is_none());

    // 性能快照应该返回 None
    let snapshot = cache.performance_snapshot().await.unwrap();
    assert!(snapshot.is_none());

    // 性能报告应该返回 None
    let report = cache.performance_report().await.unwrap();
    assert!(report.is_none());
}
