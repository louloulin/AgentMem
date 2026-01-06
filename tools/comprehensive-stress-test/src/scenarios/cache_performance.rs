//! 场景 6: 缓存性能压测

use anyhow::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::info;

use crate::monitor::SystemMonitor;
use crate::stats::{StatsCollector, StressTestStats};

pub async fn run_test(
    _cache_size_mb: usize,
    multi_progress: &MultiProgress,
) -> Result<StressTestStats> {
    info!("开始缓存性能压测");

    let total_accesses = 10000;
    let pb = multi_progress.add(ProgressBar::new(total_accesses as u64));
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.blue/blue} {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("=>-"),
    );

    let stats_collector = Arc::new(StatsCollector::new());
    let monitor = Arc::new(SystemMonitor::new());

    let stats_clone = stats_collector.clone();
    monitor
        .start_monitoring(1000, move |sys_stats| {
            let stats_clone = stats_clone.clone();
            tokio::spawn(async move {
                stats_clone
                    .record_system_stats(sys_stats.cpu_usage, sys_stats.process_memory_mb)
                    .await;
            });
        })
        .await;

    for i in 0..total_accesses {
        let op_start = Instant::now();
        let success = simulate_cache_access(i).await;
        let duration = op_start.elapsed();

        stats_collector.record_operation(duration, success).await;
        pb.inc(1);
    }

    pb.finish_with_message("缓存压测完成");
    monitor.stop_monitoring().await;

    let stats = stats_collector.get_stats().await;
    info!("缓存压测完成: 平均延迟={:.2}ms", stats.latency_mean);

    Ok(stats)
}

async fn simulate_cache_access(index: usize) -> bool {
    // 80% 缓存命中 (1ms), 20% 缓存未命中 (10ms)
    let delay_ms = if index % 5 == 0 { 10 } else { 1 };
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    true
}
