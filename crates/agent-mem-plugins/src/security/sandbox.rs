//! Sandbox configuration for plugins

use agent_mem_plugin_sdk::Capability;

/// Sandbox configuration
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Maximum memory in bytes
    pub max_memory_bytes: usize,

    /// Maximum execution time in milliseconds
    pub max_execution_time_ms: u64,

    /// Allowed capabilities
    pub allowed_capabilities: Vec<Capability>,

    /// Allow network access
    pub allow_network: bool,

    /// Allow filesystem access
    pub allow_filesystem: bool,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            max_memory_bytes: 100 * 1024 * 1024, // 100 MB
            max_execution_time_ms: 5000,         // 5 seconds
            allowed_capabilities: vec![Capability::MemoryAccess, Capability::LoggingAccess],
            allow_network: false,
            allow_filesystem: false,
        }
    }
}
