//! Procedural Memory Agent
//!
//! This agent specializes in managing procedural memories - step-by-step procedures and workflows.

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
use agent_mem_traits::ProceduralMemoryStore;

/// Procedural Memory Agent
///
/// Specializes in handling procedural memories - skills, workflows, and executable procedures.
/// Uses trait-based storage (ProceduralMemoryStore) to support multiple backends:
/// - PostgreSQL
/// - LibSQL
/// - MongoDB
/// - etc.
pub struct ProceduralAgent {
    /// Base agent functionality
    base: BaseAgent,
    /// Agent context
    context: Arc<RwLock<AgentContext>>,
    /// Procedural memory store (trait-based, supports multiple backends)
    procedural_store: Option<Arc<dyn ProceduralMemoryStore>>,
    /// Initialization status
    initialized: bool,
}

impl ProceduralAgent {
    /// Create a new procedural memory agent
    pub fn new(agent_id: String) -> Self {
        let config = AgentConfig::new(agent_id, vec![MemoryType::Procedural], 10);
        let base = BaseAgent::new(config);
        let context = base.context();
        Self {
            base,
            context,
            procedural_store: None,
            initialized: false,
        }
    }

    /// Create with procedural memory store (trait-based, supports any backend)
    pub fn with_store(agent_id: String, store: Arc<dyn ProceduralMemoryStore>) -> Self {
        let config = AgentConfig::new(agent_id, vec![MemoryType::Procedural], 10);
        let base = BaseAgent::new(config);
        let context = base.context();
        Self {
            base,
            context,
            procedural_store: Some(store),
            initialized: false,
        }
    }

    /// Set procedural memory store (trait-based, supports any backend)
    pub fn set_store(&mut self, store: Arc<dyn ProceduralMemoryStore>) {
        self.procedural_store = Some(store);
    }

    async fn handle_insert(&self, parameters: Value) -> AgentResult<Value> {
        // Use actual procedural memory store if available
        if let Some(store) = &self.procedural_store {
            use agent_mem_traits::ProceduralMemoryItem;
            use chrono::Utc;

            // Extract parameters
            let user_id = parameters
                .get("user_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    AgentError::InvalidParameters("Missing 'user_id' parameter".to_string())
                })?;

            // ✅ 修复: 从参数获取 organization_id 而非硬编码
            let organization_id = parameters
                .get("organization_id")
                .and_then(|v| v.as_str())
                .unwrap_or("default");

            let skill_name = parameters
                .get("skill_name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    AgentError::InvalidParameters("Missing 'skill_name' parameter".to_string())
                })?;

            let description = parameters
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let steps: Vec<String> = parameters
                .get("steps")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            let now = Utc::now();
            let item = ProceduralMemoryItem {
                id: uuid::Uuid::new_v4().to_string(),
                organization_id: organization_id.to_string(),
                user_id: user_id.to_string(),
                agent_id: self.agent_id().to_string(),
                skill_name: skill_name.to_string(),
                description: description.to_string(),
                steps,
                success_rate: 0.0,
                execution_count: 0,
                metadata: parameters
                    .get("metadata")
                    .cloned()
                    .unwrap_or(serde_json::json!({})),
                created_at: now,
                updated_at: now,
            };

            let created_item = store.create_item(item).await.map_err(|e| {
                AgentError::TaskExecutionError(format!("Failed to create procedure: {e}"))
            })?;

            let response = serde_json::json!({
                "success": true,
                "procedure_id": created_item.id,
                "skill_name": created_item.skill_name,
                "message": "Procedural memory inserted successfully"
            });

