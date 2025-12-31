//! Plugin loader

use crate::types::*;
use anyhow::{anyhow, Result};
use extism::{Manifest, Plugin};
use std::fs;

/// Plugin loader
pub struct PluginLoader {}

impl PluginLoader {
    pub fn new() -> Self {
        Self {}
    }

    /// Load a plugin from file
    pub fn load_plugin(&self, plugin_info: &RegisteredPlugin) -> Result<LoadedPlugin> {
        // Read WASM file
        let wasm_bytes = fs::read(&plugin_info.path)
            .map_err(|e| anyhow!("Failed to read plugin file {}: {}", plugin_info.path, e))?;

        // Create Extism plugin
        let manifest = Manifest::new([wasm_bytes]);
        let plugin = Plugin::new(&manifest, [], true)
            .map_err(|e| anyhow!("Failed to create plugin: {e}"))?;

        // Return loaded plugin
        Ok(LoadedPlugin {
            id: plugin_info.id.clone(),
            metadata: plugin_info.metadata.clone(),
            plugin,
        })
    }

    /// Call plugin function
    pub fn call_plugin(plugin: &mut Plugin, function_name: &str, input: &str) -> Result<String> {
        plugin
            .call(function_name, input)
            .map_err(|e| anyhow!("Failed to call plugin function {function_name}: {e}"))
    }
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loader_creation() {
        let loader = PluginLoader::new();
        assert_eq!(std::mem::size_of_val(&loader), 0); // Zero-sized type
    }
}
