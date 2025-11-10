//! Core data types for AgentMem

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

/// A message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: Option<DateTime<Utc>>,
}

/// Messages type supporting multiple input formats (Mem5 compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Messages {
    /// Single string message
    Single(String),
    /// Structured message with role
    Structured(Message),
    /// Multiple messages
    Multiple(Vec<Message>),
}

impl Messages {
    /// Get the number of messages
    pub fn len(&self) -> usize {
        match self {
            Messages::Single(_) => 1,
            Messages::Structured(_) => 1,
            Messages::Multiple(messages) => messages.len(),
        }
    }

    /// Check if messages is empty
    pub fn is_empty(&self) -> bool {
        match self {
            Messages::Single(s) => s.is_empty(),
            Messages::Structured(msg) => msg.content.is_empty(),
            Messages::Multiple(messages) => messages.is_empty(),
        }
    }
}

impl Messages {
    /// Validate messages content
    pub fn validate(&self) -> crate::Result<()> {
        match self {
            Messages::Single(s) => {
                if s.trim().is_empty() {
                    return Err(crate::AgentMemError::ValidationError(
                        "Empty message".to_string(),
                    ));
                }
            }
            Messages::Structured(msg) => {
                if msg.content.trim().is_empty() {
                    return Err(crate::AgentMemError::ValidationError(
                        "Empty message content".to_string(),
                    ));
                }
            }
            Messages::Multiple(msgs) => {
                if msgs.is_empty() {
                    return Err(crate::AgentMemError::ValidationError(
                        "Empty message list".to_string(),
                    ));
                }
                for msg in msgs {
                    if msg.content.trim().is_empty() {
                        return Err(crate::AgentMemError::ValidationError(
                            "Empty message content in list".to_string(),
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    /// Convert to message list
    pub fn to_message_list(&self) -> Vec<Message> {
        match self {
            Messages::Single(s) => vec![Message::user(s)],
            Messages::Structured(msg) => vec![msg.clone()],
            Messages::Multiple(msgs) => msgs.clone(),
        }
    }

    /// Convert to content string
    pub fn to_content_string(&self) -> String {
        match self {
            Messages::Single(s) => s.clone(),
            Messages::Structured(msg) => msg.content.clone(),
            Messages::Multiple(msgs) => msgs
                .iter()
                .map(|m| m.content.clone())
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }

    /// Get message count
    pub fn get_message_count(&self) -> usize {
        match self {
            Messages::Single(_) => 1,
            Messages::Structured(_) => 1,
            Messages::Multiple(msgs) => msgs.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

impl Message {
    pub fn system(content: &str) -> Self {
        Self {
            role: MessageRole::System,
            content: content.to_string(),
            timestamp: Some(Utc::now()),
        }
    }

    pub fn user(content: &str) -> Self {
        Self {
            role: MessageRole::User,
            content: content.to_string(),
            timestamp: Some(Utc::now()),
        }
    }

    pub fn assistant(content: &str) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.to_string(),
            timestamp: Some(Utc::now()),
        }
    }

    /// Validate message content
    pub fn validate(&self) -> crate::Result<()> {
        if self.content.trim().is_empty() {
            return Err(crate::AgentMemError::ValidationError(
                "Empty message content".to_string(),
            ));
        }
        Ok(())
    }
}

/// A memory item stored in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    pub id: String,
    pub content: String,
    pub hash: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub score: Option<f32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub session: Session,
    pub memory_type: MemoryType,
    pub entities: Vec<Entity>,
    pub relations: Vec<Relation>,

    // Additional fields for compatibility and advanced features
    pub agent_id: String,
    pub user_id: Option<String>,
    pub importance: f32,
    pub embedding: Option<Vec<f32>>,
    pub last_accessed_at: DateTime<Utc>,
    pub access_count: u32,
    pub expires_at: Option<DateTime<Utc>>,
    pub version: u32,
}

/// Session information for scoping memories
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Session {
    pub id: String,
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub run_id: Option<String>,
    pub actor_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            created_at: Utc::now(),
            ..Default::default()
        }
    }

    pub fn with_user_id(mut self, user_id: Option<String>) -> Self {
        self.user_id = user_id;
        self
    }

    pub fn with_agent_id(mut self, agent_id: Option<String>) -> Self {
        self.agent_id = agent_id;
        self
    }

    pub fn with_run_id(mut self, run_id: Option<String>) -> Self {
        self.run_id = run_id;
        self
    }
}

/// Types of memory (AgentMem 7.0 - 8 cognitive memory types)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum MemoryType {
    // Legacy type for backward compatibility
    Factual, // 事实性记忆 (mapped to Semantic)

    // Basic cognitive memories
    #[default]
    Episodic, // 情节性记忆 - specific events and experiences
    Procedural, // 程序性记忆 - skills and procedures
    Semantic,   // 语义记忆 - facts and general knowledge
    Working,    // 工作记忆 - temporary information processing

    // Advanced cognitive memories (AgentMem 7.0)
    Core,       // 核心记忆 - persistent identity and preferences
    Resource,   // 资源记忆 - multimedia content and documents
    Knowledge,  // 知识记忆 - structured knowledge graphs
    Contextual, // 上下文记忆 - environment-aware information
}

impl MemoryType {
    pub fn as_str(&self) -> &'static str {
        match self {
            // Legacy type
            MemoryType::Factual => "factual",
            // Basic cognitive memories
            MemoryType::Episodic => "episodic",
            MemoryType::Procedural => "procedural",
            MemoryType::Semantic => "semantic",
            MemoryType::Working => "working",
            // Advanced cognitive memories (AgentMem 7.0)
            MemoryType::Core => "core",
            MemoryType::Resource => "resource",
            MemoryType::Knowledge => "knowledge",
            MemoryType::Contextual => "contextual",
        }
    }

    /// Parse a memory type from a string
    ///
    /// # Arguments
    /// * `s` - The string to parse
    ///
    /// # Returns
    /// * `Some(MemoryType)` if the string matches a known type
    /// * `None` if the string doesn't match any known type
    pub fn parse_type(s: &str) -> Option<Self> {
        match s {
            "factual" => Some(MemoryType::Factual),
            "episodic" => Some(MemoryType::Episodic),
            "procedural" => Some(MemoryType::Procedural),
            "semantic" => Some(MemoryType::Semantic),
            "working" => Some(MemoryType::Working),
            _ => None,
        }
    }
}

/// An extracted fact from content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedFact {
    pub content: String,
    pub confidence: f32,
    pub entities: Vec<Entity>,
    pub relations: Vec<Relation>,
}

/// An entity extracted from content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub entity_type: String,
    pub attributes: HashMap<String, serde_json::Value>,
}

/// A relation between entities
/// V4: Simplified and aligned with graph model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    /// Relation type (user-defined)
    pub relation_type: String,
    
    /// Source entity ID (for backward compatibility)
    pub source: String,
    
    /// Target entity ID
    pub target: String,
    
    /// Strength/confidence (0.0-1.0)
    pub confidence: f32,
}

/// A vector with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vector {
    pub id: String,
    pub values: Vec<f32>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Vector {
    pub fn new(values: Vec<f32>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            values,
            metadata: HashMap::new(),
        }
    }
}

