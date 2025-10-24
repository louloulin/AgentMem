///! AgentMem 性能基准测试
///!
///! 这是一个完整的性能基准测试工具，测试AgentMem的实际性能表现。
///!
///! 测试项目：
///! 1. 内存操作性能（添加、搜索、删除）
///! 2. 向量搜索性能
///! 3. 并发性能
///! 4. 大规模数据性能
///! 5. 延迟统计（平均、P95、P99）

use agent_mem::{Memory, MemoryBuilder};
use anyhow::Result;
use colored::*;
use std::time::{Duration, Instant};
use tracing::{info, warn};

/// 基准测试配置
#[derive(Debug, Clone)]
struct BenchmarkConfig {
    /// 测试迭代次数
    iterations: usize,
    /// 预热迭代次数
    warmup_iterations: usize,
    /// 并发任务数
    concurrent_tasks: usize,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            iterations: 1000,
            warmup_iterations: 10,
            concurrent_tasks: 10,
        }
    }
}

/// 基准测试结果
#[derive(Debug)]
struct BenchmarkResults {
    /// 操作名称
    operation: String,
    /// 总操作数
    total_operations: usize,
    /// 总耗时
    total_duration: Duration,
    /// 每秒操作数
    ops_per_second: f64,
    /// 平均延迟（毫秒）
    average_latency_ms: f64,
    /// P95延迟（毫秒）
    p95_latency_ms: f64,
    /// P99延迟（毫秒）
    p99_latency_ms: f64,
}

impl BenchmarkResults {
    fn new(operation: String, total_operations: usize, total_duration: Duration, latencies: &[f64]) -> Self {
        let ops_per_second = total_operations as f64 / total_duration.as_secs_f64();
        
        let average_latency_ms = if latencies.is_empty() {
            0.0
        } else {
            latencies.iter().sum::<f64>() / latencies.len() as f64
        };
        
        let mut sorted_latencies = latencies.to_vec();
        sorted_latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let p95_latency_ms = if sorted_latencies.is_empty() {
            0.0
        } else {
            let p95_index = (sorted_latencies.len() as f64 * 0.95) as usize;
            sorted_latencies[p95_index.min(sorted_latencies.len() - 1)]
        };
        
        let p99_latency_ms = if sorted_latencies.is_empty() {
            0.0
        } else {
            let p99_index = (sorted_latencies.len() as f64 * 0.99) as usize;
            sorted_latencies[p99_index.min(sorted_latencies.len() - 1)]
        };
        
        Self {
            operation,
            total_operations,
            total_duration,
            ops_per_second,
            average_latency_ms,
            p95_latency_ms,
            p99_latency_ms,
        }
    }
    
    fn display(&self) {
        println!("\n{}", format!("📊 {} 性能报告", self.operation).bold().blue());
        println!("{}", "─".repeat(60));
        println!("总操作数:     {}", format!("{}", self.total_operations).green());
        println!("总耗时:       {}", format!("{:.2}s", self.total_duration.as_secs_f64()).yellow());
        println!("吞吐量:       {}", format!("{:.2} ops/s", self.ops_per_second).cyan().bold());
        println!("平均延迟:     {}", format!("{:.2} ms", self.average_latency_ms).yellow());
        println!("P95 延迟:     {}", format!("{:.2} ms", self.p95_latency_ms).yellow());
        println!("P99 延迟:     {}", format!("{:.2} ms", self.p99_latency_ms).red());
        println!("{}", "─".repeat(60));
    }
}

/// 创建测试用的Memory实例
async fn create_test_memory() -> Result<Memory> {
    MemoryBuilder::new()
        .with_agent("benchmark_agent")
        .with_embedder("fastembed", "all-MiniLM-L6-v2")
        .disable_intelligent_features()
        .build()
        .await
}

/// 1. 内存添加操作基准测试
async fn benchmark_add_operations(config: &BenchmarkConfig) -> Result<BenchmarkResults> {
    info!("开始测试：内存添加操作");
    
    let memory = create_test_memory().await?;
    let mut latencies = Vec::new();
    
    // 预热
    for i in 0..config.warmup_iterations {
        memory.add(&format!("warmup content {}", i)).await?;
    }
    
    // 实际测试
    let start = Instant::now();
    for i in 0..config.iterations {
        let op_start = Instant::now();
        memory.add(&format!("Benchmark test content number {}", i)).await?;
        let op_duration = op_start.elapsed();
        latencies.push(op_duration.as_secs_f64() * 1000.0);
    }
    let total_duration = start.elapsed();
    
    Ok(BenchmarkResults::new(
        "内存添加操作".to_string(),
        config.iterations,
        total_duration,
        &latencies,
    ))
}

