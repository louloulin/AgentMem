//! Input validation for memory API endpoints
//!
//! This module provides validation structures for all memory-related requests using the `validator` crate.
//! It ensures:
//! - Payload size limits (max 1MB)
//! - Field length constraints
//! - Content sanitization (no HTML/script tags)
//! - Metadata key-value constraints
//!
//! 🎯 P1 Task: Input validation layer implementation
//! 📅 Created: 2025-01-07
//! 🏗️ Architecture: Security validation layer at API boundary

use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError, ValidationErrorKind};
use std::collections::HashMap;

/// Maximum payload size in bytes (1MB)
const MAX_PAYLOAD_SIZE: usize = 1_048_576;

/// Maximum content length
const MAX_CONTENT_LENGTH: usize = 50_000;

/// Maximum number of metadata entries
const MAX_METADATA_ENTRIES: usize = 50;

/// Maximum metadata key length
const MAX_METADATA_KEY_LENGTH: usize = 100;

/// Maximum metadata value length
const MAX_METADATA_VALUE_LENGTH: usize = 1_000;

/// Maximum number of tags
const MAX_TAGS_COUNT: usize = 20;

/// Maximum tag length
const MAX_TAG_LENGTH: usize = 50;

/// Custom validator: Check for HTML/script tags in content
fn validate_no_html(content: &str) -> Result<(), ValidationError> {
    let dangerous_patterns = [
        "<script",
        "</script",
        "<iframe",
        "</iframe",
        "<object",
        "</object",
        "<embed",
        "javascript:",
        "onclick=",
        "onerror=",
        "onload=",
    ];

    let content_lower = content.to_lowercase();
    for pattern in &dangerous_patterns {
        if content_lower.contains(pattern) {
            return Err(ValidationError::new(ValidationErrorKind::Custom(
                String::from("content_contains_html_or_script"),
                Some(format!("Content contains potentially dangerous pattern: {}", pattern)),
            )));
        }
    }

    Ok(())
}

/// Custom validator: Check payload size
fn validate_payload_size(payload: &str) -> Result<(), ValidationError> {
    let size = payload.len();
    if size > MAX_PAYLOAD_SIZE {
        return Err(ValidationError::new(ValidationErrorKind::Custom(
            String::from("payload_too_large"),
            Some(format!(
                "Payload size {} bytes exceeds maximum {} bytes",
                size, MAX_PAYLOAD_SIZE
            )),
        )));
    }
    Ok(())
}

/// Custom validator: Validate metadata keys (alphanumeric, underscore, hyphen)
fn validate_metadata_key(key: &str) -> Result<(), ValidationError> {
    if !key.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(ValidationError::new(ValidationErrorKind::Custom(
            String::from("invalid_metadata_key"),
            Some(format!(
                "Metadata key '{}' contains invalid characters (only alphanumeric, underscore, hyphen allowed)",
                key
            )),
        )));
    }
    Ok(())
}

/// Custom validator: Validate tags (alphanumeric, underscore, hyphen)
fn validate_tag(tag: &str) -> Result<(), ValidationError> {
    if !tag.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(ValidationError::new(ValidationErrorKind::Custom(
            String::from("invalid_tag"),
            Some(format!(
                "Tag '{}' contains invalid characters (only alphanumeric, underscore, hyphen allowed)",
                tag
            )),
        )));
    }
    Ok(())
}

/// Request validator for adding a memory
#[derive(Debug, Clone, Validate, Deserialize, Serialize)]
pub struct AddMemoryRequest {
    /// Memory content
    #[validate(length(min = 1, max = 50000), custom = "validate_no_html")]
    pub content: String,

    /// Optional metadata
    #[validate(length(max = 50))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,

    /// Optional tags
    #[validate(length(max = 20))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    /// Optional importance score (0.0 to 1.0)
    #[validate(range(min = 0.0, max = 1.0))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub importance: Option<f64>,

    /// Optional agent ID
    #[validate(length(max = 100))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,

