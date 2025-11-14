//! 场景 2: 记忆检索压测

use anyhow::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tracing::info;

use crate::monitor::SystemMonitor;
use crate::stats::{StatsCollector, StressTestStats};

pub async fn run_test(
    dataset_size: usize,
    concurrency: usize,
    multi_progress: &MultiProgress,
) -> Result<StressTestStats> {
    info!("开始记忆检索压测: 数据集={}, 并发={}", dataset_size, concurrency);

    let total_queries = 1000; // 执行 1000 次查询

    let pb = multi_progress.add(ProgressBar::new(total_queries as u64));
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.green/blue} {pos}/{len} ({per_sec}) {msg}")
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

    for i in 0..total_queries {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let pb_clone = pb.clone();
        let stats_clone = stats_collector.clone();

        let handle = tokio::spawn(async move {
            let _permit = permit;
            let op_start = Instant::now();

            // 模拟向量搜索
            let success = simulate_vector_search(dataset_size, i).await;

            let duration = op_start.elapsed();
            stats_clone.record_operation(duration, success).await;

            pb_clone.inc(1);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await?;
    }

    pb.finish_with_message("检索压测完成");
    monitor.stop_monitoring().await;

    let stats = stats_collector.get_stats().await;

    info!(
        "检索压测完成: 吞吐量={:.2} qps, P95延迟={:.2}ms",
        stats.throughput, stats.latency_p95
    );

    Ok(stats)
}

async fn simulate_vector_search(dataset_size: usize, query_index: usize) -> bool {
    // 模拟向量搜索延迟（数据集越大，延迟越高）
    let base_delay = 10; // 10ms 基础延迟
    let scale_factor = (dataset_size as f64).log10() as u64; // 对数缩放
    let delay_ms = base_delay + scale_factor + (query_index % 10) as u64;
    
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;

    // 99.5% 成功率
    query_index % 200 != 0
}

