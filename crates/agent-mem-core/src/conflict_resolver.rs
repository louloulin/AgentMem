//! Memory Conflict Resolution
//!
//! Advanced conflict resolution algorithms ported from ContextEngine
//! for handling memory conflicts with multiple resolution strategies.

use crate::hierarchical_service::{ConflictResolutionStrategy, HierarchicalMemoryRecord};
use crate::types::ImportanceLevel;
use agent_mem_traits::{AgentMemError, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Conflict resolution engine
pub struct ConflictResolver {
    config: ConflictResolverConfig,
    resolution_cache: HashMap<String, ConflictResolution>,
}

/// Configuration for conflict resolver
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolverConfig {
    /// Enable semantic similarity detection
    pub enable_semantic_detection: bool,
    /// Similarity threshold for conflict detection (0.0-1.0)
    pub similarity_threshold: f64,
    /// Time window for conflict detection (hours)
    pub time_window_hours: i64,
    /// Enable automatic resolution
    pub auto_resolve: bool,
    /// Default resolution strategy
    pub default_strategy: ConflictResolutionStrategy,
    /// Quality score weight in resolution
    pub quality_weight: f64,
    /// Importance weight in resolution
    pub importance_weight: f64,
    /// Recency weight in resolution
    pub recency_weight: f64,
}

impl Default for ConflictResolverConfig {
    fn default() -> Self {
        Self {
            enable_semantic_detection: true,
            similarity_threshold: 0.85,
            time_window_hours: 24,
            auto_resolve: true,
            default_strategy: ConflictResolutionStrategy::ImportanceBased,
            quality_weight: 0.3,
            importance_weight: 0.4,
            recency_weight: 0.3,
        }
    }
}

/// Conflict resolution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolution {
    pub conflict_id: String,
    pub conflicting_memories: Vec<String>,
    pub resolution_strategy: ConflictResolutionStrategy,
    pub resolved_memory: Option<HierarchicalMemoryRecord>,
    pub merged_memories: Vec<HierarchicalMemoryRecord>,
    pub resolution_confidence: f64,
    pub resolution_timestamp: DateTime<Utc>,
    pub resolution_metadata: HashMap<String, String>,
}

/// Memory conflict detection result
#[derive(Debug, Clone)]
pub struct ConflictDetection {
    pub has_conflict: bool,
    pub conflicting_memories: Vec<HierarchicalMemoryRecord>,
    pub conflict_type: ConflictType,
    pub similarity_score: f64,
    pub confidence: f64,
}

/// Types of memory conflicts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictType {
    /// Semantic conflict - similar meaning but different content
    Semantic,
    /// Temporal conflict - conflicting information about same time period
    Temporal,
    /// Factual conflict - contradictory facts
    Factual,
    /// Duplicate - exact or near-exact duplicates
    Duplicate,
    /// Inconsistent - logically inconsistent information
    Inconsistent,
}

impl ConflictResolver {
    /// Create a new conflict resolver
    pub fn new(config: ConflictResolverConfig) -> Self {
        Self {
            config,
            resolution_cache: HashMap::new(),
        }
    }

    /// Detect conflicts for a new memory
    pub async fn detect_conflicts(
        &self,
        new_memory: &HierarchicalMemoryRecord,
        existing_memories: &[HierarchicalMemoryRecord],
    ) -> Result<ConflictDetection> {
        let mut conflicting_memories = Vec::new();
        let mut max_similarity: f64 = 0.0;
        let mut conflict_type = ConflictType::Semantic;

        // Check for conflicts within time window
        let time_threshold = Utc::now() - Duration::hours(self.config.time_window_hours);

        for existing_memory in existing_memories {
            // Skip if outside time window
            if existing_memory.created_at < time_threshold {
                continue;
            }

            // Skip if same memory
            if existing_memory.id == new_memory.id {
                continue;
            }

            // Check for different types of conflicts
            let similarity = self
                .calculate_semantic_similarity(new_memory, existing_memory)
                .await?;

            if similarity > self.config.similarity_threshold {
                conflicting_memories.push(existing_memory.clone());
                max_similarity = max_similarity.max(similarity);

                // Determine conflict type
                if similarity > 0.95 {
                    conflict_type = ConflictType::Duplicate;
                } else if self.is_factual_conflict(new_memory, existing_memory) {
                    conflict_type = ConflictType::Factual;
                } else if self.is_temporal_conflict(new_memory, existing_memory) {
                    conflict_type = ConflictType::Temporal;
                } else {
                    conflict_type = ConflictType::Semantic;
                }
            }
        }

        let has_conflict = !conflicting_memories.is_empty();
        let confidence = if has_conflict { max_similarity } else { 0.0 };

        Ok(ConflictDetection {
            has_conflict,
            conflicting_memories,
            conflict_type,
            similarity_score: max_similarity,
            confidence,
        })
    }

