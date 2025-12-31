//! Background Agent System
//!
//! This module implements background agent processing, allowing agents to
//! process messages asynchronously in the background.

use crate::agent_state::{AgentState, AgentStateMachine};
use crate::message_queue::{AgentMessage, MessageQueue};
use crate::CoreError;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

/// Background agent processor
///
/// Manages background processing of messages for agents.
pub struct BackgroundAgentManager {
    /// Message queue
    message_queue: Arc<MessageQueue>,
    /// Agent state machines
    state_machines: Arc<RwLock<HashMap<String, AgentStateMachine>>>,
    /// Background tasks
    tasks: Arc<RwLock<HashMap<String, JoinHandle<()>>>>,
}

impl BackgroundAgentManager {
    /// Create a new background agent manager
    pub fn new(message_queue: Arc<MessageQueue>) -> Self {
        Self {
            message_queue,
            state_machines: Arc::new(RwLock::new(HashMap::new())),
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start a background agent
    ///
    /// Creates a message queue for the agent and starts a background task
    /// to process messages.
    pub async fn start_agent<F>(&self, agent_id: String, processor: F) -> Result<(), CoreError>
    where
        F: Fn(AgentMessage) -> Result<(), CoreError> + Send + Sync + 'static,
    {
        // Check if agent is already running
        {
            let tasks = self.tasks.read().await;
            if tasks.contains_key(&agent_id) {
                return Err(CoreError::InvalidInput(format!(
                    "Agent {agent_id} is already running"
                )));
            }
        }

        // Create state machine
        {
            let mut state_machines = self.state_machines.write().await;
            state_machines.insert(agent_id.clone(), AgentStateMachine::new(agent_id.clone()));
        }

        // Create message queue
        let mut rx = self.message_queue.create_queue(agent_id.clone()).await;

        // Clone necessary data for the background task
        let agent_id_clone = agent_id.clone();
        let state_machines = Arc::clone(&self.state_machines);

        // Start background task
        let handle = tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                // Update state to Thinking
                {
                    let mut sms = state_machines.write().await;
                    if let Some(sm) = sms.get_mut(&agent_id_clone) {
                        let _ = sm.transition(AgentState::Thinking);
                    }
                }

                // Process message
                match processor(message) {
                    Ok(_) => {
                        // Update state to Idle
                        let mut sms = state_machines.write().await;
                        if let Some(sm) = sms.get_mut(&agent_id_clone) {
                            let _ = sm.transition(AgentState::Idle);
                        }
                    }
                    Err(e) => {
                        // Update state to Error
                        let mut sms = state_machines.write().await;
                        if let Some(sm) = sms.get_mut(&agent_id_clone) {
                            let _ = sm.transition_to_error(e.to_string());
                        }
                    }
                }
            }
        });

        // Store task handle
        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(agent_id, handle);
        }

        Ok(())
    }

    /// Stop a background agent
    pub async fn stop_agent(&self, agent_id: &str) -> Result<(), CoreError> {
        // Remove task
        let handle = {
            let mut tasks = self.tasks.write().await;
            tasks.remove(agent_id)
        };

        if let Some(handle) = handle {
            // Abort the task
            handle.abort();

            // Remove message queue
            self.message_queue.remove_queue(agent_id).await;

            // Remove state machine
            {
                let mut state_machines = self.state_machines.write().await;
                state_machines.remove(agent_id);
            }

            Ok(())
        } else {
            Err(CoreError::NotFound(format!(
                "Agent {agent_id} is not running"
            )))
        }
    }

    /// Get the state of an agent
    pub async fn get_agent_state(&self, agent_id: &str) -> Option<AgentState> {
        let state_machines = self.state_machines.read().await;
        state_machines
            .get(agent_id)
            .map(|sm| sm.current_state().clone())
    }

    /// Get the state machine for an agent
    pub async fn get_state_machine(&self, agent_id: &str) -> Option<AgentStateMachine> {
        let state_machines = self.state_machines.read().await;
        state_machines.get(agent_id).cloned()
    }

    /// Check if an agent is running
    pub async fn is_agent_running(&self, agent_id: &str) -> bool {
        let tasks = self.tasks.read().await;
        tasks.contains_key(agent_id)
    }

    /// Get all running agent IDs
    pub async fn running_agents(&self) -> Vec<String> {
        let tasks = self.tasks.read().await;
        tasks.keys().cloned().collect()
    }

    /// Get the number of running agents
    pub async fn running_agent_count(&self) -> usize {
        let tasks = self.tasks.read().await;
        tasks.len()
    }

    /// Send a message to an agent
    pub async fn send_message(&self, message: AgentMessage) -> Result<(), CoreError> {
        self.message_queue
            .send_message(message)
            .await
            .map_err(CoreError::InvalidInput)
    }

    /// Stop all background agents
    pub async fn stop_all(&self) {
        let agent_ids: Vec<String> = {
            let tasks = self.tasks.read().await;
            tasks.keys().cloned().collect()
        };

        for agent_id in agent_ids {
            let _ = self.stop_agent(&agent_id).await;
        }
    }
}

