//! Vector Store Benchmark Module
//!
//! Provides benchmarking utilities for vector store operations.

use agent_mem_traits::{VectorData, VectorSearchResult, VectorStore};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Benchmark result for a single operation
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub operation: String,
    pub iterations: usize,
    pub total_duration: Duration,
    pub avg_duration_ms: f64,
    pub min_duration_ms: f64,
    pub max_duration_ms: f64,
    pub throughput_per_sec: f64,
}

impl BenchmarkResult {
    pub fn new(operation: &str, durations: Vec<Duration>) -> Self {
        let iterations = durations.len();
        let total_duration: Duration = durations.iter().sum();
        let avg_ms = total_duration.as_secs_f64() * 1000.0 / iterations as f64;
        let min_ms = durations.iter().map(|d| d.as_secs_f64() * 1000.0).fold(f64::INFINITY, f64::min);
        let max_ms = durations.iter().map(|d| d.as_secs_f64() * 1000.0).fold(0.0, f64::max);
        let throughput = iterations as f64 / total_duration.as_secs_f64();

        Self {
            operation: operation.to_string(),
            iterations,
            total_duration,
            avg_duration_ms: avg_ms,
            min_duration_ms: min_ms,
            max_duration_ms: max_ms,
            throughput_per_sec: throughput,
        }
    }

    pub fn print_summary(&self) {
        println!("\n=== {} Benchmark ===", self.operation);
        println!("Iterations: {}", self.iterations);
        println!("Total: {:?}", self.total_duration);
        println!("Avg: {:.3} ms", self.avg_duration_ms);
        println!("Min: {:.3} ms", self.min_duration_ms);
        println!("Max: {:.3} ms", self.max_duration_ms);
        println!("Throughput: {:.2} ops/sec", self.throughput_per_sec);
    }
}

/// Benchmark runner for vector stores
pub struct VectorStoreBenchmark<S: VectorStore> {
    store: S,
}

impl<S: VectorStore> VectorStoreBenchmark<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }

    /// Benchmark add_vectors operation
    pub async fn benchmark_add(
        &self,
        vectors: Vec<VectorData>,
        iterations: usize,
    ) -> BenchmarkResult {
        let mut durations = Vec::with_capacity(iterations);

        for _ in 0..iterations {
            let ids = vectors.iter().map(|v| v.id.clone()).collect::<Vec<_>>();
            
            let start = Instant::now();
            let _ = self.store.add_vectors(vectors.clone()).await;
            durations.push(start.elapsed());
            
            // Cleanup for next iteration
            let _ = self.store.delete_vectors(ids).await;
        }

        BenchmarkResult::new("add_vectors", durations)
    }

    /// Benchmark search_vectors operation
    pub async fn benchmark_search(
        &self,
        query: Vec<f32>,
        limit: usize,
        iterations: usize,
    ) -> BenchmarkResult {
        let mut durations = Vec::with_capacity(iterations);

        for _ in 0..iterations {
            let start = Instant::now();
            let _ = self.store.search_vectors(query.clone(), limit, None).await;
            durations.push(start.elapsed());
        }

        BenchmarkResult::new("search_vectors", durations)
    }

    /// Run full benchmark suite
    pub async fn run_full_benchmark(
        &self,
        num_vectors: usize,
        dimension: usize,
        num_searches: usize,
    ) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();

        // Prepare test vectors
        let vectors: Vec<VectorData> = (0..num_vectors)
            .map(|i| VectorData {
                id: format!("bench_{}", i),
                vector: vec![i as f32 / num_vectors as f32; dimension],
                metadata: HashMap::new(),
            })
            .collect();

        // Add vectors first
        println!("Adding {} vectors...", num_vectors);
        self.store.add_vectors(vectors.clone()).await.unwrap();

        // Benchmark search
        let query = vec![0.5; dimension];
        println!("Running {} search iterations...", num_searches);
        let search_result = self.benchmark_search(query, 10, num_searches).await;
        search_result.print_summary();
        results.push(search_result);

        // Cleanup
        let ids: Vec<String> = vectors.into_iter().map(|v| v.id).collect();
        self.store.delete_vectors(ids).await.unwrap();

        results
    }
}

/// Generate random vectors for testing
pub fn generate_random_vectors(num: usize, dimension: usize, seed: u64) -> Vec<VectorData> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    (0..num)
        .map(|i| {
            let mut hasher = DefaultHasher::new();
            (i, seed).hash(&mut hasher);
            let hash = hasher.finish();
            
            VectorData {
                id: format!("vec_{}", i),
                vector: (0..dimension)
                    .map(|j| {
                        let mut h = DefaultHasher::new();
                        (hash, j).hash(&mut h);
                        ((h.finish() as f64 / u64::MAX as f64) * 2.0 - 1.0) as f32
                    })
                    .collect(),
                metadata: HashMap::new(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::memory::MemoryVectorStore;

    #[tokio::test]
    async fn test_benchmark_creation() {
        let store = MemoryVectorStore::new();
        let benchmark = VectorStoreBenchmark::new(store);
        
        // Quick smoke test
        let vectors = generate_random_vectors(10, 4, 42);
        benchmark.store.add_vectors(vectors).await.unwrap();
        
        let result = benchmark.benchmark_search(vec![0.1, 0.2, 0.3, 0.4], 5, 3).await;
        
        assert_eq!(result.operation, "search_vectors");
        assert_eq!(result.iterations, 3);
        assert!(result.avg_duration_ms >= 0.0);
    }

    #[tokio::test]
    async fn test_generate_vectors() {
        let vectors = generate_random_vectors(5, 3, 42);
        
        assert_eq!(vectors.len(), 5);
        assert_eq!(vectors[0].vector.len(), 3);
        assert_eq!(vectors[0].id, "vec_0");
    }
}