    /// Resolve conflicts using specified strategy
    pub async fn resolve_conflicts(
        &mut self,
        new_memory: &HierarchicalMemoryRecord,
        conflicting_memories: &[HierarchicalMemoryRecord],
        strategy: Option<ConflictResolutionStrategy>,
    ) -> Result<ConflictResolution> {
        let resolution_strategy = strategy.unwrap_or(self.config.default_strategy.clone());
        let conflict_id = uuid::Uuid::new_v4().to_string();

        let resolution = match resolution_strategy {
            ConflictResolutionStrategy::TimeBasedNewest => {
                self.resolve_time_based_newest(new_memory, conflicting_memories)
                    .await?
            }
            ConflictResolutionStrategy::ImportanceBased => {
                self.resolve_importance_based(new_memory, conflicting_memories)
                    .await?
            }
            ConflictResolutionStrategy::SourceReliabilityBased => {
                self.resolve_source_reliability_based(new_memory, conflicting_memories)
                    .await?
            }
            ConflictResolutionStrategy::SemanticMerge => {
                self.resolve_semantic_merge(new_memory, conflicting_memories)
                    .await?
            }
            ConflictResolutionStrategy::KeepBoth => {
                self.resolve_keep_both(new_memory, conflicting_memories)
                    .await?
            }
        };

        // Cache the resolution
        self.resolution_cache
            .insert(conflict_id.clone(), resolution.clone());

        Ok(resolution)
    }

    /// Calculate semantic similarity between two memories
    async fn calculate_semantic_similarity(
        &self,
        memory1: &HierarchicalMemoryRecord,
        memory2: &HierarchicalMemoryRecord,
    ) -> Result<f64> {
        // Simplified similarity calculation
        // In production, would use embedding-based semantic similarity
        let content1 = memory1.content.to_lowercase();
        let content2 = memory2.content.to_lowercase();

        // Simple word overlap similarity
        let words1: std::collections::HashSet<&str> = content1.split_whitespace().collect();
        let words2: std::collections::HashSet<&str> = content2.split_whitespace().collect();

        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();

        if union == 0 {
            Ok(0.0)
        } else {
            Ok(intersection as f64 / union as f64)
        }
    }

    /// Check if memories have factual conflicts
    fn is_factual_conflict(
        &self,
        memory1: &HierarchicalMemoryRecord,
        memory2: &HierarchicalMemoryRecord,
    ) -> bool {
        // Simplified factual conflict detection
        // In production, would use NLP techniques to detect contradictions
        let content1 = memory1.content.to_lowercase();
        let content2 = memory2.content.to_lowercase();

        // Look for contradictory patterns
        let contradictory_pairs = [
            ("is", "is not"),
            ("true", "false"),
            ("yes", "no"),
            ("can", "cannot"),
            ("will", "will not"),
        ];

        for (positive, negative) in contradictory_pairs.iter() {
            if (content1.contains(positive) && content2.contains(negative))
                || (content1.contains(negative) && content2.contains(positive))
            {
                return true;
            }
        }

        false
    }

    /// Check if memories have temporal conflicts
    fn is_temporal_conflict(
        &self,
        memory1: &HierarchicalMemoryRecord,
        memory2: &HierarchicalMemoryRecord,
    ) -> bool {
        // Simplified temporal conflict detection
        // In production, would parse dates and check for temporal inconsistencies
        let time_diff = (memory1.created_at - memory2.created_at)
            .num_minutes()
            .abs();
        time_diff < 60 // Conflicts if created within 1 hour
    }

