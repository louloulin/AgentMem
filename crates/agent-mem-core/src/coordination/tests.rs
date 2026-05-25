//! Tests for Multi-Agent Coordination System

use serde_json::json;
use tokio::sync::mpsc;

use crate::agents::{CoreAgent, EpisodicAgent, MemoryAgent, SemanticAgent};
use crate::coordination::{
    AgentMessage, LoadBalancingStrategy, MessageType, MetaMemoryConfig, MetaMemoryManager,
    TaskRequest,
};
use crate::types::MemoryType;

#[tokio::test]
async fn test_meta_memory_manager_creation() {
    let config = MetaMemoryConfig::default();
    let manager = MetaMemoryManager::new(config);

    let stats = manager.get_stats().await;
    assert_eq!(stats.total_agents, 0);
    assert_eq!(stats.total_tasks, 0);
}

#[tokio::test]
async fn test_agent_registration() {
    let config = MetaMemoryConfig::default();
    let manager = MetaMemoryManager::new(config);

    // Create message channel for agent
    let (tx, _rx) = mpsc::unbounded_channel();

    // Register an episodic agent
    let result = manager
        .register_agent(
            "episodic_agent_1".to_string(),
            vec![MemoryType::Episodic],
            10,
            tx,
        )
        .await;

    assert!(result.is_ok());

    let stats = manager.get_stats().await;
    assert_eq!(stats.total_agents, 1);
    assert_eq!(stats.healthy_agents, 1);

    // Check agent is listed
    let agents = manager.list_agents().await;
    assert_eq!(agents.len(), 1);
    assert!(agents.contains(&"episodic_agent_1".to_string()));
}

#[tokio::test]
async fn test_multiple_agent_registration() {
    let config = MetaMemoryConfig::default();
    let manager = MetaMemoryManager::new(config);

    // Register multiple agents
    for i in 0..3 {
        let (tx, _rx) = mpsc::unbounded_channel();
        let agent_id = format!("agent_{i}");
        let memory_type = match i {
            0 => MemoryType::Episodic,
            1 => MemoryType::Semantic,
            _ => MemoryType::Core,
        };

        let result = manager
            .register_agent(agent_id, vec![memory_type], 10, tx)
            .await;

        assert!(result.is_ok());
    }

    let stats = manager.get_stats().await;
    assert_eq!(stats.total_agents, 3);
    assert_eq!(stats.healthy_agents, 3);
}

#[tokio::test]
async fn test_agent_unregistration() {
    let config = MetaMemoryConfig::default();
    let manager = MetaMemoryManager::new(config);

    // Register an agent
    let (tx, _rx) = mpsc::unbounded_channel();
    manager
        .register_agent("test_agent".to_string(), vec![MemoryType::Episodic], 10, tx)
        .await
        .unwrap();

    // Verify registration
    let stats = manager.get_stats().await;
    assert_eq!(stats.total_agents, 1);

    // Unregister agent
    let result = manager.unregister_agent("test_agent").await;
    assert!(result.is_ok());

    // Verify unregistration
    let stats = manager.get_stats().await;
    assert_eq!(stats.total_agents, 0);

    let agents = manager.list_agents().await;
    assert!(agents.is_empty());
}

#[tokio::test]
async fn test_task_execution_no_agents() {
    let config = MetaMemoryConfig::default();
    let manager = MetaMemoryManager::new(config);

    // Try to execute a task without any registered agents
    let task = TaskRequest::new(
        MemoryType::Episodic,
        "search".to_string(),
        json!({"query": "test"}),
    );

    let result = manager.execute_task(task).await;
    assert!(result.is_err());

    // Should be a NoAvailableAgents error
    match result.unwrap_err() {
        crate::coordination::CoordinationError::NoAvailableAgents { memory_type } => {
            assert_eq!(memory_type, MemoryType::Episodic);
        }
        _ => panic!("Expected NoAvailableAgents error"),
    }
}

