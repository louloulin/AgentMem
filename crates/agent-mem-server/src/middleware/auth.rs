//! Authentication middleware for Axum
//!
//! This module provides middleware for JWT and API Key authentication.

use crate::auth::AuthService;
use crate::error::{ServerError, ServerResult};
use agent_mem_core::storage::traits::ApiKeyRepositoryTrait;
use axum::{extract::Request, http::header, middleware::Next, response::Response};
use sha2::{Digest, Sha256};
use std::sync::Arc;

/// Authenticated user information
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub org_id: String,
    pub roles: Vec<String>,
}

/// JWT authentication middleware
pub async fn jwt_auth_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, ServerError> {
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| ServerError::Unauthorized("Missing authorization header".to_string()))?;

    // Extract token from "Bearer <token>" format
    let token = AuthService::extract_token_from_header(auth_header)?;

    // Get AuthService from extensions (added by router)
    let auth_service = request
        .extensions()
        .get::<Arc<AuthService>>()
        .ok_or_else(|| ServerError::Internal("AuthService not found".to_string()))?;

    // Validate token
    let claims = auth_service.validate_token(token)?;

    // Add user info to request extensions
    let auth_user = AuthUser {
        user_id: claims.sub,
        org_id: claims.org_id,
        roles: claims.roles,
    };

    request.extensions_mut().insert(auth_user);

    Ok(next.run(request).await)
}

/// API Key authentication middleware
pub async fn api_key_auth_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, ServerError> {
    // Extract API key from header
    let api_key = request
        .headers()
        .get("X-API-Key")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| ServerError::Unauthorized("Missing API key".to_string()))?;

    // Check format
    if !api_key.starts_with("agm_") {
        return Err(ServerError::Unauthorized(
            "Invalid API key format".to_string(),
        ));
    }

    // Get ApiKeyRepository from extensions (added by router)
    let api_key_repo = request
        .extensions()
        .get::<Arc<dyn ApiKeyRepositoryTrait>>()
        .ok_or_else(|| ServerError::Internal("ApiKeyRepository not found".to_string()))?;

    // Hash the API key for lookup
    let key_hash = hash_api_key(api_key);

    // Validate API key against database
    let api_key_model = api_key_repo
        .find_by_key(&key_hash)
        .await
        .map_err(|e| ServerError::Internal(format!("API key validation failed: {e}")))?
        .ok_or_else(|| ServerError::Unauthorized("Invalid or expired API key".to_string()))?;

    // Extract user info from API key
    let auth_user = AuthUser {
        user_id: api_key_model.user_id.clone(),
        org_id: api_key_model.organization_id.clone(),
        roles: vec!["user".to_string()], // API keys have basic user role
    };

    request.extensions_mut().insert(auth_user);

    Ok(next.run(request).await)
}

/// Hash an API key using SHA-256
fn hash_api_key(api_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(api_key.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Optional authentication middleware (allows unauthenticated requests)
pub async fn optional_auth_middleware(mut request: Request, next: Next) -> Response {
    // Try to extract and validate token
    if let Some(auth_header) = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
    {
        if let Ok(token) = AuthService::extract_token_from_header(auth_header) {
            if let Some(auth_service) = request.extensions().get::<Arc<AuthService>>() {
                if let Ok(claims) = auth_service.validate_token(token) {
                    let auth_user = AuthUser {
                        user_id: claims.sub,
                        org_id: claims.org_id,
                        roles: claims.roles,
                    };

                    request.extensions_mut().insert(auth_user);
                }
            }
        }
    }

    next.run(request).await
}

/// Extract authenticated user from request
pub fn extract_auth_user(request: &Request) -> ServerResult<AuthUser> {
    request
        .extensions()
        .get::<AuthUser>()
        .cloned()
        .ok_or_else(|| ServerError::Unauthorized("Not authenticated".to_string()))
}

/// Check if user has a specific role
pub fn has_role(auth_user: &AuthUser, role: &str) -> bool {
    auth_user.roles.contains(&role.to_string())
}

/// Check if user is admin
pub fn is_admin(auth_user: &AuthUser) -> bool {
    has_role(auth_user, "admin")
}

/// Require admin role
pub fn require_admin(auth_user: &AuthUser) -> ServerResult<()> {
    if !is_admin(auth_user) {
        return Err(ServerError::Forbidden("Admin role required".to_string()));
    }
    Ok(())
}

/// Require specific role
pub fn require_role(auth_user: &AuthUser, role: &str) -> ServerResult<()> {
    if !has_role(auth_user, role) {
        return Err(ServerError::Forbidden(format!("Role '{role}' required")));
    }
    Ok(())
}

/// Tenant isolation middleware
pub async fn tenant_isolation_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, ServerError> {
    // Extract authenticated user
    let auth_user = extract_auth_user(&request)?;

    // Add organization ID to request for tenant isolation
    request.extensions_mut().insert(auth_user.org_id.clone());

    Ok(next.run(request).await)
}

/// Default authentication middleware (when auth is disabled)
///
/// This middleware injects a default AuthUser for development/testing
/// when authentication is disabled. In production, use jwt_auth_middleware
/// or api_key_auth_middleware instead.
pub async fn default_auth_middleware(mut request: Request, next: Next) -> Response {
    // Check if AuthUser already exists (from optional_auth_middleware)
    if request.extensions().get::<AuthUser>().is_none() {
        // Inject a default AuthUser for development
        let default_user = AuthUser {
            user_id: "default".to_string(),
            org_id: "default-org".to_string(),
            roles: vec!["admin".to_string(), "user".to_string()],
        };
        request.extensions_mut().insert(default_user);
    }

    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_role() {
        let auth_user = AuthUser {
            user_id: "user123".to_string(),
            org_id: "org456".to_string(),
            roles: vec!["admin".to_string(), "user".to_string()],
        };

        assert!(has_role(&auth_user, "admin"));
        assert!(has_role(&auth_user, "user"));
        assert!(!has_role(&auth_user, "viewer"));
    }

    #[test]
    fn test_is_admin() {
        let admin_user = AuthUser {
            user_id: "user123".to_string(),
            org_id: "org456".to_string(),
            roles: vec!["admin".to_string()],
        };

        let regular_user = AuthUser {
            user_id: "user789".to_string(),
            org_id: "org456".to_string(),
            roles: vec!["user".to_string()],
        };

        assert!(is_admin(&admin_user));
        assert!(!is_admin(&regular_user));
    }

    #[test]
    fn test_require_admin() {
        let admin_user = AuthUser {
            user_id: "user123".to_string(),
            org_id: "org456".to_string(),
            roles: vec!["admin".to_string()],
        };

        let regular_user = AuthUser {
            user_id: "user789".to_string(),
            org_id: "org456".to_string(),
            roles: vec!["user".to_string()],
        };

        assert!(require_admin(&admin_user).is_ok());
        assert!(require_admin(&regular_user).is_err());
    }

    #[test]
    fn test_require_role() {
        let auth_user = AuthUser {
            user_id: "user123".to_string(),
            org_id: "org456".to_string(),
            roles: vec!["editor".to_string()],
        };

        assert!(require_role(&auth_user, "editor").is_ok());
        assert!(require_role(&auth_user, "admin").is_err());
    }
}
