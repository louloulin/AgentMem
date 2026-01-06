//! Authentication and authorization
//!
//! This module provides comprehensive authentication and authorization:
//! - JWT token generation and validation
//! - API Key management
//! - Password hashing with Argon2
//! - Role-based access control (RBAC)

use crate::error::{ServerError, ServerResult};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

/// JWT claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Organization ID
    pub org_id: String,
    /// Project ID
    pub project_id: Option<String>,
    /// User roles
    pub roles: Vec<String>,
    /// Expiration time
    pub exp: i64,
    /// Issued at
    pub iat: i64,
}

/// Authentication service
pub struct AuthService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl AuthService {
    /// Create a new authentication service
    pub fn new(secret: &str) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_ref()),
            decoding_key: DecodingKey::from_secret(secret.as_ref()),
        }
    }

    /// Generate a JWT token
    pub fn generate_token(
        &self,
        user_id: &str,
        org_id: String,
        roles: Vec<String>,
        project_id: Option<String>,
    ) -> ServerResult<String> {
        let now = Utc::now();
        let exp = now + Duration::hours(24); // Token expires in 24 hours

        let claims = Claims {
            sub: user_id.to_string(),
            org_id,
            roles,
            project_id,
            exp: exp.timestamp(),
            iat: now.timestamp(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| ServerError::unauthorized(format!("Token generation failed: {e}")))
    }

    /// Validate a JWT token
    pub fn validate_token(&self, token: &str) -> ServerResult<Claims> {
        decode::<Claims>(token, &self.decoding_key, &Validation::default())
            .map(|data| data.claims)
            .map_err(|e| ServerError::unauthorized(format!("Token validation failed: {e}")))
    }

    /// Extract token from Authorization header
    pub fn extract_token_from_header(auth_header: &str) -> ServerResult<&str> {
        if auth_header.starts_with("Bearer ") {
            Ok(&auth_header[7..])
        } else {
            Err(ServerError::unauthorized(
                "Invalid authorization header format".to_string(),
            ))
        }
    }
}

/// User context extracted from JWT
#[derive(Debug, Clone)]
pub struct UserContext {
    pub user_id: String,
    pub org_id: String,
    pub roles: Vec<String>,
    pub project_id: Option<String>,
}

impl From<Claims> for UserContext {
    fn from(claims: Claims) -> Self {
        Self {
            user_id: claims.sub,
            org_id: claims.org_id,
            roles: claims.roles,
            project_id: claims.project_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_generation_and_validation() {
        let auth_service = AuthService::new("test-secret-key-that-is-long-enough");

        let token = auth_service
            .generate_token(
                "user123",
                "org456".to_string(),
                vec!["user".to_string()],
                None,
            )
            .unwrap();

        let claims = auth_service.validate_token(&token).unwrap();
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.org_id, "org456");
        assert_eq!(claims.roles, vec!["user".to_string()]);
    }

    #[test]
    fn test_extract_token_from_header() {
        let header = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
        let token = AuthService::extract_token_from_header(header).unwrap();
        assert_eq!(token, "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9");
    }

    #[test]
    fn test_invalid_header_format() {
        let header = "Invalid header";
        let result = AuthService::extract_token_from_header(header);
        assert!(result.is_err());
    }
}

/// Password hashing service using Argon2
pub struct PasswordService;

impl PasswordService {
    /// Hash a password using Argon2
    pub fn hash_password(password: &str) -> ServerResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| ServerError::internal_error(format!("Password hashing failed: {e}")))
    }

    /// Verify a password against a hash
    pub fn verify_password(password: &str, hash: &str) -> ServerResult<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| ServerError::internal_error(format!("Invalid password hash: {e}")))?;

        let argon2 = Argon2::default();

        Ok(argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}

/// API Key for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: String,
    pub key: String,
    pub name: String,
    pub user_id: String,
    pub org_id: String,
    pub scopes: HashSet<String>,
    pub created_at: i64,
    pub expires_at: Option<i64>,
    pub last_used_at: Option<i64>,
    pub is_active: bool,
}

