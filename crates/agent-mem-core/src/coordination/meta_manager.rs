//! MetaMemoryManager - Multi-Agent Coordination Engine
//!
//! This module implements the central coordinator for AgentMem's multi-agent architecture,
//! inspired by MIRIX's agent management system but optimized for Rust's performance characteristics.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot, RwLock};
use tokio::time::timeout;
use uuid::Uuid;

use crate::types::MemoryType;

/// Configuration for the MetaMemoryManager
#[derive(Debug, Clone)]
pub struct MetaMemoryConfig {
    /// Maximum number of concurrent tasks per agent
    pub max_concurrent_tasks: usize,
    /// Task timeout duration
    pub task_timeout: Duration,
    /// Health check interval
    pub health_check_interval: Duration,
    /// Maximum retry attempts for failed tasks
    pub max_retry_attempts: usize,
    /// Load balancing strategy
    pub load_balancing_strategy: LoadBalancingStrategy,
}

impl Default for MetaMemoryConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 10,
            task_timeout: Duration::from_secs(30),
            health_check_interval: Duration::from_secs(10),
            max_retry_attempts: 3,
            load_balancing_strategy: LoadBalancingStrategy::RoundRobin,
        }
    }
}

/// Load balancing strategies for task distribution
#[derive(Debug, Clone, PartialEq)]
pub enum LoadBalancingStrategy {
    /// Simple round-robin distribution
    RoundRobin,
    /// Distribute based on current load
    LeastLoaded,
    /// Distribute based on agent specialization
    SpecializationBased,
}

/// Types of messages that can be sent between agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    /// Task execution request
    TaskRequest,
    /// Task execution response
    TaskResponse,
    /// Health check ping
    HealthCheck,
    /// Agent status update
    StatusUpdate,
    /// Coordination message
    Coordination,
}

/// Message structure for inter-agent communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    /// Unique message ID
    pub id: String,
    /// Message type
    pub message_type: MessageType,
    /// Source agent ID
    pub from_agent: String,
    /// Target agent ID
    pub to_agent: String,
    /// Message payload
    pub payload: serde_json::Value,
    /// Message timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Message priority (0-10, higher is more urgent)
    pub priority: u8,
}

impl AgentMessage {
    /// Create a new agent message
    pub fn new(
        message_type: MessageType,
        from_agent: String,
        to_agent: String,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            message_type,
            from_agent,
            to_agent,
            payload,
            timestamp: chrono::Utc::now(),
            priority: 5, // Default priority
        }
    }

    /// Set message priority
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority.min(10);
        self
    }
}

/// Task request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRequest {
    /// Unique task ID
    pub task_id: String,
    /// Memory type this task operates on
    pub memory_type: MemoryType,
    /// Operation type (create, read, update, delete, search)
    pub operation: String,
    /// Task parameters
    pub parameters: serde_json::Value,
    /// Task priority
    pub priority: u8,
    /// Maximum execution time
    pub timeout: Option<Duration>,
    /// Retry count
    pub retry_count: usize,
}

impl TaskRequest {
    /// Create a new task request
    pub fn new(memory_type: MemoryType, operation: String, parameters: serde_json::Value) -> Self {
        Self {
            task_id: Uuid::new_v4().to_string(),
            memory_type,
            operation,
            parameters,
            priority: 5,
            timeout: None,
            retry_count: 0,
        }
    }

    /// Set task priority
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority.min(10);
        self
    }

    /// Set task timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

/// Task response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResponse {
    /// Task ID this response corresponds to
    pub task_id: String,
    /// Whether the task was successful
    pub success: bool,
    /// Response data
    pub data: Option<serde_json::Value>,
    /// Error message if task failed
    pub error: Option<String>,
    /// Task execution time
    pub execution_time: Duration,
    /// Agent that executed the task
    pub executed_by: String,
}

impl TaskResponse {
    /// Create a successful task response
    pub fn success(
        task_id: String,
        data: serde_json::Value,
        execution_time: Duration,
        executed_by: String,
    ) -> Self {
        Self {
            task_id,
            success: true,
            data: Some(data),
            error: None,
            execution_time,
            executed_by,
        }
    }

    /// Create a failed task response
    pub fn error(
        task_id: String,
        error: String,
        execution_time: Duration,
        executed_by: String,
    ) -> Self {
        Self {
            task_id,
            success: false,
            data: None,
            error: Some(error),
            execution_time,
            executed_by,
        }
    }
}

