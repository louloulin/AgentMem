//! Production Monitoring and Telemetry System
//!
//! Comprehensive monitoring, metrics collection, and telemetry for production
//! environments with support for alerts, dashboards, and observability.

use agent_mem_traits::{AgentMemError, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Monitoring system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable metrics collection
    pub enable_metrics: bool,
    /// Enable distributed tracing
    pub enable_tracing: bool,
    /// Enable health checks
    pub enable_health_checks: bool,
    /// Metrics retention period in hours
    pub metrics_retention_hours: u64,
    /// Alert evaluation interval in seconds
    pub alert_evaluation_interval_seconds: u64,
    /// Enable performance profiling
    pub enable_profiling: bool,
    /// Maximum metrics buffer size
    pub max_metrics_buffer_size: usize,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enable_metrics: true,
            enable_tracing: true,
            enable_health_checks: true,
            metrics_retention_hours: 24,
            alert_evaluation_interval_seconds: 60,
            enable_profiling: false,
            max_metrics_buffer_size: 10000,
        }
    }
}

/// System metrics data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    pub name: String,
    pub value: f64,
    pub timestamp: DateTime<Utc>,
    pub labels: HashMap<String, String>,
    pub metric_type: MetricType,
}

/// Types of metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    /// Counter - monotonically increasing value
    Counter,
    /// Gauge - current value that can go up or down
    Gauge,
    /// Histogram - distribution of values
    Histogram,
    /// Summary - quantiles over sliding time window
    Summary,
}

/// Health check status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub component: String,
    pub status: ComponentStatus,
    pub message: String,
    pub last_check: DateTime<Utc>,
    pub response_time_ms: u64,
    pub details: HashMap<String, String>,
}

/// Component health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ComponentStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: String,
    pub name: String,
    pub metric_name: String,
    pub condition: AlertCondition,
    pub threshold: f64,
    pub duration_seconds: u64,
    pub severity: AlertSeverity,
    pub enabled: bool,
    pub labels: HashMap<String, String>,
}

/// Alert conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    GreaterThan,
    LessThan,
    Equal,
    NotEqual,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

/// Active alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub rule_id: String,
    pub name: String,
    pub severity: AlertSeverity,
    pub message: String,
    pub started_at: DateTime<Utc>,
    pub last_triggered: DateTime<Utc>,
    pub labels: HashMap<String, String>,
    pub resolved: bool,
    pub resolved_at: Option<DateTime<Utc>>,
}

/// Performance profile data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceProfile {
    pub operation: String,
    pub duration_ms: u64,
    pub memory_usage_bytes: u64,
    pub cpu_usage_percent: f64,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

/// Comprehensive monitoring system
pub struct MonitoringSystem {
    config: MonitoringConfig,
    metrics: Arc<RwLock<VecDeque<MetricPoint>>>,
    health_checks: Arc<RwLock<HashMap<String, HealthStatus>>>,
    alert_rules: Arc<RwLock<HashMap<String, AlertRule>>>,
    active_alerts: Arc<RwLock<HashMap<String, Alert>>>,
    performance_profiles: Arc<RwLock<VecDeque<PerformanceProfile>>>,
    system_info: Arc<RwLock<SystemInfo>>,
}

/// System information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub version: String,
    pub build_time: String,
    pub uptime_seconds: u64,
    pub memory_usage_mb: u64,
    pub cpu_usage_percent: f64,
    pub active_connections: u64,
    pub total_requests: u64,
    pub error_rate: f64,
}

impl MonitoringSystem {
    /// Create a new monitoring system
    pub fn new(config: MonitoringConfig) -> Self {
        Self {
            config,
            metrics: Arc::new(RwLock::new(VecDeque::new())),
            health_checks: Arc::new(RwLock::new(HashMap::new())),
            alert_rules: Arc::new(RwLock::new(HashMap::new())),
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
            performance_profiles: Arc::new(RwLock::new(VecDeque::new())),
            system_info: Arc::new(RwLock::new(SystemInfo {
                version: env!("CARGO_PKG_VERSION").to_string(),
                build_time: "unknown".to_string(),
                uptime_seconds: 0,
                memory_usage_mb: 0,
                cpu_usage_percent: 0.0,
                active_connections: 0,
                total_requests: 0,
                error_rate: 0.0,
            })),
        }
    }

    /// Record a metric point
    pub async fn record_metric(
        &self,
        name: String,
        value: f64,
        metric_type: MetricType,
        labels: HashMap<String, String>,
    ) -> Result<()> {
        if !self.config.enable_metrics {
            return Ok(());
        }

        let metric = MetricPoint {
            name,
            value,
            timestamp: Utc::now(),
            labels,
            metric_type,
        };

        let mut metrics = self.metrics.write().await;
        metrics.push_back(metric);

        // Limit buffer size
        while metrics.len() > self.config.max_metrics_buffer_size {
            metrics.pop_front();
        }

        Ok(())
    }

