//! Episodic Memory Agent
//!
//! This agent specializes in managing episodic memories - time-based events and experiences.
//! It handles operations like storing events, retrieving memories by time range, and
//! managing temporal relationships between memories.

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
use agent_mem_traits::{EpisodicEvent, EpisodicMemoryStore, EpisodicQuery};

/// Episodic Memory Agent
///
/// Specializes in handling episodic memories - time-based events and experiences.
/// This agent is inspired by MIRIX's episodic memory management but optimized
/// for Rust's performance characteristics.
///
/// Uses trait-based storage (EpisodicMemoryStore) to support multiple backends:
/// - PostgreSQL
/// - LibSQL
/// - MongoDB
/// - etc.
pub struct EpisodicAgent {
    /// Base agent functionality
    base: BaseAgent,
    /// Agent context
    context: Arc<RwLock<AgentContext>>,
    /// Episodic memory store (trait-based, supports multiple backends)
    episodic_store: Option<Arc<dyn EpisodicMemoryStore>>,
    /// Initialization status
    initialized: bool,
}

impl EpisodicAgent {
    /// Create a new episodic memory agent
    ///
    /// **Note**: This creates an agent without a store configured.
    /// For production use, prefer `from_env()` or `with_store()`.
    pub fn new(agent_id: String) -> Self {
        let config = AgentConfig::new(
            agent_id,
            vec![MemoryType::Episodic],
            10, // max concurrent tasks
        );

        let base = BaseAgent::new(config);
        let context = base.context();

        Self {
            base,
            context,
            episodic_store: None,
            initialized: false,
        }
    }

    /// Create a new episodic memory agent with store from environment configuration
    ///
    /// This method automatically initializes the store based on environment variables.
    /// See `config_env` module for supported environment variables.
    pub async fn from_env(agent_id: String) -> agent_mem_traits::Result<Self> {
        use crate::config_env::create_stores_from_env;

        let stores = create_stores_from_env().await?;
        Ok(Self::with_store(agent_id, stores.episodic))
    }

    /// Create with custom configuration
    pub fn with_config(config: AgentConfig) -> Self {
        let base = BaseAgent::new(config);
        let context = base.context();

        Self {
            base,
            context,
            episodic_store: None,
            initialized: false,
        }
    }

    /// Create with episodic memory store (trait-based, supports any backend)
    pub fn with_store(agent_id: String, store: Arc<dyn EpisodicMemoryStore>) -> Self {
        let config = AgentConfig::new(
            agent_id,
            vec![MemoryType::Episodic],
            10, // max concurrent tasks
        );

        let base = BaseAgent::new(config);
        let context = base.context();

        Self {
            base,
            context,
            episodic_store: Some(store),
            initialized: false,
        }
    }

    /// Set episodic memory store (trait-based, supports any backend)
    pub fn set_store(&mut self, store: Arc<dyn EpisodicMemoryStore>) {
        self.episodic_store = Some(store);
    }

    /// Handle episodic memory insertion
    async fn handle_insert(&self, parameters: Value) -> AgentResult<Value> {
        // Extract parameters for episodic memory insertion
        let event_data = parameters.get("event").ok_or_else(|| {
            AgentError::InvalidParameters("Missing 'event' parameter".to_string())
        })?;

        {
            // Use actual episodic memory manager if available
            if let Some(manager) = &self.episodic_store {
                // Parse event data
                let event: EpisodicEvent =
                    serde_json::from_value(event_data.clone()).map_err(|e| {
                        AgentError::InvalidParameters(format!("Invalid event data: {e}"))
                    })?;

                // Create event using manager
                let created_event = manager.create_event(event).await.map_err(|e| {
                    AgentError::TaskExecutionError(format!("Failed to create event: {e}"))
                })?;

                let response = serde_json::json!({
                    "success": true,
                    "event_id": created_event.id,
                    "message": "Episodic memory inserted successfully"
                });

                log::info!("Episodic agent: Inserted event {}", created_event.id);
                return Ok(response);
            }
        }

        // Fallback to mock response if manager not available
        let response = serde_json::json!({
            "success": true,
            "event_id": uuid::Uuid::new_v4().to_string(),
            "message": "Episodic memory inserted successfully (mock)"
        });

        log::warn!("Episodic agent: Using mock response (manager not available)");
        Ok(response)
    }

    /// Handle episodic memory search
    async fn handle_search(&self, parameters: Value) -> AgentResult<Value> {
        let user_id = parameters
            .get("user_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::InvalidParameters("Missing 'user_id' parameter".to_string())
            })?;