/// Agent status information
#[derive(Debug, Clone)]
pub struct AgentStatus {
    /// Agent ID
    pub agent_id: String,
    /// Whether agent is healthy
    pub is_healthy: bool,
    /// Current load (number of active tasks)
    pub current_load: usize,
    /// Maximum capacity
    pub max_capacity: usize,
    /// Last health check time
    pub last_health_check: Instant,
    /// Total tasks processed
    pub total_tasks_processed: u64,
    /// Average task execution time
    pub avg_execution_time: Duration,
}

impl AgentStatus {
    /// Create a new agent status
    pub fn new(agent_id: String, max_capacity: usize) -> Self {
        Self {
            agent_id,
            is_healthy: true,
            current_load: 0,
            max_capacity,
            last_health_check: Instant::now(),
            total_tasks_processed: 0,
            avg_execution_time: Duration::from_millis(0),
        }
    }

    /// Check if agent is available for new tasks
    pub fn is_available(&self) -> bool {
        self.is_healthy && self.current_load < self.max_capacity
    }

    /// Get load percentage (0.0 to 1.0)
    pub fn load_percentage(&self) -> f64 {
        if self.max_capacity == 0 {
            0.0
        } else {
            self.current_load as f64 / self.max_capacity as f64
        }
    }
}

/// Coordination errors
#[derive(Debug, thiserror::Error)]
pub enum CoordinationError {
    /// No available agents for the task
    #[error("No available agents for memory type: {memory_type:?}")]
    NoAvailableAgents { memory_type: MemoryType },

    /// Task timeout
    #[error("Task {task_id} timed out after {timeout:?}")]
    TaskTimeout { task_id: String, timeout: Duration },

    /// Agent communication error
    #[error("Communication error with agent {agent_id}: {error}")]
    CommunicationError { agent_id: String, error: String },

    /// Task execution error
    #[error("Task {task_id} failed: {error}")]
    TaskExecutionError { task_id: String, error: String },

    /// Agent registration error
    #[error("Failed to register agent {agent_id}: {error}")]
    AgentRegistrationError { agent_id: String, error: String },

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// Internal error
    #[error("Internal coordination error: {0}")]
    InternalError(String),
}

/// Result type for coordination operations
pub type CoordinationResult<T> = Result<T, CoordinationError>;

/// Coordination statistics
#[derive(Debug, Clone, Default)]
pub struct CoordinationStats {
    /// Total tasks processed
    pub total_tasks: u64,
    /// Successful tasks
    pub successful_tasks: u64,
    /// Failed tasks
    pub failed_tasks: u64,
    /// Average task execution time
    pub avg_execution_time: Duration,
    /// Total agents registered
    pub total_agents: usize,
    /// Healthy agents count
    pub healthy_agents: usize,
    /// Tasks by memory type
    pub tasks_by_type: HashMap<MemoryType, u64>,
}

impl CoordinationStats {
    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_tasks == 0 {
            0.0
        } else {
            self.successful_tasks as f64 / self.total_tasks as f64
        }
    }

    /// Update task statistics
    pub fn update_task_stats(
        &mut self,
        memory_type: MemoryType,
        success: bool,
        execution_time: Duration,
    ) {
        self.total_tasks += 1;
        if success {
            self.successful_tasks += 1;
        } else {
            self.failed_tasks += 1;
        }

        // Update average execution time
        let total_time = self.avg_execution_time.as_nanos() as u64 * (self.total_tasks - 1)
            + execution_time.as_nanos() as u64;
        self.avg_execution_time = Duration::from_nanos(total_time / self.total_tasks);

        // Update tasks by type
        *self.tasks_by_type.entry(memory_type).or_insert(0) += 1;
    }
}

/// MetaMemoryManager - Central coordinator for multi-agent memory system
///
/// This is the core orchestrator that manages task distribution, load balancing,
/// and inter-agent communication in the AgentMem multi-agent architecture.
pub struct MetaMemoryManager {
    /// Configuration
    config: MetaMemoryConfig,
    /// Registered agents by memory type
    agents_by_type: Arc<RwLock<HashMap<MemoryType, Vec<String>>>>,
    /// Agent status tracking
    agent_status: Arc<RwLock<HashMap<String, AgentStatus>>>,
    /// Message channels for each agent
    agent_channels: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<AgentMessage>>>>,
    /// Pending tasks
    pending_tasks: Arc<RwLock<HashMap<String, oneshot::Sender<TaskResponse>>>>,
    /// Round-robin counters for load balancing
    round_robin_counters: Arc<RwLock<HashMap<MemoryType, usize>>>,
    /// Coordination statistics
    stats: Arc<RwLock<CoordinationStats>>,
}

