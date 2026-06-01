//! AgentMem Engine
//! 
//! Core engine module extracted from agent-mem-core for better modularity.
//! This crate provides the core processing engine for AgentMem.

mod engine;
mod manager;
mod pipeline;

pub use engine::{MemoryEngine, MemoryEngineConfig};
pub use manager::{MemoryManager, MemoryStats, MemoryItem};
pub use pipeline::{create_engine_pipeline, EngineStage};
