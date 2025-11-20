use agent_mem::orchestrator::{MemoryOrchestrator, OrchestratorConfig};
use agent_mem_core::types::MemoryType;
use std::collections::HashMap;
use std::time::Instant;
use tracing::{info, warn};

/// Phase 4 批量模式性能测试
///
/// 测试目标：
/// - 批量添加吞吐量：5,000+ ops/s
/// - 平均延迟：< 1ms/条
/// - 对比不同批次大小的性能
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    info!("========================================");
    info!("Phase 4: 批量模式性能测试");
    info!("========================================\n");

    // 配置 Orchestrator（启用 embedder）
    let config = OrchestratorConfig {
        storage_url: Some("libsql://./data/batch_test.db".to_string()),
        llm_provider: None,
        llm_model: None,
        embedder_provider: Some("fastembed".to_string()),
        embedder_model: Some("all-MiniLM-L6-v2".to_string()),
        vector_store_url: Some("memory".to_string()), // 使用内存向量存储
        enable_intelligent_features: false,           // 禁用智能功能以提高性能
    };

    info!("初始化 MemoryOrchestrator...");
    let orchestrator = MemoryOrchestrator::new_with_config(config).await?;
    info!("✅ MemoryOrchestrator 初始化完成\n");

    // 测试不同批次大小
    let batch_sizes = vec![10, 50, 100, 500, 1000];

    for batch_size in batch_sizes {
        test_batch_performance(&orchestrator, batch_size).await?;
        println!();
    }

    // 对比测试：单个 vs 批量
    comparison_test(&orchestrator).await?;

    info!("\n========================================");
    info!("所有测试完成！");
    info!("========================================");

    Ok(())
}

/// 测试指定批次大小的性能
async fn test_batch_performance(
    orchestrator: &MemoryOrchestrator,
    batch_size: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("----------------------------------------");
    info!("测试批次大小: {}", batch_size);
    info!("----------------------------------------");

    // 准备测试数据
    let items: Vec<_> = (0..batch_size)
        .map(|i| {
            (
                format!(
                    "批量测试记忆 #{} - 这是一条测试数据，用于验证批量添加性能",
                    i
                ),
                "test-agent".to_string(),
                Some("test-user".to_string()),
                Some(MemoryType::Episodic),
                Some(HashMap::new()),
            )
        })
        .collect();

    // 执行批量添加
    let start = Instant::now();
    let memory_ids = orchestrator.add_memories_batch(items).await?;
    let duration = start.elapsed();

    // 计算性能指标
    let total_ms = duration.as_secs_f64() * 1000.0;
    let avg_latency_ms = total_ms / batch_size as f64;
    let throughput = batch_size as f64 / duration.as_secs_f64();

    info!("✅ 批量添加完成");
    info!("  - 记忆数量: {}", memory_ids.len());
    info!("  - 总时间: {:.2}ms", total_ms);
    info!("  - 平均延迟: {:.3}ms/条", avg_latency_ms);
    info!("  - 吞吐量: {:.2} ops/s", throughput);

    // 性能评估
    if throughput >= 5000.0 {
        info!("  ✅ 达到目标 (5,000+ ops/s)");
    } else if throughput >= 3000.0 {
        warn!("  ⚠️  接近目标 ({:.0}% of 5,000 ops/s)", throughput / 50.0);
    } else {
        warn!("  ❌ 未达到目标 ({:.0}% of 5,000 ops/s)", throughput / 50.0);
    }

    if avg_latency_ms <= 1.0 {
        info!("  ✅ 延迟达标 (< 1ms/条)");
    } else {
        warn!("  ⚠️  延迟偏高 ({:.3}ms/条)", avg_latency_ms);
    }

    Ok(())
}

/// 对比测试：单个添加 vs 批量添加
async fn comparison_test(
    orchestrator: &MemoryOrchestrator,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("========================================");
    info!("对比测试: 单个 vs 批量");
    info!("========================================\n");

    let test_count = 100;

    // 测试 1: 单个添加
    info!("测试 1: 单个添加 ({} 次)", test_count);
    let start = Instant::now();
    for i in 0..test_count {
        orchestrator
            .add_memory(
                format!("单个测试记忆 #{}", i),
                "test-agent".to_string(),
                Some("test-user".to_string()),
                Some(MemoryType::Episodic),
                Some(HashMap::new()),
            )
            .await?;
    }
    let single_duration = start.elapsed();
    let single_throughput = test_count as f64 / single_duration.as_secs_f64();

    info!(
        "  - 总时间: {:.2}ms",
        single_duration.as_secs_f64() * 1000.0
    );
    info!("  - 吞吐量: {:.2} ops/s", single_throughput);

    // 测试 2: 批量添加
    info!("\n测试 2: 批量添加 ({} 条)", test_count);
    let items: Vec<_> = (0..test_count)
        .map(|i| {
            (
                format!("批量测试记忆 #{}", i),
                "test-agent".to_string(),
                Some("test-user".to_string()),
                Some(MemoryType::Episodic),
                Some(HashMap::new()),
            )
        })
        .collect();

    let start = Instant::now();
    orchestrator.add_memories_batch(items).await?;
    let batch_duration = start.elapsed();
    let batch_throughput = test_count as f64 / batch_duration.as_secs_f64();

    info!("  - 总时间: {:.2}ms", batch_duration.as_secs_f64() * 1000.0);
    info!("  - 吞吐量: {:.2} ops/s", batch_throughput);

    // 性能对比
    let speedup = batch_throughput / single_throughput;
    info!("\n📊 性能对比:");
    info!("  - 批量模式加速比: {:.2}x", speedup);
    info!(
        "  - 时间节省: {:.1}%",
        (1.0 - batch_duration.as_secs_f64() / single_duration.as_secs_f64()) * 100.0
    );

    if speedup >= 5.0 {
        info!("  ✅ 批量优化效果显著 (5x+)");
    } else if speedup >= 3.0 {
        info!("  ✅ 批量优化效果良好 (3-5x)");
    } else if speedup >= 2.0 {
        warn!("  ⚠️  批量优化效果一般 (2-3x)");
    } else {
        warn!("  ❌ 批量优化效果不明显 (<2x)");
    }

    Ok(())
}
