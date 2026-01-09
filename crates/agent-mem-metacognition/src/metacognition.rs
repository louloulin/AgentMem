//! Metacognition Service
//!
//! Provides memory health monitoring, statistics, and insights.
//!
//! # Features
//!
//! - Memory health scoring
//! - Usage statistics tracking
//! - Performance metrics
//! - Trend analysis
//!
//! # Example
//!
//! ```no_run
//! use agent_mem_metacognition::MetacognitionService;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let service = MetacognitionService::new().await?;
//!
//!     let report = service.generate_report().await?;
//!     println!("Health score: {}", report.health_score);
//!
//!     Ok(())
//! }
//! ```

use crate::history::MergeTracker;
use agent_mem_traits::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Metacognition configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetacognitionConfig {
    /// Health check interval (seconds)
    pub health_check_interval_seconds: u64,

    /// Enable automatic health monitoring
    pub enable_monitoring: bool,

    /// Retention period for statistics (days)
    pub statistics_retention_days: u64,
}

impl Default for MetacognitionConfig {
    fn default() -> Self {
        Self {
            health_check_interval_seconds: 1800, // 30 minutes
            enable_monitoring: true,
            statistics_retention_days: 30,
        }
    }
}

/// Memory health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryHealthMetrics {
    /// Total memories
    pub total_memories: usize,

    /// Active memories (accessed in last 7 days)
    pub active_memories: usize,

    /// Dormant memories (not accessed in >30 days)
    pub dormant_memories: usize,

    /// Fragmented memories (high similarity, not merged)
    pub fragmented_memories: usize,

    /// Memory health score (0-100)
    pub health_score: f64,

    /// Consolidation urgency (0-100, higher = more urgent)
    pub consolidation_urgency: f64,
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsageStats {
    /// Total memory accesses
    pub total_accesses: u64,

    /// Average accesses per memory
    pub avg_accesses_per_memory: f64,

    /// Most accessed memory ID
    pub most_accessed_memory_id: Option<String>,

    /// Access distribution (quartiles)
    pub access_distribution: AccessDistribution,
}

/// Access distribution quartiles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessDistribution {
    /// Q1 (25th percentile)
    pub q1: u64,

    /// Q2 (median, 50th percentile)
    pub q2: u64,

    /// Q3 (75th percentile)
    pub q3: u64,

    /// Maximum
    pub max: u64,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Average retrieval time (ms)
    pub avg_retrieval_time_ms: f64,

    /// Average consolidation time (ms)
    pub avg_consolidation_time_ms: f64,

    /// Cache hit rate
    pub cache_hit_rate: f64,

    /// Memory throughput (operations/second)
    pub throughput_ops_per_sec: f64,
}

/// Metacognitive report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetacognitionReport {
    /// Report generation timestamp
    pub generated_at: DateTime<Utc>,

    /// Health metrics
    pub health: MemoryHealthMetrics,

    /// Usage statistics
    pub usage: MemoryUsageStats,

    /// Performance metrics
    pub performance: PerformanceMetrics,

    /// Merge statistics
    pub merge_stats: MergeStatisticsSummary,

    /// Recommendations count
    pub recommendations_count: usize,

    /// Overall health score (0-100)
    pub health_score: f64,
}

/// Merge statistics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeStatisticsSummary {
    /// Total merges performed
    pub total_merges: u64,

    /// Merges in last 24 hours
    pub merges_last_24h: u64,

    /// Average memories per merge
    pub avg_memories_per_merge: f64,

    /// Consolidation rate (merges per day)
    pub consolidation_rate_per_day: f64,
}

/// Metacognition service
///
//! Monitors memory health and provides insights.
pub struct MetacognitionService {
    config: MetacognitionConfig,
    merge_tracker: MergeTracker,
    stats: Arc<RwLock<MetacognitionStats>>,
    
    /// Memory count callback
    #[allow(clippy::type_complexity)]
    memory_count_callback: Arc<RwLock<Option<Box<dyn Fn() -> usize + Send + Sync>>>>,
    
    /// Memory access callback
    #[allow(clippy::type_complexity)]
    memory_access_callback: Arc<RwLock<Option<Box<dyn Fn() -> MemoryAccessData + Send + Sync>>>>,
}

/// Memory access data
#[derive(Debug, Clone)]
pub struct MemoryAccessData {
    pub total_accesses: u64,
    pub avg_accesses: f64,
    pub most_accessed_id: Option<String>,
    pub distribution: AccessDistribution,
}

/// Internal metacognition statistics
#[derive(Debug, Clone)]
struct MetacognitionStats {
    total_reports_generated: u64,
    last_report_at: Option<DateTime<Utc>>,
    historical_scores: Vec<(DateTime<Utc>, f64)>,
}

impl MetacognitionService {
    /// Create new metacognition service
    pub async fn new() -> Result<Self> {
        Self::with_config(MetacognitionConfig::default()).await
    }

    /// Create with custom configuration
    pub async fn with_config(config: MetacognitionConfig) -> Result<Self> {
        let merge_tracker = MergeTracker::new();

        Ok(Self {
            config,
            merge_tracker,
            stats: Arc::new(RwLock::new(MetacognitionStats {
                total_reports_generated: 0,
                last_report_at: None,
                historical_scores: Vec::new(),
            })),
            memory_count_callback: Arc::new(RwLock::new(None)),
            memory_access_callback: Arc::new(RwLock::new(None)),
        })
    }

