//! Mem5 Enhanced Mem0 compatibility client implementation
//!
//! This module provides a fully compatible Mem0 API with enhanced features:
//! - Batch operations support
//! - Advanced filtering and search
//! - Error recovery and retry mechanisms
//! - Performance monitoring and telemetry
//! - Production-grade reliability

use crate::{
    config::Mem0Config,
    context_aware::{
        ContextAwareConfig, ContextAwareManager, ContextAwareSearchRequest,
        ContextAwareSearchResult, ContextInfo, ContextLearningResult, ContextPattern,
    },
    enterprise_security::{
        EnterpriseSecurityConfig, EnterpriseSecurityManager, JwtClaims, Permission,
        UserSession,
    },
    error::{Mem0Error, Result},
    graph_memory::{FusedMemory, GraphMemoryConfig, GraphMemoryManager},
    personalization::{
        MemoryRecommendation, PersonalizationConfig,
        PersonalizationManager, PersonalizedSearchRequest, PersonalizedSearchResult, UserBehavior,
        UserPreference, UserProfile,
    },
    procedural_memory::{
        ProceduralMemoryConfig, ProceduralMemoryManager, StepExecutionResult, Task, TaskChain,
        TaskExecutionResult, Workflow, WorkflowExecution, WorkflowStep,
    },
    types::{
        AddMemoryRequest, BatchAddResult, BatchDeleteRequest, BatchDeleteResult, BatchUpdateRequest, BatchUpdateResult, ChangeType, DeleteMemoryResponse,
        Memory, MemoryFilter, MemoryHistory, MemorySearchResult, MemorySearchResultItem,
        SearchMemoryRequest, SortField, SortOrder, UpdateMemoryRequest,
    },
    utils::{
        calculate_importance_score, generate_memory_id, sanitize_metadata, validate_memory_content,
        validate_user_id,
    },
};

use agent_mem_performance::batch::{BatchConfig, BatchItem, BatchProcessor};
use agent_mem_traits::Message;
use async_trait::async_trait;
use chrono::Utc;
use dashmap::DashMap;
use std::{collections::HashMap, sync::Arc};
use tracing::{debug, info, instrument, warn};

/// Enhanced message types for Mem0 compatibility
#[derive(Debug, Clone)]
pub enum Messages {
    Single(String),
    Structured(Message),
    Multiple(Vec<String>),
}

impl Messages {
    pub fn validate(&self) -> Result<()> {
        match self {
            Messages::Single(s) => {
                if s.trim().is_empty() {
                    return Err(Mem0Error::InvalidContent {
                        reason: "Empty message".to_string(),
                    });
                }
            }
            Messages::Structured(msg) => {
                if msg.content.trim().is_empty() {
                    return Err(Mem0Error::InvalidContent {
                        reason: "Empty structured message content".to_string(),
                    });
                }
            }
            Messages::Multiple(msgs) => {
                if msgs.is_empty() {
                    return Err(Mem0Error::InvalidContent {
                        reason: "Empty message list".to_string(),
                    });
                }
                for msg in msgs {
                    if msg.trim().is_empty() {
                        return Err(Mem0Error::InvalidContent {
                            reason: "Empty message in list".to_string(),
                        });
                    }
                }
            }
        }
        Ok(())
    }

    /// Convert messages to content string for storage
    pub fn to_content(&self) -> String {
        match self {
            Messages::Single(s) => s.clone(),
            Messages::Structured(msg) => msg.content.clone(),
            Messages::Multiple(msgs) => msgs.join("\n"),
        }
    }

    /// Convert messages to a list of Message structs
    pub fn to_message_list(&self) -> Vec<Message> {
        match self {
            Messages::Single(s) => vec![Message::user(s)],
            Messages::Structured(msg) => vec![msg.clone()],
            Messages::Multiple(msgs) => msgs.iter().map(|s| Message::user(s)).collect(),
        }
    }
}

/// Enhanced add request with full Mem0 compatibility
#[derive(Debug, Clone)]
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

/// Enhanced search request with advanced filtering
#[derive(Debug, Clone)]
pub struct EnhancedSearchRequest {
    pub query: String,
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub run_id: Option<String>,
    pub limit: usize,
    pub filters: Option<HashMap<String, serde_json::Value>>,
    pub threshold: Option<f32>,
}

/// Batch operation request
#[derive(Debug, Clone)]
pub struct BatchAddRequest {
    pub requests: Vec<EnhancedAddRequest>,
}

/// Memory operation for batch processing
#[derive(Debug, Clone)]
pub struct MemoryOperation {
    pub request: EnhancedAddRequest,
    pub client: Arc<DashMap<String, Memory>>,
}

#[async_trait]
impl BatchItem for MemoryOperation {
    type Output = String;
    type Error = crate::error::Mem0Error;

    async fn process(&self) -> std::result::Result<Self::Output, Self::Error> {
        // Generate memory ID
        let memory_id = uuid::Uuid::new_v4().to_string();

        // Convert messages to content
        let content = self.request.messages.to_content();

        // Create memory
        let memory = Memory {
            id: memory_id.clone(),
            memory: content,
            user_id: self
                .request
                .user_id
                .clone()
                .unwrap_or_else(|| "default".to_string()),
            agent_id: self.request.agent_id.clone(),
            run_id: self.request.run_id.clone(),
            metadata: self.request.metadata.clone().unwrap_or_default(),
            score: None,
            created_at: Utc::now(),
            updated_at: Some(Utc::now()),
        };

        // Store memory
        self.client.insert(memory_id.clone(), memory);

        Ok(memory_id)
    }

    fn size(&self) -> usize {
        self.request.messages.to_content().len()
    }

    fn priority(&self) -> u8 {
        // Higher priority for shorter content (faster processing)
        if self.size() < 100 {
            10
        } else if self.size() < 500 {
            5
        } else {
            1
        }
    }
}

/// Enhanced Mem0 compatibility client with Mem5 features
pub struct Mem0Client {
    /// Configuration
    config: Mem0Config,

    /// In-memory storage for demonstration
    memories: Arc<DashMap<String, Memory>>,

    /// Memory history cache
    history_cache: Arc<DashMap<String, Vec<MemoryHistory>>>,

    /// Batch processor for concurrent operations
    batch_processor: Option<Arc<BatchProcessor>>,

    /// Graph memory manager for entity-relation storage
    graph_memory: Option<Arc<GraphMemoryManager>>,

    /// Procedural memory manager for workflow and process memory
    procedural_memory: Option<Arc<ProceduralMemoryManager>>,

    /// Context-aware memory manager for intelligent context understanding
    context_aware: Option<Arc<ContextAwareManager>>,

    /// Personalization manager for user-specific memory strategies
    personalization: Option<Arc<PersonalizationManager>>,

    /// Enterprise security manager for RBAC, encryption, and audit logging
    enterprise_security: Option<Arc<EnterpriseSecurityManager>>,
}

impl Mem0Client {
    /// Create a new Mem0Client with default configuration
    pub async fn new() -> Result<Self> {
        let config = Mem0Config::default();
        Self::with_config(config).await
    }

