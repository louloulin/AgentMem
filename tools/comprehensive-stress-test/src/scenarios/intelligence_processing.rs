//! 场景 5: 智能处理压测

use anyhow::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tracing::info;

use crate::monitor::SystemMonitor;
use crate::stats::{StatsCollector, StressTestStats};

pub async fn run_test(
    concurrency: usize,
    multi_progress: &MultiProgress,
) -> Result<StressTestStats> {
    info!("开始智能处理压测: 并发={}", concurrency);

    let total_requests = 100; // LLM 调用较慢，减少总数
    let pb = multi_progress.add(ProgressBar::new(total_requests as u64));
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.red/blue} {pos}/{len} {msg}")
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

    let semaphore = Arc::new(Semaphore::new(concurrency));
    let mut handles = Vec::new();

    for i in 0..total_requests {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let pb_clone = pb.clone();
        let stats_clone = stats_collector.clone();

        let handle = tokio::spawn(async move {
            let _permit = permit;
            let op_start = Instant::now();

            // 模拟 LLM 调用
            let success = simulate_llm_call(i).await;

            let duration = op_start.elapsed();
            stats_clone.record_operation(duration, success).await;
            pb_clone.inc(1);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await?;
    }

    pb.finish_with_message("智能处理压测完成");
    monitor.stop_monitoring().await;

    let stats = stats_collector.get_stats().await;
    info!("智能处理压测完成: P95延迟={:.2}ms", stats.latency_p95);

    Ok(stats)
}

async fn simulate_llm_call(_index: usize) -> bool {
    // LLM 调用通常需要 500-2000ms
    let delay_ms = 500 + rand::random::<u64>() % 1500;
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    true
}