/// Search result from vector store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Filters for search operations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Filters {
    pub filters: HashMap<String, serde_json::Value>,
}

impl Filters {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, key: &str, value: serde_json::Value) {
        self.filters.insert(key.to_string(), value);
    }
}

/// History entry for memory changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub memory_id: String,
    pub event: MemoryEvent,
    pub timestamp: DateTime<Utc>,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryEvent {
    Create,
    Update,
    Delete,
    Merge,
}

/// Configuration types
pub type Metadata = HashMap<String, serde_json::Value>;

/// Enhanced request types for Mem5 compatibility
///
/// Enhanced add request with all Mem0 compatible parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedAddRequest {
    pub messages: Messages,
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub run_id: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub infer: bool,
    pub memory_type: Option<String>,
    pub prompt: Option<String>,
}

impl EnhancedAddRequest {
    pub fn new(messages: Messages) -> Self {
        Self {
            messages,
            user_id: None,
            agent_id: None,
            run_id: None,
            metadata: None,
            infer: true,
            memory_type: None,
            prompt: None,
        }
    }

    pub fn with_user_id(mut self, user_id: Option<String>) -> Self {
        self.user_id = user_id;
        self
    }

    pub fn with_agent_id(mut self, agent_id: Option<String>) -> Self {
        self.agent_id = agent_id;
        self
    }

    pub fn with_metadata(mut self, metadata: Option<HashMap<String, serde_json::Value>>) -> Self {
        self.metadata = metadata;
        self
    }

    /// Validate the request
    pub fn validate(&self) -> crate::Result<()> {
        self.messages.validate()?;
        Ok(())
    }
}

/// Enhanced search request with all Mem0 compatible parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedSearchRequest {
    pub query: String,
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub run_id: Option<String>,
    pub limit: usize,
    pub filters: Option<HashMap<String, serde_json::Value>>,
    pub threshold: Option<f32>,
}

impl EnhancedSearchRequest {
    pub fn new(query: String) -> Self {
        Self {
            query,
            user_id: None,
            agent_id: None,
            run_id: None,
            limit: 100,
            filters: None,
            threshold: None,
        }
    }

