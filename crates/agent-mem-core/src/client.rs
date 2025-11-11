//! Mem0 Compatible Client Interface
//!
//! This module provides a Mem0-compatible API interface for AgentMem,
//! enabling seamless migration from Mem0 to AgentMem.

use crate::{MemoryEngine, MemoryEngineConfig};
use agent_mem_traits::{
    AgentMemError, LLMConfig, Message as LLMMessage, MessageRole as LLMMessageRole, Result,
};
use chrono::{DateTime, Utc};
use futures::future;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};
use uuid::Uuid;

/// Memory type for client API
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MemoryType {
    /// Episodic memories - specific events and experiences
    Episodic,
    /// Semantic memories - facts and general knowledge
    Semantic,
    /// Procedural memories - skills and procedures
    Procedural,
    /// Working memories - temporary information
    Working,
    /// Core memories - persistent identity and preferences
    Core,
    /// Resource memories - multimedia content and documents
    Resource,
    /// Knowledge memories - structured knowledge graphs
    Knowledge,
    /// Contextual memories - environment-aware information
    Contextual,
}

impl ToString for MemoryType {
    fn to_string(&self) -> String {
        match self {
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
}

/// Client-side Memory structure compatible with Mem0
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    /// Unique memory identifier
    pub id: String,
    /// Memory content
    pub content: String,
    /// Memory type
    pub memory_type: MemoryType,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Relevance score
    pub score: Option<f32>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Update timestamp
    pub updated_at: DateTime<Utc>,
}

// TODO: Implement conversion functions between client and core Memory types
// impl Memory {
//     fn to_core_memory(&self, agent_id: String, user_id: Option<String>) -> CoreMemory { ... }
//     fn from_core_memory(core_memory: &CoreMemory) -> Self { ... }
// }

/// Messages input type - compatible with Mem0
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Messages {
    /// Single string message
    Single(String),
    /// Structured message
    Structured(Message),
    /// Multiple messages
    Multiple(Vec<Message>),
}

/// Message structure compatible with Mem0
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message role (user, assistant, system)
    pub role: String,
    /// Message content
    pub content: String,
    /// Optional metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Message {
    /// Create a user message
    pub fn user(content: String) -> Self {
        Self {
            role: "user".to_string(),
            content,
            metadata: HashMap::new(),
        }
    }

    /// Create an assistant message
    pub fn assistant(content: String) -> Self {
        Self {
            role: "assistant".to_string(),
            content,
            metadata: HashMap::new(),
        }
    }

    /// Create a system message
    pub fn system(content: String) -> Self {
        Self {
            role: "system".to_string(),
            content,
            metadata: HashMap::new(),
        }
    }

    /// Validate message
    pub fn validate(&self) -> Result<()> {
        if self.content.trim().is_empty() {
            return Err(AgentMemError::validation_error(
                "Message content cannot be empty",
            ));
        }

        if !["user", "assistant", "system"].contains(&self.role.as_str()) {
            return Err(AgentMemError::validation_error("Invalid message role"));
        }

        Ok(())
    }
}

impl Messages {
    /// Validate messages
    pub fn validate(&self) -> Result<()> {
        match self {
            Messages::Single(s) => {
                if s.trim().is_empty() {
                    return Err(AgentMemError::validation_error("Empty message"));
                }
            }
            Messages::Structured(msg) => msg.validate()?,
            Messages::Multiple(msgs) => {
                if msgs.is_empty() {
                    return Err(AgentMemError::validation_error("Empty message list"));
                }
                for msg in msgs {
                    msg.validate()?;
                }
            }
        }
        Ok(())
    }

    /// Convert to message list
    pub fn to_message_list(&self) -> Vec<Message> {
        match self {
            Messages::Single(s) => vec![Message::user(s.clone())],
            Messages::Structured(msg) => vec![msg.clone()],
            Messages::Multiple(msgs) => msgs.clone(),
        }
    }
}

/// Add request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddRequest {
    /// Messages to add
    pub messages: Messages,
    /// User ID
    pub user_id: Option<String>,
    /// Agent ID
    pub agent_id: Option<String>,
    /// Run ID
    pub run_id: Option<String>,
    /// Metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    /// Whether to infer memory type
    pub infer: bool,
    /// Memory type
    pub memory_type: Option<MemoryType>,
    /// Custom prompt
    pub prompt: Option<String>,
}

/// Add result structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddResult {
    /// Memory ID
    pub id: String,
    /// Success status
    pub success: bool,
    /// Optional message
    pub message: Option<String>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
}

/// Search result structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Found memories
    pub results: Vec<MemorySearchResult>,
    /// Total count
    pub total: usize,
    /// Query used
    pub query: String,
}

