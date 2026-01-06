//! Health check endpoints
//!
//! This module provides liveness and readiness probes for Kubernetes.

use crate::error::{ObservabilityError, ObservabilityResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// Service is healthy
    Healthy,
    /// Service is degraded but operational
    Degraded,
    /// Service is unhealthy
    Unhealthy,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Degraded => write!(f, "degraded"),
            HealthStatus::Unhealthy => write!(f, "unhealthy"),
        }
    }
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    /// Overall status
    pub status: HealthStatus,
    /// Service version
    pub version: String,
    /// Uptime in seconds
    pub uptime_seconds: u64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Component statuses
    pub components: HashMap<String, ComponentHealth>,
}

/// Component health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component status
    pub status: HealthStatus,
    /// Optional message
    pub message: Option<String>,
    /// Last check time
    pub last_check: DateTime<Utc>,
}

/// Health check manager
pub struct HealthCheck {
    start_time: Instant,
    components: Arc<RwLock<HashMap<String, ComponentHealth>>>,
}

impl HealthCheck {
    /// Create a new health check manager
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            components: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a component
    pub async fn register_component(&self, name: impl Into<String>) {
        let mut components = self.components.write().await;
        components.insert(
            name.into(),
            ComponentHealth {
                status: HealthStatus::Healthy,
                message: None,
                last_check: Utc::now(),
            },
        );
    }

    /// Update component status
    pub async fn update_component(
        &self,
        name: &str,
        status: HealthStatus,
        message: Option<String>,
    ) {
        let mut components = self.components.write().await;
        if let Some(component) = components.get_mut(name) {
            component.status = status;
            component.message = message;
            component.last_check = Utc::now();
        }
    }

    /// Check liveness (is the service running?)
    pub async fn liveness(&self) -> HealthCheckResponse {
        HealthCheckResponse {
            status: HealthStatus::Healthy,
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
            timestamp: Utc::now(),
            components: HashMap::new(),
        }
    }

    /// Check readiness (is the service ready to accept traffic?)
    pub async fn readiness(&self) -> HealthCheckResponse {
        let components = self.components.read().await;

        // Determine overall status
        let overall_status = if components
            .values()
            .all(|c| c.status == HealthStatus::Healthy)
        {
            HealthStatus::Healthy
        } else if components
            .values()
            .any(|c| c.status == HealthStatus::Unhealthy)
        {
            HealthStatus::Unhealthy
        } else {
            HealthStatus::Degraded
        };

        HealthCheckResponse {
            status: overall_status,
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
            timestamp: Utc::now(),
            components: components.clone(),
        }
    }

    /// Check a specific dependency
    pub async fn check_dependency(
        &self,
        name: &str,
        check_fn: impl std::future::Future<Output = bool>,
    ) -> bool {
        let is_healthy = check_fn.await;
        let status = if is_healthy {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy
        };

        self.update_component(name, status, None).await;
        is_healthy
    }
}

impl Default for HealthCheck {
    fn default() -> Self {
        Self::new()
    }
}

/// Start health check server
pub async fn start_health_server(
    health_check: Arc<HealthCheck>,
    port: u16,
) -> ObservabilityResult<()> {
    use axum::{extract::State, routing::get, Json, Router};
    use std::net::SocketAddr;

    let app =
        Router::new()
            .route(
                "/health/live",
                get(|State(health): State<Arc<HealthCheck>>| async move {
                    Json(health.liveness().await)
                }),
            )
            .route(
                "/health/ready",
                get(|State(health): State<Arc<HealthCheck>>| async move {
                    Json(health.readiness().await)
                }),
            )
            .with_state(health_check);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Health check server listening on {}", addr);

    axum::serve(
        tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| ObservabilityError::HealthCheckFailed(e.to_string()))?,
        app,
    )
    .await
    .map_err(|e| ObservabilityError::HealthCheckFailed(e.to_string()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check_liveness() {
        let health = HealthCheck::new();
        let response = health.liveness().await;

        assert_eq!(response.status, HealthStatus::Healthy);
        assert_eq!(response.version, env!("CARGO_PKG_VERSION"));
    }

    #[tokio::test]
    async fn test_health_check_readiness() {
        let health = HealthCheck::new();

        // Register components
        health.register_component("database").await;
        health.register_component("cache").await;

        let response = health.readiness().await;
        assert_eq!(response.status, HealthStatus::Healthy);
        assert_eq!(response.components.len(), 2);
    }

    #[tokio::test]
    async fn test_health_check_degraded() {
        let health = HealthCheck::new();

        health.register_component("database").await;
        health.register_component("cache").await;

        // Mark cache as degraded
        health
            .update_component(
                "cache",
                HealthStatus::Degraded,
                Some("Slow response".to_string()),
            )
            .await;

        let response = health.readiness().await;
        assert_eq!(response.status, HealthStatus::Degraded);
    }

    #[tokio::test]
    async fn test_health_check_unhealthy() {
        let health = HealthCheck::new();

        health.register_component("database").await;

        // Mark database as unhealthy
        health
            .update_component(
                "database",
                HealthStatus::Unhealthy,
                Some("Connection failed".to_string()),
            )
            .await;

        let response = health.readiness().await;
        assert_eq!(response.status, HealthStatus::Unhealthy);
    }

    #[tokio::test]
    async fn test_check_dependency() {
        let health = HealthCheck::new();
        health.register_component("test_service").await;

        // Check healthy dependency
        let is_healthy = health
            .check_dependency("test_service", async { true })
            .await;
        assert!(is_healthy);

        // Check unhealthy dependency
        let is_healthy = health
            .check_dependency("test_service", async { false })
            .await;
        assert!(!is_healthy);
    }
}
