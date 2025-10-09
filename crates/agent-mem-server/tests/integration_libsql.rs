//! Integration tests for LibSQL backend
//!
//! Tests all routes and repository operations with LibSQL database

use agent_mem_config::DatabaseConfig;
use agent_mem_core::storage::factory::RepositoryFactory;
use agent_mem_core::storage::models::{Agent, Organization, User};
use std::sync::Arc;

#[tokio::test]
async fn test_libsql_repository_factory() {
    // Create LibSQL config
    let config = DatabaseConfig::libsql(":memory:");

    // Create repositories
    let result = RepositoryFactory::create_repositories(&config).await;
    assert!(result.is_ok(), "Failed to create repositories: {:?}", result.err());

    let repositories = result.unwrap();

    // Verify all repositories are created
    assert!(Arc::strong_count(&repositories.users) > 0);
    assert!(Arc::strong_count(&repositories.organizations) > 0);
    assert!(Arc::strong_count(&repositories.agents) > 0);
    assert!(Arc::strong_count(&repositories.messages) > 0);
    assert!(Arc::strong_count(&repositories.tools) > 0);
}

#[tokio::test]
async fn test_user_crud_operations() {
    // Setup
    let config = DatabaseConfig::libsql(":memory:");

    let repositories = RepositoryFactory::create_repositories(&config).await.unwrap();
    let user_repo = repositories.users.clone();
    
    // Create organization first
    let org = Organization::new("Test Org".to_string());
    let org = repositories.organizations.create(&org).await.unwrap();
    
    // Create user
    let user = User::new(
        org.id.clone(),
        "Test User".to_string(),
        "test@example.com".to_string(),
        "hashed_password".to_string(),
        "UTC".to_string(),
    );
    
    let created_user = user_repo.create(&user).await.unwrap();
    assert_eq!(created_user.name, "Test User");
    assert_eq!(created_user.email, "test@example.com");
    
    // Read user
    let found_user = user_repo.find_by_id(&created_user.id).await.unwrap();
    assert!(found_user.is_some());
    assert_eq!(found_user.unwrap().id, created_user.id);
    
    // Find by email
    let found_by_email = user_repo.find_by_email("test@example.com", &org.id).await.unwrap();
    assert!(found_by_email.is_some());
    assert_eq!(found_by_email.unwrap().email, "test@example.com");
    
    // Check email exists
    let exists = user_repo.email_exists("test@example.com", &org.id).await.unwrap();
    assert!(exists);
    
    let not_exists = user_repo.email_exists("nonexistent@example.com", &org.id).await.unwrap();
    assert!(!not_exists);
    
    // Update password
    let result = user_repo.update_password(&created_user.id, "new_hashed_password").await;
    assert!(result.is_ok());
    
    // Delete user
    let result = user_repo.delete(&created_user.id).await;
    assert!(result.is_ok());
    
    // Verify deleted
    let deleted_user = user_repo.find_by_id(&created_user.id).await.unwrap();
    assert!(deleted_user.is_none() || deleted_user.unwrap().is_deleted);
}