            log::info!(
                "Procedural agent: Inserted procedure '{}' with ID {}",
                created_item.skill_name,
                created_item.id
            );
            return Ok(response);
        }

        // Fallback to mock response if store not available
        let response = serde_json::json!({
            "success": true,
            "procedure_id": uuid::Uuid::new_v4().to_string(),
            "message": "Procedural memory inserted successfully (mock)"
        });

        log::warn!("Procedural agent: Using mock response (store not available)");
        Ok(response)
    }

    async fn handle_search(&self, parameters: Value) -> AgentResult<Value> {
        let user_id = parameters
            .get("user_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::InvalidParameters("Missing 'user_id' parameter".to_string())
            })?;

        // Use actual procedural memory store if available
        if let Some(store) = &self.procedural_store {
            use agent_mem_traits::ProceduralQuery;

            // Build query from parameters
            let query = ProceduralQuery {
                skill_name_pattern: parameters
                    .get("skill_name_pattern")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                min_success_rate: parameters
                    .get("min_success_rate")
                    .and_then(|v| v.as_f64())
                    .map(|f| f as f32),
                limit: parameters.get("limit").and_then(|v| v.as_i64()),
            };

            let items = store.query_items(user_id, query).await.map_err(|e| {
                AgentError::TaskExecutionError(format!("Failed to query procedures: {e}"))
            })?;

            let results: Vec<_> = items
                .iter()
                .map(|item| {
                    serde_json::json!({
                        "id": item.id,
                        "skill_name": item.skill_name,
                        "description": item.description,
                        "steps": item.steps,
                        "success_rate": item.success_rate,
                        "execution_count": item.execution_count,
                        "created_at": item.created_at.to_rfc3339(),
                        "updated_at": item.updated_at.to_rfc3339()
                    })
                })
                .collect();

            let response = serde_json::json!({
                "success": true,
                "results": results,
                "total_count": items.len()
            });

            log::info!(
                "Procedural agent: Found {} procedures for user {}",
                items.len(),
                user_id
            );
            return Ok(response);
        }

        // Fallback to mock response if store not available
        let response = serde_json::json!({
            "success": true,
            "results": [],
            "total_count": 0,
            "message": "Mock response (store not available)"
        });

        log::warn!("Procedural agent: Using mock response (store not available)");
        Ok(response)
    }

    async fn handle_update(&self, parameters: Value) -> AgentResult<Value> {
        let user_id = parameters
            .get("user_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::InvalidParameters("Missing 'user_id' parameter".to_string())
            })?;

        let item_id = parameters
            .get("item_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::InvalidParameters("Missing 'item_id' parameter".to_string())
            })?;

        // Use actual procedural memory store if available
        if let Some(store) = &self.procedural_store {
            // Get existing item
            let existing_item = store
                .get_item(item_id, user_id)
                .await
                .map_err(|e| {
                    AgentError::TaskExecutionError(format!("Failed to get procedure: {e}"))
                })?
                .ok_or_else(|| {
                    AgentError::InternalError(format!("Procedure with ID '{item_id}' not found"))
                })?;

            // Update fields
            use agent_mem_traits::ProceduralMemoryItem;
            use chrono::Utc;

            let updated_item = ProceduralMemoryItem {
                id: existing_item.id.clone(),
                organization_id: existing_item.organization_id.clone(),
                user_id: existing_item.user_id.clone(),
                agent_id: existing_item.agent_id.clone(),
                skill_name: parameters
                    .get("skill_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&existing_item.skill_name)
                    .to_string(),
                description: parameters
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&existing_item.description)
                    .to_string(),
                steps: parameters
                    .get("steps")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or(existing_item.steps.clone()),
                success_rate: existing_item.success_rate,
                execution_count: existing_item.execution_count,
                metadata: parameters
                    .get("metadata")
                    .cloned()
                    .unwrap_or(existing_item.metadata.clone()),
                created_at: existing_item.created_at,
                updated_at: Utc::now(),
            };

            let updated = store.update_item(updated_item).await.map_err(|e| {
                AgentError::TaskExecutionError(format!("Failed to update procedure: {e}"))
            })?;

            if updated {
                let response = serde_json::json!({
                    "success": true,
                    "item_id": item_id,
                    "message": "Procedural memory updated successfully"
                });
                log::info!("Procedural agent: Updated procedure '{item_id}'");
                return Ok(response);
            } else {
                return Err(AgentError::InternalError(format!(
                    "Failed to update procedure '{item_id}'"
                )));
            }
        }

        // Fallback to mock response if store not available
        let response = serde_json::json!({
            "success": true,
            "item_id": item_id,
            "message": "Procedural memory updated successfully (mock)"
        });

        log::warn!("Procedural agent: Using mock response (store not available)");
        Ok(response)
    }

    async fn handle_delete(&self, parameters: Value) -> AgentResult<Value> {
        let user_id = parameters
            .get("user_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::InvalidParameters("Missing 'user_id' parameter".to_string())
            })?;

        let item_id = parameters
            .get("item_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::InvalidParameters("Missing 'item_id' parameter".to_string())
            })?;

        // Use actual procedural memory store if available
        if let Some(store) = &self.procedural_store {
            let deleted = store.delete_item(item_id, user_id).await.map_err(|e| {
                AgentError::TaskExecutionError(format!("Failed to delete procedure: {e}"))
            })?;

            if deleted {
                let response = serde_json::json!({
                    "success": true,
                    "item_id": item_id,
                    "message": "Procedural memory deleted successfully"
                });
                log::info!("Procedural agent: Deleted procedure '{item_id}'");
                return Ok(response);
            } else {
                return Err(AgentError::InternalError(format!(
                    "Procedure with ID '{item_id}' not found"
                )));
            }
        }

        // Fallback to mock response if store not available
        let response = serde_json::json!({
            "success": true,
            "item_id": item_id,
            "message": "Procedural memory deleted successfully (mock)"
        });

        log::warn!("Procedural agent: Using mock response (store not available)");
        Ok(response)
    }
}

