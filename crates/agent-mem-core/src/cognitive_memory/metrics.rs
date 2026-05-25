//! CognitiveMemory Performance Metrics
//!
//! Provides performance monitoring for CognitiveMemoryManager operations.
//! Tracks latency, throughput, and error rates for memory operations.

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// Performance metrics for memory operations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryMetrics {
    /// Total add operations
    pub total_adds: u64,
    /// Total retrieve operations
    pub total_retrieves: u64,
    /// Total delete operations
    pub total_deletes: u64,
    /// Total get operations
    pub total_gets: u64,
    /// Total batch operations
    pub total_batches: u64,
    /// Average add latency (microseconds)
    pub avg_add_latency_us: f64,
    /// Average retrieve latency (microseconds)
    pub avg_retrieve_latency_us: f64,
    /// Average delete latency (microseconds)
    pub avg_delete_latency_us: f64,
    /// Peak memory count
    pub peak_memory_count: usize,
    /// Operation errors
    pub errors: u64,
}

impl MemoryMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_add(&mut self, duration: Duration, count: usize) {
        self.total_adds += count as u64;
        let elapsed_us = duration.as_micros() as f64;
        if self.total_adds == count as u64 {
            self.avg_add_latency_us = elapsed_us / count as f64;
        } else {
            // Running average
            let prev_total = (self.total_adds - count as u64) as f64;
            let prev_sum = self.avg_add_latency_us * prev_total;
            self.avg_add_latency_us = (prev_sum + elapsed_us) / self.total_adds as f64;
        }
    }

    pub fn record_retrieve(&mut self, duration: Duration) {
        self.total_retrieves += 1;
        let elapsed_us = duration.as_micros() as f64;
        if self.total_retrieves == 1 {
            self.avg_retrieve_latency_us = elapsed_us;
        } else {
            let prev_total = (self.total_retrieves - 1) as f64;
            let prev_sum = self.avg_retrieve_latency_us * prev_total;
            self.avg_retrieve_latency_us = (prev_sum + elapsed_us) / self.total_retrieves as f64;
        }
    }

    pub fn record_delete(&mut self, duration: Duration) {
        self.total_deletes += 1;
        let elapsed_us = duration.as_micros() as f64;
        if self.total_deletes == 1 {
            self.avg_delete_latency_us = elapsed_us;
        } else {
            let prev_total = (self.total_deletes - 1) as f64;
            let prev_sum = self.avg_delete_latency_us * prev_total;
            self.avg_delete_latency_us = (prev_sum + elapsed_us) / self.total_deletes as f64;
        }
    }

    pub fn record_get(&mut self) {
        self.total_gets += 1;
    }

    pub fn record_batch(&mut self) {
        self.total_batches += 1;
    }

    pub fn record_error(&mut self) {
        self.errors += 1;
    }

    pub fn update_peak(&mut self, count: usize) {
        if count > self.peak_memory_count {
            self.peak_memory_count = count;
        }
    }

    /// Get throughput (operations per second)
    pub fn throughput(&self, duration_secs: f64) -> f64 {
        let total_ops =
            self.total_adds + self.total_retrieves + self.total_deletes + self.total_gets;
        total_ops as f64 / duration_secs
    }
}

/// Operation timing wrapper
pub struct OperationTimer {
    start: Instant,
}

impl OperationTimer {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

/// Memory operation statistics by type
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryStatsByType {
    pub semantic_count: usize,
    pub episodic_count: usize,
    pub procedural_count: usize,
    pub core_count: usize,
    pub working_count: usize,
    pub resource_count: usize,
    pub knowledge_count: usize,
    pub contextual_count: usize,
}

impl MemoryStatsByType {
    pub fn from_map(by_type: &HashMap<String, usize>) -> Self {
        let mut stats = Self::default();
        for (type_name, count) in by_type {
            match type_name.as_str() {
                "semantic" => stats.semantic_count = *count,
                "episodic" => stats.episodic_count = *count,
                "procedural" => stats.procedural_count = *count,
                "core" => stats.core_count = *count,
                "working" => stats.working_count = *count,
                "resource" => stats.resource_count = *count,
                "knowledge" => stats.knowledge_count = *count,
                "contextual" => stats.contextual_count = *count,
                _ => {}
            }
        }
        stats
    }

    pub fn total(&self) -> usize {
        self.semantic_count
            + self.episodic_count
            + self.procedural_count
            + self.core_count
            + self.working_count
            + self.resource_count
            + self.knowledge_count
            + self.contextual_count
    }
}

#[cfg(test)]
#[cfg(feature = "inline_tests")]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = MemoryMetrics::new();
        assert_eq!(metrics.total_adds, 0);
        assert_eq!(metrics.total_retrieves, 0);
        assert_eq!(metrics.total_deletes, 0);
    }

    #[test]
    fn test_record_add() {
        let mut metrics = MemoryMetrics::new();
        metrics.record_add(Duration::from_micros(100), 1);
        assert_eq!(metrics.total_adds, 1);
        assert_eq!(metrics.avg_add_latency_us, 100.0);
    }

    #[test]
    fn test_stats_by_type() {
        let mut by_type = HashMap::new();
        by_type.insert("semantic".to_string(), 5);
        by_type.insert("episodic".to_string(), 3);

        let stats = MemoryStatsByType::from_map(&by_type);
        assert_eq!(stats.semantic_count, 5);
        assert_eq!(stats.episodic_count, 3);
        assert_eq!(stats.total(), 8);
    }

    #[test]
    fn test_operation_timer() {
        let timer = OperationTimer::new();
        std::thread::sleep(Duration::from_micros(100));
        assert!(timer.elapsed().as_micros() >= 100);
    }
}

/// Cache statistics for memory operations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    /// Cache hits
    pub hits: u64,
    /// Cache misses
    pub misses: u64,
    /// Cache evictions
    pub evictions: u64,
    /// Current cache size
    pub current_size: usize,
}

impl CacheStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_hit(&mut self) {
        self.hits += 1;
    }

    pub fn record_miss(&mut self) {
        self.misses += 1;
    }

    pub fn record_eviction(&mut self) {
        self.evictions += 1;
    }

    pub fn update_size(&mut self, size: usize) {
        self.current_size = size;
    }

    /// Get hit rate (0.0 to 1.0)
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

#[cfg(test)]
mod cache_tests {
    use super::*;

    #[test]
    fn test_cache_stats() {
        let mut stats = CacheStats::new();
        stats.record_hit();
        stats.record_hit();
        stats.record_miss();
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate(), 2.0 / 3.0);
    }

    #[test]
    fn test_cache_stats_empty() {
        let stats = CacheStats::new();
        assert_eq!(stats.hit_rate(), 0.0);
    }
}
