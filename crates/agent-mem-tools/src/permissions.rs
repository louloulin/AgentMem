//! Permission management for tool execution
//!
//! This module provides fine-grained access control for tool execution.

use crate::error::{ToolError, ToolResult};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// Permission level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Permission {
    /// Read-only access
    Read,
    /// Write access
    Write,
    /// Execute access
    Execute,
    /// Admin access (all permissions)
    Admin,
}

/// User role
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Role {
    /// Role name
    pub name: String,
    /// Permissions granted to this role
    pub permissions: HashSet<Permission>,
}

impl Role {
    /// Create a new role
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            permissions: HashSet::new(),
        }
    }

    /// Add a permission to the role
    pub fn with_permission(mut self, permission: Permission) -> Self {
        self.permissions.insert(permission);
        self
    }

    /// Check if role has a permission
    pub fn has_permission(&self, permission: Permission) -> bool {
        self.permissions.contains(&permission) || self.permissions.contains(&Permission::Admin)
    }
}

/// Tool permission configuration
#[derive(Debug, Clone)]
pub struct ToolPermission {
    /// Tool name
    pub tool_name: String,
    /// Required permissions
    pub required_permissions: HashSet<Permission>,
    /// Allowed roles
    pub allowed_roles: HashSet<String>,
    /// Allowed users
    pub allowed_users: HashSet<String>,
}

impl ToolPermission {
    /// Create a new tool permission
    pub fn new(tool_name: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            required_permissions: HashSet::new(),
            allowed_roles: HashSet::new(),
            allowed_users: HashSet::new(),
        }
    }

    /// Require a permission
    pub fn require_permission(mut self, permission: Permission) -> Self {
        self.required_permissions.insert(permission);
        self
    }

    /// Allow a role
    pub fn allow_role(mut self, role: impl Into<String>) -> Self {
        self.allowed_roles.insert(role.into());
        self
    }

    /// Allow a user
    pub fn allow_user(mut self, user: impl Into<String>) -> Self {
        self.allowed_users.insert(user.into());
        self
    }
}

/// Permission manager
pub struct PermissionManager {
    /// User roles mapping
    user_roles: Arc<RwLock<HashMap<String, HashSet<String>>>>,
    /// Role definitions
    roles: Arc<RwLock<HashMap<String, Role>>>,
    /// Tool permissions
    tool_permissions: Arc<RwLock<HashMap<String, ToolPermission>>>,
}

