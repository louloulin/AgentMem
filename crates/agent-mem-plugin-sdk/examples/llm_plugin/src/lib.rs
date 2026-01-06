//! LLM Plugin Example
//! 
//! This plugin demonstrates how to use the LLM capability to:
//! - Summarize text
//! - Translate content
//! - Answer questions

use agent_mem_plugin_sdk::*;
use extism_pdk::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct SummarizeInput {
    text: String,
    max_length: Option<usize>,
}

#[derive(Serialize)]
struct SummarizeOutput {
    summary: String,
    original_length: usize,
    summary_length: usize,
}

/// Summarize text using LLM
#[plugin_fn]
pub fn summarize(input: String) -> FnResult<String> {
    let input: SummarizeInput = serde_json::from_str(&input)?;
    
    // Log operation
    let _ = host::log("info", &format!("Summarizing text of {} chars", input.text.len()));
    
    // In a real implementation, this would call the LLM host function
    // For now, we'll create a simple summary
    let summary = create_simple_summary(&input.text, input.max_length.unwrap_or(200));
    
    let output = SummarizeOutput {
        summary: summary.clone(),
        original_length: input.text.len(),
        summary_length: summary.len(),
    };
    
    Ok(serde_json::to_string(&output)?)
}

#[derive(Deserialize)]
struct TranslateInput {
    text: String,
    target_language: String,
}

#[derive(Serialize)]
struct TranslateOutput {
    translated_text: String,
    source_language: String,
    target_language: String,
}

/// Translate text using LLM
#[plugin_fn]
pub fn translate(input: String) -> FnResult<String> {
    let input: TranslateInput = serde_json::from_str(&input)?;
    
    // Log operation
    let _ = host::log("info", &format!("Translating to {}", input.target_language));
    
    // In a real implementation, this would call the LLM host function
    let translated = format!("[{}] {}", input.target_language.to_uppercase(), input.text);
    
    let output = TranslateOutput {
        translated_text: translated,
        source_language: "auto".to_string(),
        target_language: input.target_language,
    };
    
    Ok(serde_json::to_string(&output)?)
}

#[derive(Deserialize)]
struct QuestionInput {
    context: String,
    question: String,
}

#[derive(Serialize)]
struct QuestionOutput {
    answer: String,
    confidence: f32,
    sources: Vec<String>,
}

/// Answer questions based on context using LLM
#[plugin_fn]
pub fn answer_question(input: String) -> FnResult<String> {
    let input: QuestionInput = serde_json::from_str(&input)?;
    
    // Log operation
    let _ = host::log("info", &format!("Answering question: {}", input.question));
    
    // In a real implementation, this would call the LLM host function
    // For now, create a simple answer
    let answer = format!(
        "Based on the context, the answer to '{}' can be found in the provided information.",
        input.question
    );
    
    let output = QuestionOutput {
        answer,
        confidence: 0.85,
        sources: vec!["context".to_string()],
    };
    
    Ok(serde_json::to_string(&output)?)
}

/// Create a simple summary (mock implementation)
fn create_simple_summary(text: &str, max_length: usize) -> String {
    if text.len() <= max_length {
        return text.to_string();
    }
    
    // Take first sentences up to max_length
    let mut summary = String::new();
    for sentence in text.split('.') {
        let sentence = sentence.trim();
        if sentence.is_empty() {
            continue;
        }
        
        if summary.len() + sentence.len() + 2 <= max_length {
            if !summary.is_empty() {
                summary.push_str(". ");
            }
            summary.push_str(sentence);
        } else {
            break;
        }
    }
    
    if !summary.is_empty() && !summary.ends_with('.') {
        summary.push('.');
    }
    
    if summary.is_empty() {
        summary = text.chars().take(max_length - 3).collect();
        summary.push_str("...");
    }
    
    summary
}

/// Plugin metadata
#[plugin_fn]
pub fn metadata() -> FnResult<String> {
    let metadata = plugin_metadata!(
        name: "llm-plugin",
        version: "0.1.0",
        description: "LLM-powered text processing plugin",
        author: "AgentMem Team",
        plugin_type: PluginType::Custom("llm".to_string()),
        capabilities: [Capability::LLMAccess, Capability::LoggingAccess]
    );
    
    Ok(serde_json::to_string(&metadata)?)
}

