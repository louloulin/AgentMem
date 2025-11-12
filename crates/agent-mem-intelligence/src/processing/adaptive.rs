//! Adaptive memory management algorithms
//!
//! Implements intelligent memory lifecycle management including
//! archiving, deletion, and capacity management.

use agent_mem_traits::{MemoryV4 as Memory, Result, AttributeKey, AttributeValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Memory lifecycle action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LifecycleAction {
    /// Keep memory active
    Keep,
    /// Archive memory (reduce access priority)
    Archive,
    /// Delete memory permanently
    Delete,
    /// Compress memory (reduce content size)
    Compress,
}

/// Adaptive management strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdaptiveStrategy {
    /// Least Recently Used (LRU)
    LRU,
    /// Least Frequently Used (LFU)
    LFU,
    /// Importance-based management
    ImportanceBased,
    /// Hybrid approach combining multiple factors
    Hybrid,
}

/// Memory management thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagementThresholds {
    /// Minimum importance score to keep memory
    pub min_importance: f32,

    /// Age threshold for archiving (in seconds)
    pub archive_age_threshold: i64,

    /// Age threshold for deletion (in seconds)
    pub delete_age_threshold: i64,

    /// Minimum access count to prevent deletion
    pub min_access_count: u32,

    /// Maximum memory size before compression (in bytes)
    pub max_memory_size: usize,
}

impl Default for ManagementThresholds {
    fn default() -> Self {
        Self {
            min_importance: 0.1,
            archive_age_threshold: 7 * 24 * 60 * 60, // 7 days
            delete_age_threshold: 30 * 24 * 60 * 60, // 30 days
            min_access_count: 1,
            max_memory_size: 10000, // 10KB
        }
    }
}

/// Adaptive memory manager
pub struct AdaptiveMemoryManager {
    /// Maximum memories per scope
    max_memories: usize,

    /// Retention period in seconds
    retention_period: i64,

    /// Management strategy
    strategy: AdaptiveStrategy,

    /// Management thresholds
    thresholds: ManagementThresholds,

    /// Statistics tracking
    stats: HashMap<String, u64>,
}

impl AdaptiveMemoryManager {
    /// Create a new adaptive memory manager
    pub fn new(max_memories: usize, retention_period: i64) -> Self {
        Self {
            max_memories,
            retention_period,
            strategy: AdaptiveStrategy::Hybrid,
            thresholds: ManagementThresholds::default(),
            stats: HashMap::new(),
        }
    }