#[tokio::test]
async fn test_load_balancing_strategies() {
    // Test different load balancing strategies
    let strategies = vec![
        LoadBalancingStrategy::RoundRobin,
        LoadBalancingStrategy::LeastLoaded,
        LoadBalancingStrategy::SpecializationBased,
    ];

    for strategy in strategies {
        let config = MetaMemoryConfig {
            load_balancing_strategy: strategy.clone(),
            ..Default::default()
        };

        let manager = MetaMemoryManager::new(config);

        // Register multiple agents for same memory type
        for i in 0..3 {
            let (tx, _rx) = mpsc::unbounded_channel();
            let agent_id = format!("episodic_agent_{i}");

            manager
                .register_agent(agent_id, vec![MemoryType::Episodic], 10, tx)
                .await
                .unwrap();
        }

        let stats = manager.get_stats().await;
        assert_eq!(stats.total_agents, 3);
    }
}

#[tokio::test]
async fn test_health_check() {
    let config = MetaMemoryConfig::default();
    let manager = MetaMemoryManager::new(config);

    // Register an agent
    let (tx, _rx) = mpsc::unbounded_channel();
    manager
        .register_agent(
            "healthy_agent".to_string(),
            vec![MemoryType::Semantic],
            10,
            tx,
        )
        .await
        .unwrap();

    // Check health
    let health = manager.health_check().await;
    assert_eq!(health.len(), 1);
    assert_eq!(health.get("healthy_agent"), Some(&true));
}

#[tokio::test]
async fn test_agent_status() {
    let config = MetaMemoryConfig::default();
    let manager = MetaMemoryManager::new(config);

    // Register an agent
    let (tx, _rx) = mpsc::unbounded_channel();
    manager
        .register_agent("status_agent".to_string(), vec![MemoryType::Core], 5, tx)
        .await
        .unwrap();

    // Get agent status
    let status = manager.get_agent_status("status_agent").await;
    assert!(status.is_some());

    let status = status.unwrap();
    assert_eq!(status.agent_id, "status_agent");
    assert_eq!(status.max_capacity, 5);
    assert_eq!(status.current_load, 0);
    assert!(status.is_healthy);
    assert!(status.is_available());
}

#[tokio::test]
async fn test_episodic_agent_creation() {
    let mut agent = EpisodicAgent::new("test_episodic".to_string());

    assert_eq!(agent.agent_id(), "test_episodic");
    assert_eq!(agent.memory_types(), &[MemoryType::Episodic]);
    assert!(!agent.health_check().await);

    // Initialize agent
    let result = agent.initialize().await;
    assert!(result.is_ok());
    assert!(agent.health_check().await);

    // Check if agent can accept tasks
    assert!(agent.can_accept_task().await);
    assert_eq!(agent.current_load().await, 0);
}

#[tokio::test]
async fn test_semantic_agent_creation() {
    let mut agent = SemanticAgent::new("test_semantic".to_string());

    assert_eq!(agent.agent_id(), "test_semantic");
    assert_eq!(agent.memory_types(), &[MemoryType::Semantic]);

    let result = agent.initialize().await;
    assert!(result.is_ok());
    assert!(agent.health_check().await);
}

#[tokio::test]
async fn test_core_agent_creation() {
    let mut agent = CoreAgent::new("test_core".to_string());

    assert_eq!(agent.agent_id(), "test_core");
    assert_eq!(agent.memory_types(), &[MemoryType::Core]);

    let result = agent.initialize().await;
    assert!(result.is_ok());
    assert!(agent.health_check().await);
}

#[tokio::test]
async fn test_agent_task_execution() -> anyhow::Result<()> {
    let mut agent = EpisodicAgent::new("test_execution".to_string());
    agent.initialize().await?;

    // Create a test task
    let task = TaskRequest::new(
        MemoryType::Episodic,
        "search".to_string(),
        json!({"user_id": "test_user", "query": "test query"}),
    );

    // Execute task
    let result = agent.execute_task(task).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert!(response.success);
    assert_eq!(response.executed_by, "test_execution");
    Ok(())
}

