use agent_mem::orchestrator::{MemoryOrchestrator, OrchestratorConfig};
use std::time::Instant;
use tracing::Level;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志（只显示 INFO 级别）
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    println!("\n🚀 AgentMem Phase 3 性能测试 - 并行存储优化");
    println!("================================\n");

    // 创建 Orchestrator（快速模式，不使用LLM）
    println!("📦 初始化 Orchestrator（快速模式）...");
    let config = OrchestratorConfig {
        storage_url: Some("libsql://./data/phase3_test.db".to_string()),
        llm_provider: None, // 不使用LLM
        llm_model: None,
        embedder_provider: Some("fastembed".to_string()),
        embedder_model: Some("all-MiniLM-L6-v2".to_string()),
        vector_store_url: Some("memory://".to_string()), // 使用内存向量存储
        enable_intelligent_features: false,              // 禁用智能功能
        ..Default::default()
    };

    let orchestrator = MemoryOrchestrator::new_with_config(config).await?;
    println!("✅ Orchestrator 初始化完成\n");

    // ========== 测试 1: 单个记忆添加性能（验证并行存储） ==========
    println!("📊 测试 1: 单个记忆添加性能（Phase 3 并行存储）");
    println!("─────────────────────────────────────");

    let test_count = 100;
    let start = Instant::now();

    for i in 0..test_count {
        let content = format!("Phase 3 parallel storage test {}", i);
        orchestrator
            .add_memory_v2(
                content,
                "test_agent".to_string(),
                Some("test_user".to_string()),
                None,
                None,
                false, // 快速模式，不使用推理
                None,
                None,
            )
            .await?;
    }

    let duration = start.elapsed();
    let throughput = test_count as f64 / duration.as_secs_f64();
    let avg_latency = duration.as_secs_f64() / test_count as f64 * 1000.0;

    println!("\n✅ 测试完成");
    println!("   记忆数量: {}", test_count);
    println!("   总时间: {:?}", duration);
    println!("   平均延迟: {:.2}ms", avg_latency);
    println!("   吞吐量: {:.2} ops/s", throughput);
    println!("   目标: 1,000+ ops/s");

    if throughput >= 1000.0 {
        println!("   ✅ 达到目标！");
    } else {
        println!("   ⚠️  未达到目标（差距: {:.2}x）", 1000.0 / throughput);
    }

    // ========== 测试 2: 批量添加性能 ==========
    println!("\n📊 测试 2: 批量添加性能（10个并发）");
    println!("─────────────────────────────────────");

    let batch_size = 10;
    let batch_count = 10;
    let start = Instant::now();

    for batch in 0..batch_count {
        let mut tasks = Vec::new();

        for i in 0..batch_size {
            let content = format!("Batch {} item {}", batch, i);
            let orch = &orchestrator;

            let task = async move {
                orch.add_memory_v2(
                    content,
                    "test_agent".to_string(),
                    Some("test_user".to_string()),
                    None,
                    None,
                    false, // 快速模式，不使用推理
                    None,
                    None,
                )
                .await
            };

            tasks.push(task);
        }

        // 并发执行
        futures::future::join_all(tasks).await;
    }

    let duration = start.elapsed();
    let total_ops = batch_size * batch_count;
    let throughput = total_ops as f64 / duration.as_secs_f64();
    let avg_latency = duration.as_secs_f64() / total_ops as f64 * 1000.0;

    println!("\n✅ 测试完成");
    println!("   批次数量: {}", batch_count);
    println!("   每批大小: {}", batch_size);
    println!("   总操作数: {}", total_ops);
    println!("   总时间: {:?}", duration);
    println!("   平均延迟: {:.2}ms", avg_latency);
    println!("   吞吐量: {:.2} ops/s", throughput);
    println!("   目标: 1,500+ ops/s");

    if throughput >= 1500.0 {
        println!("   ✅ 达到目标！");
    } else {
        println!("   ⚠️  未达到目标（差距: {:.2}x）", 1500.0 / throughput);
    }

    // ========== 测试 3: 性能对比分析 ==========
    println!("\n📊 测试 3: Phase 3 优化效果分析");
    println!("─────────────────────────────────────");
    println!("Phase 3 优化: 并行存储（CoreManager + VectorStore + History）");
    println!("预期提升: 顺序执行70ms → 并行执行20ms (3.5x)");
    println!("\n实际测试结果:");
    println!("  - 单个添加平均延迟: {:.2}ms", avg_latency);
    println!("  - 批量添加吞吐量: {:.2} ops/s", throughput);

    if avg_latency < 5.0 {
        println!("  ✅ 延迟优秀（< 5ms）");
    } else if avg_latency < 20.0 {
        println!("  ✅ 延迟良好（< 20ms）");
    } else {
        println!("  ⚠️  延迟偏高（> 20ms）");
    }

    println!("\n🎉 Phase 3 测试完成！\n");

    Ok(())
}