impl PermissionManager {
    /// Create a new permission manager
    pub fn new() -> Self {
        let mut roles = HashMap::new();

        // Create default roles
        roles.insert(
            "admin".to_string(),
            Role::new("admin").with_permission(Permission::Admin),
        );
        roles.insert(
            "user".to_string(),
            Role::new("user")
                .with_permission(Permission::Read)
                .with_permission(Permission::Execute),
        );
        roles.insert(
            "guest".to_string(),
            Role::new("guest").with_permission(Permission::Read),
        );

        Self {
            user_roles: Arc::new(RwLock::new(HashMap::new())),
            roles: Arc::new(RwLock::new(roles)),
            tool_permissions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Assign a role to a user
    pub async fn assign_role(&self, user: impl Into<String>, role: impl Into<String>) {
        let user = user.into();
        let role = role.into();

        let mut user_roles = self.user_roles.write().await;
        user_roles
            .entry(user.clone())
            .or_insert_with(HashSet::new)
            .insert(role.clone());

        debug!("Assigned role '{}' to user '{}'", role, user);
    }

    /// Register a custom role
    pub async fn register_role(&self, role: Role) {
        let mut roles = self.roles.write().await;
        roles.insert(role.name.clone(), role);
    }

    /// Set tool permissions
    pub async fn set_tool_permission(&self, permission: ToolPermission) {
        let mut tool_permissions = self.tool_permissions.write().await;
        tool_permissions.insert(permission.tool_name.clone(), permission);
    }

    /// Check if a user has permission to execute a tool
    pub async fn check_permission(&self, tool_name: &str, user: &str) -> ToolResult<()> {
        debug!(
            "Checking permission for user '{}' on tool '{}'",
            user, tool_name
        );

        // Get tool permissions
        let tool_permissions = self.tool_permissions.read().await;
        let tool_perm = match tool_permissions.get(tool_name) {
            Some(perm) => perm,
            None => {
                // No specific permissions set, allow by default
                debug!("No specific permissions for tool '{}', allowing", tool_name);
                return Ok(());
            }
        };

        // Check if user is explicitly allowed
        if tool_perm.allowed_users.contains(user) {
            debug!(
                "User '{}' explicitly allowed for tool '{}'",
                user, tool_name
            );
            return Ok(());
        }

        // Get user roles
        let user_roles = self.user_roles.read().await;
        let user_role_names = match user_roles.get(user) {
            Some(roles) => roles,
            None => {
                warn!("User '{user}' has no roles assigned");
                return Err(ToolError::PermissionDenied(format!(
                    "User '{user}' has no roles assigned"
                )));
            }
        };

        // Check if any user role is allowed
        for role_name in user_role_names {
            if tool_perm.allowed_roles.contains(role_name) {
                debug!("User '{}' has allowed role '{}'", user, role_name);
                return Ok(());
            }
        }

        // Check if user has required permissions through roles
        let roles = self.roles.read().await;
        for role_name in user_role_names {
            if let Some(role) = roles.get(role_name) {
                let has_all_permissions = tool_perm
                    .required_permissions
                    .iter()
                    .all(|perm| role.has_permission(*perm));

                if has_all_permissions {
                    debug!(
                        "User '{}' has required permissions through role '{}'",
                        user, role_name
                    );
                    return Ok(());
                }
            }
        }

        warn!("User '{user}' denied access to tool '{tool_name}'");
        Err(ToolError::PermissionDenied(format!(
            "User '{user}' does not have permission to execute tool '{tool_name}'"
        )))
    }

    /// Get user roles
    pub async fn get_user_roles(&self, user: &str) -> Vec<String> {
        let user_roles = self.user_roles.read().await;
        user_roles
            .get(user)
            .map(|roles| roles.iter().cloned().collect())
            .unwrap_or_default()
    }
}

impl Default for PermissionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_role_creation() {
        let role = Role::new("test_role")
            .with_permission(Permission::Read)
            .with_permission(Permission::Execute);

        assert_eq!(role.name, "test_role");
        assert!(role.has_permission(Permission::Read));
        assert!(role.has_permission(Permission::Execute));
        assert!(!role.has_permission(Permission::Write));
    }

    #[tokio::test]
    async fn test_admin_role() {
        let role = Role::new("admin").with_permission(Permission::Admin);

        assert!(role.has_permission(Permission::Read));
        assert!(role.has_permission(Permission::Write));
        assert!(role.has_permission(Permission::Execute));
    }

    #[tokio::test]
    async fn test_permission_check_allowed() {
        let manager = PermissionManager::new();

        // Assign admin role to user
        manager.assign_role("user1", "admin").await;

        // Check permission
        let result = manager.check_permission("test_tool", "user1").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_permission_check_denied() {
        let manager = PermissionManager::new();

        // Assign guest role to user
        manager.assign_role("user1", "guest").await;

        // Set tool permission requiring execute
        let tool_perm = ToolPermission::new("test_tool").require_permission(Permission::Execute);
        manager.set_tool_permission(tool_perm).await;

        // Check permission (should fail)
        let result = manager.check_permission("test_tool", "user1").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_explicit_user_permission() {
        let manager = PermissionManager::new();

        // Set tool permission allowing specific user
        let tool_perm = ToolPermission::new("test_tool").allow_user("user1");
        manager.set_tool_permission(tool_perm).await;

        // Check permission (should succeed even without roles)
        let result = manager.check_permission("test_tool", "user1").await;
        assert!(result.is_ok());
    }
}
