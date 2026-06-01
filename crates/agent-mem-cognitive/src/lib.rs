//! AgentMem Cognitive Memory Module
//! 
//! Implements the 8 types of cognitive memories for AgentMem:
//! - Episodic: Event-based memories with temporal context
//! - Semantic: Factual knowledge and concepts
//! - Procedural: Skills and how-to knowledge
//! - Working: Short-term active processing
//! - Core: Persistent identity and preferences
//! - Resource: Multimedia and external resources
//! - Knowledge: Structured knowledge graphs
//! - Contextual: Environment-aware information
//! 
//! Also includes:
//! - Forgetting curve (Ebbinghaus)
//! - Memory consolidation and fusion
//! - Memory hierarchy (层级记忆管理)

mod episodic;
mod semantic;
mod procedural;
mod working;
mod core;
mod resource;
mod knowledge;
mod contextual;
mod types;
mod forgetting;
mod consolidation;
mod hierarchy;  // 🆕 层级记忆管理

pub use types::*;
pub use episodic::*;
pub use semantic::*;
pub use procedural::*;
pub use working::*;
pub use core::*;
pub use resource::*;
pub use knowledge::*;
pub use contextual::*;
pub use forgetting::{ForgettingCurve, DecayStatus};
pub use consolidation::{ConsolidationEngine, MemoryFusion};
pub use hierarchy::{MemoryTier, TieredMemoryItem, MemoryHierarchy, MemoryHierarchyStats};
