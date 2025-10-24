//! Metrics collection middleware
//!
//! Automatically collects request metrics for Prometheus monitoring

use agent_mem_observability::metrics::MetricsRegistry;
use axum::{
    body::Body,
    extract::{Extension, Request},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use std::time::Instant;

/// Middleware to collect request metrics
pub async fn metrics_middleware(
    Extension(metrics_registry): Extension<Arc<MetricsRegistry>>,
    req: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    let method = req.method().to_string();
    let path = req.uri().path().to_string();

    // Call the next middleware/handler
    let response = next.run(req).await;

    // Record metrics
    let duration = start.elapsed().as_secs_f64();
    let status = response.status().as_u16();

    // Get the metrics collector
    let collector = metrics_registry.collector();

    // Record request
    collector.record_request(&method, &path, status).await;

    // Record request duration
    collector.record_request_duration(&method, &path, duration).await;

    // Record error if status >= 400
    if status >= 400 {
        let error_type = if status >= 500 {
            "server_error"
        } else {
            "client_error"
        };
        collector.record_error(error_type).await;
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        middleware,
        response::IntoResponse,
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    async fn test_handler() -> impl IntoResponse {
        (StatusCode::OK, "OK")
    }

    #[tokio::test]
    async fn test_metrics_middleware() {
        let metrics = Arc::new(MetricsRegistry::new());

        // Layer order matters: they execute from bottom to top
        // So Extension must be added AFTER middleware for middleware to extract it
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(metrics_middleware))
            .layer(Extension(metrics.clone()));

        let request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Verify metrics were recorded (would need to check actual metrics values)
    }
}

