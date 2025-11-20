//! API版本兼容性中间件
//! Task 1.5: 记录使用旧版本路由的请求，便于监控迁移进度

use axum::{
    body::Body,
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use tracing::warn;

/// API版本兼容性中间件
///
/// 功能：
/// 1. 检测使用旧版本路由（不含/v1）的请求
/// 2. 记录日志，便于监控迁移进度
/// 3. （可选）在响应头中添加弃用提示
pub async fn api_version_compatibility_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = req.uri().path().to_string(); // ✅ 复制字符串避免借用问题

    // 检查是否使用旧版本路由
    let is_legacy_route = path.starts_with("/api/agents") && !path.starts_with("/api/v1/");

    if is_legacy_route {
        warn!(
            "⚠️  Client using deprecated API path: {} (recommended: /api/v1/...)",
            path
        );
    }

    let mut response = next.run(req).await;

    // ✅ Task 1.5: 在响应头中添加弃用提示（可选，Phase 2启用）
    if is_legacy_route {
        let headers = response.headers_mut();
        headers.insert("X-API-Deprecated", "true".parse().unwrap());
        headers.insert(
            "X-API-Recommended",
            format!("/api/v1{}", &path[4..]).parse().unwrap(),
        );
    }

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        middleware,
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    async fn test_handler() -> &'static str {
        "OK"
    }

    #[tokio::test]
    async fn test_legacy_route_detection() {
        let app = Router::new()
            .route("/api/agents/test/chat", get(test_handler))
            .layer(middleware::from_fn(api_version_compatibility_middleware));

        let request = Request::builder()
            .uri("/api/agents/test/chat")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // 验证响应成功
        assert_eq!(response.status(), StatusCode::OK);

        // 验证弃用提示头存在
        assert_eq!(response.headers().get("X-API-Deprecated").unwrap(), "true");
    }

    #[tokio::test]
    async fn test_v1_route_no_warning() {
        let app = Router::new()
            .route("/api/v1/agents/test/chat", get(test_handler))
            .layer(middleware::from_fn(api_version_compatibility_middleware));

        let request = Request::builder()
            .uri("/api/v1/agents/test/chat")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // 验证响应成功
        assert_eq!(response.status(), StatusCode::OK);

        // 验证没有弃用提示头
        assert!(response.headers().get("X-API-Deprecated").is_none());
    }
}
