//! AgentMem 性能基准测试
//! 对比 Mem0 和 AgentMem 的性能

use agent_mem::Memory;
use agent_mem::types::AddMemoryOptions;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=".repeat(60));
    println!("AgentMem 性能基准测试");
    println!("{}", "=".repeat(60));

    // 测试 1: 单个添加（使用 mem0_mode）
    println!("\n{}", "=".repeat(60));
    println!("测试 1: 单个添加 (mem0_mode)");
    println!("{}", "=".repeat(60));
    let ops1 = benchmark_agentmem_add(50).await?;

    // 测试 2: 批量添加
    println!("\n{}", "=".repeat(60));
    println!("测试 2: 批量添加");
    println!("{}", "=".repeat(60));
    let ops2 = benchmark_agentmem_batch_add(100).await?;

    // 测试 3: 并发单个添加（启用队列）
    println!("\n{}", "=".repeat(60));
    println!("测试 3: 并发单个添加（启用队列）");
    println!("{}", "=".repeat(60));
    let ops3 = benchmark_agentmem_concurrent_add(20, 5).await?;

    // 总结
    println!("\n{}", "=".repeat(60));
    println!("性能总结");
    println!("{}", "=".repeat(60));
    println!("单个添加 (50项): {:.2} ops/s", ops1);
    println!("批量添加 (100项): {:.2} ops/s", ops2);
    println!("并发单个添加 (20并发×5项): {:.2} ops/s", ops3);
    println!("\n注意: Mem0 目标性能为 10,000 ops/s (infer=False)");

    Ok(())
}

async fn create_test_memory() -> Result<Memory, Box<dyn std::error::Error>> {
    // 使用内存模式初始化（避免连接池问题）
    Memory::builder()
        .with_storage("memory://")  // 使用内存模式，避免连接池超时
        .with_embedder("fastembed", "BAAI/bge-small-en-v1.5")
        .enable_embedding_queue(64, 20)  // 启用队列优化
        .disable_intelligent_features()  // 禁用 LLM 推理，类似 infer=False
        .build()
        .await
        .map_err(|e| format!("初始化失败: {}", e).into())
}

async fn benchmark_agentmem_add(num_items: usize) -> Result<f64, Box<dyn std::error::Error>> {
    println!("测试项数: {}", num_items);

    // 使用内存模式初始化（避免连接池问题）
    let mem = create_test_memory().await?;

    // 准备测试数据
    let test_contents: Vec<String> = (0..num_items)
        .map(|i| format!("Test memory item {}: This is a test memory for performance benchmarking.", i))
        .collect();

    // 测试添加性能
    let start = Instant::now();
    let mut success_count = 0;

    for (i, content) in test_contents.iter().enumerate() {
        match mem.add_for_user(content.clone(), "test-user").await {
            Ok(_) => {
                success_count += 1;
                if (i + 1) % 10 == 0 {
                    println!("  已添加: {}/{}", i + 1, num_items);
                }
            }
            Err(e) => {
                eprintln!("  添加失败 (item {}): {:?}", i, e);
            }
        }
    }

    let elapsed = start.elapsed();
    let ops_per_sec = success_count as f64 / elapsed.as_secs_f64();

    println!("\n结果:");
    println!("  成功: {}/{}", success_count, num_items);
    println!("  耗时: {:.3} 秒", elapsed.as_secs_f64());
    println!("  吞吐量: {:.2} ops/s", ops_per_sec);
    println!("  平均延迟: {:.2} ms", elapsed.as_millis() as f64 / num_items as f64);

    Ok(ops_per_sec)
}

async fn benchmark_agentmem_batch_add(num_items: usize) -> Result<f64, Box<dyn std::error::Error>> {
    println!("测试项数: {}", num_items);

    // 使用内存模式初始化（避免连接池问题）
    let mem = create_test_memory().await?;
    
    // 注意：批量添加不需要单独验证 embedder，如果失败会有明确的错误信息

    // 准备测试数据
    let test_contents: Vec<String> = (0..num_items)
        .map(|i| format!("Test memory item {}: This is a test memory for performance benchmarking.", i))
        .collect();

    // 测试批量添加性能
    let start = Instant::now();

    let options = AddMemoryOptions {
        user_id: Some("test-user".to_string()),
        agent_id: Some("test-agent".to_string()),
        infer: false, // 禁用 LLM 推理，类似 Mem0 的 infer=False
        ..Default::default()
    };

    let results = mem
        .add_batch_optimized(test_contents, options)
        .await
        .map_err(|e| format!("批量添加失败: {}", e))?;

    let elapsed = start.elapsed();
    let ops_per_sec = results.len() as f64 / elapsed.as_secs_f64();

    println!("\n结果:");
    println!("  成功: {}/{}", results.len(), num_items);
    println!("  耗时: {:.3} 秒", elapsed.as_secs_f64());
    println!("  吞吐量: {:.2} ops/s", ops_per_sec);
    println!("  平均延迟: {:.2} ms", elapsed.as_millis() as f64 / num_items as f64);

    Ok(ops_per_sec)
}

async fn benchmark_agentmem_concurrent_add(
    concurrency: usize,
    items_per_task: usize,
) -> Result<f64, Box<dyn std::error::Error>> {
    println!("并发数: {}, 每任务项数: {}", concurrency, items_per_task);
    let total_items = concurrency * items_per_task;

    // 使用内存模式初始化（启用队列，避免连接池问题）
    let mem = std::sync::Arc::new(
        Memory::builder()
            .with_storage("memory://")  // 使用内存模式，避免连接池超时
            .with_embedder("fastembed", "BAAI/bge-small-en-v1.5")
            .enable_embedding_queue(64, 20)  // 启用队列优化
            .disable_intelligent_features()  // 禁用 LLM 推理，类似 infer=False
            .build()
            .await
            .map_err(|e| format!("初始化失败: {}", e))?,
    );

    // 准备测试数据
    let test_contents: Vec<Vec<String>> = (0..concurrency)
        .map(|task_id| {
            (0..items_per_task)
                .map(|item_id| {
                    format!(
                        "Concurrent test memory task {} item {}: This is a test memory for performance benchmarking.",
                        task_id, item_id
                    )
                })
                .collect()
        })
        .collect();

    // 测试并发添加性能
    let start = Instant::now();
    let mut tasks = Vec::new();

    for contents in test_contents {
        let mem_clone = std::sync::Arc::clone(&mem);
        let task = tokio::spawn(async move {
            let mut success_count = 0;
            for content in contents {
                match mem_clone.add_for_user(content, "test-user").await {
                    Ok(_) => success_count += 1,
                    Err(_) => {}
                }
            }
            success_count
        });
        tasks.push(task);
    }

    let mut total_success = 0;
    for task in tasks {
        total_success += task.await?;
    }

    let elapsed = start.elapsed();
    let ops_per_sec = total_success as f64 / elapsed.as_secs_f64();

    println!("\n结果:");
    println!("  成功: {}/{}", total_success, total_items);
    println!("  耗时: {:.3} 秒", elapsed.as_secs_f64());
    println!("  吞吐量: {:.2} ops/s", ops_per_sec);
    println!("  平均延迟: {:.2} ms", elapsed.as_millis() as f64 / total_items as f64);

    Ok(ops_per_sec)
}
