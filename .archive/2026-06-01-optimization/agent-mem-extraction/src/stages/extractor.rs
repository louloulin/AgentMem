//! Stage 3: Item Extractor
//!
//! Extracts memory items from preprocessed content

use crate::error::{ExtractionError, Result};
use crate::models::{ExtractionContext, ExtractionInput, ExtractionOutput, MemoryItem};
use crate::stage::{ExtractionStage, StagePriority};
use async_trait::async_trait;
use tracing::{debug, info};

/// Stage 3: Item Extractor
///
/// This stage:
/// - Extracts memory items from text content
/// - Identifies facts, preferences, events, skills
/// - Assigns confidence scores
pub struct ItemExtractor;

impl ItemExtractor {
    /// Create new item extractor
    pub fn new() -> Self {
        Self
    }

    /// Extract items from text
    fn extract_from_text(&self, text: &str) -> Vec<MemoryItem> {
        let mut items = Vec::new();

        // Simple extraction logic (in production, use LLM)
        let lines: Vec<&str> = text.lines().collect();

        for line in lines.iter() {
            let line = line.trim();

            // Skip empty lines
            if line.is_empty() {
                continue;
            }

            // Extract facts (sentences with periods)
            if line.contains('.') && line.len() > 10 {
                items.push(MemoryItem::new(line.to_string(), "fact".to_string()));
            }

            // Extract preferences (sentences with "prefer", "like", "want")
            if line.contains("prefer") || line.contains("like") || line.contains("want") {
                items.push(
                    MemoryItem::new(line.to_string(), "preference".to_string())
                        .with_confidence(0.8),
                );
            }

            // Extract events (sentences with time references)
            if line.contains("yesterday") || line.contains("today") || line.contains("tomorrow") {
                items.push(
                    MemoryItem::new(line.to_string(), "event".to_string()).with_confidence(0.7),
                );
            }

            // Extract skills (sentences with "can", "able to", "know how")
            if line.contains("can ") || line.contains("able to") || line.contains("know how") {
                items.push(
                    MemoryItem::new(line.to_string(), "skill".to_string()).with_confidence(0.75),
                );
            }
        }

        info!("Extracted {} items from {} lines", items.len(), lines.len());

        items
    }

    /// Extract items from JSON
    fn extract_from_json(&self, json: &serde_json::Value) -> Vec<MemoryItem> {
        let mut items = Vec::new();

        // Extract key-value pairs as memory items
        if let Some(obj) = json.as_object() {
            for (key, value) in obj {
                if let Some(str_value) = value.as_str() {
                    let content = format!("{}: {}", key, str_value);
                    items.push(MemoryItem::new(content, "fact".to_string()));
                }
            }
        }

        items
    }
}

impl Default for ItemExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ExtractionStage for ItemExtractor {
    fn name(&self) -> &str {
        "ItemExtractor"
    }

    fn priority(&self) -> StagePriority {
        StagePriority::HIGH
    }

    async fn process(
        &self,
        _input: ExtractionInput,
        mut output: ExtractionOutput,
        context: &mut ExtractionContext,
    ) -> Result<ExtractionOutput> {
        debug!("ItemExtractor processing");

        // Get preprocessed content from context
        let preprocessed = context.get_state("preprocessed_content").ok_or_else(|| {
            ExtractionError::ConfigurationError("Preprocessed content not found".to_string())
        })?;

        // Extract items based on content
        let items = self.extract_from_text(preprocessed);

        // Add items to output
        for item in items {
            output.items.push(item);
        }

        info!(
            "Item extraction completed: {} items extracted",
            output.items.len()
        );

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_items() {
        let extractor = ItemExtractor::new();

        let text = r#"
            User prefers Rust programming language.
            yesterday, user completed a project.
            User can write clean code.
            This is a simple fact.
        "#;

        let items = extractor.extract_from_text(text);

        // The current implementation creates multiple items for lines matching multiple patterns
        // Each line with "." is a fact, plus lines matching preference/event/skill keywords
        assert!(items.len() >= 4); // At least 4 items expected

        // Check that we have all types (lowercase "yesterday" triggers event)
        let types: std::collections::HashSet<_> =
            items.iter().map(|i| i.item_type.as_str()).collect();
        assert!(types.contains("preference"));
        assert!(types.contains("event"));
        assert!(types.contains("skill"));
        assert!(types.contains("fact"));
    }

    #[test]
    fn test_stage_priority() {
        let extractor = ItemExtractor::new();
        assert_eq!(extractor.priority(), StagePriority::HIGH);
    }

    #[test]
    fn test_stage_name() {
        let extractor = ItemExtractor::new();
        assert_eq!(extractor.name(), "ItemExtractor");
    }
}
