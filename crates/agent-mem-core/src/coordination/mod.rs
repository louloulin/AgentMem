//! Multi-Agent Coordination Module
//!
//! This module provides the coordination infrastructure for AgentMem's multi-agent architecture,
//! including the MetaMemoryManager coordinator and inter-agent communication protocols.
//!
//! # Architecture Overview
//!
//! The coordination system is inspired by MIRIX's multi-agent architecture but adapted for
//! Rust's performance and safety characteristics:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                MetaMemoryManager                            │
//! │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐ │
//! │  │  Task Router    │  │ Load Balancer   │  │ Fault Det.  │ │
//! │  └─────────────────┘  └─────────────────┘  └─────────────┘ │
//! └─────────────────────────────────────────────────────────────┘
//!                                │
//!                                ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Agent Network                            │
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
//! │  │ EpisodicAgent│  │SemanticAgent│  │ProceduralAgent│  ...   │
//! │  └─────────────┘  └─────────────┘  └─────────────┘         │
//! └─────────────────────────────────────────────────────────────┘
//! ```

pub mod meta_manager;

#[cfg(test)]
mod tests;

// Re-export main types
pub use meta_manager::{
    AgentMessage, AgentStatus, CoordinationError, CoordinationResult, CoordinationStats,
    LoadBalancingStrategy, MessageType, MetaMemoryConfig, MetaMemoryManager, TaskRequest,
    TaskResponse,
};
