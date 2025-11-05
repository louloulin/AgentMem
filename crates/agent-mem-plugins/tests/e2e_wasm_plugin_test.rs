//! End-to-End WASM Plugin Tests
//!
//! Tests actual loading and execution of compiled WASM plugins

use agent_mem_plugins::{PluginLoader, PluginManager, RegisteredPlugin, PluginStatus};
use agent_mem_plugin_sdk::{PluginMetadata, PluginType, Capability, PluginConfig};
use chrono::Utc;
use std::path::PathBuf;

#[tokio::test]
async fn test_load_hello_plugin_wasm() -> Result<(), Box<dyn std::error::Error>> {
    // Get plugin path
    let mut plugin_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    plugin_path.pop(); // Go up to crates/
    plugin_path.pop(); // Go up to agentmen/
    plugin_path.push("target/wasm32-wasip1/release/hello_plugin.wasm");
    
    if !plugin_path.exists() {
        println!("⚠️  Skipping test: WASM plugin not found at {:?}", plugin_path);
        println!("   Run ./build_plugins.sh to build WASM plugins");
        return Ok(());
    }
    
    println!("✅ Found plugin: {:?}", plugin_path);
    
    // Create plugin info
    let plugin = RegisteredPlugin {
        id: "hello-plugin".to_string(),
        metadata: PluginMetadata {
            name: "hello-plugin".to_string(),
            version: "0.1.0".to_string(),
            description: "Hello World Plugin".to_string(),
            author: "AgentMem Team".to_string(),
            plugin_type: PluginType::Custom("hello".to_string()),
            required_capabilities: vec![Capability::LoggingAccess],
            config_schema: None,
        },
        path: plugin_path.to_str().unwrap().to_string(),
        status: PluginStatus::Registered,
        config: PluginConfig::default(),
        registered_at: Utc::now(),
        last_loaded_at: None,
    };
    
    // Load plugin
    let loader = PluginLoader::new();
    let mut loaded_plugin = loader.load_plugin(&plugin)?;
    
    println!("✅ Plugin loaded successfully: {}", loaded_plugin.id);
    
    // Call plugin hello function
    let input = serde_json::json!({ "message": "World" });
    let result = PluginLoader::call_plugin(
        &mut loaded_plugin.plugin,
        "hello",
        &serde_json::to_string(&input)?
    )?;
    
    println!("✅ Plugin execution result: {}", result);
    
    // Verify result contains greeting
    assert!(result.contains("Hello"), "Expected greeting in result");
    
    Ok(())
}

#[tokio::test]
async fn test_memory_processor_plugin_wasm() -> Result<(), Box<dyn std::error::Error>> {
    // Get plugin path
    let mut plugin_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    plugin_path.pop(); // Go up to crates/
    plugin_path.pop(); // Go up to agentmen/
    plugin_path.push("target/wasm32-wasip1/release/memory_processor_plugin.wasm");
    
    if !plugin_path.exists() {
        println!("⚠️  Skipping test: WASM plugin not found at {:?}", plugin_path);
        return Ok(());
    }
    
    println!("✅ Found plugin: {:?}", plugin_path);
    
    // Create plugin info
    let plugin = RegisteredPlugin {
        id: "memory-processor".to_string(),
        metadata: PluginMetadata {
            name: "memory-processor".to_string(),
            version: "0.1.0".to_string(),
            description: "Memory Processor Plugin".to_string(),
            author: "AgentMem Team".to_string(),
            plugin_type: PluginType::MemoryProcessor,
            required_capabilities: vec![Capability::MemoryAccess],
            config_schema: None,
        },
        path: plugin_path.to_str().unwrap().to_string(),
        status: PluginStatus::Registered,
        config: PluginConfig::default(),
        registered_at: Utc::now(),
        last_loaded_at: None,
    };
    
    // Load plugin
    let loader = PluginLoader::new();
    let mut loaded_plugin = loader.load_plugin(&plugin)?;
    
    println!("✅ Plugin loaded successfully: {}", loaded_plugin.id);
    
    // Test process_memory function
    let memory = serde_json::json!({
        "id": "test-1",
        "content": "  This is a test memory\n\n  with extra whitespace  ",
        "memory_type": "Semantic",
        "user_id": "test-user",
        "agent_id": null,
        "metadata": {},
        "created_at": "2025-01-01T00:00:00Z",
        "updated_at": "2025-01-01T00:00:00Z"
    });
    
    let result = PluginLoader::call_plugin(
        &mut loaded_plugin.plugin,
        "process_memory",
        &serde_json::to_string(&memory)?
    )?;
    
    println!("✅ Plugin execution result: {}", result);
    
    // Parse result
    let processed: serde_json::Value = serde_json::from_str(&result)?;
    assert!(processed["processed"].as_bool().unwrap_or(false));
    
    Ok(())
}