/// 2. 内存搜索操作基准测试
async fn benchmark_search_operations(config: &BenchmarkConfig) -> Result<BenchmarkResults> {
    info!("开始测试：内存搜索操作");
    
    let memory = create_test_memory().await?;
    let mut latencies = Vec::new();
    
    // 预填充数据
    for i in 0..100 {
        memory.add(&format!("Test data {} for searching benchmark", i)).await?;
    }
    
    // 预热
    for _ in 0..config.warmup_iterations {
        memory.search("test".to_string()).await?;
    }
    
    // 实际测试
    let start = Instant::now();
    for i in 0..config.iterations {
        let op_start = Instant::now();
        let query = format!("searching {}", i % 10);
        memory.search(query).await?;
        let op_duration = op_start.elapsed();
        latencies.push(op_duration.as_secs_f64() * 1000.0);
    }
    let total_duration = start.elapsed();
    
    Ok(BenchmarkResults::new(
        "内存搜索操作".to_string(),
        config.iterations,
        total_duration,
        &latencies,
    ))
}

/// 3. 内存删除操作基准测试
async fn benchmark_delete_operations(config: &BenchmarkConfig) -> Result<BenchmarkResults> {
    info!("开始测试：内存删除操作");
    
    let memory = create_test_memory().await?;
    let mut latencies = Vec::new();
    let mut ids = Vec::new();
    
    // 预填充数据
    for i in 0..config.iterations {
        let result = memory.add(&format!("Content to be deleted {}", i)).await?;
        if let Some(id) = result.results.first() {
            ids.push(id.id.clone());
        }
    }
    
    // 实际测试
    let start = Instant::now();
    for id in ids {
        let op_start = Instant::now();
        memory.delete(&id).await?;
        let op_duration = op_start.elapsed();
        latencies.push(op_duration.as_secs_f64() * 1000.0);
    }
    let total_duration = start.elapsed();
    
    Ok(BenchmarkResults::new(
        "内存删除操作".to_string(),
        config.iterations,
        total_duration,
        &latencies,
    ))
}

/// 4. 并发添加操作基准测试
async fn benchmark_concurrent_add(config: &BenchmarkConfig) -> Result<BenchmarkResults> {
    info!("开始测试：并发添加操作");
    
    let memory = create_test_memory().await?;
    let operations_per_task = config.iterations / config.concurrent_tasks;
    
    // 实际测试
    let start = Instant::now();
    let mut handles = Vec::new();
    
    for task_id in 0..config.concurrent_tasks {
        let memory_clone = memory.clone();
        let handle = tokio::spawn(async move {
            let mut task_latencies = Vec::new();
            for i in 0..operations_per_task {
                let op_start = Instant::now();
                let content = format!("Concurrent content from task {} iteration {}", task_id, i);
                let _ = memory_clone.add(&content).await;
                let op_duration = op_start.elapsed();
                task_latencies.push(op_duration.as_secs_f64() * 1000.0);
            }
            task_latencies
        });
        handles.push(handle);
    }
    
    // 收集所有任务的延迟
    let mut all_latencies = Vec::new();
    for handle in handles {
        let task_latencies = handle.await?;
        all_latencies.extend(task_latencies);
    }
    
    let total_duration = start.elapsed();
    
    Ok(BenchmarkResults::new(
        format!("并发添加操作（{}个任务）", config.concurrent_tasks),
        config.iterations,
        total_duration,
        &all_latencies,
    ))
}

