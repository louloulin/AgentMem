//! Metrics for AgentMem Cognitive
//! 
//! Provides runtime metrics for monitoring and observability

use crate::hierarchy::MemoryHierarchyStats;
use crate::archive::ArchiveStats;
use crate::review::ReviewStats;
use std::sync::atomic::{AtomicU64, Ordering};

/// Memory metrics
#[derive(Debug, Clone)]
pub struct MemoryMetrics {
    pub total_operations: u64,
    pub add_operations: u64,
    pub access_operations: u64,
    pub search_operations: u64,
    pub review_operations: u64,
    pub working_count: u64,
    pub core_count: u64,
    pub archive_count: u64,
    pub avg_retention: f64,
    pub pending_reviews: u64,
    pub avg_operation_latency_ms: f64,
}

impl Default for MemoryMetrics {
    fn default() -> Self {
        Self {
            total_operations: 0,
            add_operations: 0,
            access_operations: 0,
            search_operations: 0,
            review_operations: 0,
            working_count: 0,
            core_count: 0,
            archive_count: 0,
            avg_retention: 0.0,
            pending_reviews: 0,
            avg_operation_latency_ms: 0.0,
        }
    }
}

/// Metrics collector
pub struct MetricsCollector {
    total_operations: AtomicU64,
    add_operations: AtomicU64,
    access_operations: AtomicU64,
    search_operations: AtomicU64,
    review_operations: AtomicU64,
    total_latency_us: AtomicU64,
    operation_count: AtomicU64,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            total_operations: AtomicU64::new(0),
            add_operations: AtomicU64::new(0),
            access_operations: AtomicU64::new(0),
            search_operations: AtomicU64::new(0),
            review_operations: AtomicU64::new(0),
            total_latency_us: AtomicU64::new(0),
            operation_count: AtomicU64::new(0),
        }
    }
    
    pub fn record_add(&self, latency_us: u64) {
        self.total_operations.fetch_add(1, Ordering::Relaxed);
        self.add_operations.fetch_add(1, Ordering::Relaxed);
        self.total_latency_us.fetch_add(latency_us, Ordering::Relaxed);
        self.operation_count.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn record_access(&self, latency_us: u64) {
        self.total_operations.fetch_add(1, Ordering::Relaxed);
        self.access_operations.fetch_add(1, Ordering::Relaxed);
        self.total_latency_us.fetch_add(latency_us, Ordering::Relaxed);
        self.operation_count.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn record_search(&self, latency_us: u64) {
        self.total_operations.fetch_add(1, Ordering::Relaxed);
        self.search_operations.fetch_add(1, Ordering::Relaxed);
        self.total_latency_us.fetch_add(latency_us, Ordering::Relaxed);
        self.operation_count.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn record_review(&self, latency_us: u64) {
        self.total_operations.fetch_add(1, Ordering::Relaxed);
        self.review_operations.fetch_add(1, Ordering::Relaxed);
        self.total_latency_us.fetch_add(latency_us, Ordering::Relaxed);
        self.operation_count.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn snapshot(&self) -> MemoryMetrics {
        let total_ops = self.total_operations.load(Ordering::Relaxed);
        let total_latency = self.total_latency_us.load(Ordering::Relaxed);
        let op_count = self.operation_count.load(Ordering::Relaxed);
        
        MemoryMetrics {
            total_operations: total_ops,
            add_operations: self.add_operations.load(Ordering::Relaxed),
            access_operations: self.access_operations.load(Ordering::Relaxed),
            search_operations: self.search_operations.load(Ordering::Relaxed),
            review_operations: self.review_operations.load(Ordering::Relaxed),
            working_count: 0,
            core_count: 0,
            archive_count: 0,
            avg_retention: 0.0,
            pending_reviews: 0,
            avg_operation_latency_ms: if op_count > 0 {
                (total_latency as f64 / op_count as f64) / 1000.0
            } else {
                0.0
            },
        }
    }
    
    pub fn get_with_stats(
        &self,
        hierarchy_stats: &MemoryHierarchyStats,
        archive_stats: &ArchiveStats,
        review_stats: &ReviewStats,
    ) -> MemoryMetrics {
        let mut metrics = self.snapshot();
        metrics.working_count = hierarchy_stats.working_count as u64;
        metrics.core_count = hierarchy_stats.core_count as u64;
        metrics.archive_count = archive_stats.total_items as u64;
        metrics.avg_retention = review_stats.avg_retention as f64;
        metrics.pending_reviews = review_stats.pending_reviews as u64;
        metrics
    }
    
    pub fn reset(&self) {
        self.total_operations.store(0, Ordering::Relaxed);
        self.add_operations.store(0, Ordering::Relaxed);
        self.access_operations.store(0, Ordering::Relaxed);
        self.search_operations.store(0, Ordering::Relaxed);
        self.review_operations.store(0, Ordering::Relaxed);
        self.total_latency_us.store(0, Ordering::Relaxed);
        self.operation_count.store(0, Ordering::Relaxed);
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

pub struct OperationTimer {
    start: std::time::Instant,
}

impl OperationTimer {
    pub fn new() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }
    
    pub fn elapsed_us(&self) -> u64 {
        self.start.elapsed().as_micros() as u64
    }
    
    pub fn elapsed_ms(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
    }
}

impl Default for OperationTimer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collection() {
        let collector = MetricsCollector::new();
        
        collector.record_add(100);
        collector.record_access(50);
        collector.record_search(200);
        
        let metrics = collector.snapshot();
        assert_eq!(metrics.total_operations, 3);
        assert_eq!(metrics.add_operations, 1);
    }
    
    #[test]
    fn test_latency_calculation() {
        let collector = MetricsCollector::new();
        
        collector.record_add(1000);
        collector.record_add(2000);
        
        let metrics = collector.snapshot();
        assert!((metrics.avg_operation_latency_ms - 1.5).abs() < 0.1);
    }
}
