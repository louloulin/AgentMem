//! Integration tests for LibSQL Message Repository

#[cfg(feature = "libsql")]
mod libsql_message_tests {
    use agent_mem_core::storage::libsql::{
        create_libsql_pool, run_migrations, LibSqlAgentRepository, LibSqlMessageRepository,
        LibSqlOrganizationRepository, LibSqlUserRepository,
    };
    use agent_mem_core::storage::models::{Agent, Message, Organization, User};
    use agent_mem_core::storage::traits::{
        AgentRepositoryTrait, MessageRepositoryTrait, OrganizationRepositoryTrait,
        UserRepositoryTrait,
    };
    use serde_json::json;
    use tempfile::TempDir;

    async fn setup_test_db() -> (
        TempDir,
        LibSqlMessageRepository,
        LibSqlOrganizationRepository,
        LibSqlUserRepository,
        LibSqlAgentRepository,
    ) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = create_libsql_pool(db_path.to_str().unwrap())
            .await
            .unwrap();
        run_migrations(conn.clone()).await.unwrap();

        let message_repo = LibSqlMessageRepository::new(conn.clone());
        let org_repo = LibSqlOrganizationRepository::new(conn.clone());
        let user_repo = LibSqlUserRepository::new(conn.clone());
        let agent_repo = LibSqlAgentRepository::new(conn);