#[async_trait]
impl MemoryAgent for ProceduralAgent {
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

        log::info!("初始化程序记忆 Agent: {}", self.agent_id());

        // 如果配置了存储后端，验证连接并加载程序/技能统计信息
        if let Some(store) = &self.procedural_store {
            log::info!("验证程序记忆存储后端连接...");

            // 尝试查询程序记忆项以验证存储可用性
            let query = agent_mem_traits::ProceduralQuery {
                skill_name_pattern: None,
                min_success_rate: None,
                limit: Some(10),
            };

            // 使用 system 用户 ID 进行测试查询
            match store.query_items("system", query).await {
                Ok(items) => {
                    log::info!("成功连接到程序记忆存储，发现 {} 个技能/程序", items.len());

                    // 更新统计信息
                    let mut context = self.context.write().await;
                    context.stats.total_tasks = items.len() as u64;
                }
                Err(e) => {
                    log::warn!("查询程序记忆失败: {e}，将从空状态开始");
                }
            }
        } else {
            log::warn!("未配置程序记忆存储后端，Agent 将以只读模式运行");
        }

        // 初始化 Agent 上下文
        {
            let mut context = self.context.write().await;
            context.stats.active_tasks = 0;
            context.stats.successful_tasks = 0;
            context.stats.failed_tasks = 0;
        }

        self.initialized = true;
        log::info!("程序记忆 Agent 初始化完成");
        Ok(())
    }

    async fn shutdown(&mut self) -> CoordinationResult<()> {
        if !self.initialized {
            return Ok(());
        }

        log::info!("关闭程序记忆 Agent: {}", self.agent_id());

        // 如果有存储后端，记录最终状态
        if let Some(_store) = &self.procedural_store {
            log::info!("程序记忆存储后端已配置，所有技能/程序已通过 trait 方法持久化");

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
        log::info!("程序记忆 Agent 已成功关闭");
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
            "update" => self.handle_update(task.parameters).await,
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
        log::debug!(
            "Procedural agent received message: {:?}",
            message.message_type
        );
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
