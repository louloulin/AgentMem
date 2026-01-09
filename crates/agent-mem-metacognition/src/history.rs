//! Merge History Tracking
//!
//! Tracks all memory merge operations with full audit trail.
//!
//! # Theory
//!
//! Every merge operation should be tracked for:
//! - Audit purposes: understand what happened and why
//! - Rollback: ability to undo merges if needed
//! - Analytics: understand merge patterns and optimize
//! - Debugging: investigate issues with merged memories
//!
//! # Example
//!
//! ```no_run
//! use agent_mem_metacognition::history::MergeTracker;
//! use agent_mem_metacognition::MergeOperation;
//!
//! let tracker = MergeTracker::new();
//!
//! // Record a merge operation
//! let operation = MergeOperation {
//!     primary_id: "mem-1".to_string(),
//!     secondary_ids: vec!["mem-2".to_string(), "mem-3".to_string()],
//!     reason: "Similar content detected".to_string(),
//!     strategy: "intelligent_merge".to_string(),
//!     ..Default::default()
//! };
//!
//! tracker.record_merge(operation).await;
//!
//! // Get history for a memory
//! let history = tracker.get_history("mem-1").await;
//! ```

use crate::DEFAULT_CONSOLIDATION_THRESHOLD;
use agent_mem_traits::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Single merge operation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeOperation {
    /// Primary memory ID (the one that remains after merge)
    pub primary_id: String,

    /// Secondary memory IDs (the ones that were merged into primary)
    pub secondary_ids: Vec<String>,

    /// Reason for the merge
    pub reason: String,

    /// Merge strategy used
    pub strategy: String,

    /// Timestamp when merge occurred
    pub timestamp: DateTime<Utc>,

    /// Similarity scores for each secondary memory
    pub similarity_scores: Vec<f32>,

    /// User who initiated the merge (empty if automatic)
    pub user_id: Option<String>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl Default for MergeOperation {
    fn default() -> Self {
        Self {
            primary_id: String::new(),
            secondary_ids: Vec::new(),
            reason: String::new(),
            strategy: String::new(),
            timestamp: Utc::now(),
            similarity_scores: Vec::new(),
            user_id: None,
            metadata: HashMap::new(),
        }
    }
}

/// Merge history for a specific memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeHistory {
    /// Memory ID
    pub memory_id: String,

    /// All merge operations involving this memory
    pub operations: Vec<MergeOperation>,

    /// Total number of times this memory was merged
    pub merge_count: usize,

    /// Last merge timestamp
    pub last_merged_at: Option<DateTime<Utc>>,

    /// All IDs that have ever been merged into this memory
    pub absorbed_ids: Vec<String>,
}

impl MergeHistory {
    /// Create empty merge history
    pub fn new(memory_id: String) -> Self {
        Self {
            memory_id,
            operations: Vec::new(),
            merge_count: 0,
            last_merged_at: None,
            absorbed_ids: Vec::new(),
        }
    }

    /// Add a merge operation to history
    pub fn add_operation(&mut self, operation: MergeOperation) {
        self.merge_count += 1;
        self.last_merged_at = Some(operation.timestamp);
        self.absorbed_ids
            .extend(operation.secondary_ids.iter().cloned());
        self.operations.push(operation);
    }

    /// Check if memory was created from a merge
    pub fn is_merged(&self) -> bool {
        self.merge_count > 0
    }

    /// Get all original IDs that make up this memory
    pub fn get_all_component_ids(&self) -> Vec<String> {
        let mut ids = vec![self.memory_id.clone()];
        ids.extend(self.absorbed_ids.iter().cloned());
        ids
    }
}

/// Merge history tracker
///
/// Tracks all merge operations across the system.
pub struct MergeTracker {
    /// Memory ID -> Merge history
    histories: Arc<RwLock<HashMap<String, MergeHistory>>>,

    /// All merge operations (chronological log)
    all_operations: Arc<RwLock<Vec<MergeOperation>>>,

    /// Maximum history size per memory
    max_history_size: usize,

    /// Maximum operations in global log
    max_global_operations: usize,

    /// Total merges tracked
    total_merges: Arc<RwLock<u64>>,
}

impl MergeTracker {
    /// Create new merge tracker
    pub fn new() -> Self {
        Self {
            histories: Arc::new(RwLock::new(HashMap::new())),
            all_operations: Arc::new(RwLock::new(Vec::new())),
            max_history_size: 1000,
            max_global_operations: 10000,
            total_merges: Arc::new(RwLock::new(0)),
        }
    }

    /// Set maximum history size per memory
    pub fn with_max_history_size(mut self, size: usize) -> Self {
        self.max_history_size = size;
        self
    }

    /// Set maximum global operations
    pub fn with_max_global_operations(mut self, max: usize) -> Self {
        self.max_global_operations = max;
        self
    }

