//! Extraction stage trait definition

use crate::error::Result;
use crate::models::{ExtractionContext, ExtractionInput, ExtractionOutput};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Stage priority (higher = executed first)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct StagePriority(pub u8);

impl StagePriority {
    /// Critical priority (must execute first)
    pub const CRITICAL: StagePriority = StagePriority(100);

    /// High priority
    pub const HIGH: StagePriority = StagePriority(75);

    /// Normal priority
    pub const NORMAL: StagePriority = StagePriority(50);

    /// Low priority
    pub const LOW: StagePriority = StagePriority(25);

    /// Optional priority
    pub const OPTIONAL: StagePriority = StagePriority(10);
}

impl Default for StagePriority {
    fn default() -> Self {
        Self::NORMAL
    }
}

/// Extraction stage trait
///
/// Each stage in the pipeline implements this trait. Stages are executed
/// in order of priority (higher priority first), and each stage receives
/// the output from the previous stage.
#[async_trait]
pub trait ExtractionStage: Send + Sync {
    /// Get stage name
    fn name(&self) -> &str;

    /// Get stage priority (higher = executed first)
    fn priority(&self) -> StagePriority {
        StagePriority::NORMAL
    }

    /// Process the extraction
    ///
    /// # Arguments
    /// * `input` - Extraction input (from previous stage or initial)
    /// * `context` - Shared extraction context
    ///
    /// # Returns
    /// Modified extraction output to pass to next stage
    async fn process(
        &self,
        input: ExtractionInput,
        output: ExtractionOutput,
        context: &mut ExtractionContext,
    ) -> Result<ExtractionOutput>;

    /// Check if this stage should be skipped
    ///
    /// Override this to implement conditional execution
    fn should_skip(&self, _input: &ExtractionInput, _context: &ExtractionContext) -> bool {
        false
    }

    /// Validate stage configuration
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage_priority_ordering() {
        assert!(StagePriority::CRITICAL > StagePriority::HIGH);
        assert!(StagePriority::HIGH > StagePriority::NORMAL);
        assert!(StagePriority::NORMAL > StagePriority::LOW);
        assert!(StagePriority::LOW > StagePriority::OPTIONAL);
    }

    #[test]
    fn test_stage_priority_default() {
        let priority = StagePriority::default();
        assert_eq!(priority, StagePriority::NORMAL);
    }
}