/// Individual memory search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchResult {
    /// Memory ID
    pub id: String,
    /// Memory content
    pub content: String,
    /// Relevance score
    pub score: f32,
    /// Memory type
    pub memory_type: MemoryType,
    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Update request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRequest {
    /// Memory ID to update
    pub memory_id: String,
    /// New content
    pub content: Option<String>,
    /// New metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Update result structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateResult {
    /// Memory ID
    pub id: String,
    /// Success status
    pub success: bool,
    /// Optional message
    pub message: Option<String>,
    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Filter builder for complex queries
#[derive(Debug, Clone, Default)]
pub struct FilterBuilder {
    filters: HashMap<String, serde_json::Value>,
}

impl FilterBuilder {
    /// Create new filter builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Add user ID filter
    pub fn user_id(mut self, user_id: String) -> Self {
        self.filters
            .insert("user_id".to_string(), serde_json::Value::String(user_id));
        self
    }

    /// Add agent ID filter
    pub fn agent_id(mut self, agent_id: String) -> Self {
        self.filters
            .insert("agent_id".to_string(), serde_json::Value::String(agent_id));
        self
    }

    /// Add run ID filter
    pub fn run_id(mut self, run_id: String) -> Self {
        self.filters
            .insert("run_id".to_string(), serde_json::Value::String(run_id));
        self
    }

    /// Add date range filter
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

    /// Add memory type filter
    pub fn memory_type(mut self, memory_type: MemoryType) -> Self {
        self.filters.insert(
            "memory_type".to_string(),
            serde_json::Value::String(memory_type.to_string()),
        );
        self
    }

    /// Add custom filter
    pub fn custom<T: Serialize>(mut self, key: String, value: T) -> Result<Self> {
        let value = serde_json::to_value(value)
            .map_err(|e| AgentMemError::validation_error(format!("Invalid filter value: {e}")))?;
        self.filters.insert(key, value);
        Ok(self)
    }

    /// Build filters
    pub fn build(self) -> HashMap<String, serde_json::Value> {
        self.filters
    }
}

/// Metadata builder for structured metadata
#[derive(Debug, Clone, Default)]
pub struct MetadataBuilder {
    data: HashMap<String, serde_json::Value>,
}

impl MetadataBuilder {
    /// Create new metadata builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Add user ID
    pub fn user_id(mut self, user_id: String) -> Self {
        self.data
            .insert("user_id".to_string(), serde_json::Value::String(user_id));
        self
    }

    /// Add agent ID
    pub fn agent_id(mut self, agent_id: String) -> Self {
        self.data
            .insert("agent_id".to_string(), serde_json::Value::String(agent_id));
        self
    }

    /// Add run ID
    pub fn run_id(mut self, run_id: String) -> Self {
        self.data
            .insert("run_id".to_string(), serde_json::Value::String(run_id));
        self
    }

    /// Add custom metadata
    pub fn custom<T: Serialize>(mut self, key: String, value: T) -> Result<Self> {
        let value = serde_json::to_value(value)
            .map_err(|e| AgentMemError::validation_error(format!("Invalid metadata value: {e}")))?;
        self.data.insert(key, value);
        Ok(self)
    }

    /// Build metadata
    pub fn build(self) -> HashMap<String, serde_json::Value> {
        self.data
    }
}

/// Performance configuration for the client
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Maximum concurrent operations
    pub max_concurrent_operations: usize,
    /// Request timeout in seconds
    pub request_timeout_seconds: u64,
    /// Enable request batching
    pub enable_batching: bool,
    /// Batch size for operations
    pub batch_size: usize,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_concurrent_operations: 100,
            request_timeout_seconds: 30,
            enable_batching: true,
            batch_size: 50,
        }
    }
}

/// User structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// User ID
    pub id: String,
    /// User name
    pub name: String,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Memory visualization structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryVisualization {
    /// User ID
    pub user_id: String,
    /// User name
    pub user_name: String,
    /// Memory summary statistics
    pub summary: MemorySummary,
    /// Memories grouped by type
    pub memories: MemoriesByType,
}

/// Memory summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySummary {
    /// Total number of memories
    pub total_count: usize,
    /// Number of episodic memories
    pub episodic_count: usize,
    /// Number of semantic memories
    pub semantic_count: usize,
    /// Number of procedural memories
    pub procedural_count: usize,
    /// Number of core memories
    pub core_count: usize,
    /// Number of resource memories
    pub resource_count: usize,
    /// Number of knowledge memories
    pub knowledge_count: usize,
    /// Number of working memories
    pub working_count: usize,
    /// Number of contextual memories
    pub contextual_count: usize,
}

/// Memories grouped by type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoriesByType {
    /// Episodic memories
    pub episodic: Vec<MemorySearchResult>,
    /// Semantic memories
    pub semantic: Vec<MemorySearchResult>,
    /// Procedural memories
    pub procedural: Vec<MemorySearchResult>,
    /// Core memories
    pub core: Vec<MemorySearchResult>,
    /// Resource memories
    pub resource: Vec<MemorySearchResult>,
    /// Knowledge memories
    pub knowledge: Vec<MemorySearchResult>,
    /// Working memories
    pub working: Vec<MemorySearchResult>,
    /// Contextual memories
    pub contextual: Vec<MemorySearchResult>,
}

/// Mem0 compatible client configuration
#[derive(Debug, Clone)]
pub struct AgentMemClientConfig {
    /// Memory engine configuration
    pub engine: MemoryEngineConfig,
    /// Performance configuration
    pub performance: PerformanceConfig,
    /// LLM configuration (optional)
    pub llm: Option<LLMConfig>,
    /// Enable telemetry
    pub enable_telemetry: bool,
    /// Enable error recovery
    pub enable_error_recovery: bool,
}

impl Default for AgentMemClientConfig {
    fn default() -> Self {
        Self {
            engine: MemoryEngineConfig::default(),
            performance: PerformanceConfig::default(),
            llm: None, // LLM is optional by default
            enable_telemetry: true,
            enable_error_recovery: true,
        }
    }
}

/// Mem0 compatible AgentMem client
#[derive(Clone)]
pub struct AgentMemClient {
    engine: Arc<MemoryEngine>,
    config: AgentMemClientConfig,
    semaphore: Arc<Semaphore>,
    user_storage: Arc<RwLock<HashMap<String, User>>>, // 简化的内存存储
    llm_client: Option<Arc<agent_mem_llm::LLMClient>>, // LLM 客户端（可选）
}

impl AgentMemClient {
    /// Create new AgentMem client
    pub fn new(config: AgentMemClientConfig) -> Self {
        let engine = Arc::new(MemoryEngine::new(config.engine.clone()));
        let semaphore = Arc::new(Semaphore::new(config.performance.max_concurrent_operations));

        // 初始化 LLM 客户端（如果配置了）
        let llm_client = config.llm.as_ref().and_then(|llm_config| {
            match agent_mem_llm::LLMClient::new(llm_config) {
                Ok(client) => Some(Arc::new(client)),
                Err(e) => {
                    eprintln!("Warning: Failed to initialize LLM client: {e}");
                    None
                }
            }
        });

        Self {
            engine,
            config,
            semaphore,
            user_storage: Arc::new(RwLock::new(HashMap::new())),
            llm_client,
        }
    }

    /// Create client with default configuration
    pub fn default() -> Self {
        Self::new(AgentMemClientConfig::default())
    }

