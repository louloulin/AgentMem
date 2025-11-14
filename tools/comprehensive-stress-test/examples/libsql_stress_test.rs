//! LibSQL 真实压测示例
//!
//! 使用 LibSQL 嵌入式数据库进行真实压测验证
//!
//! 运行方式:
//! ```bash
//! cargo run --release --example libsql_stress_test
//! ```

use comprehensive_stress_test::{LibSQLStressTestConfig, LibSQLStressTestEnv};
use agent_mem_traits::AddMemoryOptions;
use std::time::Instant;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("🚀 AgentMem LibSQL 真实压测开始");
    info!("=" .repeat(60));

    // 1. 初始化环境
    let config = LibSQLStressTestConfig::default();
    let env = LibSQLStressTestEnv::new(config).await?;

    // 2. 记忆创建压测
    info!("\n📝 测试 1: 记忆创建性能");
    info!("-".repeat(60));
    let create_result = test_memory_creation(&env, 100).await?;
    info!("✅ 记忆创建完成:");
    info!("   总数: {}", create_result.total);
    info!("   成功: {}", create_result.success);
    info!("   失败: {}", create_result.failed);
    info!("   耗时: {:.2}s", create_result.duration_secs);
    info!("   吞吐量: {:.2} ops/s", create_result.throughput);

    // 3. 记忆检索压测
    info!("\n🔍 测试 2: 记忆检索性能");
    info!("-".repeat(60));
    let search_result = test_memory_search(&env, 50).await?;
    info!("✅ 记忆检索完成:");
    info!("   总数: {}", search_result.total);
    info!("   成功: {}", search_result.success);
    info!("   失败: {}", search_result.failed);
    info!("   耗时: {:.2}s", search_result.duration_secs);
    info!("   吞吐量: {:.2} ops/s", search_result.throughput);

    // 4. 批量操作压测
    info!("\n📦 测试 3: 批量操作性能");
    info!("-".repeat(60));
    let batch_result = test_batch_operations(&env, 10, 20).await?;
    info!("✅ 批量操作完成:");
    info!("   总批次: {}", batch_result.total);
    info!("   成功: {}", batch_result.success);
    info!("   失败: {}", batch_result.failed);
    info!("   耗时: {:.2}s", batch_result.duration_secs);
    info!("   吞吐量: {:.2} batches/s", batch_result.throughput);

    // 5. 获取统计信息
    info!("\n📊 数据库统计:");
    info!("-".repeat(60));
    let stats = env.get_stats().await?;
    info!("   记忆总数: {}", stats.total_memories);
    info!("   向量总数: {}", stats.total_vectors);
    info!("   数据库大小: {} bytes", stats.db_size_bytes);

    // 6. 清理
    info!("\n🧹 清理测试数据...");
    env.cleanup().await?;

    info!("\n✅ LibSQL 真实压测完成!");
    info!("=" .repeat(60));

    Ok(())
}

/// 测试记忆创建性能
async fn test_memory_creation(
    env: &LibSQLStressTestEnv,
    count: usize,
) -> Result<TestResult, Box<dyn std::error::Error>> {
    let start = Instant::now();
    let mut success = 0;
    let mut failed = 0;

    for i in 0..count {
        let content = format!("Test memory {} - Created at {}", i, chrono::Utc::now());
        
        match env.memory.add_with_options(content, AddMemoryOptions::default()).await {
            Ok(result) => {
                if !result.results.is_empty() {
                    success += 1;
                } else {
                    failed += 1;
                }
            }
            Err(e) => {
                tracing::warn!("记忆创建失败: {}", e);
                failed += 1;
            }
        }
    }

    let duration = start.elapsed();
    let duration_secs = duration.as_secs_f64();
    let throughput = count as f64 / duration_secs;

    Ok(TestResult {
        total: count,
        success,
        failed,
        duration_secs,
        throughput,
    })
}

/// 测试记忆检索性能
async fn test_memory_search(
    env: &LibSQLStressTestEnv,
    count: usize,
) -> Result<TestResult, Box<dyn std::error::Error>> {
    let start = Instant::now();
    let mut success = 0;
    let mut failed = 0;

    for i in 0..count {
        let query = format!("Test memory {}", i % 10);
        
        match env.memory.search(&query).await {
            Ok(results) => {
                if !results.is_empty() {
                    success += 1;
                } else {
                    // 没有结果也算成功
                    success += 1;
                }
            }
            Err(e) => {
                tracing::warn!("记忆检索失败: {}", e);
                failed += 1;
            }
        }
    }

    let duration = start.elapsed();
    let duration_secs = duration.as_secs_f64();
    let throughput = count as f64 / duration_secs;

    Ok(TestResult {
        total: count,
        success,
        failed,
        duration_secs,
        throughput,
    })
}

/// 测试批量操作性能
async fn test_batch_operations(
    env: &LibSQLStressTestEnv,
    batches: usize,
    batch_size: usize,
) -> Result<TestResult, Box<dyn std::error::Error>> {
    let start = Instant::now();
    let mut success = 0;
    let mut failed = 0;

    for batch_idx in 0..batches {
        let mut contents = Vec::with_capacity(batch_size);
        for i in 0..batch_size {
            contents.push(format!(
                "Batch {} item {} - {}",
                batch_idx,
                i,
                chrono::Utc::now()
            ));
        }

        match env.memory.add_batch(contents, AddMemoryOptions::default()).await {
            Ok(results) => {
                if results.len() == batch_size {
                    success += 1;
                } else {
                    failed += 1;
                }
            }
            Err(e) => {
                tracing::warn!("批量操作失败: {}", e);
                failed += 1;
            }
        }
    }

    let duration = start.elapsed();
    let duration_secs = duration.as_secs_f64();
    let throughput = batches as f64 / duration_secs;

    Ok(TestResult {
        total: batches,
        success,
        failed,
        duration_secs,
        throughput,
    })
}

/// 测试结果
#[derive(Debug)]
struct TestResult {
    total: usize,
    success: usize,
    failed: usize,
    duration_secs: f64,
    throughput: f64,
}

