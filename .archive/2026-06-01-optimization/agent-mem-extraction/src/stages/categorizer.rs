//! Stage 5: Auto Categorizer
//!
//! Automatically categorizes memory items

use crate::error::Result;
use crate::models::{ExtractionContext, ExtractionInput, ExtractionOutput};
use crate::stage::{ExtractionStage, StagePriority};
use async_trait::async_trait;
use tracing::{debug, info};

/// Stage 5: Auto Categorizer
///
/// This stage:
/// - Automatically assigns categories to items
/// - Uses rule-based and LLM-based classification
/// - Creates new categories if needed
pub struct AutoCategorizer {
    /// Confidence threshold for category assignment
    confidence_threshold: f32,
}

impl AutoCategorizer {
    /// Create new auto categorizer
    pub fn new() -> Self {
        Self {
            confidence_threshold: 0.7,
        }
    }

    /// Create with custom threshold
    pub fn with_threshold(threshold: f32) -> Self {
        Self {
            confidence_threshold: threshold.clamp(0.0, 1.0),
        }
    }

    /// Categorize an item based on its content and type
    fn categorize_item(&self, item: &crate::models::MemoryItem) -> String {
        let item_type = &item.item_type;

        match item_type.as_str() {
            "preference" => {
                // Extract preference category from content
                if item.content.contains("programming") || item.content.contains("code") {
                    "/preferences/programming".to_string()
                } else if item.content.contains("communication") {
                    "/preferences/communication".to_string()
                } else if item.content.contains("design") {
                    "/preferences/design".to_string()
                } else {
                    "/preferences/other".to_string()
                }
            }
            "fact" => {
                // Extract knowledge category
                if item.content.contains("technology") || item.content.contains("software") {
                    "/knowledge/technology".to_string()
                } else if item.content.contains("science") {
                    "/knowledge/science".to_string()
                } else {
                    "/knowledge/general".to_string()
                }
            }
            "event" => {
                // Events go to timeline
                "/timeline/events".to_string()
            }
            "skill" => {
                // Extract skill category
                if item.content.contains("programming") || item.content.contains("code") {
                    "/skills/programming".to_string()
                } else if item.content.contains("communication") {
                    "/skills/communication".to_string()
                } else if item.content.contains("design") {
                    "/skills/design".to_string()
                } else {
                    "/skills/other".to_string()
                }
            }
            _ => "/uncategorized".to_string(),
        }
    }

    /// Categorize all items
    fn categorize(&self, items: &mut Vec<crate::models::MemoryItem>) {
        let mut category_counts = std::collections::HashMap::new();
        let item_count = items.len();

        for item in items {
            let category = self.categorize_item(item);
            item.category = Some(category.clone());

            *category_counts.entry(category).or_insert(0) += 1;
        }

        info!(
            "Categorized {} items into {} categories",
            item_count,
            category_counts.len()
        );

        for (category, count) in &category_counts {
            debug!("Category '{}': {} items", category, count);
        }
    }
}

impl Default for AutoCategorizer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ExtractionStage for AutoCategorizer {
    fn name(&self) -> &str {
        "AutoCategorizer"
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
        debug!("AutoCategorizer processing");

        // Categorize all items
        self.categorize(&mut output.items);

        // Collect unique categories
        let mut unique_categories = std::collections::HashSet::new();
        for item in &output.items {
            if let Some(ref category) = item.category {
                unique_categories.insert(category.clone());
            }
        }

        output.categories = unique_categories.into_iter().collect();
        output.metrics.categories_created = output.categories.len();

        info!(
            "Auto categorization completed: {} categories created",
            output.categories.len()
        );

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::MemoryItem;

    #[test]
    fn test_categorize_preference() {
        let categorizer = AutoCategorizer::new();

        let item = MemoryItem::new(
            "User prefers Rust programming language".to_string(),
            "preference".to_string(),
        );

        let category = categorizer.categorize_item(&item);

        assert_eq!(category, "/preferences/programming");
    }

    #[test]
    fn test_categorize_fact() {
        let categorizer = AutoCategorizer::new();

        // Test with "software" keyword to trigger technology category
        let item = MemoryItem::new(
            "Rust is a software programming language".to_string(),
            "fact".to_string(),
        );

        let category = categorizer.categorize_item(&item);

        assert_eq!(category, "/knowledge/technology");
    }

    #[test]
    fn test_categorize_skill() {
        let categorizer = AutoCategorizer::new();

        let item = MemoryItem::new("User can write clean code".to_string(), "skill".to_string());

        let category = categorizer.categorize_item(&item);

        assert_eq!(category, "/skills/programming");
    }

    #[test]
    fn test_categorize_event() {
        let categorizer = AutoCategorizer::new();

        let item = MemoryItem::new(
            "Yesterday, user completed a project".to_string(),
            "event".to_string(),
        );

        let category = categorizer.categorize_item(&item);

        assert_eq!(category, "/timeline/events");
    }

    #[test]
    fn test_stage_priority() {
        let categorizer = AutoCategorizer::new();
        assert_eq!(categorizer.priority(), StagePriority::NORMAL);
    }

    #[test]
    fn test_stage_name() {
        let categorizer = AutoCategorizer::new();
        assert_eq!(categorizer.name(), "AutoCategorizer");
    }
}
