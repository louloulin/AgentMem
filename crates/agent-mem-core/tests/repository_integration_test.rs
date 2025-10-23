//! Integration tests for repository enhancements
//!
//! These tests require a running PostgreSQL database.
//! Set DATABASE_URL environment variable to run these tests.

#![cfg(feature = "postgres")]

use agent_mem_core::storage::{
    agent_repository::AgentRepository,
    batch::BatchOperations,
    memory_repository::MemoryRepository,
    models::*,
    postgres::PostgresStorage,
    repository::{OrganizationRepository, Repository, UserRepository},
    tool_repository::ToolRepository,
    transaction::{RetryConfig, TransactionManager},
    PostgresConfig,
};

/// Helper function to get test database URL
fn get_test_db_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://agentmem:password@localhost:5432/agentmem_test".to_string()
    })
}

/// Helper function to create test PostgreSQL storage
async fn create_test_storage() -> PostgresStorage {
    let config = PostgresConfig {
        url: get_test_db_url(),
        max_connections: 5,
        connection_timeout: 30,
        query_timeout: 30,
        ssl: false,
    };

    let storage = PostgresStorage::new(config)
        .await
        .expect("Failed to create PostgreSQL storage");

    storage.migrate().await.expect("Failed to run migrations");

    storage
}

#[tokio::test]
#[ignore] // Requires database
async fn test_tool_repository() {
    let storage = create_test_storage().await;
    let pool = storage.pool().clone();
    let org_repo = OrganizationRepository::new(pool.clone());
    let tool_repo = ToolRepository::new(pool);

    // Create organization
    let org = Organization::new("Test Org".to_string());
    let org = org_repo
        .create(&org)
        .await
        .expect("Failed to create organization");

    // Create tool
    let tool = Tool {
        id: generate_id("tool"),
        organization_id: org.id.clone(),
        name: "test_function".to_string(),
        description: Some("A test function".to_string()),
        json_schema: Some(serde_json::json!({
            "type": "function",
            "function": {
                "name": "test_function",
                "parameters": {}
            }
        })),
        source_type: Some("python".to_string()),
        source_code: Some("def test(): pass".to_string()),
        tags: Some(vec!["test".to_string(), "function".to_string()]),
        metadata_: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        is_deleted: false,
        created_by_id: None,
        last_updated_by_id: None,
    };

    let created_tool = tool_repo
        .create(&tool)
        .await
        .expect("Failed to create tool");
    assert_eq!(created_tool.name, "test_function");

    // Find by name
    let found_tool = tool_repo
        .find_by_name(&org.id, "test_function")
        .await
        .expect("Failed to find tool")
        .expect("Tool not found");
    assert_eq!(found_tool.id, created_tool.id);

    // List by organization
    let tools = tool_repo
        .list_by_organization(&org.id, Some(10), Some(0))
        .await
        .expect("Failed to list tools");
    assert!(!tools.is_empty());

    // Clean up
    tool_repo.hard_delete(&created_tool.id).await.ok();
    org_repo.hard_delete(&org.id).await.ok();
}

#[tokio::test]
#[ignore] // Requires database
async fn test_memory_repository() {
    let storage = create_test_storage().await;
    let pool = storage.pool().clone();
    let org_repo = OrganizationRepository::new(pool.clone());
    let user_repo = UserRepository::new(pool.clone());
    let agent_repo = AgentRepository::new(pool.clone());
    let memory_repo = MemoryRepository::new(pool);

    // Create organization, user, and agent
    let org = Organization::new("Test Org".to_string());
    let org = org_repo
        .create(&org)
        .await
        .expect("Failed to create organization");

    let user = User::new(org.id.clone(), "Test User".to_string(), "test@example.com".to_string(), "hashed_password".to_string(), "UTC".to_string());
    let user = user_repo
        .create(&user)
        .await
        .expect("Failed to create user");

    let agent = Agent::new(org.id.clone(), Some("Test Agent".to_string()));
    let agent = agent_repo
        .create(&agent)
        .await
        .expect("Failed to create agent");

    // Create memory
    let memory = Memory {
        id: generate_id("memory"),
        organization_id: org.id.clone(),
        user_id: user.id.clone(),
        agent_id: agent.id.clone(),
        content: "The user likes pizza".to_string(),
        hash: None,
        metadata: serde_json::json!({}),
        score: Some(0.9),
        memory_type: "episodic".to_string(),
        scope: "agent".to_string(),
        level: "short_term".to_string(),
        importance: 0.8,
        access_count: 0,
        last_accessed: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        is_deleted: false,
        created_by_id: None,
        last_updated_by_id: None,
    };

    let created_memory = memory_repo
        .create(&memory)
        .await
        .expect("Failed to create memory");
    assert_eq!(created_memory.content, "The user likes pizza");

    // List by agent
    let memories = memory_repo
        .list_by_agent(&agent.id, Some(10), Some(0))
        .await
        .expect("Failed to list memories");
    assert_eq!(memories.len(), 1);

    // Full-text search
    let search_results = memory_repo
        .search_fulltext(&agent.id, "pizza", Some(10))
        .await
        .expect("Failed to search memories");
    assert!(!search_results.is_empty());

    // Update access
    memory_repo
        .update_access(&created_memory.id)
        .await
        .expect("Failed to update access");

    // Get most important
    let important = memory_repo
        .get_most_important(&agent.id, 5)
        .await
        .expect("Failed to get important memories");
    assert!(!important.is_empty());

    // Clean up
    memory_repo.hard_delete(&created_memory.id).await.ok();
    agent_repo.hard_delete(&agent.id).await.ok();
    user_repo.hard_delete(&user.id).await.ok();
    org_repo.hard_delete(&org.id).await.ok();
}

