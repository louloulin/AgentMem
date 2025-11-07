// ! Middleware modules for AgentMem server

pub mod audit;
pub mod auth;
pub mod metrics;
pub mod quota;
pub mod rbac;

// Re-export commonly used middleware functions
pub use audit::{audit_logging_middleware, log_security_event, SecurityEvent};
pub use auth::{
    api_key_auth_middleware, default_auth_middleware, extract_auth_user, has_role, is_admin,
    jwt_auth_middleware, optional_auth_middleware, require_admin, require_role,
    tenant_isolation_middleware, AuthUser,
};
pub use metrics::metrics_middleware;
pub use quota::{quota_middleware, QuotaLimits, QuotaManager, UsageStats};
pub use rbac::{
    admin_only, check_agent_permission, check_memory_permission, check_user_permission,
    no_read_only, rbac_middleware, RbacConfig,
};
