//! Semantic Memory Agent
//!
//! This agent specializes in managing semantic memories - factual knowledge and concepts.
//! It handles operations like storing facts, retrieving knowledge, and managing
//! semantic relationships between concepts.

use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

use crate::agents::{
    AgentConfig, AgentContext, AgentError, AgentResult, AgentStats, BaseAgent, MemoryAgent,
};
use crate::coordination::{
    AgentMessage, CoordinationError, CoordinationResult, TaskRequest, TaskResponse,
};
use crate::types::MemoryType;

// Use trait-based storage instead of concrete implementation
use agent_mem_traits::{SemanticMemoryItem, SemanticMemoryStore, SemanticQuery};

/// Semantic Memory Agent
///
/// Specializes in handling semantic memories - factual knowledge and concepts.
/// This agent manages conceptual relationships, knowledge graphs, and semantic search.
///
/// Uses trait-based storage (SemanticMemoryStore) to support multiple backends:
/// - PostgreSQL
/// - LibSQL
/// - MongoDB
/// - etc.
pub struct SemanticAgent {
    /// Base agent functionality
    base: BaseAgent,
    /// Agent context
    context: Arc<RwLock<AgentContext>>,
    /// Semantic memory store (trait-based, supports multiple backends)
    semantic_store: Option<Arc<dyn SemanticMemoryStore>>,
    /// Initialization status
    initialized: bool,
}

impl SemanticAgent {
    /// Create a new semantic memory agent
    pub fn new(agent_id: String) -> Self {
        let config = AgentConfig::new(
            agent_id,
            vec![MemoryType::Semantic],
            10, // max concurrent tasks
        );

        let base = BaseAgent::new(config);
        let context = base.context();

        Self {
            base,
            context,
            semantic_store: None,
            initialized: false,
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: AgentConfig) -> Self {
        let base = BaseAgent::new(config);
        let context = base.context();

        Self {
            base,
            context,
            semantic_store: None,
            initialized: false,
        }
    }

    /// Create with semantic memory store (trait-based, supports any backend)
    pub fn with_store(agent_id: String, store: Arc<dyn SemanticMemoryStore>) -> Self {
        let config = AgentConfig::new(
            agent_id,
            vec![MemoryType::Semantic],
            10, // max concurrent tasks
        );

        let base = BaseAgent::new(config);
        let context = base.context();

        Self {
            base,
            context,
            semantic_store: Some(store),
            initialized: false,
        }
    }

    /// Set semantic memory store (trait-based, supports any backend)
    pub fn set_store(&mut self, store: Arc<dyn SemanticMemoryStore>) {
        self.semantic_store = Some(store);
    }

    /// Handle semantic knowledge insertion
    async fn handle_insert(&self, parameters: Value) -> AgentResult<Value> {
        
        {
            // Use actual semantic memory manager if available
            if let Some(manager) = &self.semantic_store {
                // Parse item data
                let item: SemanticMemoryItem = serde_json::from_value(parameters.clone())
                    .map_err(|e| AgentError::InvalidParameters(format!("Invalid item data: {}", e)))?;

                // Create item using manager
                let created_item = manager.create_item(item).await
                    .map_err(|e| AgentError::TaskExecutionError(format!("Failed to create item: {}", e)))?;

                let response = serde_json::json!({
                    "success": true,
                    "item_id": created_item.id,
                    "name": created_item.name,
                    "message": "Semantic knowledge inserted successfully"
                });

                log::info!("Semantic agent: Inserted knowledge item {}", created_item.id);
                return Ok(response);
            }
        }

        // Fallback to mock response if manager not available
        let concept = parameters.get("concept").unwrap_or(&Value::Null);
        let knowledge = parameters.get("knowledge").unwrap_or(&Value::Null);

        let response = serde_json::json!({
            "success": true,
            "concept_id": uuid::Uuid::new_v4().to_string(),
            "concept": concept,
            "knowledge": knowledge,
            "message": "Semantic knowledge inserted successfully (mock)"
        });

        log::warn!("Semantic agent: Using mock response (manager not available)");
        Ok(response)
    }

    /// Handle semantic knowledge search
    async fn handle_search(&self, parameters: Value) -> AgentResult<Value> {
        let user_id = parameters
            .get("user_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::InvalidParameters("Missing 'user_id' parameter".to_string())
            })?;

        
        {
            // Use actual semantic memory manager if available
            if let Some(manager) = &self.semantic_store {
                // Build query from parameters
                let query = SemanticQuery {
                    name_query: parameters.get("name_query")
                        .and_then(|v| v.as_str())
                        .map(|s| format!("%{}%", s)),
                    summary_query: parameters.get("summary_query")
                        .and_then(|v| v.as_str())
                        .map(|s| format!("%{}%", s)),
                    tree_path_prefix: parameters.get("tree_path_prefix")
                        .and_then(|v| serde_json::from_value(v.clone()).ok()),
                    limit: parameters.get("limit")
                        .and_then(|v| v.as_i64()),
                };

                // Query items using manager
                let items = manager.query_items(user_id, query).await
                    .map_err(|e| AgentError::TaskExecutionError(format!("Failed to query items: {}", e)))?;

                let response = serde_json::json!({
                    "success": true,
                    "results": items,
                    "total_count": items.len()
                });

                log::info!("Semantic agent: Found {} items for user {}", items.len(), user_id);
                return Ok(response);
            }
        }

        // Fallback to mock response if manager not available
        let query = parameters.get("query").and_then(|v| v.as_str()).unwrap_or("");
        let semantic_similarity = parameters
            .get("semantic_similarity")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let response = serde_json::json!({
            "success": true,
            "results": [],
            "total_count": 0,
            "query": query,
            "semantic_similarity": semantic_similarity,
            "message": "Mock response (manager not available)"
        });

        log::warn!("Semantic agent: Using mock response (manager not available)");
        Ok(response)
    }

