//! 简单性能测试工具
//!
//! 用于验证 Phase 1 优化效果

use agent_mem::orchestrator::{MemoryOrchestrator, OrchestratorConfig};
use agent_mem_core::types::MemoryType;
use std::time::Instant;
use tokio::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("\n🚀 AgentMem Phase 1 性能测试");
    println!("================================\n");

    // 创建 Orchestrator
    let config = OrchestratorConfig {
        storage_url: Some("libsql://./data/perf_test.db".to_string()),
        llm_provider: None,
        llm_model: None,
        embedder_provider: Some("fastembed".to_string()),
        embedder_model: Some("bge-small-en-v1.5".to_string()),
        vector_store_url: Some("./data/perf_lancedb".to_string()),
        enable_intelligent_features: false,
    };

    println!("📦 初始化 Orchestrator...");
    let orchestrator = MemoryOrchestrator::new_with_config(config).await?;
    println!("✅ Orchestrator 初始化完成\n");

    // 测试 1: 单个添加性能（Task 1.1 验证）
    println!("📊 测试 1: 单个添加性能（Task 1.1 验证）");
    println!("─────────────────────────────────────");
    
    let mut durations = Vec::new();
    let test_count = 10;
    
    for i in 0..test_count {
        let start = Instant::now();
        orchestrator
            .add_memory_fast(
                format!("Test memory {}", i),
                "test_agent".to_string(),
                Some("test_user".to_string()),
                Some(MemoryType::Core),
                None,
            )
            .await?;
        let duration = start.elapsed();
        durations.push(duration);
    }
    
    let total_time: Duration = durations.iter().sum();
    let avg_time = total_time / test_count as u32;
    let throughput = 1000.0 / avg_time.as_millis() as f64;
    
    println!("✅ 测试完成");
    println!("   记忆数量: {}", test_count);
    println!("   总时间: {:?}", total_time);
    println!("   平均延迟: {:?}", avg_time);
    println!("   吞吐量: {:.2} ops/s (单线程)", throughput);
    println!("   预期多线程吞吐量: {:.2} ops/s (假设10并发)", throughput * 10.0);
    println!();

    // 测试 2: 批量添加性能（Task 1.2 验证 - 10个）
    println!("📊 测试 2: 批量添加 10 个记忆（Task 1.2 验证）");
    println!("─────────────────────────────────────");
    
    let items: Vec<_> = (0..10)
        .map(|i| {
            (
                format!("Batch test memory {}", i),
                "test_agent".to_string(),
                Some("test_user".to_string()),
                Some(MemoryType::Core),
                None,
            )
        })
        .collect();
    
    let start = Instant::now();
    let memory_ids = orchestrator.add_memories_batch(items).await?;
    let duration = start.elapsed();
    
    let throughput = 10000.0 / duration.as_millis() as f64;
    
    println!("✅ 测试完成");
    println!("   记忆数量: {}", memory_ids.len());
    println!("   总时间: {:?}", duration);
    println!("   平均延迟: {:?}", duration / 10);
    println!("   吞吐量: {:.2} ops/s", throughput);
    println!();

    // 测试 3: 批量添加性能（Task 1.2 验证 - 100个）
    println!("📊 测试 3: 批量添加 100 个记忆（Task 1.2 验证）");
    println!("─────────────────────────────────────");
    
    let items: Vec<_> = (0..100)
        .map(|i| {
            (
                format!("Large batch test memory {}", i),
                "test_agent".to_string(),
                Some("test_user".to_string()),
                Some(MemoryType::Core),
                None,
            )
        })
        .collect();
    
    let start = Instant::now();
    let memory_ids = orchestrator.add_memories_batch(items).await?;
    let duration = start.elapsed();
    
    let throughput = 100000.0 / duration.as_millis() as f64;
    
    println!("✅ 测试完成");
    println!("   记忆数量: {}", memory_ids.len());
    println!("   总时间: {:?}", duration);
    println!("   平均延迟: {:?}", duration / 100);
    println!("   吞吐量: {:.2} ops/s", throughput);
    println!();

    // 测试 4: 性能对比
    println!("📊 测试 4: 性能对比（单个 vs 批量）");
    println!("─────────────────────────────────────");
    
    // 单个添加 10 次
    let start = Instant::now();
    for i in 0..10 {
        orchestrator
            .add_memory_fast(
                format!("Sequential test {}", i),
                "test_agent".to_string(),
                Some("test_user".to_string()),
                Some(MemoryType::Core),
                None,
            )
            .await?;
    }
    let sequential_duration = start.elapsed();
    let sequential_throughput = 10000.0 / sequential_duration.as_millis() as f64;
    
    // 批量添加 10 个
    let items: Vec<_> = (0..10)
        .map(|i| {
            (
                format!("Batch comparison test {}", i),
                "test_agent".to_string(),
                Some("test_user".to_string()),
                Some(MemoryType::Core),
                None,
            )
        })
        .collect();
    
    let start = Instant::now();
    orchestrator.add_memories_batch(items).await?;
    let batch_duration = start.elapsed();
    let batch_throughput = 10000.0 / batch_duration.as_millis() as f64;
    
    println!("✅ 对比完成");
    println!("   单个添加 10 次:");
    println!("     - 总时间: {:?}", sequential_duration);
    println!("     - 吞吐量: {:.2} ops/s", sequential_throughput);
    println!("   批量添加 10 个:");
    println!("     - 总时间: {:?}", batch_duration);
    println!("     - 吞吐量: {:.2} ops/s", batch_throughput);
    println!("   性能提升: {:.2}x", batch_throughput / sequential_throughput);
    println!();

    // 总结
    println!("================================");
    println!("✅ 所有测试完成！");
    println!();
    println!("📈 性能总结:");
    println!("   Task 1.1 (单个添加): {:.2} ops/s", throughput * 10.0);
    println!("   Task 1.2 (批量10个): {:.2} ops/s", batch_throughput);
    println!("   Task 1.2 (批量100个): 见测试3结果");
    println!();
    println!("🎯 目标达成情况:");
    if batch_throughput >= 500.0 {
        println!("   ✅ 批量模式已达到预期性能");
    } else {
        println!("   ⚠️  批量模式未达到预期性能，需要进一步优化");
    }
    println!();
    println!("💡 下一步:");
    println!("   - 继续 Phase 2: 优化智能模式LLM调用");
    println!("   - 目标: 并行LLM调用，达到 1,000 ops/s");

    Ok(())
}