    /// Set memory count callback
    pub async fn set_memory_count_callback<F>(&self, callback: F)
    where
        F: Fn() -> usize + Send + Sync + 'static,
    {
        let mut cb = self.memory_count_callback.write().await;
        *cb = Some(Box::new(callback));
    }

    /// Set memory access callback
    pub async fn set_memory_access_callback<F>(&self, callback: F)
    where
        F: Fn() -> MemoryAccessData + Send + Sync + 'static,
    {
        let mut cb = self.memory_access_callback.write().await;
        *cb = Some(Box::new(callback));
    }

    /// Generate metacognitive report
    pub async fn generate_report(&self) -> Result<MetacognitionReport> {
        info!("Generating metacognitive report");

        let total_memories = {
            let cb = self.memory_count_callback.read().await;
            cb.as_ref().map(|f| f()).unwrap_or(0)
        };

        // Get merge statistics
        let merge_stats = self.merge_tracker.get_statistics().await;
        
        // Calculate health metrics
        let health = self.calculate_health_metrics(total_memories, &merge_stats).await;
        
        // Get usage statistics
        let usage = self.get_usage_statistics().await;
        
        // Calculate performance metrics (placeholder)
        let performance = PerformanceMetrics {
            avg_retrieval_time_ms: 50.0,
            avg_consolidation_time_ms: 200.0,
            cache_hit_rate: 0.85,
            throughput_ops_per_sec: 1000.0,
        };

        let merge_summary = MergeStatisticsSummary {
            total_merges: merge_stats.total_merges,
            merges_last_24h: 0, // TODO: implement time-based filtering
            avg_memories_per_merge: merge_stats.avg_secondaries_per_merge,
            consolidation_rate_per_day: 5.0, // TODO: calculate from history
        };

        let health_score = health.health_score;

        let report = MetacognitionReport {
            generated_at: Utc::now(),
            health,
            usage,
            performance,
            merge_stats: merge_summary,
            recommendations_count: 0,
            health_score,
        };

        // Update stats
        let mut stats = self.stats.write().await;
        stats.total_reports_generated += 1;
        stats.last_report_at = Some(Utc::now());
        stats.historical_scores.push((Utc::now(), health_score));

        // Trim historical scores
        let max_scores = (self.config.statistics_retention_days * 24) as usize;
        if stats.historical_scores.len() > max_scores {
            stats.historical_scores.drain(0..stats.historical_scores.len() - max_scores);
        }

        info!("Report generated: health score {:.1}", health_score);
        Ok(report)
    }

    /// Calculate health metrics
    async fn calculate_health_metrics(
        &self,
        total_memories: usize,
        merge_stats: &MergeStatistics,
    ) -> MemoryHealthMetrics {
        // Calculate health score based on multiple factors
        let active_ratio = 0.7; // Placeholder
        let dormant_ratio = 0.2; // Placeholder
        let fragmentation_score = 0.1; // Placeholder

        let health_score = (active_ratio * 60.0 + (1.0 - dormant_ratio) * 30.0 + (1.0 - fragmentation_score) * 10.0).min(100.0);

        let consolidation_urgency = if total_memories > 500 {
            90.0
        } else if total_memories > 200 {
            60.0
        } else if total_memories > 100 {
            30.0
        } else {
            0.0
        };

        MemoryHealthMetrics {
            total_memories,
            active_memories: (total_memories as f64 * active_ratio) as usize,
            dormant_memories: (total_memories as f64 * dormant_ratio) as usize,
            fragmented_memories: (total_memories as f64 * fragmentation_score) as usize,
            health_score,
            consolidation_urgency,
        }
    }

    /// Get usage statistics
    async fn get_usage_statistics(&self) -> MemoryUsageStats {
        let access_data = {
            let cb = self.memory_access_callback.read().await;
            cb.as_ref().map(|f| f()).unwrap_or(MemoryAccessData {
                total_accesses: 0,
                avg_accesses: 0.0,
                most_accessed_id: None,
                distribution: AccessDistribution {
                    q1: 0,
                    q2: 0,
                    q3: 0,
                    max: 0,
                },
            })
        };

        MemoryUsageStats {
            total_accesses: access_data.total_accesses,
            avg_accesses_per_memory: access_data.avg_accesses,
            most_accessed_memory_id: access_data.most_accessed_id,
            access_distribution: access_data.distribution,
        }
    }
}

// Re-export MergeStatistics from history module
pub use crate::history::MergeStatistics;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_creation() {
        let service = MetacognitionService::new().await;
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_generate_report() {
        let service = MetacognitionService::new().await.unwrap();
        
        // Set callbacks
        service.set_memory_count_callback(|| 100).await;
        service.set_memory_access_callback(|| MemoryAccessData {
            total_accesses: 1000,
            avg_accesses: 10.0,
            most_accessed_id: Some("mem-1".to_string()),
            distribution: AccessDistribution {
                q1: 5,
                q2: 10,
                q3: 15,
                max: 50,
            },
        }).await;

        let report = service.generate_report().await.unwrap();
        assert!(report.health_score >= 0.0 && report.health_score <= 100.0);
        assert_eq!(report.health.total_memories, 100);
    }
}
