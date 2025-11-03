//! Role-Based Access Control (RBAC) System
//!
//! 提供完整的RBAC功能：
//! - 角色定义 (Admin/User/ReadOnly)
//! - 权限验证
//! - 资源级别访问控制
//! - 权限审计

use crate::error::{ServerError, ServerResult};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// 系统角色定义
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// 管理员 - 完全访问权限
    Admin,
    /// 普通用户 - 读写权限
    User,
    /// 只读用户 - 仅读取权限
    ReadOnly,
}

impl Role {
    /// 从字符串解析角色
    pub fn from_str(s: &str) -> ServerResult<Self> {
        match s.to_lowercase().as_str() {
            "admin" => Ok(Role::Admin),
            "user" => Ok(Role::User),
            "readonly" | "read_only" | "read-only" => Ok(Role::ReadOnly),
            _ => Err(ServerError::BadRequest(format!("Invalid role: {}", s))),
        }
    }

    /// 获取角色字符串表示
    pub fn as_str(&self) -> &str {
        match self {
            Role::Admin => "admin",
            Role::User => "user",
            Role::ReadOnly => "readonly",
        }
    }

    /// 检查角色是否有指定权限
    pub fn has_permission(&self, permission: &Permission) -> bool {
        match self {
            Role::Admin => true, // 管理员有所有权限
            Role::User => matches!(
                permission,
                Permission::ReadMemory
                    | Permission::WriteMemory
                    | Permission::ReadAgent
                    | Permission::WriteAgent
                    | Permission::ReadUser
            ),
            Role::ReadOnly => matches!(
                permission,
                Permission::ReadMemory | Permission::ReadAgent | Permission::ReadUser
            ),
        }
    }
}

/// 权限定义
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    // 记忆相关
    ReadMemory,
    WriteMemory,
    DeleteMemory,
    
    // Agent相关
    ReadAgent,
    WriteAgent,
    DeleteAgent,
    
    // 用户相关
    ReadUser,
    WriteUser,
    DeleteUser,
    
    // 系统管理
    ManageSystem,
    ViewMetrics,
    ManageRoles,
}

/// 资源类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resource {
    Memory,
    Agent,
    User,
    System,
}

/// 操作类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Read,
    Write,
    Delete,
    Manage,
}

impl Action {
    /// 从HTTP方法转换
    pub fn from_http_method(method: &str) -> Self {
        match method.to_uppercase().as_str() {
            "GET" => Action::Read,
            "POST" | "PUT" | "PATCH" => Action::Write,
            "DELETE" => Action::Delete,
            _ => Action::Read, // 默认为读取
        }
    }
}

/// RBAC检查器
pub struct RbacChecker;

impl RbacChecker {
    /// 检查用户是否有指定权限
    pub fn check_permission(roles: &[String], permission: Permission) -> ServerResult<()> {
        let parsed_roles: Vec<Role> = roles
            .iter()
            .filter_map(|r| Role::from_str(r).ok())
            .collect();

        if parsed_roles.is_empty() {
            return Err(ServerError::Forbidden(
                "No valid roles assigned".to_string(),
            ));
        }

        // 检查是否有任一角色拥有该权限
        if parsed_roles.iter().any(|role| role.has_permission(&permission)) {
            Ok(())
        } else {
            Err(ServerError::Forbidden(format!(
                "Permission denied: {:?}",
                permission
            )))
        }
    }

    /// 检查用户是否能对资源执行操作
    pub fn check_resource_action(
        roles: &[String],
        resource: Resource,
        action: Action,
    ) -> ServerResult<()> {
        let permission = match (resource, action) {
            (Resource::Memory, Action::Read) => Permission::ReadMemory,
            (Resource::Memory, Action::Write) => Permission::WriteMemory,
            (Resource::Memory, Action::Delete) => Permission::DeleteMemory,
            (Resource::Agent, Action::Read) => Permission::ReadAgent,
            (Resource::Agent, Action::Write) => Permission::WriteAgent,
            (Resource::Agent, Action::Delete) => Permission::DeleteAgent,
            (Resource::User, Action::Read) => Permission::ReadUser,
            (Resource::User, Action::Write) => Permission::WriteUser,
            (Resource::User, Action::Delete) => Permission::DeleteUser,
            (Resource::System, Action::Manage) => Permission::ManageSystem,
            (Resource::System, Action::Read) => Permission::ViewMetrics,
            _ => {
                return Err(ServerError::BadRequest(format!(
                    "Invalid resource-action combination: {:?} {:?}",
                    resource, action
                )))
            }
        };

        Self::check_permission(roles, permission)
    }

    /// 检查用户是否有管理员权限
    pub fn is_admin(roles: &[String]) -> bool {
        roles
            .iter()
            .any(|r| Role::from_str(r).ok() == Some(Role::Admin))
    }

    /// 检查用户是否只读
    pub fn is_read_only(roles: &[String]) -> bool {
        let parsed_roles: Vec<Role> = roles
            .iter()
            .filter_map(|r| Role::from_str(r).ok())
            .collect();

        !parsed_roles.is_empty()
            && parsed_roles.iter().all(|r| matches!(r, Role::ReadOnly))
    }
}

