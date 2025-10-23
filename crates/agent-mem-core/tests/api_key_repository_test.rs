#[cfg(feature = "libsql")]
mod libsql_api_key_tests {
    use agent_mem_core::storage::libsql::{create_libsql_pool, run_migrations, LibSqlApiKeyRepository};
    use agent_mem_core::storage::models::{ApiKey, Organization, User};
    use agent_mem_core::storage::libsql::{LibSqlOrganizationRepository, LibSqlUserRepository};
    use agent_mem_core::storage::traits::{ApiKeyRepositoryTrait, OrganizationRepositoryTrait, UserRepositoryTrait};
    use chrono::Utc;
    use tempfile::TempDir;

    async fn setup_test_db() -> (TempDir, LibSqlApiKeyRepository, LibSqlUserRepository, LibSqlOrganizationRepository) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = create_libsql_pool(db_path.to_str().unwrap())
            .await
            .unwrap();

        run_migrations(conn.clone()).await.unwrap();

        let api_key_repo = LibSqlApiKeyRepository::new(conn.clone());
        let user_repo = LibSqlUserRepository::new(conn.clone());
        let org_repo = LibSqlOrganizationRepository::new(conn.clone());

        (temp_dir, api_key_repo, user_repo, org_repo)
    }

    #[tokio::test]
    async fn test_api_key_create() {
        let (_temp_dir, api_key_repo, user_repo, org_repo) = setup_test_db().await;

        // Create organization
        let org = Organization::new("Test Org".to_string());
        org_repo.create(&org).await.unwrap();

        // Create user
        let user = User::new(org.id.clone(), "Test User".to_string(), "test@example.com".to_string(), "hashed_password".to_string(), "UTC".to_string());
        user_repo.create(&user).await.unwrap();

        // Create API key
        let api_key = ApiKey::new(
            org.id.clone(),
            user.id.clone(),
            "Test API Key".to_string(),
            "test_key_hash_123".to_string(),
            None,
        );

        let created = api_key_repo.create(&api_key).await.unwrap();

        assert_eq!(created.id, api_key.id);
        assert_eq!(created.name, "Test API Key");
        assert_eq!(created.key_hash, "test_key_hash_123");
        assert_eq!(created.user_id, user.id);
        assert_eq!(created.organization_id, org.id);
        assert!(!created.is_deleted);
    }

    #[tokio::test]
    async fn test_api_key_find_by_key() {
        let (_temp_dir, api_key_repo, user_repo, org_repo) = setup_test_db().await;

        // Create organization and user
        let org = Organization::new("Test Org".to_string());
        org_repo.create(&org).await.unwrap();

        let user = User::new(org.id.clone(), "Test User".to_string(), "test@example.com".to_string(), "hashed_password".to_string(), "UTC".to_string());
        user_repo.create(&user).await.unwrap();

        // Create API key
        let api_key = ApiKey::new(
            org.id.clone(),
            user.id.clone(),
            "Test API Key".to_string(),
            "unique_key_hash_456".to_string(),
            None,
        );
        api_key_repo.create(&api_key).await.unwrap();

        // Find by key hash
        let found = api_key_repo
            .find_by_key("unique_key_hash_456")
            .await
            .unwrap();

        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.id, api_key.id);
        assert_eq!(found.key_hash, "unique_key_hash_456");
        assert_eq!(found.name, "Test API Key");
    }

    #[tokio::test]
    async fn test_api_key_find_by_user_id() {
        let (_temp_dir, api_key_repo, user_repo, org_repo) = setup_test_db().await;

        // Create organization and user
        let org = Organization::new("Test Org".to_string());
        org_repo.create(&org).await.unwrap();

        let user = User::new(org.id.clone(), "Test User".to_string(), "test@example.com".to_string(), "hashed_password".to_string(), "UTC".to_string());
        user_repo.create(&user).await.unwrap();

        // Create multiple API keys for the same user
        for i in 1..=3 {
            let api_key = ApiKey::new(
                org.id.clone(),
                user.id.clone(),
                format!("API Key {}", i),
                format!("key_hash_{}", i),
                None,
            );
            api_key_repo.create(&api_key).await.unwrap();
        }

        // Find by user ID
        let api_keys = api_key_repo.find_by_user_id(&user.id).await.unwrap();

        assert_eq!(api_keys.len(), 3);
        assert!(api_keys.iter().all(|k| k.user_id == user.id));
    }

    #[tokio::test]
    async fn test_api_key_update() {
        let (_temp_dir, api_key_repo, user_repo, org_repo) = setup_test_db().await;

        // Create organization and user
        let org = Organization::new("Test Org".to_string());
        org_repo.create(&org).await.unwrap();

        let user = User::new(org.id.clone(), "Test User".to_string(), "test@example.com".to_string(), "hashed_password".to_string(), "UTC".to_string());
        user_repo.create(&user).await.unwrap();

        // Create API key
        let mut api_key = ApiKey::new(
            org.id.clone(),
            user.id.clone(),
            "Original Name".to_string(),
            "original_hash".to_string(),
            None,
        );
        let created = api_key_repo.create(&api_key).await.unwrap();

        // Update API key
        api_key.id = created.id.clone();
        api_key.name = "Updated Name".to_string();
        api_key.last_used_at = Some(Utc::now());

        let updated = api_key_repo.update(&api_key).await.unwrap();

        assert_eq!(updated.name, "Updated Name");
        assert!(updated.last_used_at.is_some());
    }

    #[tokio::test]
    async fn test_api_key_revoke() {
        let (_temp_dir, api_key_repo, user_repo, org_repo) = setup_test_db().await;

        // Create organization and user
        let org = Organization::new("Test Org".to_string());
        org_repo.create(&org).await.unwrap();

        let user = User::new(org.id.clone(), "Test User".to_string(), "test@example.com".to_string(), "hashed_password".to_string(), "UTC".to_string());
        user_repo.create(&user).await.unwrap();

        // Create API key
        let api_key = ApiKey::new(
            org.id.clone(),
            user.id.clone(),
            "Test API Key".to_string(),
            "revoke_test_hash".to_string(),
            None,
        );
        let created = api_key_repo.create(&api_key).await.unwrap();

        // Revoke API key
        api_key_repo.revoke(&created.id).await.unwrap();

        // Verify it's revoked (soft deleted)
        let found = api_key_repo.find_by_key("revoke_test_hash").await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_api_key_delete() {
        let (_temp_dir, api_key_repo, user_repo, org_repo) = setup_test_db().await;

        // Create organization and user
        let org = Organization::new("Test Org".to_string());
        org_repo.create(&org).await.unwrap();

        let user = User::new(org.id.clone(), "Test User".to_string(), "test@example.com".to_string(), "hashed_password".to_string(), "UTC".to_string());
        user_repo.create(&user).await.unwrap();

        // Create API key
        let api_key = ApiKey::new(
            org.id.clone(),
            user.id.clone(),
            "Test API Key".to_string(),
            "delete_test_hash".to_string(),
            None,
        );
        let created = api_key_repo.create(&api_key).await.unwrap();

        // Delete API key
        api_key_repo.delete(&created.id).await.unwrap();

        // Verify it's deleted
        let found = api_key_repo.find_by_key("delete_test_hash").await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_api_key_list() {
        let (_temp_dir, api_key_repo, user_repo, org_repo) = setup_test_db().await;

        // Create organization and user
        let org = Organization::new("Test Org".to_string());
        org_repo.create(&org).await.unwrap();

        let user = User::new(org.id.clone(), "Test User".to_string(), "test@example.com".to_string(), "hashed_password".to_string(), "UTC".to_string());
        user_repo.create(&user).await.unwrap();

        // Create multiple API keys
        for i in 1..=5 {
            let api_key = ApiKey::new(
                org.id.clone(),
                user.id.clone(),
                format!("API Key {}", i),
                format!("list_hash_{}", i),
                None,
            );
            api_key_repo.create(&api_key).await.unwrap();
        }

        // List with pagination
        let api_keys = api_key_repo.list(3, 0).await.unwrap();
        assert_eq!(api_keys.len(), 3);

        let api_keys_page2 = api_key_repo.list(3, 3).await.unwrap();
        assert_eq!(api_keys_page2.len(), 2);
    }

    #[tokio::test]
    async fn test_api_key_with_expiration() {
        let (_temp_dir, api_key_repo, user_repo, org_repo) = setup_test_db().await;

        // Create organization and user
        let org = Organization::new("Test Org".to_string());
        org_repo.create(&org).await.unwrap();

        let user = User::new(org.id.clone(), "Test User".to_string(), "test@example.com".to_string(), "hashed_password".to_string(), "UTC".to_string());
        user_repo.create(&user).await.unwrap();

        // Create API key with expiration
        let expires_at = Utc::now() + chrono::Duration::days(30);
        let api_key = ApiKey::new(
            org.id.clone(),
            user.id.clone(),
            "Expiring Key".to_string(),
            "expiring_hash".to_string(),
            Some(expires_at),
        );

        let created = api_key_repo.create(&api_key).await.unwrap();

        assert!(created.expires_at.is_some());
        assert_eq!(created.expires_at.unwrap().timestamp(), expires_at.timestamp());
    }
}

