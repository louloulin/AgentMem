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
    ///
    /// **Note**: This creates an agent without a store configured.
    /// For production use, prefer `from_env()` or `with_store()`.
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

    /// Create a new semantic memory agent with store from environment configuration
    ///
    /// This method automatically initializes the store based on environment variables.
    /// See `config_env` module for supported environment variables.
    pub async fn from_env(agent_id: String) -> agent_mem_traits::Result<Self> {
        use crate::config_env::create_stores_from_env;

        let stores = create_stores_from_env().await?;
        Ok(Self::with_store(agent_id, stores.semantic))
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
                let item: SemanticMemoryItem =
                    serde_json::from_value(parameters.clone()).map_err(|e| {
                        AgentError::InvalidParameters(format!("Invalid item data: {e}"))
                    })?;

                // Create item using manager
                let created_item = manager.create_item(item).await.map_err(|e| {
                    AgentError::TaskExecutionError(format!("Failed to create item: {e}"))
                })?;

                let response = serde_json::json!({
                    "success": true,
                    "item_id": created_item.id,
                    "name": created_item.name,
                    "message": "Semantic knowledge inserted successfully"
                });

                log::info!(
                    "Semantic agent: Inserted knowledge item {}",
                    created_item.id
                );
                return Ok(response);
            }
        }

        // No store configured - return error instead of mock
        log::error!("Semantic agent: No store configured, cannot insert knowledge");
        Err(AgentError::ConfigurationError(
            "Semantic memory store not configured. Use SemanticAgent::from_env() or set_store() to configure storage.".to_string()
        ))
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
                    name_query: parameters
                        .get("name_query")
                        .and_then(|v| v.as_str())
                        .map(|s| format!("%{s}%")),
                    summary_query: parameters
                        .get("summary_query")
                        .and_then(|v| v.as_str())
                        .map(|s| format!("%{s}%")),
                    tree_path_prefix: parameters
                        .get("tree_path_prefix")
                        .and_then(|v| serde_json::from_value(v.clone()).ok()),
                    limit: parameters.get("limit").and_then(|v| v.as_i64()),
                };

                // Query items using manager
                let items = manager.query_items(user_id, query).await.map_err(|e| {
                    AgentError::TaskExecutionError(format!("Failed to query items: {e}"))
                })?;

                let response = serde_json::json!({
                    "success": true,
                    "results": items,
                    "total_count": items.len()
                });

                log::info!(
                    "Semantic agent: Found {} items for user {}",
                    items.len(),
                    user_id
                );
                return Ok(response);
            }
        }

        // No store configured - return error instead of mock
        log::error!("Semantic agent: No store configured, cannot search knowledge");
        Err(AgentError::ConfigurationError(
            "Semantic memory store not configured. Use SemanticAgent::from_env() or set_store() to configure storage.".to_string()
        ))
    }

    /// Handle concept relationship queries
    ///
    /// Note: This is a simplified implementation using tree_path for relationships.
    /// For full graph relationship support, a graph database backend would be needed.
    async fn handle_relationship_query(&self, parameters: Value) -> AgentResult<Value> {
        // Use semantic_store if available
        if let Some(store) = &self.semantic_store {
            let concept_id = parameters
                .get("concept_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    AgentError::InvalidParameters("Missing 'concept_id' parameter".to_string())
                })?;

            let user_id = parameters
                .get("user_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    AgentError::InvalidParameters("Missing 'user_id' parameter".to_string())
                })?;

            // Get the concept item
            let item = store
                .get_item(concept_id, user_id)
                .await
                .map_err(|e| AgentError::MemoryManagerError(format!("Failed to get item: {e}")))?;

            if let Some(item) = item {
                // Use tree_path to find related items (simplified relationship model)
                let related_items = store
                    .search_by_tree_path(user_id, item.tree_path.clone())
                    .await
                    .map_err(|e| {
                        AgentError::MemoryManagerError(format!(
                            "Failed to search by tree path: {e}"
                        ))
                    })?;

                // Filter out the concept itself
                let relationships: Vec<_> = related_items
                    .into_iter()
                    .filter(|r| r.id != concept_id)
                    .map(|r| {
                        serde_json::json!({
                            "id": r.id,
                            "name": r.name,
                            "summary": r.summary,
                            "tree_path": r.tree_path,
                            "relationship_type": "tree_sibling" // Simplified relationship type
                        })
                    })
                    .collect();

                log::info!(
                    "Semantic agent: Found {} relationships for concept {} using real storage",
                    relationships.len(),
                    concept_id
                );

                return Ok(serde_json::json!({
                    "success": true,
                    "concept_id": concept_id,
                    "relationships": relationships,
                    "relationship_type": "tree_based"
                }));
            } else {
                log::warn!("Semantic agent: Concept {concept_id} not found");
                return Ok(serde_json::json!({
                    "success": false,
                    "concept_id": concept_id,
                    "message": "Concept not found"
                }));
            }
        }

        // Fallback to mock response if store not available
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

        // No store configured - return error instead of mock
        log::error!("Semantic agent: No store configured, cannot query relationships");
        Err(AgentError::ConfigurationError(
            "Semantic memory store not configured. Use SemanticAgent::from_env() or set_store() to configure storage.".to_string()
        ))
    }

    /// Handle knowledge graph traversal
    ///
    /// Note: This is a simplified implementation using tree_path for traversal.
    /// For full graph traversal support, a graph database backend would be needed.
    async fn handle_graph_traversal(&self, parameters: Value) -> AgentResult<Value> {
        // Use semantic_store if available
        if let Some(store) = &self.semantic_store {
            let start_concept = parameters
                .get("start_concept")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    AgentError::InvalidParameters("Missing 'start_concept' parameter".to_string())
                })?;

            let user_id = parameters
                .get("user_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    AgentError::InvalidParameters("Missing 'user_id' parameter".to_string())
                })?;

            let max_depth = parameters
                .get("max_depth")
                .and_then(|v| v.as_u64())
                .unwrap_or(3) as usize;

            // Get the starting concept
            let start_item = store.get_item(start_concept, user_id).await.map_err(|e| {
                AgentError::MemoryManagerError(format!("Failed to get start concept: {e}"))
            })?;

            if let Some(start_item) = start_item {
                // Simplified traversal: explore tree path hierarchy
                let traversal_path = vec![serde_json::json!({
                    "id": start_item.id,
                    "name": start_item.name,
                    "depth": 0
                })];

                let mut related_concepts = Vec::new();

                // Traverse up to max_depth levels
                for depth in 1..=max_depth {
                    // Get items at current tree path level
                    let mut current_path = start_item.tree_path.clone();
                    if current_path.len() >= depth {
                        current_path.truncate(current_path.len() - depth + 1);

                        let items = store
                            .search_by_tree_path(user_id, current_path)
                            .await
                            .map_err(|e| {
                                AgentError::MemoryManagerError(format!("Failed to traverse: {e}"))
                            })?;

                        for item in items {
                            if item.id != start_concept {
                                related_concepts.push(serde_json::json!({
                                    "id": item.id,
                                    "name": item.name,
                                    "summary": item.summary,
                                    "depth": depth,
                                    "tree_path": item.tree_path
                                }));
                            }
                        }
                    }
                }

                log::info!(
                    "Semantic agent: Graph traversal from '{}' found {} related concepts using real storage",
                    start_concept,
                    related_concepts.len()
                );

                return Ok(serde_json::json!({
                    "success": true,
                    "start_concept": start_concept,
                    "max_depth": max_depth,
                    "traversal_path": traversal_path,
                    "related_concepts": related_concepts,
                    "traversal_type": "tree_based"
                }));
            } else {
                log::warn!("Semantic agent: Start concept {start_concept} not found");
                return Ok(serde_json::json!({
                    "success": false,
                    "start_concept": start_concept,
                    "message": "Start concept not found"
                }));
            }
        }

        // Fallback to mock response if store not available
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

        // No store configured - return error instead of mock
        log::error!("Semantic agent: No store configured, cannot traverse graph");
        Err(AgentError::ConfigurationError(
            "Semantic memory store not configured. Use SemanticAgent::from_env() or set_store() to configure storage.".to_string()
        ))
    }

    /// Handle semantic knowledge update
    async fn handle_update(&self, parameters: Value) -> AgentResult<Value> {
        // Use semantic_store if available
        if let Some(store) = &self.semantic_store {
            // Parse the updated item from parameters
            let item: SemanticMemoryItem = serde_json::from_value(parameters.clone())
                .map_err(|e| AgentError::InvalidParameters(format!("Invalid item data: {e}")))?;

            // Update the item in the store
            let updated = store.update_item(item.clone()).await.map_err(|e| {
                AgentError::MemoryManagerError(format!("Failed to update item: {e}"))
            })?;

            if updated {
                log::info!("Semantic agent: Updated item {} in real storage", item.id);
                return Ok(serde_json::json!({
                    "success": true,
                    "item_id": item.id,
                    "message": "Semantic knowledge updated successfully"
                }));
            } else {
                log::warn!("Semantic agent: Item {} not found for update", item.id);
                return Ok(serde_json::json!({
                    "success": false,
                    "item_id": item.id,
                    "message": "Item not found"
                }));
            }
        }

        // Fallback to mock response if store not available
        let concept_id = parameters
            .get("concept_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::InvalidParameters("Missing 'concept_id' parameter".to_string())
            })?;

        // No store configured - return error instead of mock
        log::error!("Semantic agent: No store configured, cannot update knowledge");
        Err(AgentError::ConfigurationError(
            "Semantic memory store not configured. Use SemanticAgent::from_env() or set_store() to configure storage.".to_string()
        ))
    }

    /// Handle semantic knowledge deletion
    async fn handle_delete(&self, parameters: Value) -> AgentResult<Value> {
        // Use semantic_store if available
        if let Some(store) = &self.semantic_store {
            let item_id = parameters
                .get("id")
                .or_else(|| parameters.get("item_id"))
                .or_else(|| parameters.get("concept_id"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    AgentError::InvalidParameters("Missing 'id' or 'item_id' parameter".to_string())
                })?;

            let user_id = parameters
                .get("user_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    AgentError::InvalidParameters("Missing 'user_id' parameter".to_string())
                })?;

            // Delete the item from the store
            let deleted = store.delete_item(item_id, user_id).await.map_err(|e| {
                AgentError::MemoryManagerError(format!("Failed to delete item: {e}"))
            })?;

            if deleted {
                log::info!("Semantic agent: Deleted item {item_id} from real storage");
                return Ok(serde_json::json!({
                    "success": true,
                    "item_id": item_id,
                    "message": "Semantic knowledge deleted successfully"
                }));
            } else {
                log::warn!("Semantic agent: Item {item_id} not found for deletion");
                return Ok(serde_json::json!({
                    "success": false,
                    "item_id": item_id,
                    "message": "Item not found"
                }));
            }
        }

        // No store configured - return error instead of mock
        log::error!("Semantic agent: No store configured, cannot delete knowledge");
        Err(AgentError::ConfigurationError(
            "Semantic memory store not configured. Use SemanticAgent::from_env() or set_store() to configure storage.".to_string()
        ))
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

        log::info!("初始化语义记忆 Agent: {}", self.agent_id());

        // 如果配置了存储后端，验证连接并加载知识图谱统计信息
        if let Some(store) = &self.semantic_store {
            log::info!("验证语义记忆存储后端连接...");

            // 尝试查询语义记忆项以验证存储可用性
            let query = SemanticQuery {
                name_query: None,
                summary_query: None,
                tree_path_prefix: None,
                limit: Some(10),
            };

            // 使用 system 用户 ID 进行测试查询
            match store.query_items("system", query).await {
                Ok(items) => {
                    log::info!("成功连接到语义记忆存储，发现 {} 个知识项", items.len());

                    // 更新统计信息
                    let mut context = self.context.write().await;
                    context.stats.total_tasks = items.len() as u64;
                }
                Err(e) => {
                    log::warn!("查询语义记忆失败: {e}，将从空状态开始");
                }
            }
        } else {
            log::warn!("未配置语义记忆存储后端，Agent 将以只读模式运行");
        }

        // 初始化 Agent 上下文
        {
            let mut context = self.context.write().await;
            context.stats.active_tasks = 0;
            context.stats.successful_tasks = 0;
            context.stats.failed_tasks = 0;
        }

        self.initialized = true;
        log::info!("语义记忆 Agent 初始化完成");
        Ok(())
    }

    async fn shutdown(&mut self) -> CoordinationResult<()> {
        if !self.initialized {
            return Ok(());
        }

        log::info!("关闭语义记忆 Agent: {}", self.agent_id());

        // 如果有存储后端，记录最终状态
        if let Some(_store) = &self.semantic_store {
            log::info!("语义记忆存储后端已配置，所有知识项已通过 trait 方法持久化");

            // 记录最终统计信息
            let context = self.context.read().await;
            log::info!(
                "Agent 统计: 总任务={}, 完成={}, 失败={}, 活跃={}",
                context.stats.total_tasks,
                context.stats.successful_tasks,
                context.stats.failed_tasks,
                context.stats.active_tasks
            );
        }

        // 清理上下文
        {
            let mut context = self.context.write().await;
            context.current_task = None;
            context.stats.active_tasks = 0;
        }

        self.initialized = false;
        log::info!("语义记忆 Agent 已成功关闭");
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
