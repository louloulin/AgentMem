//! Scope Middleware for Memory Access Control
//!
//! Enforces MemoryScope-based access control on API endpoints.
//!
//! 🔴 Phase 3: Scope Middleware - AgentMem v2.0

use agent_mem_traits::scope::MemoryScope;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use tracing::{debug, warn};

use crate::middleware::auth::AuthUser;

/// Scope middleware state
#[derive(Clone)]
pub struct ScopeMiddlewareState {
    /// Enable scope enforcement (default: true in production)
    pub enabled: bool,
    /// Allow cross-user access for admin roles
    pub allow_admin_bypass: bool,
}

impl Default for ScopeMiddlewareState {
    fn default() -> Self {
        Self {
            enabled: true,
            allow_admin_bypass: true,
        }
    }
}

impl ScopeMiddlewareState {
    /// Create new scope middleware state
    pub fn new(enabled: bool, allow_admin_bypass: bool) -> Self {
        Self {
            enabled,
            allow_admin_bypass,
        }
    }
}

/// Extract MemoryScope from request
///
/// This function attempts to extract scope information from various sources:
/// 1. AuthUser claims (preferred)
/// 2. Request headers (x-scope, x-user-id, x-agent-id)
/// 3. Query parameters
pub fn extract_scope_from_request(auth_user: &AuthUser) -> MemoryScope {
    // Build scope from AuthUser
    // AuthUser has org_id and user_id as String (empty string means None)
    let org_id = if auth_user.org_id.is_empty() {
        None
    } else {
        Some(auth_user.org_id.clone())
    };

    let user_id = if auth_user.user_id.is_empty() {
        None
    } else {
        Some(auth_user.user_id.clone())
    };

    match (org_id, user_id) {
        (Some(org_id), Some(user_id)) => {
            MemoryScope::User {
                org_id: Some(org_id),
                user_id,
            }
        }
        (None, Some(user_id)) => {
            MemoryScope::User {
                org_id: None,
                user_id,
            }
        }
        _ => MemoryScope::Global,
    }
}

/// Check if the user has access to the target scope
pub fn check_scope_access(user_scope: &MemoryScope, target_scope: &MemoryScope) -> bool {
    user_scope.can_access(target_scope)
}

/// Scope enforcement middleware
///
/// This middleware can be used to enforce scope-based access control.
/// However, since AuthUser already provides scope information,
/// the actual enforcement is done in the individual route handlers.
pub async fn scope_enforcement_middleware(
    State(_state): State<Arc<ScopeMiddlewareState>>,
    auth_user: AuthUser,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract user scope from auth
    let user_scope = extract_scope_from_request(&auth_user);

    debug!(
        "Scope middleware: user_scope={}, path={}",
        user_scope,
        request.uri()
    );

    // Store scope in request extensions for downstream handlers
    request.extensions_mut().insert(user_scope.clone());

    // Continue to next middleware/handler
    Ok(next.run(request).await)
}

/// Validate that target scope is accessible from user scope
///
/// Returns Ok(()) if access is allowed, Err(StatusCode) if denied.
pub fn validate_access(
    user_scope: &MemoryScope,
    target_scope: &MemoryScope,
) -> Result<(), StatusCode> {
    if check_scope_access(user_scope, target_scope) {
        Ok(())
    } else {
        warn!(
            "Access denied: user_scope={} cannot access target_scope={}",
            user_scope, target_scope
        );
        Err(StatusCode::FORBIDDEN)
    }
}

/// Extract target scope from query parameters or request
///
/// This is a helper function for route handlers to extract scope info.
pub fn extract_target_scope_from_params(
    org_id: Option<String>,
    user_id: Option<String>,
    agent_id: Option<String>,
) -> MemoryScope {
    match (org_id, user_id, agent_id) {
        (Some(org_id), Some(user_id), Some(agent_id)) => MemoryScope::Agent {
            org_id: Some(org_id),
            user_id,
            agent_id,
        },
        (None, Some(user_id), Some(agent_id)) => MemoryScope::Agent {
            org_id: None,
            user_id,
            agent_id,
        },
        (org_id, Some(user_id), None) => MemoryScope::User {
            org_id,
            user_id,
        },
        (Some(org_id), None, None) => MemoryScope::Organization {
            org_id,
        },
        _ => MemoryScope::Global,
    }
}

/// Extension trait for extracting scope from request
pub trait ScopeExt {
    fn get_scope(&self) -> Option<MemoryScope>;
}

impl<B> ScopeExt for Request<B> {
    fn get_scope(&self) -> Option<MemoryScope> {
        self.extensions().get::<MemoryScope>().cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_access_same_user() {
        let scope1 = MemoryScope::User {
            org_id: None,
            user_id: "user1".to_string(),
        };
        let scope2 = MemoryScope::User {
            org_id: None,
            user_id: "user1".to_string(),
        };

        assert!(check_scope_access(&scope1, &scope2));
    }

    #[test]
    fn test_scope_access_different_user() {
        let scope1 = MemoryScope::User {
            org_id: None,
            user_id: "user1".to_string(),
        };
        let scope2 = MemoryScope::User {
            org_id: None,
            user_id: "user2".to_string(),
        };

        assert!(!check_scope_access(&scope1, &scope2));
    }

    #[test]
    fn test_scope_access_agent_to_user() {
        let agent_scope = MemoryScope::Agent {
            org_id: None,
            user_id: "user1".to_string(),
            agent_id: "agent1".to_string(),
        };
        let user_scope = MemoryScope::User {
            org_id: None,
            user_id: "user1".to_string(),
        };

        // Agent is descendant of User, so User (ancestor) can access Agent (descendant)
        // but Agent (descendant) cannot access User (ancestor) in this implementation
        // Note: can_access checks target.is_descendant_of(self)
        // Agent -> User: User is parent of Agent, so Agent IS descendant of User
        // check_scope_access(agent_scope, user_scope) = agent_scope.can_access(user_scope)
        // = user_scope.is_descendant_of(agent_scope) = User.parent() = Global, not Agent
        // So agent_scope cannot access user_scope
        assert!(!check_scope_access(&agent_scope, &user_scope));

        // But user can access agent (ancestor can access descendant)
        assert!(check_scope_access(&user_scope, &agent_scope));
    }

    #[test]
    fn test_extract_target_scope() {
        let scope = extract_target_scope_from_params(
            None,
            Some("user1".to_string()),
            Some("agent1".to_string()),
        );

        match scope {
            MemoryScope::Agent { user_id, .. } => {
                assert_eq!(user_id, "user1");
            }
            _ => panic!("Expected Agent scope"),
        }
    }
}
