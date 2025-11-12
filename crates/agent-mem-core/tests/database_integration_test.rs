//! Integration tests for database functionality
//!
//! These tests require a running PostgreSQL database.
//! Set DATABASE_URL environment variable to run these tests.
//!
//! Example:
//! ```bash
//! export DATABASE_URL="postgresql://agentmem:password@localhost:5432/agentmem_test"
//! cargo test --package agent-mem-core --test database_integration_test
//! ```

#![cfg(feature = "postgres")]

use agent_mem_core::storage::{
    agent_repository::AgentRepository,
    block_repository::BlockRepository,
    message_repository::MessageRepository,
    models::*,
    postgres::PostgresStorage,
    repository::{OrganizationRepository, Repository, UserRepository},
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

    // Run migrations
    storage.migrate().await.expect("Failed to run migrations");

    storage
}

#[tokio::test]
#[ignore] // Requires database
async fn test_organization_crud() {
    let storage = create_test_storage().await;
    let pool = storage.pool().clone();
    let repo = OrganizationRepository::new(pool);

    // Create organization
    let org = Organization::new("Test Organization".to_string());
    let created_org = repo
        .create(&org)
        .await
        .expect("Failed to create organization");

    assert_eq!(created_org.name, "Test Organization");
    assert!(!created_org.is_deleted);

    // Read organization
    let read_org = repo
        .read(&created_org.id)
        .await
        .expect("Failed to read organization")
        .expect("Organization not found");

    assert_eq!(read_org.id, created_org.id);
    assert_eq!(read_org.name, created_org.name);

    // Update organization
    let mut updated_org = read_org.clone();
    updated_org.name = "Updated Organization".to_string();
    let updated_org = repo
        .update(&updated_org)
        .await
        .expect("Failed to update organization");

    assert_eq!(updated_org.name, "Updated Organization");

    // List organizations
    let orgs = repo
        .list(Some(10), Some(0))
        .await
        .expect("Failed to list organizations");
    assert!(!orgs.is_empty());

    // Count organizations
    let count = repo.count().await.expect("Failed to count organizations");
    assert!(count > 0);

    // Delete organization (soft delete)
    let deleted = repo
        .delete(&created_org.id)
        .await
        .expect("Failed to delete organization");
    assert!(deleted);

    // Verify soft delete
    let deleted_org = repo
        .read(&created_org.id)
        .await
        .expect("Failed to read organization");
    assert!(deleted_org.is_none());

    // Hard delete
    let hard_deleted = repo
        .hard_delete(&created_org.id)
        .await
        .expect("Failed to hard delete organization");
    assert!(hard_deleted);
}

#[tokio::test]
#[ignore] // Requires database
async fn test_user_crud() {
    let storage = create_test_storage().await;
    let pool = storage.pool().clone();
    let org_repo = OrganizationRepository::new(pool.clone());
    let user_repo = UserRepository::new(pool);

    // Create organization first
    let org = Organization::new("Test Org".to_string());
    let org = org_repo
        .create(&org)
        .await
        .expect("Failed to create organization");

    // Create user
    let user = User::new(
        org.id.clone(),
        "Test User".to_string(),
        "test@example.com".to_string(),
        "hashed_password".to_string(),
        "UTC".to_string(),
    );
    let created_user = user_repo
        .create(&user)
        .await
        .expect("Failed to create user");

    assert_eq!(created_user.name, "Test User");
    assert_eq!(created_user.organization_id, org.id);
    assert_eq!(created_user.status, "active");

    // Read user
    let read_user = user_repo
        .read(&created_user.id)
        .await
        .expect("Failed to read user")
        .expect("User not found");

    assert_eq!(read_user.id, created_user.id);

    // List users by organization
    let users = user_repo
        .list_by_organization(&org.id, Some(10), Some(0))
        .await
        .expect("Failed to list users");

    assert!(!users.is_empty());

    // Clean up
    user_repo.hard_delete(&created_user.id).await.ok();
    org_repo.hard_delete(&org.id).await.ok();
}

