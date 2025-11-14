//! LibSQL 真实压测工具
//!
//! 使用 LibSQL 嵌入式数据库进行真实压测验证，无需外部数据库服务
//!
//! 运行方式:
//! ```bash
//! cargo run --release -p libsql-stress-test
//! ```

use agent_mem::{Memory, AddMemoryOptions};
use std::time::Instant;
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("🚀 AgentMem LibSQL 真实压测开始");
    info!("{}", "=".repeat(60));

    // 1. 初始化 Memory SDK (使用 LibSQL)
    info!("\n📦 初始化 AgentMem SDK...");
    let db_path = "./data/stress-test.db";
    std::fs::create_dir_all("./data")?;
    
    let memory = Memory::builder()
        .with_storage(&format!("libsql://{}", db_path))
        .build()
        .await?;
    
    info!("✅ SDK 初始化完成");

    // 2. 记忆创建压测
    info!("\n📝 测试 1: 记忆创建性能");
    info!("{}", "-".repeat(60));
    let create_count = 100;
    let start = Instant::now();
    let mut success = 0;
    let mut failed = 0;

    for i in 0..create_count {
        let content = format!("Test memory {} - Created at {}", i, chrono::Utc::now());
        
        match memory.add_with_options(content, AddMemoryOptions::default()).await {
            Ok(result) => {
                if !result.results.is_empty() {
                    success += 1;
                    if i % 20 == 0 {
                        info!("  已创建 {} 条记忆...", i + 1);
                    }
                } else {
                    failed += 1;
                }
            }
            Err(e) => {
                warn!("记忆创建失败: {}", e);
                failed += 1;
            }
        }
    }

    let duration = start.elapsed();
    let duration_secs = duration.as_secs_f64();
    let throughput = create_count as f64 / duration_secs;

    info!("✅ 记忆创建完成:");
    info!("   总数: {}", create_count);
    info!("   成功: {}", success);
    info!("   失败: {}", failed);
    info!("   耗时: {:.2}s", duration_secs);
    info!("   吞吐量: {:.2} ops/s", throughput);
    info!("   平均延迟: {:.2}ms", duration_secs * 1000.0 / create_count as f64);

    // 3. 记忆检索压测
    info!("\n🔍 测试 2: 记忆检索性能");
    info!("{}", "-".repeat(60));
    let search_count = 50;
    let start = Instant::now();
    let mut success = 0;
    let mut failed = 0;
    let mut total_results = 0;

    for i in 0..search_count {
        let query = format!("Test memory {}", i % 10);
        
        match memory.search(&query).await {
            Ok(results) => {
                total_results += results.len();
                success += 1;
                if i % 10 == 0 {
                    info!("  已检索 {} 次...", i + 1);
                }
            }
            Err(e) => {
                warn!("记忆检索失败: {}", e);
                failed += 1;
            }
        }
    }

    let duration = start.elapsed();
    let duration_secs = duration.as_secs_f64();
    let throughput = search_count as f64 / duration_secs;

    info!("✅ 记忆检索完成:");
    info!("   总数: {}", search_count);
    info!("   成功: {}", success);
    info!("   失败: {}", failed);
    info!("   检索到记忆数: {}", total_results);
    info!("   耗时: {:.2}s", duration_secs);
    info!("   吞吐量: {:.2} qps", throughput);
    info!("   平均延迟: {:.2}ms", duration_secs * 1000.0 / search_count as f64);

    // 4. 批量操作压测
    info!("\n📦 测试 3: 批量操作性能");
    info!("{}", "-".repeat(60));
    let batches = 10;
    let batch_size = 20;
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

        match memory.add_batch(contents, AddMemoryOptions::default()).await {
            Ok(results) => {
                if results.len() == batch_size {
                    success += 1;
                    info!("  批次 {} 完成 ({} 条记忆)", batch_idx + 1, batch_size);
                } else {
                    failed += 1;
                }
            }
            Err(e) => {
                warn!("批量操作失败: {}", e);
                failed += 1;
            }
        }
    }

    let duration = start.elapsed();
    let duration_secs = duration.as_secs_f64();
    let throughput = batches as f64 / duration_secs;
    let total_items = batches * batch_size;
    let item_throughput = total_items as f64 / duration_secs;

    info!("✅ 批量操作完成:");
    info!("   总批次: {}", batches);
    info!("   成功: {}", success);
    info!("   失败: {}", failed);
    info!("   总记忆数: {}", total_items);
    info!("   耗时: {:.2}s", duration_secs);
    info!("   批次吞吐量: {:.2} batches/s", throughput);
    info!("   记忆吞吐量: {:.2} items/s", item_throughput);

    // 5. 性能总结
    info!("\n📊 性能总结:");
    info!("{}", "=".repeat(60));
    info!("✅ 所有测试完成！");
    info!("   数据库: LibSQL ({})", db_path);
    info!("   总记忆数: ~{}", create_count + total_items);
    
    // 6. 与 Mem0 对比
    info!("\n📈 与 Mem0 性能对比:");
    info!("{}", "-".repeat(60));
    info!("   AgentMem 记忆创建: {:.2} ops/s", throughput);
    info!("   Mem0 基准 (LOCOMO): ~10,000 ops/s");
    if throughput > 0.0 {
        info!("   差距: {:.1}x", 10000.0 / throughput);
    }
    info!("");
    info!("💡 优化建议:");
    info!("   1. 启用批量操作优化");
    info!("   2. 实现连接池");
    info!("   3. 添加缓存层");
    info!("   4. 优化数据库索引");

    Ok(())
}

