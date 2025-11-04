//! Deep Plugin Integration Example
//!
//! This example demonstrates how the plugin system is deeply integrated into
//! AgentMem's Memory operations.
//!
//! # Features Demonstrated
//! - Plugin registration and management
//! - Plugin hooks in memory operations
//! - Custom search algorithms via plugins
//! - Memory processing pipelines

use agent_mem::Memory;
use anyhow::Result;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("🚀 AgentMem Plugin Deep Integration Example");
    info!("============================================\n");

    // Example 1: Basic Memory with Plugin Hooks
    example_1_basic_plugin_hooks().await?;

    // Example 2: Custom Search Algorithm Plugin
    #[cfg(feature = "plugins")]
    example_2_custom_search().await?;

    // Example 3: Memory Processing Pipeline
    #[cfg(feature = "plugins")]
    example_3_processing_pipeline().await?;

    info!("\n✅ All examples completed successfully!");
    Ok(())
}

/// Example 1: Basic memory operations with plugin hooks
async fn example_1_basic_plugin_hooks() -> Result<()> {
    info!("📝 Example 1: Basic Memory Operations with Plugin Hooks");
    info!("------------------------------------------------------");

    let mem = Memory::builder()
        .with_storage("memory://")
        .build()
        .await?;

    // Add memories - plugin hooks will be called if plugins are enabled
    info!("Adding memories...");
    mem.add("I love Rust programming").await?;
    mem.add("AgentMem is a powerful memory system").await?;
    mem.add("WASM plugins provide extensibility").await?;

    // Search memories - search algorithm plugins can enhance results
    info!("Searching for 'Rust'...");
    let results = mem.search("Rust").await?;
    info!("Found {} results", results.len());

    for (i, memory) in results.iter().enumerate() {
        info!("  {}. {}", i + 1, memory.content);
    }

    info!("");
    Ok(())
}

/// Example 2: Using custom search algorithm plugin
#[cfg(feature = "plugins")]
async fn example_2_custom_search() -> Result<()> {
    use agent_mem::plugin_integration::PluginEnhancedMemory;
    use agent_mem::plugins::{
        Capability, PluginConfig, PluginStatus, PluginType, RegisteredPlugin,
    };
    use agent_mem::plugins::sdk::PluginMetadata;
    use chrono::Utc;

    info!("🔍 Example 2: Custom Search Algorithm Plugin");
    info!("--------------------------------------------");

    // Create plugin-enhanced memory
    let mut plugin_memory = PluginEnhancedMemory::new();

    // Register a search algorithm plugin
    let metadata = PluginMetadata {
        name: "semantic-search".to_string(),
        version: "1.0.0".to_string(),
        description: "Advanced semantic search algorithm".to_string(),
        author: "AgentMem Team".to_string(),
        plugin_type: PluginType::SearchAlgorithm,
        required_capabilities: vec![Capability::SearchAccess, Capability::MemoryAccess],
        config_schema: None,
    };

    let plugin = RegisteredPlugin {
        id: "semantic-search".to_string(),
        metadata,
        path: "target/wasm32-wasip1/release/search_plugin.wasm".to_string(),
        status: PluginStatus::Registered,
        config: PluginConfig::default(),
        registered_at: Utc::now(),
        last_loaded_at: None,
    };

    plugin_memory.register_plugin(plugin)?;

    info!("✅ Search algorithm plugin registered");
    info!("   Plugin can be used to enhance search results");
    info!("");

    Ok(())
}

/// Example 3: Memory processing pipeline with plugins
#[cfg(feature = "plugins")]
async fn example_3_processing_pipeline() -> Result<()> {
    use agent_mem::plugin_integration::{PluginEnhancedMemory, PluginHooks};
    use agent_mem::plugins::{
        Capability, PluginConfig, PluginStatus, PluginType, RegisteredPlugin,
    };
    use agent_mem::plugins::sdk::PluginMetadata;
    use agent_mem_traits::{MemoryItem, MemoryType};
    use chrono::Utc;

    info!("⚙️  Example 3: Memory Processing Pipeline");
    info!("----------------------------------------");

    let mut plugin_memory = PluginEnhancedMemory::new();

    // Register memory processor plugin
    let metadata = PluginMetadata {
        name: "content-cleaner".to_string(),
        version: "1.0.0".to_string(),
        description: "Cleans and formats memory content".to_string(),
        author: "AgentMem Team".to_string(),
        plugin_type: PluginType::MemoryProcessor,
        required_capabilities: vec![Capability::MemoryAccess, Capability::LoggingAccess],
        config_schema: None,
    };

    let plugin = RegisteredPlugin {
        id: "content-cleaner".to_string(),
        metadata,
        path: "target/wasm32-wasip1/release/memory_processor.wasm".to_string(),
        status: PluginStatus::Registered,
        config: PluginConfig::default(),
        registered_at: Utc::now(),
        last_loaded_at: None,
    };

    plugin_memory.register_plugin(plugin)?;

    info!("✅ Memory processor plugin registered");

    // Create a memory item
    let mut memory = MemoryItem {
        id: "test-1".to_string(),
        content: "   This is  unformatted   content   ".to_string(),
        hash: None,
        metadata: std::collections::HashMap::new(),
        score: None,
        created_at: Utc::now(),
        updated_at: None,
        session: agent_mem_traits::Session {
            id: "test-session".to_string(),
            created_at: Utc::now(),
            user_id: Some("test-user".to_string()),
            agent_id: Some("test-agent".to_string()),
            actor_id: Some("test-actor".to_string()),
            run_id: Some("test-run".to_string()),
            metadata: std::collections::HashMap::new(),
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

    info!("📄 Original content: '{}'", memory.content);

    // Process through plugin hooks
    plugin_memory.before_add_memory(&mut memory)?;

    info!("✨ After plugin processing (would be cleaned if plugin was loaded)");
    info!("   Content: '{}'", memory.content);
    info!("");

    Ok(())
}

/// Example 4: Plugin metrics and monitoring
#[cfg(feature = "plugins")]
#[allow(dead_code)]
async fn example_4_plugin_metrics() -> Result<()> {
    use agent_mem::plugin_integration::PluginEnhancedMemory;

    info!("📊 Example 4: Plugin Metrics and Monitoring");
    info!("------------------------------------------");

    let plugin_memory = PluginEnhancedMemory::new();

    info!("Plugin Manager: {:?}", plugin_memory.plugin_manager());
    info!("Plugin Registry: {:?}", plugin_memory.plugin_registry());
    info!("");

    Ok(())
}

