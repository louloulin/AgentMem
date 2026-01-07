// ! Middleware modules for AgentMem server

pub mod api_version; // ✅ Task 1.5: API版本兼容性中间件
pub mod audit;
pub mod auth;
pub mod circuit_breaker; // ✅ Phase 2.2.5: 熔断器模式
pub mod metrics;
pub mod quota;
pub mod rbac;
pub mod validation; // ✅ P1 Task: Input validation layer

// Re-export commonly used middleware functions
pub use api_version::api_version_compatibility_middleware;
pub use audit::{audit_logging_middleware, log_security_event, SecurityEvent};
pub use auth::{
    api_key_auth_middleware, require_auth_middleware, extract_auth_user, has_role, is_admin,
    jwt_auth_middleware, optional_auth_middleware, require_admin, require_role,
    tenant_isolation_middleware, AuthUser,
};
pub use circuit_breaker::{circuit_breaker_middleware, CircuitBreakerManager};
pub use metrics::metrics_middleware;
pub use quota::{quota_middleware, QuotaLimits, QuotaManager, UsageStats};
pub use rbac::{
    admin_only, check_agent_permission, check_memory_permission, check_user_permission,
    no_read_only, rbac_middleware, RbacConfig,
};
pub use validation::{
    validate_add_memory_request, validate_update_memory_request,
    validate_search_request, validate_delete_request, validate_batch_add_request,
    validation_error_response,
};