    /// Set management strategy
    pub fn with_strategy(mut self, strategy: AdaptiveStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Set management thresholds
    pub fn with_thresholds(mut self, thresholds: ManagementThresholds) -> Self {
        self.thresholds = thresholds;
        self
    }

    /// Update memory limits
    pub fn update_limits(&mut self, max_memories: usize, retention_period: i64) {
        self.max_memories = max_memories;
        self.retention_period = retention_period;
    }

    /// Manage a batch of memories
    pub async fn manage_memories(&mut self, memories: &mut Vec<Memory>) -> Result<(usize, usize)> {
        info!(
            "Managing {} memories with strategy {:?}",
            memories.len(),
            self.strategy
        );

        let current_time = chrono::Utc::now().timestamp();
        let mut archived_count = 0;
        let mut deleted_count = 0;

        // Step 1: Apply lifecycle actions based on strategy
        let actions = self
            .determine_lifecycle_actions(memories, current_time)
            .await?;

        // Step 2: Execute actions
        for (i, action) in actions.iter().enumerate() {
            match action {
                LifecycleAction::Keep => {
                    // No action needed
                }
                LifecycleAction::Archive => {
                    self.archive_memory(&mut memories[i]).await?;
                    archived_count += 1;
                }
                LifecycleAction::Delete => {
                    memories[i].content = agent_mem_traits::Content::Text(String::new()); // Mark for deletion
                    deleted_count += 1;
                }
                LifecycleAction::Compress => {
                    self.compress_memory(&mut memories[i]).await?;
                }
            }
        }

        // Step 3: Handle capacity constraints
        if memories.len() > self.max_memories {
            let excess_count = memories.len() - self.max_memories;
            let additional_deleted = self
                .handle_capacity_overflow(memories, excess_count)
                .await?;
            deleted_count += additional_deleted;
        }

        // Update statistics
        self.update_stats(archived_count, deleted_count);

        info!(
            "Memory management completed: {} archived, {} deleted",
            archived_count, deleted_count
        );
        Ok((archived_count, deleted_count))
    }

    /// Evaluate a single memory for lifecycle actions
    pub async fn evaluate_single_memory(&mut self, memory: &mut Memory) -> Result<LifecycleAction> {
        let current_time = chrono::Utc::now().timestamp();
        let action = self
            .determine_single_lifecycle_action(memory, current_time)
            .await?;

        match action {
            LifecycleAction::Archive => {
                self.archive_memory(memory).await?;
            }
            LifecycleAction::Compress => {
                self.compress_memory(memory).await?;
            }
            _ => {}
        }

        Ok(action)
    }

    /// Get access count from memory metadata
    fn get_access_count(&self, memory: &Memory) -> u32 {
        memory.metadata.access_count
    }

    /// Get last accessed timestamp from memory metadata
    fn get_last_accessed(&self, memory: &Memory) -> i64 {
        memory.metadata.accessed_at.timestamp()
    }

    /// Determine lifecycle actions for all memories
    async fn determine_lifecycle_actions(
        &self,
        memories: &[Memory],
        current_time: i64,
    ) -> Result<Vec<LifecycleAction>> {
        let mut actions = Vec::with_capacity(memories.len());

        for memory in memories {
            let action = self
                .determine_single_lifecycle_action(memory, current_time)
                .await?;
            actions.push(action);
        }

        Ok(actions)
    }

    /// Determine lifecycle action for a single memory
    async fn determine_single_lifecycle_action(
        &self,
        memory: &Memory,
        current_time: i64,
    ) -> Result<LifecycleAction> {
        let age = current_time - memory.metadata.created_at.timestamp();
        let time_since_access =
            current_time - memory.metadata.updated_at.timestamp();

        // Check for deletion conditions
        if age > self.thresholds.delete_age_threshold
            || (memory.score().unwrap_or(0.5) < self.thresholds.min_importance as f64
                && self.get_access_count(memory) < self.thresholds.min_access_count)
        {
            return Ok(LifecycleAction::Delete);
        }

        // Check for archiving conditions
        if age > self.thresholds.archive_age_threshold
            || time_since_access > self.thresholds.archive_age_threshold / 2
        {
            return Ok(LifecycleAction::Archive);
        }

        // Check for compression conditions
        let content_len = match &memory.content {
            agent_mem_traits::Content::Text(t) => t.len(),
            agent_mem_traits::Content::Structured(v) => v.to_string().len(),
            _ => 0,
        };
        if content_len > self.thresholds.max_memory_size {
            return Ok(LifecycleAction::Compress);
        }

        // Apply strategy-specific logic
        match self.strategy {
            AdaptiveStrategy::LRU => {
                if time_since_access > self.retention_period / 4 {
                    Ok(LifecycleAction::Archive)
                } else {
                    Ok(LifecycleAction::Keep)
                }
            }
            AdaptiveStrategy::LFU => {
                if self.get_access_count(memory) < 2 && age > self.retention_period / 7 {
                    Ok(LifecycleAction::Archive)
                } else {
                    Ok(LifecycleAction::Keep)
                }
            }
            AdaptiveStrategy::ImportanceBased => {
                if memory.score().unwrap_or(0.5) < 0.3 {
                    Ok(LifecycleAction::Archive)
                } else {
                    Ok(LifecycleAction::Keep)
                }
            }
            AdaptiveStrategy::Hybrid => {
                // Combine multiple factors
                let importance_factor = memory.score().unwrap_or(0.5) as f32;
                let recency_factor =
                    1.0 - (time_since_access as f32 / self.retention_period as f32).min(1.0);
                let frequency_factor = (self.get_access_count(memory) as f32 / 10.0).min(1.0);

                let combined_score =
                    importance_factor * 0.5 + recency_factor * 0.3 + frequency_factor * 0.2;

                if combined_score < 0.3 {
                    Ok(LifecycleAction::Archive)
                } else {
                    Ok(LifecycleAction::Keep)
                }
            }
        }
    }

    /// Archive a memory
    async fn archive_memory(&self, memory: &mut Memory) -> Result<()> {
        // Mark as archived in attributes
        memory.attributes.insert(
            AttributeKey::system("archived"),
            AttributeValue::Boolean(true),
        );
        memory.attributes.insert(
            AttributeKey::system("archived_at"),
            AttributeValue::Number(chrono::Utc::now().timestamp() as f64),
        );

        // Reduce importance slightly for archived memories
        if let Some(importance) = memory.importance() {
            memory.set_importance(importance * 0.8);
        }
        if let Some(score) = memory.score() {
            memory.set_score(score * 0.8);
        }
        memory.metadata.updated_at = chrono::Utc::now();

        debug!("Archived memory: {}", memory.id);
        Ok(())
    }

    /// Compress a memory by reducing content size
    async fn compress_memory(&self, memory: &mut Memory) -> Result<()> {
        let content_text = memory.content.as_text().unwrap_or("");
        let original_size = content_text.len();

        // Simple compression: keep first and last parts, summarize middle
        if original_size > self.thresholds.max_memory_size {
            let keep_size = self.thresholds.max_memory_size / 3;
            let start_part = &content_text[..keep_size.min(original_size)];
            let end_part = if original_size > keep_size * 2 {
                &content_text[original_size - keep_size..]
            } else {
                ""
            };

            let compressed_content = if !end_part.is_empty() {
                format!(
                    "{}... [compressed {} chars] ...{}",
                    start_part,
                    original_size - keep_size * 2,
                    end_part
                )
            } else {
                start_part.to_string()
            };

            memory.content = agent_mem_traits::Content::Text(compressed_content);
            memory.attributes.insert(
                AttributeKey::system("compressed"),
                AttributeValue::Boolean(true),
            );
            memory.attributes.insert(
                AttributeKey::system("original_size"),
                AttributeValue::Number(original_size as f64),
            );
            memory.metadata.updated_at = chrono::Utc::now();

            let new_size = match &memory.content {
                agent_mem_traits::Content::Text(t) => t.len(),
                agent_mem_traits::Content::Structured(v) => v.to_string().len(),
                _ => 0,
            };
            debug!(
                "Compressed memory {} from {} to {} bytes",
                memory.id,
                original_size,
                new_size
            );
        }

        Ok(())
    }

    /// Handle capacity overflow by removing least important memories
    async fn handle_capacity_overflow(
        &self,
        memories: &mut [Memory],
        excess_count: usize,
    ) -> Result<usize> {
        if excess_count == 0 {
            return Ok(0);
        }

        warn!(
            "Memory capacity exceeded, removing {} memories",
            excess_count
        );

        // Create indices with scores for sorting
        let mut memory_scores: Vec<(usize, f32)> = memories
            .iter()
            .enumerate()
            .map(|(i, memory)| {
                let score = match self.strategy {
                    AdaptiveStrategy::LRU => {
                        let current_time = chrono::Utc::now().timestamp();
                        -(current_time - self.get_last_accessed(memory)) as f32
                    }
                    AdaptiveStrategy::LFU => -(self.get_access_count(memory) as f32),
                    AdaptiveStrategy::ImportanceBased => -memory.score().unwrap_or(0.5) as f32,
                    AdaptiveStrategy::Hybrid => {
                        let current_time = chrono::Utc::now().timestamp();
                        let recency =
                            -(current_time - self.get_last_accessed(memory)) as f32 / 86400.0; // Days
                        let frequency = -(self.get_access_count(memory) as f32);
                        let importance = -memory.score().unwrap_or(0.5) as f32;
                        recency * 0.3 + frequency * 0.3 + importance * 0.4
                    }
                };
                (i, score)
            })
            .collect();

        // Sort by score (lowest first for removal)
        memory_scores.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        // Mark worst memories for deletion
        let mut deleted_count = 0;
        for (i, _) in memory_scores.iter().take(excess_count) {
            if memories[*i].content.as_text().map(|s| !s.is_empty()).unwrap_or(true) {
                // Don't double-delete
                memories[*i].content = agent_mem_traits::Content::Text(String::new()); // Mark for deletion
                deleted_count += 1;
            }
        }

        Ok(deleted_count)
    }

    /// Update internal statistics
    fn update_stats(&mut self, archived: usize, deleted: usize) {
        *self.stats.entry("total_archived".to_string()).or_insert(0) += archived as u64;
        *self.stats.entry("total_deleted".to_string()).or_insert(0) += deleted as u64;
        *self.stats.entry("management_runs".to_string()).or_insert(0) += 1;
    }

    /// Get management statistics
    pub fn get_stats(&self) -> &HashMap<String, u64> {
        &self.stats
    }

    /// Clean up memories marked for deletion
    pub fn cleanup_deleted_memories(&self, memories: &mut Vec<Memory>) {
        memories.retain(|memory| {
            memory.content.as_text().map(|s| !s.is_empty()).unwrap_or(true)
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_mem_traits::MemoryV4 as Memory;
    use chrono::Utc;
    use std::collections::HashMap;

    fn create_test_memory(id: &str, importance: f32, access_count: u32, age_days: i64) -> Memory {
        use agent_mem_traits::{MemoryId, Content, AttributeSet, AttributeKey, AttributeValue, RelationGraph, MetadataV4 as Metadata};
        let current_time = Utc::now();
        let age_duration = chrono::Duration::days(age_days);
        let created_at = current_time - age_duration;
        let mut memory = Memory {
            id: MemoryId::from_string(id.to_string()),
            content: Content::Text(format!("Test memory content for {}", id)),
            attributes: AttributeSet::new(),
            relations: RelationGraph::new(),
            metadata: Metadata {
                created_at,
                updated_at: created_at,
                accessed_at: current_time - age_duration / 2,
                access_count,
                version: 1,
                hash: None,
            },
        };
        memory.set_agent_id("test_agent");
        memory.set_user_id("test_user");
        memory.set_importance(importance as f64);
        memory.set_score(importance as f64);
        memory
    }

    #[tokio::test]
    async fn test_adaptive_manager_creation() {
        let manager = AdaptiveMemoryManager::new(100, 30 * 24 * 60 * 60);
        assert_eq!(manager.max_memories, 100);
    }

    #[tokio::test]
    async fn test_lifecycle_action_determination() {
        let manager = AdaptiveMemoryManager::new(100, 30 * 24 * 60 * 60);
        let current_time = Utc::now().timestamp();

        // Old, low importance memory should be deleted
        let old_memory = create_test_memory("old", 0.05, 1, 35);
        let action = manager
            .determine_single_lifecycle_action(&old_memory, current_time)
            .await
            .unwrap();
        assert!(matches!(action, LifecycleAction::Delete));

        // Recent, important memory should be kept
        let important_memory = create_test_memory("important", 0.9, 10, 1);
        let action = manager
            .determine_single_lifecycle_action(&important_memory, current_time)
            .await
            .unwrap();
        assert!(matches!(action, LifecycleAction::Keep));
    }

    #[tokio::test]
    async fn test_memory_archiving() {
        let manager = AdaptiveMemoryManager::new(100, 30 * 24 * 60 * 60);
        let mut memory = create_test_memory("test", 0.5, 5, 10);

        manager.archive_memory(&mut memory).await.unwrap();

        // Check if archived attribute is set to true
        if let Some(AttributeValue::Boolean(archived)) = memory.attributes.get(&AttributeKey::system("archived")) {
            assert!(*archived);
        } else {
            panic!("Expected archived attribute to be Boolean(true)");
        }
        assert!(memory.importance().unwrap_or(0.0) < 0.5); // Should be reduced
    }

    #[tokio::test]
    async fn test_memory_compression() {
        let manager = AdaptiveMemoryManager::new(100, 30 * 24 * 60 * 60);
        let mut memory = create_test_memory("test", 0.5, 5, 1);
        memory.content = agent_mem_traits::Content::Text("A".repeat(15000)); // Large content

        let original_size = memory.content.as_text().unwrap_or("").len();
        manager.compress_memory(&mut memory).await.unwrap();

        assert!(memory.content.as_text().unwrap_or("").len() < original_size);
        // Check if compressed attribute is set to true
        if let Some(AttributeValue::Boolean(compressed)) = memory.attributes.get(&AttributeKey::system("compressed")) {
            assert!(*compressed);
        } else {
            panic!("Expected compressed attribute to be Boolean(true)");
        }
    }

    #[tokio::test]
    async fn test_capacity_management() {
        let mut manager = AdaptiveMemoryManager::new(3, 30 * 24 * 60 * 60); // Max 3 memories

        let mut memories = vec![
            create_test_memory("1", 0.9, 10, 1), // High importance
            create_test_memory("2", 0.5, 5, 5),  // Medium importance
            create_test_memory("3", 0.2, 2, 10), // Low importance
            create_test_memory("4", 0.1, 1, 15), // Very low importance
            create_test_memory("5", 0.8, 8, 2),  // High importance
        ];

        let (archived, deleted) = manager.manage_memories(&mut memories).await.unwrap();

        // Should have deleted some memories due to capacity constraints
        assert!(deleted > 0);

        // Clean up and verify capacity is respected
        manager.cleanup_deleted_memories(&mut memories);
        assert!(memories.len() <= 3);
    }
}