#[tokio::test]
#[ignore] // Requires database
async fn test_agent_crud_with_blocks() {
    let storage = create_test_storage().await;
    let pool = storage.pool().clone();
    let org_repo = OrganizationRepository::new(pool.clone());
    let user_repo = UserRepository::new(pool.clone());
    let agent_repo = AgentRepository::new(pool.clone());
    let block_repo = BlockRepository::new(pool);

    // Create organization and user
    let org = Organization::new("Test Org".to_string());
    let org = org_repo
        .create(&org)
        .await
        .expect("Failed to create organization");

    let user = User::new(
        org.id.clone(),
        "Test User".to_string(),
        "test@example.com".to_string(),
        "hashed_password".to_string(),
        "UTC".to_string(),
    );
    let user = user_repo
        .create(&user)
        .await
        .expect("Failed to create user");

    // Create agent
    let agent = Agent::new(org.id.clone(), Some("Test Agent".to_string()));
    let created_agent = agent_repo
        .create(&agent)
        .await
        .expect("Failed to create agent");

    assert_eq!(created_agent.name, Some("Test Agent".to_string()));
    assert_eq!(created_agent.organization_id, org.id);

    // Create blocks (Core Memory)
    let human_block = Block::new(
        org.id.clone(),
        user.id.clone(),
        "human".to_string(),
        "User is a software engineer".to_string(),
        2000,
    );
    let human_block = block_repo
        .create_validated(&human_block)
        .await
        .expect("Failed to create human block");

    let persona_block = Block::new(
        org.id.clone(),
        user.id.clone(),
        "persona".to_string(),
        "I am a helpful AI assistant".to_string(),
        2000,
    );
    let persona_block = block_repo
        .create_validated(&persona_block)
        .await
        .expect("Failed to create persona block");

    // Link blocks to agent
    agent_repo
        .add_block(&created_agent.id, &human_block.id, &human_block.label)
        .await
        .expect("Failed to add human block to agent");

    agent_repo
        .add_block(&created_agent.id, &persona_block.id, &persona_block.label)
        .await
        .expect("Failed to add persona block to agent");

    // Get agent with blocks
    let (agent, blocks) = agent_repo
        .get_with_blocks(&created_agent.id)
        .await
        .expect("Failed to get agent with blocks")
        .expect("Agent not found");

    assert_eq!(agent.id, created_agent.id);
    assert_eq!(blocks.len(), 2);

    // Verify block labels
    let labels: Vec<String> = blocks.iter().map(|b| b.label.clone()).collect();
    assert!(labels.contains(&"human".to_string()));
    assert!(labels.contains(&"persona".to_string()));

    // Clean up
    agent_repo.hard_delete(&created_agent.id).await.ok();
    block_repo.hard_delete(&human_block.id).await.ok();
    block_repo.hard_delete(&persona_block.id).await.ok();
    user_repo.hard_delete(&user.id).await.ok();
    org_repo.hard_delete(&org.id).await.ok();
}

#[tokio::test]
#[ignore] // Requires database
async fn test_message_crud() {
    let storage = create_test_storage().await;
    let pool = storage.pool().clone();
    let org_repo = OrganizationRepository::new(pool.clone());
    let user_repo = UserRepository::new(pool.clone());
    let agent_repo = AgentRepository::new(pool.clone());
    let message_repo = MessageRepository::new(pool);

    // Create organization, user, and agent
    let org = Organization::new("Test Org".to_string());
    let org = org_repo
        .create(&org)
        .await
        .expect("Failed to create organization");

    let user = User::new(
        org.id.clone(),
        "Test User".to_string(),
        "test@example.com".to_string(),
        "hashed_password".to_string(),
        "UTC".to_string(),
    );
    let user = user_repo
        .create(&user)
        .await
        .expect("Failed to create user");

    let agent = Agent::new(org.id.clone(), Some("Test Agent".to_string()));
    let agent = agent_repo
        .create(&agent)
        .await
        .expect("Failed to create agent");

    // Create messages
    let msg1 = Message::new(
        org.id.clone(),
        user.id.clone(),
        agent.id.clone(),
        "user".to_string(),
        Some("Hello, how are you?".to_string()),
    );
    let msg1 = message_repo
        .create(&msg1)
        .await
        .expect("Failed to create message 1");

    let msg2 = Message::new(
        org.id.clone(),
        user.id.clone(),
        agent.id.clone(),
        "assistant".to_string(),
        Some("I'm doing well, thank you!".to_string()),
    );
    let msg2 = message_repo
        .create(&msg2)
        .await
        .expect("Failed to create message 2");

    // List messages by agent
    let messages = message_repo
        .list_by_agent(&agent.id, Some(10), Some(0))
        .await
        .expect("Failed to list messages");

    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].role, "user");
    assert_eq!(messages[1].role, "assistant");

    // Count messages
    let count = message_repo
        .count_by_agent(&agent.id)
        .await
        .expect("Failed to count messages");

    assert_eq!(count, 2);

    // Get recent messages
    let recent = message_repo
        .get_recent_messages(&agent.id, 1)
        .await
        .expect("Failed to get recent messages");

    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].role, "assistant");

    // Clean up
    message_repo.hard_delete(&msg1.id).await.ok();
    message_repo.hard_delete(&msg2.id).await.ok();
    agent_repo.hard_delete(&agent.id).await.ok();
    user_repo.hard_delete(&user.id).await.ok();
    org_repo.hard_delete(&org.id).await.ok();
}

#[tokio::test]
#[ignore] // Requires database
async fn test_block_validation() {
    let storage = create_test_storage().await;
    let pool = storage.pool().clone();
    let org_repo = OrganizationRepository::new(pool.clone());
    let user_repo = UserRepository::new(pool.clone());
    let block_repo = BlockRepository::new(pool);

    // Create organization and user
    let org = Organization::new("Test Org".to_string());
    let org = org_repo
        .create(&org)
        .await
        .expect("Failed to create organization");

    let user = User::new(
        org.id.clone(),
        "Test User".to_string(),
        "test@example.com".to_string(),
        "hashed_password".to_string(),
        "UTC".to_string(),
    );
    let user = user_repo
        .create(&user)
        .await
        .expect("Failed to create user");

    // Create block with value exceeding limit
    let long_value = "a".repeat(3000); // Exceeds default limit of 2000
    let block = Block::new(
        org.id.clone(),
        user.id.clone(),
        "human".to_string(),
        long_value,
        2000,
    );

    // Should fail validation
    let result = block_repo.create_validated(&block).await;
    assert!(result.is_err());

    // Clean up
    user_repo.hard_delete(&user.id).await.ok();
    org_repo.hard_delete(&org.id).await.ok();
}