    /// Validate the search request
    pub fn validate(&self) -> crate::Result<()> {
        if self.query.trim().is_empty() {
            return Err(crate::AgentMemError::ValidationError(
                "Empty search query".to_string(),
            ));
        }
        if self.limit == 0 {
            return Err(crate::AgentMemError::ValidationError(
                "Limit must be greater than 0".to_string(),
            ));
        }
        Ok(())
    }
}

/// Memory search result compatible with Mem0
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchResult {
    pub id: String,
    pub content: String,
    pub importance: Option<f64>,
    pub score: f32,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// Batch operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    pub successful: usize,
    pub failed: usize,
    pub results: Vec<String>,
    pub errors: Vec<String>,
    pub execution_time: Duration,
}

/// Metadata builder for easy metadata construction
#[derive(Debug, Clone)]
pub struct MetadataBuilder {
    data: HashMap<String, serde_json::Value>,
}

impl MetadataBuilder {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn user_id(mut self, user_id: String) -> Self {
        self.data
            .insert("user_id".to_string(), serde_json::Value::String(user_id));
        self
    }

    pub fn agent_id(mut self, agent_id: String) -> Self {
        self.data
            .insert("agent_id".to_string(), serde_json::Value::String(agent_id));
        self
    }

    pub fn memory_type(mut self, memory_type: MemoryType) -> Self {
        self.data.insert(
            "memory_type".to_string(),
            serde_json::Value::String(memory_type.to_string()),
        );
        self
    }

    pub fn importance(mut self, score: f64) -> Self {
        self.data.insert(
            "importance".to_string(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(score).unwrap_or_else(|| serde_json::Number::from(0)),
            ),
        );
        self
    }

    pub fn custom<T: Serialize>(mut self, key: String, value: T) -> crate::Result<Self> {
        let value =
            serde_json::to_value(value).map_err(crate::AgentMemError::SerializationError)?;
        self.data.insert(key, value);
        Ok(self)
    }

    pub fn build(self) -> HashMap<String, serde_json::Value> {
        self.data
    }
}

impl Default for MetadataBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Filter builder for search operations
#[derive(Debug, Clone)]
pub struct FilterBuilder {
    filters: HashMap<String, serde_json::Value>,
}

impl FilterBuilder {
    pub fn new() -> Self {
        Self {
            filters: HashMap::new(),
        }
    }

    pub fn user_id(mut self, user_id: String) -> Self {
        self.filters
            .insert("user_id".to_string(), serde_json::Value::String(user_id));
        self
    }

    pub fn agent_id(mut self, agent_id: String) -> Self {
        self.filters
            .insert("agent_id".to_string(), serde_json::Value::String(agent_id));
        self
    }

    pub fn date_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.filters.insert(
            "created_at_gte".to_string(),
            serde_json::Value::String(start.to_rfc3339()),
        );
        self.filters.insert(
            "created_at_lte".to_string(),
            serde_json::Value::String(end.to_rfc3339()),
        );
        self
    }

    pub fn memory_type(mut self, memory_type: MemoryType) -> Self {
        self.filters.insert(
            "memory_type".to_string(),
            serde_json::Value::String(memory_type.to_string()),
        );
        self
    }

    pub fn importance_range(mut self, min: f64, max: f64) -> Self {
        self.filters.insert(
            "importance_gte".to_string(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(min).unwrap_or_else(|| serde_json::Number::from(0)),
            ),
        );
        self.filters.insert(
            "importance_lte".to_string(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(max).unwrap_or_else(|| serde_json::Number::from(1)),
            ),
        );
        self
    }

    pub fn build(self) -> HashMap<String, serde_json::Value> {
        self.filters
    }
}

impl Default for FilterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub response_format: Option<String>,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(),
            model: "gpt-3.5-turbo".to_string(),
            api_key: None,
            base_url: None,
            temperature: Some(0.7),
            max_tokens: Some(1000),
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            response_format: None,
        }
    }
}

/// Vector store configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreConfig {
    pub provider: String,
    pub path: String,
    pub table_name: String,
    pub dimension: Option<usize>,
    pub api_key: Option<String>,
    pub index_name: Option<String>,
    pub url: Option<String>,
    pub collection_name: Option<String>,
}

impl Default for VectorStoreConfig {
    fn default() -> Self {
        Self {
            provider: "memory".to_string(), // Changed from "lancedb" to "memory" for zero-config embedded mode
            path: "".to_string(),           // Empty path for in-memory storage
            table_name: "memories".to_string(),
            dimension: Some(1536),
            api_key: None,
            index_name: None,
            url: None,
            collection_name: None,
        }
    }
}

impl VectorStoreConfig {
    /// Create in-memory vector store configuration (zero-config, embedded mode)
    pub fn memory() -> Self {
        Self::default()
    }

