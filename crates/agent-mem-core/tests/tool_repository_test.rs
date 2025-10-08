#[cfg(feature = "libsql")]
mod libsql_tool_tests {
    use agent_mem_core::storage::libsql::{
        create_libsql_pool, run_migrations, LibSqlOrganizationRepository, LibSqlToolRepository,
    };
    use agent_mem_core::storage::models::{Organization, Tool};
    use agent_mem_core::storage::traits::{OrganizationRepositoryTrait, ToolRepositoryTrait};
    use serde_json::json;
    use std::sync::Arc;
    use tempfile::TempDir;

    async fn setup_test_db() -> (TempDir, LibSqlToolRepository, LibSqlOrganizationRepository) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db_url = format!("file:{}", db_path.display());

        let conn = create_libsql_pool(&db_url).await.unwrap();
        run_migrations(conn.clone()).await.unwrap();

        let tool_repo = LibSqlToolRepository::new(conn.clone());
        let org_repo = LibSqlOrganizationRepository::new(conn.clone());

        (temp_dir, tool_repo, org_repo)
    }

    #[tokio::test]
    async fn test_tool_create() {
        let (_temp_dir, tool_repo, org_repo) = setup_test_db().await;

        // Create organization first
        let org = Organization::new("Test Org".to_string());
        org_repo.create(&org).await.unwrap();

        // Create tool
        let tool = Tool::new(org.id.clone(), "test_tool".to_string());
        let created = tool_repo.create(&tool).await.unwrap();

        assert_eq!(created.id, tool.id);
        assert_eq!(created.name, "test_tool");
        assert_eq!(created.organization_id, org.id);
    }

    #[tokio::test]
    async fn test_tool_find_by_id() {
        let (_temp_dir, tool_repo, org_repo) = setup_test_db().await;

        // Create organization
        let org = Organization::new("Test Org".to_string());
        org_repo.create(&org).await.unwrap();

        // Create tool
        let tool = Tool::new(org.id.clone(), "test_tool".to_string());
        let created = tool_repo.create(&tool).await.unwrap();

        // Find by ID
        let found = tool_repo.find_by_id(&created.id).await.unwrap();
        assert!(found.is_some());
        let found_tool = found.unwrap();
        assert_eq!(found_tool.id, created.id);
        assert_eq!(found_tool.name, "test_tool");
    }

    #[tokio::test]
    async fn test_tool_find_by_organization_id() {
        let (_temp_dir, tool_repo, org_repo) = setup_test_db().await;

        // Create organization
        let org = Organization::new("Test Org".to_string());
        org_repo.create(&org).await.unwrap();

        // Create multiple tools
        for i in 1..=3 {
            let tool = Tool::new(org.id.clone(), format!("tool_{}", i));
            tool_repo.create(&tool).await.unwrap();
        }

        // Find by organization ID
        let tools = tool_repo
            .find_by_organization_id(&org.id)
            .await
            .unwrap();
        assert_eq!(tools.len(), 3);
    }

    #[tokio::test]
    async fn test_tool_update() {
        let (_temp_dir, tool_repo, org_repo) = setup_test_db().await;

        // Create organization
        let org = Organization::new("Test Org".to_string());
        org_repo.create(&org).await.unwrap();

        // Create tool
        let mut tool = Tool::new(org.id.clone(), "test_tool".to_string());
        let created = tool_repo.create(&tool).await.unwrap();

        // Update tool
        tool.id = created.id.clone();
        tool.name = "updated_tool".to_string();
        tool.description = Some("Updated description".to_string());
        let updated = tool_repo.update(&tool).await.unwrap();

        assert_eq!(updated.name, "updated_tool");
        assert_eq!(updated.description, Some("Updated description".to_string()));
    }

    #[tokio::test]
    async fn test_tool_delete() {
        let (_temp_dir, tool_repo, org_repo) = setup_test_db().await;

        // Create organization
        let org = Organization::new("Test Org".to_string());
        org_repo.create(&org).await.unwrap();

        // Create tool
        let tool = Tool::new(org.id.clone(), "test_tool".to_string());
        let created = tool_repo.create(&tool).await.unwrap();

        // Delete tool
        tool_repo.delete(&created.id).await.unwrap();

        // Verify deletion (soft delete)
        let found = tool_repo.find_by_id(&created.id).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_tool_list() {
        let (_temp_dir, tool_repo, org_repo) = setup_test_db().await;

        // Create organization
        let org = Organization::new("Test Org".to_string());
        org_repo.create(&org).await.unwrap();

        // Create multiple tools
        for i in 1..=5 {
            let tool = Tool::new(org.id.clone(), format!("tool_{}", i));
            tool_repo.create(&tool).await.unwrap();
        }

        // List tools with pagination
        let tools = tool_repo.list(3, 0).await.unwrap();
        assert_eq!(tools.len(), 3);

        let tools_page2 = tool_repo.list(3, 3).await.unwrap();
        assert_eq!(tools_page2.len(), 2);
    }

    #[tokio::test]
    async fn test_tool_with_json_fields() {
        let (_temp_dir, tool_repo, org_repo) = setup_test_db().await;

        // Create organization
        let org = Organization::new("Test Org".to_string());
        org_repo.create(&org).await.unwrap();

        // Create tool with JSON fields
        let mut tool = Tool::new(org.id.clone(), "test_tool".to_string());
        tool.json_schema = Some(json!({
            "type": "object",
            "properties": {
                "input": {"type": "string"}
            }
        }));
        tool.tags = Some(vec!["tag1".to_string(), "tag2".to_string()]);
        tool.metadata_ = Some(json!({"key": "value"}));

        let created = tool_repo.create(&tool).await.unwrap();

        // Verify JSON fields
        let found = tool_repo.find_by_id(&created.id).await.unwrap().unwrap();
        assert_eq!(
            found.json_schema,
            Some(json!({
                "type": "object",
                "properties": {
                    "input": {"type": "string"}
                }
            }))
        );
        assert_eq!(
            found.tags,
            Some(vec!["tag1".to_string(), "tag2".to_string()])
        );
        assert_eq!(found.metadata_, Some(json!({"key": "value"})));
    }

    #[tokio::test]
    async fn test_tool_with_source_code() {
        let (_temp_dir, tool_repo, org_repo) = setup_test_db().await;

        // Create organization
        let org = Organization::new("Test Org".to_string());
        org_repo.create(&org).await.unwrap();

        // Create tool with source code
        let mut tool = Tool::new(org.id.clone(), "python_tool".to_string());
        tool.source_type = Some("python".to_string());
        tool.source_code = Some("def hello(): return 'Hello'".to_string());
        tool.description = Some("A Python tool".to_string());

        let created = tool_repo.create(&tool).await.unwrap();

        // Verify source code fields
        let found = tool_repo.find_by_id(&created.id).await.unwrap().unwrap();
        assert_eq!(found.source_type, Some("python".to_string()));
        assert_eq!(
            found.source_code,
            Some("def hello(): return 'Hello'".to_string())
        );
        assert_eq!(found.description, Some("A Python tool".to_string()));
    }
}

