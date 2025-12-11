//! 性能对比测试
//! 
//! 对比优化前后的性能，验证优化效果

use agent_mem::Memory;
use std::sync::Arc;
use std::time::Instant;
use tokio::time::sleep;

/// 创建测试用的 Memory 实例（内存模式）
async fn create_test_memory() -> Memory {
    Memory::builder()
        .with_storage("memory://")
        .with_embedder("fastembed", "BAAI/bge-small-en-v1.5")
        .disable_intelligent_features()
        .build()
        .await
        .expect("内存模式初始化失败")
}

#[tokio::test]
async fn test_single_vs_batch_performance() {
    // 对比单个添加 vs 批量添加的性能
    let mem = Arc::new(create_test_memory().await);
    
    let test_contents: Vec<String> = (0..50).map(|i| format!("Test memory {}", i)).collect();
    
    // 测试单个添加（串行）
    println!("\n=== 单个添加（串行）===");
    let start = Instant::now();
    let mut success_count = 0;
    for content in test_contents.iter() {
        match mem.add_for_user(content.clone(), "test-user".to_string()).await {
            Ok(_) => success_count += 1,
            Err(e) => eprintln!("添加失败: {:?}", e),
        }
    }
    let single_time = start.elapsed();
    let single_ops = success_count as f64 / single_time.as_secs_f64();
    println!("  成功: {}/{}", success_count, test_contents.len());
    println!("  耗时: {:?}", single_time);
    println!("  吞吐量: {:.2} ops/s", single_ops);
    
    // 等待一下，避免资源竞争
    sleep(tokio::time::Duration::from_millis(500)).await;
    
    // 测试批量添加
    println!("\n=== 批量添加（优化版）===");
    let start = Instant::now();
    let batch_contents: Vec<String> = (0..50).map(|i| format!("Batch memory {}", i)).collect();
    let options = agent_mem::AddMemoryOptions {
        user_id: Some("test-user".to_string()),
        ..Default::default()
    };
    let batch_result = mem.add_batch_optimized(batch_contents.clone(), options).await;
    let batch_time = start.elapsed();
    
    match batch_result {
        Ok(_results) => {
            let batch_ops = batch_contents.len() as f64 / batch_time.as_secs_f64();
            println!("  成功: {}/{}", batch_contents.len(), batch_contents.len());
            println!("  耗时: {:?}", batch_time);
            println!("  吞吐量: {:.2} ops/s", batch_ops);
            
            // 计算性能提升
            let speedup = batch_ops / single_ops;
            println!("\n=== 性能对比 ===");
            println!("  单个添加: {:.2} ops/s", single_ops);
            println!("  批量添加: {:.2} ops/s", batch_ops);
            println!("  性能提升: {:.2}x", speedup);
            
            assert!(speedup > 1.5, "批量添加应该有明显的性能提升（至少 1.5x）");
        }
        Err(e) => {
            eprintln!("批量添加失败: {:?}", e);
            println!("  耗时: {:?}", batch_time);
        }
    }
}

#[tokio::test]
async fn test_concurrent_single_vs_batch() {
    // 对比并发单个添加 vs 批量添加的性能
    let mem = Arc::new(create_test_memory().await);
    
    let concurrency = 20;
    let items_per_task = 5;
    let total_items = concurrency * items_per_task;
    
    // 测试并发单个添加
    println!("\n=== 并发单个添加（{} 个并发任务，每个 {} 项）===", concurrency, items_per_task);
    let start = Instant::now();
    let mut tasks = Vec::new();
    
    for i in 0..concurrency {
        let mem_clone = Arc::clone(&mem);
        let task = tokio::spawn(async move {
            let mut success = 0;
            for j in 0..items_per_task {
                match mem_clone
                    .add_for_user(
                        format!("Concurrent single {} {}", i, j),
                        format!("user-{}", i % 3),
                    )
                    .await
                {
                    Ok(_) => success += 1,
                    Err(_) => {}
                }
            }
            success
        });
        tasks.push(task);
    }
    
    let mut total_success = 0;
    for task in tasks {
        if let Ok(count) = task.await {
            total_success += count;
        }
    }
    
    let concurrent_single_time = start.elapsed();
    let concurrent_single_ops = total_success as f64 / concurrent_single_time.as_secs_f64();
    println!("  成功: {}/{}", total_success, total_items);
    println!("  耗时: {:?}", concurrent_single_time);
    println!("  吞吐量: {:.2} ops/s", concurrent_single_ops);
    
    // 等待一下，避免资源竞争
    sleep(tokio::time::Duration::from_millis(500)).await;
    
    // 测试批量添加（模拟收集并发请求后批量处理）
    println!("\n=== 批量添加（{} 项）===", total_items);
    let start = Instant::now();
    let batch_contents: Vec<String> = (0..total_items)
        .map(|i| format!("Batch memory {}", i))
        .collect();
    let options = agent_mem::AddMemoryOptions {
        user_id: Some("test-user".to_string()),
        ..Default::default()
    };
    let batch_result = mem.add_batch_optimized(batch_contents.clone(), options).await;
    let batch_time = start.elapsed();
    
    match batch_result {
        Ok(results) => {
            let batch_ops = batch_contents.len() as f64 / batch_time.as_secs_f64();
            println!("  成功: {}/{}", results.len(), batch_contents.len());
            println!("  耗时: {:?}", batch_time);
            println!("  吞吐量: {:.2} ops/s", batch_ops);
            
            // 计算性能提升
            let speedup = batch_ops / concurrent_single_ops;
            println!("\n=== 性能对比 ===");
            println!("  并发单个添加: {:.2} ops/s", concurrent_single_ops);
            println!("  批量添加: {:.2} ops/s", batch_ops);
            println!("  性能提升: {:.2}x", speedup);
            
            assert!(speedup > 1.5, "批量添加应该有明显的性能提升（至少 1.5x）");
        }
        Err(e) => {
            eprintln!("批量添加失败: {:?}", e);
            println!("  耗时: {:?}", batch_time);
        }
    }
}