    /// Optional session ID
    #[validate(length(max = 100))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

impl AddMemoryRequest {
    /// Validate the entire request including payload size
    pub fn validate_payload(&self) -> Result<(), String> {
        // Validate payload size
        let payload_str = serde_json::to_string(self)
            .map_err(|e| format!("Failed to serialize payload: {}", e))?;
        validate_payload_size(&payload_str)
            .map_err(|e| e.message.unwrap_or_else(|| "Payload validation failed".to_string()))?;

        // Validate struct-level validators
        self.validate()
            .map_err(|e| e.to_string())?;

        // Validate metadata keys and values
        if let Some(ref metadata) = self.metadata {
            if metadata.len() > MAX_METADATA_ENTRIES {
                return Err(format!(
                    "Metadata entries count {} exceeds maximum {}",
                    metadata.len(),
                    MAX_METADATA_ENTRIES
                ));
            }
            for (key, value) in metadata {
                validate_metadata_key(key)
                    .map_err(|e| e.message.unwrap_or_else(|| "Invalid metadata key".to_string()))?;
                if key.len() > MAX_METADATA_KEY_LENGTH {
                    return Err(format!(
                        "Metadata key length {} exceeds maximum {}",
                        key.len(),
                        MAX_METADATA_KEY_LENGTH
                    ));
                }
                if value.len() > MAX_METADATA_VALUE_LENGTH {
                    return Err(format!(
                        "Metadata value length {} exceeds maximum {}",
                        value.len(),
                        MAX_METADATA_VALUE_LENGTH
                    ));
                }
            }
        }

        // Validate tags
        if let Some(ref tags) = self.tags {
            if tags.len() > MAX_TAGS_COUNT {
                return Err(format!(
                    "Tags count {} exceeds maximum {}",
                    tags.len(),
                    MAX_TAGS_COUNT
                ));
            }
            for tag in tags {
                validate_tag(tag)
                    .map_err(|e| e.message.unwrap_or_else(|| "Invalid tag".to_string()))?;
                if tag.len() > MAX_TAG_LENGTH {
                    return Err(format!(
                        "Tag length {} exceeds maximum {}",
                        tag.len(),
                        MAX_TAG_LENGTH
                    ));
                }
            }
        }

        Ok(())
    }
}

/// Request validator for updating a memory
#[derive(Debug, Clone, Validate, Deserialize, Serialize)]
pub struct UpdateMemoryRequest {
    /// Memory ID
    #[validate(length(min = 1, max = 100))]
    pub id: String,

    /// New content
    #[validate(length(min = 1, max = 50000), custom = "validate_no_html")]
    pub content: String,

    /// Optional metadata
    #[validate(length(max = 50))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,

    /// Optional tags
    #[validate(length(max = 20))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    /// Optional importance score (0.0 to 1.0)
    #[validate(range(min = 0.0, max = 1.0))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub importance: Option<f64>,
}

impl UpdateMemoryRequest {
    /// Validate the entire request including payload size
    pub fn validate_payload(&self) -> Result<(), String> {
        // Validate payload size
        let payload_str = serde_json::to_string(self)
            .map_err(|e| format!("Failed to serialize payload: {}", e))?;
        validate_payload_size(&payload_str)
            .map_err(|e| e.message.unwrap_or_else(|| "Payload validation failed".to_string()))?;

        // Validate struct-level validators
        self.validate()
            .map_err(|e| e.to_string())?;

        // Validate metadata and tags (same logic as AddMemoryRequest)
        if let Some(ref metadata) = self.metadata {
            if metadata.len() > MAX_METADATA_ENTRIES {
                return Err(format!(
                    "Metadata entries count {} exceeds maximum {}",
                    metadata.len(),
                    MAX_METADATA_ENTRIES
                ));
            }
            for (key, value) in metadata {
                validate_metadata_key(key)
                    .map_err(|e| e.message.unwrap_or_else(|| "Invalid metadata key".to_string()))?;
                if key.len() > MAX_METADATA_KEY_LENGTH {
                    return Err(format!(
                        "Metadata key length {} exceeds maximum {}",
                        key.len(),
                        MAX_METADATA_KEY_LENGTH
                    ));
                }
                if value.len() > MAX_METADATA_VALUE_LENGTH {
                    return Err(format!(
                        "Metadata value length {} exceeds maximum {}",
                        value.len(),
                        MAX_METADATA_VALUE_LENGTH
                    ));
                }
            }
        }

        if let Some(ref tags) = self.tags {
            if tags.len() > MAX_TAGS_COUNT {
                return Err(format!(
                    "Tags count {} exceeds maximum {}",
                    tags.len(),
                    MAX_TAGS_COUNT
                ));
            }
            for tag in tags {
                validate_tag(tag)
                    .map_err(|e| e.message.unwrap_or_else(|| "Invalid tag".to_string()))?;
                if tag.len() > MAX_TAG_LENGTH {
                    return Err(format!(
                        "Tag length {} exceeds maximum {}",
                        tag.len(),
                        MAX_TAG_LENGTH
                    ));
                }
            }
        }

        Ok(())
    }
}

/// Request validator for searching memories
#[derive(Debug, Clone, Validate, Deserialize, Serialize)]
pub struct SearchMemoryRequest {
    /// Search query
    #[validate(length(min = 1, max = 1_000))]
    pub query: String,

    /// Maximum results
    #[validate(range(min = 1, max = 100))]
    #[serde(default = "default_limit")]
    pub limit: usize,

    /// Optional filter by agent ID
    #[validate(length(max = 100))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,