/// 5. 大规模数据搜索基准测试
async fn benchmark_large_scale_search(config: &BenchmarkConfig) -> Result<BenchmarkResults> {
    info!("开始测试：大规模数据搜索");
    
    let memory = create_test_memory().await?;
    let data_size = 1000; // 1000条记录
    
    // 预填充大量数据
    println!("正在预填充 {} 条记忆数据...", data_size);
    for i in 0..data_size {
        let content = format!(
            "Large scale test data item {} with various keywords like technology, AI, memory, agent, search",
            i
        );
        memory.add(&content).await?;
        
        if i % 100 == 0 {
            print!(".");
            std::io::Write::flush(&mut std::io::stdout()).ok();
        }
    }
    println!(" 完成！");
    
    // 实际测试
    let mut latencies = Vec::new();
    let search_queries = vec!["technology", "AI", "memory", "agent", "search"];
    let start = Instant::now();
    
    for i in 0..config.iterations {
        let query = search_queries[i % search_queries.len()].to_string();
        let op_start = Instant::now();
        memory.search(query).await?;
        let op_duration = op_start.elapsed();
        latencies.push(op_duration.as_secs_f64() * 1000.0);
    }
    let total_duration = start.elapsed();
    
    Ok(BenchmarkResults::new(
        format!("大规模搜索（{}条数据）", data_size),
        config.iterations,
        total_duration,
        &latencies,
    ))
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();
    
    println!("\n{}", "🚀 AgentMem 性能基准测试工具".bold().green());
    println!("{}", "═".repeat(60));
    println!("\n{}", "测试配置:".bold());
    
    let config = BenchmarkConfig {
        iterations: 100,  // 减少迭代次数以加快测试
        warmup_iterations: 10,
        concurrent_tasks: 5,
    };
    
    println!("  迭代次数:     {}", config.iterations);
    println!("  预热次数:     {}", config.warmup_iterations);
    println!("  并发任务数:   {}", config.concurrent_tasks);
    println!();
    
    // 运行所有基准测试
    let mut all_results = Vec::new();
    
    // 1. 内存添加操作
    println!("\n{}", "▶ 测试 1/5: 内存添加操作".yellow().bold());
    match benchmark_add_operations(&config).await {
        Ok(results) => {
            results.display();
            all_results.push(results);
        }
        Err(e) => warn!("测试失败: {}", e),
    }
    
    // 2. 内存搜索操作
    println!("\n{}", "▶ 测试 2/5: 内存搜索操作".yellow().bold());
    match benchmark_search_operations(&config).await {
        Ok(results) => {
            results.display();
            all_results.push(results);
        }
        Err(e) => warn!("测试失败: {}", e),
    }
    
    // 3. 内存删除操作
    println!("\n{}", "▶ 测试 3/5: 内存删除操作".yellow().bold());
    match benchmark_delete_operations(&config).await {
        Ok(results) => {
            results.display();
            all_results.push(results);
        }
        Err(e) => warn!("测试失败: {}", e),
    }
    
    // 4. 并发添加操作
    println!("\n{}", "▶ 测试 4/5: 并发添加操作".yellow().bold());
    match benchmark_concurrent_add(&config).await {
        Ok(results) => {
            results.display();
            all_results.push(results);
        }
        Err(e) => warn!("测试失败: {}", e),
    }
    
    // 5. 大规模数据搜索
    println!("\n{}", "▶ 测试 5/5: 大规模数据搜索".yellow().bold());
    match benchmark_large_scale_search(&config).await {
        Ok(results) => {
            results.display();
            all_results.push(results);
        }
        Err(e) => warn!("测试失败: {}", e),
    }
    
    // 总结报告
    println!("\n\n{}", "🎯 性能测试总结".bold().green());
    println!("{}", "═".repeat(60));
    
    for result in &all_results {
        println!(
            "{:30} | {:>12} | {:>12} | {:>12}",
            result.operation.bright_white(),
            format!("{:.0} ops/s", result.ops_per_second).cyan(),
            format!("{:.1} ms", result.average_latency_ms).yellow(),
            format!("{:.1} ms", result.p95_latency_ms).red()
        );
    }
    
    println!("{}", "═".repeat(60));
    println!("\n{}", "✅ 所有性能测试完成！".bold().green());
    
    // 性能评估
    println!("\n{}", "📈 性能评估:".bold().blue());
    for result in &all_results {
        let assessment = if result.ops_per_second > 100.0 {
            "优秀 ✨".green().bold()
        } else if result.ops_per_second > 50.0 {
            "良好 ✓".cyan()
        } else {
            "需要优化 ⚠".yellow()
        };
        
        println!("  {} - {}", result.operation, assessment);
    }
    
    println!();
    
    Ok(())
}