impl MetaMemoryManager {
    /// Create a new MetaMemoryManager
    pub fn new(config: MetaMemoryConfig) -> Self {
        Self {
            config,
            agents_by_type: Arc::new(RwLock::new(HashMap::new())),
            agent_status: Arc::new(RwLock::new(HashMap::new())),
            agent_channels: Arc::new(RwLock::new(HashMap::new())),
            pending_tasks: Arc::new(RwLock::new(HashMap::new())),
            round_robin_counters: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(CoordinationStats::default())),
        }
    }

    /// Create a new MetaMemoryManager with default configuration
    pub fn with_default_config() -> Self {
        Self::new(MetaMemoryConfig::default())
    }

    /// Register a new agent with the coordinator
    pub async fn register_agent(
        &self,
        agent_id: String,
        memory_types: Vec<MemoryType>,
        max_capacity: usize,
        message_channel: mpsc::UnboundedSender<AgentMessage>,
    ) -> CoordinationResult<()> {
        let mut agents_by_type = self.agents_by_type.write().await;
        let mut agent_status = self.agent_status.write().await;
        let mut agent_channels = self.agent_channels.write().await;

        // Register agent for each memory type it handles
        for memory_type in memory_types {
            agents_by_type
                .entry(memory_type)
                .or_insert_with(Vec::new)
                .push(agent_id.clone());
        }

        // Initialize agent status
        agent_status.insert(
            agent_id.clone(),
            AgentStatus::new(agent_id.clone(), max_capacity),
        );

        // Store message channel
        agent_channels.insert(agent_id.clone(), message_channel);

        // Update statistics
        let mut stats = self.stats.write().await;
        stats.total_agents += 1;
        stats.healthy_agents += 1;

        log::info!(
            "Registered agent {agent_id} with capacity {max_capacity}"
        );
        Ok(())
    }

    /// Unregister an agent
    pub async fn unregister_agent(&self, agent_id: &str) -> CoordinationResult<()> {
        let mut agents_by_type = self.agents_by_type.write().await;
        let mut agent_status = self.agent_status.write().await;
        let mut agent_channels = self.agent_channels.write().await;

        // Remove agent from all memory type lists
        for agents in agents_by_type.values_mut() {
            agents.retain(|id| id != agent_id);
        }

        // Remove agent status and channel
        agent_status.remove(agent_id);
        agent_channels.remove(agent_id);

        // Update statistics
        let mut stats = self.stats.write().await;
        if stats.total_agents > 0 {
            stats.total_agents -= 1;
        }
        if stats.healthy_agents > 0 {
            stats.healthy_agents -= 1;
        }

        log::info!("Unregistered agent {agent_id}");
        Ok(())
    }

    /// Execute a task by routing it to the appropriate agent
    pub async fn execute_task(&self, task: TaskRequest) -> CoordinationResult<TaskResponse> {
        let start_time = Instant::now();

        // Select the best agent for this task
        let agent_id = self.select_agent(&task.memory_type).await?;

        // Create response channel
        let (response_tx, response_rx) = oneshot::channel();

        // Store pending task
        {
            let mut pending_tasks = self.pending_tasks.write().await;
            pending_tasks.insert(task.task_id.clone(), response_tx);
        }

        // Send task to agent
        self.send_task_to_agent(&agent_id, &task).await?;

        // Wait for response with timeout
        let timeout_duration = task.timeout.unwrap_or(self.config.task_timeout);
        let response = match timeout(timeout_duration, response_rx).await {
            Ok(Ok(response)) => response,
            Ok(Err(_)) => {
                // Channel was closed
                return Err(CoordinationError::CommunicationError {
                    agent_id: agent_id.clone(),
                    error: "Response channel closed".to_string(),
                });
            }
            Err(_) => {
                // Timeout
                return Err(CoordinationError::TaskTimeout {
                    task_id: task.task_id.clone(),
                    timeout: timeout_duration,
                });
            }
        };

        // Update statistics
        let execution_time = start_time.elapsed();
        let mut stats = self.stats.write().await;
        stats.update_task_stats(task.memory_type, response.success, execution_time);

        // Update agent status
        self.update_agent_load(&agent_id, -1).await;

        Ok(response)
    }

    /// Select the best agent for a given memory type using load balancing
    async fn select_agent(&self, memory_type: &MemoryType) -> CoordinationResult<String> {
        let agents_by_type = self.agents_by_type.read().await;
        let agent_status = self.agent_status.read().await;

        let available_agents = agents_by_type.get(memory_type).ok_or({
            CoordinationError::NoAvailableAgents {
                memory_type: *memory_type,
            }
        })?;

        if available_agents.is_empty() {
            return Err(CoordinationError::NoAvailableAgents {
                memory_type: *memory_type,
            });
        }

        // Filter healthy and available agents
        let healthy_agents: Vec<&String> = available_agents
            .iter()
            .filter(|agent_id| {
                agent_status
                    .get(*agent_id)
                    .map(|status| status.is_available())
                    .unwrap_or(false)
            })
            .collect();

        if healthy_agents.is_empty() {
            return Err(CoordinationError::NoAvailableAgents {
                memory_type: *memory_type,
            });
        }

        // Apply load balancing strategy
        let selected_agent = match self.config.load_balancing_strategy {
            LoadBalancingStrategy::RoundRobin => {
                self.select_round_robin(memory_type, &healthy_agents).await
            }
            LoadBalancingStrategy::LeastLoaded => {
                self.select_least_loaded(&healthy_agents, &agent_status)
                    .await
            }
            LoadBalancingStrategy::SpecializationBased => {
                // For now, use round-robin. Can be enhanced with specialization metrics
                self.select_round_robin(memory_type, &healthy_agents).await
            }
        };

        // Update agent load
        self.update_agent_load(&selected_agent, 1).await;

        Ok(selected_agent)
    }

    /// Round-robin agent selection
    async fn select_round_robin(&self, memory_type: &MemoryType, agents: &[&String]) -> String {
        let mut counters = self.round_robin_counters.write().await;
        let counter = counters.entry(*memory_type).or_insert(0);
        let selected = agents[*counter % agents.len()].clone();
        *counter += 1;
        selected
    }

    /// Least loaded agent selection
    async fn select_least_loaded(
        &self,
        agents: &[&String],
        agent_status: &HashMap<String, AgentStatus>,
    ) -> String {
        agents
            .iter()
            .min_by_key(|agent_id| {
                agent_status
                    .get(agent_id.as_str())
                    .map(|status| status.current_load)
                    .unwrap_or(usize::MAX)
            })
            .unwrap()
            .to_string()
    }

    /// Send a task to a specific agent
    async fn send_task_to_agent(
        &self,
        agent_id: &str,
        task: &TaskRequest,
    ) -> CoordinationResult<()> {
        let agent_channels = self.agent_channels.read().await;
        let channel =
            agent_channels
                .get(agent_id)
                .ok_or_else(|| CoordinationError::CommunicationError {
                    agent_id: agent_id.to_string(),
                    error: "Agent channel not found".to_string(),
                })?;

        let message = AgentMessage::new(
            MessageType::TaskRequest,
            "meta_manager".to_string(),
            agent_id.to_string(),
            serde_json::to_value(task).map_err(|e| {
                CoordinationError::InternalError(format!("Failed to serialize task: {e}"))
            })?,
        )
        .with_priority(task.priority);

        channel
            .send(message)
            .map_err(|e| CoordinationError::CommunicationError {
                agent_id: agent_id.to_string(),
                error: format!("Failed to send message: {e}"),
            })?;

        Ok(())
    }

    /// Update agent load counter
    async fn update_agent_load(&self, agent_id: &str, delta: i32) {
        let mut agent_status = self.agent_status.write().await;
        if let Some(status) = agent_status.get_mut(agent_id) {
            if delta > 0 {
                status.current_load += delta as usize;
            } else if delta < 0 && status.current_load > 0 {
                status.current_load -= (-delta) as usize;
            }
        }
    }

    /// Handle task response from an agent
    pub async fn handle_task_response(&self, response: TaskResponse) -> CoordinationResult<()> {
        let mut pending_tasks = self.pending_tasks.write().await;
        if let Some(response_tx) = pending_tasks.remove(&response.task_id) {
            let _ = response_tx.send(response);
        }
        Ok(())
    }

    /// Get coordination statistics
    pub async fn get_stats(&self) -> CoordinationStats {
        self.stats.read().await.clone()
    }

    /// Get agent status information
    pub async fn get_agent_status(&self, agent_id: &str) -> Option<AgentStatus> {
        self.agent_status.read().await.get(agent_id).cloned()
    }

    /// List all registered agents
    pub async fn list_agents(&self) -> Vec<String> {
        self.agent_status.read().await.keys().cloned().collect()
    }

    /// Check system health
    pub async fn health_check(&self) -> HashMap<String, bool> {
        let agent_status = self.agent_status.read().await;
        agent_status
            .iter()
            .map(|(agent_id, status)| (agent_id.clone(), status.is_healthy))
            .collect()
    }
}
