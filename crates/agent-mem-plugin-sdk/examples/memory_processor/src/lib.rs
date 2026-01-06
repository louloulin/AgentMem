//! Memory Processor Plugin Example

use agent_mem_plugin_sdk::{self as sdk, *};
use extism_pdk::*;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
struct ProcessedMemory {
    id: String,
    content: String,
    metadata: HashMap<String, serde_json::Value>,
    processed: bool,
    processing_info: String,
}

/// Process a memory object
#[plugin_fn]
pub fn process_memory(input: String) -> FnResult<String> {
    // Parse input
    let memory: sdk::Memory = serde_json::from_str(&input)?;
    
    // Process memory content
    let processed_content = clean_and_format(&memory.content);
    
    // Extract metadata
    let word_count = processed_content.split_whitespace().count();
    let char_count = processed_content.chars().count();
    
    let mut metadata = memory.metadata.clone();
    metadata.insert(
        "word_count".to_string(),
        serde_json::Value::Number(word_count.into())
    );
    metadata.insert(
        "char_count".to_string(),
        serde_json::Value::Number(char_count.into())
    );
    metadata.insert(
        "processed".to_string(),
        serde_json::Value::Bool(true)
    );
    
    // Create processed result
    let processed = ProcessedMemory {
        id: memory.id.clone(),
        content: processed_content,
        metadata: metadata.clone(),
        processed: true,
        processing_info: format!("Processed {} words, {} characters", word_count, char_count),
    };
    
    // Log processing
    let _ = host::log(
        "info",
        &format!("Processed memory {} with {} words", memory.id, word_count),
    );
    
    // Return processed memory
    Ok(serde_json::to_string(&processed)?)
}

/// Clean and format memory content
fn clean_and_format(content: &str) -> String {
    content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Extract keywords from content
#[plugin_fn]
pub fn extract_keywords(input: String) -> FnResult<String> {
    let memory: sdk::Memory = serde_json::from_str(&input)?;
    
    // Simple keyword extraction (top frequent words)
    let content_lower = memory.content.to_lowercase();
    let words: Vec<&str> = content_lower
        .split_whitespace()
        .filter(|w| w.len() > 3) // Only words longer than 3 chars
        .collect();
    
    let mut word_freq: HashMap<String, usize> = HashMap::new();
    for word in words {
        *word_freq.entry(word.to_string()).or_insert(0) += 1;
    }
    
    let mut freq_vec: Vec<_> = word_freq.iter().collect();
    freq_vec.sort_by(|a, b| b.1.cmp(a.1));
    
    let keywords: Vec<String> = freq_vec
        .iter()
        .take(10)
        .map(|(word, _count)| word.to_string())
        .collect();
    
    Ok(serde_json::json!({
        "memory_id": memory.id,
        "keywords": keywords,
    }).to_string())
}

/// Summarize memory content
#[plugin_fn]
pub fn summarize_memory(input: String) -> FnResult<String> {
    let memory: sdk::Memory = serde_json::from_str(&input)?;
    
    // Simple summarization: first 200 characters
    let summary = if memory.content.len() > 200 {
        format!("{}...", memory.content.chars().take(200).collect::<String>())
    } else {
        memory.content.clone()
    };
    
    let word_count = memory.content.split_whitespace().count();
    
    Ok(serde_json::json!({
        "memory_id": memory.id,
        "summary": summary,
        "original_length": memory.content.len(),
        "word_count": word_count,
    }).to_string())
}

/// Plugin metadata
#[plugin_fn]
pub fn metadata() -> FnResult<String> {
    let metadata = plugin_metadata!(
        name: "memory-processor",
        version: "0.1.0",
        description: "Memory processing and analysis plugin",
        author: "AgentMem Team",
        plugin_type: PluginType::MemoryProcessor,
        capabilities: [Capability::MemoryAccess, Capability::LoggingAccess]
    );
    
    Ok(serde_json::to_string(&metadata)?)
}
