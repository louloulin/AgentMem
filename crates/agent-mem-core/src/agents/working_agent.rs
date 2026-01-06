//! Working Memory Agent
//!
//! This agent specializes in managing working memories - temporary, active memory contexts.

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
use agent_mem_traits::WorkingMemoryStore;

/// Working Memory Agent
///
/// Specializes in handling working memories - temporary, active memory contexts.
/// Uses trait-based storage (WorkingMemoryStore) to support multiple backends:
/// - PostgreSQL
/// - LibSQL
/// - MongoDB
/// - etc.
pub struct WorkingAgent {
    /// Base agent functionality
    base: BaseAgent,
    /// Agent context
    context: Arc<RwLock<AgentContext>>,
    /// Working memory store (trait-based, supports multiple backends)
    working_store: Option<Arc<dyn WorkingMemoryStore>>,
    /// Initialization status
    initialized: bool,
}

impl WorkingAgent {
    /// Create a new working memory agent
    pub fn new(agent_id: String) -> Self {
        let config = AgentConfig::new(agent_id, vec![MemoryType::Working], 15);
        let base = BaseAgent::new(config);
        let context = base.context();
        Self {
            base,
            context,
            working_store: None,
            initialized: false,
        }
    }

    /// Create with working memory store (trait-based, supports any backend)
    pub fn with_store(agent_id: String, store: Arc<dyn WorkingMemoryStore>) -> Self {
        let config = AgentConfig::new(agent_id, vec![MemoryType::Working], 15);
        let base = BaseAgent::new(config);
        let context = base.context();
        Self {
            base,
            context,
            working_store: Some(store),
            initialized: false,
        }
    }

    /// Set working memory store (trait-based, supports any backend)
    pub fn set_store(&mut self, store: Arc<dyn WorkingMemoryStore>) {
        self.working_store = Some(store);
    }

    async fn handle_insert(&self, parameters: Value) -> AgentResult<Value> {
        // Use actual working memory store if available
        if let Some(store) = &self.working_store {
            use agent_mem_traits::WorkingMemoryItem;
            use chrono::Utc;

            // Extract parameters
            let user_id = parameters
                .get("user_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    AgentError::InvalidParameters("Missing 'user_id' parameter".to_string())
                })?;

            let session_id = parameters
                .get("session_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    AgentError::InvalidParameters("Missing 'session_id' parameter".to_string())
                })?;

            let content = parameters
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    AgentError::InvalidParameters("Missing 'content' parameter".to_string())
                })?;

            let priority = parameters
                .get("priority")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;

            let expires_at = parameters
                .get("expires_at")
                .and_then(|v| v.as_str())
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc));

            let item = WorkingMemoryItem {
                id: uuid::Uuid::new_v4().to_string(),
                user_id: user_id.to_string(),
                agent_id: self.agent_id().to_string(),
                session_id: session_id.to_string(),
                content: content.to_string(),
                priority,
                expires_at,
                metadata: parameters
                    .get("metadata")
                    .cloned()
                    .unwrap_or(serde_json::json!({})),
                created_at: Utc::now(),
            };

            let created_item = store.add_item(item).await.map_err(|e| {
                AgentError::TaskExecutionError(format!("Failed to add working memory: {e}"))
            })?;

            let response = serde_json::json!({
                "success": true,
                "working_memory_id": created_item.id,
                "session_id": created_item.session_id,
                "message": "Working memory inserted successfully"
            });

            log::info!(
                "Working agent: Inserted working memory with ID {}",
                created_item.id
            );
            return Ok(response);
        }

        // No store configured - return error instead of mock
        log::error!("Working agent: No store configured, cannot insert item");
        Err(AgentError::ConfigurationError(
            "Working memory store not configured. Use WorkingAgent::from_env() or set_store() to configure storage.".to_string()
        ))
    }

    async fn handle_search(&self, parameters: Value) -> AgentResult<Value> {
        let session_id = parameters
            .get("session_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::InvalidParameters("Missing 'session_id' parameter".to_string())
            })?;

        // Use actual working memory store if available
        if let Some(store) = &self.working_store {
            let items = store.get_session_items(session_id).await.map_err(|e| {
                AgentError::TaskExecutionError(format!("Failed to get session items: {e}"))
            })?;

            let results: Vec<_> = items
                .iter()
                .map(|item| {
                    serde_json::json!({
                        "id": item.id,
                        "content": item.content,
                        "priority": item.priority,
                        "expires_at": item.expires_at.map(|dt| dt.to_rfc3339()),
                        "created_at": item.created_at.to_rfc3339()
                    })
                })
                .collect();

            let response = serde_json::json!({
                "success": true,
                "results": results,
                "total_count": items.len(),
                "session_id": session_id
            });

            log::info!(
                "Working agent: Found {} items for session {}",
                items.len(),
                session_id
            );
            return Ok(response);
        }

        // No store configured - return error instead of mock
        log::error!("Working agent: No store configured, cannot search items");
        Err(AgentError::ConfigurationError(
            "Working memory store not configured. Use WorkingAgent::from_env() or set_store() to configure storage.".to_string()
        ))
    }

    async fn handle_delete(&self, parameters: Value) -> AgentResult<Value> {
        let item_id = parameters
            .get("item_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::InvalidParameters("Missing 'item_id' parameter".to_string())
            })?;

        // Use actual working memory store if available
        if let Some(store) = &self.working_store {
            let deleted = store.remove_item(item_id).await.map_err(|e| {
                AgentError::TaskExecutionError(format!("Failed to delete item: {e}"))
            })?;

            if deleted {
                let response = serde_json::json!({
                    "success": true,
                    "item_id": item_id,
                    "message": "Working memory item deleted successfully"
                });
                log::info!("Working agent: Deleted item '{item_id}'");
                return Ok(response);
            } else {
                return Err(AgentError::InternalError(format!(
                    "Working memory item with ID '{item_id}' not found"
                )));
            }
        }

        // No store configured - return error instead of mock
        log::error!("Working agent: No store configured, cannot delete item");
        Err(AgentError::ConfigurationError(
            "Working memory store not configured. Use WorkingAgent::from_env() or set_store() to configure storage.".to_string()
        ))
    }
}