#[tokio::test]
#[ignore] // Requires database
async fn test_batch_operations() {
    let storage = create_test_storage().await;
    let pool = storage.pool().clone();
    let org_repo = OrganizationRepository::new(pool.clone());
    let user_repo = UserRepository::new(pool.clone());
    let batch_ops = BatchOperations::new(pool);

    // Create organization and user
    let org = Organization::new("Test Org".to_string());
    let org = org_repo
        .create(&org)
        .await
        .expect("Failed to create organization");

    let user = User::new(org.id.clone(), "Test User".to_string(), "test@example.com".to_string(), "hashed_password".to_string(), "UTC".to_string());
    let user = user_repo
        .create(&user)
        .await
        .expect("Failed to create user");

    // Batch insert agents
    let agents: Vec<Agent> = (0..5)
        .map(|i| Agent::new(org.id.clone(), Some(format!("Agent {}", i))))
        .collect();

    let inserted = batch_ops
        .batch_insert_agents(&agents)
        .await
        .expect("Failed to batch insert agents");
    assert_eq!(inserted, 5);

    // Batch insert memories
    let agent_id = agents[0].id.clone();
    let memories: Vec<Memory> = (0..10)
        .map(|i| Memory {
            id: generate_id("memory"),
            organization_id: org.id.clone(),
            user_id: user.id.clone(),
            agent_id: agent_id.clone(),
            content: format!("Memory {}", i),
            hash: None,
            metadata: serde_json::json!({}),
            score: Some(0.5),
            memory_type: "episodic".to_string(),
            scope: "agent".to_string(),
            level: "short_term".to_string(),
            importance: 0.5,
            access_count: 0,
            last_accessed: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            is_deleted: false,
            created_by_id: None,
            last_updated_by_id: None,
        })
        .collect();

    let inserted = batch_ops
        .batch_insert_memories(&memories)
        .await
        .expect("Failed to batch insert memories");
    assert_eq!(inserted, 10);

    // Batch soft delete
    let ids: Vec<String> = agents.iter().map(|a| a.id.clone()).collect();
    let deleted = batch_ops
        .batch_soft_delete("agents", &ids)
        .await
        .expect("Failed to batch soft delete");
    assert_eq!(deleted, 5);

    // Clean up
    for agent in &agents {
        batch_ops
            .batch_soft_delete("agents", &[agent.id.clone()])
            .await
            .ok();
    }
    for memory in &memories {
        batch_ops
            .batch_soft_delete("memories", &[memory.id.clone()])
            .await
            .ok();
    }
    user_repo.hard_delete(&user.id).await.ok();
    org_repo.hard_delete(&org.id).await.ok();
}

#[tokio::test]
#[ignore] // Requires database
async fn test_transaction_manager() {
    let storage = create_test_storage().await;
    let pool = storage.pool().clone();
    let tx_manager = TransactionManager::new(pool.clone());
    let org_repo = OrganizationRepository::new(pool);

    // Test transaction with retry
    let result = tx_manager
        .execute_with_retry(3, 100, 2000, 2.0, || async {
            let org = Organization::new("Test Org".to_string());
            org_repo.create(&org).await
        })
        .await;

    assert!(result.is_ok());
    let org = result.unwrap();

    // Clean up
    org_repo.hard_delete(&org.id).await.ok();
}

#[tokio::test]
#[ignore] // Requires database
async fn test_retry_config() {
    // Test default config
    let default_config = RetryConfig::default();
    assert_eq!(default_config.max_retries, 3);
    assert_eq!(default_config.base_delay_ms, 100);

    // Test aggressive config
    let aggressive_config = RetryConfig::aggressive();
    assert_eq!(aggressive_config.max_retries, 5);
    assert_eq!(aggressive_config.base_delay_ms, 50);

    // Test conservative config
    let conservative_config = RetryConfig::conservative();
    assert_eq!(conservative_config.max_retries, 2);
    assert_eq!(conservative_config.base_delay_ms, 200);
}
