//! Stage 4: Dedupe Merger
//!
//! Removes duplicate and similar memory items

use crate::error::Result;
use crate::models::{ExtractionContext, ExtractionInput, ExtractionOutput, MemoryItem};
use crate::stage::{ExtractionStage, StagePriority};
use async_trait::async_trait;
use std::collections::HashSet;
use tracing::{debug, info};

/// Stage 4: Dedupe Merger
///
/// This stage:
/// - Identifies duplicate items (exact matches)
/// - Identifies similar items (semantic similarity)
/// - Merges duplicate items
pub struct DedupeMerger {
    /// Similarity threshold (0-1, higher = more strict)
    threshold: f32,
}

impl DedupeMerger {
    /// Create new dedupe merger
    pub fn new() -> Self {
        Self { threshold: 0.85 }
    }

    /// Create with custom threshold
    pub fn with_threshold(threshold: f32) -> Self {
        Self {
            threshold: threshold.clamp(0.0, 1.0),
        }
    }

    /// Remove duplicate items
    fn deduplicate(&self, items: Vec<MemoryItem>) -> Vec<MemoryItem> {
        let initial_count = items.len();
        let mut unique_items = Vec::new();
        let mut seen = HashSet::new();

        for item in items {
            // Normalize content for comparison
            let normalized = self.normalize_content(&item.content);

            // Check for exact duplicates
            if seen.contains(&normalized) {
                debug!("Duplicate item removed: {}", normalized);
                continue;
            }

            seen.insert(normalized);
            unique_items.push(item);
        }

        info!(
            "Deduplication: {} -> {} items",
            initial_count,
            unique_items.len()
        );

        unique_items
    }

    /// Normalize content for comparison
    fn normalize_content(&self, content: &str) -> String {
        content
            .to_lowercase()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Calculate Jaccard similarity between two strings
    fn jaccard_similarity(&self, s1: &str, s2: &str) -> f32 {
        let words1: HashSet<&str> = s1.split_whitespace().collect();
        let words2: HashSet<&str> = s2.split_whitespace().collect();

        if words1.is_empty() && words2.is_empty() {
            return 1.0;
        }

        let intersection = words1.intersection(&words2).count() as f32;
        let union = words1.union(&words2).count() as f32;

        if union == 0.0 {
            0.0
        } else {
            intersection / union
        }
    }

    /// Merge similar items
    #[allow(clippy::needless_range_loop)]
    fn merge_similar(&self, items: Vec<MemoryItem>) -> Vec<MemoryItem> {
        let mut merged = Vec::new();
        let mut merged_indices = HashSet::new();

        for i in 0..items.len() {
            if merged_indices.contains(&i) {
                continue;
            }

            let mut current_item = items[i].clone();
            let mut similar_items = Vec::new();

            // Find similar items
            for j in (i + 1)..items.len() {
                if merged_indices.contains(&j) {
                    continue;
                }

                let similarity = self.jaccard_similarity(&current_item.content, &items[j].content);

                if similarity >= self.threshold {
                    similar_items.push(j);
                }
            }

            // Merge similar items
            if !similar_items.is_empty() {
                debug!("Merging {} similar items", similar_items.len() + 1);

                for idx in similar_items {
                    merged_indices.insert(idx);
                    // Merge metadata
                    for (key, value) in &items[idx].metadata {
                        current_item.metadata.insert(key.clone(), value.clone());
                    }
                    // Update confidence (use max)
                    current_item.confidence = current_item.confidence.max(items[idx].confidence);
                }
            }

            merged_indices.insert(i);
            merged.push(current_item);
        }

        merged
    }
}

impl Default for DedupeMerger {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ExtractionStage for DedupeMerger {
    fn name(&self) -> &str {
        "DedupeMerger"
    }

    fn priority(&self) -> StagePriority {
        StagePriority::NORMAL
    }

    async fn process(
        &self,
        _input: ExtractionInput,
        mut output: ExtractionOutput,
        _context: &mut ExtractionContext,
    ) -> Result<ExtractionOutput> {
        debug!("DedupeMerger processing");

        let initial_count = output.items.len();

        // Remove exact duplicates
        output.items = self.deduplicate(output.items);

        let after_dedup_count = output.items.len();
        output.metrics.items_deduped = initial_count - after_dedup_count;

        // Merge similar items
        output.items = self.merge_similar(output.items);

        let final_count = output.items.len();
        output.metrics.items_deduped += after_dedup_count - final_count;

        info!(
            "Deduplication completed: {} -> {} items ({} removed)",
            initial_count, final_count, output.metrics.items_deduped
        );

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate() {
        let merger = DedupeMerger::new();

        let items = vec![
            MemoryItem::new("User likes Rust".to_string(), "preference".to_string()),
            MemoryItem::new("User likes Rust".to_string(), "preference".to_string()),
            MemoryItem::new("User prefers Go".to_string(), "preference".to_string()),
        ];

        let unique = merger.deduplicate(items);

        assert_eq!(unique.len(), 2);
    }

    #[test]
    fn test_jaccard_similarity() {
        let merger = DedupeMerger::new();

        let s1 = "User likes Rust programming language";
        let s2 = "User likes Go programming language";

        let similarity = merger.jaccard_similarity(s1, s2);

        assert!(similarity > 0.5);
        assert!(similarity < 1.0);
    }

    #[test]
    fn test_normalize_content() {
        let merger = DedupeMerger::new();

        let content = "  User   Likes   Rust  ";
        let normalized = merger.normalize_content(content);

        assert_eq!(normalized, "user likes rust");
    }

    #[test]
    fn test_stage_priority() {
        let merger = DedupeMerger::new();
        assert_eq!(merger.priority(), StagePriority::NORMAL);
    }

    #[test]
    fn test_stage_name() {
        let merger = DedupeMerger::new();
        assert_eq!(merger.name(), "DedupeMerger");
    }
}
