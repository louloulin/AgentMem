//! Plugin Integration for AgentMem
//!
//! This module provides deep integration of the plugin system into Memory operations.

#[cfg(feature = "plugins")]
use agent_mem_plugins::{PluginManager, PluginRegistry, RegisteredPlugin, PluginStatus};
#[cfg(feature = "plugins")]
use agent_mem_plugins::sdk::{PluginMetadata, PluginType, Capability, PluginConfig};
use agent_mem_traits::{MemoryItem, Result};

/// Plugin-enhanced Memory wrapper
///
/// Provides plugin hooks for memory operations
#[cfg(feature = "plugins")]
pub struct PluginEnhancedMemory {
    /// Plugin manager
    manager: PluginManager,
    
    /// Plugin registry
    registry: PluginRegistry,
}

#[cfg(feature = "plugins")]
impl PluginEnhancedMemory {
    /// Create new plugin-enhanced memory
    pub fn new() -> Self {
        Self {
            manager: PluginManager::new(10), // LRU cache size 10
            registry: PluginRegistry::new(),
        }
    }
    
    /// Register a plugin
    pub fn register_plugin(&mut self, plugin: RegisteredPlugin) -> Result<()> {
        self.registry
            .register(plugin)
            .map_err(|e| agent_mem_traits::AgentMemError::Other(e))
    }
    
    /// Process memory through plugins before storage
    ///
    /// This allows plugins to enhance, clean, or transform memory content
    pub async fn process_memory_with_plugins(
        &self,
        memory: &mut MemoryItem,
    ) -> Result<()> {
        // Find memory processor plugins
        let plugins = self.registry.list();
        
        for plugin_info in plugins {
            if matches!(plugin_info.metadata.plugin_type, PluginType::MemoryProcessor) {
                // TODO: Load and call plugin
                tracing::debug!(
                    "Would process memory with plugin: {}",
                    plugin_info.metadata.name
                );
            }
        }
        
        Ok(())
    }
    
    /// Search using custom search algorithm plugin
    pub async fn search_with_plugin(
        &self,
        _query: &str,
        _memories: &[MemoryItem],
    ) -> Result<Vec<MemoryItem>> {
        // TODO: Implement plugin-based search
        Ok(vec![])
    }
    
    /// Get plugin manager
    pub fn plugin_manager(&self) -> &PluginManager {
        &self.manager
    }
    
    /// Get plugin registry
    pub fn plugin_registry(&self) -> &PluginRegistry {
        &self.registry
    }
}

#[cfg(not(feature = "plugins"))]
pub struct PluginEnhancedMemory;

#[cfg(not(feature = "plugins"))]
impl PluginEnhancedMemory {
    pub fn new() -> Self {
        Self
    }
}

/// Plugin hooks for memory operations
pub trait PluginHooks {
    /// Called before memory is added
    fn before_add_memory(&self, _memory: &mut MemoryItem) -> Result<()> {
        Ok(())
    }
    
    /// Called after memory is added
    fn after_add_memory(&self, _memory: &MemoryItem) -> Result<()> {
        Ok(())
    }
    
    /// Called before search
    fn before_search(&self, _query: &str) -> Result<()> {
        Ok(())
    }
    
    /// Called after search, can modify results
    fn after_search(&self, _results: &mut Vec<MemoryItem>) -> Result<()> {
        Ok(())
    }
}

#[cfg(feature = "plugins")]
impl PluginHooks for PluginEnhancedMemory {
    fn before_add_memory(&self, memory: &mut MemoryItem) -> Result<()> {
        tracing::debug!("Plugin hook: before_add_memory");
        
        // Find and execute memory processor plugins
        let plugins = self.registry.list();
        for plugin_info in plugins {
            if matches!(
                plugin_info.metadata.plugin_type,
                PluginType::MemoryProcessor
            ) {
                tracing::debug!("Processing with plugin: {}", plugin_info.metadata.name);
                // TODO: Actually load and execute the plugin
                // This would call the WASM module with the memory data
            }
        }
        
        Ok(())
    }
    
    fn after_search(&self, results: &mut Vec<MemoryItem>) -> Result<()> {
        tracing::debug!("Plugin hook: after_search, {} results", results.len());
        
        // Find and execute search algorithm plugins for reranking
        let plugins = self.registry.list();
        for plugin_info in plugins {
            if matches!(
                plugin_info.metadata.plugin_type,
                PluginType::SearchAlgorithm
            ) {
                tracing::debug!("Reranking with plugin: {}", plugin_info.metadata.name);
                // TODO: Actually load and execute the search plugin
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_plugin_enhanced_memory_creation() {
        let _plugin_memory = PluginEnhancedMemory::new();
        // Should not panic
    }
    
    #[cfg(feature = "plugins")]
    #[test]
    fn test_plugin_registration() {
        use chrono::Utc;
        
        let mut plugin_memory = PluginEnhancedMemory::new();
        
        let metadata = PluginMetadata {
            name: "test-plugin".to_string(),
            version: "0.1.0".to_string(),
            description: "Test plugin".to_string(),
            author: "Test".to_string(),
            plugin_type: PluginType::MemoryProcessor,
            required_capabilities: vec![Capability::MemoryAccess],
            config_schema: None,
        };
        
        let plugin = RegisteredPlugin {
            id: "test-plugin".to_string(),
            metadata,
            path: "/tmp/test.wasm".to_string(),
            status: PluginStatus::Registered,
            config: PluginConfig::default(),
            registered_at: Utc::now(),
            last_loaded_at: None,
        };
        
        assert!(plugin_memory.register_plugin(plugin).is_ok());
    }
    
    #[cfg(feature = "plugins")]
    #[tokio::test]
    async fn test_plugin_hooks() {
        use agent_mem_traits::{MemoryType, Session};
        use std::collections::HashMap;
        
        let plugin_memory = PluginEnhancedMemory::new();
        
        let mut memory = MemoryItem {
            id: "test-1".to_string(),
            content: "Test content".to_string(),
            hash: None,
            metadata: HashMap::new(),
            score: None,
            created_at: chrono::Utc::now(),
            updated_at: None,
            session: Session {
                id: "test-session".to_string(),
                created_at: chrono::Utc::now(),
                user_id: Some("test-user".to_string()),
                agent_id: Some("test-agent".to_string()),
                actor_id: Some("test-actor".to_string()),
                run_id: Some("test-run".to_string()),
                metadata: HashMap::new(),
            },
            memory_type: MemoryType::Semantic,
            entities: vec![],
            relations: vec![],
            agent_id: "test-agent".to_string(),
            user_id: Some("test-user".to_string()),
            importance: 1.0,
            embedding: None,
            last_accessed_at: chrono::Utc::now(),
            access_count: 0,
            expires_at: None,
            version: 1,
        };
        
        // Test hooks
        assert!(plugin_memory.before_add_memory(&mut memory).is_ok());
        assert!(plugin_memory.after_add_memory(&memory).is_ok());
        assert!(plugin_memory.before_search("test query").is_ok());
        
        let mut results = vec![memory];
        assert!(plugin_memory.after_search(&mut results).is_ok());
    }
}

