//! End-to-end integration tests for the plugin system

use agent_mem_plugin_sdk::{Capability, Memory, PluginConfig, PluginMetadata, PluginType};
use agent_mem_plugins::*;
use chrono::Utc;
use std::collections::HashMap;
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

    workspace_root.join(format!("target/wasm32-wasip1/release/{plugin_name}.wasm"))
}

/// Create a test memory object
fn create_test_memory(id: &str, content: &str, memory_type: &str, user_id: &str) -> Memory {
    let now = Utc::now().to_rfc3339();
    Memory {
        id: id.to_string(),
        content: content.to_string(),
        memory_type: memory_type.to_string(),
        user_id: user_id.to_string(),
        agent_id: Some("test-agent".to_string()),
        metadata: HashMap::new(),
        created_at: now.clone(),
        updated_at: now,
    }
}

#[tokio::test]
#[ignore]
async fn test_e2e_hello_plugin_workflow() {
    let wasm_path = get_wasm_plugin_path("hello_plugin");
    if !wasm_path.exists() {
        eprintln!("⚠️  WASM file not found: {wasm_path:?}");
        return;
    }

    println!("\n🧪 Testing Hello Plugin Workflow");

    let manager = PluginManager::new(5);

    // Register plugin
    let plugin = RegisteredPlugin {
        id: "hello".to_string(),
        metadata: PluginMetadata {
            name: "Hello Plugin".to_string(),
            version: "0.1.0".to_string(),
            description: "Test plugin".to_string(),
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
    println!("  ✅ Plugin registered");

    // Load and call plugin
    let input = serde_json::json!({"message": "World"});
    let result = manager
        .call_plugin("hello", "hello", &input.to_string())
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    println!("  ✅ Plugin executed: {output}");

    let response: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(response["greeting"], "Hello, World!");

    // Unregister plugin
    manager.unregister("hello").await.unwrap();
    println!("  ✅ Plugin unregistered");

    println!("✅ Hello Plugin workflow completed successfully\n");
}

#[tokio::test]
#[ignore]
async fn test_e2e_memory_processor_workflow() {
    let wasm_path = get_wasm_plugin_path("memory_processor_plugin");
    if !wasm_path.exists() {
        eprintln!("⚠️  WASM file not found: {wasm_path:?}");
        return;
    }

    println!("\n🧪 Testing Memory Processor Workflow");

    let manager = PluginManager::new(5);

    // Register plugin
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

    manager.register(plugin).await.unwrap();
    println!("  ✅ Plugin registered");

    // Create test memory
    let memory = create_test_memory(
        "mem-1",
        "This is a test memory with multiple lines.\n\nIt has some content that should be processed.\n",
        "note",
        "user123"
    );

    // Process memory
    let input = serde_json::to_string(&memory).unwrap();
    let result = manager
        .call_plugin("memory-processor", "process_memory", &input)
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    println!("  ✅ Memory processed");

    let processed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(processed["processed"].as_bool().unwrap());
    assert!(processed["processing_info"]
        .as_str()
        .unwrap()
        .contains("words"));
    println!("  📊 Processing info: {}", processed["processing_info"]);

    // Extract keywords
    let result = manager
        .call_plugin("memory-processor", "extract_keywords", &input)
        .await;
    assert!(result.is_ok());
    let keywords_output = result.unwrap();
    println!("  ✅ Keywords extracted");

    let keywords: serde_json::Value = serde_json::from_str(&keywords_output).unwrap();
    assert!(keywords["keywords"].is_array());
    println!("  🔑 Keywords: {:?}", keywords["keywords"]);

    // Summarize memory
    let result = manager
        .call_plugin("memory-processor", "summarize_memory", &input)
        .await;
    assert!(result.is_ok());
    let summary_output = result.unwrap();
    println!("  ✅ Memory summarized");

    let summary: serde_json::Value = serde_json::from_str(&summary_output).unwrap();
    assert!(summary["summary"].is_string());
    println!("  📝 Summary: {}", summary["summary"]);

    println!("✅ Memory Processor workflow completed successfully\n");
}

#[tokio::test]
#[ignore]
async fn test_e2e_code_analyzer_workflow() {
    let wasm_path = get_wasm_plugin_path("code_analyzer_plugin");
    if !wasm_path.exists() {
        eprintln!("⚠️  WASM file not found: {wasm_path:?}");
        return;
    }

    println!("\n🧪 Testing Code Analyzer Workflow");

    let manager = PluginManager::new(5);

    // Register plugin
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

    manager.register(plugin).await.unwrap();
    println!("  ✅ Plugin registered");

    // Analyze Rust code
    let rust_code = r#"
        use std::collections::HashMap;
        
        fn calculate_sum(a: i32, b: i32) -> i32 {
            a + b
        }
        
        pub fn process_data(data: Vec<i32>) -> i32 {
            data.iter().sum()
        }
    "#;

    let input = serde_json::json!({
        "code": rust_code,
        "language": "rust",
        "file_path": "test.rs"
    });

    let result = manager
        .call_plugin("code-analyzer", "analyze_code", &input.to_string())
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    println!("  ✅ Rust code analyzed");

    let analysis: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(analysis["language"], "rust");
    assert!(analysis["functions"].is_array());
    let functions = analysis["functions"].as_array().unwrap();
    assert!(functions.len() >= 2);
    println!("  🔍 Found {} functions", functions.len());

    // Analyze Python code
    let python_code = r#"
def hello(name):
    return f"Hello, {name}"

def calculate(x, y):
    return x + y

import math
from typing import List
    "#;

    let input = serde_json::json!({
        "code": python_code,
        "language": "python",
        "file_path": "test.py"
    });

    let result = manager
        .call_plugin("code-analyzer", "analyze_code", &input.to_string())
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    println!("  ✅ Python code analyzed");

    let analysis: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(analysis["language"], "python");
    assert!(analysis["functions"].is_array());
    let functions = analysis["functions"].as_array().unwrap();
    assert!(functions.len() >= 2);
    println!("  🔍 Found {} functions", functions.len());

    println!("✅ Code Analyzer workflow completed successfully\n");
}

#[tokio::test]
#[ignore]
async fn test_e2e_multiple_plugins_concurrent() {
    println!("\n🧪 Testing Multiple Plugins Concurrently");

    let manager = PluginManager::new(10);

    // Register all plugins
    let plugins = vec![
        ("hello", "hello_plugin"),
        ("memory-processor", "memory_processor_plugin"),
        ("code-analyzer", "code_analyzer_plugin"),
    ];

    for (id, name) in &plugins {
        let wasm_path = get_wasm_plugin_path(name);
        if !wasm_path.exists() {
            eprintln!("⚠️  Skipping {id}: WASM file not found");
            continue;
        }

        let plugin_type = match *id {
            "hello" => PluginType::Custom("greeting".to_string()),
            "memory-processor" => PluginType::MemoryProcessor,
            "code-analyzer" => PluginType::CodeAnalyzer,
            _ => PluginType::Custom("unknown".to_string()),
        };

        let plugin = RegisteredPlugin {
            id: id.to_string(),
            metadata: PluginMetadata {
                name: id.to_string(),
                version: "0.1.0".to_string(),
                description: format!("{id} plugin"),
                author: "Test".to_string(),
                plugin_type,
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
        println!("  ✅ Registered plugin: {id}");
    }

    // Call plugins concurrently
    let mut handles = vec![];

    // Hello plugin calls
    for i in 0..5 {
        let mgr = manager.clone();
        let handle = tokio::spawn(async move {
            let input = serde_json::json!({"message": format!("User{}", i)});
            mgr.call_plugin("hello", "hello", &input.to_string()).await
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }

    println!("  ✅ All concurrent calls completed successfully");

    // List all plugins
    let all_plugins = manager.list_plugins().await;
    println!("  📋 Total registered plugins: {}", all_plugins.len());

    println!("✅ Concurrent plugin test completed successfully\n");
}

#[tokio::test]
#[ignore]
async fn test_e2e_plugin_lifecycle() {
    println!("\n🧪 Testing Plugin Lifecycle");

    let manager = PluginManager::new(5);
    let wasm_path = get_wasm_plugin_path("hello_plugin");

    if !wasm_path.exists() {
        eprintln!("⚠️  WASM file not found");
        return;
    }

    // 1. Register
    let plugin = RegisteredPlugin {
        id: "lifecycle-test".to_string(),
        metadata: PluginMetadata {
            name: "Lifecycle Test".to_string(),
            version: "0.1.0".to_string(),
            description: "Test plugin lifecycle".to_string(),
            author: "Test".to_string(),
            plugin_type: PluginType::Custom("test".to_string()),
            required_capabilities: vec![],
            config_schema: None,
        },
        path: wasm_path.to_string_lossy().to_string(),
        status: PluginStatus::Registered,
        config: PluginConfig::default(),
        registered_at: Utc::now(),
        last_loaded_at: None,
    };

    manager.register(plugin).await.unwrap();
    println!("  ✅ State: Registered");

    // 2. Load (happens automatically on first call)
    let input = serde_json::json!({"message": "test"});
    let result = manager
        .call_plugin("lifecycle-test", "hello", &input.to_string())
        .await;
    assert!(result.is_ok());
    println!("  ✅ State: Loaded and Running");

    // 3. Multiple calls (using cache)
    for i in 0..3 {
        let input = serde_json::json!({"message": format!("call-{}", i)});
        let result = manager
            .call_plugin("lifecycle-test", "hello", &input.to_string())
            .await;
        assert!(result.is_ok());
    }
    println!("  ✅ Multiple calls executed (using cache)");

    // 4. Unregister
    manager.unregister("lifecycle-test").await.unwrap();
    println!("  ✅ State: Unregistered");

    // 5. Verify unregistered
    let all_plugins = manager.list_plugins().await;
    assert!(!all_plugins.iter().any(|p| p.id == "lifecycle-test"));
    println!("  ✅ Plugin removed from registry");

    println!("✅ Plugin lifecycle test completed successfully\n");
}
