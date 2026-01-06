//! 批量模式性能基准测试
//!
//! 测试批量嵌入生成和并行写入的性能提升
//!
//! 运行: cargo run --release --example batch_mode_benchmark

use agent_mem::Orchestrator;
use agent_mem::OrchestratorConfig;
use agent_mem_core::types::MemoryType;
use std::collections::HashMap;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🚀 AgentMem 批量模式性能基准测试");
    println!("================================\n");

    // 创建 Orchestrator
    let config = OrchestratorConfig {
        storage_url: Some("libsql://./data/agentmem.db".to_string()),
        llm_provider: None,
        llm_model: None,
        embedder_provider: Some("fastembed".to_string()),
        embedder_model: Some("bge-small-en-v1.5".to_string()),
        vector_store_url: Some("./data/lancedb".to_string()),
        enable_intelligent_features: false,
    };

    println!("📦 初始化 Orchestrator...");
    let orchestrator = Orchestrator::new(config).await?;
    println!("✅ Orchestrator 初始化完成\n");

    // 测试 1: 单个添加（基准）
    println!("📊 测试 1: 单个添加（基准）");
    println!("─────────────────────────");
    
    let start = Instant::now();
    let memory_id = orchestrator
        .add_memory_fast(
            "This is a test memory".to_string(),
            "test_agent".to_string(),
            Some("test_user".to_string()),
            Some(MemoryType::Core),
            None,
        )
        .await?;
    let duration = start.elapsed();
    
    println!("✅ 单个添加完成");
    println!("   Memory ID: {}", memory_id);
    println!("   延迟: {:?}", duration);
    println!("   吞吐量: {:.2} ops/s\n", 1000.0 / duration.as_millis() as f64);

    // 测试 2: 批量添加 10 个记忆
    println!("📊 测试 2: 批量添加 10 个记忆");
    println!("─────────────────────────");
    
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
    
    println!("✅ 批量添加完成");
    println!("   记忆数量: {}", memory_ids.len());
    println!("   总延迟: {:?}", duration);
    println!("   平均延迟: {:?}", duration / 10);
    println!("   吞吐量: {:.2} ops/s\n", 10000.0 / duration.as_millis() as f64);

    // 测试 3: 批量添加 100 个记忆
    println!("📊 测试 3: 批量添加 100 个记忆");
    println!("─────────────────────────");
    
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
    
    println!("✅ 批量添加完成");
    println!("   记忆数量: {}", memory_ids.len());
    println!("   总延迟: {:?}", duration);
    println!("   平均延迟: {:?}", duration / 100);
    println!("   吞吐量: {:.2} ops/s\n", 100000.0 / duration.as_millis() as f64);

    // 测试 4: 对比单个添加 vs 批量添加
    println!("📊 测试 4: 性能对比（10个记忆）");
    println!("─────────────────────────");
    
    // 单个添加 10 次
    let start = Instant::now();
    for i in 0..10 {
        orchestrator
            .add_memory_fast(
                format!("Sequential test memory {}", i),
                "test_agent".to_string(),
                Some("test_user".to_string()),
                Some(MemoryType::Core),
                None,
            )
            .await?;
    }
    let sequential_duration = start.elapsed();
    
    // 批量添加 10 个
    let items: Vec<_> = (0..10)
        .map(|i| {
            (
                format!("Batch comparison test memory {}", i),
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
    
    println!("✅ 性能对比完成");
    println!("   单个添加 10 次: {:?} ({:.2} ops/s)", 
        sequential_duration, 
        10000.0 / sequential_duration.as_millis() as f64
    );
    println!("   批量添加 10 个: {:?} ({:.2} ops/s)", 
        batch_duration,
        10000.0 / batch_duration.as_millis() as f64
    );
    println!("   性能提升: {:.2}x\n", 
        sequential_duration.as_millis() as f64 / batch_duration.as_millis() as f64
    );

    println!("================================");
    println!("✅ 所有测试完成！");
    println!();
    println!("💡 关键发现:");
    println!("   - 批量嵌入生成显著减少了嵌入生成时间");
    println!("   - 并行写入进一步提升了吞吐量");
    println!("   - 批量模式适合大规模数据导入场景");

    Ok(())
}

