//! Cognitive memory type definitions
//! 
//! Defines the 8 cognitive memory types and their characteristics.

use agent_mem_types::{MemoryType, Metadata};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Cognitive memory item with type-specific data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveMemoryItem {
    /// Memory ID
    pub id: String,
    /// Memory type
    pub memory_type: MemoryType,
    /// Content
    pub content: String,
    /// Importance score (0.0-1.0)
    pub importance: f32,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last access timestamp
    pub accessed_at: DateTime<Utc>,
    /// Access count
    pub access_count: u64,
    /// Metadata
    pub metadata: Metadata,
    /// Consolidation level (0.0-1.0)
    pub consolidation_level: f32,
    /// Decay factor
    pub decay_factor: f32,
}

impl CognitiveMemoryItem {
    /// Create new cognitive memory item
    pub fn new(id: String, memory_type: MemoryType, content: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            memory_type,
            content,
            importance: 0.5,
            created_at: now,
            accessed_at: now,
            access_count: 0,
            metadata: Metadata::new(),
            consolidation_level: 0.0,
            decay_factor: 1.0,
        }
    }

    /// Set importance score
    pub fn with_importance(mut self, importance: f32) -> Self {
        self.importance = importance.clamp(0.0, 1.0);
        self
    }

    /// Set consolidation level
    pub fn with_consolidation(mut self, level: f32) -> Self {
        self.consolidation_level = level.clamp(0.0, 1.0);
        self
    }

    /// Record access
    pub fn record_access(&mut self) {
        self.access_count += 1;
        self.accessed_at = Utc::now();
    }

    /// Calculate current strength based on Ebbinghaus curve
    pub fn calculate_strength(&self) -> f32 {
        let time_diff = (Utc::now() - self.created_at).num_hours() as f32;
        let base_strength = self.importance * self.consolidation_level;
        // Simple exponential decay
        base_strength * self.decay_factor * (-0.1 * time_diff / 24.0).exp()
    }
}

/// Memory consolidation status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[derive(Default)]
pub enum ConsolidationStatus {
    /// Not yet consolidated
    #[default]
    Fresh,
    /// Currently being consolidated
    Consolidating,
    /// Fully consolidated
    Stable,
    /// Decaying
    Decaying,
    /// Forgotten
    Forgotten,
}


/// Memory importance weighting for different cognitive types
#[derive(Debug, Clone)]
pub struct CognitiveWeights {
    /// Episodic decay rate (per hour)
    pub episodic_decay: f32,
    /// Semantic reinforcement rate
    pub semantic_reinforcement: f32,
    /// Procedural retention rate
    pub procedural_retention: f32,
    /// Working memory capacity
    pub working_capacity: usize,
}

impl Default for CognitiveWeights {
    fn default() -> Self {
        Self {
            episodic_decay: 0.1,
            semantic_reinforcement: 0.05,
            procedural_retention: 0.02,
            working_capacity: 7, // Miller's law
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cognitive_memory_creation() {
        let mem = CognitiveMemoryItem::new(
            "test-id".to_string(),
            MemoryType::Episodic,
            "Test content".to_string(),
        );
        assert_eq!(mem.id, "test-id");
        assert_eq!(mem.memory_type, MemoryType::Episodic);
        assert_eq!(mem.access_count, 0);
    }

    #[test]
    fn test_cognitive_memory_importance() {
        let mem = CognitiveMemoryItem::new(
            "test-id".to_string(),
            MemoryType::Semantic,
            "Test".to_string(),
        ).with_importance(0.8);
        assert!((mem.importance - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_record_access() {
        let mut mem = CognitiveMemoryItem::new(
            "test-id".to_string(),
            MemoryType::Procedural,
            "Test".to_string(),
        );
        mem.record_access();
        assert_eq!(mem.access_count, 1);
    }
}