    /// Create LibSQL vector store configuration (local persistence)
    pub fn libsql(path: &str) -> Self {
        Self {
            provider: "libsql".to_string(),
            path: path.to_string(),
            table_name: "memories".to_string(),
            dimension: Some(1536),
            ..Default::default()
        }
    }

    /// Create LanceDB vector store configuration
    pub fn lancedb(path: &str) -> Self {
        Self {
            provider: "lancedb".to_string(),
            path: path.to_string(),
            table_name: "memories".to_string(),
            dimension: Some(1536),
            ..Default::default()
        }
    }

    /// Create Pinecone vector store configuration
    pub fn pinecone(api_key: &str, index_name: &str) -> Self {
        Self {
            provider: "pinecone".to_string(),
            api_key: Some(api_key.to_string()),
            index_name: Some(index_name.to_string()),
            dimension: Some(1536),
            ..Default::default()
        }
    }

    /// Create Qdrant vector store configuration
    pub fn qdrant(url: &str, collection_name: &str) -> Self {
        Self {
            provider: "qdrant".to_string(),
            url: Some(url.to_string()),
            collection_name: Some(collection_name.to_string()),
            dimension: Some(1536),
            ..Default::default()
        }
    }
}

/// 向量数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorData {
    pub id: String,
    pub vector: Vec<f32>,
    pub metadata: std::collections::HashMap<String, String>,
}

/// 向量搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSearchResult {
    pub id: String,
    pub vector: Vec<f32>,
    pub metadata: std::collections::HashMap<String, String>,
    pub similarity: f32,
    pub distance: f32,
}

impl std::fmt::Display for MemoryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Legacy type
            MemoryType::Factual => write!(f, "factual"),
            // Basic cognitive memories
            MemoryType::Episodic => write!(f, "episodic"),
            MemoryType::Procedural => write!(f, "procedural"),
            MemoryType::Semantic => write!(f, "semantic"),
            MemoryType::Working => write!(f, "working"),
            // Advanced cognitive memories (AgentMem 7.0)
            MemoryType::Core => write!(f, "core"),
            MemoryType::Resource => write!(f, "resource"),
            MemoryType::Knowledge => write!(f, "knowledge"),
            MemoryType::Contextual => write!(f, "contextual"),
        }
    }
}

impl std::str::FromStr for MemoryType {
    type Err = crate::AgentMemError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            // Legacy type
            "factual" => Ok(MemoryType::Factual),
            // Basic cognitive memories
            "episodic" => Ok(MemoryType::Episodic),
            "procedural" => Ok(MemoryType::Procedural),
            "semantic" => Ok(MemoryType::Semantic),
            "working" => Ok(MemoryType::Working),
            // Advanced cognitive memories (AgentMem 7.0)
            "core" => Ok(MemoryType::Core),
            "resource" => Ok(MemoryType::Resource),
            "knowledge" => Ok(MemoryType::Knowledge),
            "contextual" => Ok(MemoryType::Contextual),
            _ => Err(crate::AgentMemError::ValidationError(format!(
                "Invalid memory type: {s}. Valid types: factual, episodic, procedural, semantic, working, core, resource, knowledge, contextual"
            ))),
        }
    }
}

/// Processing options for memory operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingOptions {
    pub extract_facts: bool,
    pub update_existing: bool,
    pub resolve_conflicts: bool,
    pub calculate_importance: bool,
}

impl Default for ProcessingOptions {
    fn default() -> Self {
        Self {
            extract_facts: true,
            update_existing: true,
            resolve_conflicts: true,
            calculate_importance: true,
        }
    }
}

/// Processing result from intelligent memory operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingResult {
    pub memory_id: String,
    pub facts_extracted: Vec<ExtractedFact>,
    pub conflicts_resolved: Vec<String>,
    pub importance_score: f64,
    pub confidence: f64,
}

/// Health status for system components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub details: HashMap<String, serde_json::Value>,
}

impl HealthStatus {
    pub fn healthy() -> Self {
        Self {
            status: "healthy".to_string(),
            message: "All systems operational".to_string(),
            timestamp: Utc::now(),
            details: HashMap::new(),
        }
    }

    pub fn unhealthy(message: &str) -> Self {
        Self {
            status: "unhealthy".to_string(),
            message: message.to_string(),
            timestamp: Utc::now(),
            details: HashMap::new(),
        }
    }

    pub fn with_details(mut self, details: HashMap<String, serde_json::Value>) -> Self {
        self.details = details;
        self
    }
}

/// System metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub memory_usage: u64,
    pub cpu_usage: f64,
    pub operations_per_second: f64,
    pub error_rate: f64,
    pub average_response_time: Duration,
    pub timestamp: DateTime<Utc>,
}

/// Performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub average_latency: Duration,
    pub p95_latency: Duration,
    pub p99_latency: Duration,
    pub throughput: f64,
}