    /// Resolve conflict by choosing newest memory
    async fn resolve_time_based_newest(
        &self,
        new_memory: &HierarchicalMemoryRecord,
        conflicting_memories: &[HierarchicalMemoryRecord],
    ) -> Result<ConflictResolution> {
        let mut all_memories = vec![new_memory.clone()];
        all_memories.extend(conflicting_memories.iter().cloned());

        // Sort by creation time, newest first
        all_memories.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        let resolved_memory = all_memories.into_iter().next();

        Ok(ConflictResolution {
            conflict_id: uuid::Uuid::new_v4().to_string(),
            conflicting_memories: conflicting_memories.iter().map(|m| m.id.clone()).collect(),
            resolution_strategy: ConflictResolutionStrategy::TimeBasedNewest,
            resolved_memory,
            merged_memories: Vec::new(),
            resolution_confidence: 0.8,
            resolution_timestamp: Utc::now(),
            resolution_metadata: HashMap::new(),
        })
    }

    /// Resolve conflict by choosing highest importance memory
    async fn resolve_importance_based(
        &self,
        new_memory: &HierarchicalMemoryRecord,
        conflicting_memories: &[HierarchicalMemoryRecord],
    ) -> Result<ConflictResolution> {
        let mut all_memories = vec![new_memory.clone()];
        all_memories.extend(conflicting_memories.iter().cloned());

        // Sort by importance, highest first
        all_memories.sort_by(|a, b| b.importance.cmp(&a.importance));

        let resolved_memory = all_memories.into_iter().next();

        Ok(ConflictResolution {
            conflict_id: uuid::Uuid::new_v4().to_string(),
            conflicting_memories: conflicting_memories.iter().map(|m| m.id.clone()).collect(),
            resolution_strategy: ConflictResolutionStrategy::ImportanceBased,
            resolved_memory,
            merged_memories: Vec::new(),
            resolution_confidence: 0.9,
            resolution_timestamp: Utc::now(),
            resolution_metadata: HashMap::new(),
        })
    }

