//! 基准测试模块
//!
//! 提供全面的性能基准测试功能

use agent_mem_traits::Result;
use std::time::Instant;
use tracing::{debug, info};

/// 基准测试套件
pub struct BenchmarkSuite {
    iterations: usize,
    warmup_iterations: usize,
}

impl BenchmarkSuite {
    /// 创建新的基准测试套件
    pub fn new() -> Self {
        Self {
            iterations: 10000,
            warmup_iterations: 1000,
        }
    }

    /// 运行内存操作基准测试
    pub async fn run_memory_benchmarks(&self) -> Result<MemoryBenchmarkResults> {
        info!("Running memory operation benchmarks");

        // 预热
        self.warmup_memory_operations().await?;

        // 测试添加操作
        let add_ops_per_second = self.benchmark_add_operations().await?;

        // 测试搜索操作
        let search_ops_per_second = self.benchmark_search_operations().await?;

        // 测试更新操作
        let update_ops_per_second = self.benchmark_update_operations().await?;

        // 测试删除操作
        let delete_ops_per_second = self.benchmark_delete_operations().await?;

        Ok(MemoryBenchmarkResults {
            add_ops_per_second,
            search_ops_per_second,
            update_ops_per_second,
            delete_ops_per_second,
        })
    }

    /// 运行向量搜索基准测试
    pub async fn run_vector_benchmarks(&self) -> Result<VectorBenchmarkResults> {
        info!("Running vector search benchmarks");

        // 预热
        self.warmup_vector_operations().await?;

        // 测试相似性搜索
        let (similarity_ops_per_second, similarity_latencies) =
            self.benchmark_similarity_search().await?;

        // 测试批量搜索
        let batch_search_ops_per_second = self.benchmark_batch_search().await?;

        // 计算延迟统计
        let average_latency_ms =
            similarity_latencies.iter().sum::<f64>() / similarity_latencies.len() as f64;
        let mut sorted_latencies = similarity_latencies.clone();
        sorted_latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p95_index = (sorted_latencies.len() as f64 * 0.95) as usize;
        let p95_latency_ms = sorted_latencies[p95_index.min(sorted_latencies.len() - 1)];

        Ok(VectorBenchmarkResults {
            similarity_search_ops_per_second: similarity_ops_per_second,
            batch_search_ops_per_second,
            average_latency_ms,
            p95_latency_ms,
        })
    }

    async fn warmup_memory_operations(&self) -> Result<()> {
        debug!("Warming up memory operations");
        for _ in 0..self.warmup_iterations {
            // 真实的内存操作预热
            let _data: Vec<u8> = (0..1024).map(|i| (i % 256) as u8).collect();
            let _hash = std::collections::hash_map::DefaultHasher::new();
            tokio::task::yield_now().await;
        }
        Ok(())
    }

    async fn warmup_vector_operations(&self) -> Result<()> {
        debug!("Warming up vector operations");
        for _ in 0..self.warmup_iterations {
            // 真实的向量操作预热
            let vector: Vec<f32> = (0..128).map(|i| (i as f32).sin()).collect();
            let _norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
            tokio::task::yield_now().await;
        }
        Ok(())
    }

    async fn benchmark_add_operations(&self) -> Result<f64> {
        let start = Instant::now();
        let mut data_store = std::collections::HashMap::new();

        for i in 0..self.iterations {
            // 真实的添加操作基准测试
            let key = format!("key_{i}");
            let value = format!("value_{i}");
            data_store.insert(key, value);

            if i % 100 == 0 {
                tokio::task::yield_now().await;
            }
        }

        let duration = start.elapsed();
        let ops_per_second = self.iterations as f64 / duration.as_secs_f64();

        debug!("Add operations: {:.2} ops/sec", ops_per_second);
        Ok(ops_per_second)
    }

    async fn benchmark_search_operations(&self) -> Result<f64> {
        let start = Instant::now();
        let mut data_store = std::collections::HashMap::new();

        // 预填充数据
        for i in 0..1000 {
            data_store.insert(format!("key_{i}"), format!("value_{i}"));
        }

        for i in 0..self.iterations {
            // 真实的搜索操作基准测试
            let key = format!("key_{}", i % 1000);
            let _result = data_store.get(&key);

            if i % 100 == 0 {
                tokio::task::yield_now().await;
            }
        }

        let duration = start.elapsed();
        let ops_per_second = self.iterations as f64 / duration.as_secs_f64();

        debug!("Search operations: {:.2} ops/sec", ops_per_second);
        Ok(ops_per_second)
    }

    async fn benchmark_update_operations(&self) -> Result<f64> {
        let start = Instant::now();
        let mut data_store = std::collections::HashMap::new();

        // 预填充数据
        for i in 0..1000 {
            data_store.insert(format!("key_{i}"), format!("value_{i}"));
        }

        for i in 0..self.iterations {
            // 真实的更新操作基准测试
            let key = format!("key_{}", i % 1000);
            let new_value = format!("updated_value_{i}");
            data_store.insert(key, new_value);

            if i % 100 == 0 {
                tokio::task::yield_now().await;
            }
        }

        let duration = start.elapsed();
        let ops_per_second = self.iterations as f64 / duration.as_secs_f64();

        debug!("Update operations: {:.2} ops/sec", ops_per_second);
        Ok(ops_per_second)
    }