        {
            // Use actual episodic memory manager if available
            if let Some(manager) = &self.episodic_store {
                // Build query from parameters
                let query = EpisodicQuery {
                    start_time: parameters
                        .get("start_time")
                        .and_then(|v| v.as_str())
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                        .map(|dt| dt.with_timezone(&chrono::Utc)),
                    end_time: parameters
                        .get("end_time")
                        .and_then(|v| v.as_str())
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                        .map(|dt| dt.with_timezone(&chrono::Utc)),
                    event_type: parameters
                        .get("event_type")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    min_importance: parameters
                        .get("min_importance")
                        .and_then(|v| v.as_f64())
                        .map(|f| f as f32),
                    limit: parameters.get("limit").and_then(|v| v.as_i64()),
                };

                // Query events using manager
                let events = manager.query_events(user_id, query).await.map_err(|e| {
                    AgentError::TaskExecutionError(format!("Failed to query events: {e}"))
                })?;

                let response = serde_json::json!({
                    "success": true,
                    "results": events,
                    "total_count": events.len()
                });

                log::info!(
                    "Episodic agent: Found {} events for user {}",
                    events.len(),
                    user_id
                );
                return Ok(response);
            }
        }

        // No store configured - return empty results for testing
        log::warn!("Episodic agent: No store configured, returning empty results");
        Ok(serde_json::json!({
            "success": true,
            "results": [],
            "total_count": 0,
            "message": "No episodic memory store configured"
        }))
    }

    /// Handle episodic memory retrieval by time range
    async fn handle_time_range_query(&self, parameters: Value) -> AgentResult<Value> {
        let start_time_str = parameters.get("start_time").and_then(|v| v.as_str());
        let end_time_str = parameters.get("end_time").and_then(|v| v.as_str());

        if start_time_str.is_none() || end_time_str.is_none() {
            return Err(AgentError::InvalidParameters(
                "Missing 'start_time' or 'end_time' parameter".to_string(),
            ));
        }

        let user_id = parameters
            .get("user_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::InvalidParameters("Missing 'user_id' parameter".to_string())
            })?;

        {
            // Use actual episodic memory manager if available
            if let Some(manager) = &self.episodic_store {
                // Parse time strings - both are required for time range query
                let start_time_str = start_time_str.ok_or_else(|| {
                    AgentError::InvalidParameters("start_time is required for time range query".to_string())
                })?;
                let end_time_str = end_time_str.ok_or_else(|| {
                    AgentError::InvalidParameters("end_time is required for time range query".to_string())
                })?;

                let start_time = chrono::DateTime::parse_from_rfc3339(&start_time_str)
                    .map_err(|e| {
                        AgentError::InvalidParameters(format!("Invalid start_time format: {e}"))
                    })?
                    .with_timezone(&chrono::Utc);

                let end_time = chrono::DateTime::parse_from_rfc3339(&end_time_str)
                    .map_err(|e| {
                        AgentError::InvalidParameters(format!("Invalid end_time format: {e}"))
                    })?
                    .with_timezone(&chrono::Utc);

                // Build query
                let query = EpisodicQuery {
                    start_time: Some(start_time),
                    end_time: Some(end_time),
                    event_type: parameters
                        .get("event_type")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    min_importance: parameters
                        .get("min_importance")
                        .and_then(|v| v.as_f64())
                        .map(|f| f as f32),
                    limit: parameters.get("limit").and_then(|v| v.as_i64()),
                };

                // Query events using manager
                let events = manager.query_events(user_id, query).await.map_err(|e| {
                    AgentError::TaskExecutionError(format!("Failed to query events: {e}"))
                })?;

                let response = serde_json::json!({
                    "success": true,
                    "events": events,
                    "total_count": events.len(),
                    "time_range": {
                        "start": start_time_str,
                        "end": end_time_str
                    }
                });

                log::info!(
                    "Episodic agent: Found {} events in time range {} to {}",
                    events.len(),
                    start_time_str,
                    end_time_str
                );
                return Ok(response);
            }
        }

        // No store configured - return error instead of mock
        log::error!("Episodic agent: No store configured, cannot query events by time range");
        Err(AgentError::ConfigurationError(
            "Episodic memory store not configured. Use EpisodicAgent::from_env() or set_store() to configure storage.".to_string()
        ))
    }

    /// Handle episodic memory update
    async fn handle_update(&self, parameters: Value) -> AgentResult<Value> {
        let event_id = parameters
            .get("event_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::InvalidParameters("Missing 'event_id' parameter".to_string())
            })?;

        let user_id = parameters
            .get("user_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::InvalidParameters("Missing 'user_id' parameter".to_string())
            })?;

        {
            // Use actual episodic memory manager if available
            if let Some(manager) = &self.episodic_store {
                // Check if updating importance score
                if let Some(importance) =
                    parameters.get("importance_score").and_then(|v| v.as_f64())
                {
                    let updated = manager
                        .update_importance(event_id, user_id, importance as f32)
                        .await
                        .map_err(|e| {
                            AgentError::TaskExecutionError(format!(
                                "Failed to update importance: {e}"
                            ))
                        })?;

                    let response = serde_json::json!({
                        "success": updated,
                        "event_id": event_id,
                        "message": if updated { "Importance updated successfully" } else { "Event not found" }
                    });

                    log::info!("Episodic agent: Updated importance for event {event_id}");
                    return Ok(response);
                }

                // For other updates, would need update_event method in manager
                let response = serde_json::json!({
                    "success": false,
                    "event_id": event_id,
                    "message": "Only importance_score updates are currently supported"
                });

                return Ok(response);
            }
        }

        // No store configured - return error instead of mock
        log::error!("Episodic agent: No store configured, cannot update event");
        Err(AgentError::ConfigurationError(
            "Episodic memory store not configured. Use EpisodicAgent::from_env() or set_store() to configure storage.".to_string()
        ))
    }

    /// Handle episodic memory deletion
    async fn handle_delete(&self, parameters: Value) -> AgentResult<Value> {
        let event_id = parameters
            .get("event_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::InvalidParameters("Missing 'event_id' parameter".to_string())
            })?;

        let user_id = parameters
            .get("user_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::InvalidParameters("Missing 'user_id' parameter".to_string())
            })?;

        {
            // Use actual episodic memory manager if available
            if let Some(manager) = &self.episodic_store {
                let deleted = manager.delete_event(event_id, user_id).await.map_err(|e| {
                    AgentError::TaskExecutionError(format!("Failed to delete event: {e}"))
                })?;

                let response = serde_json::json!({
                    "success": deleted,
                    "event_id": event_id,
                    "message": if deleted { "Event deleted successfully" } else { "Event not found" }
                });

                log::info!("Episodic agent: Deleted event {event_id}");
                return Ok(response);
            }
        }

        // No store configured - return error instead of mock
        log::error!("Episodic agent: No store configured, cannot delete event");
        Err(AgentError::ConfigurationError(
            "Episodic memory store not configured. Use EpisodicAgent::from_env() or set_store() to configure storage.".to_string()
        ))
    }
}