/// 权限审计日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub user_id: String,
    pub action: String,
    pub resource: String,
    pub resource_id: Option<String>,
    pub allowed: bool,
    pub roles: Vec<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl AuditLogEntry {
    /// 创建新的审计日志条目
    pub fn new(
        user_id: String,
        action: String,
        resource: String,
        resource_id: Option<String>,
        allowed: bool,
        roles: Vec<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            user_id,
            action,
            resource,
            resource_id,
            allowed,
            roles,
            ip_address,
            user_agent,
        }
    }

    /// 记录到日志
    pub fn log(&self) {
        if self.allowed {
            tracing::info!(
                user_id = %self.user_id,
                action = %self.action,
                resource = %self.resource,
                resource_id = ?self.resource_id,
                roles = ?self.roles,
                ip = ?self.ip_address,
                "Permission granted"
            );
        } else {
            tracing::warn!(
                user_id = %self.user_id,
                action = %self.action,
                resource = %self.resource,
                resource_id = ?self.resource_id,
                roles = ?self.roles,
                ip = ?self.ip_address,
                "Permission denied"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_parsing() {
        assert_eq!(Role::from_str("admin").unwrap(), Role::Admin);
        assert_eq!(Role::from_str("Admin").unwrap(), Role::Admin);
        assert_eq!(Role::from_str("user").unwrap(), Role::User);
        assert_eq!(Role::from_str("readonly").unwrap(), Role::ReadOnly);
        assert_eq!(Role::from_str("read-only").unwrap(), Role::ReadOnly);
        assert!(Role::from_str("invalid").is_err());
    }

    #[test]
    fn test_admin_permissions() {
        let admin = Role::Admin;
        assert!(admin.has_permission(&Permission::ReadMemory));
        assert!(admin.has_permission(&Permission::WriteMemory));
        assert!(admin.has_permission(&Permission::DeleteMemory));
        assert!(admin.has_permission(&Permission::ManageSystem));
    }

    #[test]
    fn test_user_permissions() {
        let user = Role::User;
        assert!(user.has_permission(&Permission::ReadMemory));
        assert!(user.has_permission(&Permission::WriteMemory));
        assert!(!user.has_permission(&Permission::DeleteMemory));
        assert!(!user.has_permission(&Permission::ManageSystem));
    }

    #[test]
    fn test_readonly_permissions() {
        let readonly = Role::ReadOnly;
        assert!(readonly.has_permission(&Permission::ReadMemory));
        assert!(!readonly.has_permission(&Permission::WriteMemory));
        assert!(!readonly.has_permission(&Permission::DeleteMemory));
    }

    #[test]
    fn test_rbac_checker() {
        let admin_roles = vec!["admin".to_string()];
        let user_roles = vec!["user".to_string()];
        let readonly_roles = vec!["readonly".to_string()];

        // Admin可以做任何事
        assert!(RbacChecker::check_permission(&admin_roles, Permission::DeleteMemory).is_ok());

        // User可以读写
        assert!(RbacChecker::check_permission(&user_roles, Permission::ReadMemory).is_ok());
        assert!(RbacChecker::check_permission(&user_roles, Permission::WriteMemory).is_ok());
        assert!(RbacChecker::check_permission(&user_roles, Permission::DeleteMemory).is_err());

        // ReadOnly只能读
        assert!(RbacChecker::check_permission(&readonly_roles, Permission::ReadMemory).is_ok());
        assert!(RbacChecker::check_permission(&readonly_roles, Permission::WriteMemory).is_err());
    }

    #[test]
    fn test_resource_action_check() {
        let user_roles = vec!["user".to_string()];

        assert!(
            RbacChecker::check_resource_action(&user_roles, Resource::Memory, Action::Read)
                .is_ok()
        );
        assert!(
            RbacChecker::check_resource_action(&user_roles, Resource::Memory, Action::Write)
                .is_ok()
        );
        assert!(
            RbacChecker::check_resource_action(&user_roles, Resource::Memory, Action::Delete)
                .is_err()
        );
    }

    #[test]
    fn test_is_admin() {
        assert!(RbacChecker::is_admin(&vec!["admin".to_string()]));
        assert!(!RbacChecker::is_admin(&vec!["user".to_string()]));
    }

    #[test]
    fn test_is_read_only() {
        assert!(RbacChecker::is_read_only(&vec!["readonly".to_string()]));
        assert!(!RbacChecker::is_read_only(&vec!["user".to_string()]));
        assert!(!RbacChecker::is_read_only(&vec!["admin".to_string()]));
    }

    #[test]
    fn test_audit_log() {
        let log = AuditLogEntry::new(
            "user123".to_string(),
            "DELETE".to_string(),
            "memory".to_string(),
            Some("mem456".to_string()),
            false,
            vec!["user".to_string()],
            Some("192.168.1.1".to_string()),
            Some("Mozilla/5.0".to_string()),
        );

        assert_eq!(log.user_id, "user123");
        assert_eq!(log.action, "DELETE");
        assert!(!log.allowed);
    }
}

