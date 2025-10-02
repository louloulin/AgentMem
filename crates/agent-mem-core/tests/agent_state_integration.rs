//! Integration tests for Agent State Management
//!
//! These tests verify the end-to-end functionality of the agent state management system,
//! including state machine, message queue, and background agent processing.

use agent_mem_core::{
    agent_state::{AgentState, AgentStateMachine},
    background_agent::BackgroundAgentManager,
    message_queue::{AgentMessage, MessageAccumulator, MessageQueue},
};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_agent_state_machine_transitions() {
    let mut sm = AgentStateMachine::new("test-agent".to_string());

    // Test initial state
    assert_eq!(sm.current_state(), &AgentState::Idle);

    // Test valid transition: Idle -> Thinking
    assert!(sm.transition(AgentState::Thinking).is_ok());
    assert_eq!(sm.current_state(), &AgentState::Thinking);

    // Test valid transition: Thinking -> Executing
    assert!(sm.transition(AgentState::Executing).is_ok());
    assert_eq!(sm.current_state(), &AgentState::Executing);

    // Test valid transition: Executing -> Idle
    assert!(sm.transition(AgentState::Idle).is_ok());
    assert_eq!(sm.current_state(), &AgentState::Idle);
}

#[tokio::test]
async fn test_agent_state_machine_invalid_transitions() {
    let mut sm = AgentStateMachine::new("test-agent".to_string());

    // Test invalid transition: Idle -> Executing
    assert!(sm.transition(AgentState::Executing).is_err());
    assert_eq!(sm.current_state(), &AgentState::Idle);

    // Test invalid transition: Idle -> Waiting
    assert!(sm.transition(AgentState::Waiting).is_err());
    assert_eq!(sm.current_state(), &AgentState::Idle);
}

#[tokio::test]
async fn test_agent_state_machine_error_handling() {
    let mut sm = AgentStateMachine::new("test-agent".to_string());

    // Transition to error state
    assert!(sm
        .transition_to_error("Test error message".to_string())
        .is_ok());
    assert_eq!(sm.current_state(), &AgentState::Error);
    assert_eq!(sm.error_message(), Some("Test error message"));

    // Test recovery: Error -> Idle
    assert!(sm.transition(AgentState::Idle).is_ok());
    assert_eq!(sm.current_state(), &AgentState::Idle);
    assert!(sm.error_message().is_none());
}

#[tokio::test]
async fn test_message_queue_basic_operations() {
    let queue = MessageQueue::new();

    // Create queue for agent
    let mut rx = queue.create_queue("agent-1".to_string()).await;

    // Send message
    let message = AgentMessage::new(
        "agent-1".to_string(),
        "user-1".to_string(),
        "Hello, agent!".to_string(),
    );

    assert!(queue.send_message(message.clone()).await.is_ok());

    // Receive message
    let received = tokio::time::timeout(Duration::from_secs(1), rx.recv())
        .await
        .expect("Timeout waiting for message")
        .expect("No message received");

    assert_eq!(received.agent_id, "agent-1");
    assert_eq!(received.user_id, "user-1");
    assert_eq!(received.content, "Hello, agent!");
}

#[tokio::test]
async fn test_message_queue_multiple_agents() {
    let queue = MessageQueue::new();

    // Create queues for multiple agents
    let mut rx1 = queue.create_queue("agent-1".to_string()).await;
    let mut rx2 = queue.create_queue("agent-2".to_string()).await;

    // Send messages to different agents
    let msg1 = AgentMessage::new(
        "agent-1".to_string(),
        "user-1".to_string(),
        "Message for agent 1".to_string(),
    );
    let msg2 = AgentMessage::new(
        "agent-2".to_string(),
        "user-1".to_string(),
        "Message for agent 2".to_string(),
    );

    assert!(queue.send_message(msg1).await.is_ok());
    assert!(queue.send_message(msg2).await.is_ok());

    // Verify each agent receives only their message
    let received1 = tokio::time::timeout(Duration::from_secs(1), rx1.recv())
        .await
        .expect("Timeout")
        .expect("No message");
    assert_eq!(received1.content, "Message for agent 1");

    let received2 = tokio::time::timeout(Duration::from_secs(1), rx2.recv())
        .await
        .expect("Timeout")
        .expect("No message");
    assert_eq!(received2.content, "Message for agent 2");
}

#[tokio::test]
async fn test_message_accumulator() {
    let mut accumulator = MessageAccumulator::new(3, Duration::from_secs(60));

    // Add messages
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

    // First two messages should not trigger flush
    assert!(accumulator.add_message(msg1).is_none());
    assert!(accumulator.add_message(msg2).is_none());

    // Third message should trigger flush
    let flushed = accumulator.add_message(msg3);
    assert!(flushed.is_some());
    let messages = flushed.unwrap();
    assert_eq!(messages.len(), 3);
    assert_eq!(accumulator.len(), 0);
}

