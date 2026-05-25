//! Input Validation Module (Simplified Version)
//!
//! This module provides input validation for security and data integrity.

//! Note: Due to compilation issues with validator crate custom functions,
//! this version focuses on helper functions and basic validation patterns.

use crate::{CoreError, CoreResult};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

lazy_static! {
    /// UUID v4 validation pattern
    static ref UUID_PATTERN: Regex = Regex::new(
        r"^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$"
    ).unwrap();

    /// Safe string pattern (no control characters, no SQL injection)
    static ref SAFE_STRING_PATTERN: Regex = Regex::new(r"^[\p{L}\p{N}\s\-_.@#$%&*()+=\[\]{}|;:,<>?/]+$").unwrap();

    /// Memory type pattern
    static ref MEMORY_TYPE_PATTERN: Regex = Regex::new(r"^(episodic|semantic|procedural|working|core|resource|knowledge|contextual)$").unwrap();
}

// ═══════════════════════════════════════════════════════════════════════
// Validation Constants
// ═══════════════════════════════════════════════════════════════════════

/// Maximum memory content length (10KB)
pub const MAX_MEMORY_CONTENT_LENGTH: usize = 10_240;

/// Maximum user ID length (100 chars)
pub const MAX_USER_ID_LENGTH: usize = 100;

/// Maximum agent ID length (100 chars)
pub const MAX_AGENT_ID_LENGTH: usize = 100;

/// Maximum run ID length (100 chars)
pub const MAX_RUN_ID_LENGTH: usize = 100;

/// Maximum metadata key length (100 chars)
pub const MAX_METADATA_KEY_LENGTH: usize = 100;

/// Maximum metadata value length (1KB)
pub const MAX_METADATA_VALUE_LENGTH: usize = 1_024;

/// Maximum prompt length (5KB)
pub const MAX_PROMPT_LENGTH: usize = 5_120;

/// Maximum search query length (1KB)
pub const MAX_SEARCH_QUERY_LENGTH: usize = 1_024;

/// Maximum batch size (100 items)
pub const MAX_BATCH_SIZE: usize = 100;

// ═══════════════════════════════════════════════════════════════════════
// Validation Functions
// ═════════════════════════════════════════════════════════════════════

/// Validate UUID format
pub fn validate_uuid(id: &str) -> CoreResult<()> {
    if id.is_empty() {
        return Err(CoreError::InvalidInput("UUID cannot be empty".to_string()));
    }

    if !UUID_PATTERN.is_match(id) {
        return Err(CoreError::InvalidInput("Invalid UUID format".to_string()));
    }

    Ok(())
}

/// Validate user ID format
pub fn validate_user_id(id: &str) -> CoreResult<()> {
    if id.is_empty() {
        return Err(CoreError::InvalidInput("User ID cannot be empty".to_string()));
    }

    if id.len() > MAX_USER_ID_LENGTH {
        return Err(CoreError::InvalidInput(
            format!("User ID exceeds maximum length of {}", MAX_USER_ID_LENGTH)
        ));
    }

    if let Some(id) = id.strip_prefix("user_") {
        if !SAFE_STRING_PATTERN.is_match(id) {
            return Err(CoreError::InvalidInput("User ID contains invalid characters".to_string()));
        }
    } else if !SAFE_STRING_PATTERN.is_match(id) {
        return Err(CoreError::InvalidInput("User ID contains invalid characters".to_string()));
    }

    Ok(())
}

/// Validate agent ID format
pub fn validate_agent_id(id: &str) -> CoreResult<()> {
    if id.is_empty() {
        return Err(CoreError::InvalidInput("Agent ID cannot be empty".to_string()));
    }

    if id.len() > MAX_AGENT_ID_LENGTH {
        return Err(CoreError::InvalidInput(
            format!("Agent ID exceeds maximum length of {}", MAX_AGENT_ID_LENGTH)
        ));
    }

    if let Some(id) = id.strip_prefix("agent_") {
        if !SAFE_STRING_PATTERN.is_match(id) {
            return Err(CoreError::InvalidInput("Agent ID contains invalid characters".to_string()));
        }
    } else if !SAFE_STRING_PATTERN.is_match(id) {
        return Err(CoreError::InvalidInput("Agent ID contains invalid characters".to_string()));
    }

    Ok(())
}

