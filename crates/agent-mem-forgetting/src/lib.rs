//! AgentMem Forgetting Mechanism
//!
//! Memory forgetting system based on cognitive science:
//! - Ebbinghaus forgetting curve
//! - Automatic memory cleanup scheduler
//! - Memory protection levels
//!
//! # Features
//!
//! - Ebbinghaus forgetting curve implementation
//! - Automatic forgetting check scheduler
//! - Memory protection mechanism (ProtectionLevel)
//! - EventBus integration for forget events
//!
//! # Example
//!
//! ```no_run
//! use agent_mem_forgetting::{ForgettingConfig, ForgettingScheduler};
//! use agent_mem_forgetting::protection::ProtectionLevel;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = ForgettingConfig::default()
//!         .with_check_interval(3600); // Check every hour
//!
//!     let scheduler = ForgettingScheduler::new(config).await?;
//!
//!     // Start automatic forgetting
//!     scheduler.start().await?;
//!
//!     Ok(())
//! }
//! ```

pub mod curve;
pub mod protection;
pub mod scheduler;

pub use curve::{EbbinghausCurve, ForgettingCurve};
pub use protection::{MemoryProtection, ProtectionLevel};
pub use scheduler::{ForgettingConfig, ForgettingScheduler};

use agent_mem_traits::Result;

/// Default check interval (1 hour)
pub const DEFAULT_CHECK_INTERVAL_SECONDS: u64 = 3600;

/// Default forgetting threshold (retention rate < 10%)
pub const DEFAULT_FORGETTING_THRESHOLD: f64 = 0.1;
