//! Specialized Memory Agents Module
//!
//! This module provides specialized memory agents for each cognitive memory type,
//! inspired by MIRIX's multi-agent architecture but optimized for Rust's performance.
//!
//! # Architecture Overview
//!
//! Each memory type has a dedicated agent that specializes in handling operations
//! for that specific type of memory:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Memory Agents                            │
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
//! │  │ EpisodicAgent│  │SemanticAgent│  │ProceduralAgent│       │
//! │  └─────────────┘  └─────────────┘  └─────────────┘         │
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
//! │  │ WorkingAgent │  │  CoreAgent  │  │ResourceAgent│         │
//! │  └─────────────┘  └─────────────┘  └─────────────┘         │
//! │  ┌─────────────┐  ┌─────────────┐                          │
//! │  │KnowledgeAgent│  │ContextualAgent│                       │
//! │  └─────────────┘  └─────────────┘                          │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Agent Responsibilities
//!
//! - **EpisodicAgent**: Manages time-based episodic memories and events
//! - **SemanticAgent**: Handles factual knowledge and semantic relationships
//! - **ProceduralAgent**: Manages step-by-step procedures and workflows
//! - **WorkingAgent**: Handles temporary working memory and active contexts
//! - **CoreAgent**: Manages persistent core memory blocks
//! - **ResourceAgent**: Handles multimedia resources and file storage
//! - **KnowledgeAgent**: Manages encrypted knowledge vault and sensitive data
//! - **ContextualAgent**: Handles environmental context and situational awareness

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use crate::coordination::{AgentMessage, CoordinationResult, TaskRequest, TaskResponse};
use crate::types::MemoryType;

pub mod contextual_agent;
pub mod core_agent;
pub mod episodic_agent;
pub mod knowledge_agent;
pub mod procedural_agent;
pub mod resource_agent;
pub mod semantic_agent;
pub mod working_agent;

// Re-export agent types
pub use contextual_agent::ContextualAgent;
pub use core_agent::CoreAgent;
pub use episodic_agent::EpisodicAgent;
pub use knowledge_agent::KnowledgeAgent;
pub use procedural_agent::ProceduralAgent;
pub use resource_agent::ResourceAgent;
pub use semantic_agent::SemanticAgent;
pub use working_agent::WorkingAgent;

/// Agent configuration
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// Agent unique identifier
    pub agent_id: String,
    /// Memory types this agent handles
    pub memory_types: Vec<MemoryType>,
    /// Maximum concurrent tasks
    pub max_concurrent_tasks: usize,
    /// Agent-specific configuration
    pub agent_specific_config: serde_json::Value,
}

impl AgentConfig {
    /// Create a new agent configuration
    pub fn new(
        agent_id: String,
        memory_types: Vec<MemoryType>,
        max_concurrent_tasks: usize,
    ) -> Self {
        Self {
            agent_id,
            memory_types,
            max_concurrent_tasks,
            agent_specific_config: serde_json::Value::Null,
        }
    }

    /// Set agent-specific configuration
    pub fn with_config(mut self, config: serde_json::Value) -> Self {
        self.agent_specific_config = config;
        self
    }
}

/// Agent execution context
#[derive(Debug, Clone)]
pub struct AgentContext {
    /// Current task being executed
    pub current_task: Option<TaskRequest>,
    /// Agent statistics
    pub stats: AgentStats,
    /// Agent configuration
    pub config: AgentConfig,
}

/// Agent statistics
#[derive(Debug, Clone, Default)]
pub struct AgentStats {
    /// Total tasks processed
    pub total_tasks: u64,
    /// Successful tasks
    pub successful_tasks: u64,
    /// Failed tasks
    pub failed_tasks: u64,
    /// Average execution time in milliseconds
    pub avg_execution_time_ms: f64,
    /// Current active tasks
    pub active_tasks: usize,
}

