//! Plugin registry

use crate::types::*;
use anyhow::{anyhow, Result};
use std::collections::HashMap;

/// Plugin registry
pub struct PluginRegistry {
    plugins: HashMap<String, RegisteredPlugin>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }
    
    /// Register a plugin
    pub fn register(&mut self, plugin: RegisteredPlugin) -> Result<()> {
        if self.plugins.contains_key(&plugin.id) {
            return Err(anyhow!("Plugin already registered: {}", plugin.id));
        }
        
        self.plugins.insert(plugin.id.clone(), plugin);
        Ok(())
    }
    
    /// Get a plugin
    pub fn get(&self, id: &str) -> Option<&RegisteredPlugin> {
        self.plugins.get(id)
    }
    
    /// Get a mutable reference to a plugin
    pub fn get_mut(&mut self, id: &str) -> Option<&mut RegisteredPlugin> {
        self.plugins.get_mut(id)
    }
    
    /// List all plugins
    pub fn list(&self) -> Vec<&RegisteredPlugin> {
        self.plugins.values().collect()
    }
    
    /// Update plugin status
    pub fn update_status(&mut self, id: &str, status: PluginStatus) -> Result<()> {
        let plugin = self
            .plugins
            .get_mut(id)
            .ok_or_else(|| anyhow!("Plugin not found: {}", id))?;
        
        plugin.status = status;
        Ok(())
    }
    
    /// Unregister a plugin
    pub fn unregister(&mut self, id: &str) -> Result<RegisteredPlugin> {
        self.plugins
            .remove(id)
            .ok_or_else(|| anyhow!("Plugin not found: {}", id))
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_mem_plugin_sdk::{Capability, PluginMetadata, PluginType};
    use chrono::Utc;
    
    fn create_test_plugin() -> RegisteredPlugin {
        RegisteredPlugin {
            id: "test-plugin".to_string(),
            metadata: PluginMetadata {
                name: "Test Plugin".to_string(),
                version: "0.1.0".to_string(),
                description: "Test".to_string(),
                author: "Test".to_string(),
                plugin_type: PluginType::Custom("test".to_string()),
                required_capabilities: vec![Capability::LoggingAccess],
                config_schema: None,
            },
            path: "/path/to/plugin.wasm".to_string(),
            status: PluginStatus::Registered,
            config: agent_mem_plugin_sdk::PluginConfig::default(),
            registered_at: Utc::now(),
            last_loaded_at: None,
        }
    }
    
    #[test]
    fn test_register_plugin() {
        let mut registry = PluginRegistry::new();
        let plugin = create_test_plugin();
        
        assert!(registry.register(plugin).is_ok());
        assert!(registry.get("test-plugin").is_some());
    }
    
    #[test]
    fn test_register_duplicate() {
        let mut registry = PluginRegistry::new();
        let plugin1 = create_test_plugin();
        let plugin2 = create_test_plugin();
        
        assert!(registry.register(plugin1).is_ok());
        assert!(registry.register(plugin2).is_err());
    }
    
    #[test]
    fn test_update_status() {
        let mut registry = PluginRegistry::new();
        let plugin = create_test_plugin();
        
        registry.register(plugin).unwrap();
        assert!(registry.update_status("test-plugin", PluginStatus::Loading).is_ok());
        
        let plugin = registry.get("test-plugin").unwrap();
        assert_eq!(plugin.status, PluginStatus::Loading);
    }
    
    #[test]
    fn test_unregister() {
        let mut registry = PluginRegistry::new();
        let plugin = create_test_plugin();
        
        registry.register(plugin).unwrap();
        assert!(registry.unregister("test-plugin").is_ok());
        assert!(registry.get("test-plugin").is_none());
    }
}

