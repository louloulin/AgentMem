//! Integration tests for authentication and authorization
//!
//! These tests verify the authentication and authorization functionality:
//! - JWT token generation and validation
//! - API Key authentication
//! - Password hashing and verification
//! - Role-based access control (RBAC)
//! - User management
//! - Organization management

use agent_mem_server::auth::{ApiKey, AuthService, PasswordService, Permission, Role};
use std::collections::HashSet;

#[test]
fn test_jwt_token_lifecycle() {
    let auth_service = AuthService::new("test-secret-key-that-is-long-enough-for-security");

    // Generate token
    let token = auth_service
        .generate_token(
            "user123",
            "org456".to_string(),
            vec!["user".to_string(), "editor".to_string()],
            Some("project789".to_string()),
        )
        .expect("Failed to generate token");

    assert!(!token.is_empty());

    // Validate token
    let claims = auth_service
        .validate_token(&token)
        .expect("Failed to validate token");

    assert_eq!(claims.sub, "user123");
    assert_eq!(claims.org_id, "org456");
    assert_eq!(claims.roles, vec!["user", "editor"]);
    assert_eq!(claims.project_id, Some("project789".to_string()));
}

#[test]
fn test_jwt_token_expiration() {
    let auth_service = AuthService::new("test-secret-key-that-is-long-enough-for-security");

    // Generate token
    let token = auth_service
        .generate_token(
            "user123",
            "org456".to_string(),
            vec!["user".to_string()],
            None,
        )
        .expect("Failed to generate token");

    // Token should be valid immediately
    assert!(auth_service.validate_token(&token).is_ok());

    // Note: Testing actual expiration would require waiting 24 hours or mocking time
}

#[test]
fn test_password_hashing_and_verification() {
    let password = "my_secure_password_123!";

    // Hash password
    let hash = PasswordService::hash_password(password).expect("Failed to hash password");

    assert!(!hash.is_empty());
    assert_ne!(hash, password); // Hash should be different from password

    // Verify correct password
    let is_valid =
        PasswordService::verify_password(password, &hash).expect("Failed to verify password");
    assert!(is_valid);

    // Verify incorrect password
    let is_valid = PasswordService::verify_password("wrong_password", &hash)
        .expect("Failed to verify password");
    assert!(!is_valid);
}

#[test]
fn test_password_hash_uniqueness() {
    let password = "same_password";

    // Hash the same password twice
    let hash1 = PasswordService::hash_password(password).expect("Failed to hash password");
    let hash2 = PasswordService::hash_password(password).expect("Failed to hash password");

    // Hashes should be different due to random salt
    assert_ne!(hash1, hash2);

    // But both should verify correctly
    assert!(PasswordService::verify_password(password, &hash1).unwrap());
    assert!(PasswordService::verify_password(password, &hash2).unwrap());
}

#[test]
fn test_api_key_generation() {
    let api_key = ApiKey::generate(
        "Production API Key".to_string(),
        "user123".to_string(),
        "org456".to_string(),
        HashSet::from(["read:memories".to_string(), "write:memories".to_string()]),
    );

    assert!(api_key.key.starts_with("agm_"));
    assert_eq!(api_key.name, "Production API Key");
    assert_eq!(api_key.user_id, "user123");
    assert_eq!(api_key.org_id, "org456");
    assert!(api_key.is_active);
    assert!(api_key.is_valid());
}

#[test]
fn test_api_key_scopes() {
    let api_key = ApiKey::generate(
        "Test Key".to_string(),
        "user123".to_string(),
        "org456".to_string(),
        HashSet::from(["read:memories".to_string(), "write:memories".to_string()]),
    );

    assert!(api_key.has_scope("read:memories"));
    assert!(api_key.has_scope("write:memories"));
    assert!(!api_key.has_scope("delete:memories"));
    assert!(!api_key.has_scope("admin"));
}

#[test]
fn test_api_key_wildcard_scope() {
    let api_key = ApiKey::generate(
        "Admin Key".to_string(),
        "admin123".to_string(),
        "org456".to_string(),
        HashSet::from(["*".to_string()]),
    );

    // Wildcard scope should grant all permissions
    assert!(api_key.has_scope("read:memories"));
    assert!(api_key.has_scope("write:memories"));
    assert!(api_key.has_scope("delete:memories"));
    assert!(api_key.has_scope("admin"));
}

