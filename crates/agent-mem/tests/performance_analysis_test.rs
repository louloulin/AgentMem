//! 性能瓶颈分析测试
//! 
//! 分析为什么性能这么差，找出真正的瓶颈

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
async fn test_embedding_performance_bottleneck() {
    // 测试嵌入生成的性能瓶颈
    let mem = Arc::new(create_test_memory().await);
    
    // 获取 embedder
    let orchestrator = mem.orchestrator.read().await;
    let embedder = orchestrator.embedder.as_ref().expect("Embedder should be initialized");
    
    let test_content = "This is a test memory for performance analysis";
    
    // 测试单个嵌入生成时间
    let start = Instant::now();
    for _ in 0..10 {
        let _ = embedder.embed(test_content).await;
    }
    let single_embed_time = start.elapsed();
    println!("单个嵌入生成（10次串行）: {:?}, 平均: {:?}", single_embed_time, single_embed_time / 10);
    
    // 测试批量嵌入生成时间
    let contents: Vec<String> = (0..10).map(|i| format!("Test memory {}", i)).collect();
    let start = Instant::now();
    let _ = embedder.embed_batch(&contents).await;
    let batch_embed_time = start.elapsed();
    println!("批量嵌入生成（10个）: {:?}, 平均: {:?}", batch_embed_time, batch_embed_time / 10);
    
    // 计算性能提升
    let speedup = single_embed_time.as_secs_f64() / batch_embed_time.as_secs_f64();
    println!("批量嵌入性能提升: {:.2}x", speedup);
    
    drop(orchestrator);
}

#[tokio::test]
async fn test_concurrent_embedding_bottleneck() {
    // 测试并发场景下的嵌入生成瓶颈
    let mem = Arc::new(create_test_memory().await);
    
    let start = Instant::now();
    
    // 并发执行 10 个添加操作（每个都要生成嵌入）
    let mut tasks = Vec::new();
    for i in 0..10 {
        let mem_clone = Arc::clone(&mem);
        let task = tokio::spawn(async move {
            let embed_start = Instant::now();
            let result = mem_clone
                .add_for_user(
                    format!("Concurrent memory item {}", i),
                    format!("user-{}", i % 3),
                )
                .await;
            let embed_time = embed_start.elapsed();
            (result, embed_time)
        });
        tasks.push(task);
    }
    
    let mut total_embed_time = std::time::Duration::ZERO;
    let mut success_count = 0;
    for task in tasks {
        match task.await {
            Ok((Ok(_), embed_time)) => {
                success_count += 1;
                total_embed_time += embed_time;
            }
            Ok((Err(e), _)) => eprintln!("添加失败: {:?}", e),
            Err(e) => eprintln!("任务失败: {:?}", e),
        }
    }
    
    let total_time = start.elapsed();
    let avg_embed_time = total_embed_time / success_count as u32;
    
    println!("并发添加性能分析:");
    println!("  总耗时: {:?}", total_time);
    println!("  平均每个操作耗时: {:?}", total_time / success_count as u32);
    println!("  平均嵌入生成时间: {:?}", avg_embed_time);
    println!("  嵌入生成占比: {:.1}%", avg_embed_time.as_secs_f64() / (total_time.as_secs_f64() / success_count as f64) * 100.0);
    println!("  吞吐量: {:.2} ops/s", success_count as f64 / total_time.as_secs_f64());
}

#[tokio::test]
async fn test_database_write_performance() {
    // 测试数据库写入性能
    let mem = Arc::new(create_test_memory().await);
    
    // 先准备嵌入（避免嵌入生成影响测试）
    let orchestrator = mem.orchestrator.read().await;
    let embedder = orchestrator.embedder.as_ref().expect("Embedder should be initialized");
    let test_embedding = embedder.embed("Test content for database write").await.expect("Embedding failed");
    drop(orchestrator);
    
    let start = Instant::now();
    
    // 测试直接数据库写入（使用 add_memory_fast，但跳过嵌入生成）
    // 由于 add_memory_fast 内部会生成嵌入，我们测试整个流程
    let mut tasks = Vec::new();
    for i in 0..50 {
        let mem_clone = Arc::clone(&mem);
        let task = tokio::spawn(async move {
            mem_clone
                .add_for_user(
                    format!("DB write test {}", i),
                    "test-user".to_string(),
                )
                .await
        });
        tasks.push(task);
    }
    
    let mut success_count = 0;
    for task in tasks {
        match task.await {
            Ok(Ok(_)) => success_count += 1,
            Ok(Err(e)) => eprintln!("写入失败: {:?}", e),
            Err(e) => eprintln!("任务失败: {:?}", e),
        }
    }
    
    let total_time = start.elapsed();
    let ops_per_sec = success_count as f64 / total_time.as_secs_f64();
    
    println!("数据库写入性能测试:");
    println!("  成功: {}/50", success_count);
    println!("  总耗时: {:?}", total_time);
    println!("  吞吐量: {:.2} ops/s", ops_per_sec);
}

#[tokio::test]
async fn test_batch_vs_concurrent_performance() {
    // 对比批量操作 vs 并发操作的性能
    let mem = Arc::new(create_test_memory().await);
    
    // 测试并发操作（每个操作独立生成嵌入）
    let start = Instant::now();
    let mut tasks = Vec::new();
    for i in 0..50 {
        let mem_clone = Arc::clone(&mem);
        let task = tokio::spawn(async move {
            mem_clone
                .add_for_user(
                    format!("Concurrent test {}", i),
                    "batch-user".to_string(),
                )
                .await
        });
        tasks.push(task);
    }
    
    let mut concurrent_success = 0;
    for task in tasks {
        match task.await {
            Ok(Ok(_)) => concurrent_success += 1,
            _ => {}
        }
    }
    let concurrent_time = start.elapsed();
    let concurrent_ops = concurrent_success as f64 / concurrent_time.as_secs_f64();
    
    println!("并发操作性能:");
    println!("  成功: {}/50", concurrent_success);
    println!("  耗时: {:?}", concurrent_time);
    println!("  吞吐量: {:.2} ops/s", concurrent_ops);
    
    // 等待一下，避免资源竞争
    sleep(tokio::time::Duration::from_millis(100)).await;
    
    // 测试批量操作（使用批量嵌入）
    let start = Instant::now();
    let contents: Vec<String> = (0..50).map(|i| format!("Batch test {}", i)).collect();
    
    // 使用批量添加（需要检查是否有批量 API）
    let mut batch_success = 0;
    for content in contents {
        match mem.add_for_user(content, "batch-user".to_string()).await {
            Ok(_) => batch_success += 1,
            _ => {}
        }
    }
    let batch_time = start.elapsed();
    let batch_ops = batch_success as f64 / batch_time.as_secs_f64();
    
    println!("批量操作性能（串行）:");
    println!("  成功: {}/50", batch_success);
    println!("  耗时: {:?}", batch_time);
    println!("  吞吐量: {:.2} ops/s", batch_ops);
    
    println!("性能对比:");
    println!("  并发操作: {:.2} ops/s", concurrent_ops);
    println!("  批量操作: {:.2} ops/s", batch_ops);
    println!("  差异: {:.2}x", if batch_ops > 0.0 { concurrent_ops / batch_ops } else { 0.0 });
}