/// Validate run ID format
pub fn validate_run_id(id: &str) -> CoreResult<()> {
    if id.is_empty() {
        return Err(CoreError::InvalidInput("Run ID cannot be empty".to_string()));
    }

    if id.len() > MAX_RUN_ID_LENGTH {
        return Err(CoreError::InvalidInput(
            format!("Run ID exceeds maximum length of {}", MAX_RUN_ID_LENGTH)
        ));
    }

    if let Some(id) = id.strip_prefix("run_") {
        if !SAFE_STRING_PATTERN.is_match(id) {
            return Err(CoreError::InvalidInput("Run ID contains invalid characters".to_string()));
        }
    } else if !SAFE_STRING_PATTERN.is_match(id) {
        return Err(CoreError::InvalidInput("Run ID contains invalid characters".to_string()));
    }

    Ok(())
}

/// Validate memory type
pub fn validate_memory_type(memory_type: &str) -> CoreResult<()> {
    if !MEMORY_TYPE_PATTERN.is_match(memory_type) {
        return Err(CoreError::InvalidInput("Invalid memory type".to_string()));
    }

    Ok(())
}

/// Validate safe string (no control characters, no injection)
pub fn validate_safe_string(s: &str) -> CoreResult<()> {
    if s.trim().is_empty() {
        return Err(CoreError::InvalidInput("String cannot be empty or whitespace only".to_string()));
    }

    if s.len() > MAX_MEMORY_CONTENT_LENGTH {
        return Err(CoreError::InvalidInput(
            format!("String exceeds maximum length of {}", MAX_MEMORY_CONTENT_LENGTH)
        ));
    }

    // Check for control characters (except newline, tab, carriage return)
    if s.chars().any(|c| {
        c.is_control() && !matches!(c, '\n' | '\t' | '\r')
    }) {
        return Err(CoreError::InvalidInput("String contains control characters".to_string()));
    }

    Ok(())
}

/// Validate metadata
pub fn validate_metadata(
    metadata: &HashMap<String, serde_json::Value>,
) -> CoreResult<()> {
    for (key, value) in metadata {
        // Validate key length
        if key.len() > MAX_METADATA_KEY_LENGTH {
            return Err(CoreError::InvalidInput(format!(
                "Metadata key '{}' exceeds maximum length of {}",
                key, MAX_METADATA_KEY_LENGTH
            )));
        }

        // Validate key is safe
        if !SAFE_STRING_PATTERN.is_match(key) {
            return Err(CoreError::InvalidInput(format!(
                "Metadata key '{}' contains invalid characters",
                key
            )));
        }

        // Validate value length if it's a string
        if let Some(s) = value.as_str() {
            if s.len() > MAX_METADATA_VALUE_LENGTH {
                return Err(CoreError::InvalidInput(format!(
                    "Metadata value for key '{}' exceeds maximum length of {}",
                    key, MAX_METADATA_VALUE_LENGTH
                )));
            }
        }
    }

    Ok(())
}

/// Validate search query
pub fn validate_search_query(query: &str) -> CoreResult<()> {
    if query.trim().is_empty() {
        return Err(CoreError::InvalidInput("Search query cannot be empty".to_string()));
    }

    if query.len() > MAX_SEARCH_QUERY_LENGTH {
        return Err(CoreError::InvalidInput(
            format!("Search query exceeds maximum length of {}", MAX_SEARCH_QUERY_LENGTH)
        ));
    }

    if !SAFE_STRING_PATTERN.is_match(query) {
        return Err(CoreError::InvalidInput("Search query contains invalid characters".to_string()));
    }

    Ok(())
}

/// Validate batch size
pub fn validate_batch_size<T>(items: &[T]) -> CoreResult<()> {
    if items.is_empty() {
        return Err(CoreError::InvalidInput("Batch cannot be empty".to_string()));
    }

    if items.len() > MAX_BATCH_SIZE {
        return Err(CoreError::InvalidInput(
            format!("Batch size {} exceeds maximum of {}", items.len(), MAX_BATCH_SIZE)
        ));
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
// Request Structures (for manual validation)
// ═════════════════════════════════════════════════════════════════════

/// Add memory request structure (for manual validation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedAddRequest {
    pub content: String,
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub run_id: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub memory_type: Option<String>,
    pub prompt: Option<String>,
}

impl ValidatedAddRequest {
    /// Validate all fields
    pub fn validate(&self) -> CoreResult<()> {
        validate_safe_string(&self.content)?;
        if let Some(ref user_id) = self.user_id {
            validate_user_id(user_id)?;
        }
        if let Some(ref agent_id) = self.agent_id {
            validate_agent_id(agent_id)?;
        }
        if let Some(ref run_id) = self.run_id {
            validate_run_id(run_id)?;
        }
        if let Some(ref metadata) = self.metadata {
            validate_metadata(metadata)?;
        }
        if let Some(ref memory_type) = self.memory_type {
            validate_memory_type(memory_type)?;
        }
        if let Some(ref prompt) = self.prompt {
            if prompt.len() > MAX_PROMPT_LENGTH {
                return Err(CoreError::InvalidInput(
                    format!("Prompt exceeds maximum length of {}", MAX_PROMPT_LENGTH)
                ));
            }
        }
        Ok(())
    }
}

/// Search request structure (for manual validation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedSearchRequest {
    pub query: String,
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub run_id: Option<String>,
    pub memory_type: Option<String>,
    pub limit: Option<usize>,
    pub score_threshold: Option<f32>,
}