    /// Record a counter metric
    pub async fn increment_counter(
        &self,
        name: &str,
        labels: HashMap<String, String>,
    ) -> Result<()> {
        self.record_metric(name.to_string(), 1.0, MetricType::Counter, labels)
            .await
    }

    /// Record a gauge metric
    pub async fn set_gauge(
        &self,
        name: &str,
        value: f64,
        labels: HashMap<String, String>,
    ) -> Result<()> {
        self.record_metric(name.to_string(), value, MetricType::Gauge, labels)
            .await
    }

    /// Record a histogram metric
    pub async fn record_histogram(
        &self,
        name: &str,
        value: f64,
        labels: HashMap<String, String>,
    ) -> Result<()> {
        self.record_metric(name.to_string(), value, MetricType::Histogram, labels)
            .await
    }

    /// Update health check status
    pub async fn update_health_check(
        &self,
        component: String,
        status: ComponentStatus,
        message: String,
        response_time_ms: u64,
        details: HashMap<String, String>,
    ) -> Result<()> {
        if !self.config.enable_health_checks {
            return Ok(());
        }

        let health_status = HealthStatus {
            component: component.clone(),
            status,
            message,
            last_check: Utc::now(),
            response_time_ms,
            details,
        };

        let mut health_checks = self.health_checks.write().await;
        health_checks.insert(component, health_status);

        Ok(())
    }

    /// Add alert rule
    pub async fn add_alert_rule(&self, rule: AlertRule) -> Result<()> {
        let mut alert_rules = self.alert_rules.write().await;
        alert_rules.insert(rule.id.clone(), rule);
        Ok(())
    }

    /// Evaluate alert rules
    pub async fn evaluate_alerts(&self) -> Result<Vec<Alert>> {
        let alert_rules = self.alert_rules.read().await;
        let metrics = self.metrics.read().await;
        let mut new_alerts = Vec::new();

        for rule in alert_rules.values() {
            if !rule.enabled {
                continue;
            }

            // Find recent metrics for this rule
            let recent_metrics: Vec<_> = metrics
                .iter()
                .filter(|m| {
                    m.name == rule.metric_name
                        && Utc::now().signed_duration_since(m.timestamp).num_seconds()
                            <= rule.duration_seconds as i64
                })
                .collect();

            if recent_metrics.is_empty() {
                continue;
            }

            // Calculate aggregate value (average for simplicity)
            let avg_value =
                recent_metrics.iter().map(|m| m.value).sum::<f64>() / recent_metrics.len() as f64;

            // Check condition
            let condition_met = match rule.condition {
                AlertCondition::GreaterThan => avg_value > rule.threshold,
                AlertCondition::LessThan => avg_value < rule.threshold,
                AlertCondition::Equal => (avg_value - rule.threshold).abs() < f64::EPSILON,
                AlertCondition::NotEqual => (avg_value - rule.threshold).abs() >= f64::EPSILON,
                AlertCondition::GreaterThanOrEqual => avg_value >= rule.threshold,
                AlertCondition::LessThanOrEqual => avg_value <= rule.threshold,
            };

            if condition_met {
                let alert = Alert {
                    id: Uuid::new_v4().to_string(),
                    rule_id: rule.id.clone(),
                    name: rule.name.clone(),
                    severity: rule.severity.clone(),
                    message: format!(
                        "Alert {} triggered: {} {} {} (current: {})",
                        rule.name,
                        rule.metric_name,
                        format!("{:?}", rule.condition),
                        rule.threshold,
                        avg_value
                    ),
                    started_at: Utc::now(),
                    last_triggered: Utc::now(),
                    labels: rule.labels.clone(),
                    resolved: false,
                    resolved_at: None,
                };

                new_alerts.push(alert.clone());

                // Store active alert
                let mut active_alerts = self.active_alerts.write().await;
                active_alerts.insert(alert.id.clone(), alert);
            }
        }

        Ok(new_alerts)
    }

    /// Record performance profile
    pub async fn record_performance_profile(&self, profile: PerformanceProfile) -> Result<()> {
        if !self.config.enable_profiling {
            return Ok(());
        }

        let mut profiles = self.performance_profiles.write().await;
        profiles.push_back(profile);

        // Limit buffer size
        while profiles.len() > 1000 {
            profiles.pop_front();
        }

        Ok(())
    }

