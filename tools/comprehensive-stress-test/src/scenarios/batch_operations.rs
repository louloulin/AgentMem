//! 场景 7: 批量操作压测

use anyhow::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::info;

use crate::monitor::SystemMonitor;
use crate::stats::{StatsCollector, StressTestStats};

pub async fn run_test(
    batch_size: usize,
    multi_progress: &MultiProgress,
) -> Result<StressTestStats> {
    info!("开始批量操作压测: 批量大小={}", batch_size);

    let total_batches = 100;
    let pb = multi_progress.add(ProgressBar::new(total_batches as u64));
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.white/blue} {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("=>-"),
    );

    let stats_collector = Arc::new(StatsCollector::new());
    let monitor = Arc::new(SystemMonitor::new());
    
    let stats_clone = stats_collector.clone();
    monitor.start_monitoring(1000, move |sys_stats| {
        let stats_clone = stats_clone.clone();
        tokio::spawn(async move {
            stats_clone
                .record_system_stats(sys_stats.cpu_usage, sys_stats.process_memory_mb)
                .await;
        });
    }).await;

    for i in 0..total_batches {
        let op_start = Instant::now();
        let success = simulate_batch_operation(batch_size, i).await;
        let duration = op_start.elapsed();
        
        stats_collector.record_operation(duration, success).await;
        pb.inc(1);
    }

    pb.finish_with_message("批量操作压测完成");
    monitor.stop_monitoring().await;

    let stats = stats_collector.get_stats().await;
    info!("批量操作压测完成: P95延迟={:.2}ms", stats.latency_p95);

    Ok(stats)
}

async fn simulate_batch_operation(batch_size: usize, _batch_index: usize) -> bool {
    // 批量操作延迟与批量大小相关，但有优化效果
    let delay_ms = (batch_size as f64 * 0.5) as u64; // 每个操作 0.5ms
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    true
}

