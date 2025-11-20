//! Conflict detection and resolution

use crate::types::Memory;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Conflict detector
pub struct ConflictDetector {
    /// Similarity threshold for detecting conflicts (0.0-1.0)
    pub similarity_threshold: f32,
    /// Time window in seconds for considering conflicts
    pub time_window_secs: i64,
}

impl ConflictDetector {
    /// Create new conflict detector
    pub fn new() -> Self {
        Self {
            similarity_threshold: 0.85,
            time_window_secs: 3600, // 1 hour
        }
    }

    /// Create with custom threshold
    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.similarity_threshold = threshold.max(0.0).min(1.0);
        self
    }

    /// Create with custom time window
    pub fn with_time_window(mut self, seconds: i64) -> Self {
        self.time_window_secs = seconds.max(0);
        self
    }

    /// Detect conflicts between memories
    pub fn detect_conflicts(&self, memories: &[Memory]) -> Vec<ConflictPair> {
        let mut conflicts = Vec::new();

        for i in 0..memories.len() {
            for j in (i + 1)..memories.len() {
                if let Some(conflict_type) = self.check_conflict(&memories[i], &memories[j]) {
                    conflicts.push(ConflictPair {
                        memory1_id: memories[i].id.clone(),
                        memory2_id: memories[j].id.clone(),
                        conflict_type,
                        similarity: self.calculate_similarity(&memories[i], &memories[j]),
                    });
                }
            }
        }

        conflicts
    }

    /// Check if two memories conflict
    fn check_conflict(&self, m1: &Memory, m2: &Memory) -> Option<ConflictType> {
        // Check time proximity
        let time_diff = (m1.metadata.created_at - m2.metadata.created_at).abs();
        if time_diff > chrono::TimeDelta::seconds(self.time_window_secs) {
            return None;
        }

        // Calculate content similarity
        let similarity = self.calculate_similarity(m1, m2);

        if similarity >= self.similarity_threshold {
            // Check version conflict (by comparing updated_at timestamps)
            if m1.metadata.updated_at != m1.metadata.created_at
                || m2.metadata.updated_at != m2.metadata.created_at
            {
                return Some(ConflictType::VersionConflict);
            }

            // Check content conflict (similar but not identical)
            if m1.content.to_string() != m2.content.to_string() {
                return Some(ConflictType::ContentConflict);
            }

            // Exact duplicate
            return Some(ConflictType::Duplicate);
        }

        None
    }

    /// Calculate similarity between two memories
    fn calculate_similarity(&self, m1: &Memory, m2: &Memory) -> f32 {
        let content1 = m1.content.to_string();
        let content2 = m2.content.to_string();

        // Simple Jaccard similarity based on words
        let words1: HashMap<&str, usize> =
            content1
                .split_whitespace()
                .fold(HashMap::new(), |mut map, word| {
                    *map.entry(word).or_insert(0) += 1;
                    map
                });

        let words2: HashMap<&str, usize> =
            content2
                .split_whitespace()
                .fold(HashMap::new(), |mut map, word| {
                    *map.entry(word).or_insert(0) += 1;
                    map
                });

        let intersection: usize = words1
            .keys()
            .filter(|k| words2.contains_key(*k))
            .map(|k| words1[k].min(*words2.get(k).unwrap_or(&0)))
            .sum();

        let union: usize =
            words1.values().sum::<usize>() + words2.values().sum::<usize>() - intersection;

        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }
}

impl Default for ConflictDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// A pair of conflicting memories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictPair {
    /// First memory ID
    pub memory1_id: String,
    /// Second memory ID
    pub memory2_id: String,
    /// Type of conflict
    pub conflict_type: ConflictType,
    /// Similarity score (0.0-1.0)
    pub similarity: f32,
}

/// Type of conflict detected
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConflictType {
    /// Exact duplicate
    Duplicate,
    /// Content conflict (similar but different)
    ContentConflict,
    /// Version conflict
    VersionConflict,
    /// Temporal conflict
    TemporalConflict,
}

/// Conflict resolution strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolution {
    /// Keep the first memory
    KeepFirst,

    /// Keep the last memory
    KeepLast,

    /// Merge memories
    Merge,

    /// Manual resolution required
    Manual,
}
