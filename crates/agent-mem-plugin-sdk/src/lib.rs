//! AgentMem Plugin SDK
//!
//! This SDK provides the core types and utilities for developing AgentMem plugins.

pub mod host;
pub mod macros;
pub mod plugin;
pub mod types;

// Re-export commonly used types
pub use host::*;
pub use plugin::*;
pub use types::*;

// Re-export extism_pdk for plugin development
pub use extism_pdk;
