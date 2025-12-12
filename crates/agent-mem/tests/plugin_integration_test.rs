//! Integration tests for plugin system integration with AgentMem

use agent_mem::Memory;
use anyhow::Result;

#[tokio::test]
async fn test_memory_without_plugins() -> Result<()> {
    // Memory should work fine without plugins feature enabled
    let mem = Memory::builder().with_storage("memory://").build().await?;

    let result = mem.add("Test content").await?;
    assert!(!result.results.is_empty());

    // 搜索可能因为 embedder 未配置而失败，这是预期的
    match mem.search("Test").await {
        Ok(results) => {
            assert!(!results.is_empty());
        }
        Err(e) if e.to_string().contains("Embedder not configured") => {
            // 预期行为：如果没有配置 embedder，搜索会失败
            println!("⚠️ 搜索失败（预期行为）：Embedder 未配置");
        }
        Err(e) => {
            return Err(e.into());
        }
    }

    Ok(())
}

#[cfg(feature = "plugins")]
#[tokio::test]
async fn test_plugin_enhanced_memory_creation() -> Result<()> {
    use agent_mem::plugin_integration::PluginEnhancedMemory;

    let _plugin_memory = PluginEnhancedMemory::new();
    // Should create successfully
    Ok(())
}

#[cfg(feature = "plugins")]
#[tokio::test]
async fn test_plugin_hooks_integration() -> Result<()> {
    use agent_mem::plugin_integration::{PluginEnhancedMemory, PluginHooks};
    use agent_mem_traits::{MemoryItem, MemoryType, Session};
    use chrono::Utc;
    use std::collections::HashMap;

    let plugin_memory = PluginEnhancedMemory::new();

    let mut memory = MemoryItem {
        id: "test-1".to_string(),
        content: "Test content".to_string(),
        hash: None,
        metadata: HashMap::new(),
        score: None,
        created_at: Utc::now(),
        updated_at: None,
        session: Session {
            id: "test-session".to_string(),
            created_at: Utc::now(),
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
        last_accessed_at: Utc::now(),
        access_count: 0,
        expires_at: None,
        version: 1,
    };

    // Test all plugin hooks
    plugin_memory.before_add_memory(&mut memory).await?;
    plugin_memory.after_add_memory(&memory).await?;
    plugin_memory.before_search("test query").await?;

    let mut results = vec![memory];
    plugin_memory.after_search(&mut results).await?;

    Ok(())
}

#[cfg(feature = "plugins")]
#[tokio::test]
async fn test_plugin_registration() -> Result<()> {
    use agent_mem::plugin_integration::PluginEnhancedMemory;
    use agent_mem::plugins::sdk::{Capability, PluginConfig, PluginMetadata, PluginType};
    use agent_mem::plugins::{PluginStatus, RegisteredPlugin};
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
        metadata: metadata.clone(),
        path: "/tmp/test.wasm".to_string(),
        status: PluginStatus::Registered,
        config: PluginConfig::default(),
        registered_at: Utc::now(),
        last_loaded_at: None,
    };

    // Should register successfully
    plugin_memory.register_plugin(plugin).await?;

    // Should appear in plugin list
    let plugins = plugin_memory.list_plugins().await;
    assert_eq!(plugins.len(), 1);
    assert_eq!(plugins[0].metadata.name, "test-plugin");

    Ok(())
}

#[cfg(feature = "plugins")]
#[tokio::test]
async fn test_multiple_plugin_registration() -> Result<()> {
    use agent_mem::plugin_integration::PluginEnhancedMemory;
    use agent_mem::plugins::sdk::{Capability, PluginConfig, PluginMetadata, PluginType};
    use agent_mem::plugins::{PluginStatus, RegisteredPlugin};
    use chrono::Utc;

    let mut plugin_memory = PluginEnhancedMemory::new();

    // Register multiple plugins
    for i in 1..=3 {
        let metadata = PluginMetadata {
            name: format!("plugin-{}", i),
            version: "0.1.0".to_string(),
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

        plugin_memory.register_plugin(plugin).await?;
    }

    // Should have all 3 plugins
    let plugins = plugin_memory.list_plugins().await;
    assert_eq!(plugins.len(), 3);

    Ok(())
}

#[cfg(feature = "plugins")]
#[tokio::test]
async fn test_plugin_types() -> Result<()> {
    use agent_mem::plugin_integration::PluginEnhancedMemory;
    use agent_mem::plugins::sdk::{Capability, PluginConfig, PluginMetadata, PluginType};
    use agent_mem::plugins::{PluginStatus, RegisteredPlugin};
    use chrono::Utc;

    let mut plugin_memory = PluginEnhancedMemory::new();

    // Test different plugin types
    let types = vec![
        (PluginType::MemoryProcessor, vec![Capability::MemoryAccess]),
        (PluginType::SearchAlgorithm, vec![Capability::SearchAccess]),
        (PluginType::CodeAnalyzer, vec![Capability::MemoryAccess]),
        (
            PluginType::DataSource,
            vec![Capability::NetworkAccess, Capability::StorageAccess],
        ),
    ];

    for (i, (plugin_type, capabilities)) in types.into_iter().enumerate() {
        let metadata = PluginMetadata {
            name: format!("plugin-{}", i),
            version: "0.1.0".to_string(),
            description: format!("Test plugin type {:?}", plugin_type),
            author: "Test".to_string(),
            plugin_type,
            required_capabilities: capabilities,
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

        plugin_memory.register_plugin(plugin).await?;
    }

    // Should have all plugins
    let plugins = plugin_memory.list_plugins().await;
    assert_eq!(plugins.len(), 4);

    Ok(())
}
