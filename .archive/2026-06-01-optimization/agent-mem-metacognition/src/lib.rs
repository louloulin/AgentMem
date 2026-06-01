//! AgentMem Metacognition and Auto-Consolidation
//!
//! This crate provides:
//! - Automatic memory consolidation triggers
//! - Merge history tracking
//! - Metacognitive statistics
//! - Intelligent recommendations
//!
//! # Features
//!
//! - **Auto-Consolidation**: Automatically trigger memory consolidation based on thresholds
//! - **History Tracking**: Track all merge operations with full audit trail
//! - **Metacognition**: Monitor memory health and provide insights
//! - **Recommendations**: AI-powered suggestions for memory optimization
//!
//! # Example
//!
//! ```no_run
//! use agent_mem_metacognition::{
//!     MetacognitionConfig, MetacognitionService
//! };
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = MetacognitionConfig::default();
//!     let service = MetacognitionService::new(config).await?;
//!
//!     // Enable auto-consolidation
//!     service.start_auto_consolidation().await?;
//!
//!     // Get metacognitive report
//!     let report = service.generate_report().await?;
//!     println!("Memory health: {}", report.health_score);
//!
//!     Ok(())
//! }
//! ```

pub mod consolidation;
pub mod history;
pub mod metacognition;
pub mod recommendations;

pub use consolidation::{AutoConsolidationConfig, AutoConsolidationTrigger};
pub use history::{MergeHistory, MergeOperation, MergeTracker};
pub use metacognition::{MetacognitionConfig, MetacognitionReport, MetacognitionService};
pub use recommendations::{Recommendation, RecommendationEngine, RecommendationType};

// Default values
pub const DEFAULT_CONSOLIDATION_THRESHOLD: usize = 100;
pub const DEFAULT_AUTO_CONSOLIDATION_INTERVAL_SECONDS: u64 = 3600;
pub const DEFAULT_HEALTH_CHECK_INTERVAL_SECONDS: u64 = 1800;
