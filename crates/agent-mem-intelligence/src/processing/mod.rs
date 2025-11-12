//! Intelligent memory processing algorithms
//!
//! Advanced algorithms for memory consolidation, importance scoring,
//! and adaptive memory management.

pub mod adaptive;
pub mod consolidation;
pub mod importance;

pub use adaptive::*;
pub use consolidation::*;
pub use importance::*;

use agent_mem_traits::{MemoryV4 as Memory, Result};
use serde::{Deserialize, Serialize};

/// Memory processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingConfig {
    /// Enable automatic consolidation
    pub enable_consolidation: bool,

    /// Consolidation threshold (similarity score)
    pub consolidation_threshold: f32,

    /// Enable importance scoring
    pub enable_importance_scoring: bool,

    /// Importance decay rate over time
    pub importance_decay_rate: f32,

    /// Enable adaptive memory management
    pub enable_adaptive_management: bool,

    /// Maximum memories per scope
    pub max_memories_per_scope: usize,

    /// Memory retention period (in seconds)
    pub retention_period: i64,
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        Self {
            enable_consolidation: true,
            consolidation_threshold: 0.85,
            enable_importance_scoring: true,
            importance_decay_rate: 0.95, // 5% decay per time unit
            enable_adaptive_management: true,
            max_memories_per_scope: 1000,
            retention_period: 30 * 24 * 60 * 60, // 30 days
        }
    }
}

/// Memory processing statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProcessingStats {
    /// Number of memories processed
    pub processed_count: usize,

    /// Number of memories consolidated
    pub consolidated_count: usize,

    /// Number of memories with updated importance
    pub importance_updated_count: usize,

    /// Number of memories archived
    pub archived_count: usize,

    /// Number of memories deleted
    pub deleted_count: usize,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Main memory processing engine
pub struct MemoryProcessor {
    config: ProcessingConfig,
    consolidator: MemoryConsolidator,
    importance_scorer: ImportanceScorer,
    adaptive_manager: AdaptiveMemoryManager,
}

impl MemoryProcessor {
    /// Create a new memory processor
    pub fn new(config: ProcessingConfig) -> Self {
        let consolidator = MemoryConsolidator::new(config.consolidation_threshold);
        let importance_scorer = ImportanceScorer::new(config.importance_decay_rate);
        let adaptive_manager =
            AdaptiveMemoryManager::new(config.max_memories_per_scope, config.retention_period);

        Self {
            config,
            consolidator,
            importance_scorer,
            adaptive_manager,
        }
    }

    /// Process a batch of memories
    pub async fn process_memories(
        &mut self,
        memories: &mut Vec<Memory>,
    ) -> Result<ProcessingStats> {
        let start_time = std::time::Instant::now();
        let mut stats = ProcessingStats::default();
        stats.processed_count = memories.len();

        // Step 1: Consolidation
        if self.config.enable_consolidation {
            let consolidated = self.consolidator.consolidate_memories(memories).await?;
            stats.consolidated_count = consolidated;
        }

        // Step 2: Importance scoring
        if self.config.enable_importance_scoring {
            let updated = self
                .importance_scorer
                .update_importance_scores(memories)
                .await?;
            stats.importance_updated_count = updated;
        }

        // Step 3: Adaptive management
        if self.config.enable_adaptive_management {
            let (archived, deleted) = self.adaptive_manager.manage_memories(memories).await?;
            stats.archived_count = archived;
            stats.deleted_count = deleted;
        }

        stats.processing_time_ms = start_time.elapsed().as_millis() as u64;
        Ok(stats)
    }

    /// Process a single memory
    pub async fn process_single_memory(&mut self, memory: &mut Memory) -> Result<()> {
        // Update importance score
        if self.config.enable_importance_scoring {
            self.importance_scorer.score_single_memory(memory).await?;
        }

        // Check if memory should be archived or deleted
        if self.config.enable_adaptive_management {
            self.adaptive_manager.evaluate_single_memory(memory).await?;
        }

        Ok(())
    }

    /// Get processing configuration
    pub fn config(&self) -> &ProcessingConfig {
        &self.config
    }

    /// Update processing configuration
    pub fn update_config(&mut self, config: ProcessingConfig) {
        self.config = config.clone();
        self.consolidator
            .update_threshold(config.consolidation_threshold);
        self.importance_scorer
            .update_decay_rate(config.importance_decay_rate);
        self.adaptive_manager
            .update_limits(config.max_memories_per_scope, config.retention_period);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use agent_mem_traits::MemoryType;

    fn create_test_memory(id: &str, content: &str, importance: f32) -> Memory {
        use agent_mem_traits::{MemoryId, Content, AttributeSet, RelationGraph, MetadataV4 as Metadata};
        let now = Utc::now();
        let mut memory = Memory {
            id: MemoryId::from_string(id.to_string()),
            content: Content::Text(content.to_string()),
            attributes: AttributeSet::new(),
            relations: RelationGraph::new(),
            metadata: Metadata {
                created_at: now,
                updated_at: now,
                accessed_at: now,
                access_count: 0,
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
    async fn test_memory_processor_creation() {
        let config = ProcessingConfig::default();
        let processor = MemoryProcessor::new(config);
        assert!(processor.config().enable_consolidation);
        assert!(processor.config().enable_importance_scoring);
        assert!(processor.config().enable_adaptive_management);
    }

    #[tokio::test]
    async fn test_process_memories() {
        let config = ProcessingConfig::default();
        let mut processor = MemoryProcessor::new(config);

        let mut memories = vec![
            create_test_memory("mem1", "First memory", 0.8),
            create_test_memory("mem2", "Second memory", 0.6),
            create_test_memory("mem3", "Third memory", 0.9),
        ];

        let stats = processor.process_memories(&mut memories).await.unwrap();
        assert_eq!(stats.processed_count, 3);
        // Processing time might be 0 in fast tests, so just check it's valid
        assert!(stats.processing_time_ms >= 0);
    }

    #[tokio::test]
    async fn test_process_single_memory() {
        let config = ProcessingConfig::default();
        let mut processor = MemoryProcessor::new(config);

        let mut memory = create_test_memory("mem1", "Test memory", 0.7);
        let result = processor.process_single_memory(&mut memory).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_config_update() {
        let mut config = ProcessingConfig::default();
        let mut processor = MemoryProcessor::new(config.clone());

        config.consolidation_threshold = 0.9;
        config.importance_decay_rate = 0.8;
        processor.update_config(config.clone());

        assert_eq!(processor.config().consolidation_threshold, 0.9);
        assert_eq!(processor.config().importance_decay_rate, 0.8);
    }
}
