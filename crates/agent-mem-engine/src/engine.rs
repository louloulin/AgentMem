//! Memory Engine - Core orchestration module
//! 
//! Provides the core MemoryEngine for AgentMem operations.

use agent_mem_types::{Result, MemoryType};
use serde::{Deserialize, Serialize};

/// Memory engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEngineConfig {
    /// Enable auto processing
    pub auto_processing: bool,
    /// Processing interval in seconds
    pub processing_interval_seconds: u64,
    /// Maximum batch size
    pub max_batch_size: usize,
    /// Enable enhanced search
    pub enable_enhanced_search: bool,
}

impl Default for MemoryEngineConfig {
    fn default() -> Self {
        Self {
            auto_processing: true,
            processing_interval_seconds: 300,
            max_batch_size: 100,
            enable_enhanced_search: false,
        }
    }
}

/// MemoryEngine is the core orchestration engine for AgentMem
/// 
/// This provides the main interface for memory operations including:
/// - Add, get, update, delete memory items
/// - Search across memory types
/// - Hierarchy management
/// - Importance scoring
/// - Conflict resolution
pub struct MemoryEngine {
    config: MemoryEngineConfig,
}

impl MemoryEngine {
    /// Create new engine with default config
    pub fn new() -> Self {
        Self {
            config: MemoryEngineConfig::default(),
        }
    }

    /// Create new engine with custom config
    pub fn with_config(config: MemoryEngineConfig) -> Self {
        Self { config }
    }

    /// Get configuration
    pub fn config(&self) -> &MemoryEngineConfig {
        &self.config
    }

    /// Check if auto processing is enabled
    pub fn is_auto_processing_enabled(&self) -> bool {
        self.config.auto_processing
    }

    /// Get processing interval
    pub fn processing_interval(&self) -> u64 {
        self.config.processing_interval_seconds
    }

    /// Get max batch size
    pub fn max_batch_size(&self) -> usize {
        self.config.max_batch_size
    }

    /// Check if enhanced search is enabled
    pub fn is_enhanced_search_enabled(&self) -> bool {
        self.config.enable_enhanced_search
    }
}

impl Default for MemoryEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = MemoryEngine::new();
        assert!(engine.is_auto_processing_enabled());
        assert_eq!(engine.processing_interval(), 300);
    }

    #[test]
    fn test_engine_config() {
        let config = MemoryEngineConfig {
            auto_processing: false,
            processing_interval_seconds: 600,
            max_batch_size: 50,
            enable_enhanced_search: true,
        };
        let engine = MemoryEngine::with_config(config);
        assert!(!engine.is_auto_processing_enabled());
        assert!(engine.is_enhanced_search_enabled());
    }
}
