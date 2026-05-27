//! Performance Benchmark Module for AgentMem
//!
//! This module provides performance benchmarking capabilities for:
//! - EventStore operations
//! - OptimisticLock operations
//! - Memory lifecycle operations
//!
//! # Usage
//!
//! ```rust
//! use crate::benchmarks::{run_all_benchmarks, BenchmarkConfig};
//!
//! #[tokio::main]
//! async fn main() {
//!     let results = run_all_benchmarks(BenchmarkConfig::default()).await;
//!     println!("{:?}", results);
//! }
//! ```

use crate::event_sourcing::{EventStore, MemoryEvent};
use crate::optimistic_lock::{LockManagerConfig, OptimisticLockManager};
use chrono::Utc;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Benchmark configuration
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    /// Number of iterations for each benchmark
    pub iterations: usize,
    /// Warmup iterations
    pub warmup_iterations: usize,
    /// Batch size for operations
    pub batch_size: usize,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            iterations: 1000,
            warmup_iterations: 10,
            batch_size: 100,
        }
    }
}

/// Benchmark result for a single operation
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// Operation name
    pub name: String,
    /// Total duration
    pub total_duration: Duration,
    /// Average duration per operation
    pub avg_duration: Duration,
    /// Minimum duration
    pub min_duration: Duration,
    /// Maximum duration
    pub max_duration: Duration,
    /// Operations per second
    pub ops_per_second: f64,
    /// Total operations
    pub total_operations: usize,
}

impl BenchmarkResult {
    /// Create a new benchmark result from durations
    pub fn new(name: String, durations: &[Duration]) -> Self {
        let total_duration: std::time::Duration = durations.iter().sum();
        let total_operations = durations.len();
        let avg_duration = total_duration / total_operations as u32;
        let min_duration = durations.iter().min().copied().unwrap_or_default();
        let max_duration = durations.iter().max().copied().unwrap_or_default();
        let ops_per_second = if total_duration.as_secs_f64() > 0.0 {
            total_operations as f64 / total_duration.as_secs_f64()
        } else {
            0.0
        };

        Self {
            name,
            total_duration,
            avg_duration,
            min_duration,
            max_duration,
            ops_per_second,
            total_operations,
        }
    }
}

/// Benchmark suite results
#[derive(Debug, Clone)]
pub struct BenchmarkSuiteResult {
    /// All benchmark results
    pub results: Vec<BenchmarkResult>,
    /// Timestamp
    pub timestamp: chrono::DateTime<Utc>,
    /// Configuration used
    pub config: BenchmarkConfig,
}

impl BenchmarkSuiteResult {
    /// Get summary statistics
    pub fn summary(&self) -> String {
        let mut output = format!("=== Benchmark Suite Results ===\n");
        output.push_str(&format!("Timestamp: {}\n", self.timestamp));
        output.push_str(&format!("Iterations: {}\n", self.config.iterations));
        output.push_str("----------------------------\n\n");

        for result in &self.results {
            output.push_str(&format!("{}:\n", result.name));
            output.push_str(&format!(
                "  Total: {:.2}ms\n",
                result.total_duration.as_secs_f64() * 1000.0
            ));
            output.push_str(&format!(
                "  Avg: {:.3}ms\n",
                result.avg_duration.as_secs_f64() * 1000.0
            ));
            output.push_str(&format!(
                "  Min/Max: {:.3}/{:.3}ms\n",
                result.min_duration.as_secs_f64() * 1000.0,
                result.max_duration.as_secs_f64() * 1000.0
            ));
            output.push_str(&format!(
                "  Ops/sec: {:.0}\n\n",
                result.ops_per_second
            ));
        }

        output
    }
}

/// Run all benchmarks
pub async fn run_all_benchmarks(config: BenchmarkConfig) -> BenchmarkSuiteResult {
    let mut results = Vec::new();

    // EventStore benchmarks
    results.push(event_store_append_benchmark(&config).await);
    results.push(event_store_replay_benchmark(&config).await);
    results.push(event_store_rebuild_benchmark(&config).await);
    results.push(event_store_snapshot_benchmark(&config).await);

    // OptimisticLock benchmarks
    results.push(optimistic_lock_init_benchmark(&config).await);
    results.push(optimistic_lock_verify_benchmark(&config).await);
    results.push(optimistic_lock_conflict_benchmark(&config).await);

    // Lifecycle benchmarks
    results.push(memory_lifecycle_benchmark(&config).await);

    BenchmarkSuiteResult {
        results,
        timestamp: Utc::now(),
        config,
    }
}

