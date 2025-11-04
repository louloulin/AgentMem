//! AgentMem Plugin Manager
//!
//! This crate provides the plugin management system for AgentMem.

pub mod capabilities;
pub mod loader;
pub mod manager;
pub mod registry;
pub mod security;
pub mod types;

// Re-export commonly used types
pub use agent_mem_plugin_sdk as sdk;
pub use loader::*;
pub use manager::*;
pub use registry::*;
pub use types::*;

// Re-export extism for host development
pub use extism;
