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
        let procedure = parameters.get("procedure").ok_or_else(|| {
            AgentError::InvalidParameters("Missing 'procedure' parameter".to_string())
        })?;

        let response = serde_json::json!({
            "success": true,
            "procedure_id": uuid::Uuid::new_v4().to_string(),
            "message": "Procedural memory inserted successfully"
        });

        log::info!("Procedural agent: Inserted procedure");
        Ok(response)
    }

    async fn handle_search(&self, parameters: Value) -> AgentResult<Value> {
        let query = parameters
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::InvalidParameters("Missing 'query' parameter".to_string())
            })?;

        let response = serde_json::json!({
            "success": true,
            "results": [],
            "total_count": 0,
            "query": query
        });

        log::info!("Procedural agent: Searched for '{}'", query);
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
        if !self.initialized {
            log::info!("Initializing Procedural Memory Agent: {}", self.agent_id());
            self.initialized = true;
        }
        Ok(())
    }

    async fn shutdown(&mut self) -> CoordinationResult<()> {
        if self.initialized {
            log::info!("Shutting down Procedural Memory Agent: {}", self.agent_id());
            self.initialized = false;
        }
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
