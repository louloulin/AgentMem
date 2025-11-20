//! 场景 8: 长时间稳定性测试

use anyhow::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{info, warn};

use crate::monitor::SystemMonitor;
use crate::stats::{StatsCollector, StressTestStats};

pub async fn run_test(hours: u64, multi_progress: &MultiProgress) -> Result<StressTestStats> {
    info!("开始长时间稳定性测试: {}小时", hours);
    warn!("此测试将运行 {} 小时，请确保系统资源充足", hours);

    let duration_seconds = hours * 3600;
    let pb = multi_progress.add(ProgressBar::new(duration_seconds));
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len}s {msg}")
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

    let start_time = Instant::now();
    let test_duration = Duration::from_secs(duration_seconds);

    // 进度条更新
    let pb_clone = pb.clone();
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(1));
        while start_time.elapsed() < test_duration {
            ticker.tick().await;
            pb_clone.inc(1);
        }
    });

    // 持续执行操作
    while start_time.elapsed() < test_duration {
        let op_start = Instant::now();
        let success = simulate_mixed_operation().await;
        let duration = op_start.elapsed();

        stats_collector.record_operation(duration, success).await;

        // 模拟负载变化
        let elapsed_hours = start_time.elapsed().as_secs() / 3600;
        if elapsed_hours % 2 == 0 {
            // 偶数小时：高负载
            tokio::time::sleep(Duration::from_millis(10)).await;
        } else {
            // 奇数小时：低负载
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    pb.finish_with_message("稳定性测试完成");
    monitor.stop_monitoring().await;

    let stats = stats_collector.get_stats().await;
    info!(
        "稳定性测试完成: 总操作={}, 错误率={:.4}%",
        stats.total_operations,
        stats.error_rate * 100.0
    );

    Ok(stats)
}

async fn simulate_mixed_operation() -> bool {
    // 混合操作
    let delay_ms = 5 + rand::random::<u64>() % 20;
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    true
}
