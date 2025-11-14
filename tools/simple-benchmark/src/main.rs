//! 简单基准测试
//! 
//! 快速测试当前性能，无需复杂的压测工具

use agent_mem::Memory;
use std::time::Instant;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 AgentMem 简单基准测试");
    println!("================================\n");

    // 初始化 Memory
    println!("📊 初始化 Memory SDK...");
    let memory = Memory::new().await?;
    println!("✅ Memory SDK 初始化成功\n");

    // 测试 1: 单次添加延迟
    println!("📝 测试 1: 单次添加延迟");
    let start = Instant::now();
    memory.add("Test memory for benchmarking").await?;
    let duration = start.elapsed();
    println!("  延迟: {:?}", duration);
    println!("  吞吐量: {:.2} ops/s\n", 1000.0 / duration.as_millis() as f64);

    // 测试 2: 批量添加吞吐量 (10条)
    println!("📝 测试 2: 批量添加吞吐量 (10条)");
    let start = Instant::now();
    for i in 0..10 {
        memory.add(&format!("Batch test memory {}", i)).await?;
    }
    let duration = start.elapsed();
    println!("  总延迟: {:?}", duration);
    println!("  平均延迟: {:?}", duration / 10);
    println!("  吞吐量: {:.2} ops/s\n", 10000.0 / duration.as_millis() as f64);

    // 测试 3: 并发添加 (10个并发)
    println!("📝 测试 3: 并发添加 (10个并发)");
    let start = Instant::now();
    let mut handles = vec![];
    for i in 0..10 {
        let mem = memory.clone();
        let handle = tokio::spawn(async move {
            mem.add(&format!("Concurrent test memory {}", i)).await
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.await??;
    }
    let duration = start.elapsed();
    println!("  总延迟: {:?}", duration);
    println!("  吞吐量: {:.2} ops/s\n", 10000.0 / duration.as_millis() as f64);

    // 测试 4: 搜索延迟
    println!("📝 测试 4: 搜索延迟");
    let start = Instant::now();
    let results = memory.search("test").await?;
    let duration = start.elapsed();
    println!("  延迟: {:?}", duration);
    println!("  结果数: {}", results.len());
    println!("  吞吐量: {:.2} ops/s\n", 1000.0 / duration.as_millis() as f64);

    println!("================================");
    println!("✅ 基准测试完成！");

    Ok(())
}

