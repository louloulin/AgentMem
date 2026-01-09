//! Message Queue System
//!
//! This module implements a message queue system for agent communication,
//! allowing agents to receive and process messages asynchronously.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// Message for agent communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    /// Message ID
    pub id: String,
    /// Agent ID
    pub agent_id: String,
    /// User ID
    pub user_id: String,
    /// Message content
    pub content: String,
    /// Message metadata
    pub metadata: Option<serde_json::Value>,
    /// Message timestamp
    pub timestamp: DateTime<Utc>,
    /// Message priority (0-10, higher is more important)
    pub priority: u8,
}

impl AgentMessage {
    /// Create a new agent message
    pub fn new(agent_id: String, user_id: String, content: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            agent_id,
            user_id,
            content,
            metadata: None,
            timestamp: Utc::now(),
            priority: 5,
        }
    }

    /// Create a new agent message with priority
    pub fn with_priority(agent_id: String, user_id: String, content: String, priority: u8) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            agent_id,
            user_id,
            content,
            metadata: None,
            timestamp: Utc::now(),
            priority: priority.min(10),
        }
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Message queue for agent communication
pub struct MessageQueue {
    /// Queues for each agent (agent_id -> sender)
    queues: Arc<RwLock<HashMap<String, mpsc::Sender<AgentMessage>>>>,
    /// Queue capacity
    capacity: usize,
}

impl MessageQueue {
    /// Create a new message queue with default capacity (100)
    pub fn new() -> Self {
        Self {
            queues: Arc::new(RwLock::new(HashMap::new())),
            capacity: 100,
        }
    }

    /// Create a new message queue with custom capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            queues: Arc::new(RwLock::new(HashMap::new())),
            capacity,
        }
    }

    /// Create a queue for an agent
    ///
    /// Returns a receiver that can be used to receive messages for the agent.
    pub async fn create_queue(&self, agent_id: String) -> mpsc::Receiver<AgentMessage> {
        let (tx, rx) = mpsc::channel(self.capacity);
        let mut queues = self.queues.write().await;
        queues.insert(agent_id, tx);
        rx
    }

    /// Send a message to an agent
    ///
    /// Returns `Ok(())` if the message was sent successfully, `Err(String)` otherwise.
    pub async fn send_message(&self, message: AgentMessage) -> Result<(), String> {
        let queues = self.queues.read().await;
        if let Some(tx) = queues.get(&message.agent_id) {
            tx.send(message)
                .await
                .map_err(|e| format!("Failed to send message: {e}"))?;
            Ok(())
        } else {
            Err(format!("Agent queue not found: {}", message.agent_id))
        }
    }

    /// Remove a queue for an agent
    pub async fn remove_queue(&self, agent_id: &str) {
        let mut queues = self.queues.write().await;
        queues.remove(agent_id);
    }

    /// Check if a queue exists for an agent
    pub async fn has_queue(&self, agent_id: &str) -> bool {
        let queues = self.queues.read().await;
        queues.contains_key(agent_id)
    }

    /// Get the number of active queues
    pub async fn queue_count(&self) -> usize {
        let queues = self.queues.read().await;
        queues.len()
    }

    /// Get all active agent IDs
    pub async fn active_agents(&self) -> Vec<String> {
        let queues = self.queues.read().await;
        queues.keys().cloned().collect()
    }
}

impl Default for MessageQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Message accumulator for batching messages
///
/// Accumulates messages and flushes them when a threshold is reached
/// (either by count or by time).
pub struct MessageAccumulator {
    /// Accumulated messages
    messages: Vec<AgentMessage>,
    /// Maximum number of messages before flushing
    max_messages: usize,
    /// Maximum duration before flushing
    max_duration: std::time::Duration,
    /// Last flush time
    last_flush: std::time::Instant,
}

impl MessageAccumulator {
    /// Create a new message accumulator
    pub fn new(max_messages: usize, max_duration: std::time::Duration) -> Self {
        Self {
            messages: Vec::new(),
            max_messages,
            max_duration,
            last_flush: std::time::Instant::now(),
        }
    }

    /// Add a message to the accumulator
    ///
    /// Returns `Some(Vec<AgentMessage>)` if the accumulator should be flushed,
    /// `None` otherwise.
    pub fn add_message(&mut self, message: AgentMessage) -> Option<Vec<AgentMessage>> {
        self.messages.push(message);

        if self.should_flush() {
            self.flush()
        } else {
            None
        }
    }

    /// Check if the accumulator should be flushed
    fn should_flush(&self) -> bool {
        self.messages.len() >= self.max_messages || self.last_flush.elapsed() >= self.max_duration
    }

