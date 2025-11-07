//! Tests for Memory plugin integration

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
async fn test_memory_has_plugin_layer() -> Result<()> {
    let mem = Memory::builder().with_storage("memory://").build().await?;

    // Memory should be created with a plugin layer
    // We can verify this by calling list_plugins
    let plugins = mem.list_plugins().await;
    assert_eq!(plugins.len(), 0); // No plugins registered yet

    Ok(())
}

#[cfg(feature = "plugins")]
#[tokio::test]
async fn test_register_plugin_via_memory() -> Result<()> {
    let mem = Memory::builder().with_storage("memory://").build().await?;

    // Create a test plugin
    let metadata = PluginMetadata {
        name: "test-memory-plugin".to_string(),
        version: "1.0.0".to_string(),
        description: "Test plugin for Memory".to_string(),
        author: "Test".to_string(),
        plugin_type: PluginType::MemoryProcessor,
        required_capabilities: vec![Capability::MemoryAccess],
        config_schema: None,
    };

    let plugin = RegisteredPlugin {
        id: "test-memory-plugin".to_string(),
        metadata: metadata.clone(),
        path: "/tmp/test.wasm".to_string(),
        status: PluginStatus::Registered,
        config: PluginConfig::default(),
        registered_at: Utc::now(),
        last_loaded_at: None,
    };

    // Register the plugin
    mem.register_plugin(plugin).await?;

    // Verify plugin is registered
    let plugins = mem.list_plugins().await;
    assert_eq!(plugins.len(), 1);
    assert_eq!(plugins[0].name, "test-memory-plugin");

    Ok(())
}

#[cfg(feature = "plugins")]
#[tokio::test]
async fn test_register_multiple_plugins_via_memory() -> Result<()> {
    let mem = Memory::builder().with_storage("memory://").build().await?;

    // Register 3 different plugins
    for i in 1..=3 {
        let metadata = PluginMetadata {
            name: format!("memory-plugin-{}", i),
            version: "1.0.0".to_string(),
            description: format!("Test plugin {}", i),
            author: "Test".to_string(),
            plugin_type: PluginType::MemoryProcessor,
            required_capabilities: vec![Capability::MemoryAccess],
            config_schema: None,
        };

        let plugin = RegisteredPlugin {
            id: format!("memory-plugin-{}", i),
            metadata,
            path: format!("/tmp/plugin-{}.wasm", i),
            status: PluginStatus::Registered,
            config: PluginConfig::default(),
            registered_at: Utc::now(),
            last_loaded_at: None,
        };

        mem.register_plugin(plugin).await?;
    }

    // Verify all plugins are registered
    let plugins = mem.list_plugins().await;
    assert_eq!(plugins.len(), 3);

    Ok(())
}

#[cfg(feature = "plugins")]
#[tokio::test]
async fn test_memory_operations_with_plugins() -> Result<()> {
    let mem = Memory::builder().with_storage("memory://").build().await?;

    // Register a plugin
    let metadata = PluginMetadata {
        name: "processor-plugin".to_string(),
        version: "1.0.0".to_string(),
        description: "Memory processor".to_string(),
        author: "Test".to_string(),
        plugin_type: PluginType::MemoryProcessor,
        required_capabilities: vec![Capability::MemoryAccess, Capability::LoggingAccess],
        config_schema: None,
    };

    let plugin = RegisteredPlugin {
        id: "processor-plugin".to_string(),
        metadata,
        path: "/tmp/processor.wasm".to_string(),
        status: PluginStatus::Registered,
        config: PluginConfig::default(),
        registered_at: Utc::now(),
        last_loaded_at: None,
    };

    mem.register_plugin(plugin).await?;

    // Perform memory operations - plugins should not interfere
    let result = mem.add("Test memory content").await?;
    assert!(!result.results.is_empty());

    let search_results = mem.search("Test").await?;
    assert!(!search_results.is_empty());

    Ok(())
}

#[cfg(feature = "plugins")]
#[tokio::test]
async fn test_different_plugin_types() -> Result<()> {
    let mem = Memory::builder().with_storage("memory://").build().await?;

    let plugin_types = vec![
        (
            PluginType::MemoryProcessor,
            vec![Capability::MemoryAccess],
            "memory-proc",
        ),
        (
            PluginType::SearchAlgorithm,
            vec![Capability::SearchAccess],
            "search-algo",
        ),
        (
            PluginType::CodeAnalyzer,
            vec![Capability::MemoryAccess],
            "code-analyzer",
        ),
        (
            PluginType::DataSource,
            vec![Capability::NetworkAccess, Capability::StorageAccess],
            "data-source",
        ),
    ];

    for (i, (plugin_type, capabilities, name_prefix)) in plugin_types.into_iter().enumerate() {
        let metadata = PluginMetadata {
            name: format!("{}-{}", name_prefix, i),
            version: "1.0.0".to_string(),
            description: format!("Test {:?} plugin", plugin_type),
            author: "Test".to_string(),
            plugin_type,
            required_capabilities: capabilities,
            config_schema: None,
        };

        let plugin = RegisteredPlugin {
            id: format!("plugin-type-{}", i),
            metadata,
            path: format!("/tmp/{}.wasm", name_prefix),
            status: PluginStatus::Registered,
            config: PluginConfig::default(),
            registered_at: Utc::now(),
            last_loaded_at: None,
        };

        mem.register_plugin(plugin).await?;
    }

    let plugins = mem.list_plugins().await;
    assert_eq!(plugins.len(), 4);

    Ok(())
}

#[cfg(feature = "plugins")]
#[tokio::test]
async fn test_plugin_registry_access() -> Result<()> {
    let mem = Memory::builder().with_storage("memory://").build().await?;

    // Register a plugin
    let metadata = PluginMetadata {
        name: "test-registry-plugin".to_string(),
        version: "1.0.0".to_string(),
        description: "Test".to_string(),
        author: "Test".to_string(),
        plugin_type: PluginType::MemoryProcessor,
        required_capabilities: vec![Capability::MemoryAccess],
        config_schema: None,
    };

    let plugin = RegisteredPlugin {
        id: "test-registry-plugin".to_string(),
        metadata,
        path: "/tmp/test.wasm".to_string(),
        status: PluginStatus::Registered,
        config: PluginConfig::default(),
        registered_at: Utc::now(),
        last_loaded_at: None,
    };

    mem.register_plugin(plugin).await?;

    // List plugins
    {
        let plugins = mem.list_plugins().await;
        assert_eq!(plugins.len(), 1);
    }

    Ok(())
}

// Test that Memory works fine without plugins feature
#[cfg(not(feature = "plugins"))]
#[tokio::test]
async fn test_memory_without_plugins() {
    use agent_mem::Memory;

    let mem = Memory::builder()
        .with_storage("memory://")
        .build()
        .await
        .unwrap();

    let result = mem.add("Test content").await.unwrap();
    assert!(!result.results.is_empty());
}
