//! 性能基准测试模块
//!
//! 提供全面的性能基准测试功能，包括：
//! - 向量搜索基准测试
//! - 图查询基准测试
//! - 缓存性能基准测试
//! - 批量操作基准测试

use agent_mem_traits::Result;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// 基准测试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    /// 测试迭代次数
    pub iterations: usize,

    /// 预热迭代次数
    pub warmup_iterations: usize,

    /// 向量维度
    pub vector_dimension: usize,

    /// 测试数据集大小
    pub dataset_size: usize,

    /// 是否启用详细输出
    pub verbose: bool,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            iterations: 1000,
            warmup_iterations: 100,
            vector_dimension: 384,
            dataset_size: 10000,
            verbose: false,
        }
    }
}

/// 性能基准测试
pub struct PerformanceBenchmark {
    /// 配置
    config: BenchmarkConfig,

    /// 测试结果
    results: Vec<BenchmarkResult>,
}

impl PerformanceBenchmark {
    /// 创建新的基准测试
    pub fn new(config: BenchmarkConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
        }
    }

    /// 运行所有基准测试
    pub async fn run_all_benchmarks(&mut self) -> Result<BenchmarkResult> {
        tracing::info!("Running all performance benchmarks...");

        let mut result = BenchmarkResult::default();

        // 向量搜索基准测试
        result.vector_search = self.benchmark_vector_search().await?;

        // 图查询基准测试
        result.graph_query = self.benchmark_graph_query().await?;

        // 缓存性能基准测试
        result.cache_performance = self.benchmark_cache().await?;

        // 批量操作基准测试
        result.batch_operations = self.benchmark_batch_operations().await?;

        self.results.push(result.clone());

        tracing::info!("All benchmarks completed");

        Ok(result)
    }

    /// 向量搜索基准测试
    async fn benchmark_vector_search(&self) -> Result<MetricResult> {
        tracing::info!("Benchmarking vector search...");

        let mut durations = Vec::new();

        // 预热
        for _ in 0..self.config.warmup_iterations {
            let _ = self.simulate_vector_search().await;
        }

        // 实际测试
        for _ in 0..self.config.iterations {
            let start = Instant::now();
            let _ = self.simulate_vector_search().await;
            durations.push(start.elapsed());
        }

        Ok(MetricResult::from_durations(&durations))
    }

    /// 图查询基准测试
    async fn benchmark_graph_query(&self) -> Result<MetricResult> {
        tracing::info!("Benchmarking graph query...");

        let mut durations = Vec::new();

        // 预热
        for _ in 0..self.config.warmup_iterations {
            let _ = self.simulate_graph_query().await;
        }

        // 实际测试
        for _ in 0..self.config.iterations {
            let start = Instant::now();
            let _ = self.simulate_graph_query().await;
            durations.push(start.elapsed());
        }

        Ok(MetricResult::from_durations(&durations))
    }

    /// 缓存性能基准测试
    async fn benchmark_cache(&self) -> Result<MetricResult> {
        tracing::info!("Benchmarking cache performance...");

        let mut durations = Vec::new();

        // 预热
        for _ in 0..self.config.warmup_iterations {
            let _ = self.simulate_cache_access().await;
        }

        // 实际测试
        for _ in 0..self.config.iterations {
            let start = Instant::now();
            let _ = self.simulate_cache_access().await;
            durations.push(start.elapsed());
        }

        Ok(MetricResult::from_durations(&durations))
    }

    /// 批量操作基准测试
    async fn benchmark_batch_operations(&self) -> Result<MetricResult> {
        tracing::info!("Benchmarking batch operations...");

        let mut durations = Vec::new();

        // 预热
        for _ in 0..self.config.warmup_iterations {
            let _ = self.simulate_batch_operation().await;
        }

        // 实际测试
        for _ in 0..self.config.iterations {
            let start = Instant::now();
            let _ = self.simulate_batch_operation().await;
            durations.push(start.elapsed());
        }

        Ok(MetricResult::from_durations(&durations))
    }

    /// 模拟向量搜索
    async fn simulate_vector_search(&self) -> Result<()> {
        // 模拟向量搜索操作
        let query_vector: Vec<f32> = (0..self.config.vector_dimension)
            .map(|_| rand::random::<f32>())
            .collect();

        // 模拟计算相似度
        let _ = query_vector.iter().map(|x| x * x).sum::<f32>();

        // 模拟异步延迟
        tokio::time::sleep(Duration::from_micros(10)).await;

        Ok(())
    }

    /// 模拟图查询
    async fn simulate_graph_query(&self) -> Result<()> {
        // 模拟图遍历操作
        let mut visited = std::collections::HashSet::new();
        for i in 0..100 {
            visited.insert(i);
        }

        // 模拟异步延迟
        tokio::time::sleep(Duration::from_micros(20)).await;

        Ok(())
    }

    /// 模拟缓存访问
    async fn simulate_cache_access(&self) -> Result<()> {
        // 模拟缓存查找
        let mut cache = std::collections::HashMap::new();
        cache.insert("key", "value");
        let _ = cache.get("key");

        // 模拟异步延迟
        tokio::time::sleep(Duration::from_micros(1)).await;

        Ok(())
    }

    /// 模拟批量操作
    async fn simulate_batch_operation(&self) -> Result<()> {
        // 模拟批量处理
        let items: Vec<i32> = (0..100).collect();
        let _results: Vec<_> = items.iter().map(|x| x * 2).collect();

        // 模拟异步延迟
        tokio::time::sleep(Duration::from_micros(50)).await;

        Ok(())
    }

    /// 获取所有测试结果
    pub fn get_results(&self) -> &[BenchmarkResult] {
        &self.results
    }

    /// 打印结果
    pub fn print_results(&self) {
        if self.results.is_empty() {
            println!("No benchmark results available");
            return;
        }

        let latest = &self.results[self.results.len() - 1];

        println!("\n=== Performance Benchmark Results ===\n");

        println!("Vector Search:");
        latest.vector_search.print("  ");

        println!("\nGraph Query:");
        latest.graph_query.print("  ");

        println!("\nCache Performance:");
        latest.cache_performance.print("  ");

        println!("\nBatch Operations:");
        latest.batch_operations.print("  ");

        println!("\n=====================================\n");
    }
}