    /// Convert CoreError to AgentMemError
    fn convert_error(err: crate::CoreError) -> AgentMemError {
        match err {
            crate::CoreError::Storage(msg) => AgentMemError::storage_error(&msg),
            crate::CoreError::Database(msg) => AgentMemError::storage_error(&msg),
            crate::CoreError::NotFound(msg) => AgentMemError::not_found(&msg),
            crate::CoreError::ValidationError(msg) => AgentMemError::validation_error(&msg),
            crate::CoreError::InvalidInput(msg) => AgentMemError::validation_error(&msg),
            crate::CoreError::SerializationError(msg) => {
                AgentMemError::internal_error(format!("Serialization error: {msg}"))
            }
            _ => AgentMemError::internal_error(err.to_string()),
        }
    }

    /// Add memory - Mem0 compatible API
    pub async fn add(
        &self,
        messages: Messages,
        user_id: Option<String>,
        agent_id: Option<String>,
        run_id: Option<String>,
        metadata: Option<HashMap<String, serde_json::Value>>,
        infer: bool,
        memory_type: Option<MemoryType>,
        prompt: Option<String>,
    ) -> Result<AddResult> {
        let request = AddRequest {
            messages,
            user_id,
            agent_id,
            run_id,
            metadata,
            infer,
            memory_type,
            prompt,
        };

        self.add_single(request).await
    }

    /// Search memories - Mem0 compatible API
    pub async fn search(
        &self,
        query: String,
        user_id: Option<String>,
        agent_id: Option<String>,
        run_id: Option<String>,
        limit: usize,
        filters: Option<HashMap<String, serde_json::Value>>,
        _threshold: Option<f32>,
    ) -> Result<SearchResult> {
        // Validate inputs
        if query.trim().is_empty() {
            return Err(AgentMemError::validation_error("Query cannot be empty"));
        }

        if limit == 0 {
            return Err(AgentMemError::validation_error(
                "Limit must be greater than 0",
            ));
        }

        // Acquire semaphore permit
        let _permit =
            self.semaphore.acquire().await.map_err(|e| {
                AgentMemError::internal_error(format!("Failed to acquire permit: {e}"))
            })?;

        // Search using engine
        let memories = self
            .engine
            .search_memories(&query, None, Some(limit))
            .await
            .map_err(Self::convert_error)?;

        // Convert to search results and apply filters
        let results: Vec<MemorySearchResult> = memories
            .into_iter()
            .filter(|memory| {
                // Apply user_id filter
                if let Some(ref uid) = user_id {
                    if memory.user_id().as_ref() != Some(uid) {
                        return false;
                    }
                }
                // Apply agent_id filter
                if let Some(ref aid) = agent_id {
                    if memory.agent_id().as_ref() != Some(aid) {
                        return false;
                    }
                }
                // Apply run_id filter - not directly supported in V4, skip
                if run_id.is_some() {
                    // V4 doesn't have run_id in the same way
                    return true;
                }
                true
            })
            .map(|memory| {
                let content = match &memory.content {
                    agent_mem_traits::Content::Text(t) => t.clone(),
                    agent_mem_traits::Content::Structured(v) => v.to_string(),
                    _ => String::new(),
                };
                let mem_type_str = memory.memory_type().unwrap_or_else(|| "episodic".to_string());
                let mem_type = match mem_type_str.as_str() {
                    "semantic" => MemoryType::Semantic,
                    "procedural" => MemoryType::Procedural,
                    "working" => MemoryType::Working,
                    "core" => MemoryType::Core,
                    _ => MemoryType::Episodic,
                };
                
                MemorySearchResult {
                    id: memory.id.as_str().to_string(),
                    content,
                    score: memory.score().unwrap_or(0.0) as f32,
                    memory_type: mem_type,
                    metadata: HashMap::new(), // V4 doesn't expose metadata as HashMap directly
                    created_at: memory.metadata.created_at,
                    updated_at: memory.metadata.updated_at,
                }
            })
            .collect();

        let total = results.len();

        Ok(SearchResult {
            results,
            total,
            query,
        })
    }

    /// Get memory by ID
    pub async fn get(&self, memory_id: String) -> Result<Option<MemorySearchResult>> {
        if memory_id.trim().is_empty() {
            return Err(AgentMemError::validation_error("Memory ID cannot be empty"));
        }

        // Acquire semaphore permit
        let _permit =
            self.semaphore.acquire().await.map_err(|e| {
                AgentMemError::internal_error(format!("Failed to acquire permit: {e}"))
            })?;

        // Get from engine
        let memory_opt = self
            .engine
            .get_memory(&memory_id)
            .await
            .map_err(Self::convert_error)?;

        Ok(memory_opt.map(|memory| {
            let content = match &memory.content {
                agent_mem_traits::Content::Text(t) => t.clone(),
                agent_mem_traits::Content::Structured(v) => v.to_string(),
                _ => String::new(),
            };
            let mem_type_str = memory.memory_type().unwrap_or_else(|| "episodic".to_string());
            let mem_type = match mem_type_str.as_str() {
                "semantic" => MemoryType::Semantic,
                "procedural" => MemoryType::Procedural,
                "working" => MemoryType::Working,
                "core" => MemoryType::Core,
                _ => MemoryType::Episodic,
            };
            
            MemorySearchResult {
                id: memory.id.as_str().to_string(),
                content,
                score: memory.score().unwrap_or(0.0) as f32,
                memory_type: mem_type,
                metadata: HashMap::new(),
                created_at: memory.metadata.created_at,
                updated_at: memory.metadata.updated_at,
            }
        }))
    }

