//! Integration tests for LibSQL Organization Repository

#[cfg(feature = "libsql")]
mod libsql_organization_tests {
    use agent_mem_core::storage::libsql::{
        create_libsql_pool, run_migrations, LibSqlOrganizationRepository,
    };
    use agent_mem_core::storage::models::Organization;
    use agent_mem_core::storage::traits::OrganizationRepositoryTrait;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_organization_create() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = create_libsql_pool(db_path.to_str().unwrap())
            .await
            .unwrap();
        run_migrations(conn.clone()).await.unwrap();

        let repo = LibSqlOrganizationRepository::new(conn);

        // Create organization
        let org = Organization::new("Test Organization".to_string());
        let created = repo.create(&org).await.unwrap();
        assert_eq!(created.name, "Test Organization");
        assert_eq!(created.id, org.id);
    }

    #[tokio::test]
    async fn test_organization_find_by_id() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = create_libsql_pool(db_path.to_str().unwrap())
            .await
            .unwrap();
        run_migrations(conn.clone()).await.unwrap();

        let repo = LibSqlOrganizationRepository::new(conn);

        // Create organization
        let org = Organization::new("Find By ID Org".to_string());
        let created = repo.create(&org).await.unwrap();

        // Find by ID
        let found = repo.find_by_id(&created.id).await.unwrap();
        assert!(found.is_some());
        let found_org = found.unwrap();
        assert_eq!(found_org.id, created.id);
        assert_eq!(found_org.name, "Find By ID Org");
    }

    #[tokio::test]
    async fn test_organization_find_by_name() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = create_libsql_pool(db_path.to_str().unwrap())
            .await
            .unwrap();
        run_migrations(conn.clone()).await.unwrap();

        let repo = LibSqlOrganizationRepository::new(conn);

        // Create organization
        let org = Organization::new("Unique Name Org".to_string());
        repo.create(&org).await.unwrap();

        // Find by name
        let found = repo.find_by_name("Unique Name Org").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Unique Name Org");
    }

    #[tokio::test]
    async fn test_organization_update() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = create_libsql_pool(db_path.to_str().unwrap())
            .await
            .unwrap();
        run_migrations(conn.clone()).await.unwrap();

        let repo = LibSqlOrganizationRepository::new(conn);

        // Create organization
        let org = Organization::new("Original Name".to_string());
        let created = repo.create(&org).await.unwrap();

        // Update organization
        let mut updated_org = created.clone();
        updated_org.name = "Updated Name".to_string();
        let updated = repo.update(&updated_org).await.unwrap();
        assert_eq!(updated.name, "Updated Name");

        // Verify update
        let found = repo.find_by_id(&created.id).await.unwrap();
        assert_eq!(found.unwrap().name, "Updated Name");
    }

    #[tokio::test]
    async fn test_organization_delete() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = create_libsql_pool(db_path.to_str().unwrap())
            .await
            .unwrap();
        run_migrations(conn.clone()).await.unwrap();

        let repo = LibSqlOrganizationRepository::new(conn);

        // Create organization
        let org = Organization::new("To Be Deleted".to_string());
        let created = repo.create(&org).await.unwrap();

        // Delete organization
        repo.delete(&created.id).await.unwrap();

        // Verify deletion (soft delete)
        let found = repo.find_by_id(&created.id).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_organization_list() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = create_libsql_pool(db_path.to_str().unwrap())
            .await
            .unwrap();
        run_migrations(conn.clone()).await.unwrap();

        let repo = LibSqlOrganizationRepository::new(conn);

        // Create multiple organizations
        for i in 1..=5 {
            let org = Organization::new(format!("Org {}", i));
            repo.create(&org).await.unwrap();
        }

        // List organizations
        let orgs = repo.list(10, 0).await.unwrap();
        assert_eq!(orgs.len(), 5);

        // Test pagination
        let page1 = repo.list(2, 0).await.unwrap();
        assert_eq!(page1.len(), 2);

        let page2 = repo.list(2, 2).await.unwrap();
        assert_eq!(page2.len(), 2);
    }

    #[tokio::test]
    async fn test_organization_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = create_libsql_pool(db_path.to_str().unwrap())
            .await
            .unwrap();
        run_migrations(conn.clone()).await.unwrap();

        let repo = LibSqlOrganizationRepository::new(conn);

        // Try to find non-existent organization
        let found = repo.find_by_id("non-existent-id").await.unwrap();
        assert!(found.is_none());

        let found_by_name = repo.find_by_name("Non Existent Org").await.unwrap();
        assert!(found_by_name.is_none());
    }
}

