//! 场景 4: 图推理压测

use anyhow::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::info;

use crate::monitor::SystemMonitor;
use crate::stats::{StatsCollector, StressTestStats};

pub async fn run_test(
    nodes: usize,
    edges: usize,
    multi_progress: &MultiProgress,
) -> Result<StressTestStats> {
    info!("开始图推理压测: 节点={}, 边={}", nodes, edges);

    let total_queries = 500;
    let pb = multi_progress.add(ProgressBar::new(total_queries as u64));
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.magenta/blue} {pos}/{len} {msg}")
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

    // 模拟图查询
    for i in 0..total_queries {
        let op_start = Instant::now();
        let success = simulate_graph_query(nodes, edges, i).await;
        let duration = op_start.elapsed();

        stats_collector.record_operation(duration, success).await;
        pb.inc(1);
    }

    pb.finish_with_message("图推理压测完成");
    monitor.stop_monitoring().await;

    let stats = stats_collector.get_stats().await;
    info!("图推理压测完成: P95延迟={:.2}ms", stats.latency_p95);

    Ok(stats)
}

async fn simulate_graph_query(nodes: usize, edges: usize, _query_index: usize) -> bool {
    // 图查询延迟与图规模相关
    let complexity = ((nodes as f64).log10() + (edges as f64).log10()) as u64;
    let delay_ms = 20 + complexity * 2;
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    true
}
