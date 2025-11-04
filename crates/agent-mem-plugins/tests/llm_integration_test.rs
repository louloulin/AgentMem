//! LLM Capability Integration Tests

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
#[ignore]
async fn test_llm_plugin_summarize() {
    let wasm_path = get_wasm_plugin_path("llm_plugin");
    
    if !wasm_path.exists() {
        eprintln!("⚠️  WASM file not found: {:?}", wasm_path);
        eprintln!("   Run: ./build_plugins.sh");
        return;
    }
    
    println!("\n🧪 Testing LLM Plugin - Summarize");
    
    let manager = PluginManager::new(10);
    
    let plugin = RegisteredPlugin {
        id: "llm-plugin".to_string(),
        metadata: PluginMetadata {
            name: "LLM Plugin".to_string(),
            version: "0.1.0".to_string(),
            description: "LLM-powered plugin".to_string(),
            author: "Test".to_string(),
            plugin_type: PluginType::Custom("llm".to_string()),
            required_capabilities: vec![Capability::LLMAccess],
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
    
    // Test summarize
    let input = serde_json::json!({
        "text": "This is a long text that needs to be summarized. It contains multiple sentences. The plugin should extract the key points and create a concise summary.",
        "max_length": 100
    });
    
    let result = manager
        .call_plugin("llm-plugin", "summarize", &input.to_string())
        .await;
    
    match result {
        Ok(output) => {
            println!("  ✅ Summarize function executed");
            let response: serde_json::Value = serde_json::from_str(&output).unwrap();
            println!("  📝 Summary: {}", response["summary"]);
            assert!(response["summary"].is_string());
            assert!(response["original_length"].as_u64().unwrap() > 0);
        }
        Err(e) => {
            eprintln!("  ❌ Failed: {}", e);
            panic!("Summarize test failed");
        }
    }
    
    println!("✅ LLM Plugin summarize test completed\n");
}

#[tokio::test]
#[ignore]
async fn test_llm_plugin_translate() {
    let wasm_path = get_wasm_plugin_path("llm_plugin");
    
    if !wasm_path.exists() {
        eprintln!("⚠️  WASM file not found: {:?}", wasm_path);
        return;
    }
    
    println!("\n🧪 Testing LLM Plugin - Translate");
    
    let manager = PluginManager::new(10);
    
    let plugin = RegisteredPlugin {
        id: "llm-translate".to_string(),
        metadata: PluginMetadata {
            name: "LLM Plugin".to_string(),
            version: "0.1.0".to_string(),
            description: "LLM-powered plugin".to_string(),
            author: "Test".to_string(),
            plugin_type: PluginType::Custom("llm".to_string()),
            required_capabilities: vec![Capability::LLMAccess],
            config_schema: None,
        },
        path: wasm_path.to_string_lossy().to_string(),
        status: PluginStatus::Registered,
        config: PluginConfig::default(),
        registered_at: Utc::now(),
        last_loaded_at: None,
    };
    
    manager.register(plugin).await.unwrap();
    
    // Test translate
    let input = serde_json::json!({
        "text": "Hello, how are you?",
        "target_language": "zh-CN"
    });
    
    let result = manager
        .call_plugin("llm-translate", "translate", &input.to_string())
        .await;
    
    match result {
        Ok(output) => {
            println!("  ✅ Translate function executed");
            let response: serde_json::Value = serde_json::from_str(&output).unwrap();
            println!("  🌐 Translation: {}", response["translated_text"]);
            assert!(response["translated_text"].is_string());
            assert_eq!(response["target_language"], "zh-CN");
        }
        Err(e) => {
            eprintln!("  ❌ Failed: {}", e);
            panic!("Translate test failed");
        }
    }
    
    println!("✅ LLM Plugin translate test completed\n");
}

#[tokio::test]
#[ignore]
async fn test_llm_plugin_answer_question() {
    let wasm_path = get_wasm_plugin_path("llm_plugin");
    
    if !wasm_path.exists() {
        eprintln!("⚠️  WASM file not found: {:?}", wasm_path);
        return;
    }
    
    println!("\n🧪 Testing LLM Plugin - Answer Question");
    
    let manager = PluginManager::new(10);
    
    let plugin = RegisteredPlugin {
        id: "llm-qa".to_string(),
        metadata: PluginMetadata {
            name: "LLM Plugin".to_string(),
            version: "0.1.0".to_string(),
            description: "LLM-powered plugin".to_string(),
            author: "Test".to_string(),
            plugin_type: PluginType::Custom("llm".to_string()),
            required_capabilities: vec![Capability::LLMAccess],
            config_schema: None,
        },
        path: wasm_path.to_string_lossy().to_string(),
        status: PluginStatus::Registered,
        config: PluginConfig::default(),
        registered_at: Utc::now(),
        last_loaded_at: None,
    };
    
    manager.register(plugin).await.unwrap();
    
    // Test answer_question
    let input = serde_json::json!({
        "context": "The AgentMem plugin system uses WebAssembly for security and performance.",
        "question": "What does AgentMem use for plugins?"
    });
    
    let result = manager
        .call_plugin("llm-qa", "answer_question", &input.to_string())
        .await;
    
    match result {
        Ok(output) => {
            println!("  ✅ Answer question function executed");
            let response: serde_json::Value = serde_json::from_str(&output).unwrap();
            println!("  💬 Answer: {}", response["answer"]);
            assert!(response["answer"].is_string());
            assert!(response["confidence"].as_f64().unwrap() > 0.0);
        }
        Err(e) => {
            eprintln!("  ❌ Failed: {}", e);
            panic!("Answer question test failed");
        }
    }
    
    println!("✅ LLM Plugin Q&A test completed\n");
}

#[test]
fn test_llm_capability_api() {
    println!("\n📖 LLM Capability API:");
    println!("   • call_llm(request) -> LlmResponse");
    println!("   • get_history() -> Vec<LlmRequest>");
    println!("   • clear_history()");
    println!("\n💡 LLM Plugin Functions:");
    println!("   • summarize(text, max_length) -> summary");
    println!("   • translate(text, target_language) -> translation");
    println!("   • answer_question(context, question) -> answer");
}