impl AgentStats {
    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_tasks == 0 {
            0.0
        } else {
            self.successful_tasks as f64 / self.total_tasks as f64
        }
    }

    /// Update statistics after task completion
    pub fn update_task_completion(&mut self, success: bool, execution_time_ms: f64) {
        self.total_tasks += 1;
        if success {
            self.successful_tasks += 1;
        } else {
            self.failed_tasks += 1;
        }

        // Update average execution time
        let total_time =
            self.avg_execution_time_ms * (self.total_tasks - 1) as f64 + execution_time_ms;
        self.avg_execution_time_ms = total_time / self.total_tasks as f64;
    }
}

/// Core trait that all memory agents must implement
#[async_trait]
pub trait MemoryAgent: Send + Sync {
    /// Get the agent's unique identifier
    fn agent_id(&self) -> &str;

    /// Get the memory types this agent handles
    fn memory_types(&self) -> &[MemoryType];

    /// Initialize the agent
    async fn initialize(&mut self) -> CoordinationResult<()>;

    /// Shutdown the agent gracefully
    async fn shutdown(&mut self) -> CoordinationResult<()>;

    /// Execute a task
    async fn execute_task(&mut self, task: TaskRequest) -> CoordinationResult<TaskResponse>;

    /// Handle an incoming message
    async fn handle_message(&mut self, message: AgentMessage) -> CoordinationResult<()>;

    /// Get agent statistics
    async fn get_stats(&self) -> AgentStats;

    /// Check if agent is healthy
    async fn health_check(&self) -> bool;

    /// Get current load (number of active tasks)
    async fn current_load(&self) -> usize;

    /// Check if agent can accept more tasks
    async fn can_accept_task(&self) -> bool;
}

/// Agent error types
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    /// Task execution error
    #[error("Task execution failed: {0}")]
    TaskExecutionError(String),

    /// Invalid task parameters
    #[error("Invalid task parameters: {0}")]
    InvalidParameters(String),

    /// Agent not initialized
    #[error("Agent not initialized")]
    NotInitialized,

    /// Agent overloaded
    #[error("Agent overloaded, cannot accept more tasks")]
    Overloaded,

    /// Memory manager error
    #[error("Memory manager error: {0}")]
    MemoryManagerError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Internal error
    #[error("Internal agent error: {0}")]
    InternalError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

/// Result type for agent operations
pub type AgentResult<T> = Result<T, AgentError>;

/// Base agent implementation with common functionality
pub struct BaseAgent {
    /// Agent configuration
    config: AgentConfig,
    /// Agent context
    context: Arc<RwLock<AgentContext>>,
    /// Message receiver channel
    message_rx: Option<mpsc::UnboundedReceiver<AgentMessage>>,
    /// Response sender for coordination
    response_tx: Option<mpsc::UnboundedSender<TaskResponse>>,
}

impl BaseAgent {
    /// Create a new base agent
    pub fn new(config: AgentConfig) -> Self {
        let context = AgentContext {
            current_task: None,
            stats: AgentStats::default(),
            config: config.clone(),
        };

        Self {
            config,
            context: Arc::new(RwLock::new(context)),
            message_rx: None,
            response_tx: None,
        }
    }

    /// Set message channels
    pub fn set_channels(
        &mut self,
        message_rx: mpsc::UnboundedReceiver<AgentMessage>,
        response_tx: mpsc::UnboundedSender<TaskResponse>,
    ) {
        self.message_rx = Some(message_rx);
        self.response_tx = Some(response_tx);
    }

    /// Get agent configuration
    pub fn config(&self) -> &AgentConfig {
        &self.config
    }

    /// Get agent context
    pub fn context(&self) -> Arc<RwLock<AgentContext>> {
        self.context.clone()
    }

    /// Send task response
    pub async fn send_response(&self, response: TaskResponse) -> AgentResult<()> {
        if let Some(ref tx) = self.response_tx {
            tx.send(response).map_err(|e| {
                AgentError::InternalError(format!("Failed to send response: {e}"))
            })?;
        }
        Ok(())
    }
}