    /// Flush the accumulator
    ///
    /// Returns all accumulated messages and resets the accumulator.
    fn flush(&mut self) -> Option<Vec<AgentMessage>> {
        if self.messages.is_empty() {
            return None;
        }

        let messages = std::mem::take(&mut self.messages);
        self.last_flush = std::time::Instant::now();
        Some(messages)
    }

    /// Force flush the accumulator
    pub fn force_flush(&mut self) -> Vec<AgentMessage> {
        let messages = std::mem::take(&mut self.messages);
        self.last_flush = std::time::Instant::now();
        messages
    }

    /// Get the number of accumulated messages
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Check if the accumulator is empty
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_message_queue_creation() {
        let queue = MessageQueue::new();
        assert_eq!(queue.queue_count().await, 0);
    }

    #[tokio::test]
    async fn test_create_and_send_message() -> anyhow::Result<()> {
        let queue = MessageQueue::new();
        let mut rx = queue.create_queue("agent-1".to_string()).await;

        let message = AgentMessage::new(
            "agent-1".to_string(),
            "user-1".to_string(),
            "Hello".to_string(),
        );

        assert!(queue.send_message(message.clone()).await.is_ok());

        let received = rx.recv().await?;
        assert_eq!(received.content, "Hello");
    }

    #[tokio::test]
    async fn test_send_to_nonexistent_queue() {
        let queue = MessageQueue::new();

        let message = AgentMessage::new(
            "agent-1".to_string(),
            "user-1".to_string(),
            "Hello".to_string(),
        );

        assert!(queue.send_message(message).await.is_err());
    }

    #[tokio::test]
    async fn test_remove_queue() {
        let queue = MessageQueue::new();
        let _rx = queue.create_queue("agent-1".to_string()).await;

        assert!(queue.has_queue("agent-1").await);

        queue.remove_queue("agent-1").await;

        assert!(!queue.has_queue("agent-1").await);
    }

    #[test]
    fn test_message_accumulator() {
        let mut accumulator = MessageAccumulator::new(3, std::time::Duration::from_secs(60));

        let msg1 = AgentMessage::new(
            "agent-1".to_string(),
            "user-1".to_string(),
            "Message 1".to_string(),
        );
        let msg2 = AgentMessage::new(
            "agent-1".to_string(),
            "user-1".to_string(),
            "Message 2".to_string(),
        );
        let msg3 = AgentMessage::new(
            "agent-1".to_string(),
            "user-1".to_string(),
            "Message 3".to_string(),
        );

        assert!(accumulator.add_message(msg1).is_none());
        assert!(accumulator.add_message(msg2).is_none());

        let flushed = accumulator.add_message(msg3);
        assert!(flushed.is_some());
        assert_eq!(flushed.unwrap().len(), 3);
        assert_eq!(accumulator.len(), 0);
    }
}

    async fn test_create_and_send_message() -> anyhow::Result<()> {
        let queue = MessageQueue::new();
        let mut rx = queue.create_queue("agent-1".to_string()).await;

        let message = AgentMessage::new(
            "agent-1".to_string(),
            "user-1".to_string(),
            "Hello".to_string(),
        );

        assert!(queue.send_message(message.clone()).await.is_ok());

        let received = rx.recv().await.ok_or_else(|| anyhow::anyhow!("Failed to receive message"))?;
        assert_eq!(received.content, "Hello");
        Ok(())
    }

    #[tokio::test]
    async fn test_send_to_nonexistent_queue() {
        let queue = MessageQueue::new();

        let message = AgentMessage::new(
            "agent-1".to_string(),
            "user-1".to_string(),
            "Hello".to_string(),
        );

        assert!(queue.send_message(message).await.is_err());
    }

    #[tokio::test]
    async fn test_remove_queue() {
        let queue = MessageQueue::new();
        let _rx = queue.create_queue("agent-1".to_string()).await;

        assert!(queue.has_queue("agent-1").await);

        queue.remove_queue("agent-1").await;

        assert!(!queue.has_queue("agent-1").await);
    }

    #[test]
    fn test_message_accumulator() {
        let mut accumulator = MessageAccumulator::new(3, std::time::Duration::from_secs(60));

        let msg1 = AgentMessage::new(
            "agent-1".to_string(),
            "user-1".to_string(),
            "Message 1".to_string(),
        );
        let msg2 = AgentMessage::new(
            "agent-1".to_string(),
            "user-1".to_string(),
            "Message 2".to_string(),
        );
        let msg3 = AgentMessage::new(
            "agent-1".to_string(),
            "user-1".to_string(),
            "Message 3".to_string(),
        );

        assert!(accumulator.add_message(msg1).is_none());
        assert!(accumulator.add_message(msg2).is_none());

        let flushed = accumulator.add_message(msg3);
        assert!(flushed.is_some());
        assert_eq!(flushed.unwrap().len(), 3);
        assert_eq!(accumulator.len(), 0);
    }