#[tokio::test]
async fn test_background_agent_lifecycle() {
    let queue = Arc::new(MessageQueue::new());
    let manager = BackgroundAgentManager::new(Arc::clone(&queue));

    // Create a channel to track processed messages
    let (tx, mut rx) = tokio::sync::mpsc::channel(10);

    let processor = move |msg: AgentMessage| {
        let tx = tx.clone();
        tokio::spawn(async move {
            let _ = tx.send(msg.content).await;
        });
        Ok(())
    };

    // Start agent
    assert!(manager
        .start_agent("agent-1".to_string(), processor)
        .await
        .is_ok());
    assert!(manager.is_agent_running("agent-1").await);

    // Send message
    let message = AgentMessage::new(
        "agent-1".to_string(),
        "user-1".to_string(),
        "Test message".to_string(),
    );
    assert!(manager.send_message(message).await.is_ok());

    // Wait for processing
    let processed = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .expect("Timeout waiting for processing")
        .expect("No message processed");
    assert_eq!(processed, "Test message");

    // Stop agent
    assert!(manager.stop_agent("agent-1").await.is_ok());
    assert!(!manager.is_agent_running("agent-1").await);
}

#[tokio::test]
async fn test_background_agent_state_transitions() {
    let queue = Arc::new(MessageQueue::new());
    let manager = BackgroundAgentManager::new(Arc::clone(&queue));

    let processor = |_msg: AgentMessage| {
        // Simulate processing
        std::thread::sleep(Duration::from_millis(100));
        Ok(())
    };

    // Start agent
    manager
        .start_agent("agent-1".to_string(), processor)
        .await
        .unwrap();

    // Initial state should be Idle
    tokio::time::sleep(Duration::from_millis(50)).await;
    let state = manager.get_agent_state("agent-1").await;
    assert_eq!(state, Some(AgentState::Idle));

    // Send message
    let message = AgentMessage::new(
        "agent-1".to_string(),
        "user-1".to_string(),
        "Test".to_string(),
    );
    manager.send_message(message).await.unwrap();

    // Wait for processing to complete
    tokio::time::sleep(Duration::from_millis(200)).await;

    // State should be back to Idle
    let state = manager.get_agent_state("agent-1").await;
    assert_eq!(state, Some(AgentState::Idle));

    // Stop agent
    manager.stop_agent("agent-1").await.unwrap();
}

#[tokio::test]
async fn test_background_agent_error_handling() {
    let queue = Arc::new(MessageQueue::new());
    let manager = BackgroundAgentManager::new(Arc::clone(&queue));

    let processor = |_msg: AgentMessage| {
        Err(agent_mem_core::CoreError::InvalidInput(
            "Test error".to_string(),
        ))
    };

    // Start agent
    manager
        .start_agent("agent-1".to_string(), processor)
        .await
        .unwrap();

    // Send message that will cause error
    let message = AgentMessage::new(
        "agent-1".to_string(),
        "user-1".to_string(),
        "Test".to_string(),
    );
    manager.send_message(message).await.unwrap();

    // Wait for processing
    tokio::time::sleep(Duration::from_millis(200)).await;

    // State should be Error
    let state = manager.get_agent_state("agent-1").await;
    assert_eq!(state, Some(AgentState::Error));

    // Get state machine to check error message
    let sm = manager.get_state_machine("agent-1").await;
    assert!(sm.is_some());
    let sm = sm.unwrap();
    assert!(sm.error_message().is_some());

    // Stop agent
    manager.stop_agent("agent-1").await.unwrap();
}

#[tokio::test]
async fn test_multiple_background_agents() {
    let queue = Arc::new(MessageQueue::new());
    let manager = BackgroundAgentManager::new(Arc::clone(&queue));

    let (tx1, mut rx1) = tokio::sync::mpsc::channel(10);
    let (tx2, mut rx2) = tokio::sync::mpsc::channel(10);

    let processor1 = move |msg: AgentMessage| {
        let tx = tx1.clone();
        tokio::spawn(async move {
            let _ = tx.send(format!("agent1: {}", msg.content)).await;
        });
        Ok(())
    };

    let processor2 = move |msg: AgentMessage| {
        let tx = tx2.clone();
        tokio::spawn(async move {
            let _ = tx.send(format!("agent2: {}", msg.content)).await;
        });
        Ok(())
    };

    // Start multiple agents
    manager
        .start_agent("agent-1".to_string(), processor1)
        .await
        .unwrap();
    manager
        .start_agent("agent-2".to_string(), processor2)
        .await
        .unwrap();

    assert_eq!(manager.running_agent_count().await, 2);

    // Send messages to both agents
    let msg1 = AgentMessage::new(
        "agent-1".to_string(),
        "user-1".to_string(),
        "Hello 1".to_string(),
    );
    let msg2 = AgentMessage::new(
        "agent-2".to_string(),
        "user-1".to_string(),
        "Hello 2".to_string(),
    );

    manager.send_message(msg1).await.unwrap();
    manager.send_message(msg2).await.unwrap();

    // Verify processing
    let result1 = tokio::time::timeout(Duration::from_secs(1), rx1.recv())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(result1, "agent1: Hello 1");

    let result2 = tokio::time::timeout(Duration::from_secs(1), rx2.recv())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(result2, "agent2: Hello 2");

    // Stop all agents
    manager.stop_all().await;
    assert_eq!(manager.running_agent_count().await, 0);
}