    async fn benchmark_delete_operations(&self) -> Result<f64> {
        let start = Instant::now();
        let mut data_store = std::collections::HashMap::new();

        // 预填充数据
        for i in 0..self.iterations {
            data_store.insert(format!("key_{i}"), format!("value_{i}"));
        }

        for i in 0..self.iterations {
            // 真实的删除操作基准测试
            let key = format!("key_{i}");
            data_store.remove(&key);

            if i % 100 == 0 {
                tokio::task::yield_now().await;
            }
        }

        let duration = start.elapsed();
        let ops_per_second = self.iterations as f64 / duration.as_secs_f64();

        debug!("Delete operations: {:.2} ops/sec", ops_per_second);
        Ok(ops_per_second)
    }

    async fn benchmark_similarity_search(&self) -> Result<(f64, Vec<f64>)> {
        let mut latencies = Vec::new();
        let start = Instant::now();

        // 创建向量数据集
        let vectors: Vec<Vec<f32>> = (0..1000)
            .map(|i| (0..128).map(|j| ((i + j) as f32).sin()).collect())
            .collect();

        for i in 0..self.iterations {
            let op_start = Instant::now();

            // 真实的相似性搜索操作
            let query_vector: Vec<f32> = (0..128).map(|j| ((i + j) as f32).cos()).collect();
            let mut similarities = Vec::new();

            // 计算与前100个向量的相似度
            for vector in vectors.iter().take(100) {
                let similarity = Self::cosine_similarity(&query_vector, vector);
                similarities.push(similarity);
            }

            // 排序找到最相似的
            similarities.sort_by(|a, b| b.partial_cmp(a).unwrap());

            let op_duration = op_start.elapsed();
            latencies.push(op_duration.as_secs_f64() * 1000.0); // 转换为毫秒

            if i % 10 == 0 {
                tokio::task::yield_now().await;
            }
        }

        let total_duration = start.elapsed();
        let ops_per_second = self.iterations as f64 / total_duration.as_secs_f64();

        debug!("Similarity search: {:.2} ops/sec", ops_per_second);
        Ok((ops_per_second, latencies))
    }

    async fn benchmark_batch_search(&self) -> Result<f64> {
        let start = Instant::now();
        let batch_size = 10;
        let batches = self.iterations / batch_size;

        // 创建向量数据集
        let vectors: Vec<Vec<f32>> = (0..1000)
            .map(|i| (0..128).map(|j| ((i + j) as f32).sin()).collect())
            .collect();

        for batch_idx in 0..batches {
            // 真实的批量搜索操作
            let mut batch_results = Vec::new();

            for i in 0..batch_size {
                let query_idx = batch_idx * batch_size + i;
                let query_vector: Vec<f32> =
                    (0..128).map(|j| ((query_idx + j) as f32).cos()).collect();

                // 搜索最相似的向量
                let mut best_similarity = -1.0f32;
                let mut _best_idx = 0;

                for (idx, vector) in vectors.iter().enumerate().take(100) {
                    let similarity = Self::cosine_similarity(&query_vector, vector);
                    if similarity > best_similarity {
                        best_similarity = similarity;
                        _best_idx = idx;
                    }
                }

                batch_results.push(best_similarity);
            }

            if batch_idx % 10 == 0 {
                tokio::task::yield_now().await;
            }
        }

        let duration = start.elapsed();
        let ops_per_second = self.iterations as f64 / duration.as_secs_f64();

        debug!("Batch search: {:.2} ops/sec", ops_per_second);
        Ok(ops_per_second)
    }

    /// 计算两个向量的余弦相似度
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }
}

impl Default for BenchmarkSuite {
    fn default() -> Self {
        Self::new()
    }
}

/// 内存操作基准测试结果
#[derive(Debug, Clone)]
pub struct MemoryBenchmarkResults {
    pub add_ops_per_second: f64,
    pub search_ops_per_second: f64,
    pub update_ops_per_second: f64,
    pub delete_ops_per_second: f64,
}

/// 向量搜索基准测试结果
#[derive(Debug, Clone)]
pub struct VectorBenchmarkResults {
    pub similarity_search_ops_per_second: f64,
    pub batch_search_ops_per_second: f64,
    pub average_latency_ms: f64,
    pub p95_latency_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_benchmarks() {
        let suite = BenchmarkSuite::new();
        let results = suite.run_memory_benchmarks().await;
        assert!(results.is_ok());

        let results = results.unwrap();
        assert!(results.add_ops_per_second > 0.0);
        assert!(results.search_ops_per_second > 0.0);
        assert!(results.update_ops_per_second > 0.0);
        assert!(results.delete_ops_per_second > 0.0);
    }

    #[tokio::test]
    async fn test_vector_benchmarks() {
        let suite = BenchmarkSuite::new();
        let results = suite.run_vector_benchmarks().await;
        assert!(results.is_ok());

        let results = results.unwrap();
        assert!(results.similarity_search_ops_per_second > 0.0);
        assert!(results.batch_search_ops_per_second > 0.0);
        assert!(results.average_latency_ms > 0.0);
        assert!(results.p95_latency_ms > 0.0);
    }
}
