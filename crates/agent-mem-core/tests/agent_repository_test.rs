//! Integration tests for LibSQL Agent Repository

#[cfg(feature = "libsql")]
mod libsql_agent_tests {
    use agent_mem_core::storage::libsql::{
        create_libsql_pool, run_migrations, LibSqlAgentRepository, LibSqlOrganizationRepository,
    };
    use agent_mem_core::storage::models::{Agent, Organization};
    use agent_mem_core::storage::traits::{AgentRepositoryTrait, OrganizationRepositoryTrait};
    use serde_json::json;
    use tempfile::TempDir;

    async fn setup_test_db() -> (TempDir, LibSqlAgentRepository, LibSqlOrganizationRepository) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = create_libsql_pool(db_path.to_str().unwrap()).await.unwrap();
        run_migrations(conn.clone()).await.unwrap();

        let agent_repo = LibSqlAgentRepository::new(conn.clone());
        let org_repo = LibSqlOrganizationRepository::new(conn);

        (temp_dir, agent_repo, org_repo)
    }

    #[tokio::test]
    async fn test_agent_create() {
        let (_temp_dir, agent_repo, org_repo) = setup_test_db().await;

        // Create organization first
        let org = Organization::new("Test Org".to_string());
        org_repo.create(&org).await.unwrap();

        // Create agent
        let agent = Agent::new(org.id.clone(), Some("Test Agent".to_string()));
        let created = agent_repo.create(&agent).await.unwrap();
        assert_eq!(created.name, Some("Test Agent".to_string()));
        assert_eq!(created.organization_id, org.id);
    }

    #[tokio::test]
    async fn test_agent_find_by_id() {
        let (_temp_dir, agent_repo, org_repo) = setup_test_db().await;

        // Create organization
        let org = Organization::new("Test Org".to_string());
        org_repo.create(&org).await.unwrap();

        // Create agent
        let agent = Agent::new(org.id.clone(), Some("Find By ID Agent".to_string()));
        let created = agent_repo.create(&agent).await.unwrap();

        // Find by ID
        let found = agent_repo.find_by_id(&created.id).await.unwrap();
        assert!(found.is_some());
        let found_agent = found.unwrap();
        assert_eq!(found_agent.id, created.id);
        assert_eq!(found_agent.name, Some("Find By ID Agent".to_string()));
    }

    #[tokio::test]
    async fn test_agent_find_by_organization_id() {
        let (_temp_dir, agent_repo, org_repo) = setup_test_db().await;

        // Create organization
        let org = Organization::new("Test Org".to_string());
        org_repo.create(&org).await.unwrap();

        // Create multiple agents
        for i in 1..=3 {
            let agent = Agent::new(org.id.clone(), Some(format!("Agent {}", i)));
            agent_repo.create(&agent).await.unwrap();
        }

        // Find by organization ID
        let agents = agent_repo.find_by_organization_id(&org.id).await.unwrap();
        assert_eq!(agents.len(), 3);
    }

    #[tokio::test]
    async fn test_agent_update() {
        let (_temp_dir, agent_repo, org_repo) = setup_test_db().await;

        // Create organization
        let org = Organization::new("Test Org".to_string());
        org_repo.create(&org).await.unwrap();

        // Create agent
        let agent = Agent::new(org.id.clone(), Some("Original Name".to_string()));
        let created = agent_repo.create(&agent).await.unwrap();

        // Update agent
        let mut updated_agent = created.clone();
        updated_agent.name = Some("Updated Name".to_string());
        updated_agent.description = Some("Updated description".to_string());
        let updated = agent_repo.update(&updated_agent).await.unwrap();
        assert_eq!(updated.name, Some("Updated Name".to_string()));

        // Verify update
        let found = agent_repo.find_by_id(&created.id).await.unwrap();
        assert_eq!(found.unwrap().name, Some("Updated Name".to_string()));
    }

    #[tokio::test]
    async fn test_agent_delete() {
        let (_temp_dir, agent_repo, org_repo) = setup_test_db().await;

        // Create organization
        let org = Organization::new("Test Org".to_string());
        org_repo.create(&org).await.unwrap();

        // Create agent
        let agent = Agent::new(org.id.clone(), Some("To Be Deleted".to_string()));
        let created = agent_repo.create(&agent).await.unwrap();

        // Delete agent
        agent_repo.delete(&created.id).await.unwrap();

        // Verify deletion (soft delete)
        let found = agent_repo.find_by_id(&created.id).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_agent_list() {
        let (_temp_dir, agent_repo, org_repo) = setup_test_db().await;

        // Create organization
        let org = Organization::new("Test Org".to_string());
        org_repo.create(&org).await.unwrap();

        // Create multiple agents
        for i in 1..=5 {
            let agent = Agent::new(org.id.clone(), Some(format!("Agent {}", i)));
            agent_repo.create(&agent).await.unwrap();
        }

        // List agents
        let agents = agent_repo.list(10, 0).await.unwrap();
        assert_eq!(agents.len(), 5);

        // Test pagination
        let page1 = agent_repo.list(2, 0).await.unwrap();
        assert_eq!(page1.len(), 2);

        let page2 = agent_repo.list(2, 2).await.unwrap();
        assert_eq!(page2.len(), 2);
    }

    #[tokio::test]
    async fn test_agent_with_json_fields() {
        let (_temp_dir, agent_repo, org_repo) = setup_test_db().await;

        // Create organization
        let org = Organization::new("Test Org".to_string());
        org_repo.create(&org).await.unwrap();

        // Create agent with JSON fields
        let mut agent = Agent::new(org.id.clone(), Some("Agent with JSON".to_string()));
        agent.metadata_ = Some(json!({"key": "value"}));
        agent.llm_config = Some(json!({"model": "gpt-4", "temperature": 0.7}));
        agent.message_ids = Some(vec!["msg1".to_string(), "msg2".to_string()]);
        agent.mcp_tools = Some(vec!["tool1".to_string(), "tool2".to_string()]);

        let created = agent_repo.create(&agent).await.unwrap();

        // Verify JSON fields
        let found = agent_repo.find_by_id(&created.id).await.unwrap().unwrap();
        assert_eq!(found.metadata_, Some(json!({"key": "value"})));
        assert_eq!(
            found.llm_config,
            Some(json!({"model": "gpt-4", "temperature": 0.7}))
        );
        assert_eq!(
            found.message_ids,
            Some(vec!["msg1".to_string(), "msg2".to_string()])
        );
        assert_eq!(
            found.mcp_tools,
            Some(vec!["tool1".to_string(), "tool2".to_string()])
        );
    }

    #[tokio::test]
    async fn test_agent_not_found() {
        let (_temp_dir, agent_repo, _org_repo) = setup_test_db().await;

        // Try to find non-existent agent
        let found = agent_repo.find_by_id("non-existent-id").await.unwrap();
        assert!(found.is_none());
    }
}
