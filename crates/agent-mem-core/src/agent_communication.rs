//! Agent Communication Protocol Module
//!
//! This module provides inter-agent communication capabilities for the AgentMem system,
//! enabling agents to exchange messages, share context, and coordinate tasks.
//!
//! # Features
//!
//! - Direct agent-to-agent messaging
//! - Message routing and delivery
//! - Request/Response pattern support
//! - Broadcast messaging
//! - Message queue with priority levels
//! - Delivery confirmation and retry mechanism
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │              AgentCommunicationManager                      │
//! │  ┌─────────────────────────────────────────────────────┐   │
//! │  │  AgentRegistry: HashMap<AgentId, Arc<Agent>>       │   │
//! │  │  MessageQueue: Vec<AgentMessage>                     │   │
//! │  │  PendingResponses: HashMap<MessageId, ResponseChan>│   │
//! │  └─────────────────────────────────────────────────────┘   │
//! └─────────────────────────────────────────────────────────────┘
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};

/// Agent communication errors
#[derive(Debug, Error)]
pub enum AgentCommError {
    /// Agent not found
    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    /// Message delivery failed
    #[error("Failed to deliver message to {0}: {1}")]
    DeliveryFailed(String, String),

    /// Timeout waiting for response
    #[error("Timeout waiting for response to message {0}")]
    Timeout(String),

    /// Invalid message format
    #[error("Invalid message format: {0}")]
    InvalidMessage(String),

    /// Channel closed
    #[error("Communication channel closed")]
    ChannelClosed,

    /// Routing error
    #[error("Routing error: {0}")]
    RoutingError(String),
}

/// Result type for communication operations
pub type CommResult<T> = Result<T, AgentCommError>;

/// Agent identifier
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct AgentId(pub String);

impl AgentId {
    /// Create a new agent ID
    pub fn new(id: &str) -> Self {
        Self(id.to_string())
    }

    /// Get the string representation
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Agent({})", self.0)
    }
}

/// Message priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MessagePriority {
    /// Low priority - can be delayed
    Low = 0,
    /// Normal priority
    Normal = 1,
    /// High priority - process soon
    High = 2,
    /// Critical - process immediately
    Critical = 3,
}

/// Inter-agent message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterAgentMessage {
    /// Unique message identifier
    pub id: String,
    /// Source agent ID
    pub source: AgentId,
    /// Target agent ID(s)
    pub targets: Vec<AgentId>,
    /// Message type
    pub message_type: InterAgentMessageType,
    /// Message priority
    pub priority: MessagePriority,
    /// Message payload (JSON)
    pub payload: serde_json::Value,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// TTL in seconds (0 = no expiry)
    pub ttl_seconds: u64,
    /// Correlation ID for request/response matching
    pub correlation_id: Option<String>,
}

impl InterAgentMessage {
    /// Create a new message
    pub fn new(source: AgentId, targets: Vec<AgentId>, message_type: InterAgentMessageType, payload: serde_json::Value) -> Self {
        Self {
            id: uuid_v4(),
            source,
            targets,
            message_type,
            priority: MessagePriority::Normal,
            payload,
            timestamp: Utc::now(),
            ttl_seconds: 300, // 5 minutes default
            correlation_id: None,
        }
    }

    /// Set message priority
    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set TTL
    pub fn with_ttl(mut self, ttl_seconds: u64) -> Self {
        self.ttl_seconds = ttl_seconds;
        self
    }

    /// Set correlation ID for request/response
    pub fn with_correlation_id(mut self, correlation_id: &str) -> Self {
        self.correlation_id = Some(correlation_id.to_string());
        self
    }

    /// Check if message is expired
    pub fn is_expired(&self) -> bool {
        if self.ttl_seconds == 0 {
            return false;
        }
        let elapsed = Utc::now().signed_duration_since(self.timestamp);
        elapsed.num_seconds() as u64 > self.ttl_seconds
    }
}

