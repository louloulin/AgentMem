//! Utility functions for Mem0 compatibility

use crate::types::MemoryFilter;
use agent_mem_traits::{MemoryType, Session};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Convert Mem0 memory type string to AgentMem MemoryType
pub fn parse_memory_type(type_str: &str) -> MemoryType {
    match type_str.to_lowercase().as_str() {
        "episodic" => MemoryType::Episodic,
        "semantic" => MemoryType::Semantic,
        "procedural" => MemoryType::Procedural,
        "working" => MemoryType::Working,
        _ => MemoryType::Episodic, // Default fallback
    }
}

/// Convert AgentMem MemoryType to Mem0 memory type string
pub fn memory_type_to_string(memory_type: &MemoryType) -> String {
    match memory_type {
        MemoryType::Factual => "factual".to_string(),
        MemoryType::Episodic => "episodic".to_string(),
        MemoryType::Semantic => "semantic".to_string(),
        MemoryType::Procedural => "procedural".to_string(),
        MemoryType::Working => "working".to_string(),
        MemoryType::Core => "core".to_string(),
        MemoryType::Resource => "resource".to_string(),
        MemoryType::Knowledge => "knowledge".to_string(),
        MemoryType::Contextual => "contextual".to_string(),
    }
}

/// Create a session from Mem0 parameters
pub fn create_session(
    user_id: Option<String>,
    _agent_id: Option<String>,
    run_id: Option<String>,
) -> Session {
    let mut session = Session::new();

    if let Some(user_id) = user_id {
        session = session.with_user_id(Some(user_id));
    }

    if let Some(run_id) = run_id {
        session = session.with_run_id(Some(run_id));
    }

    session
}

/// Validate user ID format
pub fn validate_user_id(user_id: &str) -> Result<(), crate::error::Mem0Error> {
    if user_id.is_empty() {
        return Err(crate::error::Mem0Error::InvalidUserId {
            user_id: user_id.to_string(),
        });
    }

    if user_id.len() > 255 {
        return Err(crate::error::Mem0Error::InvalidUserId {
            user_id: "User ID too long (max 255 characters)".to_string(),
        });
    }

    // Check for invalid characters
    if user_id.contains('\0') || user_id.contains('\n') || user_id.contains('\r') {
        return Err(crate::error::Mem0Error::InvalidUserId {
            user_id: "User ID contains invalid characters".to_string(),
        });
    }

    Ok(())
}

/// Validate memory content
pub fn validate_memory_content(content: &str) -> Result<(), crate::error::Mem0Error> {
    if content.is_empty() {
        return Err(crate::error::Mem0Error::InvalidContent {
            reason: "Memory content cannot be empty".to_string(),
        });
    }

    if content.len() > 100_000 {
        return Err(crate::error::Mem0Error::InvalidContent {
            reason: "Memory content too long (max 100,000 characters)".to_string(),
        });
    }

    Ok(())
}

/// Sanitize metadata values
pub fn sanitize_metadata(metadata: &mut HashMap<String, serde_json::Value>) {
    // Remove null values
    metadata.retain(|_, v| !v.is_null());

    // Limit string values to reasonable length
    for (_, value) in metadata.iter_mut() {
        if let serde_json::Value::String(s) = value {
            if s.len() > 10_000 {
                *s = format!("{}...[truncated]", &s[..10_000]);
            }
        }
    }
}

/// Convert Mem0 filter to AgentMem query parameters
pub fn convert_filter_to_query_params(filter: &MemoryFilter) -> HashMap<String, String> {
    let mut params = HashMap::new();

    if let Some(agent_id) = &filter.agent_id {
        params.insert("agent_id".to_string(), agent_id.clone());
    }

    if let Some(run_id) = &filter.run_id {
        params.insert("run_id".to_string(), run_id.clone());
    }

    if let Some(memory_type) = &filter.memory_type {
        params.insert("memory_type".to_string(), memory_type.clone());
    }

    if let Some(created_after) = &filter.created_after {
        params.insert("created_after".to_string(), created_after.to_rfc3339());
    }

    if let Some(created_before) = &filter.created_before {
        params.insert("created_before".to_string(), created_before.to_rfc3339());
    }

    if let Some(limit) = filter.limit {
        params.insert("limit".to_string(), limit.to_string());
    }

    if let Some(offset) = filter.offset {
        params.insert("offset".to_string(), offset.to_string());
    }

    params
}

