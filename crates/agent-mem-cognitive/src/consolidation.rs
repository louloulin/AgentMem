//! Memory Consolidation Module
//! 
//! Implements memory consolidation processes:
//! - Short-term to long-term transfer
//! - Memory integration and schema formation
//! - Important memory prioritization

use super::{CognitiveMemoryItem, ConsolidationStatus};
use chrono::{DateTime, Utc};

/// Memory consolidation engine
#[derive(Debug, Clone)]
pub struct ConsolidationEngine {
    /// Threshold for short-term to long-term transfer (0.0-1.0)
    pub transfer_threshold: f32,
    /// Maximum consolidated memories
    pub max_long_term: usize,
}

impl ConsolidationEngine {
    pub fn new() -> Self {
        Self {
            transfer_threshold: 0.7,
            max_long_term: 1000,
        }
    }

    /// Determine consolidation status for a memory
    pub fn evaluate(&self, item: &CognitiveMemoryItem) -> ConsolidationStatus {
        let retention = item.calculate_strength();
        let access_count = item.access_count;
        let importance = item.importance;
        
        // Consolidation based on multiple factors
        let consolidation_score = retention * 0.4 + 
                                (access_count as f32 / 10.0).min(1.0) * 0.3 +
                                importance * 0.3;
        
        if consolidation_score >= self.transfer_threshold {
            ConsolidationStatus::Stable
        } else if consolidation_score >= 0.3 {
            ConsolidationStatus::Consolidating
        } else {
            ConsolidationStatus::Fresh
        }
    }

    /// Check if memory should be transferred to long-term
    pub fn should_transfer(&self, item: &CognitiveMemoryItem) -> bool {
        let status = self.evaluate(item);
        matches!(status, ConsolidationStatus::Stable | ConsolidationStatus::Consolidating)
    }

    /// Calculate priority for memory retention
    pub fn priority(&self, item: &CognitiveMemoryItem) -> f32 {
        let importance = item.importance;
        let consolidation_level = item.consolidation_level;
        let retention = item.calculate_strength();
        let access_count = item.access_count.min(10) as f32 / 10.0;
        
        // Weighted combination
        importance * 0.4 + consolidation_level * 0.3 + retention * 0.2 + access_count * 0.1
    }
}

impl Default for ConsolidationEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory fusion - combining related memories
pub struct MemoryFusion;

impl MemoryFusion {
    /// Fuse two memories into one (if similar enough)
    pub fn can_fuse(m1: &CognitiveMemoryItem, m2: &CognitiveMemoryItem, threshold: f32) -> bool {
        // Same type and similar importance
        if m1.memory_type != m2.memory_type {
            return false;
        }
        
        // Similar importance (within threshold)
        (m1.importance - m2.importance).abs() < threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_mem_types::MemoryType;

    #[test]
    fn test_consolidation_evaluation() {
        let engine = ConsolidationEngine::new();
        // Create a high importance item with some consolidation
        let item = CognitiveMemoryItem::new(
            "test".to_string(),
            MemoryType::Episodic,
            "test content".to_string(),
        ).with_importance(1.0).with_consolidation(1.0);
        
        let status = engine.evaluate(&item);
        // High importance and consolidation should lead to stable status
        assert!(matches!(status, ConsolidationStatus::Stable));
    }

    #[test]
    fn test_consolidation_fresh() {
        let engine = ConsolidationEngine::new();
        // Create a fresh item (low consolidation)
        let item = CognitiveMemoryItem::new(
            "test2".to_string(),
            MemoryType::Semantic,
            "test".to_string(),
        );
        
        let status = engine.evaluate(&item);
        // Fresh item should be Fresh or Consolidating
        assert!(matches!(status, ConsolidationStatus::Fresh | ConsolidationStatus::Consolidating));
    }

    #[test]
    fn test_priority_calculation() {
        let engine = ConsolidationEngine::new();
        let item = CognitiveMemoryItem::new(
            "test".to_string(),
            MemoryType::Semantic,
            "test".to_string(),
        ).with_importance(0.8);
        
        let priority = engine.priority(&item);
        assert!(priority > 0.0 && priority <= 1.0);
    }

    #[test]
    fn test_memory_fusion() {
        let m1 = CognitiveMemoryItem::new("1".to_string(), MemoryType::Procedural, "test".to_string())
            .with_importance(0.8);
        let m2 = CognitiveMemoryItem::new("2".to_string(), MemoryType::Procedural, "test".to_string())
            .with_importance(0.75);
        
        assert!(MemoryFusion::can_fuse(&m1, &m2, 0.2));
        assert!(!MemoryFusion::can_fuse(&m1, &m2, 0.01)); // Too different
    }

    #[test]
    fn test_should_transfer() {
        let engine = ConsolidationEngine::new();
        let item = CognitiveMemoryItem::new("1".to_string(), MemoryType::Episodic, "test".to_string())
            .with_importance(1.0).with_consolidation(1.0);
        
        assert!(engine.should_transfer(&item));
    }
}