#[async_trait]
impl MemoryAgent for WorkingAgent {
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

        log::info!("初始化工作记忆 Agent: {}", self.agent_id());

        // 如果配置了存储后端，验证连接并加载工作记忆统计信息
        if let Some(store) = &self.working_store {
            log::info!("验证工作记忆存储后端连接...");

            // 尝试获取会话工作记忆项以验证存储可用性
            // 使用 system 会话 ID 进行测试查询
            match store.get_session_items("system").await {
                Ok(items) => {
                    log::info!("成功连接到工作记忆存储，发现 {} 个活跃项", items.len());

                    // 更新统计信息
                    let mut context = self.context.write().await;
                    context.stats.total_tasks = items.len() as u64;
                }
                Err(e) => {
                    log::warn!("查询工作记忆失败: {e}，将从空状态开始");
                }
            }
        } else {
            log::warn!("未配置工作记忆存储后端，Agent 将以只读模式运行");
        }

        // 初始化 Agent 上下文
        {
            let mut context = self.context.write().await;
            context.stats.active_tasks = 0;
            context.stats.successful_tasks = 0;
            context.stats.failed_tasks = 0;
        }

        self.initialized = true;
        log::info!("工作记忆 Agent 初始化完成");
        Ok(())
    }

    async fn shutdown(&mut self) -> CoordinationResult<()> {
        if !self.initialized {
            return Ok(());
        }

        log::info!("关闭工作记忆 Agent: {}", self.agent_id());

        // 如果有存储后端，记录最终状态
        if let Some(_store) = &self.working_store {
            log::info!("工作记忆存储后端已配置，所有工作记忆项已通过 trait 方法持久化");

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
        log::info!("工作记忆 Agent 已成功关闭");
        Ok(())
    }

    async fn execute_task(&mut self, task: TaskRequest) -> CoordinationResult<TaskResponse> {
        if !self.initialized {
            return Err(CoordinationError::InternalError(
                "Agent not initialized".to_string(),
            ));
        }

        let start_time = Instant::now();

        {
            let mut context = self.context.write().await;
            context.current_task = Some(task.clone());
            context.stats.active_tasks += 1;
        }

        let result = match task.operation.as_str() {
            "insert" => self.handle_insert(task.parameters).await,
            "search" => self.handle_search(task.parameters).await,
            "delete" => self.handle_delete(task.parameters).await,
            _ => Err(AgentError::InvalidParameters(format!(
                "Unknown operation: {}",
                task.operation
            ))),
        };

        let execution_time = start_time.elapsed();

        {
            let mut context = self.context.write().await;
            context.current_task = None;
            context.stats.active_tasks = context.stats.active_tasks.saturating_sub(1);
            context
                .stats
                .update_task_completion(result.is_ok(), execution_time.as_millis() as f64);
        }

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
        log::debug!("Working agent received message: {:?}", message.message_type);
        Ok(())
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
