//! Core Memory Agent
//!
//! This agent specializes in managing core memories - persistent memory blocks
//! that form the foundation of the agent's identity and context.

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
use agent_mem_traits::CoreMemoryStore;

/// Core Memory Agent
///
/// Specializes in handling core memories - persistent memory blocks that define
/// the agent's identity, persona, and fundamental context.
/// Uses trait-based storage (CoreMemoryStore) to support multiple backends:
/// - PostgreSQL
/// - LibSQL
/// - MongoDB
/// - etc.
pub struct CoreAgent {
    /// Base agent functionality
    base: BaseAgent,
    /// Agent context
    context: Arc<RwLock<AgentContext>>,
    /// Core memory store (trait-based, supports multiple backends)
    core_store: Option<Arc<dyn CoreMemoryStore>>,
    /// Initialization status
    initialized: bool,
}

impl CoreAgent {
    /// Create a new core memory agent
    ///
    /// **Note**: This creates an agent without a store configured.
    /// For production use, prefer `from_env()` or `with_store()`.
    pub fn new(agent_id: String) -> Self {
        let config = AgentConfig::new(
            agent_id,
            vec![MemoryType::Core],
            5, // max concurrent tasks (lower for core memory)
        );

        let base = BaseAgent::new(config);
        let context = base.context();

        Self {
            base,
            context,
            core_store: None,
            initialized: false,
        }
    }

    /// Create a new core memory agent with store from environment configuration
    ///
    /// This method automatically initializes the store based on environment variables:
    /// - `DATABASE_URL`: Full database connection string
    /// - `AGENTMEM_DB_PATH`: Path to LibSQL database file (default: "agentmem.db")
    /// - `AGENTMEM_DB_BACKEND`: Backend type ("postgres" or "libsql", default: "libsql")
    ///
    /// # Example
    ///
    /// ```no_run
    /// use agent_mem_core::agents::CoreAgent;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Automatically uses LibSQL with default path
    /// let agent = CoreAgent::from_env("agent1".to_string()).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn from_env(agent_id: String) -> agent_mem_traits::Result<Self> {
        use crate::config_env::create_stores_from_env;

