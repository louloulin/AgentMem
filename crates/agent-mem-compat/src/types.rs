//! Type definitions for Mem0 compatibility

use chrono::{DateTime, Utc};
use regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A memory item in Mem0 format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    /// Unique identifier for the memory
    pub id: String,

    /// The memory content/text
    pub memory: String,

    /// User ID associated with this memory
    pub user_id: String,

    /// Optional agent ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,

    /// Optional run ID for session tracking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,

    /// Memory metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,

    /// Memory score/relevance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f32>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last updated timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Search result containing memories and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchResult {
    /// List of matching memories
    pub memories: Vec<Memory>,

    /// Total number of results (may be more than returned)
    pub total: usize,

    /// Search metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Sort order for memory operations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum SortOrder {
    /// Ascending order
    Asc,
    /// Descending order
    #[default]
    Desc,
}

/// Sort field for memory operations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum SortField {
    /// Sort by creation date
    #[default]
    CreatedAt,
    /// Sort by update date
    UpdatedAt,
    /// Sort by relevance score
    Score,
    /// Sort by memory content length
    ContentLength,
    /// Sort by custom metadata field
    Metadata(String),
}

/// Complex filter operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterOperation {
    /// Equals
    Eq(serde_json::Value),
    /// Not equals
    Ne(serde_json::Value),
    /// Greater than
    Gt(serde_json::Value),
    /// Greater than or equal
    Gte(serde_json::Value),
    /// Less than
    Lt(serde_json::Value),
    /// Less than or equal
    Lte(serde_json::Value),
    /// Contains (for strings)
    Contains(String),
    /// Starts with (for strings)
    StartsWith(String),
    /// Ends with (for strings)
    EndsWith(String),
    /// In list
    In(Vec<serde_json::Value>),
    /// Not in list
    NotIn(Vec<serde_json::Value>),
    /// Regex match
    Regex(String),
}

/// Advanced filter options for memory operations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryFilter {
    /// Filter by agent ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,

    /// Filter by run ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,

    /// Filter by memory type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_type: Option<String>,

    /// Filter by date range (start)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_after: Option<DateTime<Utc>>,

    /// Filter by date range (end)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_before: Option<DateTime<Utc>>,

    /// Filter by update date range (start)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_after: Option<DateTime<Utc>>,

    /// Filter by update date range (end)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_before: Option<DateTime<Utc>>,

    /// Filter by minimum score
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_score: Option<f32>,

    /// Filter by maximum score
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_score: Option<f32>,

    /// Filter by content length range
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_content_length: Option<usize>,

    /// Filter by content length range
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_content_length: Option<usize>,

    /// Complex metadata filters with operations
    #[serde(default)]
    pub metadata_filters: HashMap<String, FilterOperation>,

    /// Simple metadata filters (for backward compatibility)
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,

    /// Content search patterns
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_contains: Option<String>,

    /// Content regex patterns
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_regex: Option<String>,

    /// Tags filter
    #[serde(default)]
    pub tags: Vec<String>,

    /// Exclude tags
    #[serde(default)]
    pub exclude_tags: Vec<String>,

    /// Sort field
    #[serde(default)]
    pub sort_field: SortField,

    /// Sort order
    #[serde(default)]
    pub sort_order: SortOrder,

    /// Maximum number of results to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,

    /// Offset for pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<usize>,
}

impl MemoryFilter {
    /// Create a new empty filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Add agent ID filter
    pub fn with_agent_id(mut self, agent_id: String) -> Self {
        self.agent_id = Some(agent_id);
        self
    }

    /// Add run ID filter
    pub fn with_run_id(mut self, run_id: String) -> Self {
        self.run_id = Some(run_id);
        self
    }

    /// Add date range filter
    pub fn with_date_range(
        mut self,
        after: Option<DateTime<Utc>>,
        before: Option<DateTime<Utc>>,
    ) -> Self {
        self.created_after = after;
        self.created_before = before;
        self
    }

    /// Add score range filter
    pub fn with_score_range(mut self, min: Option<f32>, max: Option<f32>) -> Self {
        self.min_score = min;
        self.max_score = max;
        self
    }

    /// Add content length filter
    pub fn with_content_length_range(mut self, min: Option<usize>, max: Option<usize>) -> Self {
        self.min_content_length = min;
        self.max_content_length = max;
        self
    }

    /// Add content search filter
    pub fn with_content_contains(mut self, pattern: String) -> Self {
        self.content_contains = Some(pattern);
        self
    }

    /// Add content regex filter
    pub fn with_content_regex(mut self, pattern: String) -> Self {
        self.content_regex = Some(pattern);
        self
    }

