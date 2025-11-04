//! Integration tests for plugin system

use agent_mem_plugin_sdk::{Capability, PluginConfig, PluginMetadata, PluginType};
use agent_mem_plugins::*;
use chrono::Utc;

#[tokio::test]
async fn test_plugin_manager_lifecycle() {
    // Create manager
    let manager = PluginManager::new(10);
    
    // Register a plugin
    let plugin = RegisteredPlugin {
        id: "test-plugin".to_string(),
        metadata: PluginMetadata {
            name: "Test Plugin".to_string(),
            version: "0.1.0".to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            plugin_type: PluginType::Custom("test".to_string()),
            required_capabilities: vec![Capability::LoggingAccess],
            config_schema: None,
        },
        path: "/path/to/plugin.wasm".to_string(),
        status: PluginStatus::Registered,
        config: PluginConfig::default(),
        registered_at: Utc::now(),
        last_loaded_at: None,
    };
    
    assert!(manager.register(plugin).await.is_ok());
    
    // List plugins
    let plugins = manager.list_plugins().await;
    assert_eq!(plugins.len(), 1);
    assert_eq!(plugins[0].id, "test-plugin");
    
    // Unregister
    assert!(manager.unregister("test-plugin").await.is_ok());
    assert_eq!(manager.list_plugins().await.len(), 0);
}

#[tokio::test]
async fn test_registry_operations() {
    let mut registry = PluginRegistry::new();
    
    let plugin = RegisteredPlugin {
        id: "test-1".to_string(),
        metadata: PluginMetadata {
            name: "Test 1".to_string(),
            version: "0.1.0".to_string(),
            description: "Test".to_string(),
            author: "Test".to_string(),
            plugin_type: PluginType::MemoryProcessor,
            required_capabilities: vec![],
            config_schema: None,
        },
        path: "/test.wasm".to_string(),
        status: PluginStatus::Registered,
        config: PluginConfig::default(),
        registered_at: Utc::now(),
        last_loaded_at: None,
    };
    
    // Register
    assert!(registry.register(plugin).is_ok());
    
    // Get
    assert!(registry.get("test-1").is_some());
    assert!(registry.get("nonexistent").is_none());
    
    // Update status
    assert!(registry
        .update_status("test-1", PluginStatus::Loading)
        .is_ok());
    let plugin = registry.get("test-1").unwrap();
    assert_eq!(plugin.status, PluginStatus::Loading);
    
    // List
    assert_eq!(registry.list().len(), 1);
    
    // Unregister
    assert!(registry.unregister("test-1").is_ok());
    assert!(registry.get("test-1").is_none());
}

#[test]
fn test_permission_checker() {
    use agent_mem_plugins::security::PermissionChecker;
    
    let checker = PermissionChecker::new(vec![
        Capability::MemoryAccess,
        Capability::LoggingAccess,
    ]);
    
    // Allowed capabilities
    assert!(checker.check(&Capability::MemoryAccess).is_ok());
    assert!(checker.check(&Capability::LoggingAccess).is_ok());
    
    // Denied capabilities
    assert!(checker.check(&Capability::NetworkAccess).is_err());
    assert!(checker.check(&Capability::FileSystemAccess).is_err());
    
    // Check all
    assert!(checker
        .check_all(&[Capability::MemoryAccess, Capability::LoggingAccess])
        .is_ok());
    assert!(checker
        .check_all(&[Capability::MemoryAccess, Capability::NetworkAccess])
        .is_err());
}

#[test]
fn test_sandbox_config() {
    use agent_mem_plugins::security::SandboxConfig;
    
    // Default config
    let config = SandboxConfig::default();
    assert_eq!(config.max_memory_bytes, 100 * 1024 * 1024);
    assert_eq!(config.max_execution_time_ms, 5000);
    assert!(!config.allow_network);
    assert!(!config.allow_filesystem);
    
    // Custom config
    let config = SandboxConfig {
        max_memory_bytes: 50 * 1024 * 1024,
        max_execution_time_ms: 3000,
        allowed_capabilities: vec![Capability::MemoryAccess],
        allow_network: true,
        allow_filesystem: false,
    };
    
    assert_eq!(config.max_memory_bytes, 50 * 1024 * 1024);
    assert!(config.allow_network);
}