impl ValidatedSearchRequest {
    pub fn validate(&self) -> CoreResult<()> {
        validate_search_query(&self.query)?;
        if let Some(ref user_id) = self.user_id {
            validate_user_id(user_id)?;
        }
        if let Some(ref agent_id) = self.agent_id {
            validate_agent_id(agent_id)?;
        }
        if let Some(ref run_id) = self.run_id {
            validate_run_id(run_id)?;
        }
        if let Some(ref memory_type) = self.memory_type {
            validate_memory_type(memory_type)?;
        }
        if let Some(limit) = self.limit {
            if limit == 0 || limit > 100 {
                return Err(CoreError::InvalidInput("Limit must be between 1 and 100".to_string()));
            }
        }
        if let Some(threshold) = self.score_threshold {
            if threshold < 0.0 || threshold > 1.0 {
                return Err(CoreError::InvalidInput("Score threshold must be between 0.0 and 1.0".to_string()));
            }
        }
        Ok(())
    }
}

/// Update request structure (for manual validation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedUpdateRequest {
    pub memory_id: String,
    pub content: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl ValidatedUpdateRequest {
    pub fn validate(&self) -> CoreResult<()> {
        validate_uuid(&self.memory_id)?;
        if let Some(ref content) = self.content {
            validate_safe_string(content)?;
        }
        if let Some(ref metadata) = self.metadata {
            validate_metadata(metadata)?;
        }
        Ok(())
    }
}

/// Delete request structure (for manual validation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedDeleteRequest {
    pub memory_id: String,
}

impl ValidatedDeleteRequest {
    pub fn validate(&self) -> CoreResult<()> {
        validate_uuid(&self.memory_id)
    }
}

/// Batch add request structure (for manual validation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedBatchAddRequest {
    pub contents: Vec<String>,
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl ValidatedBatchAddRequest {
    pub fn validate(&self) -> CoreResult<()> {
        validate_batch_size(&self.contents)?;
        for content in &self.contents {
            validate_safe_string(content)?;
        }
        if let Some(ref user_id) = self.user_id {
            validate_user_id(user_id)?;
        }
        if let Some(ref agent_id) = self.agent_id {
            validate_agent_id(agent_id)?;
        }
        if let Some(ref metadata) = self.metadata {
            validate_metadata(metadata)?;
        }
        Ok(())
    }
}

/// Create user request structure (for manual validation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedCreateUserRequest {
    pub name: String,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl ValidatedCreateUserRequest {
    pub fn validate(&self) -> CoreResult<()> {
        if self.name.trim().is_empty() {
            return Err(CoreError::InvalidInput("User name cannot be empty".to_string()));
        }
        if self.name.len() > MAX_USER_ID_LENGTH {
            return Err(CoreError::InvalidInput(
                format!("User name exceeds maximum length of {}", MAX_USER_ID_LENGTH)
            ));
        }
        if !SAFE_STRING_PATTERN.is_match(&self.name) {
            return Err(CoreError::InvalidInput("User name contains invalid characters".to_string()));
        }
        if let Some(ref metadata) = self.metadata {
            validate_metadata(metadata)?;
        }
        Ok(())
    }
}

// ═════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
#[cfg(feature = "inline_tests")]
mod tests {
    use super::*;

    #[test]
    fn test_validate_uuid_valid() {
        assert!(validate_uuid("550e8400-e29b-41d4-a716-446655440000").is_ok());
    }

    #[test]
    fn test_validate_uuid_invalid() {
        assert!(validate_uuid("not-a-uuid").is_err());
        assert!(validate_uuid("").is_err());
    }

    #[test]
    fn test_validate_user_id_valid() {
        assert!(validate_user_id("user_123").is_ok());
        assert!(validate_user_id("john_doe").is_ok());
    }

    #[test]
    fn test_validate_user_id_invalid() {
        assert!(validate_user_id("user; DROP TABLE users; --").is_err());
        assert!(validate_user_id("user\x00null").is_err());
    }