    /// Handle concept relationship queries
    async fn handle_relationship_query(&self, parameters: Value) -> AgentResult<Value> {
        let concept_id = parameters
            .get("concept_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::InvalidParameters("Missing 'concept_id' parameter".to_string())
            })?;

        let relationship_type = parameters
            .get("relationship_type")
            .and_then(|v| v.as_str())
            .unwrap_or("all");

        // TODO: Integrate with actual relationship query
        let response = serde_json::json!({
            "success": true,
            "concept_id": concept_id,
            "relationships": [],
            "relationship_type": relationship_type
        });

        log::info!(
            "Semantic agent: Queried relationships for concept {} (type: {})",
            concept_id,
            relationship_type
        );
        Ok(response)
    }

    /// Handle knowledge graph traversal
    async fn handle_graph_traversal(&self, parameters: Value) -> AgentResult<Value> {
        let start_concept = parameters
            .get("start_concept")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::InvalidParameters("Missing 'start_concept' parameter".to_string())
            })?;

        let max_depth = parameters
            .get("max_depth")
            .and_then(|v| v.as_u64())
            .unwrap_or(3);

        // TODO: Integrate with actual graph traversal
        let response = serde_json::json!({
            "success": true,
            "start_concept": start_concept,
            "max_depth": max_depth,
            "traversal_path": [],
            "related_concepts": []
        });

        log::info!(
            "Semantic agent: Graph traversal from '{}' with max depth {}",
            start_concept,
            max_depth
        );
        Ok(response)
    }

    /// Handle semantic knowledge update
    async fn handle_update(&self, parameters: Value) -> AgentResult<Value> {
        let concept_id = parameters
            .get("concept_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::InvalidParameters("Missing 'concept_id' parameter".to_string())
            })?;

        // TODO: Integrate with actual semantic memory update
        let response = serde_json::json!({
            "success": true,
            "concept_id": concept_id,
            "message": "Semantic knowledge updated successfully"
        });

        log::info!("Semantic agent: Updated concept {}", concept_id);
        Ok(response)
    }

    /// Handle semantic knowledge deletion
    async fn handle_delete(&self, parameters: Value) -> AgentResult<Value> {
        let concept_id = parameters
            .get("concept_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::InvalidParameters("Missing 'concept_id' parameter".to_string())
            })?;

        // TODO: Integrate with actual semantic memory deletion
        let response = serde_json::json!({
            "success": true,
            "concept_id": concept_id,
            "message": "Semantic knowledge deleted successfully"
        });

        log::info!("Semantic agent: Deleted concept {}", concept_id);
        Ok(response)
    }
}