    /// Record a merge operation
    ///
    /// # Parameters
    ///
    /// - `operation`: The merge operation to record
    pub async fn record_merge(&self, operation: MergeOperation) -> Result<()> {
        debug!(
            "Recording merge: {} <- {:?}",
            operation.primary_id, operation.secondary_ids
        );

        let mut all_ops = self.all_operations.write().await;
        let mut total = self.total_merges.write().await;

        // Add to global log
        all_ops.push(operation.clone());
        *total += 1;

        // Trim global log if needed
        if all_ops.len() > self.max_global_operations {
            let remove_count = all_ops.len() - self.max_global_operations;
            all_ops.drain(0..remove_count);
        }

        drop(all_ops);
        drop(total);

        // Update history for primary memory
        let mut histories = self.histories.write().await;
        let primary_history = histories
            .entry(operation.primary_id.clone())
            .or_insert_with(|| MergeHistory::new(operation.primary_id.clone()));
        primary_history.add_operation(operation.clone());

        // Update history for secondary memories
        for secondary_id in &operation.secondary_ids {
            let secondary_history = histories
                .entry(secondary_id.clone())
                .or_insert_with(|| MergeHistory::new(secondary_id.clone()));

            // Create reverse operation (secondary -> primary)
            let mut reverse_op = operation.clone();
            reverse_op.primary_id = secondary_id.clone();
            reverse_op.secondary_ids = vec![operation.primary_id.clone()];
            reverse_op.metadata.insert(
                "reverse_merge".to_string(),
                "true".to_string()
            );

            secondary_history.add_operation(reverse_op);
        }

        info!("Merge recorded successfully");
        Ok(())
    }

    /// Get merge history for a specific memory
    ///
    /// # Parameters
    ///
    /// - `memory_id`: Memory ID to get history for
    ///
    /// # Returns
    ///
    /// Merge history, or None if memory has no history
    pub async fn get_history(&self, memory_id: &str) -> Option<MergeHistory> {
        let histories = self.histories.read().await;
        histories.get(memory_id).cloned()
    }

    /// Get all merge operations
    pub async fn get_all_operations(&self) -> Vec<MergeOperation> {
        let all_ops = self.all_operations.read().await;
        all_ops.clone()
    }

    /// Get recent merge operations
    ///
    /// # Parameters
    ///
    /// - `count`: Number of recent operations to return
    pub async fn get_recent_operations(&self, count: usize) -> Vec<MergeOperation> {
        let all_ops = self.all_operations.read().await;
        let start = if all_ops.len() > count {
            all_ops.len() - count
        } else {
            0
        };
        all_ops[start..].to_vec()
    }

    /// Get total number of merges tracked
    pub async fn total_merges(&self) -> u64 {
        *this.total_merges.read().await
    }

    /// Get merge statistics
    pub async fn get_statistics(&self) -> MergeStatistics {
        let all_ops = self.all_operations.read().await;
        let histories = self.histories.read().await;

        let total_merges = all_ops.len() as u64;
        let unique_memories_merged = histories.len() as u64;

        // Calculate average secondary memories per merge
        let avg_secondaries: f64 = if all_ops.is_empty() {
            0.0
        } else {
            let total_secondaries: usize = all_ops.iter().map(|op| op.secondary_ids.len()).sum();
            total_secondaries as f64 / all_ops.len() as f64
        };

        // Count by strategy
        let mut strategy_counts: HashMap<String, u64> = HashMap::new();
        for op in all_ops.iter() {
            *strategy_counts.entry(op.strategy.clone()).or_insert(0) += 1;
        }

        MergeStatistics {
            total_merges,
            unique_memories_merged,
            avg_secondaries_per_merge: avg_secondaries,
            strategy_counts,
        }
    }

    /// Clear all history
    pub async fn clear_all(&self) -> Result<()> {
        let mut histories = self.histories.write().await;
        let mut all_ops = self.all_operations.write().await;
        let mut total = self.total_merges.write().await;

        histories.clear();
        all_ops.clear();
        *total = 0;

        info!("All merge history cleared");
        Ok(())
    }

    /// Clear history for a specific memory
    pub async fn clear_memory_history(&self, memory_id: &str) -> Result<()> {
        let mut histories = self.histories.write().await;
        histories.remove(memory_id);

        debug!("History cleared for memory: {}", memory_id);
        Ok(())
    }
}

impl Clone for MergeTracker {
    fn clone(&self) -> Self {
        Self {
            histories: Arc::clone(&this.histories),
            all_operations: Arc::clone(&self.all_operations),
            max_history_size: self.max_history_size,
            max_global_operations: self.max_global_operations,
            total_merges: Arc::clone(&self.total_merges),
        }
    }
}

impl Default for MergeTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Merge statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeStatistics {
    /// Total number of merge operations
    pub total_merges: u64,

    /// Number of unique memories involved in merges
    pub unique_memories_merged: u64,

    /// Average number of secondary memories per merge
    pub avg_secondaries_per_merge: f64,