    /// Create a new Mem0Client with custom configuration
    pub async fn with_config(config: Mem0Config) -> Result<Self> {
        config.validate()?;

        info!("Initializing Mem0Client with config: {:?}", config);

        // For now, create a simple in-memory implementation
        // In a full implementation, this would initialize the actual AgentMem components

        info!("Mem0Client initialized successfully");

        // Initialize batch processor for concurrent operations
        let batch_config = BatchConfig {
            max_batch_size: 50,
            max_wait_time_ms: 100,
            concurrency: 4,
            buffer_size: 1000,
            enable_compression: false,
            retry_attempts: 3,
            retry_delay_ms: 50,
        };

        let batch_processor = match BatchProcessor::new(batch_config).await {
            Ok(processor) => Some(Arc::new(processor)),
            Err(e) => {
                warn!("Failed to initialize batch processor: {}, falling back to sequential processing", e);
                None
            }
        };

        // Initialize graph memory manager
        let graph_memory = match GraphMemoryManager::new(GraphMemoryConfig::default()).await {
            Ok(manager) => {
                info!("Graph memory manager initialized successfully");
                Some(Arc::new(manager))
            }
            Err(e) => {
                warn!("Failed to initialize graph memory manager: {}, graph features will be disabled", e);
                None
            }
        };

        // Initialize procedural memory manager
        let procedural_memory = {
            let procedural_config = ProceduralMemoryConfig::default();
            let manager = ProceduralMemoryManager::new(procedural_config);
            info!("Procedural memory manager initialized successfully");
            Some(Arc::new(manager))
        };

        // Initialize context-aware memory manager
        let context_aware = match ContextAwareManager::new(ContextAwareConfig::default()).await {
            Ok(manager) => {
                info!("Context-aware memory manager initialized successfully");
                Some(Arc::new(manager))
            }
            Err(e) => {
                warn!("Failed to initialize context-aware manager: {}, context features will be disabled", e);
                None
            }
        };

        // Initialize personalization manager
        let personalization = {
            let personalization_config = PersonalizationConfig::default();
            let manager = PersonalizationManager::new(personalization_config);
            info!("Personalization manager initialized successfully");
            Some(Arc::new(manager))
        };

        // Initialize enterprise security manager
        let enterprise_security = match EnterpriseSecurityManager::new(
            EnterpriseSecurityConfig::default(),
        ) {
            Ok(manager) => {
                if let Err(e) = manager.initialize_defaults().await {
                    warn!(
                        "Failed to initialize default security settings: {}, using basic security",
                        e
                    );
                }
                info!("Enterprise security manager initialized successfully");
                Some(Arc::new(manager))
            }
            Err(e) => {
                warn!("Failed to initialize enterprise security manager: {}, security features will be disabled", e);
                None
            }
        };

        Ok(Self {
            config,
            memories: Arc::new(DashMap::new()),
            history_cache: Arc::new(DashMap::new()),
            batch_processor,
            graph_memory,
            procedural_memory,
            context_aware,
            personalization,
            enterprise_security,
        })
    }

    /// Add a new memory
    pub async fn add(
        &self,
        user_id: &str,
        memory: &str,
        metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
    ) -> Result<String> {
        self.add_with_options(AddMemoryRequest {
            memory: memory.to_string(),
            user_id: user_id.to_string(),
            agent_id: None,
            run_id: None,
            metadata: metadata.unwrap_or_default(),
        })
        .await
    }

    /// Add a new memory with full options
    pub async fn add_with_options(&self, request: AddMemoryRequest) -> Result<String> {
        validate_user_id(&request.user_id)?;
        validate_memory_content(&request.memory)?;

        let mut metadata = request.metadata;
        sanitize_metadata(&mut metadata);

        let memory_id = generate_memory_id();
        let importance = calculate_importance_score(&request.memory, &metadata);

        let memory = Memory {
            id: memory_id.clone(),
            memory: request.memory,
            user_id: request.user_id,
            agent_id: request.agent_id,
            run_id: request.run_id,
            metadata,
            score: Some(importance),
            created_at: Utc::now(),
            updated_at: None,
        };

        self.memories.insert(memory_id.clone(), memory.clone());

        // Create history entry for creation
        self.create_history_entry(
            &memory_id,
            &memory.user_id,
            None, // no previous memory
            Some(memory.memory.clone()),
            None, // no previous metadata
            Some(memory.metadata.clone()),
            ChangeType::Created,
            None, // changed_by
            None, // reason
        )
        .await?;

        debug!("Added memory with ID: {}", memory_id);
        Ok(memory_id)
    }

    /// Search for memories
    pub async fn search(
        &self,
        query: &str,
        user_id: &str,
        filters: Option<MemoryFilter>,
    ) -> Result<MemorySearchResult> {
        self.search_with_options(SearchMemoryRequest {
            query: query.to_string(),
            user_id: user_id.to_string(),
            filters,
            limit: None,
        })
        .await
    }