/// Generate a unique memory ID
pub fn generate_memory_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Calculate memory importance score based on content and metadata
pub fn calculate_importance_score(
    content: &str,
    metadata: &HashMap<String, serde_json::Value>,
) -> f32 {
    let mut score = 0.5; // Base score

    // Adjust based on content length
    let content_len = content.len();
    if content_len > 1000 {
        score += 0.1;
    } else if content_len < 50 {
        score -= 0.1;
    }

    // Adjust based on metadata
    if let Some(importance) = metadata.get("importance") {
        if let Some(importance_val) = importance.as_f64() {
            score = (score + importance_val as f32) / 2.0;
        }
    }

    // Check for priority indicators
    let content_lower = content.to_lowercase();
    if content_lower.contains("important") || content_lower.contains("critical") {
        score += 0.2;
    }

    if content_lower.contains("urgent") || content_lower.contains("priority") {
        score += 0.15;
    }

    // Clamp to valid range
    score.clamp(0.0, 1.0)
}

/// Extract keywords from memory content for better searchability
pub fn extract_keywords(content: &str) -> Vec<String> {
    // Simple keyword extraction - in a real implementation, you might use NLP libraries
    let words: Vec<String> = content
        .split_whitespace()
        .filter(|word| word.len() > 3) // Filter out short words
        .filter(|word| !is_stop_word(word))
        .map(|word| {
            word.to_lowercase()
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_string()
        })
        .filter(|word| !word.is_empty())
        .collect();

    // Remove duplicates and return
    let mut unique_words: Vec<String> = words.into_iter().collect();
    unique_words.sort();
    unique_words.dedup();
    unique_words
}

/// Check if a word is a stop word
fn is_stop_word(word: &str) -> bool {
    const STOP_WORDS: &[&str] = &[
        "the",
        "and",
        "or",
        "but",
        "in",
        "on",
        "at",
        "to",
        "for",
        "of",
        "with",
        "by",
        "from",
        "up",
        "about",
        "into",
        "through",
        "during",
        "before",
        "after",
        "above",
        "below",
        "between",
        "among",
        "within",
        "without",
        "under",
        "over",
        "this",
        "that",
        "these",
        "those",
        "i",
        "you",
        "he",
        "she",
        "it",
        "we",
        "they",
        "me",
        "him",
        "her",
        "us",
        "them",
        "my",
        "your",
        "his",
        "her",
        "its",
        "our",
        "their",
        "mine",
        "yours",
        "ours",
        "theirs",
        "myself",
        "yourself",
        "himself",
        "herself",
        "itself",
        "ourselves",
        "yourselves",
        "themselves",
        "what",
        "which",
        "who",
        "whom",
        "whose",
        "where",
        "when",
        "why",
        "how",
        "all",
        "any",
        "both",
        "each",
        "few",
        "more",
        "most",
        "other",
        "some",
        "such",
        "no",
        "nor",
        "not",
        "only",
        "own",
        "same",
        "so",
        "than",
        "too",
        "very",
        "can",
        "will",
        "just",
        "should",
        "now",
    ];

    STOP_WORDS.contains(&word.to_lowercase().as_str())
}

/// Format timestamp for display
pub fn format_timestamp(timestamp: &DateTime<Utc>) -> String {
    timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_memory_type() {
        assert_eq!(parse_memory_type("episodic"), MemoryType::Episodic);
        assert_eq!(parse_memory_type("SEMANTIC"), MemoryType::Semantic);
        assert_eq!(parse_memory_type("procedural"), MemoryType::Procedural);
        assert_eq!(parse_memory_type("working"), MemoryType::Working);
        assert_eq!(parse_memory_type("unknown"), MemoryType::Episodic);
    }

    #[test]
    fn test_validate_user_id() {
        assert!(validate_user_id("valid_user_123").is_ok());
        assert!(validate_user_id("").is_err());
        assert!(validate_user_id(&"x".repeat(256)).is_err());
        assert!(validate_user_id("user\0id").is_err());
    }

    #[test]
    fn test_validate_memory_content() {
        assert!(validate_memory_content("Valid content").is_ok());
        assert!(validate_memory_content("").is_err());
        assert!(validate_memory_content(&"x".repeat(100_001)).is_err());
    }

    #[test]
    fn test_calculate_importance_score() {
        let metadata = HashMap::new();
        let score = calculate_importance_score("This is important information", &metadata);
        assert!(score > 0.5);

        let score = calculate_importance_score("urgent task", &metadata);
        assert!(score > 0.5); // 0.5 - 0.1 + 0.15 = 0.55
    }

    #[test]
    fn test_extract_keywords() {
        let keywords = extract_keywords("This is an important message about machine learning");
        assert!(keywords.contains(&"important".to_string()));
        assert!(keywords.contains(&"message".to_string()));
        assert!(keywords.contains(&"machine".to_string()));
        assert!(keywords.contains(&"learning".to_string()));
        assert!(!keywords.contains(&"this".to_string())); // Stop word
        assert!(!keywords.contains(&"is".to_string())); // Stop word
    }
}