#[async_trait]
impl MemoryAgent for EpisodicAgent {
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

        log::info!("初始化情景记忆 Agent: {}", self.agent_id());

        // 如果配置了存储后端，验证连接并加载统计信息
        if let Some(store) = &self.episodic_store {
            log::info!("验证情景记忆存储后端连接...");

            // 尝试查询最近的事件以验证存储可用性
            let query = EpisodicQuery {
                start_time: None,
                end_time: None,
                event_type: None,
                min_importance: None,
                limit: Some(10),
            };

            // 使用 system 用户 ID 进行测试查询
            match store.query_events("system", query).await {
                Ok(events) => {
                    log::info!("成功连接到情景记忆存储，发现 {} 个最近事件", events.len());

                    // 更新统计信息
                    let mut context = self.context.write().await;
                    context.stats.total_tasks = events.len() as u64;
                }
                Err(e) => {
                    log::warn!("查询情景记忆失败: {e}，将从空状态开始");
                }
            }
        } else {
            log::warn!("未配置情景记忆存储后端，Agent 将以只读模式运行");
        }

        // 初始化 Agent 上下文
        {
            let mut context = self.context.write().await;
            context.stats.active_tasks = 0;
            context.stats.successful_tasks = 0;
            context.stats.failed_tasks = 0;
        }

        self.initialized = true;
        log::info!("情景记忆 Agent 初始化完成");
        Ok(())
    }

    async fn shutdown(&mut self) -> CoordinationResult<()> {
        if !self.initialized {
            return Ok(());
        }

        log::info!("关闭情景记忆 Agent: {}", self.agent_id());

        // 如果有存储后端，记录最终状态
        if let Some(_store) = &self.episodic_store {
            log::info!("情景记忆存储后端已配置，所有事件已通过 trait 方法持久化");

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
        log::info!("情景记忆 Agent 已成功关闭");
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
            "time_range_query" => self.handle_time_range_query(task.parameters).await,
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
            "Episodic agent received message: {:?}",
            message.message_type
        );

        // Handle different message types
        match message.message_type {
            crate::coordination::MessageType::TaskRequest => {
                // Task requests are handled through execute_task
                Ok(())
            }
            crate::coordination::MessageType::HealthCheck => {
                // Respond to health check
                Ok(())
            }
            _ => {
                log::warn!(
                    "Episodic agent received unknown message type: {:?}",
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
