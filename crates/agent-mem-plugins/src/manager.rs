//! Plugin manager

use crate::{loader::PluginLoader, registry::PluginRegistry, types::*};
use anyhow::{anyhow, Result};
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

/// Plugin manager
pub struct PluginManager {
    registry: Arc<RwLock<PluginRegistry>>,
    loader: Arc<PluginLoader>,
    plugin_cache: Arc<Mutex<LruCache<String, Arc<Mutex<LoadedPlugin>>>>>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new(cache_size: usize) -> Self {
        Self {
            registry: Arc::new(RwLock::new(PluginRegistry::new())),
            loader: Arc::new(PluginLoader::new()),
            plugin_cache: Arc::new(Mutex::new(LruCache::new(
                NonZeroUsize::new(cache_size).unwrap(),
            ))),
        }
    }

    /// Register a plugin
    pub async fn register(&self, plugin: RegisteredPlugin) -> Result<()> {
        let mut registry = self.registry.write().await;
        registry.register(plugin)
    }

    /// Get plugin (with caching)
    pub async fn get_plugin(&self, plugin_id: &str) -> Result<Arc<Mutex<LoadedPlugin>>> {
        // Try to get from cache
        {
            let mut cache = self.plugin_cache.lock().await;
            if let Some(plugin) = cache.get(plugin_id) {
                return Ok(plugin.clone());
            }
        }

        // Load plugin
        let plugin_info = {
            let registry = self.registry.read().await;
            registry
                .get(plugin_id)
                .cloned()
                .ok_or_else(|| anyhow!("Plugin not found: {}", plugin_id))?
        };

        let loaded_plugin = self.loader.load_plugin(&plugin_info)?;
        let loaded_plugin = Arc::new(Mutex::new(loaded_plugin));

        // Update status
        {
            let mut registry = self.registry.write().await;
            registry.update_status(plugin_id, PluginStatus::Loaded)?;
        }

        // Cache it
        {
            let mut cache = self.plugin_cache.lock().await;
            cache.put(plugin_id.to_string(), loaded_plugin.clone());
        }

        Ok(loaded_plugin)
    }

    /// Call plugin function
    pub async fn call_plugin(
        &self,
        plugin_id: &str,
        function_name: &str,
        input: &str,
    ) -> Result<String> {
        let plugin = self.get_plugin(plugin_id).await?;
        let mut plugin = plugin.lock().await;

        PluginLoader::call_plugin(&mut plugin.plugin, function_name, input)
    }

    /// List all plugins
    pub async fn list_plugins(&self) -> Vec<RegisteredPlugin> {
        let registry = self.registry.read().await;
        registry.list().into_iter().cloned().collect()
    }

    /// Unregister a plugin
    pub async fn unregister(&self, plugin_id: &str) -> Result<()> {
        // Remove from cache
        {
            let mut cache = self.plugin_cache.lock().await;
            cache.pop(plugin_id);
        }

        // Unregister
        let mut registry = self.registry.write().await;
        registry.unregister(plugin_id)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_mem_plugin_sdk::{Capability, PluginConfig, PluginMetadata, PluginType};
    use chrono::Utc;

    #[tokio::test]
    async fn test_manager_creation() {
        let manager = PluginManager::new(10);
        let plugins = manager.list_plugins().await;
        assert_eq!(plugins.len(), 0);
    }

    #[tokio::test]
    async fn test_register_plugin() {
        let manager = PluginManager::new(10);

        let plugin = RegisteredPlugin {
            id: "test-plugin".to_string(),
            metadata: PluginMetadata {
                name: "Test".to_string(),
                version: "0.1.0".to_string(),
                description: "Test plugin".to_string(),
                author: "Test".to_string(),
                plugin_type: PluginType::Custom("test".to_string()),
                required_capabilities: vec![Capability::LoggingAccess],
                config_schema: None,
            },
            path: "/nonexistent/plugin.wasm".to_string(),
            status: PluginStatus::Registered,
            config: PluginConfig::default(),
            registered_at: Utc::now(),
            last_loaded_at: None,
        };

        assert!(manager.register(plugin).await.is_ok());

        let plugins = manager.list_plugins().await;
        assert_eq!(plugins.len(), 1);
    }
}