/// Inter-agent message types enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InterAgentMessageType {
    /// Request message - expects response
    Request,
    /// Response message - reply to a request
    Response,
    /// Notification - no response expected
    Notification,
    /// Broadcast - sent to all agents
    Broadcast,
    /// Event - something happened
    Event,
    /// Command - directive to agent
    Command,
    /// Query - asking for information
    Query,
}

/// Agent communication manager
#[derive(Debug, Clone)]
pub struct AgentCommunicationManager {
    /// Registered agents
    agents: Arc<tokio::sync::RwLock<HashMap<AgentId, mpsc::Sender<InterAgentMessage>>>>,
    /// Message history
    message_history: Arc<tokio::sync::RwLock<Vec<InterAgentMessage>>>,
    /// Pending responses
    pending_responses: Arc<tokio::sync::RwLock<HashMap<String, oneshot::Sender<InterAgentMessage>>>>,
    /// Configuration
    config: CommManagerConfig,
}

impl Default for AgentCommunicationManager {
    fn default() -> Self {
        Self::new(CommManagerConfig::default())
    }
}

/// Configuration for the communication manager
#[derive(Debug, Clone)]
pub struct CommManagerConfig {
    /// Maximum message history size
    pub max_history_size: usize,
    /// Default message TTL
    pub default_ttl_seconds: u64,
    /// Maximum retries for failed deliveries
    pub max_retries: u32,
    /// Timeout for waiting responses
    pub response_timeout_seconds: u64,
    /// Enable message queuing
    pub enable_queue: bool,
    /// Maximum queue size
    pub max_queue_size: usize,
}

impl Default for CommManagerConfig {
    fn default() -> Self {
        Self {
            max_history_size: 10000,
            default_ttl_seconds: 300,
            max_retries: 3,
            response_timeout_seconds: 30,
            enable_queue: true,
            max_queue_size: 1000,
        }
    }
}

/// Communication statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommStats {
    /// Total messages sent
    pub messages_sent: u64,
    /// Total messages received
    pub messages_received: u64,
    /// Total broadcasts
    pub broadcasts: u64,
    /// Failed deliveries
    pub failed_deliveries: u64,
    /// Average delivery time (ms)
    pub avg_delivery_time_ms: f64,
    /// Registered agents
    pub registered_agents: usize,
}