    /// Update system information
    pub async fn update_system_info(&self, info: SystemInfo) -> Result<()> {
        let mut system_info = self.system_info.write().await;
        *system_info = info;
        Ok(())
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> Vec<MetricPoint> {
        let metrics = self.metrics.read().await;
        metrics.iter().cloned().collect()
    }

    /// Get health status
    pub async fn get_health_status(&self) -> HashMap<String, HealthStatus> {
        let health_checks = self.health_checks.read().await;
        health_checks.clone()
    }

    /// Get overall system health
    pub async fn get_overall_health(&self) -> ComponentStatus {
        let health_checks = self.health_checks.read().await;

        if health_checks.is_empty() {
            return ComponentStatus::Unknown;
        }

        let mut has_unhealthy = false;
        let mut has_degraded = false;

        for status in health_checks.values() {
            match status.status {
                ComponentStatus::Unhealthy => has_unhealthy = true,
                ComponentStatus::Degraded => has_degraded = true,
                ComponentStatus::Healthy => {}
                ComponentStatus::Unknown => has_degraded = true,
            }
        }

        if has_unhealthy {
            ComponentStatus::Unhealthy
        } else if has_degraded {
            ComponentStatus::Degraded
        } else {
            ComponentStatus::Healthy
        }
    }

    /// Get active alerts
    pub async fn get_active_alerts(&self) -> Vec<Alert> {
        let active_alerts = self.active_alerts.read().await;
        active_alerts.values().cloned().collect()
    }

    /// Get system information
    pub async fn get_system_info(&self) -> SystemInfo {
        let system_info = self.system_info.read().await;
        system_info.clone()
    }

    /// Get performance profiles
    pub async fn get_performance_profiles(&self) -> Vec<PerformanceProfile> {
        let profiles = self.performance_profiles.read().await;
        profiles.iter().cloned().collect()
    }

    /// Clean up old data
    pub async fn cleanup_old_data(&self) -> Result<()> {
        let cutoff_time = Utc::now() - Duration::hours(self.config.metrics_retention_hours as i64);

        // Clean up old metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.retain(|m| m.timestamp > cutoff_time);
        }

        // Clean up old performance profiles
        {
            let mut profiles = self.performance_profiles.write().await;
            profiles.retain(|p| p.timestamp > cutoff_time);
        }

        Ok(())
    }

    /// Start monitoring background tasks
    pub async fn start_background_tasks(&self) -> Result<()> {
        // Start alert evaluation task
        let monitoring_system = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(
                monitoring_system.config.alert_evaluation_interval_seconds,
            ));

            loop {
                interval.tick().await;
                if let Err(e) = monitoring_system.evaluate_alerts().await {
                    eprintln!("Alert evaluation error: {}", e);
                }
            }
        });

        // Start cleanup task
        let monitoring_system = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600)); // Every hour

            loop {
                interval.tick().await;
                if let Err(e) = monitoring_system.cleanup_old_data().await {
                    eprintln!("Cleanup error: {}", e);
                }
            }
        });

        Ok(())
    }
}

impl Clone for MonitoringSystem {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            metrics: Arc::clone(&self.metrics),
            health_checks: Arc::clone(&self.health_checks),
            alert_rules: Arc::clone(&self.alert_rules),
            active_alerts: Arc::clone(&self.active_alerts),
            performance_profiles: Arc::clone(&self.performance_profiles),
            system_info: Arc::clone(&self.system_info),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_monitoring_system_creation() {
        let config = MonitoringConfig::default();
        let monitoring = MonitoringSystem::new(config);

        let system_info = monitoring.get_system_info().await;
        assert!(!system_info.version.is_empty());
    }

    #[tokio::test]
    async fn test_metric_recording() {
        let config = MonitoringConfig::default();
        let monitoring = MonitoringSystem::new(config);

        let mut labels = HashMap::new();
        labels.insert("component".to_string(), "test".to_string());

        monitoring
            .increment_counter("test_counter", labels.clone())
            .await
            .unwrap();
        monitoring
            .set_gauge("test_gauge", 42.0, labels)
            .await
            .unwrap();

        let metrics = monitoring.get_metrics().await;
        assert_eq!(metrics.len(), 2);
    }

    #[tokio::test]
    async fn test_health_checks() {
        let config = MonitoringConfig::default();
        let monitoring = MonitoringSystem::new(config);

        let mut details = HashMap::new();
        details.insert("version".to_string(), "1.0.0".to_string());

        monitoring
            .update_health_check(
                "database".to_string(),
                ComponentStatus::Healthy,
                "All systems operational".to_string(),
                50,
                details,
            )
            .await
            .unwrap();

        let health_status = monitoring.get_health_status().await;
        assert_eq!(health_status.len(), 1);
        assert_eq!(
            health_status.get("database").unwrap().status,
            ComponentStatus::Healthy
        );

        let overall_health = monitoring.get_overall_health().await;
        assert_eq!(overall_health, ComponentStatus::Healthy);
    }

    #[tokio::test]
    async fn test_alert_rules() -> anyhow::Result<()> {
        let config = MonitoringConfig::default();
        let monitoring = MonitoringSystem::new(config);

        let rule = AlertRule {
            id: "test_rule".to_string(),
            name: "High CPU Usage".to_string(),
            metric_name: "cpu_usage".to_string(),
            condition: AlertCondition::GreaterThan,
            threshold: 80.0,
            duration_seconds: 300,
            severity: AlertSeverity::Warning,
            enabled: true,
            labels: HashMap::new(),
        Ok(())
        };

        monitoring.add_alert_rule(rule).await?;

        // Record high CPU usage
        monitoring
            .set_gauge("cpu_usage", 85.0, HashMap::new())
            .await
            .unwrap();

        let alerts = monitoring.evaluate_alerts().await?;
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].severity, AlertSeverity::Warning);
    }
}