impl Drop for BackgroundAgentManager {
    fn drop(&mut self) {
        // Abort all tasks
        // Use try_write to avoid blocking in async context
        if let Ok(tasks) = self.tasks.try_write() {
            for (_, handle) in tasks.iter() {
                handle.abort();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_background_agent_creation() {
        let queue = Arc::new(MessageQueue::new());
        let manager = BackgroundAgentManager::new(queue);

        assert_eq!(manager.running_agent_count().await, 0);
    }

    #[tokio::test]
    async fn test_start_and_stop_agent() {
        let queue = Arc::new(MessageQueue::new());
        let manager = BackgroundAgentManager::new(queue);

        let processor = |_msg: AgentMessage| Ok(());

        // Start agent
        assert!(manager
            .start_agent("agent-1".to_string(), processor)
            .await
            .is_ok());
        assert!(manager.is_agent_running("agent-1").await);

        // Stop agent
        assert!(manager.stop_agent("agent-1").await.is_ok());
        assert!(!manager.is_agent_running("agent-1").await);
    }

    #[tokio::test]
    async fn test_send_message_to_agent() {
        let queue = Arc::new(MessageQueue::new());
        let manager = BackgroundAgentManager::new(Arc::clone(&queue));

        let (tx, mut rx) = tokio::sync::mpsc::channel(10);

        let processor = move |msg: AgentMessage| {
            let tx = tx.clone();
            tokio::spawn(async move {
                let _ = tx.send(msg.content).await;
            });
            Ok(())
        };

        // Start agent
        manager
            .start_agent("agent-1".to_string(), processor)
            .await
            .unwrap();

        // Send message
        let message = AgentMessage::new(
            "agent-1".to_string(),
            "user-1".to_string(),
            "Hello".to_string(),
        );
        manager.send_message(message).await?;

        // Wait for message to be processed
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Check if message was received
        let received = rx.try_recv();
        assert!(received.is_ok());
        assert_eq!(received.unwrap(), "Hello");

        // Stop agent
        manager.stop_agent("agent-1").await?;
    }

    #[tokio::test]
    async fn test_agent_state_transitions() {
        let queue = Arc::new(MessageQueue::new());
        let manager = BackgroundAgentManager::new(Arc::clone(&queue));

        let processor = |_msg: AgentMessage| {
            // Simulate processing
            std::thread::sleep(std::time::Duration::from_millis(50));
            Ok(())
        };

        // Start agent
        manager
            .start_agent("agent-1".to_string(), processor)
            .await
            .unwrap();

        // Initial state should be Idle
        let state = manager.get_agent_state("agent-1").await;
        assert_eq!(state, Some(AgentState::Idle));

        // Send message
        let message = AgentMessage::new(
            "agent-1".to_string(),
            "user-1".to_string(),
            "Hello".to_string(),
        );
        manager.send_message(message).await?;

        // Wait for processing
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        // State should be back to Idle
        let state = manager.get_agent_state("agent-1").await;
        assert_eq!(state, Some(AgentState::Idle));

        // Stop agent
        manager.stop_agent("agent-1").await?;
    }
}
