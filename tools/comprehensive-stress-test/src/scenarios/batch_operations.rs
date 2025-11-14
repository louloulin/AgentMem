//! 场景 7: 批量操作压测 - 真实实现

use anyhow::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{info, warn};
use uuid::Uuid;

use crate::monitor::SystemMonitor;
use crate::real_config::RealStressTestEnv;
use crate::stats::{StatsCollector, StressTestStats};

use agent_mem::AddMemoryOptions;

/// 真实批量操作压测
///
/// 使用 AgentMem SDK 真实批量添加记忆，替代 Mock 实现
pub async fn run_test_real(
    env: &RealStressTestEnv,
    batch_size: usize,
    multi_progress: &MultiProgress,
) -> Result<StressTestStats> {
    info!("🚀 开始真实批量操作压测: 批量大小={}", batch_size);
    info!("📊 使用真实 AgentMem SDK 批量 API");

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

        // ✅ 真实批量操作 - 使用 AgentMem SDK
        let success = real_batch_operation(&env.memory, batch_size, i).await;

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

/// 真实批量操作
///
/// 使用 AgentMem SDK 批量添加记忆
async fn real_batch_operation(
    memory: &agent_mem::Memory,
    batch_size: usize,
    batch_index: usize,
) -> bool {
    // 生成批量内容
    let mut contents = Vec::with_capacity(batch_size);
    for i in 0..batch_size {
        contents.push(format!(
            "Batch {} item {} - UUID: {} - Timestamp: {}",
            batch_index,
            i,
            Uuid::new_v4(),
            chrono::Utc::now().to_rfc3339()
        ));
    }

    let options = AddMemoryOptions::default();

    match memory.add_batch(contents, options).await {
        Ok(results) => {
            // 成功批量添加
            results.len() == batch_size
        }
        Err(e) => {
            if batch_index % 10 == 0 {
                warn!("批量操作失败 (batch={}): {}", batch_index, e);
            }
            false
        }
    }
}
