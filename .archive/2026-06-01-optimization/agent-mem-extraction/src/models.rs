//! Data models for extraction pipeline

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;

/// Unique identifier for extraction operations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExtractionId(pub String);

impl ExtractionId {
    /// Generate a new extraction ID
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Create from string
    pub fn from_string(s: String) -> Self {
        Self(s)
    }

    /// Get string reference
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for ExtractionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ExtractionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Input to extraction pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionInput {
    /// Unique extraction ID
    pub id: ExtractionId,

    /// Resource URI (file://, http://, conv://, doc://)
    pub uri: String,

    /// Optional content (if already loaded)
    pub content: Option<ResourceContent>,

    /// Media type (if known)
    pub media_type: Option<String>,

    /// Metadata
    pub metadata: HashMap<String, String>,

    /// User/agent scope
    pub scope: ExtractionScope,
}

impl ExtractionInput {
    /// Create new extraction input
    pub fn new(uri: String, scope: ExtractionScope) -> Self {
        Self {
            id: ExtractionId::new(),
            uri,
            content: None,
            media_type: None,
            metadata: HashMap::new(),
            scope,
        }
    }

    /// Create from URI string
    pub fn from_uri(uri: &str, user_id: &str) -> Self {
        Self::new(uri.to_string(), ExtractionScope::new(user_id.to_string()))
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Set content
    pub fn with_content(mut self, content: ResourceContent) -> Self {
        self.content = Some(content);
        self
    }
}

/// Resource content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceContent {
    /// Text content
    Text(String),
    /// Binary content
    Binary(Vec<u8>),
    /// JSON content
    JSON(serde_json::Value),
    /// Multi-part content (e.g., document with images)
    MultiPart { parts: Vec<ResourceContent> },
}

impl ResourceContent {
    /// Get text content if available
    pub fn as_text(&self) -> Option<&str> {
        match self {
            ResourceContent::Text(s) => Some(s),
            _ => None,
        }
    }

    /// Get binary content if available
    pub fn as_binary(&self) -> Option<&[u8]> {
        match self {
            ResourceContent::Binary(b) => Some(b),
            _ => None,
        }
    }

    /// Get content size in bytes
    pub fn size(&self) -> usize {
        match self {
            ResourceContent::Text(s) => s.len(),
            ResourceContent::Binary(b) => b.len(),
            ResourceContent::JSON(_) => 0, // JSON size varies
            ResourceContent::MultiPart { parts } => parts.iter().map(|p| p.size()).sum(),
        }
    }
}

/// Extraction scope (user/agent)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExtractionScope {
    /// User ID
    pub user_id: String,

    /// Optional agent ID
    pub agent_id: Option<String>,
}

impl ExtractionScope {
    /// Create new scope
    pub fn new(user_id: String) -> Self {
        Self {
            user_id,
            agent_id: None,
        }
    }

    /// Create with agent
    pub fn with_agent(user_id: String, agent_id: String) -> Self {
        Self {
            user_id,
            agent_id: Some(agent_id),
        }
    }
}

/// Output from extraction pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionOutput {
    /// Extraction ID
    pub id: ExtractionId,

    /// Extracted memory items
    pub items: Vec<MemoryItem>,

    /// Categories assigned
    pub categories: Vec<String>,

    /// Resources created
    pub resources: Vec<String>,

    /// Execution metrics
    pub metrics: ExtractionMetrics,

    /// Warnings (non-fatal issues)
    pub warnings: Vec<String>,

    /// Created at
    pub created_at: DateTime<Utc>,
}

impl ExtractionOutput {
    /// Create new extraction output
    pub fn new(id: ExtractionId) -> Self {
        Self {
            id,
            items: Vec::new(),
            categories: Vec::new(),
            resources: Vec::new(),
            metrics: ExtractionMetrics::default(),
            warnings: Vec::new(),
            created_at: Utc::now(),
        }
    }

    /// Add memory item
    pub fn with_item(mut self, item: MemoryItem) -> Self {
        self.items.push(item);
        self.metrics.items_extracted += 1;
        self
    }

    /// Add warning
    pub fn with_warning(mut self, warning: String) -> Self {
        self.warnings.push(warning);
        self
    }
}

/// Memory item extracted from resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    /// Unique ID
    pub id: String,

    /// Item content
    pub content: String,

    /// Item type (fact, preference, event, skill)
    pub item_type: String,

    /// Category path
    pub category: Option<String>,

    /// Source resource ID
    pub source_resource_id: Option<String>,

    /// Confidence score (0-1)
    pub confidence: f32,

    /// Metadata
    pub metadata: HashMap<String, String>,

    /// Created at
    pub created_at: DateTime<Utc>,
}

impl MemoryItem {
    /// Create new memory item
    pub fn new(content: String, item_type: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            content,
            item_type,
            category: None,
            source_resource_id: None,
            confidence: 1.0,
            metadata: HashMap::new(),
            created_at: Utc::now(),
        }
    }

    /// With category
    pub fn with_category(mut self, category: String) -> Self {
        self.category = Some(category);
        self
    }

    /// With confidence
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence;
        self
    }

    /// With source resource
    pub fn with_source(mut self, resource_id: String) -> Self {
        self.source_resource_id = Some(resource_id);
        self
    }
}

/// Extraction execution metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExtractionMetrics {
    /// Total execution time in milliseconds
    pub total_duration_ms: u64,

    /// Number of items extracted
    pub items_extracted: usize,

    /// Number of items deduplicated
    pub items_deduped: usize,

    /// Number of categories created
    pub categories_created: usize,

    /// Stage timings (stage_name -> duration_ms)
    pub stage_timings: HashMap<String, u64>,

    /// Resource size in bytes
    pub resource_size_bytes: usize,

    /// LLM tokens used
    pub llm_tokens_used: u64,
}

/// Extraction context passed between stages
#[derive(Debug, Clone)]
pub struct ExtractionContext {
    /// Extraction ID
    pub id: ExtractionId,

    /// User/agent scope
    pub scope: ExtractionScope,

    /// Configuration
    pub config: PipelineConfig,

    /// Shared state between stages
    pub state: HashMap<String, String>,
}

impl ExtractionContext {
    /// Create new context
    pub fn new(id: ExtractionId, scope: ExtractionScope, config: PipelineConfig) -> Self {
        Self {
            id,
            scope,
            config,
            state: HashMap::new(),
        }
    }

    /// Get state value
    pub fn get_state(&self, key: &str) -> Option<&String> {
        self.state.get(key)
    }

    /// Set state value
    pub fn set_state(&mut self, key: String, value: String) {
        self.state.insert(key, value);
    }
}

/// Pipeline configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    /// Execution mode (sequential, parallel)
    pub execution_mode: ExecutionMode,

    /// Enable stage caching
    pub enable_caching: bool,

    /// Timeout per stage (seconds)
    pub stage_timeout_secs: u64,

    /// Maximum retries per stage
    pub max_retries: usize,

    /// Enable detailed logging
    pub verbose: bool,

    /// Deduplication threshold (0-1, lower = more strict)
    pub dedup_threshold: f32,

    /// Categorization confidence threshold (0-1)
    pub category_confidence_threshold: f32,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            execution_mode: ExecutionMode::Sequential,
            enable_caching: true,
            stage_timeout_secs: 60,
            max_retries: 3,
            verbose: false,
            dedup_threshold: 0.85,
            category_confidence_threshold: 0.7,
        }
    }
}

/// Pipeline execution mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionMode {
    /// Execute stages sequentially
    Sequential,
    /// Execute independent stages in parallel
    Parallel,
    /// Execute with conditional branching
    Conditional,
}