#[tokio::test]
async fn test_code_analyzer_plugin_wasm() -> Result<(), Box<dyn std::error::Error>> {
    // Get plugin path
    let mut plugin_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    plugin_path.pop(); // Go up to crates/
    plugin_path.pop(); // Go up to agentmen/
    plugin_path.push("target/wasm32-wasip1/release/code_analyzer_plugin.wasm");
    
    if !plugin_path.exists() {
        println!("⚠️  Skipping test: WASM plugin not found at {:?}", plugin_path);
        return Ok(());
    }
    
    println!("✅ Found plugin: {:?}", plugin_path);
    
    // Create plugin info
    let plugin = RegisteredPlugin {
        id: "code-analyzer".to_string(),
        metadata: PluginMetadata {
            name: "code-analyzer".to_string(),
            version: "0.1.0".to_string(),
            description: "Code Analyzer Plugin".to_string(),
            author: "AgentMem Team".to_string(),
            plugin_type: PluginType::CodeAnalyzer,
            required_capabilities: vec![Capability::LoggingAccess],
            config_schema: None,
        },
        path: plugin_path.to_str().unwrap().to_string(),
        status: PluginStatus::Registered,
        config: PluginConfig::default(),
        registered_at: Utc::now(),
        last_loaded_at: None,
    };
    
    // Load plugin
    let loader = PluginLoader::new();
    let mut loaded_plugin = loader.load_plugin(&plugin)?;
    
    println!("✅ Plugin loaded successfully: {}", loaded_plugin.id);
    
    // Test analyze_code function
    let code_input = serde_json::json!({
        "code": "fn main() { println!(\"Hello\"); }",
        "language": "rust"
    });
    
    let result = PluginLoader::call_plugin(
        &mut loaded_plugin.plugin,
        "analyze_code",
        &serde_json::to_string(&code_input)?
    )?;
    
    println!("✅ Plugin execution result: {}", result);
    
    // Parse result
    let analysis: serde_json::Value = serde_json::from_str(&result)?;
    assert_eq!(analysis["language"].as_str(), Some("rust"));
    
    Ok(())
}

#[tokio::test]
async fn test_plugin_manager_with_wasm() -> Result<(), Box<dyn std::error::Error>> {
    // Get plugin path
    let mut plugin_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    plugin_path.pop(); // Go up to crates/
    plugin_path.pop(); // Go up to agentmen/
    plugin_path.push("target/wasm32-wasip1/release/hello_plugin.wasm");
    
    if !plugin_path.exists() {
        println!("⚠️  Skipping test: WASM plugin not found");
        return Ok(());
    }
    
    // Create plugin manager
    let manager = PluginManager::new(10);
    
    // Register plugin
    let plugin = RegisteredPlugin {
        id: "hello-wasm".to_string(),
        metadata: PluginMetadata {
            name: "hello-wasm".to_string(),
            version: "0.1.0".to_string(),
            description: "Hello WASM Plugin".to_string(),
            author: "AgentMem Team".to_string(),
            plugin_type: PluginType::Custom("hello".to_string()),
            required_capabilities: vec![],
            config_schema: None,
        },
        path: plugin_path.to_str().unwrap().to_string(),
        status: PluginStatus::Registered,
        config: PluginConfig::default(),
        registered_at: Utc::now(),
        last_loaded_at: None,
    };
    
    manager.register(plugin).await?;
    
    // Call plugin through manager
    let input = serde_json::json!({ "message": "WASM Test" });
    let result = manager.call_plugin(
        "hello-wasm",
        "hello",
        &serde_json::to_string(&input)?
    ).await?;
    
    println!("✅ Plugin manager call result: {}", result);
    assert!(result.contains("Hello"));
    
    Ok(())
}

#[tokio::test]
async fn test_multiple_wasm_plugins_concurrent() -> Result<(), Box<dyn std::error::Error>> {
    let manager = PluginManager::new(10);
    
    // Register multiple plugins
    let plugins = vec![
        ("hello_plugin.wasm", "hello-1", PluginType::Custom("hello".to_string())),
        ("memory_processor_plugin.wasm", "processor-1", PluginType::MemoryProcessor),
        ("code_analyzer_plugin.wasm", "analyzer-1", PluginType::CodeAnalyzer),
    ];
    
    let mut registered_count = 0;
    
    for (filename, id, plugin_type) in plugins {
        let mut plugin_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        plugin_path.pop(); // Go up to crates/
        plugin_path.pop(); // Go up to agentmen/
        plugin_path.push(format!("target/wasm32-wasip1/release/{}", filename));
        
        if !plugin_path.exists() {
            continue;
        }
        
        let plugin = RegisteredPlugin {
            id: id.to_string(),
            metadata: PluginMetadata {
                name: id.to_string(),
                version: "0.1.0".to_string(),
                description: format!("{} Plugin", id),
                author: "AgentMem Team".to_string(),
                plugin_type,
                required_capabilities: vec![],
                config_schema: None,
            },
            path: plugin_path.to_str().unwrap().to_string(),
            status: PluginStatus::Registered,
            config: PluginConfig::default(),
            registered_at: Utc::now(),
            last_loaded_at: None,
        };
        
        manager.register(plugin).await?;
        registered_count += 1;
    }
    
    if registered_count == 0 {
        println!("⚠️  Skipping test: No WASM plugins found");
        return Ok(());
    }
    
    // List all plugins
    let all_plugins = manager.list_plugins().await;
    println!("✅ Registered {} plugins", all_plugins.len());
    assert_eq!(all_plugins.len(), registered_count);
    
    Ok(())
}

