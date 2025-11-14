//! 场景 1: 记忆构建压测

use anyhow::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tracing::{info, warn};

use crate::monitor::SystemMonitor;
use crate::stats::{StatsCollector, StressTestStats};

pub async fn run_test(
    concurrency: usize,
    total_memories: usize,
    multi_progress: &MultiProgress,
) -> Result<StressTestStats> {
    info!("开始记忆构建压测: 并发={}, 总数={}", concurrency, total_memories);

    // 创建进度条
    let pb = multi_progress.add(ProgressBar::new(total_memories as u64));
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({per_sec}) {msg}")
            .unwrap()
            .progress_chars("=>-"),
    );

    // 创建统计收集器
    let stats_collector = Arc::new(StatsCollector::new());

    // 创建系统监控器
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

    // 并发控制
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let mut handles = Vec::new();

    let start_time = Instant::now();

    for i in 0..total_memories {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let pb_clone = pb.clone();
        let stats_clone = stats_collector.clone();

        let handle = tokio::spawn(async move {
            let _permit = permit;
            let op_start = Instant::now();

            // 模拟记忆创建操作
            let success = simulate_memory_creation(i).await;

            let duration = op_start.elapsed();
            stats_clone.record_operation(duration, success).await;

            pb_clone.inc(1);
            if i % 100 == 0 {
                pb_clone.set_message(format!("已完成 {}/{}", i, total_memories));
            }
        });

        handles.push(handle);
    }

    // 等待所有任务完成
    for handle in handles {
        handle.await?;
    }

    pb.finish_with_message("记忆构建完成");

    // 停止监控
    monitor.stop_monitoring().await;

    // 获取统计结果
    let stats = stats_collector.get_stats().await;

    info!(
        "记忆构建压测完成: 耗时={:.2}s, 吞吐量={:.2} ops/s, P95延迟={:.2}ms",
        stats.duration_seconds, stats.throughput, stats.latency_p95
    );

    Ok(stats)
}

async fn simulate_memory_creation(index: usize) -> bool {
    // 模拟记忆创建的延迟
    let delay_ms = 5 + (index % 20) as u64; // 5-25ms
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;

    // 模拟 99% 成功率
    index % 100 != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_creation() {
        let multi_progress = MultiProgress::new();
        let stats = run_test(10, 100, &multi_progress).await.unwrap();

        assert!(stats.total_operations == 100);
        assert!(stats.successful_operations >= 95); // 至少 95% 成功
        assert!(stats.throughput > 0.0);
    }
}

