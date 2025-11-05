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
    /// Plugin manager (handles loading and caching)
    manager: std::sync::Arc<PluginManager>,
}

#[cfg(feature = "plugins")]
impl PluginEnhancedMemory {
    /// Create new plugin-enhanced memory
    pub fn new() -> Self {
        Self {
            manager: std::sync::Arc::new(PluginManager::new(10)), // LRU cache size 10
        }
    }
    
    /// Register a plugin
    pub async fn register_plugin(&mut self, plugin: RegisteredPlugin) -> Result<()> {
        self.manager.register(plugin).await
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
        let plugins = self.manager.list_plugins().await;
        
        for plugin_info in plugins {
            if matches!(plugin_info.metadata.plugin_type, PluginType::MemoryProcessor) {
                tracing::debug!(
                    "Processing memory with plugin: {}",
                    plugin_info.metadata.name
                );
                
                // Serialize memory to JSON
                let input = serde_json::to_string(memory)
                    .map_err(|e| agent_mem_traits::AgentMemError::Other(e.into()))?;
                
                // Call plugin
                match self.manager.call_plugin(&plugin_info.id, "process_memory", &input).await {
                    Ok(output) => {
                        // Try to deserialize the processed memory
                        match serde_json::from_str::<MemoryItem>(&output) {
                            Ok(processed_memory) => {
                                *memory = processed_memory;
                                tracing::debug!("Memory processed successfully by plugin: {}", plugin_info.metadata.name);
                            }
                            Err(e) => {
                                tracing::warn!("Plugin {} returned invalid memory format: {}", plugin_info.metadata.name, e);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Plugin {} failed to process memory: {}", plugin_info.metadata.name, e);
                        // Continue with other plugins
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Search using custom search algorithm plugin
    pub async fn search_with_plugin(
        &self,
        query: &str,
        memories: &[MemoryItem],
    ) -> Result<Vec<MemoryItem>> {
        // Find search algorithm plugins
        let plugins = self.manager.list_plugins().await;
        
        for plugin_info in plugins {
            if matches!(plugin_info.metadata.plugin_type, PluginType::SearchAlgorithm) {
                tracing::debug!(
                    "Searching with plugin: {}",
                    plugin_info.metadata.name
                );
                
                // Prepare search request
                let request = serde_json::json!({
                    "query": query,
                    "memories": memories,
                });
                
                let input = serde_json::to_string(&request)
                    .map_err(|e| agent_mem_traits::AgentMemError::Other(e.into()))?;
                
                // Call plugin
                match self.manager.call_plugin(&plugin_info.id, "search", &input).await {
                    Ok(output) => {
                        // Try to deserialize the search results
                        match serde_json::from_str::<Vec<MemoryItem>>(&output) {
                            Ok(results) => {
                                tracing::debug!("Search completed by plugin: {}, found {} results", 
                                    plugin_info.metadata.name, results.len());
                                return Ok(results);
                            }
                            Err(e) => {
                                tracing::warn!("Plugin {} returned invalid search results: {}", 
                                    plugin_info.metadata.name, e);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Plugin {} failed to search: {}", plugin_info.metadata.name, e);
                        // Try next plugin
                    }
                }
            }
        }
        
        // If no plugin succeeded, return original memories
        Ok(memories.to_vec())
    }
    
    /// Get plugin manager
    pub fn plugin_manager(&self) -> &PluginManager {
        &self.manager
    }
    
    /// List all registered plugins
    pub async fn list_plugins(&self) -> Vec<RegisteredPlugin> {
        self.manager.list_plugins().await
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
#[async_trait::async_trait]
pub trait PluginHooks {
    /// Called before memory is added
    async fn before_add_memory(&self, _memory: &mut MemoryItem) -> Result<()> {
        Ok(())
    }
    
    /// Called after memory is added
    async fn after_add_memory(&self, _memory: &MemoryItem) -> Result<()> {
        Ok(())
    }
    
    /// Called before search
    async fn before_search(&self, _query: &str) -> Result<()> {
        Ok(())
    }
    
    /// Called after search, can modify results
    async fn after_search(&self, _results: &mut Vec<MemoryItem>) -> Result<()> {
        Ok(())
    }
}

#[cfg(feature = "plugins")]
#[async_trait::async_trait]
impl PluginHooks for PluginEnhancedMemory {
    async fn before_add_memory(&self, memory: &mut MemoryItem) -> Result<()> {
        tracing::debug!("Plugin hook: before_add_memory");
        
        // Process memory through plugins
        let manager = self.manager.clone();
        let mut mem = memory.clone();
        
        // Find and execute memory processor plugins
        let plugins = manager.list_plugins().await;
                for plugin_info in plugins {
                    if matches!(
                        plugin_info.metadata.plugin_type,
                        PluginType::MemoryProcessor
                    ) {
                        tracing::debug!("Processing with plugin: {}", plugin_info.metadata.name);
                        
                        // Serialize memory to JSON
                        let input = match serde_json::to_string(&mem) {
                            Ok(json) => json,
                            Err(e) => {
                                tracing::warn!("Failed to serialize memory: {}", e);
                                continue;
                            }
                        };
                        
                    // Call plugin
                    match manager.call_plugin(&plugin_info.id, "process_memory", &input).await {
                        Ok(output) => {
                            // Try to deserialize the processed memory
                            match serde_json::from_str::<MemoryItem>(&output) {
                                Ok(processed_memory) => {
                                    mem = processed_memory;
                                    tracing::debug!("Memory processed successfully by plugin: {}", plugin_info.metadata.name);
                                }
                                Err(e) => {
                                    tracing::warn!("Plugin {} returned invalid memory format: {}", plugin_info.metadata.name, e);
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Plugin {} failed to process memory: {}", plugin_info.metadata.name, e);
                        }
                    }
                }
            }
        
        *memory = mem;
        Ok(())
    }
    
    async fn after_search(&self, results: &mut Vec<MemoryItem>) -> Result<()> {
        tracing::debug!("Plugin hook: after_search, {} results", results.len());
        
        if results.is_empty() {
            return Ok(());
        }
        
        // Find and execute search algorithm plugins for reranking
        let manager = self.manager.clone();
        let mut reranked_results = results.clone();
        
        let plugins = manager.list_plugins().await;
                for plugin_info in plugins {
                    if matches!(
                        plugin_info.metadata.plugin_type,
                        PluginType::SearchAlgorithm
                    ) {
                        tracing::debug!("Reranking with plugin: {}", plugin_info.metadata.name);
                        
                        // Prepare rerank request
                        let request = serde_json::json!({
                            "results": reranked_results,
                        });
                        
                        let input = match serde_json::to_string(&request) {
                            Ok(json) => json,
                            Err(e) => {
                                tracing::warn!("Failed to serialize search results: {}", e);
                                continue;
                            }
                        };
                        
                    // Call plugin
                    match manager.call_plugin(&plugin_info.id, "rerank", &input).await {
                        Ok(output) => {
                            // Try to deserialize the reranked results
                            match serde_json::from_str::<Vec<MemoryItem>>(&output) {
                                Ok(reranked) => {
                                    reranked_results = reranked;
                                    tracing::debug!("Results reranked by plugin: {}", plugin_info.metadata.name);
                                }
                                Err(e) => {
                                    tracing::warn!("Plugin {} returned invalid rerank format: {}", plugin_info.metadata.name, e);
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Plugin {} failed to rerank: {}", plugin_info.metadata.name, e);
                        }
                    }
                }
            }
        
        *results = reranked_results;
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
        
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            assert!(plugin_memory.register_plugin(plugin).await.is_ok());
        });
    }
    
    #[cfg(feature = "plugins")]
    #[tokio::test]
    async fn test_plugin_hooks() {
        use agent_mem_traits::{MemoryType, Session};
        use std::collections::HashMap;
        use crate::PluginHooks;
        
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
        assert!(plugin_memory.before_add_memory(&mut memory).await.is_ok());
        assert!(plugin_memory.after_add_memory(&memory).await.is_ok());
        assert!(plugin_memory.before_search("test query").await.is_ok());
        
        let mut results = vec![memory];
        assert!(plugin_memory.after_search(&mut results).await.is_ok());
    }
}