impl ApiKey {
    /// Generate a new API key
    pub fn generate(
        name: String,
        user_id: String,
        org_id: String,
        scopes: HashSet<String>,
    ) -> Self {
        let id = Uuid::new_v4().to_string();
        let key = format!("agm_{}", Uuid::new_v4().to_string().replace('-', ""));

        Self {
            id,
            key,
            name,
            user_id,
            org_id,
            scopes,
            created_at: Utc::now().timestamp(),
            expires_at: None,
            last_used_at: None,
            is_active: true,
        }
    }

    /// Check if the API key is valid
    pub fn is_valid(&self) -> bool {
        if !self.is_active {
            return false;
        }

        if let Some(expires_at) = self.expires_at {
            if Utc::now().timestamp() > expires_at {
                return false;
            }
        }

        true
    }

    /// Check if the API key has a specific scope
    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.contains(scope) || self.scopes.contains("*")
    }
}

/// Permission for RBAC
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    // Memory operations
    ReadMemory,
    WriteMemory,
    DeleteMemory,

    // Agent operations
    ReadAgent,
    WriteAgent,
    DeleteAgent,

    // User operations
    ReadUser,
    WriteUser,
    DeleteUser,

    // Organization operations
    ReadOrganization,
    WriteOrganization,
    DeleteOrganization,

    // Admin operations
    ManageRoles,
    ManagePermissions,
    ViewAuditLogs,
    ManageApiKeys,

    // Wildcard
    All,
}

/// Role for RBAC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub description: String,
    pub permissions: HashSet<Permission>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Role {
    /// Create a new role
    pub fn new(name: String, description: String, permissions: HashSet<Permission>) -> Self {
        let now = Utc::now().timestamp();

        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            permissions,
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if the role has a specific permission
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission) || self.permissions.contains(&Permission::All)
    }

    /// Create a default admin role
    pub fn admin() -> Self {
        Self::new(
            "admin".to_string(),
            "Administrator with full access".to_string(),
            HashSet::from([Permission::All]),
        )
    }

    /// Create a default user role
    pub fn user() -> Self {
        Self::new(
            "user".to_string(),
            "Regular user with basic access".to_string(),
            HashSet::from([
                Permission::ReadMemory,
                Permission::WriteMemory,
                Permission::ReadAgent,
                Permission::ReadUser,
            ]),
        )
    }

    /// Create a default viewer role
    pub fn viewer() -> Self {
        Self::new(
            "viewer".to_string(),
            "Read-only access".to_string(),
            HashSet::from([
                Permission::ReadMemory,
                Permission::ReadAgent,
                Permission::ReadUser,
                Permission::ReadOrganization,
            ]),
        )
    }
}

#[cfg(test)]
mod auth_tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "secure_password_123";
        let hash = PasswordService::hash_password(password).unwrap();

        assert!(PasswordService::verify_password(password, &hash).unwrap());
        assert!(!PasswordService::verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_api_key_generation() {
        let api_key = ApiKey::generate(
            "Test Key".to_string(),
            "user123".to_string(),
            "org456".to_string(),
            HashSet::from(["read".to_string(), "write".to_string()]),
        );

        assert!(api_key.key.starts_with("agm_"));
        assert!(api_key.is_valid());
        assert!(api_key.has_scope("read"));
        assert!(api_key.has_scope("write"));
        assert!(!api_key.has_scope("admin"));
    }

    #[test]
    fn test_role_permissions() {
        let admin_role = Role::admin();
        assert!(admin_role.has_permission(&Permission::All));
        assert!(admin_role.has_permission(&Permission::ReadMemory));

        let user_role = Role::user();
        assert!(user_role.has_permission(&Permission::ReadMemory));
        assert!(!user_role.has_permission(&Permission::DeleteOrganization));

        let viewer_role = Role::viewer();
        assert!(viewer_role.has_permission(&Permission::ReadMemory));
        assert!(!viewer_role.has_permission(&Permission::WriteMemory));
    }
}
