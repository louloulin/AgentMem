//! 场景 3: 并发操作压测

use anyhow::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tracing::info;

use crate::monitor::SystemMonitor;
use crate::stats::{StatsCollector, StressTestStats};

pub async fn run_test(
    concurrent_users: usize,
    duration_seconds: u64,
    multi_progress: &MultiProgress,
) -> Result<StressTestStats> {
    info!(
        "开始并发操作压测: 用户数={}, 持续={}s",
        concurrent_users, duration_seconds
    );

    let pb = multi_progress.add(ProgressBar::new(duration_seconds));
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.yellow/blue} {pos}/{len}s {msg}")
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

    let semaphore = Arc::new(Semaphore::new(concurrent_users));
    let start_time = Instant::now();
    let mut handles = Vec::new();

    // 启动并发用户
    for user_id in 0..concurrent_users {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let stats_clone = stats_collector.clone();
        let duration = Duration::from_secs(duration_seconds);

        let handle = tokio::spawn(async move {
            let _permit = permit;

            while start_time.elapsed() < duration {
                let op_start = Instant::now();

                // 随机选择操作类型
                let op_type = rand::random::<u8>() % 100;
                let success = if op_type < 70 {
                    // 70% 读操作
                    simulate_read_operation(user_id).await
                } else if op_type < 90 {
                    // 20% 写操作
                    simulate_write_operation(user_id).await
                } else {
                    // 10% 更新/删除操作
                    simulate_update_operation(user_id).await
                };

                let op_duration = op_start.elapsed();
                stats_clone.record_operation(op_duration, success).await;

                // 模拟用户思考时间
                let think_time = 10 + (rand::random::<u64>() % 90);
                tokio::time::sleep(Duration::from_millis(think_time)).await;
            }
        });

        handles.push(handle);
    }

    // 进度条更新
    let pb_clone = pb.clone();
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(1));
        for _ in 0..duration_seconds {
            ticker.tick().await;
            pb_clone.inc(1);
        }
    });

    // 等待所有用户完成
    for handle in handles {
        handle.await?;
    }

    pb.finish_with_message("并发压测完成");
    monitor.stop_monitoring().await;

    let stats = stats_collector.get_stats().await;

    info!(
        "并发压测完成: 总操作={}, 吞吐量={:.2} ops/s, 错误率={:.2}%",
        stats.total_operations,
        stats.throughput,
        stats.error_rate * 100.0
    );

    Ok(stats)
}

async fn simulate_read_operation(_user_id: usize) -> bool {
    tokio::time::sleep(Duration::from_millis(5)).await;
    true
}

async fn simulate_write_operation(_user_id: usize) -> bool {
    tokio::time::sleep(Duration::from_millis(15)).await;
    true
}

async fn simulate_update_operation(_user_id: usize) -> bool {
    tokio::time::sleep(Duration::from_millis(20)).await;
    true
}