// Benchmark: EventStore append operations
async fn event_store_append_benchmark(config: &BenchmarkConfig) -> BenchmarkResult {
    let mut store = EventStore::new();
    let mut durations = Vec::with_capacity(config.iterations);

    // Warmup
    for i in 0..config.warmup_iterations {
        let event = create_test_event(&format!("warmup-{}", i));
        let _ = store.append("benchmark-memory", event).await;
    }
    store.clear("benchmark-memory");

    // Benchmark
    for i in 0..config.iterations {
        let event = create_test_event(&format!("bench-{}", i));
        let start = Instant::now();
        let _ = store.append("benchmark-memory", event).await;
        durations.push(start.elapsed());
    }

    BenchmarkResult::new("EventStore::append".to_string(), &durations)
}

// Benchmark: EventStore replay operations
async fn event_store_replay_benchmark(config: &BenchmarkConfig) -> BenchmarkResult {
    let mut store = EventStore::new();
    let memory_id = "replay-benchmark-memory";

    // Setup: Create events
    for i in 0..config.batch_size {
        let event = create_test_event(&format!("setup-{}", i));
        let _ = store.append(memory_id, event).await;
    }

    let mut durations = Vec::with_capacity(config.iterations);

    // Warmup
    for _ in 0..config.warmup_iterations {
        let _ = store.replay(memory_id).await;
    }

    // Benchmark
    for _ in 0..config.iterations {
        let start = Instant::now();
        let _ = store.replay(memory_id).await;
        durations.push(start.elapsed());
    }

    BenchmarkResult::new("EventStore::replay".to_string(), &durations)
}

// Benchmark: EventStore rebuild operations
async fn event_store_rebuild_benchmark(config: &BenchmarkConfig) -> BenchmarkResult {
    let mut store = EventStore::new();
    let memory_id = "rebuild-benchmark-memory";

    // Setup: Create events
    for i in 0..config.batch_size {
        let event = create_test_event(&format!("setup-{}", i));
        let _ = store.append(memory_id, event).await;
    }

    let mut durations = Vec::with_capacity(config.iterations);

    // Warmup
    for _ in 0..config.warmup_iterations {
        let _ = store.rebuild(memory_id).await;
    }

    // Benchmark
    for _ in 0..config.iterations {
        let start = Instant::now();
        let _ = store.rebuild(memory_id).await;
        durations.push(start.elapsed());
    }

    BenchmarkResult::new("EventStore::rebuild".to_string(), &durations)
}

// Benchmark: EventStore snapshot operations
async fn event_store_snapshot_benchmark(config: &BenchmarkConfig) -> BenchmarkResult {
    let mut store = EventStore::new();
    let memory_id = "snapshot-benchmark-memory";

    // Setup: Create events
    for i in 0..config.batch_size {
        let event = create_test_event(&format!("setup-{}", i));
        let _ = store.append(memory_id, event).await;
    }

    let mut durations = Vec::with_capacity(config.iterations);

    // Warmup
    for _ in 0..config.warmup_iterations {
        let _ = store.snapshot(memory_id).await;
    }

    // Benchmark
    for _ in 0..config.iterations {
        let start = Instant::now();
        let _ = store.snapshot(memory_id).await;
        durations.push(start.elapsed());
    }

    BenchmarkResult::new("EventStore::snapshot".to_string(), &durations)
}

// Benchmark: OptimisticLock init operations
async fn optimistic_lock_init_benchmark(config: &BenchmarkConfig) -> BenchmarkResult {
    let mut manager = OptimisticLockManager::new();
    let mut durations = Vec::with_capacity(config.iterations);

    // Warmup
    for i in 0..config.warmup_iterations {
        let _ = manager.init_version(&format!("warmup-{}", i));
    }

    // Benchmark
    for i in 0..config.iterations {
        let start = Instant::now();
        let _ = manager.init_version(&format!("bench-{}", i));
        durations.push(start.elapsed());
    }

    BenchmarkResult::new("OptimisticLock::init_version".to_string(), &durations)
}

// Benchmark: OptimisticLock verify and update operations
async fn optimistic_lock_verify_benchmark(config: &BenchmarkConfig) -> BenchmarkResult {
    let mut manager = OptimisticLockManager::new();
    manager.init_version("verify-benchmark").unwrap();

    let mut durations = Vec::with_capacity(config.iterations);
    let mut version = 1u64;

    // Warmup
    for _ in 0..config.warmup_iterations {
        manager.verify_and_update("verify-benchmark", version, "content").unwrap();
        version += 1;
    }

    // Reset
    manager.init_version("verify-benchmark").unwrap();
    version = 1;

    // Benchmark
    for _ in 0..config.iterations {
        let start = Instant::now();
        let _ = manager.verify_and_update("verify-benchmark", version, "content");
        duration_loop(
            &mut manager,
            &mut version,
            &mut durations,
            start,
            config.iterations,
        );
    }

    fn duration_loop(
        manager: &mut OptimisticLockManager,
        version: &mut u64,
        durations: &mut Vec<Duration>,
        start: Instant,
        _iterations: usize,
    ) {
        let _ = manager.verify_and_update("verify-benchmark", *version, "content");
        durations.push(start.elapsed());
        *version += 1;
    }

    BenchmarkResult::new("OptimisticLock::verify_and_update".to_string(), &durations)
}

