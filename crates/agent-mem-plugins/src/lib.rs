//! AgentMem Plugin Manager
//!
//! This crate provides the plugin management system for AgentMem.

pub mod registry;
pub mod loader;
pub mod manager;
pub mod capabilities;
pub mod security;
pub mod types;

// Re-export commonly used types
pub use agent_mem_plugin_sdk as sdk;
pub use registry::*;
pub use loader::*;
pub use manager::*;
pub use types::*;

// Re-export extism for host development
pub use extism;

