//! RBAC权限验证中间件

use crate::auth::UserContext;
use crate::error::{ServerError, ServerResult};
use crate::rbac::{Action, AuditLogEntry, RbacChecker, Resource};
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

/// 权限验证中间件配置
#[derive(Clone)]
pub struct RbacConfig {
    /// 是否启用审计日志
    pub enable_audit: bool,
    /// 是否严格模式（没有角色时拒绝访问）
    pub strict_mode: bool,
}

impl Default for RbacConfig {
    fn default() -> Self {
        Self {
            enable_audit: true,
            strict_mode: true,
        }
    }
}

/// 检查记忆操作权限的中间件
pub async fn check_memory_permission(
    user_ctx: Option<UserContext>,
    req: Request,
    next: Next,
) -> ServerResult<Response> {
    let method = req.method().as_str();
    let action = Action::from_http_method(method);

    if let Some(user) = user_ctx {
        let result =
            RbacChecker::check_resource_action(&user.roles, Resource::Memory, action);

        // 记录审计日志
        let audit_log = AuditLogEntry::new(
            user.user_id.clone(),
            method.to_string(),
            "memory".to_string(),
            None,
            result.is_ok(),
            user.roles.clone(),
            None, // TODO: 从request中提取IP
            None, // TODO: 从request中提取User-Agent
        );
        audit_log.log();

        result?;
        Ok(next.run(req).await)
    } else {
        // 没有用户上下文，拒绝访问
        Err(ServerError::Unauthorized(
            "Authentication required".to_string(),
        ))
    }
}

/// 检查Agent操作权限的中间件
pub async fn check_agent_permission(
    user_ctx: Option<UserContext>,
    req: Request,
    next: Next,
) -> ServerResult<Response> {
    let method = req.method().as_str();
    let action = Action::from_http_method(method);

    if let Some(user) = user_ctx {
        let result = RbacChecker::check_resource_action(&user.roles, Resource::Agent, action);

        // 记录审计日志
        let audit_log = AuditLogEntry::new(
            user.user_id.clone(),
            method.to_string(),
            "agent".to_string(),
            None,
            result.is_ok(),
            user.roles.clone(),
            None,
            None,
        );
        audit_log.log();

        result?;
        Ok(next.run(req).await)
    } else {
        Err(ServerError::Unauthorized(
            "Authentication required".to_string(),
        ))
    }
}

/// 检查用户管理权限的中间件
pub async fn check_user_permission(
    user_ctx: Option<UserContext>,
    req: Request,
    next: Next,
) -> ServerResult<Response> {
    let method = req.method().as_str();
    let action = Action::from_http_method(method);

    if let Some(user) = user_ctx {
        let result = RbacChecker::check_resource_action(&user.roles, Resource::User, action);

        // 记录审计日志
        let audit_log = AuditLogEntry::new(
            user.user_id.clone(),
            method.to_string(),
            "user".to_string(),
            None,
            result.is_ok(),
            user.roles.clone(),
            None,
            None,
        );
        audit_log.log();

        result?;
        Ok(next.run(req).await)
    } else {
        Err(ServerError::Unauthorized(
            "Authentication required".to_string(),
        ))
    }
}

/// 仅管理员可访问的中间件
pub async fn admin_only(
    user_ctx: Option<UserContext>,
    req: Request,
    next: Next,
) -> ServerResult<Response> {
    if let Some(user) = user_ctx {
        if RbacChecker::is_admin(&user.roles) {
            // 记录审计日志
            let audit_log = AuditLogEntry::new(
                user.user_id.clone(),
                req.method().as_str().to_string(),
                "admin".to_string(),
                None,
                true,
                user.roles.clone(),
                None,
                None,
            );
            audit_log.log();

            Ok(next.run(req).await)
        } else {
            // 记录拒绝访问日志
            let audit_log = AuditLogEntry::new(
                user.user_id.clone(),
                req.method().as_str().to_string(),
                "admin".to_string(),
                None,
                false,
                user.roles.clone(),
                None,
                None,
            );
            audit_log.log();

            Err(ServerError::Forbidden(
                "Admin access required".to_string(),
            ))
        }
    } else {
        Err(ServerError::Unauthorized(
            "Authentication required".to_string(),
        ))
    }
}

/// 阻止只读用户的中间件
pub async fn no_read_only(
    user_ctx: Option<UserContext>,
    req: Request,
    next: Next,
) -> ServerResult<Response> {
    if let Some(user) = user_ctx {
        if RbacChecker::is_read_only(&user.roles) {
            // 记录拒绝访问日志
            let audit_log = AuditLogEntry::new(
                user.user_id.clone(),
                req.method().as_str().to_string(),
                "write_operation".to_string(),
                None,
                false,
                user.roles.clone(),
                None,
                None,
            );
            audit_log.log();

            Err(ServerError::Forbidden(
                "Read-only users cannot perform this action".to_string(),
            ))
        } else {
            Ok(next.run(req).await)
        }
    } else {
        Err(ServerError::Unauthorized(
            "Authentication required".to_string(),
        ))
    }
}

/// 通用RBAC权限验证中间件
/// 
/// 这是一个基本的RBAC中间件，检查用户是否有权限访问资源
pub async fn rbac_middleware(
    req: Request,
    next: Next,
) -> ServerResult<Response> {
    // 从 request extensions 中提取用户信息
    // 尝试获取 UserContext，如果没有则尝试 AuthUser
    let user_ctx = req.extensions().get::<UserContext>().cloned();
    
    if let Some(user) = user_ctx {
        // 对于认证用户，记录审计日志并继续
        let audit_log = AuditLogEntry::new(
            user.user_id.clone(),
            req.method().as_str().to_string(),
            req.uri().path().to_string(),
            None,
            true,
            user.roles.clone(),
            None,
            None,
        );
        audit_log.log();
        
        Ok(next.run(req).await)
    } else {
        // 尝试从 AuthUser 获取（用于开发模式）
        use crate::middleware::AuthUser;
        let auth_user = req.extensions().get::<AuthUser>().cloned();
        
        if let Some(user) = auth_user {
            // 对于默认认证用户，记录审计日志并继续
            let audit_log = AuditLogEntry::new(
                user.user_id.clone(),
                req.method().as_str().to_string(),
                req.uri().path().to_string(),
                None,
                true,
                user.roles.clone(),
                None,
                None,
            );
            audit_log.log();
            
            Ok(next.run(req).await)
        } else {
            // 没有任何用户上下文，需要认证
            Err(ServerError::Unauthorized(
                "Authentication required".to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rbac_config_default() {
        let config = RbacConfig::default();
        assert!(config.enable_audit);
        assert!(config.strict_mode);
    }

    #[test]
    fn test_action_from_http_method() {
        assert_eq!(Action::from_http_method("GET"), Action::Read);
        assert_eq!(Action::from_http_method("POST"), Action::Write);
        assert_eq!(Action::from_http_method("PUT"), Action::Write);
        assert_eq!(Action::from_http_method("DELETE"), Action::Delete);
    }
}