#[tokio::test]
async fn test_organization_crud_operations() {
    let config = DatabaseConfig::libsql(":memory:");

    let repositories = RepositoryFactory::create_repositories(&config).await.unwrap();
    let org_repo = repositories.organizations.clone();
    
    // Create
    let org = Organization::new("Test Organization".to_string());
    let created_org = org_repo.create(&org).await.unwrap();
    assert_eq!(created_org.name, "Test Organization");
    
    // Read
    let found_org = org_repo.find_by_id(&created_org.id).await.unwrap();
    assert!(found_org.is_some());
    assert_eq!(found_org.unwrap().name, "Test Organization");
    
    // Update
    let mut updated_org = created_org.clone();
    updated_org.name = "Updated Organization".to_string();
    let result = org_repo.update(&updated_org).await;
    assert!(result.is_ok());
    
    let found_updated = org_repo.find_by_id(&created_org.id).await.unwrap().unwrap();
    assert_eq!(found_updated.name, "Updated Organization");
    
    // Delete
    let result = org_repo.delete(&created_org.id).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_agent_crud_operations() {
    let config = DatabaseConfig::libsql(":memory:");

    let repositories = RepositoryFactory::create_repositories(&config).await.unwrap();
    
    // Create organization
    let org = Organization::new("Test Org".to_string());
    let org = repositories.organizations.create(&org).await.unwrap();
    
    // Create agent
    let agent = Agent::new(
        org.id.clone(),
        Some("Test Agent".to_string()),
    );

    let created_agent = repositories.agents.create(&agent).await.unwrap();
    assert_eq!(created_agent.name, Some("Test Agent".to_string()));
    
    // Find by ID
    let found_agent = repositories.agents.find_by_id(&created_agent.id).await.unwrap();
    assert!(found_agent.is_some());
    
    // Find by organization
    let org_agents = repositories.agents.find_by_organization_id(&org.id).await.unwrap();
    assert_eq!(org_agents.len(), 1);
    assert_eq!(org_agents[0].id, created_agent.id);
    
    // Update
    let mut updated_agent = created_agent.clone();
    updated_agent.name = Some("Updated Agent".to_string());
    let result = repositories.agents.update(&updated_agent).await;
    assert!(result.is_ok());
    
    // Delete
    let result = repositories.agents.delete(&created_agent.id).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_message_operations() {
    let config = DatabaseConfig::libsql(":memory:");

    let repositories = RepositoryFactory::create_repositories(&config).await.unwrap();
    
    // Create organization and agent
    let org = Organization::new("Test Org".to_string());
    let org = repositories.organizations.create(&org).await.unwrap();
    
    let agent = Agent::new(org.id.clone(), Some("Test Agent".to_string()));
    let agent = repositories.agents.create(&agent).await.unwrap();

    // Create user
    let user = User::new(
        org.id.clone(),
        "Test User".to_string(),
        "test@example.com".to_string(),
        "password".to_string(),
        "UTC".to_string(),
    );
    let user = repositories.users.create(&user).await.unwrap();

    // Create message
    use agent_mem_core::storage::models::Message;
    let message = Message::new(
        org.id.clone(),
        user.id.clone(),
        agent.id.clone(),
        "user".to_string(),
        Some("Hello, agent!".to_string()),
    );

    let created_message = repositories.messages.create(&message).await.unwrap();
    assert_eq!(created_message.text, Some("Hello, agent!".to_string()));
    
    // Find by agent
    let agent_messages = repositories.messages.find_by_agent_id(&agent.id, 10).await.unwrap();
    assert_eq!(agent_messages.len(), 1);
    
    // Find by user
    let user_messages = repositories.messages.find_by_user_id(&user.id, 10).await.unwrap();
    assert_eq!(user_messages.len(), 1);
}

#[tokio::test]
async fn test_tool_operations() {
    let config = DatabaseConfig::libsql(":memory:");

    let repositories = RepositoryFactory::create_repositories(&config).await.unwrap();
    
    // Create organization
    let org = Organization::new("Test Org".to_string());
    let org = repositories.organizations.create(&org).await.unwrap();
    
    // Create tool
    use agent_mem_core::storage::models::Tool;
    let mut tool = Tool::new(
        org.id.clone(),
        "test_tool".to_string(),
    );
    tool.description = Some("A test tool".to_string());
    tool.tags = Some(vec!["testing".to_string(), "demo".to_string()]);

    let created_tool = repositories.tools.create(&tool).await.unwrap();
    assert_eq!(created_tool.name, "test_tool");
    
    // Find by tags
    let tools_by_tag = repositories.tools.find_by_tags(&org.id, &["testing".to_string()]).await.unwrap();
    assert_eq!(tools_by_tag.len(), 1);
    
    // Find by organization
    let org_tools = repositories.tools.find_by_organization_id(&org.id).await.unwrap();
    assert_eq!(org_tools.len(), 1);
}

#[tokio::test]
async fn test_concurrent_operations() {
    let config = DatabaseConfig::libsql(":memory:");

    let repositories = Arc::new(RepositoryFactory::create_repositories(&config).await.unwrap());
    
    // Create organization
    let org = Organization::new("Test Org".to_string());
    let org = repositories.organizations.create(&org).await.unwrap();
    
    // Spawn multiple concurrent user creation tasks
    let mut handles = vec![];
    for i in 0..5 {
        let repos = repositories.clone();
        let org_id = org.id.clone();
        
        let handle = tokio::spawn(async move {
            let user = User::new(
                org_id,
                format!("User {}", i),
                format!("user{}@example.com", i),
                "password".to_string(),
                "UTC".to_string(),
            );
            repos.users.create(&user).await
        });
        
        handles.push(handle);
    }
    
    // Wait for all tasks
    let results: Vec<_> = futures::future::join_all(handles).await;
    
    // Verify all succeeded
    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }
    
    // Verify all users were created
    let users = repositories.users.find_by_organization_id(&org.id).await.unwrap();
    assert_eq!(users.len(), 5);
}

