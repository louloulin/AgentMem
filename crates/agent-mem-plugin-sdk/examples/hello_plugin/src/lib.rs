//! Hello World Plugin Example

use agent_mem_plugin_sdk::*;
use extism_pdk::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Input {
    message: String,
}

#[derive(Serialize)]
struct Output {
    greeting: String,
}

/// Main plugin function
#[plugin_fn]
pub fn hello(input: String) -> FnResult<String> {
    // Parse input
    let input: Input = serde_json::from_str(&input)?;
    
    // Process
    let greeting = format!("Hello, {}!", input.message);
    
    // Log the action
    if let Err(e) = host::log("info", &format!("Greeting: {}", greeting)) {
        eprintln!("Failed to log: {}", e);
    }
    
    // Build output
    let output = Output { greeting };
    
    // Return result
    Ok(serde_json::to_string(&output)?)
}

/// Plugin metadata
#[plugin_fn]
pub fn metadata() -> FnResult<String> {
    let metadata = plugin_metadata!(
        name: "hello-plugin",
        version: "0.1.0",
        description: "A simple hello world plugin",
        author: "AgentMem Team",
        plugin_type: PluginType::Custom("greeting".to_string()),
        capabilities: [Capability::LoggingAccess]
    );
    
    Ok(serde_json::to_string(&metadata)?)
}