    /// Add tags filter
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Add exclude tags filter
    pub fn with_exclude_tags(mut self, tags: Vec<String>) -> Self {
        self.exclude_tags = tags;
        self
    }

    /// Add metadata filter with operation
    pub fn with_metadata_filter(mut self, key: String, operation: FilterOperation) -> Self {
        self.metadata_filters.insert(key, operation);
        self
    }

    /// Add simple metadata filter
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Set sorting
    pub fn with_sort(mut self, field: SortField, order: SortOrder) -> Self {
        self.sort_field = field;
        self.sort_order = order;
        self
    }

    /// Set pagination
    pub fn with_pagination(mut self, limit: Option<usize>, offset: Option<usize>) -> Self {
        self.limit = limit;
        self.offset = offset;
        self
    }

    /// Check if a memory matches this filter
    pub fn matches(&self, memory: &Memory) -> bool {
        // Agent ID filter
        if let Some(ref agent_id) = self.agent_id {
            if memory.agent_id.as_ref() != Some(agent_id) {
                return false;
            }
        }

        // Run ID filter
        if let Some(ref run_id) = self.run_id {
            if memory.run_id.as_ref() != Some(run_id) {
                return false;
            }
        }

        // Date range filters
        if let Some(after) = self.created_after {
            if memory.created_at < after {
                return false;
            }
        }

        if let Some(before) = self.created_before {
            if memory.created_at > before {
                return false;
            }
        }

        // Update date filters
        if let Some(after) = self.updated_after {
            if let Some(updated_at) = memory.updated_at {
                if updated_at < after {
                    return false;
                }
            } else {
                return false;
            }
        }

        if let Some(before) = self.updated_before {
            if let Some(updated_at) = memory.updated_at {
                if updated_at > before {
                    return false;
                }
            }
        }

        // Score range filters
        if let Some(min_score) = self.min_score {
            if memory.score.unwrap_or(0.0) < min_score {
                return false;
            }
        }

        if let Some(max_score) = self.max_score {
            if memory.score.unwrap_or(0.0) > max_score {
                return false;
            }
        }

        // Content length filters
        let content_length = memory.memory.len();
        if let Some(min_length) = self.min_content_length {
            if content_length < min_length {
                return false;
            }
        }

        if let Some(max_length) = self.max_content_length {
            if content_length > max_length {
                return false;
            }
        }

        // Content search filters
        if let Some(ref pattern) = self.content_contains {
            if !memory.memory.contains(pattern) {
                return false;
            }
        }

        if let Some(ref pattern) = self.content_regex {
            if let Ok(regex) = regex::Regex::new(pattern) {
                if !regex.is_match(&memory.memory) {
                    return false;
                }
            }
        }

        // Simple metadata filters
        for (key, expected_value) in &self.metadata {
            if let Some(actual_value) = memory.metadata.get(key) {
                if actual_value != expected_value {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Complex metadata filters
        for (key, operation) in &self.metadata_filters {
            if let Some(actual_value) = memory.metadata.get(key) {
                if !self.matches_operation(actual_value, operation) {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    /// Check if a value matches a filter operation
    fn matches_operation(&self, value: &serde_json::Value, operation: &FilterOperation) -> bool {
        match operation {
            FilterOperation::Eq(expected) => value == expected,
            FilterOperation::Ne(expected) => value != expected,
            FilterOperation::Gt(expected) => {
                if let (Some(v), Some(e)) = (value.as_f64(), expected.as_f64()) {
                    v > e
                } else {
                    false
                }
            }
            FilterOperation::Gte(expected) => {
                if let (Some(v), Some(e)) = (value.as_f64(), expected.as_f64()) {
                    v >= e
                } else {
                    false
                }
            }
            FilterOperation::Lt(expected) => {
                if let (Some(v), Some(e)) = (value.as_f64(), expected.as_f64()) {
                    v < e
                } else {
                    false
                }
            }
            FilterOperation::Lte(expected) => {
                if let (Some(v), Some(e)) = (value.as_f64(), expected.as_f64()) {
                    v <= e
                } else {
                    false
                }
            }
            FilterOperation::Contains(pattern) => {
                if let Some(s) = value.as_str() {
                    s.contains(pattern)
                } else {
                    false
                }
            }
            FilterOperation::StartsWith(pattern) => {
                if let Some(s) = value.as_str() {
                    s.starts_with(pattern)
                } else {
                    false
                }
            }
            FilterOperation::EndsWith(pattern) => {
                if let Some(s) = value.as_str() {
                    s.ends_with(pattern)
                } else {
                    false
                }
            }
            FilterOperation::In(list) => list.contains(value),
            FilterOperation::NotIn(list) => !list.contains(value),
            FilterOperation::Regex(pattern) => {
                if let (Some(s), Ok(regex)) = (value.as_str(), regex::Regex::new(pattern)) {
                    regex.is_match(s)
                } else {
                    false
                }
            }
        }
    }
}

/// Memory addition request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddMemoryRequest {
    /// The memory content
    pub memory: String,

    /// User ID
    pub user_id: String,

    /// Optional agent ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,

    /// Optional run ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,

    /// Optional metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Memory update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMemoryRequest {
    /// New memory content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<String>,

    /// Updated metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Memory search request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMemoryRequest {
    /// Search query
    pub query: String,

    /// User ID to search within
    pub user_id: String,

    /// Optional filters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<MemoryFilter>,

    /// Maximum number of results
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

/// Memory deletion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteMemoryResponse {
    /// Whether the deletion was successful
    pub success: bool,

    /// Optional message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Change type for memory history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    /// Memory was created
    Created,
    /// Memory content was updated
    ContentUpdated,
    /// Memory metadata was updated
    MetadataUpdated,
    /// Memory was deleted
    Deleted,
    /// Memory was merged with another
    Merged,
    /// Memory was split into multiple memories
    Split,
}

impl std::fmt::Display for ChangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangeType::Created => write!(f, "created"),
            ChangeType::ContentUpdated => write!(f, "content_updated"),
            ChangeType::MetadataUpdated => write!(f, "metadata_updated"),
            ChangeType::Deleted => write!(f, "deleted"),
            ChangeType::Merged => write!(f, "merged"),
            ChangeType::Split => write!(f, "split"),
        }
    }
}

/// Memory history entry with enhanced tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryHistory {
    /// Unique history entry ID
    pub id: String,

    /// Memory ID this history belongs to
    pub memory_id: String,

    /// User ID
    pub user_id: String,

    /// Previous content (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_memory: Option<String>,

    /// New content (if applicable)
    pub new_memory: Option<String>,

    /// Previous metadata (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_metadata: Option<HashMap<String, serde_json::Value>>,

    /// New metadata (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_metadata: Option<HashMap<String, serde_json::Value>>,

    /// Timestamp of change
    pub timestamp: DateTime<Utc>,

    /// Change type
    pub change_type: ChangeType,

    /// User who made the change
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changed_by: Option<String>,

    /// Reason for the change
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    /// Version number
    pub version: u32,

    /// Related memory IDs (for merge/split operations)
    #[serde(default)]
    pub related_memory_ids: Vec<String>,

    /// Additional change metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

// Note: Conversion implementations would be added here when integrating with full AgentMem
// For now, the compatibility layer uses its own Memory type

/// Individual memory search result item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchResultItem {
    /// Memory ID
    pub id: String,
    /// Memory content
    pub content: String,
    /// User ID
    pub user_id: String,
    /// Agent ID (optional)
    pub agent_id: Option<String>,
    /// Run ID (optional)
    pub run_id: Option<String>,
    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Relevance score
    pub score: Option<f32>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: Option<DateTime<Utc>>,
}

/// Batch add result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchAddResult {
    /// Number of successful additions
    pub successful: usize,
    /// Number of failed additions
    pub failed: usize,
    /// List of successfully created memory IDs
    pub results: Vec<String>,
    /// List of error messages for failed operations
    pub errors: Vec<String>,
}

/// Batch update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchUpdateRequest {
    /// List of update requests
    pub requests: Vec<BatchUpdateItem>,
}

/// Individual batch update item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchUpdateItem {
    /// Memory ID to update
    pub memory_id: String,
    /// User ID
    pub user_id: String,
    /// New memory content (optional)
    pub memory: Option<String>,
    /// New metadata (optional)
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Batch update result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchUpdateResult {
    /// Number of successful updates
    pub successful: usize,
    /// Number of failed updates
    pub failed: usize,
    /// List of successfully updated memory IDs
    pub results: Vec<String>,
    /// List of error messages for failed operations
    pub errors: Vec<String>,
}

/// Batch delete request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchDeleteRequest {
    /// List of delete requests
    pub requests: Vec<BatchDeleteItem>,
}

/// Individual batch delete item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchDeleteItem {
    /// Memory ID to delete
    pub memory_id: String,
    /// User ID
    pub user_id: String,
}

/// Batch delete result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchDeleteResult {
    /// Number of successful deletions
    pub successful: usize,
    /// Number of failed deletions
    pub failed: usize,
    /// List of successfully deleted memory IDs
    pub results: Vec<String>,
    /// List of error messages for failed operations
    pub errors: Vec<String>,
}