        let stores = create_stores_from_env().await?;
        Ok(Self::with_store(agent_id, stores.core))
    }

    /// Create with core memory store (trait-based, supports any backend)
    pub fn with_store(agent_id: String, store: Arc<dyn CoreMemoryStore>) -> Self {
        let config = AgentConfig::new(
            agent_id,
            vec![MemoryType::Core],
            5,
        );

        let base = BaseAgent::new(config);
        let context = base.context();

        Self {
            base,
            context,
            core_store: Some(store),
            initialized: false,
        }
    }

    /// Set core memory store (trait-based, supports any backend)
    pub fn set_store(&mut self, store: Arc<dyn CoreMemoryStore>) {
        self.core_store = Some(store);
    }

    /// Create with custom configuration
    pub fn with_config(config: AgentConfig) -> Self {
        let base = BaseAgent::new(config);
        let context = base.context();

        Self {
            base,
            context,
            core_store: None,
            initialized: false,
        }
    }

    /// Handle core memory block creation
    async fn handle_create_block(&self, parameters: Value) -> AgentResult<Value> {
        let label = parameters
            .get("label")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::InvalidParameters("Missing 'label' parameter".to_string())
            })?;

        let content = parameters
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::InvalidParameters("Missing 'content' parameter".to_string())
            })?;

        let block_type = parameters
            .get("block_type")
            .and_then(|v| v.as_str())
            .unwrap_or("general");

        // Get user_id from parameters or use default
        let user_id = parameters
            .get("user_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default_user");

        // Use core_store if available, otherwise return mock response
        if let Some(store) = &self.core_store {
            use agent_mem_traits::CoreMemoryItem;
            use chrono::Utc;

            let now = Utc::now();
            let item = CoreMemoryItem {
                id: uuid::Uuid::new_v4().to_string(),
                user_id: user_id.to_string(),
                agent_id: self.agent_id().to_string(),
                key: label.to_string(),
                value: content.to_string(),
                category: block_type.to_string(),
                is_mutable: true,
                metadata: serde_json::json!({
                    "block_type": block_type,
                    "created_by": "core_agent"
                }),
                created_at: now,
                updated_at: now,
            };

            let created_item = store
                .set_value(item)
                .await
                .map_err(|e| AgentError::MemoryManagerError(e.to_string()))?;

            let response = serde_json::json!({
                "success": true,
                "block_id": created_item.id,
                "label": created_item.key,
                "content": created_item.value,
                "block_type": created_item.category,
                "message": "Core memory block created successfully"
            });

            log::info!("Core agent: Created memory block '{}' with ID {}", label, created_item.id);
            Ok(response)
        } else {
            // Fallback to mock response if no store is configured
            let response = serde_json::json!({
                "success": true,
                "block_id": uuid::Uuid::new_v4().to_string(),
                "label": label,
                "content": content,
                "block_type": block_type,
                "message": "Core memory block created successfully (mock)"
            });

            log::warn!("Core agent: No store configured, using mock response");
            Ok(response)
        }
    }

    /// Handle core memory block reading
    async fn handle_read_block(&self, parameters: Value) -> AgentResult<Value> {
        let block_id = parameters.get("block_id").and_then(|v| v.as_str());
        let label = parameters.get("label").and_then(|v| v.as_str());

        if block_id.is_none() && label.is_none() {
            return Err(AgentError::InvalidParameters(
                "Missing 'block_id' or 'label' parameter".to_string(),
            ));
        }

        // Get user_id from parameters or use default
        let user_id = parameters
            .get("user_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default_user");

        // Use core_store if available
        if let Some(store) = &self.core_store {
            // If label is provided, use it as key to retrieve
            if let Some(key) = label {
                let item_opt = store
                    .get_value(user_id, key)
                    .await
                    .map_err(|e| AgentError::MemoryManagerError(e.to_string()))?;

                if let Some(item) = item_opt {
                    let response = serde_json::json!({
                        "success": true,
                        "block": {
                            "id": item.id,
                            "label": item.key,
                            "content": item.value,
                            "block_type": item.category,
                            "created_at": item.created_at.to_rfc3339(),
                            "updated_at": item.updated_at.to_rfc3339()
                        }
                    });

                    log::info!("Core agent: Read memory block with label '{}'", key);
                    return Ok(response);
                } else {
                    return Err(AgentError::InternalError(format!(
                        "Core memory block with label '{}' not found",
                        key
                    )));
                }
            } else {
                // If only block_id is provided, we need to get all and filter
                // This is not efficient, but CoreMemoryStore doesn't have get_by_id
                return Err(AgentError::InvalidParameters(
                    "Retrieving by block_id is not supported, use 'label' instead".to_string(),
                ));
            }
        } else {
            // Fallback to mock response if no store is configured
            let response = serde_json::json!({
                "success": true,
                "block": {
                    "id": block_id.unwrap_or("unknown"),
                    "label": label.unwrap_or("unknown"),
                    "content": "Sample core memory content (mock)",
                    "block_type": "general",
                    "created_at": chrono::Utc::now().to_rfc3339(),
                    "updated_at": chrono::Utc::now().to_rfc3339()
                }
            });

            log::warn!("Core agent: No store configured, using mock response");
            Ok(response)
        }
    }

    /// Handle core memory block update
    async fn handle_update_block(&self, parameters: Value) -> AgentResult<Value> {
        let label = parameters
            .get("label")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::InvalidParameters("Missing 'label' parameter".to_string())
            })?;

        let content = parameters
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::InvalidParameters("Missing 'content' parameter".to_string())
            })?;

        // Get user_id from parameters or use default
        let user_id = parameters
            .get("user_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default_user");

        // Use core_store if available
        if let Some(store) = &self.core_store {
            let updated = store
                .update_value(user_id, label, content)
                .await
                .map_err(|e| AgentError::MemoryManagerError(e.to_string()))?;

            if updated {
                let response = serde_json::json!({
                    "success": true,
                    "label": label,
                    "content": content,
                    "message": "Core memory block updated successfully"
                });

                log::info!("Core agent: Updated memory block '{}'", label);
                Ok(response)
            } else {
                Err(AgentError::InternalError(format!(
                    "Core memory block with label '{}' not found",
                    label
                )))
            }
        } else {
            // Fallback to mock response if no store is configured
            let response = serde_json::json!({
                "success": true,
                "label": label,
                "content": content,
                "message": "Core memory block updated successfully (mock)"
            });

            log::warn!("Core agent: No store configured, using mock response");
            Ok(response)
        }
    }

    /// Handle core memory block deletion
    async fn handle_delete_block(&self, parameters: Value) -> AgentResult<Value> {
        let label = parameters
            .get("label")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::InvalidParameters("Missing 'label' parameter".to_string())
            })?;

        // Get user_id from parameters or use default
        let user_id = parameters
            .get("user_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default_user");

        // Use core_store if available
        if let Some(store) = &self.core_store {
            let deleted = store
                .delete_value(user_id, label)
                .await
                .map_err(|e| AgentError::MemoryManagerError(e.to_string()))?;

            if deleted {
                let response = serde_json::json!({
                    "success": true,
                    "label": label,
                    "message": "Core memory block deleted successfully"
                });

                log::info!("Core agent: Deleted memory block '{}'", label);
                Ok(response)
            } else {
                Err(AgentError::InternalError(format!(
                    "Core memory block with label '{}' not found",
                    label
                )))
            }
        } else {
            // Fallback to mock response if no store is configured
            let response = serde_json::json!({
                "success": true,
                "label": label,
                "message": "Core memory block deleted successfully (mock)"
            });

            log::warn!("Core agent: No store configured, using mock response");
            Ok(response)
        }
    }

    /// Handle core memory search
    async fn handle_search(&self, parameters: Value) -> AgentResult<Value> {
        let query = parameters
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::InvalidParameters("Missing 'query' parameter".to_string())
            })?;

        let block_type = parameters.get("block_type").and_then(|v| v.as_str());

        // Get user_id from parameters or use default
        let user_id = parameters
            .get("user_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default_user");

        // Use core_store if available
        if let Some(store) = &self.core_store {
            // Get items by category if block_type is specified, otherwise get all
            let items = if let Some(category) = block_type {
                store
                    .get_by_category(user_id, category)
                    .await
                    .map_err(|e| AgentError::MemoryManagerError(e.to_string()))?
            } else {
                store
                    .get_all(user_id)
                    .await
                    .map_err(|e| AgentError::MemoryManagerError(e.to_string()))?
            };

            // Simple text search in memory (filter by query in key or value)
            let query_lower = query.to_lowercase();
            let filtered_items: Vec<_> = items
                .into_iter()
                .filter(|item| {
                    item.key.to_lowercase().contains(&query_lower)
                        || item.value.to_lowercase().contains(&query_lower)
                })
                .collect();

            let results: Vec<_> = filtered_items
                .iter()
                .map(|item| {
                    serde_json::json!({
                        "id": item.id,
                        "label": item.key,
                        "content": item.value,
                        "block_type": item.category,
                        "created_at": item.created_at.to_rfc3339(),
                        "updated_at": item.updated_at.to_rfc3339()
                    })
                })
                .collect();

            let response = serde_json::json!({
                "success": true,
                "results": results,
                "total_count": filtered_items.len(),
                "query": query,
                "block_type": block_type
            });

            log::info!(
                "Core agent: Found {} results for query '{}' in block type: {:?}",
                filtered_items.len(),
                query,
                block_type
            );
            Ok(response)
        } else {
            // Fallback to mock response if no store is configured
            let response = serde_json::json!({
                "success": true,
                "results": [],
                "total_count": 0,
                "query": query,
                "block_type": block_type,
                "message": "No store configured (mock)"
            });

            log::warn!("Core agent: No store configured, using mock response");
            Ok(response)
        }
    }

    /// Handle memory compilation (render all blocks as context)
    async fn handle_compile(&self, parameters: Value) -> AgentResult<Value> {
        // Get user_id from parameters or use default
        let user_id = parameters
            .get("user_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default_user");

        // Use core_store if available
        if let Some(store) = &self.core_store {
            let items = store
                .get_all(user_id)
                .await
                .map_err(|e| AgentError::MemoryManagerError(e.to_string()))?;

            // Compile all blocks into a single context string
            let mut compiled_parts = Vec::new();
            let mut total_chars = 0;

            for item in &items {
                let block_text = format!(
                    "[{}] {}: {}",
                    item.category, item.key, item.value
                );
                total_chars += block_text.len();
                compiled_parts.push(block_text);
            }

            let compiled_memory = compiled_parts.join("\n");

            let response = serde_json::json!({
                "success": true,
                "compiled_memory": compiled_memory,
                "block_count": items.len(),
                "total_characters": total_chars
            });

            log::info!(
                "Core agent: Compiled {} core memory blocks ({} chars)",
                items.len(),
                total_chars
            );
            Ok(response)
        } else {
            // Fallback to mock response if no store is configured
            let response = serde_json::json!({
                "success": true,
                "compiled_memory": "Compiled core memory context (mock)",
                "block_count": 0,
                "total_characters": 0
            });

            log::warn!("Core agent: No store configured, using mock response");
            Ok(response)
        }
    }
}