    /// Update memory
    pub async fn update(&self, request: UpdateRequest) -> Result<UpdateResult> {
        if request.memory_id.trim().is_empty() {
            return Err(AgentMemError::validation_error("Memory ID cannot be empty"));
        }

        // Acquire semaphore permit
        let _permit =
            self.semaphore.acquire().await.map_err(|e| {
                AgentMemError::internal_error(format!("Failed to acquire permit: {e}"))
            })?;

        // Get existing memory
        let mut memory = self
            .engine
            .get_memory(&request.memory_id)
            .await
            .map_err(Self::convert_error)?
            .ok_or_else(|| {
                AgentMemError::not_found(format!("Memory {} not found", request.memory_id))
            })?;

        // Update content if provided
        if let Some(content) = request.content {
            memory.content = agent_mem_traits::Content::Text(content);
        }

        // Update metadata if provided - V4 doesn't support arbitrary metadata update this way
        if request.metadata.is_some() {
            // Metadata update not supported in V4 directly
        }

        // Update timestamp
        memory.metadata.updated_at = chrono::Utc::now();

        // Update in engine
        let updated_memory = self
            .engine
            .update_memory(memory)
            .await
            .map_err(Self::convert_error)?;

        Ok(UpdateResult {
            id: updated_memory.id.as_str().to_string(),
            success: true,
            message: Some("Memory updated successfully".to_string()),
            updated_at: Utc::now(),
        })
    }

    /// Delete memory
    pub async fn delete(&self, memory_id: String) -> Result<bool> {
        if memory_id.trim().is_empty() {
            return Err(AgentMemError::validation_error("Memory ID cannot be empty"));
        }

        // Acquire semaphore permit
        let _permit =
            self.semaphore.acquire().await.map_err(|e| {
                AgentMemError::internal_error(format!("Failed to acquire permit: {e}"))
            })?;

        // Delete from engine
        let deleted = self
            .engine
            .remove_memory(&memory_id)
            .await
            .map_err(Self::convert_error)?;

        Ok(deleted)
    }

    /// Get all memories for a user
    pub async fn get_all(
        &self,
        user_id: Option<String>,
        agent_id: Option<String>,
        run_id: Option<String>,
        limit: Option<usize>,
    ) -> Result<Vec<MemorySearchResult>> {
        // Acquire semaphore permit
        let _permit =
            self.semaphore.acquire().await.map_err(|e| {
                AgentMemError::internal_error(format!("Failed to acquire permit: {e}"))
            })?;

        // Get all memories from engine (using empty query to get all)
        let memories = self
            .engine
            .search_memories("", None, limit)
            .await
            .map_err(Self::convert_error)?;

        // Apply filters
        let results: Vec<MemorySearchResult> = memories
            .into_iter()
            .filter(|memory| {
                // Apply user_id filter
                if let Some(ref uid) = user_id {
                    if memory.user_id().as_ref() != Some(uid) {
                        return false;
                    }
                }
                // Apply agent_id filter
                if let Some(ref aid) = agent_id {
                    if memory.agent_id().as_ref() != Some(aid) {
                        return false;
                    }
                }
                // Apply run_id filter - V4 doesn't support run_id
                if run_id.is_some() {
                    // Skip this filter in V4
                    return true;
                }
                true
            })
            .map(|memory| {
                let content = match &memory.content {
                    agent_mem_traits::Content::Text(t) => t.clone(),
                    agent_mem_traits::Content::Structured(v) => v.to_string(),
                    _ => String::new(),
                };
                let mem_type_str = memory.memory_type().unwrap_or_else(|| "episodic".to_string());
                let mem_type = match mem_type_str.as_str() {
                    "semantic" => MemoryType::Semantic,
                    "procedural" => MemoryType::Procedural,
                    "working" => MemoryType::Working,
                    "core" => MemoryType::Core,
                    _ => MemoryType::Episodic,
                };
                
                MemorySearchResult {
                    id: memory.id.as_str().to_string(),
                    content,
                    score: memory.score().unwrap_or(0.0) as f32,
                    memory_type: mem_type,
                    metadata: HashMap::new(),
                    created_at: memory.metadata.created_at,
                    updated_at: memory.metadata.updated_at,
                }
            })
            .collect();

        Ok(results)
    }

    /// Reset all memories (dangerous operation)
    pub async fn reset(&self) -> Result<bool> {
        // TODO: Implement reset functionality
        // This should clear all memories
        Ok(true)
    }

    /// Add multiple memories in batch - Mem0 compatible API
    pub async fn add_batch(&self, requests: Vec<AddRequest>) -> Result<Vec<AddResult>> {
        if requests.is_empty() {
            return Ok(vec![]);
        }

        // Use semaphore to control concurrency
        let semaphore = Arc::new(Semaphore::new(
            self.config.performance.max_concurrent_operations,
        ));
        let mut tasks = Vec::new();

        for request in requests {
            let client = self.clone();
            let permit = semaphore.clone().acquire_owned().await.map_err(|e| {
                AgentMemError::internal_error(format!("Failed to acquire permit: {e}"))
            })?;

            let task = tokio::spawn(async move {
                let _permit = permit;
                client.add_single(request).await
            });

            tasks.push(task);
        }

        // Wait for all tasks to complete
        let results = future::join_all(tasks).await;
        let mut final_results = Vec::new();

        for result in results {
            match result {
                Ok(Ok(add_result)) => final_results.push(add_result),
                Ok(Err(e)) => return Err(e),
                Err(e) => return Err(AgentMemError::internal_error(format!("Task failed: {e}"))),
            }
        }

        Ok(final_results)
    }

    /// Update multiple memories in batch
    pub async fn update_batch(&self, requests: Vec<UpdateRequest>) -> Result<Vec<UpdateResult>> {
        if requests.is_empty() {
            return Ok(vec![]);
        }

        let semaphore = Arc::new(Semaphore::new(
            self.config.performance.max_concurrent_operations,
        ));
        let mut tasks = Vec::new();

        for request in requests {
            let client = self.clone();
            let permit = semaphore.clone().acquire_owned().await.map_err(|e| {
                AgentMemError::internal_error(format!("Failed to acquire permit: {e}"))
            })?;

            let task = tokio::spawn(async move {
                let _permit = permit;
                client.update(request).await
            });

            tasks.push(task);
        }

        // Wait for all tasks to complete
        let results = future::join_all(tasks).await;
        let mut final_results = Vec::new();

        for result in results {
            match result {
                Ok(Ok(update_result)) => final_results.push(update_result),
                Ok(Err(e)) => return Err(e),
                Err(e) => return Err(AgentMemError::internal_error(format!("Task failed: {e}"))),
            }
        }

        Ok(final_results)
    }