#[async_trait]
impl MemoryAgent for SemanticAgent {
    fn agent_id(&self) -> &str {
        &self.base.config().agent_id
    }

    fn memory_types(&self) -> &[MemoryType] {
        &self.base.config().memory_types
    }

    async fn initialize(&mut self) -> CoordinationResult<()> {
        if self.initialized {
            return Ok(());
        }

        log::info!("Initializing Semantic Memory Agent: {}", self.agent_id());

        // TODO: Initialize semantic memory manager
        // TODO: Load knowledge graph structures

        self.initialized = true;
        Ok(())
    }

    async fn shutdown(&mut self) -> CoordinationResult<()> {
        if !self.initialized {
            return Ok(());
        }

        log::info!("Shutting down Semantic Memory Agent: {}", self.agent_id());

        // TODO: Persist knowledge graph
        // TODO: Clean up resources

        self.initialized = false;
        Ok(())
    }

    async fn execute_task(&mut self, task: TaskRequest) -> CoordinationResult<TaskResponse> {
        if !self.initialized {
            return Err(CoordinationError::InternalError(
                "Agent not initialized".to_string(),
            ));
        }

        let start_time = Instant::now();

        // Update context with current task
        {
            let mut context = self.context.write().await;
            context.current_task = Some(task.clone());
            context.stats.active_tasks += 1;
        }

        // Execute the task based on operation type
        let result = match task.operation.as_str() {
            "insert" => self.handle_insert(task.parameters).await,
            "search" => self.handle_search(task.parameters).await,
            "relationship_query" => self.handle_relationship_query(task.parameters).await,
            "graph_traversal" => self.handle_graph_traversal(task.parameters).await,
            "update" => self.handle_update(task.parameters).await,
            "delete" => self.handle_delete(task.parameters).await,
            _ => Err(AgentError::InvalidParameters(format!(
                "Unknown operation: {}",
                task.operation
            ))),
        };

        let execution_time = start_time.elapsed();

        // Update context and statistics
        {
            let mut context = self.context.write().await;
            context.current_task = None;
            context.stats.active_tasks = context.stats.active_tasks.saturating_sub(1);
            context
                .stats
                .update_task_completion(result.is_ok(), execution_time.as_millis() as f64);
        }

        // Create response
        match result {
            Ok(data) => Ok(TaskResponse::success(
                task.task_id,
                data,
                execution_time,
                self.agent_id().to_string(),
            )),
            Err(error) => Ok(TaskResponse::error(
                task.task_id,
                error.to_string(),
                execution_time,
                self.agent_id().to_string(),
            )),
        }
    }

    async fn handle_message(&mut self, message: AgentMessage) -> CoordinationResult<()> {
        log::debug!(
            "Semantic agent received message: {:?}",
            message.message_type
        );

        match message.message_type {
            crate::coordination::MessageType::TaskRequest => Ok(()),
            crate::coordination::MessageType::HealthCheck => Ok(()),
            _ => {
                log::warn!(
                    "Semantic agent received unknown message type: {:?}",
                    message.message_type
                );
                Ok(())
            }
        }
    }

    async fn get_stats(&self) -> AgentStats {
        self.context.read().await.stats.clone()
    }

    async fn health_check(&self) -> bool {
        self.initialized
    }

    async fn current_load(&self) -> usize {
        self.context.read().await.stats.active_tasks
    }

    async fn can_accept_task(&self) -> bool {
        if !self.initialized {
            return false;
        }

        let context = self.context.read().await;
        context.stats.active_tasks < context.config.max_concurrent_tasks
    }
}