    /// Optional filter by tags
    #[validate(length(max = 20))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    /// Minimum importance score
    #[validate(range(min = 0.0, max = 1.0))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_importance: Option<f64>,
}

fn default_limit() -> usize {
    10
}

impl SearchMemoryRequest {
    /// Validate the entire request
    pub fn validate_payload(&self) -> Result<(), String> {
        // Validate payload size
        let payload_str = serde_json::to_string(self)
            .map_err(|e| format!("Failed to serialize payload: {}", e))?;
        validate_payload_size(&payload_str)
            .map_err(|e| e.message.unwrap_or_else(|| "Payload validation failed".to_string()))?;

        // Validate struct-level validators
        self.validate()
            .map_err(|e| e.to_string())?;

        // Validate tags if present
        if let Some(ref tags) = self.tags {
            for tag in tags {
                validate_tag(tag)
                    .map_err(|e| e.message.unwrap_or_else(|| "Invalid tag".to_string()))?;
            }
        }

        Ok(())
    }
}

/// Request validator for deleting a memory
#[derive(Debug, Clone, Validate, Deserialize, Serialize)]
pub struct DeleteMemoryRequest {
    /// Memory ID
    #[validate(length(min = 1, max = 100))]
    pub id: String,
}

impl DeleteMemoryRequest {
    /// Validate the request
    pub fn validate_payload(&self) -> Result<(), String> {
        self.validate()
            .map_err(|e| e.to_string())
    }
}

/// Request validator for batch operations
#[derive(Debug, Clone, Validate, Deserialize, Serialize)]
pub struct BatchAddMemoriesRequest {
    /// List of memories to add
    #[validate(length(min = 1, max = 100))]
    pub memories: Vec<AddMemoryRequest>,
}

impl BatchAddMemoriesRequest {
    /// Validate the entire request
    pub fn validate_payload(&self) -> Result<(), String> {
        // Validate overall payload size
        let payload_str = serde_json::to_string(self)
            .map_err(|e| format!("Failed to serialize payload: {}", e))?;
        validate_payload_size(&payload_str)
            .map_err(|e| e.message.unwrap_or_else(|| "Payload validation failed".to_string()))?;

        // Validate struct-level validators
        self.validate()
            .map_err(|e| e.to_string())?;

        // Validate each memory in the batch
        for (index, memory) in self.memories.iter().enumerate() {
            memory.validate_payload()
                .map_err(|e| format!("Memory at index {} validation failed: {}", index, e))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_add_memory_request() {
        let request = AddMemoryRequest {
            content: "This is a valid memory content".to_string(),
            metadata: None,
            tags: None,
            importance: Some(0.5),
            agent_id: Some("agent-123".to_string()),
            session_id: Some("session-456".to_string()),
        };

        assert!(request.validate_payload().is_ok());
    }

    #[test]
    fn test_content_too_long() {
        let request = AddMemoryRequest {
            content: "a".repeat(50_001), // Exceeds MAX_CONTENT_LENGTH
            metadata: None,
            tags: None,
            importance: None,
            agent_id: None,
            session_id: None,
        };

        assert!(request.validate_payload().is_err());
    }

    #[test]
    fn test_content_contains_html() {
        let request = AddMemoryRequest {
            content: "Check out this <script>alert('xss')</script> content".to_string(),
            metadata: None,
            tags: None,
            importance: None,
            agent_id: None,
            session_id: None,
        };

        assert!(request.validate_payload().is_err());
    }

    #[test]
    fn test_invalid_metadata_key() {
        let mut metadata = HashMap::new();
        metadata.insert("invalid key!".to_string(), "value".to_string());

        let request = AddMemoryRequest {
            content: "Valid content".to_string(),
            metadata: Some(metadata),
            tags: None,
            importance: None,
            agent_id: None,
            session_id: None,
        };

        assert!(request.validate_payload().is_err());
    }

    #[test]
    fn test_invalid_tag() {
        let request = AddMemoryRequest {
            content: "Valid content".to_string(),
            metadata: None,
            tags: Some(vec!["invalid tag!".to_string()]),
            importance: None,
            agent_id: None,
            session_id: None,
        };

        assert!(request.validate_payload().is_err());
    }

    #[test]
    fn test_importance_out_of_range() {
        let request = AddMemoryRequest {
            content: "Valid content".to_string(),
            metadata: None,
            tags: None,
            importance: Some(1.5), // Exceeds max 1.0
            agent_id: None,
            session_id: None,
        };

        assert!(request.validate_payload().is_err());
    }

    #[test]
    fn test_valid_search_request() {
        let request = SearchMemoryRequest {
            query: "rust programming".to_string(),
            limit: 10,
            agent_id: Some("agent-123".to_string()),
            tags: Some(vec!["rust".to_string(), "programming".to_string()]),
            min_importance: Some(0.3),
        };

        assert!(request.validate_payload().is_ok());
    }

    #[test]
    fn test_batch_add_memories_request() {
        let request = BatchAddMemoriesRequest {
            memories: vec![
                AddMemoryRequest {
                    content: "First memory".to_string(),
                    metadata: None,
                    tags: None,
                    importance: None,
                    agent_id: None,
                    session_id: None,
                },
                AddMemoryRequest {
                    content: "Second memory".to_string(),
                    metadata: None,
                    tags: None,
                    importance: None,
                    agent_id: None,
                    session_id: None,
                },
            ],
        };

        assert!(request.validate_payload().is_ok());
    }
}