// Benchmark: OptimisticLock conflict detection
async fn optimistic_lock_conflict_benchmark(config: &BenchmarkConfig) -> BenchmarkResult {
    let mut manager = OptimisticLockManager::new();
    manager.init_version("conflict-benchmark").unwrap();

    let mut durations = Vec::with_capacity(config.iterations);

    // Benchmark: Try to update with stale version repeatedly
    for _ in 0..config.iterations {
        let start = Instant::now();
        let _ = manager.verify_and_update("conflict-benchmark", 1, "content");
        durations.push(start.elapsed());
    }

    BenchmarkResult::new(
        "OptimisticLock::version_conflict_detection".to_string(),
        &durations,
    )
}

// Benchmark: Memory lifecycle (combined operations)
async fn memory_lifecycle_benchmark(config: &BenchmarkConfig) -> BenchmarkResult {
    let mut event_store = EventStore::new();
    let mut lock_manager = OptimisticLockManager::new();
    let mut durations = Vec::with_capacity(config.iterations);

    // Warmup
    for i in 0..config.warmup_iterations {
        let memory_id = format!("lifecycle-warmup-{}", i);
        lock_manager.init_version(&memory_id).unwrap();
        event_store
            .append(
                &memory_id,
                create_test_event(&format!("warmup-{}", i)),
            )
            .await
            .unwrap();
        let _ = event_store.rebuild(&memory_id).await;
    }

    // Benchmark
    for i in 0..config.iterations {
        let memory_id = format!("lifecycle-bench-{}", i);
        let start = Instant::now();

        // Init version
        lock_manager.init_version(&memory_id).unwrap();

        // Append event
        event_store
            .append(&memory_id, create_test_event(&format!("bench-{}", i)))
            .await
            .unwrap();

        // Rebuild
        let _ = event_store.rebuild(&memory_id).await;

        durations.push(start.elapsed());
    }

    BenchmarkResult::new("Memory_Lifecycle".to_string(), &durations)
}

// Helper: Create a test memory event
fn create_test_event(content: &str) -> MemoryEvent {
    MemoryEvent::Created {
        memory_id: content.to_string(),
        content: format!("Test content for {}", content),
        memory_type: "benchmark".to_string(),
        importance: 0.5,
        timestamp: Utc::now(),
        metadata: HashMap::new(),
    }
}

/// Run quick benchmark (for CI/development)
pub async fn run_quick_benchmark() -> BenchmarkSuiteResult {
    let config = BenchmarkConfig {
        iterations: 100,
        warmup_iterations: 5,
        batch_size: 50,
    };
    run_all_benchmarks(config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_benchmark_execution() {
        let config = BenchmarkConfig {
            iterations: 10,
            warmup_iterations: 2,
            batch_size: 5,
        };

        let results = run_all_benchmarks(config).await;

        // Verify we have results for all benchmarks
        assert_eq!(results.results.len(), 8);

        // Verify each result has valid data
        for result in &results.results {
            assert!(result.total_operations > 0);
            assert!(result.avg_duration > Duration::default());
            assert!(result.ops_per_second >= 0.0);
        }
    }

    #[tokio::test]
    async fn test_quick_benchmark() {
        let results = run_quick_benchmark().await;

        // Verify we have results
        assert!(!results.results.is_empty());

        // Print summary for debugging
        println!("{}", results.summary());
    }

    #[test]
    fn test_benchmark_result_creation() {
        let durations = vec![
            Duration::from_micros(100),
            Duration::from_micros(150),
            Duration::from_micros(120),
        ];

        let result = BenchmarkResult::new("test".to_string(), &durations);

        assert_eq!(result.total_operations, 3);
        assert!(result.total_duration > Duration::default());
        assert!(result.avg_duration > Duration::default());
    }

    #[test]
    fn test_benchmark_config_default() {
        let config = BenchmarkConfig::default();
        assert_eq!(config.iterations, 1000);
        assert_eq!(config.warmup_iterations, 10);
        assert_eq!(config.batch_size, 100);
    }
}