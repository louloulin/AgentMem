//! Middleware for the server
//!
//! This module provides various middleware for the server:
//! - Authentication (JWT and API Key)
//! - Request logging
//! - Audit logging
//! - Quota management
//! - Tenant isolation

pub mod audit;
pub mod auth;
pub mod metrics;
pub mod quota;

use axum::{extract::Request, middleware::Next, response::Response};

// Re-export auth middleware
pub use auth::{
    api_key_auth_middleware, extract_auth_user, has_role, is_admin, jwt_auth_middleware,
    optional_auth_middleware, require_admin, require_role, tenant_isolation_middleware, AuthUser,
};

// Re-export audit middleware
pub use audit::{audit_logging_middleware, log_security_event, SecurityEvent};

// Re-export metrics middleware
pub use metrics::metrics_middleware;

// Re-export quota middleware
pub use quota::{quota_middleware, QuotaLimits, QuotaManager, UsageStats};

/// Request logging middleware
pub async fn request_logging_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();

    tracing::info!("Processing {} {}", method, uri);

    let start = std::time::Instant::now();
    let response = next.run(request).await;
    let duration = start.elapsed();

    tracing::info!(
        "Completed {} {} - Status: {} - Duration: {:?}",
        method,
        uri,
        response.status(),
        duration
    );

    response
}

/// Authentication middleware (placeholder)
pub async fn auth_middleware(request: Request, next: Next) -> Response {
    // TODO: Implement JWT authentication
    // For now, just pass through
    next.run(request).await
}

/// Rate limiting middleware (placeholder)
pub async fn rate_limit_middleware(request: Request, next: Next) -> Response {
    // TODO: Implement rate limiting
    // For now, just pass through
    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_middleware_module_exists() {
        // Simple test to verify the module compiles
        assert!(true);
    }
}
