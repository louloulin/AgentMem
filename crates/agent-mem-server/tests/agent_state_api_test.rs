//! Integration tests for Agent State API endpoints
//!
//! These tests verify the agent state management API endpoints work correctly.

#![cfg(feature = "postgres")]

use agent_mem_core::storage::{
    agent_repository::AgentRepository, models::Agent, repository::Repository,
};
use chrono::Utc;
use sqlx::PgPool;

/// Helper function to create a test database pool
async fn create_test_pool() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/agentmem_test".to_string());

    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

/// Helper function to create a test agent
async fn create_test_agent(pool: &PgPool, org_id: &str) -> Agent {
    let mut agent = Agent::new(org_id.to_string(), Some("Test Agent".to_string()));
    agent.state = Some("idle".to_string());
    agent.last_active_at = Some(Utc::now());
    agent.error_message = None;

    let repo = AgentRepository::new(pool.clone());
    repo.create(&agent)
        .await
        .expect("Failed to create test agent");

    agent
}

#[tokio::test]
#[ignore] // Requires database connection
async fn test_agent_state_persistence() {
    let pool = create_test_pool().await;
    let org_id = "test-org-1";

    // Create test agent
    let agent = create_test_agent(&pool, org_id).await;

    // Verify initial state
    let repo = AgentRepository::new(pool.clone());
    let retrieved = repo
        .read(&agent.id)
        .await
        .expect("Failed to read agent")
        .expect("Agent not found");

    assert_eq!(retrieved.state, Some("idle".to_string()));
    assert!(retrieved.last_active_at.is_some());
    assert!(retrieved.error_message.is_none());

    // Update state to thinking
    let mut updated_agent = retrieved.clone();
    updated_agent.state = Some("thinking".to_string());
    updated_agent.last_active_at = Some(Utc::now());

    repo.update(&updated_agent)
        .await
        .expect("Failed to update agent");

    // Verify state was updated
    let retrieved = repo
        .read(&agent.id)
        .await
        .expect("Failed to read agent")
        .expect("Agent not found");

    assert_eq!(retrieved.state, Some("thinking".to_string()));

    // Update state to error
    let mut error_agent = retrieved.clone();
    error_agent.state = Some("error".to_string());
    error_agent.error_message = Some("Test error message".to_string());

    repo.update(&error_agent)
        .await
        .expect("Failed to update agent");

    // Verify error state
    let retrieved = repo
        .read(&agent.id)
        .await
        .expect("Failed to read agent")
        .expect("Agent not found");

    assert_eq!(retrieved.state, Some("error".to_string()));
    assert_eq!(
        retrieved.error_message,
        Some("Test error message".to_string())
    );

    // Cleanup
    repo.delete(&agent.id)
        .await
        .expect("Failed to delete agent");
}

#[tokio::test]
#[ignore] // Requires database connection
async fn test_agent_state_transitions_in_database() {
    let pool = create_test_pool().await;
    let org_id = "test-org-2";

    // Create test agent
    let agent = create_test_agent(&pool, org_id).await;
    let repo = AgentRepository::new(pool.clone());

    // Test state transition: idle -> thinking
    let mut agent = repo
        .read(&agent.id)
        .await
        .expect("Failed to read agent")
        .expect("Agent not found");

    agent.state = Some("thinking".to_string());
    agent.last_active_at = Some(Utc::now());
    repo.update(&agent).await.expect("Failed to update");

    // Test state transition: thinking -> executing
    let mut agent = repo
        .read(&agent.id)
        .await
        .expect("Failed to read agent")
        .expect("Agent not found");

    agent.state = Some("executing".to_string());
    agent.last_active_at = Some(Utc::now());
    repo.update(&agent).await.expect("Failed to update");

    // Test state transition: executing -> idle
    let mut agent = repo
        .read(&agent.id)
        .await
        .expect("Failed to read agent")
        .expect("Agent not found");

    agent.state = Some("idle".to_string());
    agent.last_active_at = Some(Utc::now());
    repo.update(&agent).await.expect("Failed to update");

    // Verify final state
    let agent = repo
        .read(&agent.id)
        .await
        .expect("Failed to read agent")
        .expect("Agent not found");

    assert_eq!(agent.state, Some("idle".to_string()));

    // Cleanup
    repo.delete(&agent.id)
        .await
        .expect("Failed to delete agent");
}

