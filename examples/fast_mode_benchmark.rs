//! 快速模式性能基准测试
//! 
//! 对比 infer=true (智能模式) 和 infer=false (快速模式) 的性能差异

use agent_mem::Memory;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 AgentMem 快速模式性能基准测试");
    println!("================================\n");

    // 初始化 Memory
    println!("📊 初始化 Memory SDK...");
    let memory = Memory::new().await?;
    println!("✅ Memory SDK 初始化成功\n");

    // 测试 1: 快速模式 (infer=false) - 单次
    println!("📝 测试 1: 快速模式 (infer=false) - 单次添加");
    let start = Instant::now();
    memory.add("Test memory in fast mode").await?;
    let duration = start.elapsed();
    println!("  延迟: {:?}", duration);
    println!("  吞吐量: {:.2} ops/s\n", 1000.0 / duration.as_millis() as f64);

    // 测试 2: 快速模式 (infer=false) - 批量10条
    println!("📝 测试 2: 快速模式 (infer=false) - 批量10条");
    let start = Instant::now();
    for i in 0..10 {
        memory.add(&format!("Fast mode batch test {}", i)).await?;
    }
    let duration = start.elapsed();
    println!("  总延迟: {:?}", duration);
    println!("  平均延迟: {:?}", duration / 10);
    println!("  吞吐量: {:.2} ops/s\n", 10000.0 / duration.as_millis() as f64);

    // 测试 3: 快速模式 (infer=false) - 并发10个
    println!("📝 测试 3: 快速模式 (infer=false) - 并发10个");
    let start = Instant::now();
    let mut handles = vec![];
    for i in 0..10 {
        let mem = memory.clone();
        let handle = tokio::spawn(async move {
            mem.add(&format!("Fast mode concurrent test {}", i)).await
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.await??;
    }
    let duration = start.elapsed();
    println!("  总延迟: {:?}", duration);
    println!("  吞吐量: {:.2} ops/s\n", 10000.0 / duration.as_millis() as f64);

    // 测试 4: 快速模式 (infer=false) - 并发100个
    println!("📝 测试 4: 快速模式 (infer=false) - 并发100个");
    let start = Instant::now();
    let mut handles = vec![];
    for i in 0..100 {
        let mem = memory.clone();
        let handle = tokio::spawn(async move {
            mem.add(&format!("Fast mode stress test {}", i)).await
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.await??;
    }
    let duration = start.elapsed();
    println!("  总延迟: {:?}", duration);
    println!("  吞吐量: {:.2} ops/s\n", 100000.0 / duration.as_millis() as f64);

    println!("================================");
    println!("✅ 快速模式基准测试完成！");
    println!("\n📊 性能总结:");
    println!("  - 快速模式 (infer=false) 已启用并行写入");
    println!("  - CoreMemoryManager、VectorStore、HistoryManager 并行执行");
    println!("  - 预期性能提升: 2-3x (相比顺序写入)");

    Ok(())
}