/// 基准测试结果
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// 向量搜索结果
    pub vector_search: MetricResult,

    /// 图查询结果
    pub graph_query: MetricResult,

    /// 缓存性能结果
    pub cache_performance: MetricResult,

    /// 批量操作结果
    pub batch_operations: MetricResult,
}

/// 指标结果
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetricResult {
    /// 平均延迟（微秒）
    pub avg_latency_us: f64,

    /// 最小延迟（微秒）
    pub min_latency_us: f64,

    /// 最大延迟（微秒）
    pub max_latency_us: f64,

    /// P50 延迟（微秒）
    pub p50_latency_us: f64,

    /// P95 延迟（微秒）
    pub p95_latency_us: f64,

    /// P99 延迟（微秒）
    pub p99_latency_us: f64,

    /// 吞吐量（操作/秒）
    pub throughput_ops: f64,
}

impl MetricResult {
    /// 从持续时间列表创建结果
    fn from_durations(durations: &[Duration]) -> Self {
        if durations.is_empty() {
            return Self::default();
        }

        let mut sorted: Vec<_> = durations.iter().map(|d| d.as_micros() as f64).collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let avg = sorted.iter().sum::<f64>() / sorted.len() as f64;
        let min = sorted[0];
        let max = sorted[sorted.len() - 1];
        let p50 = sorted[sorted.len() / 2];
        let p95 = sorted[(sorted.len() as f64 * 0.95) as usize];
        let p99 = sorted[(sorted.len() as f64 * 0.99) as usize];

        let throughput = 1_000_000.0 / avg; // 操作/秒

        Self {
            avg_latency_us: avg,
            min_latency_us: min,
            max_latency_us: max,
            p50_latency_us: p50,
            p95_latency_us: p95,
            p99_latency_us: p99,
            throughput_ops: throughput,
        }
    }

    /// 打印结果
    fn print(&self, prefix: &str) {
        println!("{}Avg: {:.2} μs", prefix, self.avg_latency_us);
        println!("{}Min: {:.2} μs", prefix, self.min_latency_us);
        println!("{}Max: {:.2} μs", prefix, self.max_latency_us);
        println!("{}P50: {:.2} μs", prefix, self.p50_latency_us);
        println!("{}P95: {:.2} μs", prefix, self.p95_latency_us);
        println!("{}P99: {:.2} μs", prefix, self.p99_latency_us);
        println!("{}Throughput: {:.2} ops/s", prefix, self.throughput_ops);
    }
}