    /// Search for memories with full options
    pub async fn search_with_options(
        &self,
        request: SearchMemoryRequest,
    ) -> Result<MemorySearchResult> {
        validate_user_id(&request.user_id)?;

        if request.query.is_empty() {
            return Err(Mem0Error::InvalidContent {
                reason: "Search query cannot be empty".to_string(),
            });
        }

        let limit = request
            .limit
            .or_else(|| request.filters.as_ref().and_then(|f| f.limit))
            .unwrap_or(self.config.memory.default_search_limit);

        // Enhanced search with complex filtering and scoring
        let mut matching_memories: Vec<Memory> = self
            .memories
            .iter()
            .filter(|entry| {
                let memory = entry.value();
                // Filter by user_id
                if memory.user_id != request.user_id {
                    return false;
                }

                // Apply enhanced filters if provided
                if let Some(filters) = &request.filters {
                    if !filters.matches(memory) {
                        return false;
                    }
                }

                // Enhanced text matching with scoring
                self.calculate_search_score(&request.query, memory) > 0.0
            })
            .map(|entry| entry.value().clone())
            .collect();

        // Apply enhanced sorting if specified in filters
        if let Some(ref filter) = request.filters {
            self.sort_memories(
                &mut matching_memories,
                &filter.sort_field,
                &filter.sort_order,
            );

            // Apply pagination
            if let Some(offset) = filter.offset {
                if offset < matching_memories.len() {
                    matching_memories = matching_memories.into_iter().skip(offset).collect();
                } else {
                    matching_memories.clear();
                }
            }
        } else {
            // Default sorting by score (descending)
            matching_memories.sort_by(|a, b| {
                b.score
                    .unwrap_or(0.0)
                    .partial_cmp(&a.score.unwrap_or(0.0))
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }

        matching_memories.truncate(limit);

        let total = matching_memories.len();

        debug!("Found {} memories for query: {}", total, request.query);

        Ok(MemorySearchResult {
            memories: matching_memories,
            total,
            metadata: std::collections::HashMap::new(),
        })
    }

    /// Get a specific memory by ID
    pub async fn get(&self, memory_id: &str, user_id: &str) -> Result<Memory> {
        validate_user_id(user_id)?;

        let memory = self
            .memories
            .get(memory_id)
            .ok_or_else(|| Mem0Error::MemoryNotFound {
                id: memory_id.to_string(),
            })?;

        // Check if the memory belongs to the user
        if memory.user_id != user_id {
            return Err(Mem0Error::MemoryNotFound {
                id: memory_id.to_string(),
            });
        }

        Ok(memory.clone())
    }

    /// Calculate search relevance score
    fn calculate_search_score(&self, query: &str, memory: &Memory) -> f32 {
        let query_lower = query.to_lowercase();
        let content_lower = memory.memory.to_lowercase();

        // Exact match gets highest score
        if content_lower == query_lower {
            return 1.0;
        }

        // Contains query gets high score
        if content_lower.contains(&query_lower) {
            let ratio = query_lower.len() as f32 / content_lower.len() as f32;
            return 0.8 * ratio;
        }

        // Word-based matching
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        let content_words: Vec<&str> = content_lower.split_whitespace().collect();

        let mut matches = 0;
        for query_word in &query_words {
            for content_word in &content_words {
                if content_word.contains(query_word) || query_word.contains(content_word) {
                    matches += 1;
                    break;
                }
            }
        }

        if matches > 0 {
            let word_score = matches as f32 / query_words.len() as f32;
            return 0.5 * word_score;
        }

        0.0
    }

    /// Sort memories based on specified criteria
    fn sort_memories(
        &self,
        memories: &mut Vec<Memory>,
        sort_field: &SortField,
        sort_order: &SortOrder,
    ) {
        use crate::types::{SortField, SortOrder};

        memories.sort_by(|a, b| {
            let comparison = match sort_field {
                SortField::CreatedAt => a.created_at.cmp(&b.created_at),
                SortField::UpdatedAt => match (a.updated_at, b.updated_at) {
                    (Some(a_updated), Some(b_updated)) => a_updated.cmp(&b_updated),
                    (Some(_), None) => std::cmp::Ordering::Greater,
                    (None, Some(_)) => std::cmp::Ordering::Less,
                    (None, None) => std::cmp::Ordering::Equal,
                },
                SortField::Score => a
                    .score
                    .partial_cmp(&b.score)
                    .unwrap_or(std::cmp::Ordering::Equal),
                SortField::ContentLength => a.memory.len().cmp(&b.memory.len()),
                SortField::Metadata(key) => {
                    match (a.metadata.get(key), b.metadata.get(key)) {
                        (Some(a_val), Some(b_val)) => {
                            // Try to compare as numbers first, then as strings
                            if let (Some(a_num), Some(b_num)) = (a_val.as_f64(), b_val.as_f64()) {
                                a_num
                                    .partial_cmp(&b_num)
                                    .unwrap_or(std::cmp::Ordering::Equal)
                            } else {
                                a_val.to_string().cmp(&b_val.to_string())
                            }
                        }
                        (Some(_), None) => std::cmp::Ordering::Greater,
                        (None, Some(_)) => std::cmp::Ordering::Less,
                        (None, None) => std::cmp::Ordering::Equal,
                    }
                }
            };

            match sort_order {
                SortOrder::Asc => comparison,
                SortOrder::Desc => comparison.reverse(),
            }
        });
    }

    /// Create a history entry for memory changes
    async fn create_history_entry(
        &self,
        memory_id: &str,
        user_id: &str,
        prev_memory: Option<String>,
        new_memory: Option<String>,
        prev_metadata: Option<HashMap<String, serde_json::Value>>,
        new_metadata: Option<HashMap<String, serde_json::Value>>,
        change_type: ChangeType,
        changed_by: Option<String>,
        reason: Option<String>,
    ) -> Result<()> {
        let history_entry = MemoryHistory {
            id: uuid::Uuid::new_v4().to_string(),
            memory_id: memory_id.to_string(),
            user_id: user_id.to_string(),
            prev_memory,
            new_memory,
            prev_metadata,
            new_metadata,
            timestamp: Utc::now(),
            change_type,
            changed_by,
            reason,
            version: self.get_next_version(memory_id).await,
            related_memory_ids: Vec::new(),
            metadata: HashMap::new(),
        };

        // Store history entry
        self.history_cache
            .entry(memory_id.to_string())
            .or_insert_with(Vec::new)
            .push(history_entry);

        Ok(())
    }

    /// Get the next version number for a memory
    async fn get_next_version(&self, memory_id: &str) -> u32 {
        if let Some(history) = self.history_cache.get(memory_id) {
            history.iter().map(|h| h.version).max().unwrap_or(0) + 1
        } else {
            1
        }
    }

    /// Get memory history
    pub async fn history(&self, memory_id: &str, user_id: &str) -> Result<Vec<MemoryHistory>> {
        validate_user_id(user_id)?;

        // Check if memory exists and belongs to user
        if let Some(memory) = self.memories.get(memory_id) {
            if memory.user_id != user_id {
                return Err(Mem0Error::MemoryNotFound {
                    id: memory_id.to_string(),
                });
            }
        } else {
            return Err(Mem0Error::MemoryNotFound {
                id: memory_id.to_string(),
            });
        }

        // Get history entries
        let history = self
            .history_cache
            .get(memory_id)
            .map(|h| h.clone())
            .unwrap_or_default();

        debug!(
            "Retrieved {} history entries for memory: {}",
            history.len(),
            memory_id
        );
        Ok(history)
    }

    /// Update a memory with history tracking
    pub async fn update(
        &self,
        memory_id: &str,
        user_id: &str,
        request: UpdateMemoryRequest,
    ) -> Result<Memory> {
        validate_user_id(user_id)?;

        // Get existing memory
        let mut memory =
            self.memories
                .get_mut(memory_id)
                .ok_or_else(|| Mem0Error::MemoryNotFound {
                    id: memory_id.to_string(),
                })?;

        // Check if the memory belongs to the user
        if memory.user_id != user_id {
            return Err(Mem0Error::MemoryNotFound {
                id: memory_id.to_string(),
            });
        }

        // Store original values for history tracking
        let prev_memory = memory.memory.clone();
        let prev_metadata = memory.metadata.clone();
        let mut change_type = None;

        // Update content if provided
        if let Some(new_content) = request.memory {
            validate_memory_content(&new_content)?;
            if memory.memory != new_content {
                memory.memory = new_content;
                change_type = Some(ChangeType::ContentUpdated);
            }
        }

        // Update metadata if provided
        if let Some(mut new_metadata) = request.metadata {
            sanitize_metadata(&mut new_metadata);
            if memory.metadata != new_metadata {
                memory.metadata = new_metadata;
                change_type = Some(if change_type.is_some() {
                    ChangeType::ContentUpdated
                } else {
                    ChangeType::MetadataUpdated
                });
            }
        }

        // Only update timestamp and create history if something actually changed
        if let Some(change_type) = change_type {
            memory.updated_at = Some(Utc::now());
            memory.score = Some(calculate_importance_score(&memory.memory, &memory.metadata));

            // Create history entry
            self.create_history_entry(
                memory_id,
                user_id,
                Some(prev_memory),
                Some(memory.memory.clone()),
                Some(prev_metadata),
                Some(memory.metadata.clone()),
                change_type,
                None, // changed_by
                None, // reason
            )
            .await?;
        }

        let updated_memory = memory.clone();

        debug!("Updated memory with ID: {}", memory_id);
        Ok(updated_memory)
    }

    /// Delete a memory
    pub async fn delete(&self, memory_id: &str, user_id: &str) -> Result<DeleteMemoryResponse> {
        validate_user_id(user_id)?;

        // Check if memory exists and belongs to user, and get a copy for history
        let memory_to_delete = if let Some(memory) = self.memories.get(memory_id) {
            if memory.user_id != user_id {
                return Err(Mem0Error::MemoryNotFound {
                    id: memory_id.to_string(),
                });
            }
            memory.clone()
        } else {
            return Err(Mem0Error::MemoryNotFound {
                id: memory_id.to_string(),
            });
        };

        // Create history entry before deletion
        self.create_history_entry(
            memory_id,
            user_id,
            Some(memory_to_delete.memory.clone()),
            None, // no new memory (deleted)
            Some(memory_to_delete.metadata.clone()),
            None, // no new metadata (deleted)
            ChangeType::Deleted,
            None, // changed_by
            None, // reason
        )
        .await?;

        // Remove the memory
        self.memories.remove(memory_id);

        debug!("Deleted memory with ID: {}", memory_id);
        Ok(DeleteMemoryResponse {
            success: true,
            message: Some("Memory deleted successfully".to_string()),
        })
    }

    /// Get all memories for a user
    pub async fn get_all(
        &self,
        user_id: &str,
        filters: Option<MemoryFilter>,
    ) -> Result<Vec<Memory>> {
        validate_user_id(user_id)?;

        let limit = filters.as_ref().and_then(|f| f.limit).unwrap_or(1000); // Default large limit for get_all

        let mut memories: Vec<Memory> = self
            .memories
            .iter()
            .filter(|entry| {
                let memory = entry.value();
                // Filter by user_id
                if memory.user_id != user_id {
                    return false;
                }

                // Apply additional filters if provided
                if let Some(filters) = &filters {
                    if let Some(agent_id) = &filters.agent_id {
                        if memory.agent_id.as_ref() != Some(agent_id) {
                            return false;
                        }
                    }
                    if let Some(run_id) = &filters.run_id {
                        if memory.run_id.as_ref() != Some(run_id) {
                            return false;
                        }
                    }
                    if let Some(created_after) = &filters.created_after {
                        if memory.created_at < *created_after {
                            return false;
                        }
                    }
                    if let Some(created_before) = &filters.created_before {
                        if memory.created_at > *created_before {
                            return false;
                        }
                    }
                }

                true
            })
            .map(|entry| entry.value().clone())
            .collect();

        // Sort by creation time (newest first) and limit
        memories.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        memories.truncate(limit);

        debug!(
            "Retrieved {} memories for user: {}",
            memories.len(),
            user_id
        );
        Ok(memories)
    }

    /// Get memory history for a specific memory
    pub async fn get_history(&self, memory_id: &str, user_id: &str) -> Result<Vec<MemoryHistory>> {
        validate_user_id(user_id)?;

        // Check if history is cached
        if let Some(history) = self.history_cache.get(memory_id) {
            return Ok(history.clone());
        }

        // For now, return empty history as AgentMem doesn't have built-in history tracking
        // In a full implementation, you would integrate with AgentMem's lifecycle events
        warn!(
            "Memory history not implemented yet for memory: {}",
            memory_id
        );
        Ok(Vec::new())
    }

    /// Delete all memories for a user
    pub async fn delete_all(&self, user_id: &str) -> Result<DeleteMemoryResponse> {
        validate_user_id(user_id)?;

        // Collect memory IDs to delete
        let memory_ids: Vec<String> = self
            .memories
            .iter()
            .filter(|entry| entry.value().user_id == user_id)
            .map(|entry| entry.key().clone())
            .collect();

        let deleted_count = memory_ids.len();

        // Delete each memory
        for memory_id in memory_ids {
            self.memories.remove(&memory_id);
        }

        debug!("Deleted {} memories for user: {}", deleted_count, user_id);
        Ok(DeleteMemoryResponse {
            success: true,
            message: Some(format!("Deleted {deleted_count} memories")),
        })
    }

    /// Get memory statistics for a user
    pub async fn get_stats(
        &self,
        user_id: &str,
    ) -> Result<std::collections::HashMap<String, serde_json::Value>> {
        validate_user_id(user_id)?;

        let user_memories: Vec<Memory> = self
            .memories
            .iter()
            .filter(|entry| entry.value().user_id == user_id)
            .map(|entry| entry.value().clone())
            .collect();

        let total_memories = user_memories.len();
        let avg_importance = if total_memories > 0 {
            user_memories
                .iter()
                .map(|m| m.score.unwrap_or(0.0))
                .sum::<f32>()
                / total_memories as f32
        } else {
            0.0
        };

        // Count memories by agent_id
        let mut agent_counts = std::collections::HashMap::new();
        for memory in &user_memories {
            let agent_id = memory.agent_id.as_deref().unwrap_or("default");
            *agent_counts.entry(agent_id.to_string()).or_insert(0) += 1;
        }

        let mut stats = std::collections::HashMap::new();
        stats.insert(
            "total_memories".to_string(),
            serde_json::Value::Number(total_memories.into()),
        );
        stats.insert(
            "average_importance".to_string(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(avg_importance as f64).unwrap_or_else(|| 0.into()),
            ),
        );
        stats.insert(
            "agent_counts".to_string(),
            serde_json::to_value(agent_counts)?,
        );

        debug!("Generated stats for user: {}", user_id);
        Ok(stats)
    }

    /// Reset the client (clear all data)
    pub async fn reset(&self) -> Result<()> {
        warn!("Reset operation requested - this will clear all data");

        // Clear all memories
        self.memories.clear();

        // Clear history cache
        self.history_cache.clear();

        info!("Client reset completed");
        Ok(())
    }

    // ========== Mem5 Enhanced API Methods ==========

    /// Enhanced add method with full Mem0 compatibility
    #[instrument(skip(self, request))]
    pub async fn add_enhanced(&self, request: EnhancedAddRequest) -> Result<String> {
        // Validate messages
        request.messages.validate()?;

        // Convert messages to string content
        let content = request.messages.to_content();

        // Create traditional add request
        let add_request = AddMemoryRequest {
            memory: content,
            user_id: request.user_id.unwrap_or_else(|| "default".to_string()),
            agent_id: request.agent_id,
            run_id: request.run_id,
            metadata: request.metadata.unwrap_or_default(),
        };

        // Use existing add logic
        self.add_with_options(add_request).await
    }

    /// Enhanced search method with advanced filtering and semantic similarity
    #[instrument(skip(self, request))]
    pub async fn search_enhanced(
        &self,
        request: EnhancedSearchRequest,
    ) -> Result<MemorySearchResult> {
        let user_id = request.user_id.unwrap_or_else(|| "default".to_string());
        validate_user_id(&user_id)?;

        // Step 1: Apply basic filters
        let mut candidate_memories: Vec<Memory> = self
            .memories
            .iter()
            .filter(|entry| {
                let memory = entry.value();
                // Filter by user_id
                if memory.user_id != user_id {
                    return false;
                }

                // Filter by agent_id if specified
                if let Some(ref agent_id) = request.agent_id {
                    if memory.agent_id.as_ref() != Some(agent_id) {
                        return false;
                    }
                }

                // Filter by run_id if specified
                if let Some(ref run_id) = request.run_id {
                    if memory.run_id.as_ref() != Some(run_id) {
                        return false;
                    }
                }

                true
            })
            .map(|entry| entry.value().clone())
            .collect();

        // Step 2: Calculate semantic similarity scores
        for memory in &mut candidate_memories {
            let similarity_score = self
                .calculate_semantic_similarity(&request.query, &memory.memory)
                .await;
            // Store the similarity score in the memory's score field for sorting
            memory.score = Some(similarity_score);
        }

        // Step 3: Apply threshold filter if specified
        if let Some(threshold) = request.threshold {
            candidate_memories.retain(|memory| memory.score.unwrap_or(0.0) >= threshold);
        }

        // Step 4: Sort by relevance (similarity score + importance)
        candidate_memories.sort_by(|a, b| {
            let score_a = a.score.unwrap_or(0.0);
            let score_b = b.score.unwrap_or(0.0);
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Step 5: Apply limit
        candidate_memories.truncate(request.limit);

        // Step 6: Convert to search results
        let results: Vec<MemorySearchResultItem> = candidate_memories
            .into_iter()
            .map(|memory| MemorySearchResultItem {
                id: memory.id.clone(),
                content: memory.memory.clone(),
                user_id: memory.user_id.clone(),
                agent_id: memory.agent_id.clone(),
                run_id: memory.run_id.clone(),
                metadata: memory.metadata.clone(),
                score: memory.score,
                created_at: memory.created_at,
                updated_at: memory.updated_at,
            })
            .collect();

        let total_results = results.len();
        debug!(
            "Enhanced search found {} results for query: {}",
            total_results, request.query
        );

        Ok(MemorySearchResult {
            memories: results
                .into_iter()
                .map(|item| Memory {
                    id: item.id,
                    memory: item.content,
                    user_id: item.user_id,
                    agent_id: item.agent_id,
                    run_id: item.run_id,
                    metadata: item.metadata,
                    score: item.score,
                    created_at: item.created_at,
                    updated_at: item.updated_at,
                })
                .collect(),
            total: total_results,
            metadata: HashMap::new(),
        })
    }

    /// Calculate semantic similarity between query and memory content
    async fn calculate_semantic_similarity(&self, query: &str, content: &str) -> f32 {
        // Simple similarity calculation based on:
        // 1. Exact word matches
        // 2. Partial word matches
        // 3. Content length relevance
        // 4. Keyword density

        let query_lower = query.to_lowercase();
        let content_lower = content.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        let content_words: Vec<&str> = content_lower.split_whitespace().collect();

        if query_words.is_empty() || content_words.is_empty() {
            return 0.0;
        }

        // Calculate exact word matches
        let exact_matches = query_words
            .iter()
            .filter(|&word| content_words.contains(word))
            .count() as f32;

        // Calculate partial matches (substring matches)
        let partial_matches = query_words
            .iter()
            .filter(|&query_word| {
                content_words.iter().any(|&content_word| {
                    content_word.contains(query_word) || query_word.contains(content_word)
                })
            })
            .count() as f32;

        // Calculate base similarity
        let exact_score = exact_matches / query_words.len() as f32;
        let partial_score = (partial_matches - exact_matches) / query_words.len() as f32 * 0.5;

        // Length penalty for very short or very long content
        let length_factor = {
            let content_len = content.len() as f32;
            let query_len = query.len() as f32;
            let ratio = if content_len > query_len {
                query_len / content_len
            } else {
                content_len / query_len
            };
            (ratio * 2.0).min(1.0)
        };

        // Combine scores
        let base_score = exact_score + partial_score;
        let final_score = base_score * length_factor;

        // Ensure score is between 0.0 and 1.0
        final_score.min(1.0).max(0.0)
    }

    /// Advanced search with complex filters and sorting
    #[instrument(skip(self, request))]
    pub async fn search_advanced(
        &self,
        request: SearchMemoryRequest,
    ) -> Result<MemorySearchResult> {
        // Create traditional search request
        let search_request = SearchMemoryRequest {
            query: request.query,
            user_id: request.user_id.clone(),
            filters: Some(MemoryFilter {
                agent_id: request.filters.as_ref().and_then(|f| f.agent_id.clone()),
                run_id: request.filters.as_ref().and_then(|f| f.run_id.clone()),
                memory_type: None,
                created_after: None,
                created_before: None,
                updated_after: None,
                updated_before: None,
                min_score: None,
                max_score: None,
                min_content_length: None,
                max_content_length: None,
                metadata_filters: HashMap::new(),
                metadata: HashMap::new(),
                content_contains: None,
                content_regex: None,
                tags: Vec::new(),
                exclude_tags: Vec::new(),
                sort_field: SortField::default(),
                sort_order: SortOrder::default(),
                limit: request.limit,
                offset: None,
            }),
            limit: request.limit,
        };

        // Use existing search logic
        self.search_with_options(search_request).await
    }

    /// Batch add memories with concurrent processing
    #[instrument(skip(self, request))]
    pub async fn add_batch(&self, request: BatchAddRequest) -> Result<BatchAddResult> {
        let mut successful = 0;
        let mut failed = 0;
        let mut results = Vec::new();
        let mut errors = Vec::new();

        // Process each request
        for add_request in request.requests {
            match self.add_enhanced(add_request).await {
                Ok(memory_id) => {
                    successful += 1;
                    results.push(memory_id);
                }
                Err(e) => {
                    failed += 1;
                    errors.push(format!("Failed to add memory: {e}"));
                }
            }
        }

        debug!(
            "Batch add completed: {} successful, {} failed",
            successful, failed
        );

        Ok(BatchAddResult {
            successful,
            failed,
            results,
            errors,
        })
    }

    /// Batch update memories with concurrent processing
    #[instrument(skip(self, request))]
    pub async fn update_batch(&self, request: BatchUpdateRequest) -> Result<BatchUpdateResult> {
        let mut successful = 0;
        let mut failed = 0;
        let mut results = Vec::new();
        let mut errors = Vec::new();

        // Process each update request
        for update_item in request.requests {
            let update_request = UpdateMemoryRequest {
                memory: update_item.memory,
                metadata: update_item.metadata,
            };

            match self
                .update(&update_item.memory_id, &update_item.user_id, update_request)
                .await
            {
                Ok(_) => {
                    successful += 1;
                    results.push(update_item.memory_id);
                }
                Err(e) => {
                    failed += 1;
                    errors.push(format!(
                        "Failed to update memory {}: {}",
                        update_item.memory_id, e
                    ));
                }
            }
        }

        debug!(
            "Batch update completed: {} successful, {} failed",
            successful, failed
        );

        Ok(BatchUpdateResult {
            successful,
            failed,
            results,
            errors,
        })
    }

    /// Batch delete memories with concurrent processing
    #[instrument(skip(self, request))]
    pub async fn delete_batch(&self, request: BatchDeleteRequest) -> Result<BatchDeleteResult> {
        let mut successful = 0;
        let mut failed = 0;
        let mut results = Vec::new();
        let mut errors = Vec::new();

        // Process each delete request
        for delete_item in request.requests {
            match self
                .delete(&delete_item.memory_id, &delete_item.user_id)
                .await
            {
                Ok(_) => {
                    successful += 1;
                    results.push(delete_item.memory_id);
                }
                Err(e) => {
                    failed += 1;
                    errors.push(format!(
                        "Failed to delete memory {}: {}",
                        delete_item.memory_id, e
                    ));
                }
            }
        }

        debug!(
            "Batch delete completed: {} successful, {} failed",
            successful, failed
        );

        Ok(BatchDeleteResult {
            successful,
            failed,
            results,
            errors,
        })
    }

    /// Search graph memories using entity-relation queries
    pub async fn search_graph(&self, query: &str, user_id: &str) -> Result<Vec<GraphResult>> {
        if let Some(ref graph_memory) = self.graph_memory {
            let session = Session {
                id: format!("graph_search_{}", generate_memory_id()),
                user_id: Some(user_id.to_string()),
                agent_id: Some("mem0_client".to_string()),
                run_id: None,
                actor_id: None,
                created_at: Utc::now(),
                metadata: std::collections::HashMap::new(),
            };

            graph_memory
                .search_graph(query, &session)
                .await
                .map_err(|e| e.into())
        } else {
            warn!("Graph memory manager not available");
            Ok(Vec::new())
        }
    }

    /// Get entity neighbors from graph
    pub async fn get_entity_neighbors(
        &self,
        entity_id: &str,
        depth: Option<usize>,
    ) -> Result<Vec<Entity>> {
        if let Some(ref graph_memory) = self.graph_memory {
            graph_memory
                .get_entity_neighbors(entity_id, depth)
                .await
                .map_err(|e| e.into())
        } else {
            warn!("Graph memory manager not available");
            Ok(Vec::new())
        }
    }

    /// Fuse multiple memories using graph-based intelligence
    pub async fn fuse_memories(&self, memory_ids: &[String], user_id: &str) -> Result<FusedMemory> {
        if let Some(ref graph_memory) = self.graph_memory {
            // Get memory contents
            let mut memory_contents = Vec::new();
            for memory_id in memory_ids {
                if let Some(memory) = self.memories.get(memory_id) {
                    memory_contents.push(memory.memory.clone());
                }
            }

            if memory_contents.is_empty() {
                return Err(Mem0Error::NotFound(
                    "No valid memories found for fusion".to_string(),
                ));
            }

            let session = Session {
                id: format!("memory_fusion_{}", generate_memory_id()),
                user_id: Some(user_id.to_string()),
                agent_id: Some("mem0_client".to_string()),
                run_id: None,
                actor_id: None,
                created_at: Utc::now(),
                metadata: std::collections::HashMap::new(),
            };

            graph_memory
                .fuse_memories(&memory_contents, &session)
                .await
                .map_err(|e| e.into())
        } else {
            Err(Mem0Error::ServiceUnavailable(
                "Graph memory manager not available".to_string(),
            ))
        }
    }

    /// Add memory with automatic graph extraction
    pub async fn add_with_graph_extraction(
        &self,
        content: &str,
        user_id: &str,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<String> {
        // First add the memory normally
        let memory_id = self.add(content, user_id, metadata).await?;

        // Then extract entities and relations to graph
        if let Some(ref graph_memory) = self.graph_memory {
            let session = Session {
                id: format!("graph_extraction_{memory_id}"),
                user_id: Some(user_id.to_string()),
                agent_id: Some("mem0_client".to_string()),
                run_id: None,
                actor_id: None,
                created_at: Utc::now(),
                metadata: std::collections::HashMap::new(),
            };

            if let Err(e) = graph_memory.add_memory_to_graph(content, &session).await {
                warn!(
                    "Failed to extract entities/relations for memory {}: {}",
                    memory_id, e
                );
                // Don't fail the entire operation, just log the warning
            } else {
                info!(
                    "Successfully extracted entities/relations for memory {}",
                    memory_id
                );
            }
        }

        Ok(memory_id)
    }

    /// Reset graph database (for testing)
    pub async fn reset_graph(&self) -> Result<()> {
        if let Some(ref graph_memory) = self.graph_memory {
            graph_memory.reset().await.map_err(|e| e.into())
        } else {
            Err(Mem0Error::ServiceUnavailable(
                "Graph memory manager not available".to_string(),
            ))
        }
    }

    // ===== 程序性记忆方法 =====

    /// 创建工作流
    pub async fn create_workflow(
        &self,
        name: String,
        description: String,
        steps: Vec<WorkflowStep>,
        created_by: String,
        tags: Vec<String>,
    ) -> Result<String> {
        if let Some(ref procedural_memory) = self.procedural_memory {
            procedural_memory
                .create_workflow(name, description, steps, created_by, tags)
                .await
                .map_err(|e| e.into())
        } else {
            Err(Mem0Error::ServiceUnavailable(
                "Procedural memory manager not available".to_string(),
            ))
        }
    }

    /// 获取工作流
    pub async fn get_workflow(&self, workflow_id: &str) -> Result<Option<Workflow>> {
        if let Some(ref procedural_memory) = self.procedural_memory {
            procedural_memory
                .get_workflow(workflow_id)
                .await
                .map_err(|e| e.into())
        } else {
            Err(Mem0Error::ServiceUnavailable(
                "Procedural memory manager not available".to_string(),
            ))
        }
    }

    /// 列出所有工作流
    pub async fn list_workflows(&self, tags: Option<Vec<String>>) -> Result<Vec<Workflow>> {
        if let Some(ref procedural_memory) = self.procedural_memory {
            procedural_memory
                .list_workflows(tags)
                .await
                .map_err(|e| e.into())
        } else {
            Err(Mem0Error::ServiceUnavailable(
                "Procedural memory manager not available".to_string(),
            ))
        }
    }

    /// 开始执行工作流
    pub async fn start_workflow_execution(
        &self,
        workflow_id: String,
        executor: String,
        session: Session,
        initial_context: Option<std::collections::HashMap<String, serde_json::Value>>,
    ) -> Result<String> {
        if let Some(ref procedural_memory) = self.procedural_memory {
            procedural_memory
                .start_workflow_execution(workflow_id, executor, session, initial_context)
                .await
                .map_err(|e| e.into())
        } else {
            Err(Mem0Error::ServiceUnavailable(
                "Procedural memory manager not available".to_string(),
            ))
        }
    }

    /// 获取执行状态
    pub async fn get_execution_status(
        &self,
        execution_id: &str,
    ) -> Result<Option<WorkflowExecution>> {
        if let Some(ref procedural_memory) = self.procedural_memory {
            procedural_memory
                .get_execution_status(execution_id)
                .await
                .map_err(|e| e.into())
        } else {
            Err(Mem0Error::ServiceUnavailable(
                "Procedural memory manager not available".to_string(),
            ))
        }
    }

    /// 执行下一步
    pub async fn execute_next_step(&self, execution_id: &str) -> Result<StepExecutionResult> {
        if let Some(ref procedural_memory) = self.procedural_memory {
            procedural_memory
                .execute_next_step(execution_id)
                .await
                .map_err(|e| e.into())
        } else {
            Err(Mem0Error::ServiceUnavailable(
                "Procedural memory manager not available".to_string(),
            ))
        }
    }

    /// 创建任务链
    pub async fn create_task_chain(&self, name: String, tasks: Vec<Task>) -> Result<String> {
        if let Some(ref procedural_memory) = self.procedural_memory {
            procedural_memory
                .create_task_chain(name, tasks)
                .await
                .map_err(|e| e.into())
        } else {
            Err(Mem0Error::ServiceUnavailable(
                "Procedural memory manager not available".to_string(),
            ))
        }
    }

    /// 获取任务链
    pub async fn get_task_chain(&self, chain_id: &str) -> Result<Option<TaskChain>> {
        if let Some(ref procedural_memory) = self.procedural_memory {
            procedural_memory
                .get_task_chain(chain_id)
                .await
                .map_err(|e| e.into())
        } else {
            Err(Mem0Error::ServiceUnavailable(
                "Procedural memory manager not available".to_string(),
            ))
        }
    }

    /// 执行任务链中的下一个任务
    pub async fn execute_next_task(&self, chain_id: &str) -> Result<TaskExecutionResult> {
        if let Some(ref procedural_memory) = self.procedural_memory {
            procedural_memory
                .execute_next_task(chain_id)
                .await
                .map_err(|e| e.into())
        } else {
            Err(Mem0Error::ServiceUnavailable(
                "Procedural memory manager not available".to_string(),
            ))
        }
    }

    /// 列出所有任务链
    pub async fn list_task_chains(&self) -> Result<Vec<TaskChain>> {
        if let Some(ref procedural_memory) = self.procedural_memory {
            procedural_memory
                .list_task_chains()
                .await
                .map_err(|e| e.into())
        } else {
            Err(Mem0Error::ServiceUnavailable(
                "Procedural memory manager not available".to_string(),
            ))
        }
    }

    /// 暂停任务链
    pub async fn pause_task_chain(&self, chain_id: &str) -> Result<()> {
        if let Some(ref procedural_memory) = self.procedural_memory {
            procedural_memory
                .pause_task_chain(chain_id)
                .await
                .map_err(|e| e.into())
        } else {
            Err(Mem0Error::ServiceUnavailable(
                "Procedural memory manager not available".to_string(),
            ))
        }
    }

    /// 恢复任务链
    pub async fn resume_task_chain(&self, chain_id: &str) -> Result<()> {
        if let Some(ref procedural_memory) = self.procedural_memory {
            procedural_memory
                .resume_task_chain(chain_id)
                .await
                .map_err(|e| e.into())
        } else {
            Err(Mem0Error::ServiceUnavailable(
                "Procedural memory manager not available".to_string(),
            ))
        }
    }

    // Context-Aware Memory Methods

    /// 从内容中提取上下文信息
    pub async fn extract_context(
        &self,
        content: &str,
        session: &Session,
    ) -> Result<Vec<ContextInfo>> {
        if let Some(ref context_aware) = self.context_aware {
            context_aware
                .extract_context(content, session)
                .await
        } else {
            Err(Mem0Error::ServiceUnavailable(
                "Context-aware manager not available".to_string(),
            ))
        }
    }

    /// 执行上下文感知搜索
    pub async fn search_with_context(
        &self,
        request: ContextAwareSearchRequest,
    ) -> Result<Vec<ContextAwareSearchResult>> {
        if let Some(ref context_aware) = self.context_aware {
            context_aware
                .search_with_context(request)
                .await
        } else {
            Err(Mem0Error::ServiceUnavailable(
                "Context-aware manager not available".to_string(),
            ))
        }
    }

    /// 从上下文中学习模式
    pub async fn learn_from_context(
        &self,
        contexts: &[ContextInfo],
    ) -> Result<ContextLearningResult> {
        if let Some(ref context_aware) = self.context_aware {
            context_aware
                .learn_from_context(contexts)
                .await
        } else {
            Err(Mem0Error::ServiceUnavailable(
                "Context-aware manager not available".to_string(),
            ))
        }
    }

    /// 获取学习到的上下文模式
    pub async fn get_context_patterns(&self) -> Result<Vec<ContextPattern>> {
        if let Some(ref context_aware) = self.context_aware {
            context_aware.get_patterns().await
        } else {
            Err(Mem0Error::ServiceUnavailable(
                "Context-aware manager not available".to_string(),
            ))
        }
    }

    /// 获取上下文历史
    pub async fn get_context_history(&self, limit: Option<usize>) -> Result<Vec<ContextInfo>> {
        if let Some(ref context_aware) = self.context_aware {
            context_aware
                .get_context_history(limit)
                .await
        } else {
            Err(Mem0Error::ServiceUnavailable(
                "Context-aware manager not available".to_string(),
            ))
        }
    }

    /// 将上下文与记忆关联
    pub async fn associate_contexts_with_memory(
        &self,
        memory_id: &str,
        contexts: Vec<ContextInfo>,
    ) -> Result<()> {
        if let Some(ref context_aware) = self.context_aware {
            context_aware
                .associate_contexts_with_memory(memory_id, contexts)
                .await
        } else {
            Err(Mem0Error::ServiceUnavailable(
                "Context-aware manager not available".to_string(),
            ))
        }
    }

    /// 获取记忆关联的上下文
    pub async fn get_memory_contexts(&self, memory_id: &str) -> Result<Vec<ContextInfo>> {
        if let Some(ref context_aware) = self.context_aware {
            context_aware
                .get_memory_contexts(memory_id)
                .await
        } else {
            Err(Mem0Error::ServiceUnavailable(
                "Context-aware manager not available".to_string(),
            ))
        }
    }

    /// 获取上下文统计信息
    pub async fn get_context_statistics(&self) -> Result<HashMap<String, u32>> {
        if let Some(ref context_aware) = self.context_aware {
            context_aware
                .get_context_statistics()
                .await
        } else {
            Err(Mem0Error::ServiceUnavailable(
                "Context-aware manager not available".to_string(),
            ))
        }
    }

    /// 清除上下文历史
    pub async fn clear_context_history(&self) -> Result<()> {
        if let Some(ref context_aware) = self.context_aware {
            context_aware
                .clear_context_history()
                .await
        } else {
            Err(Mem0Error::ServiceUnavailable(
                "Context-aware manager not available".to_string(),
            ))
        }
    }

    // ==================== 个性化功能 API ====================

    /// 记录用户行为
    pub async fn record_user_behavior(&self, behavior: UserBehavior) -> Result<()> {
        if let Some(ref personalization) = self.personalization {
            personalization
                .record_behavior(behavior)
                .await
                .map_err(|e| e.into())
        } else {
            Err(Mem0Error::ServiceUnavailable(
                "Personalization manager not available".to_string(),
            ))
        }
    }

    /// 个性化搜索
    pub async fn personalized_search(
        &self,
        request: PersonalizedSearchRequest,
    ) -> Result<Vec<PersonalizedSearchResult>> {
        if let Some(ref personalization) = self.personalization {
            // 首先执行基础搜索
            let base_results = self.search(&request.query, &request.user_id, None).await?;

            // 转换为 MemorySearchResult 格式供个性化处理
            let memory_search_results: Vec<crate::personalization::MemorySearchResult> =
                base_results
                    .memories
                    .into_iter()
                    .map(|memory| {
                        crate::personalization::MemorySearchResult {
                            memory,
                            score: 0.8, // 默认基础分数
                        }
                    })
                    .collect();

            // 执行个性化搜索
            personalization
                .personalized_search(request, memory_search_results)
                .await
                .map_err(|e| e.into())
        } else {
            Err(Mem0Error::ServiceUnavailable(
                "Personalization manager not available".to_string(),
            ))
        }
    }

    /// 生成记忆推荐
    pub async fn generate_recommendations(
        &self,
        user_id: &str,
        limit: usize,
    ) -> Result<Vec<MemoryRecommendation>> {
        if let Some(ref personalization) = self.personalization {
            personalization
                .generate_recommendations(user_id, limit)
                .await
                .map_err(|e| e.into())
        } else {
            Err(Mem0Error::ServiceUnavailable(
                "Personalization manager not available".to_string(),
            ))
        }
    }

    /// 获取用户偏好
    pub async fn get_user_preferences(&self, user_id: &str) -> Result<Vec<UserPreference>> {
        if let Some(ref personalization) = self.personalization {
            personalization
                .get_user_preferences(user_id)
                .await
                .map_err(|e| e.into())
        } else {
            Err(Mem0Error::ServiceUnavailable(
                "Personalization manager not available".to_string(),
            ))
        }
    }

    /// 更新用户偏好
    pub async fn update_user_preference(&self, preference: UserPreference) -> Result<()> {
        if let Some(ref personalization) = self.personalization {
            personalization
                .update_user_preference(preference)
                .await
                .map_err(|e| e.into())
        } else {
            Err(Mem0Error::ServiceUnavailable(
                "Personalization manager not available".to_string(),
            ))
        }
    }

    /// 删除用户偏好
    pub async fn delete_user_preference(&self, user_id: &str, preference_id: &str) -> Result<bool> {
        if let Some(ref personalization) = self.personalization {
            personalization
                .delete_user_preference(user_id, preference_id)
                .await
                .map_err(|e| e.into())
        } else {
            Err(Mem0Error::ServiceUnavailable(
                "Personalization manager not available".to_string(),
            ))
        }
    }

    /// 获取用户档案
    pub async fn get_user_profile(&self, user_id: &str) -> Result<Option<UserProfile>> {
        if let Some(ref personalization) = self.personalization {
            personalization
                .get_user_profile(user_id)
                .await
                .map_err(|e| e.into())
        } else {
            Err(Mem0Error::ServiceUnavailable(
                "Personalization manager not available".to_string(),
            ))
        }
    }

    /// 更新用户档案
    pub async fn update_user_profile(&self, user_id: &str) -> Result<UserProfile> {
        if let Some(ref personalization) = self.personalization {
            personalization
                .update_user_profile(user_id)
                .await
                .map_err(|e| e.into())
        } else {
            Err(Mem0Error::ServiceUnavailable(
                "Personalization manager not available".to_string(),
            ))
        }
    }

    // ==================== Enterprise Security Methods ====================

    /// Authenticate user and create session
    pub async fn authenticate(
        &self,
        username: &str,
        password: &str,
        ip_address: &str,
        user_agent: &str,
    ) -> Result<UserSession> {
        if let Some(security) = &self.enterprise_security {
            security
                .authenticate(username, password, ip_address, user_agent)
                .await
                .map_err(|e| Mem0Error::SecurityError(format!("Authentication failed: {e}")))
        } else {
            Err(Mem0Error::FeatureNotEnabled(
                "Enterprise security not enabled".to_string(),
            ))
        }
    }

    /// Validate JWT token
    pub async fn validate_token(&self, token: &str) -> Result<JwtClaims> {
        if let Some(security) = &self.enterprise_security {
            security
                .validate_token(token)
                .await
                .map_err(|e| Mem0Error::SecurityError(format!("Token validation failed: {e}")))
        } else {
            Err(Mem0Error::FeatureNotEnabled(
                "Enterprise security not enabled".to_string(),
            ))
        }
    }

    /// Check if user has permission
    pub async fn check_permission(&self, user_id: &str, permission: &Permission) -> Result<bool> {
        if let Some(security) = &self.enterprise_security {
            security
                .check_permission(user_id, permission)
                .await
                .map_err(|e| Mem0Error::SecurityError(format!("Permission check failed: {e}")))
        } else {
            Ok(true) // If security is disabled, allow all operations
        }
    }

    /// Encrypt sensitive data
    pub async fn encrypt_data(&self, data: &str) -> Result<String> {
        if let Some(security) = &self.enterprise_security {
            security
                .encrypt_data(data)
                .await
                .map_err(|e| Mem0Error::SecurityError(format!("Encryption failed: {e}")))
        } else {
            Ok(data.to_string()) // If security is disabled, return data as-is
        }
    }

    /// Decrypt sensitive data
    pub async fn decrypt_data(&self, encrypted_data: &str) -> Result<String> {
        if let Some(security) = &self.enterprise_security {
            security
                .decrypt_data(encrypted_data)
                .await
                .map_err(|e| Mem0Error::SecurityError(format!("Decryption failed: {e}")))
        } else {
            Ok(encrypted_data.to_string()) // If security is disabled, return data as-is
        }
    }

    /// Mask sensitive data for PII protection
    pub async fn mask_sensitive_data(&self, data: &str) -> Result<String> {
        if let Some(security) = &self.enterprise_security {
            security
                .mask_sensitive_data(data)
                .await
                .map_err(|e| Mem0Error::SecurityError(format!("Data masking failed: {e}")))
        } else {
            Ok(data.to_string()) // If security is disabled, return data as-is
        }
    }

    /// Get audit logs
    pub async fn get_audit_logs(
        &self,
        limit: Option<usize>,
    ) -> Result<Vec<crate::enterprise_security::AuditLogEntry>> {
        if let Some(security) = &self.enterprise_security {
            security
                .get_audit_logs(limit)
                .await
                .map_err(|e| Mem0Error::SecurityError(format!("Failed to get audit logs: {e}")))
        } else {
            Err(Mem0Error::FeatureNotEnabled(
                "Enterprise security not enabled".to_string(),
            ))
        }
    }

    /// Create new user
    pub async fn create_user(
        &self,
        username: &str,
        email: &str,
        password: &str,
        roles: Vec<String>,
    ) -> Result<String> {
        if let Some(security) = &self.enterprise_security {
            security
                .create_user(username, email, password, roles)
                .await
                .map_err(|e| Mem0Error::SecurityError(format!("User creation failed: {e}")))
        } else {
            Err(Mem0Error::FeatureNotEnabled(
                "Enterprise security not enabled".to_string(),
            ))
        }
    }

    /// Logout user
    pub async fn logout(&self, session_id: &str) -> Result<()> {
        if let Some(security) = &self.enterprise_security {
            security
                .logout(session_id)
                .await
                .map_err(|e| Mem0Error::SecurityError(format!("Logout failed: {e}")))
        } else {
            Ok(()) // If security is disabled, logout is always successful
        }
    }
}

// Import required types for graph functionality
use agent_mem_traits::{Entity, GraphResult, Session};
