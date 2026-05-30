//! Predictive Health Monitoring for AgentMem
//!
//! This module provides predictive capabilities for system health monitoring:
//! - Anomaly detection using statistical methods
//! - Capacity trend analysis
//! - Self-healing recommendations
//! - Predictive alerting based on time-series forecasting
//!
//! # Features
//!
//! - Statistical anomaly detection (Z-score, IQR)
//! - Moving average trend analysis
//! - Capacity planning and forecasting
//! - Automated threshold adjustment
//! - Self-healing action recommendations

use crate::monitoring::{AlertSeverity, ComponentStatus, MetricPoint};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Predictive monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictiveConfig {
    /// Enable anomaly detection
    pub enable_anomaly_detection: bool,
    /// Enable capacity forecasting
    pub enable_forecasting: bool,
    /// Enable self-healing recommendations
    pub enable_self_healing: bool,
    /// Anomaly detection sensitivity (0.0 - 1.0)
    pub sensitivity: f64,
    /// Minimum data points for forecasting
    pub min_data_points: usize,
    /// Forecast horizon (in minutes)
    pub forecast_horizon_minutes: u32,
    /// Confidence level for predictions (0.0 - 1.0)
    pub confidence_level: f64,
}

impl Default for PredictiveConfig {
    fn default() -> Self {
        Self {
            enable_anomaly_detection: true,
            enable_forecasting: true,
            enable_self_healing: true,
            sensitivity: 0.95,
            min_data_points: 30,
            forecast_horizon_minutes: 60,
            confidence_level: 0.95,
        }
    }
}

/// Anomaly detection method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalyMethod {
    /// Z-score based detection
    ZScore,
    /// Interquartile range (IQR) based detection
    IQR,
    /// Exponential moving average deviation
    EMA,
    /// Isolation forest (simplified)
    IsolationForest,
}

/// Detected anomaly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    /// Unique anomaly ID
    pub id: String,
    /// Metric name
    pub metric_name: String,
    /// Anomaly type
    pub anomaly_type: AnomalyType,
    /// Severity of the anomaly
    pub severity: AnomalySeverity,
    /// Value that triggered the anomaly
    pub value: f64,
    /// Expected value (based on model)
    pub expected_value: f64,
    /// Deviation from expected
    pub deviation: f64,
    /// Detection timestamp
    pub detected_at: DateTime<Utc>,
    /// Labels associated with the metric
    pub labels: HashMap<String, String>,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
}

/// Types of anomalies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalyType {
    /// Value spike (sudden increase)
    Spike,
    /// Value drop (sudden decrease)
    Drop,
    /// Trend deviation
    TrendDeviation,
    /// Level shift
    LevelShift,
    /// Pattern change
    PatternChange,
    /// Outlier
    Outlier,
}

/// Anomaly severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalySeverity {
    /// Low severity anomaly
    Low,
    /// Medium severity anomaly
    Medium,
    /// High severity anomaly
    High,
    /// Critical anomaly
    Critical,
}

/// Capacity forecast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityForecast {
    /// Metric name
    pub metric_name: String,
    /// Current value
    pub current_value: f64,
    /// Forecasted values
    pub forecast: Vec<ForecastPoint>,
    /// When capacity will be exhausted (if predictable)
    pub exhaustion_time: Option<DateTime<Utc>>,
    /// Recommended capacity increase
    pub recommended_increase: f64,
    /// Confidence interval
    pub confidence_interval: ConfidenceInterval,
    /// Forecast timestamp
    pub forecast_at: DateTime<Utc>,
}

/// Single forecast point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastPoint {
    /// Time of forecast
    pub timestamp: DateTime<Utc>,
    /// Predicted value
    pub value: f64,
    /// Lower bound
    pub lower_bound: f64,
    /// Upper bound
    pub upper_bound: f64,
}

/// Confidence interval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceInterval {
    /// Lower bound (at confidence level)
    pub lower: f64,
    /// Upper bound (at confidence level)
    pub upper: f64,
    /// Confidence level
    pub level: f64,
}

