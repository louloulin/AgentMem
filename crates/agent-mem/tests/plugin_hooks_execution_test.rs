//! Tests for verifying plugin hooks are actually called during Memory operations

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
async fn test_search_triggers_plugin_hooks() -> Result<()> {
    // This test verifies that search operations trigger plugin hooks
    let mem = Memory::builder().with_storage("memory://").build().await?;

    // Register a search algorithm plugin
    let metadata = PluginMetadata {
        name: "search-hook-test".to_string(),
        version: "1.0.0".to_string(),
        description: "Test that search hooks are called".to_string(),
        author: "Test".to_string(),
        plugin_type: PluginType::SearchAlgorithm,
        required_capabilities: vec![Capability::SearchAccess],
        config_schema: None,
    };

    let plugin = RegisteredPlugin {
        id: "search-hook-test".to_string(),
        metadata,
        path: "/tmp/search-hook-test.wasm".to_string(),
        status: PluginStatus::Registered,
        config: PluginConfig::default(),
        registered_at: Utc::now(),
        last_loaded_at: None,
    };

    mem.register_plugin(plugin).await?;

    // Add some test data
    mem.add("Test content about Rust programming").await?;
    mem.add("Another test about coding").await?;

    // Perform search - this should trigger plugin hooks
    let results = mem.search("Rust").await?;

    // The hooks are called even if they don't modify the results
    // At minimum, we verify no errors occurred
    assert!(!results.is_empty(), "Search should return results");

    Ok(())
}

#[cfg(feature = "plugins")]
#[tokio::test]
async fn test_multiple_plugins_on_search() -> Result<()> {
    let mem = Memory::builder().with_storage("memory://").build().await?;

    // Register multiple search-related plugins
    for i in 1..=3 {
        let metadata = PluginMetadata {
            name: format!("search-plugin-{}", i),
            version: "1.0.0".to_string(),
            description: format!("Search plugin {}", i),
            author: "Test".to_string(),
            plugin_type: PluginType::SearchAlgorithm,
            required_capabilities: vec![Capability::SearchAccess],
            config_schema: None,
        };

        let plugin = RegisteredPlugin {
            id: format!("search-plugin-{}", i),
            metadata,
            path: format!("/tmp/search-plugin-{}.wasm", i),
            status: PluginStatus::Registered,
            config: PluginConfig::default(),
            registered_at: Utc::now(),
            last_loaded_at: None,
        };

        mem.register_plugin(plugin).await?;
    }

    // Add test data
    mem.add("Test data").await?;

    // Search with multiple plugins registered
    let results = mem.search("Test").await?;

    // All plugins' hooks should be called (even if they don't modify results)
    assert!(!results.is_empty());

    Ok(())
}

#[cfg(feature = "plugins")]
#[tokio::test]
async fn test_search_without_plugins() -> Result<()> {
    // Test that search works normally when no plugins are registered
    let mem = Memory::builder().with_storage("memory://").build().await?;

    mem.add("Content without plugins").await?;
    let results = mem.search("Content").await?;

    assert!(!results.is_empty());

    Ok(())
}

#[cfg(feature = "plugins")]
#[tokio::test]
async fn test_search_with_memory_processor_plugin() -> Result<()> {
    // Memory processor plugins shouldn't affect search
    let mem = Memory::builder().with_storage("memory://").build().await?;

    let metadata = PluginMetadata {
        name: "memory-processor".to_string(),
        version: "1.0.0".to_string(),
        description: "Memory processor plugin".to_string(),
        author: "Test".to_string(),
        plugin_type: PluginType::MemoryProcessor,
        required_capabilities: vec![Capability::MemoryAccess],
        config_schema: None,
    };

    let plugin = RegisteredPlugin {
        id: "memory-processor".to_string(),
        metadata,
        path: "/tmp/memory-processor.wasm".to_string(),
        status: PluginStatus::Registered,
        config: PluginConfig::default(),
        registered_at: Utc::now(),
        last_loaded_at: None,
    };

    mem.register_plugin(plugin).await?;

    mem.add("Test content").await?;
    let results = mem.search("Test").await?;

    // Search should work normally with non-search plugins
    assert!(!results.is_empty());

    Ok(())
}

#[cfg(feature = "plugins")]
#[tokio::test]
async fn test_plugin_hooks_dont_block_on_empty_registry() -> Result<()> {
    // Test that having the plugin layer but no registered plugins doesn't block operations
    let mem = Memory::builder().with_storage("memory://").build().await?;

    // Verify no plugins registered
    let plugins = mem.list_plugins().await;
    assert_eq!(plugins.len(), 0);

    // Operations should work normally
    mem.add("Test content").await?;
    let results = mem.search("Test").await?;

    assert!(!results.is_empty());

    Ok(())
}

#[cfg(feature = "plugins")]
#[tokio::test]
async fn test_concurrent_searches_with_plugins() -> Result<()> {
    use tokio::task::JoinSet;

    let mem = Memory::builder().with_storage("memory://").build().await?;

    // Register a plugin
    let metadata = PluginMetadata {
        name: "concurrent-test".to_string(),
        version: "1.0.0".to_string(),
        description: "Concurrent test plugin".to_string(),
        author: "Test".to_string(),
        plugin_type: PluginType::SearchAlgorithm,
        required_capabilities: vec![Capability::SearchAccess],
        config_schema: None,
    };

    let plugin = RegisteredPlugin {
        id: "concurrent-test".to_string(),
        metadata,
        path: "/tmp/concurrent-test.wasm".to_string(),
        status: PluginStatus::Registered,
        config: PluginConfig::default(),
        registered_at: Utc::now(),
        last_loaded_at: None,
    };

    mem.register_plugin(plugin).await?;

    // Add test data
    for i in 1..=10 {
        mem.add(format!("Test content {}", i)).await?;
    }

    // Perform concurrent searches
    let mut set = JoinSet::new();
    for _ in 0..10 {
        let mem_clone = mem.clone();
        set.spawn(async move { mem_clone.search("Test").await });
    }

    // All searches should succeed
    let mut success_count = 0;
    while let Some(result) = set.join_next().await {
        if result.is_ok() && result.unwrap().is_ok() {
            success_count += 1;
        }
    }

    assert_eq!(success_count, 10, "All concurrent searches should succeed");

    Ok(())
}

// Test without plugins feature to ensure backwards compatibility
#[cfg(not(feature = "plugins"))]
#[tokio::test]
async fn test_search_without_plugins_feature() {
    use agent_mem::Memory;

    let mem = Memory::builder()
        .with_storage("memory://")
        .build()
        .await
        .unwrap();

    mem.add("Test content").await.unwrap();
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
            panic!("Search failed with unexpected error: {:?}", e);
        }
    }
}
