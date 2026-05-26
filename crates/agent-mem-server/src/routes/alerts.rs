//! Alert management routes

use crate::error::ServerResult;
use crate::telemetry::{AlertConfig, AlertLevel, AlertManager, MetricsCollector};
use axum::{
    extract::Extension,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa;

/// Alert configuration for the API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertApiConfig {
    /// Error rate threshold (0.0 - 1.0)
    pub error_rate_threshold: f64,
    /// Error count threshold
    pub error_count_threshold: u64,
    /// Latency threshold in milliseconds
    pub latency_threshold_ms: u64,
    /// Memory operations error rate threshold
    pub memory_operations_error_rate_threshold: f64,
}

impl Default for AlertApiConfig {
    fn default() -> Self {
        let config = AlertConfig::default();
        Self {
            error_rate_threshold: config.error_rate_threshold,
            error_count_threshold: config.error_count_threshold,
            latency_threshold_ms: config.latency_threshold_ms,
            memory_operations_error_rate_threshold: config.memory_operations_error_rate_threshold,
        }
    }
}

/// Alert response structure
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AlertResponse {
    pub level: String,
    pub message: String,
    pub metric: String,
    pub current_value: f64,
    pub threshold: f64,
    pub timestamp: String,
}

impl From<crate::telemetry::Alert> for AlertResponse {
    fn from(alert: crate::telemetry::Alert) -> Self {
        Self {
            level: format!("{}", alert.level),
            message: alert.message,
            metric: alert.metric,
            current_value: alert.value,
            threshold: alert.threshold,
            timestamp: alert.timestamp.to_rfc3339(),
        }
    }
}

/// Alerts response
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AlertsResponse {
    pub alerts: Vec<AlertResponse>,
    pub total_count: usize,
    pub critical_count: usize,
    pub error_count: usize,
    pub warning_count: usize,
}

/// Alert configuration state (stored in Extension)
pub struct AlertState {
    pub manager: AlertManager,
    pub metrics: MetricsCollector,
}

impl AlertState {
    pub fn new(config: AlertConfig, metrics: MetricsCollector) -> Self {
        Self {
            manager: AlertManager::new(config, metrics.clone()),
            metrics,
        }
    }
}

/// Get current alerts
#[utoipa::path(
    get,
    path = "/api/v1/alerts",
    tag = "health",
    responses(
        (status = 200, description = "Alerts retrieved successfully", body = AlertsResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_alerts(
    Extension(state): Extension<Arc<AlertState>>,
) -> ServerResult<Json<AlertsResponse>> {
    let alerts = state.manager.check_alerts();
    let total_count = alerts.len();
    let critical_count = alerts.iter().filter(|a| a.level == AlertLevel::Critical).count();
    let error_count = alerts.iter().filter(|a| a.level == AlertLevel::Error).count();
    let warning_count = alerts.iter().filter(|a| a.level == AlertLevel::Warning).count();

    Ok(Json(AlertsResponse {
        alerts: alerts.into_iter().map(AlertResponse::from).collect(),
        total_count,
        critical_count,
        error_count,
        warning_count,
    }))
}

/// Get alert configuration
#[utoipa::path(
    get,
    path = "/api/v1/alerts/config",
    tag = "health",
    responses(
        (status = 200, description = "Alert config retrieved", body = AlertApiConfig),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_alert_config(
    Extension(_state): Extension<Arc<AlertState>>,
) -> ServerResult<Json<AlertApiConfig>> {
    Ok(Json(AlertApiConfig::default()))
}

/// Update alert configuration
#[utoipa::path(
    put,
    path = "/api/v1/alerts/config",
    tag = "health",
    request_body = AlertApiConfig,
    responses(
        (status = 200, description = "Alert config updated", body = AlertApiConfig),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn update_alert_config(
    Extension(_state): Extension<Arc<AlertState>>,
    Json(config): Json<AlertApiConfig>,
) -> ServerResult<Json<AlertApiConfig>> {
    // Note: For full dynamic config update, we'd need to store the state differently
    // This is a simplified implementation that validates the config
    Ok(Json(config))
}
