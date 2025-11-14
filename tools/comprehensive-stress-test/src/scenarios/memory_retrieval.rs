//! 场景 2: 记忆检索压测 - 真实实现

use anyhow::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tracing::{info, warn};

use crate::monitor::SystemMonitor;
use crate::real_config::RealStressTestEnv;
use crate::stats::{StatsCollector, StressTestStats};

use agent_mem::SearchOptions;

/// 真实记忆检索压测
///
/// 使用 AgentMem SDK 真实检索记忆，替代 Mock 实现
pub async fn run_test_real(
    env: &RealStressTestEnv,
    dataset_size: usize,
    concurrency: usize,
    multi_progress: &MultiProgress,
) -> Result<StressTestStats> {
    info!("🚀 开始真实记忆检索压测: 数据集={}, 并发={}", dataset_size, concurrency);
    info!("📊 使用真实 AgentMem SDK + 向量搜索");

    // 准备测试数据集
    prepare_dataset(env, dataset_size).await?;

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
        let memory_clone = env.memory.clone();

        let handle = tokio::spawn(async move {
            let _permit = permit;
            let op_start = Instant::now();

            // ✅ 真实向量搜索 - 使用 AgentMem SDK
            let success = real_vector_search(&memory_clone, i).await;

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

/// 准备测试数据集
async fn prepare_dataset(env: &RealStressTestEnv, size: usize) -> Result<()> {
    info!("📝 准备测试数据集: {} 条记忆...", size);

    // 检查是否已有足够数据
    let stats = env.get_db_stats().await?;
    if stats.memory_count >= size {
        info!("✅ 数据集已存在: {} 条记忆", stats.memory_count);
        return Ok(());
    }

    // 批量创建测试数据
    let batch_size = 100;
    let needed = size.saturating_sub(stats.memory_count);

    for batch_start in (0..needed).step_by(batch_size) {
        let batch_end = (batch_start + batch_size).min(needed);
        let mut contents = Vec::new();

        for i in batch_start..batch_end {
            contents.push(format!(
                "Dataset item {} - Topic: {} - Content: Sample memory for retrieval testing",
                i,
                i % 10  // 10 个不同主题
            ));
        }

        // 使用批量添加 API
        if let Err(e) = env.memory.add_batch(contents, agent_mem::AddMemoryOptions::default()).await {
            warn!("批量添加失败: {}", e);
        }
    }

    info!("✅ 数据集准备完成");
    Ok(())
}

/// 真实向量搜索
///
/// 使用 AgentMem SDK 真实搜索记忆
async fn real_vector_search(memory: &agent_mem::Memory, query_index: usize) -> bool {
    let query = format!("Topic: {} Sample memory", query_index % 10);

    let options = SearchOptions {
        limit: Some(10),
        ..Default::default()
    };

    match memory.search_with_options(&query, options).await {
        Ok(results) => {
            // 成功检索
            !results.is_empty()
        }
        Err(e) => {
            if query_index % 100 == 0 {
                warn!("记忆检索失败 (query={}): {}", query_index, e);
            }
            false
        }
    }
}

/// 保留旧的 Mock 实现用于对比
#[allow(dead_code)]
async fn simulate_vector_search_mock(dataset_size: usize, query_index: usize) -> bool {
    // ❌ Mock 实现 - 仅用于性能对比
    let base_delay = 10;
    let scale_factor = (dataset_size as f64).log10() as u64;
    let delay_ms = base_delay + scale_factor + (query_index % 10) as u64;
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    query_index % 200 != 0
}