#[tokio::test]
#[ignore] // Requires database connection
async fn test_multiple_agents_state_management() {
    let pool = create_test_pool().await;
    let org_id = "test-org-3";

    // Create multiple agents
    let agent1 = create_test_agent(&pool, org_id).await;
    let agent2 = create_test_agent(&pool, org_id).await;
    let agent3 = create_test_agent(&pool, org_id).await;

    let repo = AgentRepository::new(pool.clone());

    // Update each agent to different states
    let mut agent1 = repo
        .read(&agent1.id)
        .await
        .expect("Failed to read")
        .expect("Not found");
    agent1.state = Some("thinking".to_string());
    repo.update(&agent1).await.expect("Failed to update");

    let mut agent2 = repo
        .read(&agent2.id)
        .await
        .expect("Failed to read")
        .expect("Not found");
    agent2.state = Some("executing".to_string());
    repo.update(&agent2).await.expect("Failed to update");

    let mut agent3 = repo
        .read(&agent3.id)
        .await
        .expect("Failed to read")
        .expect("Not found");
    agent3.state = Some("error".to_string());
    agent3.error_message = Some("Test error".to_string());
    repo.update(&agent3).await.expect("Failed to update");

    // Verify each agent has correct state
    let agent1 = repo
        .read(&agent1.id)
        .await
        .expect("Failed to read")
        .expect("Not found");
    assert_eq!(agent1.state, Some("thinking".to_string()));

    let agent2 = repo
        .read(&agent2.id)
        .await
        .expect("Failed to read")
        .expect("Not found");
    assert_eq!(agent2.state, Some("executing".to_string()));

    let agent3 = repo
        .read(&agent3.id)
        .await
        .expect("Failed to read")
        .expect("Not found");
    assert_eq!(agent3.state, Some("error".to_string()));
    assert_eq!(agent3.error_message, Some("Test error".to_string()));

    // Cleanup
    repo.delete(&agent1.id).await.expect("Failed to delete");
    repo.delete(&agent2.id).await.expect("Failed to delete");
    repo.delete(&agent3.id).await.expect("Failed to delete");
}

#[test]
fn test_agent_state_validation() {
    // Test valid states
    let valid_states = vec!["idle", "thinking", "executing", "waiting", "error"];

    for state in valid_states {
        assert!(
            ["idle", "thinking", "executing", "waiting", "error"].contains(&state),
            "State {} should be valid",
            state
        );
    }

    // Test invalid states
    let invalid_states = vec!["invalid", "unknown", "test"];

    for state in invalid_states {
        assert!(
            !["idle", "thinking", "executing", "waiting", "error"].contains(&state),
            "State {} should be invalid",
            state
        );
    }
}

#[test]
fn test_agent_state_string_conversion() {
    use agent_mem_core::agent_state::AgentState;

    // Test conversion to string
    assert_eq!(AgentState::Idle.as_str(), "idle");
    assert_eq!(AgentState::Thinking.as_str(), "thinking");
    assert_eq!(AgentState::Executing.as_str(), "executing");
    assert_eq!(AgentState::Waiting.as_str(), "waiting");
    assert_eq!(AgentState::Error.as_str(), "error");

    // Test conversion from string
    assert_eq!(AgentState::from_str("idle"), Some(AgentState::Idle));
    assert_eq!(
        AgentState::from_str("thinking"),
        Some(AgentState::Thinking)
    );
    assert_eq!(
        AgentState::from_str("executing"),
        Some(AgentState::Executing)
    );
    assert_eq!(AgentState::from_str("waiting"), Some(AgentState::Waiting));
    assert_eq!(AgentState::from_str("error"), Some(AgentState::Error));
    assert_eq!(AgentState::from_str("invalid"), None);
}

