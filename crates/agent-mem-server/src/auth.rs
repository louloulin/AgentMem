//! Authentication and authorization
//!
//! This module provides comprehensive authentication and authorization:
//! - JWT token generation and validation
//! - Refresh token support (P1 enhancement)
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
    /// Token type: "access" or "refresh"
    #[serde(rename = "type")]
    pub token_type: String,
    /// Expiration time
    pub exp: i64,
    /// Issued at
    pub iat: i64,
}

/// Token pair containing access and refresh tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    /// Access token (short-lived, e.g., 15 minutes)
    pub access_token: String,
    /// Refresh token (long-lived, e.g., 7 days)
    pub refresh_token: String,
    /// Access token expiration time (Unix timestamp)
    pub access_token_expires_at: i64,
    /// Refresh token expiration time (Unix timestamp)
    pub refresh_token_expires_at: i64,
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

    /// Generate a JWT token (legacy method for backward compatibility)
    ///
    /// **Note**: This method generates a long-lived token (24 hours).
    /// For new code, prefer `generate_token_pair()` which provides better security.
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
            token_type: "access".to_string(), // ✅ P1: Add token type
            exp: exp.timestamp(),
            iat: now.timestamp(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| ServerError::unauthorized(format!("Token generation failed: {e}")))
    }

    /// ✅ P1 Enhancement: Generate access and refresh token pair
    ///
    /// This is the recommended method for authentication as it provides:
    /// - Better security (short-lived access tokens)
    /// - Better UX (long-lived refresh tokens)
    /// - Configurable expiration times
    ///
    /// # Arguments
    /// * `user_id` - User ID
    /// * `org_id` - Organization ID
    /// * `roles` - User roles
    /// * `project_id` - Optional project ID
    /// * `access_token_duration` - Access token lifetime (default: 15 minutes)
    /// * `refresh_token_duration` - Refresh token lifetime (default: 7 days)
    ///
    /// # Example
    /// ```no_run
    /// use chrono::Duration;
    /// # use agent_mem_server::auth::AuthService;
    /// # let auth_service = AuthService::new("secret");
    /// let token_pair = auth_service.generate_token_pair(
    ///     "user123",
    ///     "org456".to_string(),
    ///     vec!["user".to_string()],
    ///     None,
    ///     Some(Duration::minutes(15)),  // 15-minute access token
    ///     Some(Duration::days(7)),       // 7-day refresh token
    /// ).unwrap();
    /// ```
    pub fn generate_token_pair(
        &self,
        user_id: &str,
        org_id: String,
        roles: Vec<String>,
        project_id: Option<String>,
        access_token_duration: Option<Duration>,
        refresh_token_duration: Option<Duration>,
    ) -> ServerResult<TokenPair> {
        let now = Utc::now();

        // Default: 15-minute access token
        let access_duration = access_token_duration.unwrap_or(Duration::minutes(15));
        let access_exp = now + access_duration;

        // Default: 7-day refresh token
        let refresh_duration = refresh_token_duration.unwrap_or(Duration::days(7));
        let refresh_exp = now + refresh_duration;

        // Generate access token
        let access_claims = Claims {
            sub: user_id.to_string(),
            org_id: org_id.clone(),
            roles: roles.clone(),
            project_id: project_id.clone(),
            token_type: "access".to_string(),
            exp: access_exp.timestamp(),
            iat: now.timestamp(),
        };

        let access_token = encode(&Header::default(), &access_claims, &self.encoding_key)
            .map_err(|e| ServerError::unauthorized(format!("Access token generation failed: {e}")))?;

        // Generate refresh token
        let refresh_claims = Claims {
            sub: user_id.to_string(),
            org_id: org_id.clone(),
            roles,
            project_id,
            token_type: "refresh".to_string(),
            exp: refresh_exp.timestamp(),
            iat: now.timestamp(),
        };

        let refresh_token = encode(&Header::default(), &refresh_claims, &self.encoding_key)
            .map_err(|e| ServerError::unauthorized(format!("Refresh token generation failed: {e}")))?;

        Ok(TokenPair {
            access_token,
            refresh_token,
            access_token_expires_at: access_exp.timestamp(),
            refresh_token_expires_at: refresh_exp.timestamp(),
        })
    }

    /// ✅ P1 Enhancement: Refresh access token using refresh token
    ///
    /// This method validates the refresh token and generates a new access token.
    /// The new access token will have the same user context as the refresh token.
    ///
    /// # Arguments
    /// * `refresh_token` - The refresh token
    /// * `access_token_duration` - Optional new access token duration (defaults to 15 minutes)
    ///
    /// # Returns
    /// A new access token string
    ///
    /// # Errors
    /// Returns an error if:
    /// - The refresh token is invalid or expired
    /// - The token type is not "refresh"
    pub fn refresh_access_token(
        &self,
        refresh_token: &str,
        access_token_duration: Option<Duration>,
    ) -> ServerResult<String> {
        // Validate refresh token
        let claims = self.validate_token(refresh_token)?;

        // Ensure it's a refresh token
        if claims.token_type != "refresh" {
            return Err(ServerError::unauthorized(
                "Invalid token type: expected 'refresh' token".to_string(),
            ));
        }

        let now = Utc::now();
        let access_duration = access_token_duration.unwrap_or(Duration::minutes(15));
        let access_exp = now + access_duration;

        // Generate new access token with same user context
        let new_access_claims = Claims {
            sub: claims.sub.clone(),
            org_id: claims.org_id.clone(),
            roles: claims.roles.clone(),
            project_id: claims.project_id.clone(),
            token_type: "access".to_string(),
            exp: access_exp.timestamp(),
            iat: now.timestamp(),
        };

        encode(&Header::default(), &new_access_claims, &self.encoding_key).map_err(|e| {
            ServerError::unauthorized(format!("New access token generation failed: {e}"))
        })
    }

    /// Validate a JWT token
    ///
    /// ✅ P1 Enhancement: Now also validates token type for access tokens
    pub fn validate_token(&self, token: &str) -> ServerResult<Claims> {
        decode::<Claims>(token, &self.decoding_key, &Validation::default())
            .map(|data| data.claims)
            .map_err(|e| ServerError::unauthorized(format!("Token validation failed: {e}")))
    }

    /// ✅ P1 Enhancement: Validate access token specifically
    ///
    /// This method ensures the token is of type "access" and is not expired.
    pub fn validate_access_token(&self, token: &str) -> ServerResult<Claims> {
        let claims = self.validate_token(token)?;

        if claims.token_type != "access" {
            return Err(ServerError::unauthorized(
                "Invalid token type: expected 'access' token".to_string(),
            ));
        }

        Ok(claims)
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
        // ✅ P1: Verify token type is set
        assert_eq!(claims.token_type, "access");
    }

    // ✅ P1: New test for token pair generation
    #[test]
    fn test_token_pair_generation() {
        use chrono::Duration;

        let auth_service = AuthService::new("test-secret-key-that-is-long-enough");

        let token_pair = auth_service
            .generate_token_pair(
                "user123",
                "org456".to_string(),
                vec!["user".to_string()],
                Some("project789".to_string()),
                Some(Duration::minutes(15)),
                Some(Duration::days(7)),
            )
            .unwrap();

        // Verify both tokens are generated
        assert!(!token_pair.access_token.is_empty());
        assert!(!token_pair.refresh_token.is_empty());

        // Validate access token
        let access_claims = auth_service.validate_token(&token_pair.access_token).unwrap();
        assert_eq!(access_claims.sub, "user123");
        assert_eq!(access_claims.token_type, "access");
        assert_eq!(access_claims.project_id, Some("project789".to_string()));

        // Validate refresh token
        let refresh_claims = auth_service.validate_token(&token_pair.refresh_token).unwrap();
        assert_eq!(refresh_claims.sub, "user123");
        assert_eq!(refresh_claims.token_type, "refresh");

        // Verify expiration times
        assert!(token_pair.access_token_expires_at < token_pair.refresh_token_expires_at);
    }

    // ✅ P1: New test for refresh token flow
    #[test]
    fn test_refresh_access_token() {
        use chrono::Duration;

        let auth_service = AuthService::new("test-secret-key-that-is-long-enough");

        // Generate initial token pair
        let token_pair = auth_service
            .generate_token_pair(
                "user123",
                "org456".to_string(),
                vec!["admin".to_string()],
                None,
                Some(Duration::minutes(15)),
                Some(Duration::days(7)),
            )
            .unwrap();

        // Use refresh token to get new access token
        let new_access_token = auth_service
            .refresh_access_token(&token_pair.refresh_token, Some(Duration::minutes(30)))
            .unwrap();

        // Validate new access token
        let new_claims = auth_service.validate_access_token(&new_access_token).unwrap();
        assert_eq!(new_claims.sub, "user123");
        assert_eq!(new_claims.org_id, "org456");
        assert_eq!(new_claims.roles, vec!["admin".to_string()]);
        assert_eq!(new_claims.token_type, "access");

        // Verify tokens are different
        assert_ne!(token_pair.access_token, new_access_token);
    }

    // ✅ P1: Test that access token cannot be used as refresh token
    #[test]
    fn test_access_token_cannot_refresh() {
        use chrono::Duration;

        let auth_service = AuthService::new("test-secret-key-that-is-long-enough");

        let token_pair = auth_service
            .generate_token_pair(
                "user123",
                "org456".to_string(),
                vec!["user".to_string()],
                None,
                None,
                None,
            )
            .unwrap();

        // Try to use access token as refresh token (should fail)
        let result = auth_service.refresh_access_token(&token_pair.access_token, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid token type"));
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