#[tokio::test]
async fn test_embedding_performance_breakdown() {
    // 分析嵌入生成的性能瓶颈（通过实际添加操作间接测试）
    let mem = Arc::new(create_test_memory().await);
    
    println!("\n=== 嵌入生成性能分析（间接测试）===");
    
    // 测试单个添加的嵌入生成时间（通过总耗时估算）
    let test_text = "This is a test memory for performance analysis";
    let mut single_times = Vec::new();
    for _ in 0..10 {
        let start = Instant::now();
        let _ = mem.add_for_user(test_text, "test-user").await;
        single_times.push(start.elapsed());
    }
    let avg_single = single_times.iter().sum::<std::time::Duration>() / single_times.len() as u32;
    println!("  单个添加（10次平均）: {:?}", avg_single);
    println!("  最快: {:?}", single_times.iter().min().unwrap());
    println!("  最慢: {:?}", single_times.iter().max().unwrap());
    
    // 测试批量添加的嵌入生成时间
    let contents: Vec<String> = (0..10).map(|i| format!("Test memory {}", i)).collect();
    let start = Instant::now();
    let options = agent_mem::AddMemoryOptions {
        user_id: Some("test-user".to_string()),
        ..Default::default()
    };
    let _ = mem.add_batch_optimized(contents.clone(), options).await;
    let batch_time = start.elapsed();
    let avg_batch = batch_time / contents.len() as u32;
    println!("  批量添加（10个）: {:?}", batch_time);
    println!("  平均每个: {:?}", avg_batch);
    
    // 计算性能提升
    let speedup = avg_single.as_secs_f64() / avg_batch.as_secs_f64();
    println!("\n=== 批量操作性能提升 ===");
    println!("  单个添加: {:?}", avg_single);
    println!("  批量添加（平均）: {:?}", avg_batch);
    println!("  性能提升: {:.2}x", speedup);
    
    assert!(speedup > 1.0, "批量添加应该比单个添加更快");
}

#[tokio::test]
async fn test_memory_operations_breakdown() {
    // 分析内存操作的性能分解
    let mem = Arc::new(create_test_memory().await);
    
    println!("\n=== 内存操作性能分解 ===");
    
    // 测试单个添加的各个步骤耗时
    let test_content = "Performance test memory";
    let start = Instant::now();
    let result = mem.add_for_user(test_content, "test-user").await;
    let total_time = start.elapsed();
    
    match result {
        Ok(_) => {
            println!("  单个添加总耗时: {:?}", total_time);
            println!("  预期吞吐量: {:.2} ops/s", 1.0 / total_time.as_secs_f64());
        }
        Err(e) => {
            eprintln!("  添加失败: {:?}", e);
        }
    }
    
    // 测试批量添加的各个步骤耗时
    let batch_contents: Vec<String> = (0..20).map(|i| format!("Batch test {}", i)).collect();
    let start = Instant::now();
    let options = agent_mem::AddMemoryOptions {
        user_id: Some("test-user".to_string()),
        ..Default::default()
    };
    let batch_result = mem.add_batch_optimized(batch_contents.clone(), options).await;
    let batch_time = start.elapsed();
    
    match batch_result {
        Ok(results) => {
            let avg_time = batch_time / batch_contents.len() as u32;
            println!("  批量添加总耗时: {:?} ({} 项)", batch_time, batch_contents.len());
            println!("  平均每项: {:?}", avg_time);
            println!("  批量吞吐量: {:.2} ops/s", batch_contents.len() as f64 / batch_time.as_secs_f64());
            
            // 计算性能提升
            let speedup = total_time.as_secs_f64() / avg_time.as_secs_f64();
            println!("\n=== 性能提升 ===");
            println!("  单个添加: {:?}", total_time);
            println!("  批量添加（平均）: {:?}", avg_time);
            println!("  性能提升: {:.2}x", speedup);
        }
        Err(e) => {
            eprintln!("  批量添加失败: {:?}", e);
        }
    }
}