    /// Resolve conflict by choosing most reliable source
    async fn resolve_source_reliability_based(
        &self,
        new_memory: &HierarchicalMemoryRecord,
        conflicting_memories: &[HierarchicalMemoryRecord],
    ) -> Result<ConflictResolution> {
        let mut all_memories = vec![new_memory.clone()];
        all_memories.extend(conflicting_memories.iter().cloned());

        // Sort by source reliability, highest first
        all_memories.sort_by(|a, b| {
            b.source_reliability
                .partial_cmp(&a.source_reliability)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let resolved_memory = all_memories.into_iter().next();

        Ok(ConflictResolution {
            conflict_id: uuid::Uuid::new_v4().to_string(),
            conflicting_memories: conflicting_memories.iter().map(|m| m.id.clone()).collect(),
            resolution_strategy: ConflictResolutionStrategy::SourceReliabilityBased,
            resolved_memory,
            merged_memories: Vec::new(),
            resolution_confidence: 0.85,
            resolution_timestamp: Utc::now(),
            resolution_metadata: HashMap::new(),
        })
    }

    /// Resolve conflict by merging semantically similar memories
    async fn resolve_semantic_merge(
        &self,
        new_memory: &HierarchicalMemoryRecord,
        conflicting_memories: &[HierarchicalMemoryRecord],
    ) -> Result<ConflictResolution> {
        // Create merged memory combining information from all conflicting memories
        let mut merged_content = new_memory.content.clone();
        let mut merged_metadata = new_memory.metadata.clone();
        let mut merged_tags = new_memory.tags.clone();

        for memory in conflicting_memories {
            // Simple merge - in production would use more sophisticated NLP
            if !merged_content.contains(&memory.content) {
                merged_content.push_str(&format!(" | {}", memory.content));
            }

            // Merge metadata
            for (key, value) in &memory.metadata {
                merged_metadata.entry(key.clone()).or_insert(value.clone());
            }

            // Merge tags
            for tag in &memory.tags {
                if !merged_tags.contains(tag) {
                    merged_tags.push(tag.clone());
                }
            }
        }

        let mut merged_memory = new_memory.clone();
        merged_memory.content = merged_content;
        merged_memory.metadata = merged_metadata;
        merged_memory.tags = merged_tags;
        merged_memory.updated_at = Utc::now();

        Ok(ConflictResolution {
            conflict_id: uuid::Uuid::new_v4().to_string(),
            conflicting_memories: conflicting_memories.iter().map(|m| m.id.clone()).collect(),
            resolution_strategy: ConflictResolutionStrategy::SemanticMerge,
            resolved_memory: Some(merged_memory),
            merged_memories: Vec::new(),
            resolution_confidence: 0.7,
            resolution_timestamp: Utc::now(),
            resolution_metadata: HashMap::new(),
        })
    }

    /// Resolve conflict by keeping both memories with conflict markers
    async fn resolve_keep_both(
        &self,
        new_memory: &HierarchicalMemoryRecord,
        conflicting_memories: &[HierarchicalMemoryRecord],
    ) -> Result<ConflictResolution> {
        let mut all_memories = vec![new_memory.clone()];
        all_memories.extend(conflicting_memories.iter().cloned());

        // Mark all memories as conflicting
        for memory in &mut all_memories {
            memory
                .metadata
                .insert("conflict_marker".to_string(), "true".to_string());
            memory
                .metadata
                .insert("conflict_timestamp".to_string(), Utc::now().to_rfc3339());
        }

        Ok(ConflictResolution {
            conflict_id: uuid::Uuid::new_v4().to_string(),
            conflicting_memories: conflicting_memories.iter().map(|m| m.id.clone()).collect(),
            resolution_strategy: ConflictResolutionStrategy::KeepBoth,
            resolved_memory: None,
            merged_memories: all_memories,
            resolution_confidence: 1.0,
            resolution_timestamp: Utc::now(),
            resolution_metadata: HashMap::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hierarchy::{MemoryLevel, MemoryScope};

    fn create_test_memory(content: &str, importance: ImportanceLevel) -> HierarchicalMemoryRecord {
        HierarchicalMemoryRecord {
            id: uuid::Uuid::new_v4().to_string(),
            content: content.to_string(),
            scope: MemoryScope::Global,
            level: MemoryLevel::Operational,
            importance,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            accessed_at: Utc::now(),
            access_count: 0,
            metadata: HashMap::new(),
            tags: Vec::new(),
            parent_memory_id: None,
            child_memory_ids: Vec::new(),
            conflict_resolution_strategy: ConflictResolutionStrategy::ImportanceBased,
            quality_score: 1.0,
            source_reliability: 1.0,
        }
    }

    #[tokio::test]
    async fn test_conflict_detection() {
        let resolver = ConflictResolver::new(ConflictResolverConfig::default());

        let new_memory = create_test_memory("The sky is blue", ImportanceLevel::Medium);
        let existing_memory = create_test_memory("The sky is blue", ImportanceLevel::High); // Make them more similar

        let detection = resolver
            .detect_conflicts(&new_memory, &[existing_memory])
            .await
            .unwrap();

        assert!(detection.has_conflict);
        assert_eq!(detection.conflicting_memories.len(), 1);
        assert!(detection.similarity_score > 0.5);
    }

    #[tokio::test]
    async fn test_importance_based_resolution() {
        let mut resolver = ConflictResolver::new(ConflictResolverConfig::default());

        let new_memory = create_test_memory("Important fact", ImportanceLevel::Medium);
        let conflicting_memory = create_test_memory("Very important fact", ImportanceLevel::High);

        let resolution = resolver
            .resolve_conflicts(
                &new_memory,
                &[conflicting_memory.clone()],
                Some(ConflictResolutionStrategy::ImportanceBased),
            )
            .await
            .unwrap();

        assert!(resolution.resolved_memory.is_some());
        let resolved = resolution.resolved_memory.unwrap();
        assert_eq!(resolved.importance, ImportanceLevel::High);
        assert_eq!(resolved.content, "Very important fact");
    }
}