    /// Delete multiple memories in batch
    pub async fn delete_batch(&self, memory_ids: Vec<String>) -> Result<Vec<bool>> {
        if memory_ids.is_empty() {
            return Ok(vec![]);
        }

        let semaphore = Arc::new(Semaphore::new(
            self.config.performance.max_concurrent_operations,
        ));
        let mut tasks = Vec::new();

        for memory_id in memory_ids {
            let client = self.clone();
            let permit = semaphore.clone().acquire_owned().await.map_err(|e| {
                AgentMemError::internal_error(format!("Failed to acquire permit: {e}"))
            })?;

            let task = tokio::spawn(async move {
                let _permit = permit;
                client.delete(memory_id).await
            });

            tasks.push(task);
        }

        // Wait for all tasks to complete
        let results = future::join_all(tasks).await;
        let mut final_results = Vec::new();

        for result in results {
            match result {
                Ok(Ok(delete_result)) => final_results.push(delete_result),
                Ok(Err(_)) => final_results.push(false), // Failed to delete
                Err(_) => final_results.push(false),     // Task failed
            }
        }

        Ok(final_results)
    }

    /// Internal method to add a single memory
    async fn add_single(&self, request: AddRequest) -> Result<AddResult> {
        // Validate request
        request.messages.validate()?;

        // Acquire semaphore permit for concurrency control
        let _permit =
            self.semaphore.acquire().await.map_err(|e| {
                AgentMemError::internal_error(format!("Failed to acquire permit: {e}"))
            })?;

        // Convert messages to memory content
        let messages = request.messages.to_message_list();
        let content = messages
            .iter()
            .map(|m| format!("{}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n");

        // Create memory object using the Memory::new constructor
        let agent_id = request.agent_id.unwrap_or_else(|| "default".to_string());
        let user_id = request.user_id;
        let memory_type = match request.memory_type {
            Some(MemoryType::Episodic) => agent_mem_traits::MemoryType::Episodic,
            Some(MemoryType::Semantic) => agent_mem_traits::MemoryType::Semantic,
            Some(MemoryType::Procedural) => agent_mem_traits::MemoryType::Procedural,
            Some(MemoryType::Working) => agent_mem_traits::MemoryType::Working,
            Some(MemoryType::Core) => agent_mem_traits::MemoryType::Core,
            Some(MemoryType::Resource) => agent_mem_traits::MemoryType::Resource,
            Some(MemoryType::Knowledge) => agent_mem_traits::MemoryType::Knowledge,
            Some(MemoryType::Contextual) => agent_mem_traits::MemoryType::Contextual,
            None => agent_mem_traits::MemoryType::Episodic, // Default to episodic
        };

        let now = Utc::now();

        // Create MemoryItem (which is aliased as crate::Memory)
        let memory_item = agent_mem_traits::MemoryItem {
            id: Uuid::new_v4().to_string(),
            content,
            hash: None,
            metadata: request.metadata.unwrap_or_default(),
            score: Some(0.5), // Default importance
            created_at: now,
            updated_at: Some(now),
            session: agent_mem_traits::Session {
                id: Uuid::new_v4().to_string(),
                user_id: user_id.clone(),
                agent_id: Some(agent_id.clone()),
                run_id: None,
                actor_id: None,
                created_at: now,
                metadata: HashMap::new(),
            },
            memory_type,
            entities: Vec::new(),
            relations: Vec::new(),
            agent_id,
            user_id,
            importance: 0.5,
            embedding: None,
            last_accessed_at: now,
            access_count: 0,
            expires_at: None,
            version: 1,
        };

        // Add to engine
        // Convert MemoryItem to V4 Memory
        use crate::storage::conversion::legacy_to_v4;
        let v4_memory = legacy_to_v4(&memory_item);
        
        let id = self
            .engine
            .add_memory(v4_memory)
            .await
            .map_err(Self::convert_error)?;

        Ok(AddResult {
            id,
            success: true,
            message: Some("Memory added successfully".to_string()),
            created_at: Utc::now(),
        })
    }

    // ==================== 用户管理 API ====================

    /// 创建新用户
    pub async fn create_user(&self, user_name: String) -> Result<User> {
        // 验证用户名
        if user_name.trim().is_empty() {
            return Err(AgentMemError::validation_error("User name cannot be empty"));
        }

        // 检查用户是否已存在
        {
            let storage = self.user_storage.read().await;
            if let Some(existing_user) = storage.get(&user_name) {
                return Ok(existing_user.clone());
            }
        }

        // 创建新用户
        let user = User {
            id: Uuid::new_v4().to_string(),
            name: user_name.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // 保存到内存存储
        {
            let mut storage = self.user_storage.write().await;
            storage.insert(user_name, user.clone());
        }

        Ok(user)
    }

    /// 列出所有用户
    pub async fn list_users(&self) -> Result<Vec<User>> {
        let storage = self.user_storage.read().await;
        Ok(storage.values().cloned().collect())
    }

    /// 按名称查询用户
    pub async fn get_user_by_name(&self, user_name: String) -> Result<Option<User>> {
        let storage = self.user_storage.read().await;
        Ok(storage.get(&user_name).cloned())
    }

    /// 按 ID 查询用户
    pub async fn get_user_by_id(&self, user_id: String) -> Result<Option<User>> {
        let storage = self.user_storage.read().await;
        Ok(storage.values().find(|u| u.id == user_id).cloned())
    }

    // ==================== 系统提示和对话功能 ====================

    /// 提取记忆用于系统提示
    ///
    /// 搜索与给定消息相关的记忆，并格式化为系统提示文本
    ///
    /// # Arguments
    ///
    /// * `message` - 用户消息
    /// * `user_id` - 用户 ID（可选）
    /// * `limit` - 返回的记忆数量限制（默认为 5）
    ///
    /// # Returns
    ///
    /// 返回格式化的系统提示文本
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use agent_mem_core::AgentMemClient;
    /// # async fn example() -> agent_mem_traits::Result<()> {
    /// let client = AgentMemClient::default();
    /// let prompt = client.extract_memory_for_system_prompt(
    ///     "What do you know about me?".to_string(),
    ///     Some("alice".to_string()),
    ///     Some(5),
    /// ).await?;
    /// println!("System prompt: {}", prompt);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn extract_memory_for_system_prompt(
        &self,
        message: String,
        user_id: Option<String>,
        limit: Option<usize>,
    ) -> Result<String> {
        // 1. 搜索相关记忆
        let search_results = self
            .search(
                message.clone(),
                user_id.clone(),
                None,
                None,
                limit.unwrap_or(5),
                None,
                None,
            )
            .await?;

        // 2. 格式化为提示文本
        if search_results.results.is_empty() {
            return Ok("No relevant memories found.".to_string());
        }

        let mut prompt = String::from("Relevant memories:\n\n");

        for (i, result) in search_results.results.iter().enumerate() {
            prompt.push_str(&format!(
                "{}. [{}] {} (score: {:.2})\n",
                i + 1,
                result.memory_type.to_string(),
                result.content,
                result.score
            ));
        }

        Ok(prompt)
    }

    /// 构建系统消息
    ///
    /// 提取相关记忆并构建完整的系统消息，包含记忆上下文和用户消息
    ///
    /// # Arguments
    ///
    /// * `message` - 用户消息
    /// * `user_id` - 用户 ID（可选）
    /// * `system_prefix` - 系统消息前缀（可选，默认为标准提示）
    ///
    /// # Returns
    ///
    /// 返回完整的系统消息
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use agent_mem_core::AgentMemClient;
    /// # async fn example() -> agent_mem_traits::Result<()> {
    /// let client = AgentMemClient::default();
    /// let system_message = client.construct_system_message(
    ///     "Tell me about my preferences".to_string(),
    ///     Some("alice".to_string()),
    ///     None,
    /// ).await?;
    /// println!("System message: {}", system_message);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn construct_system_message(
        &self,
        message: String,
        user_id: Option<String>,
        system_prefix: Option<String>,
    ) -> Result<String> {
        // 1. 提取记忆上下文
        let memory_context = self
            .extract_memory_for_system_prompt(message.clone(), user_id, Some(5))
            .await?;

        // 2. 构建完整消息
        let prefix = system_prefix.unwrap_or_else(|| {
            "You are a helpful AI assistant with access to the following memories:".to_string()
        });

        Ok(format!("{prefix}\n\n{memory_context}\n\nUser: {message}"))
    }