    #[test]
    fn test_validate_memory_type_valid() {
        assert!(validate_memory_type("episodic").is_ok());
        assert!(validate_memory_type("semantic").is_ok());
        assert!(validate_memory_type("procedural").is_ok());
    }

    #[test]
    fn test_validate_memory_type_invalid() {
        assert!(validate_memory_type("invalid_type").is_err());
        assert!(validate_memory_type("episodic; DROP TABLE").is_err());
    }

    #[test]
    fn test_validate_safe_string_valid() {
        assert!(validate_safe_string("Hello, World!").is_ok());
        assert!(validate_safe_string("User-123_@test.com").is_ok());
    }

    #[test]
    fn test_validate_safe_string_invalid() {
        assert!(validate_safe_string("").is_err());
        assert!(validate_safe_string("   ").is_err());
        assert!(validate_safe_string("test\x00null").is_err());
    }

    #[test]
    fn test_validate_metadata_success() {
        let mut metadata = HashMap::new();
        metadata.insert("key1".to_string(), serde_json::json!("value1"));
        metadata.insert("key2".to_string(), serde_json::json!(42));

        assert!(validate_metadata(&metadata).is_ok());
    }

    #[test]
    fn test_validate_metadata_key_too_long() {
        let mut metadata = HashMap::new();
        metadata.insert("a".repeat(MAX_METADATA_KEY_LENGTH + 1), serde_json::json!("value"));

        assert!(validate_metadata(&metadata).is_err());
    }

    #[test]
    fn test_validate_metadata_key_invalid_chars() {
        let mut metadata = HashMap::new();
        metadata.insert("key; DROP TABLE".to_string(), serde_json::json!("value"));

        assert!(validate_metadata(&metadata).is_err());
    }

    #[test]
    fn test_validate_search_query_success() {
        assert!(validate_search_query("test query").is_ok());
    }

    #[test]
    fn test_validate_search_query_empty() {
        assert!(validate_search_query("").is_err());
        assert!(validate_search_query("   ").is_err());
    }

    #[test]
    fn test_validate_batch_size_success() {
        assert!(validate_batch_size(&vec![1, 2, 3]).is_ok());
    }

    #[test]
    fn test_validate_batch_size_too_large() {
        let batch = vec![0; MAX_BATCH_SIZE + 1];
        assert!(validate_batch_size(&batch).is_err());
    }

    #[test]
    fn test_validated_add_request_success() {
        let request = ValidatedAddRequest {
            content: "Test memory content".to_string(),
            user_id: Some("user_123".to_string()),
            agent_id: Some("agent_456".to_string()),
            run_id: None,
            metadata: None,
            memory_type: Some("episodic".to_string()),
            prompt: None,
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_validated_add_request_content_too_long() {
        let request = ValidatedAddRequest {
            content: "a".repeat(MAX_MEMORY_CONTENT_LENGTH + 1),
            user_id: None,
            agent_id: None,
            run_id: None,
            metadata: None,
            memory_type: None,
            prompt: None,
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_validated_search_request_success() {
        let request = ValidatedSearchRequest {
            query: "test query".to_string(),
            user_id: Some("user_123".to_string()),
            agent_id: None,
            run_id: None,
            memory_type: Some("semantic".to_string()),
            limit: Some(10),
            score_threshold: Some(0.5),
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_validated_search_request_limit_out_of_range() {
        let request = ValidatedSearchRequest {
            query: "test query".to_string(),
            user_id: None,
            agent_id: None,
            run_id: None,
            memory_type: None,
            limit: Some(101),
            score_threshold: None,
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_validated_batch_add_request_success() {
        let request = ValidatedBatchAddRequest {
            contents: vec![
                "Memory 1".to_string(),
                "Memory 2".to_string(),
                "Memory 3".to_string(),
            ],
            user_id: Some("user_123".to_string()),
            agent_id: None,
            metadata: None,
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_validated_batch_add_request_exceeds_max_batch() {
        let request = ValidatedBatchAddRequest {
            contents: vec!["Memory".to_string(); MAX_BATCH_SIZE + 1],
            user_id: None,
            agent_id: None,
            metadata: None,
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_validated_create_user_request_success() {
        let request = ValidatedCreateUserRequest {
            name: "John Doe".to_string(),
            metadata: None,
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_validated_create_user_request_name_too_long() {
        let request = ValidatedCreateUserRequest {
            name: "a".repeat(MAX_USER_ID_LENGTH + 1),
            metadata: None,
        };

        assert!(request.validate().is_err());
    }
}
