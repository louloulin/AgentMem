//! 场景 1: 记忆构建压测 - 真实实现

use anyhow::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tracing::{info, warn};
use uuid::Uuid;

use crate::monitor::SystemMonitor;
use crate::real_config::RealStressTestEnv;
use crate::stats::{StatsCollector, StressTestStats};

use agent_mem::AddMemoryOptions;

/// 真实记忆创建压测
///
/// 使用 AgentMem SDK 真实创建记忆，替代 Mock 实现
pub async fn run_test_real(
    env: &RealStressTestEnv,
    concurrency: usize,
    total_memories: usize,
    multi_progress: &MultiProgress,
) -> Result<StressTestStats> {
    info!(
        "🚀 开始真实记忆构建压测: 并发={}, 总数={}",
        concurrency, total_memories
    );
    info!("📊 使用真实 AgentMem SDK + PostgreSQL");

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

    // 并发控制
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let mut handles = Vec::new();

    let start_time = Instant::now();

    for i in 0..total_memories {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let pb_clone = pb.clone();
        let stats_clone = stats_collector.clone();
        let memory_clone = env.memory.clone();

        let handle = tokio::spawn(async move {
            let _permit = permit;
            let op_start = Instant::now();

            // ✅ 真实记忆创建 - 使用 AgentMem SDK
            let success = real_memory_creation(&memory_clone, i).await;

            let duration = op_start.elapsed();
            stats_clone.record_operation(duration, success).await;

            pb_clone.inc(1);
            if i % 100 == 0 {
                pb_clone.set_message(format!("已完成 {i}/{total_memories}"));
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

/// 真实记忆创建
///
/// 使用 AgentMem SDK 真实创建记忆到 PostgreSQL
async fn real_memory_creation(memory: &agent_mem::Memory, index: usize) -> bool {
    let content = format!(
        "Test memory content {} - Created at {} - UUID: {}",
        index,
        chrono::Utc::now().to_rfc3339(),
        Uuid::new_v4()
    );

    let options = AddMemoryOptions::default();

    match memory.add_with_options(content, options).await {
        Ok(result) => {
            // 成功创建记忆
            !result.results.is_empty()
        }
        Err(e) => {
            // 记录错误但不中断测试
            if index % 100 == 0 {
                warn!("记忆创建失败 (index={}): {}", index, e);
            }
            false
        }
    }
}

/// Mock 版本的 run_test（用于对比测试）
///
/// 这个函数用于在没有真实环境的情况下运行测试
pub async fn run_test(
    concurrency: usize,
    total_memories: usize,
    multi_progress: &MultiProgress,
) -> Result<StressTestStats> {
    info!(
        "🚀 开始 Mock 记忆构建压测: 并发={}, 总数={}",
        concurrency, total_memories
    );

    let pb = multi_progress.add(ProgressBar::new(total_memories as u64));
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({per_sec}) {msg}")
            .unwrap()
            .progress_chars("=>-"),
    );

    let stats_collector = Arc::new(StatsCollector::new());
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let mut handles = Vec::new();

    for i in 0..total_memories {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let pb_clone = pb.clone();
        let stats_clone = stats_collector.clone();

        let handle = tokio::spawn(async move {
            let _permit = permit;
            let op_start = Instant::now();

            // Mock 实现
            let success = simulate_memory_creation_mock(i).await;

            let duration = op_start.elapsed();
            stats_clone.record_operation(duration, success).await;

            pb_clone.inc(1);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await?;
    }

    pb.finish_with_message("Mock 记忆构建完成");
    let stats = stats_collector.get_stats().await;

    Ok(stats)
}

/// 保留旧的 Mock 实现用于对比
#[allow(dead_code)]
async fn simulate_memory_creation_mock(index: usize) -> bool {
    // ❌ Mock 实现 - 仅用于性能对比
    let delay_ms = 5 + (index % 20) as u64;
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    index % 100 != 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::real_config::{RealStressTestConfig, RealStressTestEnv};

    #[tokio::test]
    #[ignore] // 需要真实环境，默认跳过
    async fn test_memory_creation() {
        // 初始化真实环境
        let config = RealStressTestConfig::default();
        let env = RealStressTestEnv::new(config).await.unwrap();
        let multi_progress = MultiProgress::new();

        // 运行真实测试
        let stats = run_test_real(&env, 10, 100, &multi_progress).await.unwrap();

        assert!(stats.total_operations == 100);
        assert!(stats.successful_operations >= 95); // 至少 95% 成功
        assert!(stats.throughput > 0.0);

        // 清理
        env.cleanup().await.unwrap();
    }
}