    // ==================== 清空功能 ====================

    /// 清空所有记忆
    ///
    /// 删除指定用户的所有记忆。如果未指定用户，则删除所有记忆。
    ///
    /// # Arguments
    ///
    /// * `user_id` - 用户 ID（可选）
    ///
    /// # Returns
    ///
    /// 返回删除的记忆数量
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use agent_mem_core::client::AgentMemClient;
    /// # async fn example() -> agent_mem_traits::Result<()> {
    /// let client = AgentMemClient::default();
    /// let count = client.clear(Some("alice".to_string())).await?;
    /// println!("Deleted {} memories", count);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn clear(&self, user_id: Option<String>) -> Result<usize> {
        let memories = self.get_all(user_id, None, None, None).await?;
        let count = memories.len();

        for memory in memories {
            self.delete(memory.id).await?;
        }

        Ok(count)
    }

    /// 清空对话历史
    ///
    /// 只删除情景记忆（对话历史），保留其他类型的记忆
    ///
    /// # Arguments
    ///
    /// * `user_id` - 用户 ID
    ///
    /// # Returns
    ///
    /// 返回删除的对话记录数量
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use agent_mem_core::client::AgentMemClient;
    /// # async fn example() -> agent_mem_traits::Result<()> {
    /// let client = AgentMemClient::default();
    /// let count = client.clear_conversation_history("alice".to_string()).await?;
    /// println!("Deleted {} conversation records", count);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn clear_conversation_history(&self, user_id: String) -> Result<usize> {
        let memories = self.get_all(Some(user_id), None, None, None).await?;

        // 只删除情景记忆（对话历史）
        let mut count = 0;
        for memory in memories {
            if memory.memory_type == MemoryType::Episodic {
                self.delete(memory.id).await?;
                count += 1;
            }
        }

        Ok(count)
    }

    /// Chat with the AI assistant using memory context
    ///
    /// This method provides a complete chat experience by:
    /// 1. Extracting relevant memories for context
    /// 2. Constructing a system message with memory context
    /// 3. Calling the LLM to generate a response
    /// 4. Optionally saving the conversation to memory
    ///
    /// # Arguments
    ///
    /// * `message` - The user's message
    /// * `user_id` - Optional user ID for personalized context
    /// * `save_to_memory` - Whether to save the conversation to memory
    ///
    /// # Returns
    ///
    /// Returns the AI assistant's response
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - LLM is not configured
    /// - LLM API call fails
    /// - Memory operations fail (if save_to_memory is true)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use agent_mem_core::client::AgentMemClient;
    /// # use agent_mem_traits::{LLMConfig, Result};
    /// # async fn example() -> Result<()> {
    /// use agent_mem_core::client::{AgentMemClientConfig, Messages};
    ///
    /// let mut config = AgentMemClientConfig::default();
    /// config.llm = Some(LLMConfig {
    ///     provider: "openai".to_string(),
    ///     model: "gpt-3.5-turbo".to_string(),
    ///     api_key: Some("your-api-key".to_string()),
    ///     ..Default::default()
    /// });
    ///
    /// let client = AgentMemClient::new(config);
    ///
    /// // Add some background information
    /// client.add(
    ///     Messages::Single("I am a software engineer".to_string()),
    ///     Some("alice".to_string()),
    ///     None, None, None, false, None, None,
    /// ).await?;
    ///
    /// // Chat with memory context
    /// let response = client.chat(
    ///     "What is my profession?".to_string(),
    ///     Some("alice".to_string()),
    ///     true, // Save conversation to memory
    /// ).await?;
    ///
    /// println!("Assistant: {}", response);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn chat(
        &self,
        message: String,
        user_id: Option<String>,
        save_to_memory: bool,
    ) -> Result<String> {
        // 检查 LLM 是否配置
        let llm_client = self.llm_client.as_ref().ok_or_else(|| {
            AgentMemError::config_error(
                "LLM not configured. Please provide LLM configuration in AgentMemClientConfig.",
            )
        })?;

        // 1. 构建系统消息（包含记忆上下文）
        let system_message = self
            .construct_system_message(message.clone(), user_id.clone(), None)
            .await?;

        // 2. 构建 LLM 消息
        let llm_messages = vec![
            LLMMessage {
                role: LLMMessageRole::System,
                content: system_message,
                timestamp: Some(chrono::Utc::now()),
            },
            LLMMessage {
                role: LLMMessageRole::User,
                content: message.clone(),
                timestamp: Some(chrono::Utc::now()),
            },
        ];

        // 3. 调用 LLM 生成回复
        let response = llm_client.generate(&llm_messages).await?;

        // 4. 可选：保存对话到记忆
        if save_to_memory {
            // 保存用户消息
            self.add(
                Messages::Single(message),
                user_id.clone(),
                None,
                None,
                None,
                false,
                Some(MemoryType::Episodic),
                None,
            )
            .await?;

            // 保存助手回复
            self.add(
                Messages::Single(format!("Assistant: {response}")),
                user_id,
                None,
                None,
                None,
                false,
                Some(MemoryType::Episodic),
                None,
            )
            .await?;
        }

        Ok(response)
    }

    // ==================== 辅助方法 ====================

    /// 简化的添加记忆方法（用于测试和简单场景）
    ///
    /// 这是一个便捷方法，简化了 `add` 方法的调用
    ///
    /// # Arguments
    ///
    /// * `content` - 记忆内容
    /// * `user_id` - 用户 ID（可选）
    /// * `run_id` - 运行 ID（可选）
    /// * `memory_type` - 记忆类型（可选，默认为 Episodic）
    ///
    /// # Returns
    ///
    /// 返回添加结果
    pub async fn add_simple(
        &self,
        content: String,
        user_id: Option<String>,
        run_id: Option<String>,
        memory_type: Option<MemoryType>,
    ) -> Result<AddResult> {
        self.add(
            Messages::Single(content),
            user_id,
            None, // agent_id
            run_id,
            None,  // metadata
            false, // infer
            memory_type,
            None, // prompt
        )
        .await
    }

    // ==================== 记忆可视化 API ====================

    /// 可视化用户的所有记忆
    ///
    /// 按记忆类型分组显示所有记忆，并提供统计摘要
    ///
    /// # Arguments
    ///
    /// * `user_id` - 用户 ID（可选）
    ///
    /// # Returns
    ///
    /// 返回 `MemoryVisualization` 结构，包含：
    /// - 用户信息
    /// - 记忆统计摘要
    /// - 按类型分组的记忆列表
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use agent_mem_core::client::AgentMemClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = AgentMemClient::default();
    ///
    ///     // 可视化特定用户的记忆
    ///     let viz = client.visualize_memories(Some("alice".to_string())).await?;
    ///     println!("Total memories: {}", viz.summary.total_count);
    ///     println!("Episodic: {}", viz.summary.episodic_count);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn visualize_memories(&self, user_id: Option<String>) -> Result<MemoryVisualization> {
        // 1. 获取用户信息
        let user_name = if let Some(ref uid) = user_id {
            // 尝试通过 ID 查找用户
            if let Some(user) = self.get_user_by_id(uid.clone()).await? {
                user.name
            } else {
                // 如果通过 ID 找不到，尝试通过名称查找
                self.get_user_by_name(uid.clone())
                    .await?
                    .map(|u| u.name)
                    .unwrap_or_else(|| "Unknown".to_string())
            }
        } else {
            "Default".to_string()
        };

        // 2. 获取所有记忆
        let all_memories = self.get_all(user_id.clone(), None, None, None).await?;

        // 3. 按类型分组
        let mut episodic = Vec::new();
        let mut semantic = Vec::new();
        let mut procedural = Vec::new();
        let mut core = Vec::new();
        let mut resource = Vec::new();
        let mut knowledge = Vec::new();
        let mut working = Vec::new();
        let mut contextual = Vec::new();

        for mem in all_memories {
            match mem.memory_type {
                MemoryType::Episodic => episodic.push(mem),
                MemoryType::Semantic => semantic.push(mem),
                MemoryType::Procedural => procedural.push(mem),
                MemoryType::Core => core.push(mem),
                MemoryType::Resource => resource.push(mem),
                MemoryType::Knowledge => knowledge.push(mem),
                MemoryType::Working => working.push(mem),
                MemoryType::Contextual => contextual.push(mem),
            }
        }

        // 4. 构建摘要
        let summary = MemorySummary {
            total_count: episodic.len()
                + semantic.len()
                + procedural.len()
                + core.len()
                + resource.len()
                + knowledge.len()
                + working.len()
                + contextual.len(),
            episodic_count: episodic.len(),
            semantic_count: semantic.len(),
            procedural_count: procedural.len(),
            core_count: core.len(),
            resource_count: resource.len(),
            knowledge_count: knowledge.len(),
            working_count: working.len(),
            contextual_count: contextual.len(),
        };

        Ok(MemoryVisualization {
            user_id: user_id.unwrap_or_else(|| "default".to_string()),
            user_name,
            summary,
            memories: MemoriesByType {
                episodic,
                semantic,
                procedural,
                core,
                resource,
                knowledge,
                working,
                contextual,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let _client = AgentMemClient::default();
        assert!(true); // Basic test to ensure client can be created
    }

    #[tokio::test]
    async fn test_add_memory_basic() {
        let client = AgentMemClient::default();

        let result = client
            .add(
                Messages::Single("I love pizza".to_string()),
                Some("user123".to_string()),
                Some("agent456".to_string()),
                None,
                None,
                true,
                Some(MemoryType::Episodic),
                None,
            )
            .await;

        assert!(result.is_ok());
        let add_result = result.unwrap();
        assert!(add_result.success);
        assert!(!add_result.id.is_empty());
    }

    #[tokio::test]
    async fn test_search_memory_basic() {
        let client = AgentMemClient::default();

        let result = client
            .search(
                "food preferences".to_string(),
                Some("user123".to_string()),
                None,
                None,
                10,
                None,
                Some(0.7),
            )
            .await;

        assert!(result.is_ok());
        let search_result = result.unwrap();
        assert_eq!(search_result.query, "food preferences");
        assert_eq!(search_result.total, 0); // Empty for now
    }

    #[tokio::test]
    async fn test_message_validation() {
        let valid_message = Messages::Single("Valid message".to_string());
        assert!(valid_message.validate().is_ok());

        let empty_message = Messages::Single("".to_string());
        assert!(empty_message.validate().is_err());

        let structured_message = Messages::Structured(Message::user("Hello".to_string()));
        assert!(structured_message.validate().is_ok());
    }

    #[tokio::test]
    async fn test_filter_builder() {
        let filters = FilterBuilder::new()
            .user_id("user123".to_string())
            .agent_id("agent456".to_string())
            .memory_type(MemoryType::Semantic)
            .build();

        assert_eq!(filters.get("user_id").unwrap().as_str().unwrap(), "user123");
        assert_eq!(
            filters.get("agent_id").unwrap().as_str().unwrap(),
            "agent456"
        );
        assert_eq!(
            filters.get("memory_type").unwrap().as_str().unwrap(),
            "semantic"
        );
    }

    #[tokio::test]
    async fn test_metadata_builder() {
        let metadata = MetadataBuilder::new()
            .user_id("user123".to_string())
            .agent_id("agent456".to_string())
            .custom("category".to_string(), "food")
            .unwrap()
            .build();

        assert_eq!(
            metadata.get("user_id").unwrap().as_str().unwrap(),
            "user123"
        );
        assert_eq!(
            metadata.get("agent_id").unwrap().as_str().unwrap(),
            "agent456"
        );
        assert_eq!(metadata.get("category").unwrap().as_str().unwrap(), "food");
    }

    #[tokio::test]
    async fn test_batch_add_memories() {
        let client = AgentMemClient::default();

        let requests = vec![
            AddRequest {
                messages: Messages::Single("I love pizza".to_string()),
                user_id: Some("user123".to_string()),
                agent_id: Some("agent456".to_string()),
                run_id: None,
                metadata: None,
                infer: true,
                memory_type: Some(MemoryType::Episodic),
                prompt: None,
            },
            AddRequest {
                messages: Messages::Single("I enjoy hiking".to_string()),
                user_id: Some("user123".to_string()),
                agent_id: Some("agent456".to_string()),
                run_id: None,
                metadata: None,
                infer: true,
                memory_type: Some(MemoryType::Episodic),
                prompt: None,
            },
        ];

        let results = client.add_batch(requests).await;
        assert!(results.is_ok());

        let add_results = results.unwrap();
        assert_eq!(add_results.len(), 2);
        assert!(add_results[0].success);
        assert!(add_results[1].success);
    }

    #[tokio::test]
    async fn test_batch_update_memories() {
        let client = AgentMemClient::default();

        // First, create some memories to update
        let add_requests = vec![
            AddRequest {
                messages: Messages::Single("Original content 1".to_string()),
                user_id: Some("user123".to_string()),
                agent_id: Some("agent456".to_string()),
                run_id: None,
                metadata: None,
                infer: true,
                memory_type: Some(MemoryType::Episodic),
                prompt: None,
            },
            AddRequest {
                messages: Messages::Single("Original content 2".to_string()),
                user_id: Some("user123".to_string()),
                agent_id: Some("agent456".to_string()),
                run_id: None,
                metadata: None,
                infer: true,
                memory_type: Some(MemoryType::Episodic),
                prompt: None,
            },
        ];

        let add_results = client.add_batch(add_requests).await.unwrap();
        let mem1_id = add_results[0].id.clone();
        let mem2_id = add_results[1].id.clone();

        // Now update them
        let requests = vec![
            UpdateRequest {
                memory_id: mem1_id,
                content: Some("Updated content 1".to_string()),
                metadata: None,
            },
            UpdateRequest {
                memory_id: mem2_id,
                content: Some("Updated content 2".to_string()),
                metadata: None,
            },
        ];

        let results = client.update_batch(requests).await;
        assert!(results.is_ok());

        let update_results = results.unwrap();
        assert_eq!(update_results.len(), 2);
        assert!(update_results[0].success);
        assert!(update_results[1].success);
    }

    #[tokio::test]
    async fn test_batch_delete_memories() {
        let client = AgentMemClient::default();

        // First, create some memories to delete
        let add_requests = vec![
            AddRequest {
                messages: Messages::Single("Memory to delete 1".to_string()),
                user_id: Some("user123".to_string()),
                agent_id: Some("agent456".to_string()),
                run_id: None,
                metadata: None,
                infer: true,
                memory_type: Some(MemoryType::Episodic),
                prompt: None,
            },
            AddRequest {
                messages: Messages::Single("Memory to delete 2".to_string()),
                user_id: Some("user123".to_string()),
                agent_id: Some("agent456".to_string()),
                run_id: None,
                metadata: None,
                infer: true,
                memory_type: Some(MemoryType::Episodic),
                prompt: None,
            },
            AddRequest {
                messages: Messages::Single("Memory to delete 3".to_string()),
                user_id: Some("user123".to_string()),
                agent_id: Some("agent456".to_string()),
                run_id: None,
                metadata: None,
                infer: true,
                memory_type: Some(MemoryType::Episodic),
                prompt: None,
            },
        ];

        let add_results = client.add_batch(add_requests).await.unwrap();
        let memory_ids = vec![
            add_results[0].id.clone(),
            add_results[1].id.clone(),
            add_results[2].id.clone(),
        ];

        // Now delete them
        let results = client.delete_batch(memory_ids).await;
        assert!(results.is_ok());

        let delete_results = results.unwrap();
        assert_eq!(delete_results.len(), 3);
        // All should succeed with the real implementation
        assert!(delete_results.iter().all(|&result| result));
    }
}
