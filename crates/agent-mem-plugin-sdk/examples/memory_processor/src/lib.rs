//! Memory Processor Plugin Example

use agent_mem_plugin_sdk::*;
use extism_pdk::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct ProcessedMemory {
    id: String,
    content: String,
    memory_type: String,
    metadata: serde_json::Value,
    processed: bool,
    processing_info: String,
}

/// Process a memory object
#[plugin_fn]
pub fn process_memory(input: String) -> FnResult<String> {
    // Parse input
    let memory: Memory = serde_json::from_str(&input)?;
    
    // Process memory content
    let processed_content = clean_and_format(&memory.content);
    
    // Extract metadata
    let extracted_metadata = extract_metadata(&processed_content);
    
    // Build processed memory
    let processed = ProcessedMemory {
        id: memory.id,
        content: processed_content,
        memory_type: memory.memory_type,
        metadata: serde_json::to_value(extracted_metadata)?,
        processed: true,
        processing_info: "Cleaned and formatted".to_string(),
    };
    
    // Log processing
    let _ = host::log("info", &format!("Processed memory: {}", processed.id));
    
    // Return result
    Ok(serde_json::to_string(&processed)?)
}

/// Clean and format text
fn clean_and_format(content: &str) -> String {
    content
        .trim()
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Extract metadata from content
fn extract_metadata(content: &str) -> serde_json::Value {
    serde_json::json!({
        "word_count": content.split_whitespace().count(),
        "line_count": content.lines().count(),
        "char_count": content.chars().count(),
    })
}

/// Plugin metadata
#[plugin_fn]
pub fn metadata() -> FnResult<String> {
    let metadata = plugin_metadata!(
        name: "memory-processor",
        version: "0.1.0",
        description: "Memory content processor and formatter",
        author: "AgentMem Team",
        plugin_type: PluginType::MemoryProcessor,
        capabilities: [Capability::MemoryAccess, Capability::LoggingAccess]
    );
    
    Ok(serde_json::to_string(&metadata)?)
}