/// Self-healing recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfHealingRecommendation {
    /// Unique recommendation ID
    pub id: String,
    /// Component affected
    pub component: String,
    /// Issue detected
    pub issue: String,
    /// Root cause analysis
    pub root_cause: Option<String>,
    /// Recommended action
    pub action: HealingAction,
    /// Estimated impact
    pub estimated_impact: String,
    /// Risk level of the action
    pub risk_level: RiskLevel,
    /// Priority (1-5, 1 is highest)
    pub priority: u8,
    /// Confidence in recommendation
    pub confidence: f64,
    /// Timestamp
    pub generated_at: DateTime<Utc>,
}

/// Healing action types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealingAction {
    /// Scale up resource
    ScaleUp { target: String, amount: f64 },
    /// Scale down resource
    ScaleDown { target: String, amount: f64 },
    /// Restart component
    Restart { component: String },
    /// Clear cache
    ClearCache { target: String },
    /// Increase timeout
    IncreaseTimeout { target: String, new_value: u64 },
    /// Adjust replica count
    AdjustReplicas { target: String, count: i32 },
    /// Enable failover
    EnableFailover { component: String },
    /// Trigger garbage collection
    TriggerGC,
    /// Reconfigure parameter
    Reconfigure { parameter: String, value: String },
    /// Manual intervention required
    ManualIntervention { instructions: String },
}

/// Risk levels for healing actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    /// No risk
    None,
    /// Low risk
    Low,
    /// Medium risk
    Medium,
    /// High risk
    High,
}

/// System health prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthPrediction {
    /// Predicted overall health status
    pub predicted_status: ComponentStatus,
    /// Time to potential failure (if applicable)
    pub time_to_failure: Option<Duration>,
    /// Contributing factors
    pub contributing_factors: Vec<String>,
    /// Risk score (0.0 - 1.0)
    pub risk_score: f64,
    /// Recommendations
    pub recommendations: Vec<SelfHealingRecommendation>,
    /// Prediction timestamp
    pub predicted_at: DateTime<Utc>,
    /// Prediction horizon
    pub horizon: Duration,
}

/// Health trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthTrend {
    /// Metric name
    pub metric_name: String,
    /// Trend direction
    pub direction: TrendDirection,
    /// Trend strength (0.0 - 1.0)
    pub strength: f64,
    /// Rate of change per hour
    pub rate_of_change: f64,
    /// Predicted time to threshold
    pub time_to_threshold: Option<Duration>,
    /// Historical values for trend calculation
    pub values: Vec<f64>,
}

/// Trend directions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrendDirection {
    /// Improving trend
    Improving,
    /// Stable trend
    Stable,
    /// Degrading trend
    Degrading,
    /// Unknown/no clear trend
    Unknown,
}

/// Statistical summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalSummary {
    pub count: usize,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub variance: f64,
    pub p25: f64,
    pub p75: f64,
    pub p90: f64,
    pub p95: f64,
    pub p99: f64,
}

/// Predictive monitoring engine
pub struct PredictiveMonitor {
    config: PredictiveConfig,
    historical_data: Arc<RwLock<HashMap<String, VecDeque<MetricPoint>>>>,
    anomalies: Arc<RwLock<VecDeque<Anomaly>>>,
    forecasts: Arc<RwLock<HashMap<String, CapacityForecast>>>,
    recommendations: Arc<RwLock<VecDeque<SelfHealingRecommendation>>>,
}