#[test]
fn test_api_key_expiration() {
    let mut api_key = ApiKey::generate(
        "Expired Key".to_string(),
        "user123".to_string(),
        "org456".to_string(),
        HashSet::from(["read:memories".to_string()]),
    );

    // Set expiration to past
    api_key.expires_at = Some(chrono::Utc::now().timestamp() - 3600);

    assert!(!api_key.is_valid());
}

#[test]
fn test_api_key_inactive() {
    let mut api_key = ApiKey::generate(
        "Inactive Key".to_string(),
        "user123".to_string(),
        "org456".to_string(),
        HashSet::from(["read:memories".to_string()]),
    );

    api_key.is_active = false;

    assert!(!api_key.is_valid());
}

#[test]
fn test_role_permissions() {
    let admin_role = Role::admin();
    let user_role = Role::user();
    let viewer_role = Role::viewer();

    // Admin should have all permissions
    assert!(admin_role.has_permission(&Permission::All));
    assert!(admin_role.has_permission(&Permission::ReadMemory));
    assert!(admin_role.has_permission(&Permission::DeleteOrganization));

    // User should have basic permissions
    assert!(user_role.has_permission(&Permission::ReadMemory));
    assert!(user_role.has_permission(&Permission::WriteMemory));
    assert!(!user_role.has_permission(&Permission::DeleteOrganization));

    // Viewer should only have read permissions
    assert!(viewer_role.has_permission(&Permission::ReadMemory));
    assert!(!viewer_role.has_permission(&Permission::WriteMemory));
    assert!(!viewer_role.has_permission(&Permission::DeleteMemory));
}

#[test]
fn test_custom_role() {
    let custom_role = Role::new(
        "editor".to_string(),
        "Can read and write but not delete".to_string(),
        HashSet::from([
            Permission::ReadMemory,
            Permission::WriteMemory,
            Permission::ReadAgent,
            Permission::WriteAgent,
        ]),
    );

    assert_eq!(custom_role.name, "editor");
    assert!(custom_role.has_permission(&Permission::ReadMemory));
    assert!(custom_role.has_permission(&Permission::WriteMemory));
    assert!(!custom_role.has_permission(&Permission::DeleteMemory));
    assert!(!custom_role.has_permission(&Permission::ManageRoles));
}

#[test]
fn test_role_hierarchy() {
    // Create a role with specific permissions
    let role = Role::new(
        "data_manager".to_string(),
        "Manages data but not users".to_string(),
        HashSet::from([
            Permission::ReadMemory,
            Permission::WriteMemory,
            Permission::DeleteMemory,
            Permission::ReadAgent,
            Permission::WriteAgent,
            Permission::DeleteAgent,
        ]),
    );

    // Should have data permissions
    assert!(role.has_permission(&Permission::ReadMemory));
    assert!(role.has_permission(&Permission::WriteMemory));
    assert!(role.has_permission(&Permission::DeleteMemory));

    // Should not have user management permissions
    assert!(!role.has_permission(&Permission::WriteUser));
    assert!(!role.has_permission(&Permission::ManageRoles));
}

#[test]
fn test_extract_token_from_header() {
    let valid_header = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0";
    let token =
        AuthService::extract_token_from_header(valid_header).expect("Failed to extract token");

    assert_eq!(
        token,
        "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0"
    );

    // Test invalid header
    let invalid_header = "Invalid header";
    assert!(AuthService::extract_token_from_header(invalid_header).is_err());

    // Test missing Bearer prefix
    let no_bearer = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
    assert!(AuthService::extract_token_from_header(no_bearer).is_err());
}

#[test]
fn test_multiple_roles() {
    let auth_service = AuthService::new("test-secret-key-that-is-long-enough-for-security");

    let token = auth_service
        .generate_token(
            "user123",
            "org456".to_string(),
            vec![
                "user".to_string(),
                "editor".to_string(),
                "reviewer".to_string(),
            ],
            None,
        )
        .expect("Failed to generate token");

    let claims = auth_service
        .validate_token(&token)
        .expect("Failed to validate token");

    assert_eq!(claims.roles.len(), 3);
    assert!(claims.roles.contains(&"user".to_string()));
    assert!(claims.roles.contains(&"editor".to_string()));
    assert!(claims.roles.contains(&"reviewer".to_string()));
}
