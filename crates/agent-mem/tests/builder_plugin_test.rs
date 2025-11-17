//! Tests for Builder plugin integration

#[cfg(feature = "plugins")]
use agent_mem::plugins::sdk::{Capability, PluginConfig, PluginMetadata, PluginType};
#[cfg(feature = "plugins")]
use agent_mem::plugins::{PluginStatus, RegisteredPlugin};
#[cfg(feature = "plugins")]
use agent_mem::Memory;
#[cfg(feature = "plugins")]
use anyhow::Result;
#[cfg(feature = "plugins")]
use chrono::Utc;

#[cfg(feature = "plugins")]
#[tokio::test]
async fn test_builder_with_plugin() -> Result<()> {
    // Create a test plugin
    let metadata = PluginMetadata {
        name: "test-plugin".to_string(),
        version: "1.0.0".to_string(),
        description: "Test plugin".to_string(),
        author: "Test".to_string(),
        plugin_type: PluginType::SearchAlgorithm,
        required_capabilities: vec![Capability::SearchAccess],
        config_schema: None,
    };

    let plugin = RegisteredPlugin {
        id: "test-plugin".to_string(),
        metadata,
        path: "/tmp/test-plugin.wasm".to_string(),
        status: PluginStatus::Registered,
        config: PluginConfig::default(),
        registered_at: Utc::now(),
        last_loaded_at: None,
    };

    // Build Memory with plugin
    let mem = Memory::builder()
        .with_storage("memory://")
        .with_plugin(plugin)
        .build()
        .await?;

    // Verify plugin is registered
    let plugins = mem.list_plugins().await;
    assert_eq!(plugins.len(), 1);
    assert_eq!(plugins[0].name, "test-plugin");

    Ok(())
}

#[cfg(feature = "plugins")]
#[tokio::test]
async fn test_builder_with_multiple_plugins() -> Result<()> {
    let mut mem_builder = Memory::builder().with_storage("memory://");

    // Add 3 plugins via builder
    for i in 1..=3 {
        let metadata = PluginMetadata {
            name: format!("plugin-{}", i),
            version: "1.0.0".to_string(),
            description: format!("Test plugin {}", i),
            author: "Test".to_string(),
            plugin_type: PluginType::MemoryProcessor,
            required_capabilities: vec![Capability::MemoryAccess],
            config_schema: None,
        };

        let plugin = RegisteredPlugin {
            id: format!("plugin-{}", i),
            metadata,
            path: format!("/tmp/plugin-{}.wasm", i),
            status: PluginStatus::Registered,
            config: PluginConfig::default(),
            registered_at: Utc::now(),
            last_loaded_at: None,
        };

        mem_builder = mem_builder.with_plugin(plugin);
    }

    let mem = mem_builder.build().await?;

    // Verify all plugins are registered
    let plugins = mem.list_plugins().await;
    assert_eq!(plugins.len(), 3);

    // Verify plugin names
    let names: Vec<String> = plugins.iter().map(|p| p.name.clone()).collect();
    assert!(names.contains(&"plugin-1".to_string()));
    assert!(names.contains(&"plugin-2".to_string()));
    assert!(names.contains(&"plugin-3".to_string()));

    Ok(())
}