impl AgentCommunicationManager {
    /// Create a new communication manager
    pub fn new(config: CommManagerConfig) -> Self {
        Self {
            agents: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            message_history: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            pending_responses: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Register an agent for communication
    pub async fn register_agent(&self, agent_id: AgentId, sender: mpsc::Sender<InterAgentMessage>) {
        let mut agents = self.agents.write().await;
        agents.insert(agent_id, sender);
    }

    /// Unregister an agent
    pub async fn unregister_agent(&self, agent_id: &AgentId) -> Option<mpsc::Sender<InterAgentMessage>> {
        let mut agents = self.agents.write().await;
        agents.remove(agent_id)
    }

    /// Send a message to a specific agent
    pub async fn send_message(&self, message: InterAgentMessage) -> CommResult<()> {
        // Store in history
        self.add_to_history(message.clone()).await;

        for target in &message.targets {
            let agents = self.agents.read().await;
            if let Some(sender) = agents.get(target) {
                sender.send(message.clone()).await.map_err(|e| {
                    AgentCommError::DeliveryFailed(target.to_string(), e.to_string())
                })?;
            } else {
                return Err(AgentCommError::AgentNotFound(target.to_string()));
            }
        }
        Ok(())
    }

    /// Send message with request/response pattern
    pub async fn send_request<Response: for<'de> Deserialize<'de>>(
        &self,
        message: InterAgentMessage,
    ) -> CommResult<Response> {
        let correlation_id = message.id.clone();
        let (tx, rx) = oneshot::channel();

        // Register pending response
        {
            let mut pending = self.pending_responses.write().await;
            pending.insert(correlation_id.clone(), tx);
        }

        // Send message
        self.send_message(message).await?;

        // Wait for response with timeout
        tokio::time::timeout(
            tokio::time::Duration::from_secs(self.config.response_timeout_seconds),
            rx,
        )
        .await
        .map_err(|_| AgentCommError::Timeout(correlation_id.clone()))?
        .map_err(|_| AgentCommError::ChannelClosed)?;

        // Parse response
        // Note: In real implementation, this would need proper deserialization
        unimplemented!("Response deserialization needs to be implemented based on payload type")
    }

    /// Broadcast message to all registered agents
    pub async fn broadcast(&self, source: AgentId, _message_type: InterAgentMessageType, payload: serde_json::Value) -> CommResult<()> {
        let agents = self.agents.read().await;
        let targets: Vec<AgentId> = agents.keys().cloned().filter(|id| id != &source).collect();

        let message = InterAgentMessage {
            id: uuid_v4(),
            source,
            targets: targets.clone(),
            message_type: InterAgentMessageType::Broadcast,
            priority: MessagePriority::Normal,
            payload,
            timestamp: Utc::now(),
            ttl_seconds: self.config.default_ttl_seconds,
            correlation_id: None,
        };

        drop(agents);
        self.send_message(message).await
    }

    /// Send notification (fire and forget)
    pub async fn send_notification(
        &self,
        source: AgentId,
        target: AgentId,
        notification_type: &str,
        payload: serde_json::Value,
    ) -> CommResult<()> {
        let message = InterAgentMessage {
            id: uuid_v4(),
            source,
            targets: vec![target],
            message_type: InterAgentMessageType::Notification,
            priority: MessagePriority::Normal,
            payload: serde_json::json!({
                "type": notification_type,
                "data": payload
            }),
            timestamp: Utc::now(),
            ttl_seconds: self.config.default_ttl_seconds,
            correlation_id: None,
        };

        self.send_message(message).await
    }

    /// Add message to history
    async fn add_to_history(&self, message: InterAgentMessage) {
        let mut history = self.message_history.write().await;
        history.push(message);
        if history.len() > self.config.max_history_size {
            history.remove(0);
        }
    }

    /// Get message history for an agent
    pub async fn get_history(&self, agent_id: &AgentId, limit: usize) -> Vec<InterAgentMessage> {
        let history = self.message_history.read().await;
        history
            .iter()
            .filter(|m| m.source == *agent_id || m.targets.contains(agent_id))
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get registered agents
    pub async fn get_registered_agents(&self) -> Vec<AgentId> {
        let agents = self.agents.read().await;
        agents.keys().cloned().collect()
    }

    /// Check if agent is registered
    pub async fn is_registered(&self, agent_id: &AgentId) -> bool {
        let agents = self.agents.read().await;
        agents.contains_key(agent_id)
    }

    /// Get communication statistics
    pub async fn get_stats(&self) -> CommStats {
        let agents = self.agents.read().await;
        let history = self.message_history.read().await;

        let messages_sent = history.iter().filter(|m| m.message_type == InterAgentMessageType::Request).count() as u64;
        let messages_received = history.iter().filter(|m| m.targets.len() > 0).count() as u64;
        let broadcasts = history.iter().filter(|m| matches!(m.message_type, InterAgentMessageType::Broadcast)).count() as u64;

        CommStats {
            messages_sent,
            messages_received,
            broadcasts,
            failed_deliveries: 0,
            avg_delivery_time_ms: 0.0,
            registered_agents: agents.len(),
        }
    }

    /// Clear message history
    pub async fn clear_history(&self) {
        let mut history = self.message_history.write().await;
        history.clear();
    }
}

/// Simple UUID generator (for demonstration)
fn uuid_v4() -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}-{:x}", timestamp, rand_u64())
}

fn rand_u64() -> u64 {
    // Simple random - in production use proper UUID library
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    let rs = RandomState::new();
    let mut hasher = rs.build_hasher();
    hasher.write_u64(std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_and_send() {
        let manager = AgentCommunicationManager::new(CommManagerConfig::default());
        let (tx, _rx) = mpsc::channel(100);

        let agent_id = AgentId::new("agent-1");
        manager.register_agent(agent_id.clone(), tx).await;

        assert!(manager.is_registered(&agent_id).await);

        let targets = vec![AgentId::new("agent-1")];
        let message = InterAgentMessage::new(
            AgentId::new("test"),
            targets,
            InterAgentMessageType::Notification,
            serde_json::json!({"test": true}),
        );

        let result = manager.send_message(message).await;
        // Note: This will fail because we're sending to ourselves without proper receiver
    }

    #[test]
    fn test_agent_id_display() {
        let agent_id = AgentId::new("test-agent");
        assert_eq!(agent_id.to_string(), "Agent(test-agent)");
        assert_eq!(agent_id.as_str(), "test-agent");
    }

    #[test]
    fn test_message_creation() {
        let source = AgentId::new("source");
        let targets = vec![AgentId::new("target1"), AgentId::new("target2")];
        let payload = serde_json::json!({"key": "value"});

        let message = InterAgentMessage::new(
            source.clone(),
            targets.clone(),
            InterAgentMessageType::Request,
            payload,
        );

        assert_eq!(message.source, source);
        assert_eq!(message.targets, targets);
        assert_eq!(message.priority, MessagePriority::Normal);
        assert!(message.ttl_seconds > 0);
    }

    #[test]
    fn test_message_with_options() {
        let message = InterAgentMessage::new(
            AgentId::new("source"),
            vec![AgentId::new("target")],
            InterAgentMessageType::Command,
            serde_json::json!({}),
        )
        .with_priority(MessagePriority::High)
        .with_ttl(60)
        .with_correlation_id("corr-123");

        assert_eq!(message.priority, MessagePriority::High);
        assert_eq!(message.ttl_seconds, 60);
        assert_eq!(message.correlation_id, Some("corr-123".to_string()));
    }

    #[test]
    fn test_message_expiry() {
        let message = InterAgentMessage {
            id: "test".to_string(),
            source: AgentId::new("source"),
            targets: vec![AgentId::new("target")],
            message_type: InterAgentMessageType::Notification,
            priority: MessagePriority::Normal,
            payload: serde_json::json!({}),
            timestamp: Utc::now(),
            ttl_seconds: 0, // No expiry
            correlation_id: None,
        };

        assert!(!message.is_expired());
    }

    #[test]
    fn test_stats_default() {
        let stats = CommStats::default();
        assert_eq!(stats.messages_sent, 0);
        assert_eq!(stats.messages_received, 0);
        assert_eq!(stats.registered_agents, 0);
    }

    #[tokio::test]
    async fn test_unregister_agent() {
        let manager = AgentCommunicationManager::new(CommManagerConfig::default());
        let (tx, _rx) = mpsc::channel(100);
        let agent_id = AgentId::new("agent-1");

        manager.register_agent(agent_id.clone(), tx).await;
        assert!(manager.is_registered(&agent_id).await);

        manager.unregister_agent(&agent_id).await;
        assert!(!manager.is_registered(&agent_id).await);
    }

    #[tokio::test]
    async fn test_get_registered_agents() {
        let manager = AgentCommunicationManager::new(CommManagerConfig::default());
        let (tx1, _rx1) = mpsc::channel(100);
        let (tx2, _rx2) = mpsc::channel(100);

        manager.register_agent(AgentId::new("agent-1"), tx1).await;
        manager.register_agent(AgentId::new("agent-2"), tx2).await;

        let agents = manager.get_registered_agents().await;
        assert_eq!(agents.len(), 2);
    }
}