    /// Count of merges by strategy
    pub strategy_counts: HashMap<String, u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_merge_operation_creation() {
        let operation = MergeOperation {
            primary_id: "mem-1".to_string(),
            secondary_ids: vec!["mem-2".to_string()],
            reason: "Test merge".to_string(),
            strategy: "intelligent".to_string(),
            ..Default::default()
        };

        assert_eq!(operation.primary_id, "mem-1");
        assert_eq!(operation.secondary_ids.len(), 1);
    }

    #[tokio::test]
    async fn test_merge_history() {
        let mut history = MergeHistory::new("mem-1".to_string());

        assert!(!history.is_merged());
        assert_eq!(history.merge_count, 0);

        let operation = MergeOperation {
            primary_id: "mem-1".to_string(),
            secondary_ids: vec!["mem-2".to_string()],
            reason: "Test".to_string(),
            strategy: "test".to_string(),
            ..Default::default()
        };

        history.add_operation(operation);

        assert!(history.is_merged());
        assert_eq!(history.merge_count, 1);
        assert!(history.last_merged_at.is_some());
    }

    #[tokio::test]
    async fn test_merge_tracker() {
        let tracker = MergeTracker::new();

        let operation = MergeOperation {
            primary_id: "mem-1".to_string(),
            secondary_ids: vec!["mem-2".to_string(), "mem-3".to_string()],
            reason: "Similar content".to_string(),
            strategy: "merge".to_string(),
            ..Default::default()
        };

        tracker.record_merge(operation).await.unwrap();

        // Check primary history
        let primary_history = tracker.get_history("mem-1").await;
        assert!(primary_history.is_some());
        assert_eq!(primary_history.unwrap().merge_count, 1);

        // Check secondary history
        let secondary_history = tracker.get_history("mem-2").await;
        assert!(secondary_history.is_some());
    }

    #[tokio::test]
    async fn test_total_merges() {
        let tracker = MergeTracker::new();

        assert_eq!(tracker.total_merges().await, 0);

        let operation = MergeOperation {
            primary_id: "mem-1".to_string(),
            secondary_ids: vec!["mem-2".to_string()],
            reason: "Test".to_string(),
            strategy: "test".to_string(),
            ..Default::default()
        };

        tracker.record_merge(operation).await.unwrap();
        assert_eq!(tracker.total_merges().await, 1);
    }

    #[tokio::test]
    async fn test_get_all_operations() {
        let tracker = MergeTracker::new();

        let op1 = MergeOperation {
            primary_id: "mem-1".to_string(),
            secondary_ids: vec!["mem-2".to_string()],
            reason: "Test1".to_string(),
            strategy: "test".to_string(),
            ..Default::default()
        };

        let op2 = MergeOperation {
            primary_id: "mem-3".to_string(),
            secondary_ids: vec!["mem-4".to_string()],
            reason: "Test2".to_string(),
            strategy: "test".to_string(),
            ..Default::default()
        };

        tracker.record_merge(op1).await.unwrap();
        tracker.record_merge(op2).await.unwrap();

        let all_ops = tracker.get_all_operations().await;
        assert_eq!(all_ops.len(), 2);
    }

    #[tokio::test]
    async fn test_clear_memory_history() {
        let tracker = MergeTracker::new();

        let operation = MergeOperation {
            primary_id: "mem-1".to_string(),
            secondary_ids: vec!["mem-2".to_string()],
            reason: "Test".to_string(),
            strategy: "test".to_string(),
            ..Default::default()
        };

        tracker.record_merge(operation).await.unwrap();

        assert!(tracker.get_history("mem-1").await.is_some());

        tracker.clear_memory_history("mem-1").await.unwrap();
        assert!(tracker.get_history("mem-1").await.is_none());
    }

    #[tokio::test]
    async fn test_merge_statistics() {
        let tracker = MergeTracker::new();

        for i in 0..5 {
            let operation = MergeOperation {
                primary_id: format!("mem-{}", i),
                secondary_ids: vec![format!("mem-{}", i + 10)],
                reason: "Test".to_string(),
                strategy: "test".to_string(),
                ..Default::default()
            };
            tracker.record_merge(operation).await.unwrap();
        }

        let stats = tracker.get_statistics().await;
        assert_eq!(stats.total_merges, 5);
        assert_eq!(stats.unique_memories_merged, 10);
    }

    #[tokio::test]
    async fn test_get_all_component_ids() {
        let mut history = MergeHistory::new("mem-1".to_string());

        let operation = MergeOperation {
            primary_id: "mem-1".to_string(),
            secondary_ids: vec!["mem-2".to_string(), "mem-3".to_string()],
            reason: "Test".to_string(),
            strategy: "test".to_string(),
            ..Default::default()
        };

        history.add_operation(operation);

        let component_ids = history.get_all_component_ids();
        assert_eq!(component_ids.len(), 3);
        assert!(component_ids.contains(&"mem-1".to_string()));
        assert!(component_ids.contains(&"mem-2".to_string()));
        assert!(component_ids.contains(&"mem-3".to_string()));
    }
}
