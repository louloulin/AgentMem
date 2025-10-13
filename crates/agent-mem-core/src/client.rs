//! Mem0 Compatible Client Interface
//!
//! This module provides a Mem0-compatible API interface for AgentMem,
//! enabling seamless migration from Mem0 to AgentMem.

use crate::{MemoryEngine, MemoryEngineConfig};
use agent_mem_traits::{AgentMemError, Result};
use chrono::{DateTime, Utc};
use futures::future;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Semaphore;
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
}

impl ToString for MemoryType {
    fn to_string(&self) -> String {
        match self {
            MemoryType::Episodic => "episodic".to_string(),
            MemoryType::Semantic => "semantic".to_string(),
            MemoryType::Procedural => "procedural".to_string(),
            MemoryType::Working => "working".to_string(),
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
        let value = serde_json::to_value(value).map_err(|e| {
            AgentMemError::validation_error(&format!("Invalid filter value: {}", e))
        })?;
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
        let value = serde_json::to_value(value).map_err(|e| {
            AgentMemError::validation_error(&format!("Invalid metadata value: {}", e))
        })?;
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

/// Mem0 compatible client configuration
#[derive(Debug, Clone)]
pub struct AgentMemClientConfig {
    /// Memory engine configuration
    pub engine: MemoryEngineConfig,
    /// Performance configuration
    pub performance: PerformanceConfig,
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
}

impl AgentMemClient {
    /// Create new AgentMem client
    pub fn new(config: AgentMemClientConfig) -> Self {
        let engine = Arc::new(MemoryEngine::new(config.engine.clone()));
        let semaphore = Arc::new(Semaphore::new(config.performance.max_concurrent_operations));

        Self {
            engine,
            config,
            semaphore,
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
                AgentMemError::internal_error(&format!("Serialization error: {}", msg))
            }
            _ => AgentMemError::internal_error(&err.to_string()),
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
        let _permit = self.semaphore.acquire().await.map_err(|e| {
            AgentMemError::internal_error(&format!("Failed to acquire permit: {}", e))
        })?;

        // Search using engine
        let memories = self
            .engine
            .search_memories(&query, None, Some(limit))
            .await
            .map_err(Self::convert_error)?;

        // Convert to search results and apply filters
        let mut results: Vec<MemorySearchResult> = memories
            .into_iter()
            .filter(|memory| {
                // Apply user_id filter
                if let Some(ref uid) = user_id {
                    if memory.session.user_id.as_ref() != Some(uid) {
                        return false;
                    }
                }
                // Apply agent_id filter
                if let Some(ref aid) = agent_id {
                    if memory.session.agent_id.as_ref() != Some(aid) {
                        return false;
                    }
                }
                // Apply run_id filter (from session)
                if let Some(ref rid) = run_id {
                    if memory.session.run_id.as_ref() != Some(rid) {
                        return false;
                    }
                }
                true
            })
            .map(|memory| MemorySearchResult {
                id: memory.id.clone(),
                content: memory.content.clone(),
                score: memory.score.unwrap_or(0.0),
                memory_type: match memory.memory_type {
                    agent_mem_traits::MemoryType::Episodic => MemoryType::Episodic,
                    agent_mem_traits::MemoryType::Semantic => MemoryType::Semantic,
                    agent_mem_traits::MemoryType::Procedural => MemoryType::Procedural,
                    agent_mem_traits::MemoryType::Working => MemoryType::Working,
                    _ => MemoryType::Episodic,
                },
                metadata: memory.metadata,
                created_at: memory.created_at,
                updated_at: memory.updated_at.unwrap_or(memory.created_at),
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
        let _permit = self.semaphore.acquire().await.map_err(|e| {
            AgentMemError::internal_error(&format!("Failed to acquire permit: {}", e))
        })?;

        // Get from engine
        let memory_opt = self
            .engine
            .get_memory(&memory_id)
            .await
            .map_err(Self::convert_error)?;

        Ok(memory_opt.map(|memory| MemorySearchResult {
            id: memory.id.clone(),
            content: memory.content.clone(),
            score: memory.score.unwrap_or(0.0),
            memory_type: match memory.memory_type {
                agent_mem_traits::MemoryType::Episodic => MemoryType::Episodic,
                agent_mem_traits::MemoryType::Semantic => MemoryType::Semantic,
                agent_mem_traits::MemoryType::Procedural => MemoryType::Procedural,
                agent_mem_traits::MemoryType::Working => MemoryType::Working,
                _ => MemoryType::Episodic,
            },
            metadata: memory.metadata,
            created_at: memory.created_at,
            updated_at: memory.updated_at.unwrap_or(memory.created_at),
        }))
    }

    /// Update memory
    pub async fn update(&self, request: UpdateRequest) -> Result<UpdateResult> {
        if request.memory_id.trim().is_empty() {
            return Err(AgentMemError::validation_error("Memory ID cannot be empty"));
        }

        // Acquire semaphore permit
        let _permit = self.semaphore.acquire().await.map_err(|e| {
            AgentMemError::internal_error(&format!("Failed to acquire permit: {}", e))
        })?;

        // Get existing memory
        let mut memory = self
            .engine
            .get_memory(&request.memory_id)
            .await
            .map_err(Self::convert_error)?
            .ok_or_else(|| {
                AgentMemError::not_found(&format!("Memory {} not found", request.memory_id))
            })?;

        // Update content if provided
        if let Some(content) = request.content {
            memory.content = content;
        }

        // Update metadata if provided
        if let Some(metadata) = request.metadata {
            memory.metadata = metadata;
        }

        // Update timestamp
        memory.updated_at = Some(chrono::Utc::now());

        // Update in engine
        let updated_memory = self
            .engine
            .update_memory(memory)
            .await
            .map_err(Self::convert_error)?;

        Ok(UpdateResult {
            id: updated_memory.id,
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
        let _permit = self.semaphore.acquire().await.map_err(|e| {
            AgentMemError::internal_error(&format!("Failed to acquire permit: {}", e))
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
        let _permit = self.semaphore.acquire().await.map_err(|e| {
            AgentMemError::internal_error(&format!("Failed to acquire permit: {}", e))
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
                    if memory.session.user_id.as_ref() != Some(uid) {
                        return false;
                    }
                }
                // Apply agent_id filter
                if let Some(ref aid) = agent_id {
                    if memory.session.agent_id.as_ref() != Some(aid) {
                        return false;
                    }
                }
                // Apply run_id filter (from session)
                if let Some(ref rid) = run_id {
                    if memory.session.run_id.as_ref() != Some(rid) {
                        return false;
                    }
                }
                true
            })
            .map(|memory| MemorySearchResult {
                id: memory.id.clone(),
                content: memory.content.clone(),
                score: memory.score.unwrap_or(0.0),
                memory_type: match memory.memory_type {
                    agent_mem_traits::MemoryType::Episodic => MemoryType::Episodic,
                    agent_mem_traits::MemoryType::Semantic => MemoryType::Semantic,
                    agent_mem_traits::MemoryType::Procedural => MemoryType::Procedural,
                    agent_mem_traits::MemoryType::Working => MemoryType::Working,
                    _ => MemoryType::Episodic,
                },
                metadata: memory.metadata,
                created_at: memory.created_at,
                updated_at: memory.updated_at.unwrap_or(memory.created_at),
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
                AgentMemError::internal_error(&format!("Failed to acquire permit: {}", e))
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
                Err(e) => {
                    return Err(AgentMemError::internal_error(&format!(
                        "Task failed: {}",
                        e
                    )))
                }
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
                AgentMemError::internal_error(&format!("Failed to acquire permit: {}", e))
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
                Err(e) => {
                    return Err(AgentMemError::internal_error(&format!(
                        "Task failed: {}",
                        e
                    )))
                }
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
                AgentMemError::internal_error(&format!("Failed to acquire permit: {}", e))
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
        let _permit = self.semaphore.acquire().await.map_err(|e| {
            AgentMemError::internal_error(&format!("Failed to acquire permit: {}", e))
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
        let id = self
            .engine
            .add_memory(memory_item)
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
        if let Some(existing_user) = self.get_user_by_name(user_name.clone()).await? {
            // 返回已存在的用户
            return Ok(existing_user);
        }

        // 创建新用户
        let user = User {
            id: Uuid::new_v4().to_string(),
            name: user_name,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // TODO: 将用户保存到数据库
        // 目前先返回内存中的用户对象

        Ok(user)
    }

    /// 列出所有用户
    pub async fn list_users(&self) -> Result<Vec<User>> {
        // TODO: 从数据库查询所有用户
        // 目前返回空列表
        Ok(Vec::new())
    }

    /// 按名称查询用户
    pub async fn get_user_by_name(&self, user_name: String) -> Result<Option<User>> {
        // TODO: 从数据库查询用户
        // 目前返回 None
        Ok(None)
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

        let requests = vec![
            UpdateRequest {
                memory_id: "mem1".to_string(),
                content: Some("Updated content 1".to_string()),
                metadata: None,
            },
            UpdateRequest {
                memory_id: "mem2".to_string(),
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

        let memory_ids = vec!["mem1".to_string(), "mem2".to_string(), "mem3".to_string()];

        let results = client.delete_batch(memory_ids).await;
        assert!(results.is_ok());

        let delete_results = results.unwrap();
        assert_eq!(delete_results.len(), 3);
        // All should succeed with the real implementation
        assert!(delete_results.iter().all(|&result| result));
    }
}