        (temp_dir, message_repo, org_repo, user_repo, agent_repo)
    }

    #[tokio::test]
    async fn test_message_create() {
        let (_temp_dir, message_repo, org_repo, user_repo, agent_repo) = setup_test_db().await;

        // Create prerequisites
        let org = Organization::new("Test Org".to_string());
        org_repo.create(&org).await.unwrap();

        let user = User::new(org.id.clone(), "Test User".to_string(), "UTC".to_string());
        user_repo.create(&user).await.unwrap();

        let agent = Agent::new(org.id.clone(), Some("Test Agent".to_string()));
        agent_repo.create(&agent).await.unwrap();

        // Create message
        let message = Message::new(
            org.id.clone(),
            user.id.clone(),
            agent.id.clone(),
            "user".to_string(),
            Some("Hello, world!".to_string()),
        );
        let created = message_repo.create(&message).await.unwrap();
        assert_eq!(created.text, Some("Hello, world!".to_string()));
        assert_eq!(created.role, "user");
    }

    #[tokio::test]
    async fn test_message_find_by_id() {
        let (_temp_dir, message_repo, org_repo, user_repo, agent_repo) = setup_test_db().await;

        // Create prerequisites
        let org = Organization::new("Test Org".to_string());
        org_repo.create(&org).await.unwrap();

        let user = User::new(org.id.clone(), "Test User".to_string(), "UTC".to_string());
        user_repo.create(&user).await.unwrap();

        let agent = Agent::new(org.id.clone(), Some("Test Agent".to_string()));
        agent_repo.create(&agent).await.unwrap();

        // Create message
        let message = Message::new(
            org.id.clone(),
            user.id.clone(),
            agent.id.clone(),
            "user".to_string(),
            Some("Test message".to_string()),
        );
        let created = message_repo.create(&message).await.unwrap();

        // Find by ID
        let found = message_repo.find_by_id(&created.id).await.unwrap();
        assert!(found.is_some());
        let found_message = found.unwrap();
        assert_eq!(found_message.id, created.id);
        assert_eq!(found_message.text, Some("Test message".to_string()));
    }

    #[tokio::test]
    async fn test_message_find_by_agent_id() {
        let (_temp_dir, message_repo, org_repo, user_repo, agent_repo) = setup_test_db().await;

        // Create prerequisites
        let org = Organization::new("Test Org".to_string());
        org_repo.create(&org).await.unwrap();

        let user = User::new(org.id.clone(), "Test User".to_string(), "UTC".to_string());
        user_repo.create(&user).await.unwrap();

        let agent = Agent::new(org.id.clone(), Some("Test Agent".to_string()));
        agent_repo.create(&agent).await.unwrap();

        // Create multiple messages
        for i in 1..=3 {
            let message = Message::new(
                org.id.clone(),
                user.id.clone(),
                agent.id.clone(),
                "user".to_string(),
                Some(format!("Message {}", i)),
            );
            message_repo.create(&message).await.unwrap();
        }

        // Find by agent ID
        let messages = message_repo
            .find_by_agent_id(&agent.id, 10)
            .await
            .unwrap();
        assert_eq!(messages.len(), 3);
    }

    #[tokio::test]
    async fn test_message_find_by_user_id() {
        let (_temp_dir, message_repo, org_repo, user_repo, agent_repo) = setup_test_db().await;

        // Create prerequisites
        let org = Organization::new("Test Org".to_string());
        org_repo.create(&org).await.unwrap();

        let user = User::new(org.id.clone(), "Test User".to_string(), "UTC".to_string());
        user_repo.create(&user).await.unwrap();

        let agent = Agent::new(org.id.clone(), Some("Test Agent".to_string()));
        agent_repo.create(&agent).await.unwrap();

        // Create multiple messages
        for i in 1..=3 {
            let message = Message::new(
                org.id.clone(),
                user.id.clone(),
                agent.id.clone(),
                "user".to_string(),
                Some(format!("Message {}", i)),
            );
            message_repo.create(&message).await.unwrap();
        }

        // Find by user ID
        let messages = message_repo.find_by_user_id(&user.id, 10).await.unwrap();
        assert_eq!(messages.len(), 3);
    }

    #[tokio::test]
    async fn test_message_update() {
        let (_temp_dir, message_repo, org_repo, user_repo, agent_repo) = setup_test_db().await;

        // Create prerequisites
        let org = Organization::new("Test Org".to_string());
        org_repo.create(&org).await.unwrap();

        let user = User::new(org.id.clone(), "Test User".to_string(), "UTC".to_string());
        user_repo.create(&user).await.unwrap();

        let agent = Agent::new(org.id.clone(), Some("Test Agent".to_string()));
        agent_repo.create(&agent).await.unwrap();

        // Create message
        let message = Message::new(
            org.id.clone(),
            user.id.clone(),
            agent.id.clone(),
            "user".to_string(),
            Some("Original text".to_string()),
        );
        let created = message_repo.create(&message).await.unwrap();

        // Update message
        let mut updated_message = created.clone();
        updated_message.text = Some("Updated text".to_string());
        let updated = message_repo.update(&updated_message).await.unwrap();
        assert_eq!(updated.text, Some("Updated text".to_string()));

        // Verify update
        let found = message_repo.find_by_id(&created.id).await.unwrap();
        assert_eq!(found.unwrap().text, Some("Updated text".to_string()));
    }

    #[tokio::test]
    async fn test_message_delete() {
        let (_temp_dir, message_repo, org_repo, user_repo, agent_repo) = setup_test_db().await;

        // Create prerequisites
        let org = Organization::new("Test Org".to_string());
        org_repo.create(&org).await.unwrap();

        let user = User::new(org.id.clone(), "Test User".to_string(), "UTC".to_string());
        user_repo.create(&user).await.unwrap();

        let agent = Agent::new(org.id.clone(), Some("Test Agent".to_string()));
        agent_repo.create(&agent).await.unwrap();

        // Create message
        let message = Message::new(
            org.id.clone(),
            user.id.clone(),
            agent.id.clone(),
            "user".to_string(),
            Some("To be deleted".to_string()),
        );
        let created = message_repo.create(&message).await.unwrap();

        // Delete message
        message_repo.delete(&created.id).await.unwrap();

        // Verify deletion (soft delete)
        let found = message_repo.find_by_id(&created.id).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_message_delete_by_agent_id() {
        let (_temp_dir, message_repo, org_repo, user_repo, agent_repo) = setup_test_db().await;

        // Create prerequisites
        let org = Organization::new("Test Org".to_string());
        org_repo.create(&org).await.unwrap();

        let user = User::new(org.id.clone(), "Test User".to_string(), "UTC".to_string());
        user_repo.create(&user).await.unwrap();

        let agent = Agent::new(org.id.clone(), Some("Test Agent".to_string()));
        agent_repo.create(&agent).await.unwrap();

        // Create multiple messages
        for i in 1..=3 {
            let message = Message::new(
                org.id.clone(),
                user.id.clone(),
                agent.id.clone(),
                "user".to_string(),
                Some(format!("Message {}", i)),
            );
            message_repo.create(&message).await.unwrap();
        }

        // Delete by agent ID
        let deleted_count = message_repo.delete_by_agent_id(&agent.id).await.unwrap();
        assert_eq!(deleted_count, 3);

        // Verify deletion
        let messages = message_repo
            .find_by_agent_id(&agent.id, 10)
            .await
            .unwrap();
        assert_eq!(messages.len(), 0);
    }

    #[tokio::test]
    async fn test_message_with_json_fields() {
        let (_temp_dir, message_repo, org_repo, user_repo, agent_repo) = setup_test_db().await;

        // Create prerequisites
        let org = Organization::new("Test Org".to_string());
        org_repo.create(&org).await.unwrap();

        let user = User::new(org.id.clone(), "Test User".to_string(), "UTC".to_string());
        user_repo.create(&user).await.unwrap();

        let agent = Agent::new(org.id.clone(), Some("Test Agent".to_string()));
        agent_repo.create(&agent).await.unwrap();

        // Create message with JSON fields
        let mut message = Message::new(
            org.id.clone(),
            user.id.clone(),
            agent.id.clone(),
            "assistant".to_string(),
            Some("Message with JSON".to_string()),
        );
        message.content = Some(json!({"type": "text", "text": "Hello"}));
        message.tool_calls = Some(json!([{"name": "search", "args": {}}]));

        let created = message_repo.create(&message).await.unwrap();

        // Verify JSON fields
        let found = message_repo.find_by_id(&created.id).await.unwrap().unwrap();
        assert_eq!(
            found.content,
            Some(json!({"type": "text", "text": "Hello"}))
        );
        assert_eq!(
            found.tool_calls,
            Some(json!([{"name": "search", "args": {}}]))
        );
    }
}

