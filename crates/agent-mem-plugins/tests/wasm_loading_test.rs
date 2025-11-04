//! WASM Plugin Loading and Execution Tests

use agent_mem_plugin_sdk::{Capability, PluginConfig, PluginMetadata, PluginType};
use agent_mem_plugins::*;
use chrono::Utc;
use std::path::PathBuf;

/// Get the path to compiled WASM plugins
fn get_wasm_plugin_path(plugin_name: &str) -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = PathBuf::from(manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    
    workspace_root.join(format!(
        "target/wasm32-wasip1/release/{}.wasm",
        plugin_name
    ))
}

#[tokio::test]
#[ignore] // Only run when WASM files are compiled
async fn test_load_hello_plugin_wasm() {
    let wasm_path = get_wasm_plugin_path("hello_plugin");
    
    if !wasm_path.exists() {
        eprintln!("⚠️  WASM file not found: {:?}", wasm_path);
        eprintln!("   Run: cd examples/hello_plugin && cargo build --target wasm32-wasip1 --release");
        return;
    }
    
    let manager = PluginManager::new(10);
    
    let plugin = RegisteredPlugin {
        id: "hello-plugin".to_string(),
        metadata: PluginMetadata {
            name: "Hello Plugin".to_string(),
            version: "0.1.0".to_string(),
            description: "Test hello plugin".to_string(),
            author: "Test".to_string(),
            plugin_type: PluginType::Custom("greeting".to_string()),
            required_capabilities: vec![Capability::LoggingAccess],
            config_schema: None,
        },
        path: wasm_path.to_string_lossy().to_string(),
        status: PluginStatus::Registered,
        config: PluginConfig::default(),
        registered_at: Utc::now(),
        last_loaded_at: None,
    };
    
    // Register plugin
    assert!(manager.register(plugin).await.is_ok());
    
    // Try to load plugin
    let result = manager.get_plugin("hello-plugin").await;
    
    match result {
        Ok(loaded_plugin) => {
            println!("✅ Successfully loaded hello_plugin.wasm");
            let mut plugin = loaded_plugin.lock().await;
            
            // Try to call metadata function
            let metadata_result = PluginLoader::call_plugin(
                &mut plugin.plugin,
                "metadata",
                ""
            );
            
            if let Ok(metadata_json) = metadata_result {
                println!("✅ Plugin metadata: {}", metadata_json);
                assert!(metadata_json.contains("hello-plugin"));
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to load plugin: {}", e);
            panic!("Plugin loading failed");
        }
    }
}

#[tokio::test]
#[ignore] // Only run when WASM files are compiled
async fn test_execute_hello_plugin() {
    let wasm_path = get_wasm_plugin_path("hello_plugin");
    
    if !wasm_path.exists() {
        eprintln!("⚠️  WASM file not found: {:?}", wasm_path);
        return;
    }
    
    let manager = PluginManager::new(10);
    
    let plugin = RegisteredPlugin {
        id: "hello-plugin".to_string(),
        metadata: PluginMetadata {
            name: "Hello Plugin".to_string(),
            version: "0.1.0".to_string(),
            description: "Test hello plugin".to_string(),
            author: "Test".to_string(),
            plugin_type: PluginType::Custom("greeting".to_string()),
            required_capabilities: vec![Capability::LoggingAccess],
            config_schema: None,
        },
        path: wasm_path.to_string_lossy().to_string(),
        status: PluginStatus::Registered,
        config: PluginConfig::default(),
        registered_at: Utc::now(),
        last_loaded_at: None,
    };
    
    manager.register(plugin).await.unwrap();
    
    // Call hello function
    let input = serde_json::json!({"message": "World"});
    let result = manager
        .call_plugin("hello-plugin", "hello", &input.to_string())
        .await;
    
    match result {
        Ok(output) => {
            println!("✅ Plugin output: {}", output);
            assert!(output.contains("Hello"));
            
            let response: serde_json::Value = serde_json::from_str(&output).unwrap();
            assert_eq!(response["greeting"], "Hello, World!");
        }
        Err(e) => {
            eprintln!("❌ Plugin execution failed: {}", e);
            panic!("Plugin execution failed");
        }
    }
}

#[tokio::test]
#[ignore] // Only run when WASM files are compiled
async fn test_load_memory_processor_plugin() {
    let wasm_path = get_wasm_plugin_path("memory_processor_plugin");
    
    if !wasm_path.exists() {
        eprintln!("⚠️  WASM file not found: {:?}", wasm_path);
        return;
    }
    
    let manager = PluginManager::new(10);
    
    let plugin = RegisteredPlugin {
        id: "memory-processor".to_string(),
        metadata: PluginMetadata {
            name: "Memory Processor".to_string(),
            version: "0.1.0".to_string(),
            description: "Memory processing plugin".to_string(),
            author: "Test".to_string(),
            plugin_type: PluginType::MemoryProcessor,
            required_capabilities: vec![Capability::MemoryAccess],
            config_schema: None,
        },
        path: wasm_path.to_string_lossy().to_string(),
        status: PluginStatus::Registered,
        config: PluginConfig::default(),
        registered_at: Utc::now(),
        last_loaded_at: None,
    };
    
    assert!(manager.register(plugin).await.is_ok());
    
    let result = manager.get_plugin("memory-processor").await;
    assert!(result.is_ok(), "Should load memory processor plugin");
    
    println!("✅ Successfully loaded memory_processor.wasm");
}

#[tokio::test]
#[ignore] // Only run when WASM files are compiled
async fn test_load_code_analyzer_plugin() {
    let wasm_path = get_wasm_plugin_path("code_analyzer_plugin");
    
    if !wasm_path.exists() {
        eprintln!("⚠️  WASM file not found: {:?}", wasm_path);
        return;
    }
    
    let manager = PluginManager::new(10);
    
    let plugin = RegisteredPlugin {
        id: "code-analyzer".to_string(),
        metadata: PluginMetadata {
            name: "Code Analyzer".to_string(),
            version: "0.1.0".to_string(),
            description: "Code analysis plugin".to_string(),
            author: "Test".to_string(),
            plugin_type: PluginType::CodeAnalyzer,
            required_capabilities: vec![Capability::LoggingAccess],
            config_schema: None,
        },
        path: wasm_path.to_string_lossy().to_string(),
        status: PluginStatus::Registered,
        config: PluginConfig::default(),
        registered_at: Utc::now(),
        last_loaded_at: None,
    };
    
    assert!(manager.register(plugin).await.is_ok());
    
    let result = manager.get_plugin("code-analyzer").await;
    assert!(result.is_ok(), "Should load code analyzer plugin");
    
    println!("✅ Successfully loaded code_analyzer.wasm");
}

#[test]
fn test_wasm_paths_documentation() {
    println!("\n📖 WASM Plugin Paths:");
    println!("   hello_plugin: {}", get_wasm_plugin_path("hello_plugin").display());
    println!("   memory_processor: {}", get_wasm_plugin_path("memory_processor_plugin").display());
    println!("   code_analyzer: {}", get_wasm_plugin_path("code_analyzer_plugin").display());
    println!("\n💡 To compile WASM plugins:");
    println!("   cd crates/agent-mem-plugin-sdk/examples/hello_plugin");
    println!("   cargo build --target wasm32-wasip1 --release");
}