impl PredictiveMonitor {
    /// Create a new predictive monitor
    pub fn new(config: PredictiveConfig) -> Self {
        Self {
            config,
            historical_data: Arc::new(RwLock::new(HashMap::new())),
            anomalies: Arc::new(RwLock::new(VecDeque::new())),
            forecasts: Arc::new(RwLock::new(HashMap::new())),
            recommendations: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    /// Record a metric for predictive analysis
    pub async fn record_metric(&self, point: MetricPoint) {
        let mut data = self.historical_data.write().await;
        let queue = data.entry(point.name.clone()).or_insert_with(VecDeque::new);

        queue.push_back(point);

        // Keep only last 1000 points per metric
        while queue.len() > 1000 {
            queue.pop_front();
        }
    }

    /// Detect anomalies in a metric
    pub async fn detect_anomalies(&self, metric_name: &str) -> Vec<Anomaly> {
        if !self.config.enable_anomaly_detection {
            return Vec::new();
        }

        let data = self.historical_data.read().await;
        let points = match data.get(metric_name) {
            Some(p) => p,
            None => return Vec::new(),
        };

        if points.len() < self.config.min_data_points {
            return Vec::new();
        }

        let values: Vec<f64> = points.iter().map(|p| p.value).collect();
        let mut anomalies = Vec::new();

        // Z-score anomaly detection
        let z_score_anomalies = self.detect_z_score_anomalies(metric_name, &values, points);
        anomalies.extend(z_score_anomalies);

        // IQR anomaly detection
        let iqr_anomalies = self.detect_iqr_anomalies(metric_name, &values, points);
        anomalies.extend(iqr_anomalies);

        // Store detected anomalies
        let mut anomaly_store = self.anomalies.write().await;
        for anomaly in &anomalies {
            anomaly_store.push_back(anomaly.clone());
        }

        // Keep only recent anomalies
        while anomaly_store.len() > 500 {
            anomaly_store.pop_front();
        }

        anomalies
    }

    /// Z-score based anomaly detection
    fn detect_z_score_anomalies(
        &self,
        metric_name: &str,
        values: &[f64],
        points: &VecDeque<MetricPoint>,
    ) -> Vec<Anomaly> {
        let mut anomalies = Vec::new();

        let stats = self.calculate_statistics(values);
        let last_idx = values.len() - 1;
        let last_value = values[last_idx];
        let last_point = &points[last_idx];

        // Calculate Z-score for the latest value
        let z_score = if stats.std_dev > 0.0 {
            (last_value - stats.mean) / stats.std_dev
        } else {
            0.0
        };

        // Threshold based on sensitivity
        let threshold = 2.0 + (1.0 - self.config.sensitivity) * 2.0;

        if z_score.abs() > threshold {
            let anomaly_type = if z_score > 0.0 {
                AnomalyType::Spike
            } else {
                AnomalyType::Drop
            };

            let severity = if z_score.abs() > threshold * 2.0 {
                AnomalySeverity::Critical
            } else if z_score.abs() > threshold * 1.5 {
                AnomalySeverity::High
            } else {
                AnomalySeverity::Medium
            };

            anomalies.push(Anomaly {
                id: format!("anomaly_{}_{}", metric_name, Utc::now().timestamp()),
                metric_name: metric_name.to_string(),
                anomaly_type,
                severity,
                value: last_value,
                expected_value: stats.mean,
                deviation: z_score,
                detected_at: Utc::now(),
                labels: last_point.labels.clone(),
                confidence: self.config.sensitivity,
            });
        }

        anomalies
    }

    /// IQR (Interquartile Range) anomaly detection
    fn detect_iqr_anomalies(
        &self,
        metric_name: &str,
        values: &[f64],
        points: &VecDeque<MetricPoint>,
    ) -> Vec<Anomaly> {
        let mut anomalies = Vec::new();

        let stats = self.calculate_statistics(values);
        let iqr = stats.p75 - stats.p25;
        let last_idx = values.len() - 1;
        let last_value = values[last_idx];
        let last_point = &points[last_idx];

        let lower_bound = stats.p25 - 1.5 * iqr;
        let upper_bound = stats.p75 + 1.5 * iqr;

        if last_value < lower_bound || last_value > upper_bound {
            let anomaly_type = if last_value < lower_bound {
                AnomalyType::Drop
            } else {
                AnomalyType::Spike
            };

            let deviation = if last_value < lower_bound {
                lower_bound - last_value
            } else {
                last_value - upper_bound
            };

            anomalies.push(Anomaly {
                id: format!("anomaly_iqr_{}_{}", metric_name, Utc::now().timestamp()),
                metric_name: metric_name.to_string(),
                anomaly_type,
                severity: AnomalySeverity::Medium,
                value: last_value,
                expected_value: stats.median,
                deviation,
                detected_at: Utc::now(),
                labels: last_point.labels.clone(),
                confidence: 0.85,
            });
        }

        anomalies
    }

    /// Calculate statistics for a set of values
    fn calculate_statistics(&self, values: &[f64]) -> StatisticalSummary {
        if values.is_empty() {
            return StatisticalSummary {
                count: 0,
                min: 0.0,
                max: 0.0,
                mean: 0.0,
                median: 0.0,
                std_dev: 0.0,
                variance: 0.0,
                p25: 0.0,
                p75: 0.0,
                p90: 0.0,
                p95: 0.0,
                p99: 0.0,
            };
        }

        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let count = sorted.len();
        let sum: f64 = sorted.iter().sum();
        let mean = sum / count as f64;

        let variance = if count > 1 {
            sorted.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / count as f64
        } else {
            0.0
        };

        let std_dev = variance.sqrt();

        let percentile = |p: f64| -> f64 {
            let idx = ((p / 100.0) * (count - 1) as f64).round() as usize;
            sorted[idx.min(count - 1)]
        };

        StatisticalSummary {
            count,
            min: sorted[0],
            max: sorted[count - 1],
            mean,
            median: percentile(50.0),
            std_dev,
            variance,
            p25: percentile(25.0),
            p75: percentile(75.0),
            p90: percentile(90.0),
            p95: percentile(95.0),
            p99: percentile(99.0),
        }
    }

    /// Generate capacity forecast
    pub async fn forecast_capacity(&self, metric_name: &str) -> Option<CapacityForecast> {
        if !self.config.enable_forecasting {
            return None;
        }

        let data = self.historical_data.read().await;
        let points = match data.get(metric_name) {
            Some(p) if p.len() >= self.config.min_data_points => p,
            _ => return None,
        };

        let values: Vec<f64> = points.iter().map(|p| p.value).collect();
        let stats = self.calculate_statistics(&values);

        // Simple linear regression for trend
        let n = values.len() as f64;
        let x_mean = (n - 1.0) / 2.0;
        let y_mean = stats.mean;

        let mut numerator = 0.0;
        let mut denominator = 0.0;
        for (i, &y) in values.iter().enumerate() {
            let x = i as f64;
            numerator += (x - x_mean) * (y - y_mean);
            denominator += (x - x_mean).powi(2);
        }

        let slope = if denominator > 0.0 { numerator / denominator } else { 0.0 };
        let intercept = y_mean - slope * x_mean;

        // Generate forecast points
        let mut forecast_points = Vec::new();
        let horizon_points = self.config.forecast_horizon_minutes as usize / 5; // Assume 5-minute intervals

        let last_x = n - 1.0;
        let last_value = values.last().copied().unwrap_or(0.0);

        // Calculate standard error for confidence intervals
        let residuals: Vec<f64> = values
            .iter()
            .enumerate()
            .map(|(i, &y)| {
                let predicted = slope * i as f64 + intercept;
                y - predicted
            })
            .collect();
        let se = if residuals.len() > 2 {
            residuals.iter().map(|r| r.powi(2)).sum::<f64>() / (residuals.len() - 2) as f64
        } else {
            stats.std_dev
        };
        let se = se.sqrt();

        // Critical value for confidence interval (approximate)
        let critical_value = 1.96; // 95% confidence

        for i in 1..=horizon_points.min(100) {
            let future_x = last_x + i as f64;
            let predicted = slope * future_x + intercept;
            let margin = critical_value * se * (1.0 + i as f64 / n).sqrt();

            forecast_points.push(ForecastPoint {
                timestamp: Utc::now() + Duration::minutes((i * 5) as i64),
                value: predicted,
                lower_bound: (predicted - margin).max(0.0),
                upper_bound: predicted + margin,
            });
        }

        // Calculate when capacity might be exhausted (if trending up)
        let exhaustion_time = if slope > 0.0 {
            let capacity_limit = stats.p99 * 1.5; // Assume 150% of P99 is capacity limit
            if predicted_at_horizon(&forecast_points) > capacity_limit {
                // Estimate when it will hit capacity
                let time_to_capacity = (capacity_limit - last_value) / slope;
                if time_to_capacity.is_finite() && time_to_capacity > 0.0 {
                    Some(Utc::now() + Duration::minutes(time_to_capacity as i64))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let current_value = values.last().copied().unwrap_or(0.0);
        let predicted_at_horizon = forecast_points.last().map(|p| p.value).unwrap_or(current_value);
        let recommended_increase = if slope > 0.0 {
            (predicted_at_horizon - current_value).max(0.0) * 1.2 // 20% buffer
        } else {
            0.0
        };

        let first_bound = forecast_points.first().map(|p| p.lower_bound).unwrap_or(current_value);
        let last_bound = forecast_points.last().map(|p| p.upper_bound).unwrap_or(current_value);

        let forecast = CapacityForecast {
            metric_name: metric_name.to_string(),
            current_value,
            forecast: forecast_points,
            exhaustion_time,
            recommended_increase,
            confidence_interval: ConfidenceInterval {
                lower: first_bound,
                upper: last_bound,
                level: self.config.confidence_level,
            },
            forecast_at: Utc::now(),
        };

        // Store forecast
        let mut forecasts = self.forecasts.write().await;
        forecasts.insert(metric_name.to_string(), forecast.clone());

        Some(forecast)
    }

    /// Analyze health trends
    pub async fn analyze_trends(&self, metric_name: &str) -> Option<HealthTrend> {
        let data = self.historical_data.read().await;
        let points = data.get(metric_name)?;

        if points.len() < 10 {
            return None;
        }

        let values: Vec<f64> = points.iter().map(|p| p.value).collect();
        let stats = self.calculate_statistics(&values);

        // Calculate trend using simple linear regression
        let n = values.len() as f64;
        let x_mean = (n - 1.0) / 2.0;
        let y_mean = stats.mean;

        let mut numerator = 0.0;
        let mut denominator = 0.0;
        for (i, &y) in values.iter().enumerate() {
            let x = i as f64;
            numerator += (x - x_mean) * (y - y_mean);
            denominator += (x - x_mean).powi(2);
        }

        let slope = if denominator > 0.0 { numerator / denominator } else { 0.0 };
        let intercept = y_mean - slope * x_mean;

        // Calculate R-squared for trend strength
        let ss_res: f64 = values
            .iter()
            .enumerate()
            .map(|(i, &y)| {
                let predicted = slope * i as f64 + intercept;
                (y - predicted).powi(2)
            })
            .sum();
        let ss_tot: f64 = values.iter().map(|&y| (y - y_mean).powi(2)).sum();
        let r_squared = if ss_tot > 0.0 { 1.0 - ss_res / ss_tot } else { 0.0 };

        // Determine trend direction
        let direction = if slope > stats.std_dev / 10.0 {
            TrendDirection::Improving
        } else if slope < -stats.std_dev / 10.0 {
            TrendDirection::Degrading
        } else {
            TrendDirection::Stable
        };

        // Calculate time to threshold (if degrading)
        let time_to_threshold = if slope < 0.0 {
            let threshold = stats.p25; // Use P25 as threshold
            let current = *values.last().unwrap_or(&stats.mean);
            if current > threshold {
                let diff = current - threshold;
                if slope.abs() > 0.0 {
                    Some(Duration::minutes((diff / slope.abs()) as i64))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        Some(HealthTrend {
            metric_name: metric_name.to_string(),
            direction,
            strength: r_squared.abs().sqrt(),
            rate_of_change: slope * 60.0, // Per hour
            time_to_threshold,
            values,
        })
    }

    /// Generate self-healing recommendations
    pub async fn generate_recommendations(&self, metric_name: &str) -> Vec<SelfHealingRecommendation> {
        if !self.config.enable_self_healing {
            return Vec::new();
        }

        let mut recommendations = Vec::new();

        // Check for anomalies
        let anomalies = self.detect_anomalies(metric_name).await;
        for anomaly in anomalies {
            if let Some(rec) = self.generate_recommendation_for_anomaly(&anomaly).await {
                recommendations.push(rec);
            }
        }

        // Check for trends
        if let Some(trend) = self.analyze_trends(metric_name).await {
            if let Some(rec) = self.generate_recommendation_for_trend(&trend) {
                recommendations.push(rec);
            }
        }

        // Check for capacity issues
        if let Some(forecast) = self.forecast_capacity(metric_name).await {
            if let Some(rec) = self.generate_recommendation_for_forecast(&forecast) {
                recommendations.push(rec);
            }
        }

        // Store recommendations
        let mut rec_store = self.recommendations.write().await;
        for rec in &recommendations {
            rec_store.push_back(rec.clone());
        }

        recommendations
    }

    /// Generate recommendation for an anomaly
    async fn generate_recommendation_for_anomaly(&self, anomaly: &Anomaly) -> Option<SelfHealingRecommendation> {
        let action = match anomaly.anomaly_type {
            AnomalyType::Spike => HealingAction::ClearCache {
                target: "memory_cache".to_string(),
            },
            AnomalyType::Drop => HealingAction::IncreaseTimeout {
                target: anomaly.metric_name.clone(),
                new_value: 60000,
            },
            AnomalyType::LevelShift => HealingAction::Restart {
                component: anomaly.metric_name.clone(),
            },
            _ => return None,
        };

        Some(SelfHealingRecommendation {
            id: format!("rec_{}_{}", anomaly.id, Utc::now().timestamp()),
            component: anomaly.metric_name.clone(),
            issue: format!("{:?} anomaly detected", anomaly.anomaly_type),
            root_cause: Some("Anomalous metric behavior detected".to_string()),
            action,
            estimated_impact: "Resolve metric anomaly".to_string(),
            risk_level: RiskLevel::Low,
            priority: match anomaly.severity {
                AnomalySeverity::Critical => 1,
                AnomalySeverity::High => 2,
                AnomalySeverity::Medium => 3,
                AnomalySeverity::Low => 4,
            },
            confidence: anomaly.confidence,
            generated_at: Utc::now(),
        })
    }

    /// Generate recommendation for a trend
    fn generate_recommendation_for_trend(&self, trend: &HealthTrend) -> Option<SelfHealingRecommendation> {
        match trend.direction {
            TrendDirection::Degrading => {
                if trend.strength > 0.7 {
                    Some(SelfHealingRecommendation {
                        id: format!("rec_trend_{}_{}", trend.metric_name, Utc::now().timestamp()),
                        component: trend.metric_name.clone(),
                        issue: "Degrading trend detected".to_string(),
                        root_cause: Some("Consistent negative trend in metric".to_string()),
                        action: HealingAction::ScaleUp {
                            target: trend.metric_name.clone(),
                            amount: 20.0,
                        },
                        estimated_impact: "Prevent resource exhaustion".to_string(),
                        risk_level: RiskLevel::Medium,
                        priority: if trend.time_to_threshold.is_some() { 2 } else { 4 },
                        confidence: trend.strength,
                        generated_at: Utc::now(),
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Generate recommendation for a forecast
    fn generate_recommendation_for_forecast(&self, forecast: &CapacityForecast) -> Option<SelfHealingRecommendation> {
        if let Some(exhaustion) = forecast.exhaustion_time {
            let hours_until = (exhaustion - Utc::now()).num_hours();

            if hours_until > 0 && hours_until < 168 { // Less than 1 week
                return Some(SelfHealingRecommendation {
                    id: format!("rec_capacity_{}", Utc::now().timestamp()),
                    component: forecast.metric_name.clone(),
                    issue: format!("Capacity exhaustion predicted in {} hours", hours_until),
                    root_cause: Some("Capacity trending towards exhaustion".to_string()),
                    action: HealingAction::ScaleUp {
                        target: forecast.metric_name.clone(),
                        amount: forecast.recommended_increase,
                    },
                    estimated_impact: "Prevent service disruption".to_string(),
                    risk_level: RiskLevel::High,
                    priority: if hours_until < 24 { 1 } else { 2 },
                    confidence: 0.8,
                    generated_at: Utc::now(),
                });
            }
        }
        None
    }

    /// Predict overall system health
    pub async fn predict_health(&self) -> HealthPrediction {
        let mut risk_score = 0.0;
        let mut contributing_factors = Vec::new();
        let mut recommendations = Vec::new();
        let mut time_to_failure: Option<Duration> = None;

        // Analyze all tracked metrics - collect metric names first
        let metric_names: Vec<String> = {
            let data = self.historical_data.read().await;
            data.keys().cloned().collect()
        };

        for metric_name in metric_names {
            // Check if metric still has enough data points
            let has_enough_data = {
                let data = self.historical_data.read().await;
                data.get(&metric_name).map(|p| p.len() >= self.config.min_data_points).unwrap_or(false)
            };

            if has_enough_data {
                // Check for anomalies
                let anomalies = self.detect_anomalies(&metric_name).await;
                for anomaly in &anomalies {
                    risk_score += match anomaly.severity {
                        AnomalySeverity::Critical => 0.3,
                        AnomalySeverity::High => 0.2,
                        AnomalySeverity::Medium => 0.1,
                        AnomalySeverity::Low => 0.05,
                    };
                    contributing_factors.push(format!("{:?}: {:?}", anomaly.anomaly_type, anomaly.severity));
                }

                // Check trends
                if let Some(trend) = self.analyze_trends(&metric_name).await {
                    if trend.direction == TrendDirection::Degrading {
                        risk_score += trend.strength * 0.15;
                        contributing_factors.push(format!("Degrading trend: {:?}", trend.strength));

                        if let Some(tt) = trend.time_to_threshold {
                            if time_to_failure.is_none() || tt < time_to_failure.unwrap() {
                                time_to_failure = Some(tt);
                            }
                        }
                    }
                }

                // Check forecasts
                if let Some(forecast) = self.forecast_capacity(&metric_name).await {
                    if forecast.exhaustion_time.is_some() {
                        risk_score += 0.25;
                        contributing_factors.push("Capacity exhaustion predicted".to_string());
                    }
                }

                // Generate recommendations
                let recs = self.generate_recommendations(&metric_name).await;
                recommendations.extend(recs);
            }
        }

        // Normalize risk score
        risk_score = risk_score.min(1.0);

        // Determine predicted status
        let predicted_status = if risk_score < 0.2 {
            ComponentStatus::Healthy
        } else if risk_score < 0.5 {
            ComponentStatus::Degraded
        } else {
            ComponentStatus::Unhealthy
        };

        // Sort recommendations by priority
        recommendations.sort_by_key(|r| r.priority);

        HealthPrediction {
            predicted_status,
            time_to_failure,
            contributing_factors,
            risk_score,
            recommendations,
            predicted_at: Utc::now(),
            horizon: Duration::minutes(self.config.forecast_horizon_minutes as i64),
        }
    }

    /// Get all detected anomalies
    pub async fn get_anomalies(&self, limit: Option<usize>) -> Vec<Anomaly> {
        let anomalies = self.anomalies.read().await;
        let limit = limit.unwrap_or(100);
        anomalies.iter().rev().take(limit).cloned().collect()
    }

    /// Get all recommendations
    pub async fn get_recommendations(&self, limit: Option<usize>) -> Vec<SelfHealingRecommendation> {
        let recommendations = self.recommendations.read().await;
        let limit = limit.unwrap_or(50);
        recommendations.iter().rev().take(limit).cloned().collect()
    }

    /// Get forecast for a metric
    pub async fn get_forecast(&self, metric_name: &str) -> Option<CapacityForecast> {
        let forecasts = self.forecasts.read().await;
        forecasts.get(metric_name).cloned()
    }

    /// Clear historical data for a metric
    pub async fn clear_data(&self, metric_name: &str) {
        let mut data = self.historical_data.write().await;
        data.remove(metric_name);
    }
}

/// Helper function for forecast value prediction
fn predicted_at_horizon(forecast: &[ForecastPoint]) -> f64 {
    forecast.last().map(|p| p.value).unwrap_or(0.0)
}

impl HealingAction {
    /// Convert to any type for type erasure (used in recommendation generation)
    #[allow(clippy::should_implement_trait)]
    pub fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
#[cfg(feature = "inline_tests")]
mod tests {
    use super::*;

    fn create_test_metric(name: &str, value: f64) -> MetricPoint {
        MetricPoint {
            name: name.to_string(),
            value,
            timestamp: Utc::now(),
            labels: HashMap::new(),
            metric_type: crate::monitoring::MetricType::Gauge,
        }
    }

    #[tokio::test]
    async fn test_anomaly_detection_spike() {
        let monitor = PredictiveMonitor::new(PredictiveConfig::default());

        // Add normal values
        for _ in 0..50 {
            monitor.record_metric(create_test_metric("test_metric", 100.0)).await;
        }

        // Add a spike
        monitor.record_metric(create_test_metric("test_metric", 300.0)).await;

        let anomalies = monitor.detect_anomalies("test_metric").await;
        assert!(!anomalies.is_empty());
        assert!(matches!(anomalies[0].anomaly_type, AnomalyType::Spike));
    }

    #[tokio::test]
    async fn test_anomaly_detection_drop() {
        let monitor = PredictiveMonitor::new(PredictiveConfig::default());

        // Add normal values
        for _ in 0..50 {
            monitor.record_metric(create_test_metric("test_metric", 100.0)).await;
        }

        // Add a drop
        monitor.record_metric(create_test_metric("test_metric", 10.0)).await;

        let anomalies = monitor.detect_anomalies("test_metric").await;
        assert!(!anomalies.is_empty());
    }

    #[tokio::test]
    async fn test_capacity_forecast() {
        let monitor = PredictiveMonitor::new(PredictiveConfig::default());

        // Add increasing values
        for i in 0..50 {
            monitor.record_metric(create_test_metric("capacity", 50.0 + i as f64)).await;
        }

        let forecast = monitor.forecast_capacity("capacity").await;
        assert!(forecast.is_some());

        let forecast = forecast.unwrap();
        assert!(!forecast.forecast.is_empty());
        assert!(forecast.current_value < forecast.forecast.last().unwrap().value);
    }

    #[tokio::test]
    async fn test_trend_analysis_degrading() {
        let monitor = PredictiveMonitor::new(PredictiveConfig::default());

        // Add decreasing values
        for i in 0..50 {
            monitor.record_metric(create_test_metric("degrading", 100.0 - i as f64)).await;
        }

        let trend = monitor.analyze_trends("degrading").await;
        assert!(trend.is_some());

        let trend = trend.unwrap();
        assert!(matches!(trend.direction, TrendDirection::Degrading));
    }

    #[tokio::test]
    async fn test_health_prediction() {
        let monitor = PredictiveMonitor::new(PredictiveConfig::default());

        // Add some normal data
        for i in 0..50 {
            monitor.record_metric(create_test_metric("health", 100.0 + (i % 10) as f64)).await;
        }

        let prediction = monitor.predict_health().await;
        assert!(prediction.risk_score >= 0.0);
        assert!(prediction.risk_score <= 1.0);
    }

    #[tokio::test]
    async fn test_self_healing_recommendations() {
        let monitor = PredictiveMonitor::new(PredictiveConfig::default());

        // Add data with a spike
        for _ in 0..50 {
            monitor.record_metric(create_test_metric("recommendation_test", 100.0)).await;
        }
        monitor.record_metric(create_test_metric("recommendation_test", 500.0)).await;

        let recommendations = monitor.generate_recommendations("recommendation_test").await;
        // Should generate at least one recommendation for the spike
        assert!(!recommendations.is_empty());
    }
}