#[async_trait]
impl MemoryAgent for CoreAgent {
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

        log::info!("初始化核心记忆 Agent: {}", self.agent_id());

        // 如果配置了存储后端，加载现有的核心记忆块
        if let Some(store) = &self.core_store {
            log::info!("从存储后端加载核心记忆块...");

            // 尝试加载核心记忆项以验证存储可用性
            // 使用 system 用户 ID 进行测试查询
            match store.get_all("system").await {
                Ok(items) => {
                    log::info!(
                        "成功连接到核心记忆存储，发现 {} 个记忆项",
                        items.len()
                    );

                    // 更新统计信息
                    let mut context = self.context.write().await;
                    context.stats.total_tasks = items.len() as u64;
                }
                Err(e) => {
                    log::warn!("加载核心记忆项失败: {}，将从空状态开始", e);
                }
            }
        } else {
            log::warn!("未配置核心记忆存储后端，Agent 将以只读模式运行");
        }

        // 初始化 Agent 上下文
        {
            let mut context = self.context.write().await;
            context.stats.active_tasks = 0;
            context.stats.successful_tasks = 0;
            context.stats.failed_tasks = 0;
        }

        self.initialized = true;
        log::info!("核心记忆 Agent 初始化完成");
        Ok(())
    }

    async fn shutdown(&mut self) -> CoordinationResult<()> {
        if !self.initialized {
            return Ok(());
        }

        log::info!("关闭核心记忆 Agent: {}", self.agent_id());

        // 如果有存储后端，确保所有待处理的操作都已完成
        if let Some(_store) = &self.core_store {
            log::info!("核心记忆存储后端已配置，所有操作已通过 trait 方法持久化");

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
        log::info!("核心记忆 Agent 已成功关闭");
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
            "create_block" => self.handle_create_block(task.parameters).await,
            "read_block" => self.handle_read_block(task.parameters).await,
            "update_block" => self.handle_update_block(task.parameters).await,
            "delete_block" => self.handle_delete_block(task.parameters).await,
            "search" => self.handle_search(task.parameters).await,
            "compile" => self.handle_compile(task.parameters).await,
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
        log::debug!("Core agent received message: {:?}", message.message_type);

        match message.message_type {
            crate::coordination::MessageType::TaskRequest => Ok(()),
            crate::coordination::MessageType::HealthCheck => Ok(()),
            _ => {
                log::warn!(
                    "Core agent received unknown message type: {:?}",
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
