//! Extraction Pipeline Framework for AgentMem
//!
//! This crate provides a flexible, multi-stage extraction pipeline that transforms
//! resources into structured memory items. The pipeline follows a 7-stage workflow:
//!
//! 1. **ResourceIngestor** - Mount and validate resources
//! 2. **MultimodalPreprocessor** - Preprocess text, images, audio, video
//! 3. **ItemExtractor** - Extract memory items from resources
//! 4. **DedupeMerger** - Remove duplicates and merge similar items
//! 5. **AutoCategorizer** - Automatically categorize memory items
//! 6. **IndexPersistor** - Persist items and update search indexes
//! 7. **ResponseBuilder** - Build response with extracted items
//!
//! # Architecture
//!
//! ```text
//! ExtractionPipeline
//!     ├── stages: Vec<Box<dyn ExtractionStage>>
//!     ├── execute(input) -> Output
//!     └── config: PipelineConfig
//!
//! ExtractionStage (trait)
//!     ├── process(input) -> Result<Output>
//!     ├── name() -> &str
//!     └── priority() -> u8
//!
//! 7 Standard Stages:
//!     ├── ResourceIngestor
//!     ├── MultimodalPreprocessor
//!     ├── ItemExtractor
//!     ├── DedupeMerger
//!     ├── AutoCategorizer
//!     ├── IndexPersistor
//!     └── ResponseBuilder
//! ```
//!
//! # Example
//!
//! ```no_run
//! use agent_mem_extraction::{ExtractionPipeline, stages::ResourceIngestor, stages::ItemExtractor, PipelineConfig, ExtractionInput};
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create pipeline with default config
//! let config = PipelineConfig::default();
//! let pipeline = ExtractionPipeline::new(config);
//!
//! // Note: In real usage, you would add stages and execute
//! // This is just a compile example
//! println!("Pipeline created with {} stages", pipeline.stage_names().len());
//! # Ok(())
//! # }
//! ```

pub mod error;
pub mod models;
pub mod pipeline;
pub mod stage;
pub mod stages;

// Re-exports for convenience
pub use error::{ExtractionError, Result};
pub use models::{
    ExecutionMode, ExtractionContext, ExtractionInput, ExtractionMetrics, ExtractionOutput,
    PipelineConfig,
};
pub use pipeline::ExtractionPipeline;
pub use stage::{ExtractionStage, StagePriority};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library name
pub const LIB_NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
        assert_eq!(LIB_NAME, "agent-mem-extraction");
    }
}