#[tokio::test]
async fn test_agent_message_handling() -> anyhow::Result<()> {
    let mut agent = EpisodicAgent::new("test_messages".to_string());
    agent.initialize().await?;

    // Create a test message
    let message = AgentMessage::new(
        MessageType::HealthCheck,
        "meta_manager".to_string(),
        "test_messages".to_string(),
        json!({}),
    );

    // Handle message
    let result = agent.handle_message(message).await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_agent_statistics() -> anyhow::Result<()> {
    let mut agent = EpisodicAgent::new("test_stats".to_string());
    agent.initialize().await?;

    // Get initial stats
    let stats = agent.get_stats().await;
    assert_eq!(stats.total_tasks, 0);
    assert_eq!(stats.successful_tasks, 0);
    assert_eq!(stats.failed_tasks, 0);
    assert_eq!(stats.active_tasks, 0);

    // Execute a task to update stats
    let task = TaskRequest::new(
        MemoryType::Episodic,
        "search".to_string(),
        json!({"user_id": "test_user", "query": "test"}),
    );

    agent.execute_task(task).await?;

    // Check updated stats
    let stats = agent.get_stats().await;
    assert_eq!(stats.total_tasks, 1);
    assert_eq!(stats.successful_tasks, 1);
    assert_eq!(stats.failed_tasks, 0);
    assert_eq!(stats.active_tasks, 0);
    Ok(())
}

// ================ Block-related tests ================

#[tokio::test]
async fn test_human_block_creation_and_retrieval() -> anyhow::Result<()> {
    let mut agent = SemanticAgent::new("block_agent".to_string());
    agent.initialize().await?;

    // Create a human memory block
    let user_id = "user123".to_string();
    let content = "This is a block of memories".to_string();
    let metadata = json!({
        "user_id": user_id,
        "block_id": "test_block_1"
    });

    let task = TaskRequest::new(
        MemoryType::Episodic,
        "create_block".to_string(),
        metadata.clone(),
    );

    let result = agent.execute_task(task).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert!(response.success);

    // Retrieve block
    let retrieve_task = TaskRequest::new(
        MemoryType::Episodic,
        "retrieve_block".to_string(),
        json!({"block_id": "test_block_1"}),
    );

    let retrieve_result = agent.execute_task(retrieve_task).await?;
    assert!(retrieve_result.success);

    Ok(())
}

#[tokio::test]
async fn test_persona_block_creation_and_retrieval() -> anyhow::Result<()> {
    let mut agent = SemanticAgent::new("persona_agent".to_string());
    agent.initialize().await?;

    // Create a persona block
    let content = "Persona preferences".to_string();
    let metadata = json!({
        "persona_id": "developer",
        "block_id": "persona_block_1"
    });

    let task = TaskRequest::new(
        MemoryType::Core,
        "create_block".to_string(),
        metadata.clone(),
    );

    let result = agent.execute_task(task).await;
    assert!(result.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_block_content_update() -> anyhow::Result<()> {
    let mut agent = SemanticAgent::new("block_agent".to_string());
    agent.initialize().await?;

    // Create initial block
    let metadata = json!({
        "block_id": "test_block_2"
    });

    let task = TaskRequest::new(
        MemoryType::Episodic,
        "create_block".to_string(),
        metadata.clone(),
    );

    agent.execute_task(task).await?;

    // Update block content
    let update_metadata = json!({
        "block_id": "test_block_2",
        "action": "append",
        "content": "Additional content"
    });

    let update_task = TaskRequest::new(
        MemoryType::Episodic,
        "update_block".to_string(),
        update_metadata,
    );

    let result = agent.execute_task(update_task).await;
    assert!(result.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_block_content_append() -> anyhow::Result<()> {
    let mut agent = SemanticAgent::new("block_agent".to_string());
    agent.initialize().await?;

    // Create and append to block
    let metadata = json!({
        "block_id": "test_block_3"
    });

    let task = TaskRequest::new(
        MemoryType::Episodic,
        "create_block".to_string(),
        metadata.clone(),
    );

    agent.execute_task(task).await?;

    Ok(())
}


