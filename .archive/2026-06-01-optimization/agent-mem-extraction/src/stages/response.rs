//! Stage 7: Response Builder
//!
//! Builds final response with extracted items

use crate::error::Result;
use crate::models::{ExtractionContext, ExtractionInput, ExtractionOutput};
use crate::stage::{ExtractionStage, StagePriority};
use async_trait::async_trait;
use tracing::{debug, info};

/// Stage 7: Response Builder
///
/// This stage:
/// - Builds final response with all extracted data
/// - Validates output completeness
/// - Adds summary and metadata
pub struct ResponseBuilder;

impl ResponseBuilder {
    /// Create new response builder
    pub fn new() -> Self {
        Self
    }

    /// Validate output completeness
    fn validate_output(&self, output: &ExtractionOutput) -> Result<()> {
        if output.items.is_empty() {
            debug!("No items extracted, but this is not necessarily an error");
        }

        // Validate metrics
        if output.metrics.total_duration_ms == 0 {
            debug!("Warning: Total duration is 0ms");
        }

        Ok(())
    }

    /// Generate summary
    fn generate_summary(&self, output: &ExtractionOutput) -> String {
        format!(
            "Extraction completed: {} items, {} categories, {} resources, {} warnings",
            output.items.len(),
            output.categories.len(),
            output.resources.len(),
            output.warnings.len()
        )
    }
}

impl Default for ResponseBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ExtractionStage for ResponseBuilder {
    fn name(&self) -> &str {
        "ResponseBuilder"
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
        debug!("ResponseBuilder processing");

        // Validate output
        self.validate_output(&output)?;

        // Generate summary
        let summary = self.generate_summary(&output);

        info!("{}", summary);

        // Add summary to warnings if there are any
        if !output.warnings.is_empty() {
            output.warnings.push(summary);
        }

        info!(
            "Response building completed: {} items in {}ms",
            output.items.len(),
            output.metrics.total_duration_ms
        );

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_summary() {
        let builder = ResponseBuilder::new();

        let output = ExtractionOutput::new(crate::models::ExtractionId::new());

        let summary = builder.generate_summary(&output);

        assert!(summary.contains("0 items"));
        assert!(summary.contains("0 categories"));
        assert!(summary.contains("0 resources"));
    }

    #[test]
    fn test_stage_priority() {
        let builder = ResponseBuilder::new();
        assert_eq!(builder.priority(), StagePriority::NORMAL);
    }

    #[test]
    fn test_stage_name() {
        let builder = ResponseBuilder::new();
        assert_eq!(builder.name(), "ResponseBuilder");
    }
}