#[cfg(feature = "plugins")]
#[tokio::test]
async fn test_builder_with_plugin_and_config() -> Result<()> {
    // Create plugin with config
    let metadata = PluginMetadata {
        name: "configured-plugin".to_string(),
        version: "2.0.0".to_string(),
        description: "Plugin with config".to_string(),
        author: "Test".to_string(),
        plugin_type: PluginType::CodeAnalyzer,
        required_capabilities: vec![Capability::MemoryAccess, Capability::StorageAccess],
        config_schema: Some(serde_json::json!({
            "type": "object",
            "properties": {
                "max_depth": {"type": "number"}
            }
        })),
    };

    let mut config = PluginConfig::default();
    config
        .settings
        .insert("max_depth".to_string(), serde_json::json!(5));

    let plugin = RegisteredPlugin {
        id: "configured-plugin".to_string(),
        metadata,
        path: "/tmp/configured-plugin.wasm".to_string(),
        status: PluginStatus::Registered,
        config,
        registered_at: Utc::now(),
        last_loaded_at: None,
    };

    // Build with plugin
    let mem = Memory::builder()
        .with_storage("memory://")
        .with_plugin(plugin)
        .build()
        .await?;

    // Verify plugin
    let plugins = mem.list_plugins().await;
    assert_eq!(plugins.len(), 1);
    assert_eq!(plugins[0].name, "configured-plugin");
    assert_eq!(plugins[0].version, "2.0.0");
    assert!(plugins[0]
        .required_capabilities
        .contains(&Capability::MemoryAccess));
    assert!(plugins[0]
        .required_capabilities
        .contains(&Capability::StorageAccess));

    Ok(())
}

#[cfg(feature = "plugins")]
#[tokio::test]
async fn test_builder_load_plugins_from_nonexistent_dir() -> Result<()> {
    // Try to load from non-existent directory - should not fail
    let mem = Memory::builder()
        .with_storage("memory://")
        .load_plugins_from_dir("/nonexistent/directory")
        .await?
        .build()
        .await?;

    // Should have no plugins
    let plugins = mem.list_plugins().await;
    assert_eq!(plugins.len(), 0);

    Ok(())
}

#[cfg(feature = "plugins")]
#[tokio::test]
async fn test_builder_chain_with_other_configs() -> Result<()> {
    // Test that plugin methods work well with other builder methods
    let metadata = PluginMetadata {
        name: "chain-test".to_string(),
        version: "1.0.0".to_string(),
        description: "Chain test".to_string(),
        author: "Test".to_string(),
        plugin_type: PluginType::SearchAlgorithm,
        required_capabilities: vec![Capability::SearchAccess],
        config_schema: None,
    };

    let plugin = RegisteredPlugin {
        id: "chain-test".to_string(),
        metadata,
        path: "/tmp/chain-test.wasm".to_string(),
        status: PluginStatus::Registered,
        config: PluginConfig::default(),
        registered_at: Utc::now(),
        last_loaded_at: None,
    };

    let mem = Memory::builder()
        .with_storage("memory://")
        .with_user("alice")
        .with_agent("test-agent")
        .with_plugin(plugin)
        .disable_intelligent_features()
        .build()
        .await?;

    // Verify plugin is registered
    let plugins = mem.list_plugins().await;
    assert_eq!(plugins.len(), 1);

    // Verify memory works
    mem.add("Test content").await?;
    let results = mem.search("Test").await?;
    assert!(!results.is_empty());

    Ok(())
}

#[cfg(feature = "plugins")]
#[tokio::test]
async fn test_builder_without_plugins() -> Result<()> {
    // Test that builder works without any plugins
    let mem = Memory::builder().with_storage("memory://").build().await?;

    // Should have no plugins
    let plugins = mem.list_plugins().await;
    assert_eq!(plugins.len(), 0);

    // Memory should work normally
    mem.add("Test").await?;
    let results = mem.search("Test").await?;
    assert!(!results.is_empty());

    Ok(())
}

// Test without plugins feature to ensure backwards compatibility
#[cfg(not(feature = "plugins"))]
use agent_mem::types::GetAllOptions;
#[cfg(not(feature = "plugins"))]
#[tokio::test]
async fn test_builder_without_plugins_feature() {
    use agent_mem::Memory;

    let mem = Memory::builder()
        .with_storage("memory://")
        .build()
        .await
        .unwrap();

    mem.add("Test content").await.unwrap();
    let results = mem.get_all(GetAllOptions::default()).await.unwrap();
    assert!(!results.is_empty());
}
