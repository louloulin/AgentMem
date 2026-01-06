//! Telemetry and monitoring system for AgentMem
//!
//! This module provides comprehensive telemetry capabilities including:
//! - Event tracking and logging
//! - Performance monitoring
//! - Adaptive optimization
//! - Real-time metrics collection
//! - Production-grade monitoring with Prometheus and Jaeger integration
//! - Structured logging and intelligent alerting

use agent_mem_traits::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info};

#[cfg(feature = "jaeger")]
use opentelemetry::trace::Tracer;

#[cfg(feature = "jaeger")]
use tracing_opentelemetry::OpenTelemetryLayer;

/// Memory event types for tracking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventType {
    /// Memory creation event
    MemoryCreated,
    /// Memory update event
    MemoryUpdated,
    /// Memory deletion event
    MemoryDeleted,
    /// Memory search event
    MemorySearched,
    /// Memory retrieval event
    MemoryRetrieved,
    /// Cache hit event
    CacheHit,
    /// Cache miss event
    CacheMiss,
    /// Performance optimization event
    OptimizationApplied,
    /// Error event
    Error,
    /// Custom event
    Custom(String),
}

/// Memory event for telemetry tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEvent {
    /// Event type
    pub event_type: EventType,
    /// Memory ID (if applicable)
    pub memory_id: Option<String>,
    /// User ID (if applicable)
    pub user_id: Option<String>,
    /// Agent ID (if applicable)
    pub agent_id: Option<String>,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Event duration (if applicable)
    pub duration: Option<Duration>,
    /// Event metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Success status
    pub success: bool,
    /// Error message (if any)
    pub error_message: Option<String>,
}

impl MemoryEvent {
    /// Create a new memory event
    pub fn new(event_type: EventType) -> Self {
        Self {
            event_type,
            memory_id: None,
            user_id: None,
            agent_id: None,
            timestamp: Utc::now(),
            duration: None,
            metadata: HashMap::new(),
            success: true,
            error_message: None,
        }
    }

    /// Set memory ID
    pub fn with_memory_id(mut self, memory_id: String) -> Self {
        self.memory_id = Some(memory_id);
        self
    }

    /// Set user ID
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Set agent ID
    pub fn with_agent_id(mut self, agent_id: String) -> Self {
        self.agent_id = Some(agent_id);
        self
    }

    /// Set duration
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Set success status
    pub fn with_success(mut self, success: bool) -> Self {
        self.success = success;
        self
    }

    /// Set error message
    pub fn with_error(mut self, error_message: String) -> Self {
        self.error_message = Some(error_message);
        self.success = false;
        self
    }
}

/// Event tracker for collecting and managing events
pub struct EventTracker {
    events: Arc<RwLock<Vec<MemoryEvent>>>,
    max_events: usize,
    enabled: bool,
    start_time: Instant,
}

impl EventTracker {
    /// Create a new event tracker
    pub fn new(max_events: usize, enabled: bool) -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            max_events,
            enabled,
            start_time: Instant::now(),
        }
    }

    /// Track a new event
    pub async fn track_event(&self, event: MemoryEvent) {
        if !self.enabled {
            return;
        }

        let mut events = self.events.write().await;
        events.push(event.clone());

        // Keep only the most recent events
        if events.len() > self.max_events {
            let excess = events.len() - self.max_events;
            events.drain(0..excess);
        }

        debug!("Event tracked: {:?}", event.event_type);
    }

    /// Get recent events
    pub async fn get_recent_events(&self, limit: Option<usize>) -> Vec<MemoryEvent> {
        let events = self.events.read().await;
        let limit = limit.unwrap_or(events.len());
        events.iter().rev().take(limit).cloned().collect()
    }

    /// Get events by type
    pub async fn get_events_by_type(&self, event_type: &EventType) -> Vec<MemoryEvent> {
        let events = self.events.read().await;
        events
            .iter()
            .filter(|e| &e.event_type == event_type)
            .cloned()
            .collect()
    }

    /// Get event statistics
    pub async fn get_event_stats(&self) -> EventStats {
        let events = self.events.read().await;
        let total_events = events.len();
        let mut event_counts = HashMap::new();
        let mut error_count = 0;
        let mut total_duration = Duration::from_secs(0);
        let mut duration_count = 0;

        for event in events.iter() {
            // Count by event type
            let type_key = format!("{:?}", event.event_type);
            *event_counts.entry(type_key).or_insert(0) += 1;

            // Count errors
            if !event.success {
                error_count += 1;
            }

            // Sum durations
            if let Some(duration) = event.duration {
                total_duration += duration;
                duration_count += 1;
            }
        }

        let average_duration = if duration_count > 0 {
            Some(total_duration / duration_count as u32)
        } else {
            None
        };

        EventStats {
            total_events,
            event_counts,
            error_count,
            error_events: error_count,
            error_rate: if total_events > 0 {
                error_count as f64 / total_events as f64
            } else {
                0.0
            },
            average_duration,
        }
    }

    /// Clear all events
    pub async fn clear_events(&self) {
        let mut events = self.events.write().await;
        events.clear();
        info!("Event tracker cleared");
    }

    /// Get telemetry report
    pub async fn get_telemetry_report(&self) -> SimpleTelemetryReport {
        let event_stats = self.get_event_stats().await;
        let uptime_seconds = self.start_time.elapsed().as_secs_f64();

        SimpleTelemetryReport {
            event_stats,
            uptime_seconds,
        }
    }
}

/// Event statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStats {
    pub total_events: usize,
    pub event_counts: HashMap<String, usize>,
    pub error_count: usize,
    pub error_events: usize,
    pub error_rate: f64,
    pub average_duration: Option<Duration>,
}

/// Simple telemetry report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleTelemetryReport {
    pub event_stats: EventStats,
    pub uptime_seconds: f64,
}

/// Performance monitor for real-time monitoring
pub struct PerformanceMonitor {
    start_time: Instant,
    metrics: Arc<RwLock<MonitoringMetrics>>,
    enabled: bool,
}

/// Monitoring metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringMetrics {
    pub uptime_seconds: u64,
    pub cpu_usage_percent: f64,
    pub memory_usage_bytes: usize,
    pub active_requests: usize,
    pub requests_per_second: f64,
    pub average_response_time_ms: f64,
    pub error_rate: f64,
    pub cache_hit_rate: f64,
}

/// Performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub average_response_time_ms: f64,
    pub throughput_requests_per_second: f64,
    pub memory_usage_bytes: usize,
    pub cache_hit_rate: f64,
    pub error_rate: f64,
    pub uptime_seconds: f64,
}

impl Default for MonitoringMetrics {
    fn default() -> Self {
        Self {
            uptime_seconds: 0,
            cpu_usage_percent: 0.0,
            memory_usage_bytes: 0,
            active_requests: 0,
            requests_per_second: 0.0,
            average_response_time_ms: 0.0,
            error_rate: 0.0,
            cache_hit_rate: 0.0,
        }
    }
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new(enabled: bool) -> Self {
        Self {
            start_time: Instant::now(),
            metrics: Arc::new(RwLock::new(MonitoringMetrics::default())),
            enabled,
        }
    }

    /// Update monitoring metrics
    pub async fn update_metrics(&self, metrics: MonitoringMetrics) {
        if !self.enabled {
            return;
        }

        let mut current_metrics = self.metrics.write().await;
        current_metrics.uptime_seconds = self.start_time.elapsed().as_secs();
        current_metrics.cpu_usage_percent = metrics.cpu_usage_percent;
        current_metrics.memory_usage_bytes = metrics.memory_usage_bytes;
        current_metrics.active_requests = metrics.active_requests;
        current_metrics.requests_per_second = metrics.requests_per_second;
        current_metrics.average_response_time_ms = metrics.average_response_time_ms;
        current_metrics.error_rate = metrics.error_rate;
        current_metrics.cache_hit_rate = metrics.cache_hit_rate;

        debug!("Performance metrics updated");
    }

    /// Get current monitoring metrics
    pub async fn get_metrics(&self) -> MonitoringMetrics {
        let mut metrics = self.metrics.read().await.clone();
        metrics.uptime_seconds = self.start_time.elapsed().as_secs();
        metrics
    }

    /// Record a request
    pub async fn record_request(&self, duration: Duration, success: bool) {
        if !self.enabled {
            return;
        }

        let mut metrics = self.metrics.write().await;

        // Update response time (simple moving average)
        let duration_ms = duration.as_millis() as f64;
        if metrics.average_response_time_ms == 0.0 {
            metrics.average_response_time_ms = duration_ms;
        } else {
            metrics.average_response_time_ms =
                (metrics.average_response_time_ms * 0.9) + (duration_ms * 0.1);
        }

        // Update error rate
        if !success {
            metrics.error_rate = (metrics.error_rate * 0.9) + 0.1;
        } else {
            metrics.error_rate *= 0.9;
        }

        debug!(
            "Request recorded: duration={:?}, success={}",
            duration, success
        );
    }

    /// Get performance report
    pub async fn get_performance_report(&self) -> PerformanceReport {
        let metrics = self.get_metrics().await;

        PerformanceReport {
            average_response_time_ms: metrics.average_response_time_ms,
            throughput_requests_per_second: metrics.requests_per_second,
            memory_usage_bytes: metrics.memory_usage_bytes,
            cache_hit_rate: metrics.cache_hit_rate,
            error_rate: metrics.error_rate,
            uptime_seconds: metrics.uptime_seconds as f64,
        }
    }

    /// Check if system is healthy
    pub async fn is_healthy(&self) -> bool {
        let metrics = self.get_metrics().await;

        // Define health thresholds
        metrics.cpu_usage_percent < 90.0
            && metrics.error_rate < 0.05  // Less than 5% error rate
            && metrics.average_response_time_ms < 1000.0 // Less than 1 second
    }

    /// Get health status
    pub async fn get_health_status(&self) -> HealthStatus {
        let metrics = self.get_metrics().await;
        let is_healthy = self.is_healthy().await;

        HealthStatus {
            healthy: is_healthy,
            uptime_seconds: metrics.uptime_seconds,
            cpu_usage_percent: metrics.cpu_usage_percent,
            memory_usage_bytes: metrics.memory_usage_bytes,
            error_rate: metrics.error_rate,
            response_time_ms: metrics.average_response_time_ms,
            timestamp: Utc::now(),
        }
    }
}

/// Health status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub healthy: bool,
    pub uptime_seconds: u64,
    pub cpu_usage_percent: f64,
    pub memory_usage_bytes: usize,
    pub error_rate: f64,
    pub response_time_ms: f64,
    pub timestamp: DateTime<Utc>,
}

/// Telemetry system configuration
#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    pub enabled: bool,
    pub max_events: usize,
    pub monitoring_interval_seconds: u64,
    pub health_check_interval_seconds: u64,
    pub adaptive_optimization_enabled: bool,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_events: 10000,
            monitoring_interval_seconds: 30,
            health_check_interval_seconds: 60,
            adaptive_optimization_enabled: true,
        }
    }
}

/// Main telemetry system
pub struct TelemetrySystem {
    config: TelemetryConfig,
    event_tracker: Arc<EventTracker>,
    performance_monitor: Arc<PerformanceMonitor>,
    adaptive_optimizer: Arc<AdaptiveOptimizer>,
}

impl TelemetrySystem {
    /// Create a new telemetry system
    pub fn new(config: TelemetryConfig) -> Self {
        let event_tracker = Arc::new(EventTracker::new(config.max_events, config.enabled));
        let performance_monitor = Arc::new(PerformanceMonitor::new(config.enabled));
        let adaptive_optimizer =
            Arc::new(AdaptiveOptimizer::new(config.adaptive_optimization_enabled));

        Self {
            config,
            event_tracker,
            performance_monitor,
            adaptive_optimizer,
        }
    }

    /// Get event tracker
    pub fn event_tracker(&self) -> Arc<EventTracker> {
        Arc::clone(&self.event_tracker)
    }

    /// Get performance monitor
    pub fn performance_monitor(&self) -> Arc<PerformanceMonitor> {
        Arc::clone(&self.performance_monitor)
    }

    /// Get adaptive optimizer
    pub fn adaptive_optimizer(&self) -> Arc<AdaptiveOptimizer> {
        Arc::clone(&self.adaptive_optimizer)
    }

    /// Track a memory event
    pub async fn track_event(&self, event: MemoryEvent) {
        self.event_tracker.track_event(event).await;
    }

    /// Get comprehensive telemetry report
    pub async fn get_telemetry_report(&self) -> TelemetryReport {
        let event_stats = self.event_tracker.get_event_stats().await;
        let monitoring_metrics = self.performance_monitor.get_metrics().await;
        let health_status = self.performance_monitor.get_health_status().await;
        let optimization_stats = self.adaptive_optimizer.get_optimization_stats().await;

        TelemetryReport {
            timestamp: Utc::now(),
            event_stats,
            monitoring_metrics,
            health_status,
            optimization_stats,
        }
    }

    /// Start background monitoring
    pub async fn start_monitoring(&self) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        info!("Telemetry system monitoring started");

        // In a real implementation, this would start background tasks
        // for periodic monitoring and optimization

        Ok(())
    }

    /// Stop monitoring
    pub async fn stop_monitoring(&self) -> Result<()> {
        info!("Telemetry system monitoring stopped");
        Ok(())
    }
}

/// Comprehensive telemetry report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryReport {
    pub timestamp: DateTime<Utc>,
    pub event_stats: EventStats,
    pub monitoring_metrics: MonitoringMetrics,
    pub health_status: HealthStatus,
    pub optimization_stats: OptimizationStats,
}

/// Adaptive optimizer for performance tuning
pub struct AdaptiveOptimizer {
    enabled: bool,
    optimization_history: Arc<RwLock<Vec<OptimizationEvent>>>,
    current_settings: Arc<RwLock<OptimizationSettings>>,
}

/// Optimization event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationEvent {
    pub timestamp: DateTime<Utc>,
    pub optimization_type: OptimizationType,
    pub old_value: f64,
    pub new_value: f64,
    pub performance_impact: f64,
    pub success: bool,
}

/// Optimization type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationType {
    BatchSize,
    CacheSize,
    ConcurrencyLimit,
    EmbeddingDimensions,
    QueryTimeout,
    Custom(String),
}

/// Current optimization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSettings {
    pub batch_size: usize,
    pub cache_size: usize,
    pub concurrency_limit: usize,
    pub embedding_dimensions: usize,
    pub query_timeout_ms: u64,
    pub last_updated: DateTime<Utc>,
}

impl Default for OptimizationSettings {
    fn default() -> Self {
        Self {
            batch_size: 100,
            cache_size: 1000,
            concurrency_limit: 10,
            embedding_dimensions: 1536,
            query_timeout_ms: 5000,
            last_updated: Utc::now(),
        }
    }
}

/// Optimization statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationStats {
    pub total_optimizations: usize,
    pub successful_optimizations: usize,
    pub performance_improvement_percent: f64,
    pub last_optimization: Option<DateTime<Utc>>,
    pub current_settings: OptimizationSettings,
}

impl AdaptiveOptimizer {
    /// Create a new adaptive optimizer
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            optimization_history: Arc::new(RwLock::new(Vec::new())),
            current_settings: Arc::new(RwLock::new(OptimizationSettings::default())),
        }
    }

    /// Optimize batch size based on performance data
    pub async fn optimize_batch_size(&self, performance_data: &PerformanceData) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let mut settings = self.current_settings.write().await;
        let old_batch_size = settings.batch_size;

        // Simple optimization logic - increase batch size if performance is good
        let new_batch_size = if performance_data.average_response_time_ms < 100.0 {
            (old_batch_size as f64 * 1.1) as usize
        } else {
            (old_batch_size as f64 * 0.9) as usize
        }
        .max(10)
        .min(1000);

        if new_batch_size != old_batch_size {
            settings.batch_size = new_batch_size;
            settings.last_updated = Utc::now();

            let optimization_event = OptimizationEvent {
                timestamp: Utc::now(),
                optimization_type: OptimizationType::BatchSize,
                old_value: old_batch_size as f64,
                new_value: new_batch_size as f64,
                performance_impact: 0.0, // Would be calculated based on actual performance
                success: true,
            };

            let mut history = self.optimization_history.write().await;
            history.push(optimization_event);

            info!(
                "Batch size optimized: {} -> {}",
                old_batch_size, new_batch_size
            );
        }

        Ok(())
    }

    /// Adjust cache strategy based on access patterns
    pub async fn adjust_cache_strategy(&self, access_patterns: &AccessPatterns) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let mut settings = self.current_settings.write().await;
        let old_cache_size = settings.cache_size;

        // Adjust cache size based on hit rate
        let new_cache_size = if access_patterns.cache_hit_rate < 0.7 {
            (old_cache_size as f64 * 1.2) as usize
        } else if access_patterns.cache_hit_rate > 0.95 {
            (old_cache_size as f64 * 0.9) as usize
        } else {
            old_cache_size
        }
        .max(100)
        .min(10000);

        if new_cache_size != old_cache_size {
            settings.cache_size = new_cache_size;
            settings.last_updated = Utc::now();

            let optimization_event = OptimizationEvent {
                timestamp: Utc::now(),
                optimization_type: OptimizationType::CacheSize,
                old_value: old_cache_size as f64,
                new_value: new_cache_size as f64,
                performance_impact: 0.0,
                success: true,
            };

            let mut history = self.optimization_history.write().await;
            history.push(optimization_event);

            info!(
                "Cache size optimized: {} -> {}",
                old_cache_size, new_cache_size
            );
        }

        Ok(())
    }

    /// Get optimization statistics
    pub async fn get_optimization_stats(&self) -> OptimizationStats {
        let history = self.optimization_history.read().await;
        let settings = self.current_settings.read().await;

        let total_optimizations = history.len();
        let successful_optimizations = history.iter().filter(|e| e.success).count();
        let last_optimization = history.last().map(|e| e.timestamp);

        // Calculate average performance improvement
        let performance_improvement_percent = if !history.is_empty() {
            history.iter().map(|e| e.performance_impact).sum::<f64>() / history.len() as f64
        } else {
            0.0
        };

        OptimizationStats {
            total_optimizations,
            successful_optimizations,
            performance_improvement_percent,
            last_optimization,
            current_settings: settings.clone(),
        }
    }
}

/// Performance data for optimization
#[derive(Debug, Clone)]
pub struct PerformanceData {
    pub average_response_time_ms: f64,
    pub throughput_requests_per_second: f64,
    pub error_rate: f64,
    pub memory_usage_bytes: usize,
}

/// Access patterns for cache optimization
#[derive(Debug, Clone)]
pub struct AccessPatterns {
    pub cache_hit_rate: f64,
    pub access_frequency: HashMap<String, usize>,
    pub temporal_patterns: Vec<AccessTime>,
}

/// Access time information
#[derive(Debug, Clone)]
pub struct AccessTime {
    pub timestamp: DateTime<Utc>,
    pub memory_id: String,
    pub access_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_memory_event_creation() {
        let event = MemoryEvent::new(EventType::MemoryCreated)
            .with_memory_id("test_memory_123".to_string())
            .with_user_id("user_456".to_string())
            .with_duration(Duration::from_millis(100));

        assert_eq!(event.event_type, EventType::MemoryCreated);
        assert_eq!(event.memory_id, Some("test_memory_123".to_string()));
        assert_eq!(event.user_id, Some("user_456".to_string()));
        assert!(event.duration.is_some());
        assert!(event.success);
    }

    #[tokio::test]
    async fn test_event_tracker() {
        let tracker = EventTracker::new(100, true);

        let event =
            MemoryEvent::new(EventType::MemoryCreated).with_memory_id("test_memory".to_string());

        tracker.track_event(event).await;

        let recent_events = tracker.get_recent_events(Some(10)).await;
        assert_eq!(recent_events.len(), 1);
        assert_eq!(recent_events[0].event_type, EventType::MemoryCreated);
    }

    #[tokio::test]
    async fn test_event_stats() {
        let tracker = EventTracker::new(100, true);

        // Track multiple events
        tracker
            .track_event(MemoryEvent::new(EventType::MemoryCreated))
            .await;
        tracker
            .track_event(MemoryEvent::new(EventType::MemoryUpdated))
            .await;
        tracker
            .track_event(
                MemoryEvent::new(EventType::MemoryCreated).with_error("Test error".to_string()),
            )
            .await;

        let stats = tracker.get_event_stats().await;
        assert_eq!(stats.total_events, 3);
        assert_eq!(stats.error_count, 1);
        assert!((stats.error_rate - 0.333).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_performance_monitor() {
        let monitor = PerformanceMonitor::new(true);

        let metrics = MonitoringMetrics {
            cpu_usage_percent: 50.0,
            memory_usage_bytes: 1024 * 1024,
            active_requests: 5,
            requests_per_second: 100.0,
            average_response_time_ms: 50.0,
            error_rate: 0.01,
            cache_hit_rate: 0.85,
            ..Default::default()
        };

        monitor.update_metrics(metrics).await;

        let current_metrics = monitor.get_metrics().await;
        assert_eq!(current_metrics.cpu_usage_percent, 50.0);
        assert_eq!(current_metrics.memory_usage_bytes, 1024 * 1024);

        let health_status = monitor.get_health_status().await;
        assert!(health_status.healthy);
    }

    #[tokio::test]
    async fn test_telemetry_system() {
        let config = TelemetryConfig::default();
        let telemetry = TelemetrySystem::new(config);

        let event =
            MemoryEvent::new(EventType::MemoryCreated).with_memory_id("test_memory".to_string());

        telemetry.track_event(event).await;

        let report = telemetry.get_telemetry_report().await;
        assert_eq!(report.event_stats.total_events, 1);
    }

    #[tokio::test]
    async fn test_adaptive_optimizer() {
        let optimizer = AdaptiveOptimizer::new(true);

        let performance_data = PerformanceData {
            average_response_time_ms: 50.0,
            throughput_requests_per_second: 200.0,
            error_rate: 0.01,
            memory_usage_bytes: 1024 * 1024,
        };

        optimizer
            .optimize_batch_size(&performance_data)
            .await
            .unwrap();

        let stats = optimizer.get_optimization_stats().await;
        assert!(stats.total_optimizations > 0);
    }

    #[tokio::test]
    async fn test_cache_optimization() {
        let optimizer = AdaptiveOptimizer::new(true);

        let access_patterns = AccessPatterns {
            cache_hit_rate: 0.6, // Low hit rate should increase cache size
            access_frequency: HashMap::new(),
            temporal_patterns: Vec::new(),
        };

        optimizer
            .adjust_cache_strategy(&access_patterns)
            .await
            .unwrap();

        let stats = optimizer.get_optimization_stats().await;
        assert!(stats.total_optimizations > 0);
    }
}

/// Production-grade telemetry system with Prometheus and Jaeger integration
pub struct ProductionTelemetrySystem {
    /// Event tracker
    event_tracker: Arc<EventTracker>,
    /// Performance monitor
    performance_monitor: Arc<PerformanceMonitor>,
    /// Adaptive optimizer
    adaptive_optimizer: Arc<AdaptiveOptimizer>,
    /// Prometheus metrics enabled
    prometheus_enabled: bool,
    /// Jaeger tracing enabled
    jaeger_enabled: bool,
}

impl ProductionTelemetrySystem {
    /// Create a new production telemetry system
    pub async fn new(config: ProductionTelemetryConfig) -> Result<Self> {
        let event_tracker = Arc::new(EventTracker::new(1000, config.enabled));
        let performance_monitor = Arc::new(PerformanceMonitor::new(config.enabled));
        let adaptive_optimizer = Arc::new(AdaptiveOptimizer::new(config.enabled));

        let system = Self {
            event_tracker,
            performance_monitor,
            adaptive_optimizer,
            prometheus_enabled: config.prometheus_enabled,
            jaeger_enabled: config.jaeger_enabled,
        };

        // Initialize Prometheus metrics if enabled
        if system.prometheus_enabled {
            system.initialize_prometheus().await?;
        }

        // Initialize Jaeger tracing if enabled
        if system.jaeger_enabled {
            system.initialize_jaeger().await?;
        }

        info!("Production telemetry system initialized");
        Ok(system)
    }

    /// Initialize Prometheus metrics
    #[cfg(feature = "prometheus")]
    async fn initialize_prometheus(&self) -> Result<()> {
        use metrics_exporter_prometheus::PrometheusBuilder;

        let builder = PrometheusBuilder::new();
        builder.install().map_err(|e| {
            agent_mem_traits::AgentMemError::internal_error(format!(
                "Failed to initialize Prometheus: {e}"
            ))
        })?;

        // Register core metrics
        metrics::describe_counter!("agentmem_requests_total", "Total number of requests");
        metrics::describe_counter!("agentmem_errors_total", "Total number of errors");
        metrics::describe_histogram!(
            "agentmem_request_duration_seconds",
            "Request duration in seconds"
        );
        metrics::describe_gauge!("agentmem_memory_usage_bytes", "Memory usage in bytes");
        metrics::describe_gauge!("agentmem_cache_hit_rate", "Cache hit rate");
        metrics::describe_gauge!(
            "agentmem_active_connections",
            "Number of active connections"
        );

        info!("Prometheus metrics initialized");
        Ok(())
    }

    #[cfg(not(feature = "prometheus"))]
    async fn initialize_prometheus(&self) -> Result<()> {
        debug!("Prometheus feature not enabled, skipping initialization");
        Ok(())
    }

    /// Initialize Jaeger tracing
    #[cfg(feature = "jaeger")]
    async fn initialize_jaeger(&self) -> Result<()> {
        use opentelemetry_jaeger::new_agent_pipeline;
        use tracing_subscriber::{layer::SubscriberExt, Registry};

        let tracer = new_agent_pipeline()
            .with_service_name("agentmem")
            .install_simple()
            .map_err(|e| {
                agent_mem_traits::AgentMemError::internal_error(format!(
                    "Failed to initialize Jaeger: {e}"
                ))
            })?;

        let telemetry = OpenTelemetryLayer::new(tracer);
        let subscriber = Registry::default().with(telemetry);

        // Try to set global default, but ignore error if already set
        if tracing::subscriber::set_global_default(subscriber).is_err() {
            tracing::warn!("Tracing subscriber already initialized, skipping Jaeger setup");
        }

        info!("Jaeger tracing initialized");
        Ok(())
    }

    #[cfg(not(feature = "jaeger"))]
    async fn initialize_jaeger(&self) -> Result<()> {
        debug!("Jaeger feature not enabled, skipping initialization");
        Ok(())
    }

    /// Track a memory operation
    pub async fn track_memory_operation(
        &self,
        operation: &str,
        user_id: Option<&str>,
        duration: Duration,
        success: bool,
    ) {
        // Track event
        let event = MemoryEvent::new(EventType::Custom(operation.to_string()))
            .with_duration(duration)
            .with_success(success);

        let event = if let Some(user_id) = user_id {
            event.with_user_id(user_id.to_string())
        } else {
            event
        };

        self.event_tracker.track_event(event).await;

        // Record performance metrics
        self.performance_monitor
            .record_request(duration, success)
            .await;

        // Update Prometheus metrics if enabled
        #[cfg(feature = "prometheus")]
        if self.prometheus_enabled {
            metrics::increment_counter!("agentmem_requests_total");
            if !success {
                metrics::increment_counter!("agentmem_errors_total");
            }
            metrics::histogram!("agentmem_request_duration_seconds", duration.as_secs_f64());
        }

        // Create Jaeger span if enabled
        #[cfg(feature = "jaeger")]
        if self.jaeger_enabled {
            let span = tracing::info_span!(
                "memory_operation",
                operation = operation,
                user_id = user_id,
                duration_ms = duration.as_millis(),
                success = success
            );
            let _enter = span.enter();
            info!("Memory operation completed");
        }
    }

    /// Collect system metrics
    pub async fn collect_system_metrics(&self) -> SystemMetrics {
        let telemetry_report = self.event_tracker.get_telemetry_report().await;
        let performance_report = self.performance_monitor.get_performance_report().await;

        SystemMetrics {
            total_requests: telemetry_report.event_stats.total_events as u64,
            error_rate: if telemetry_report.event_stats.total_events > 0 {
                telemetry_report.event_stats.error_events as f64
                    / telemetry_report.event_stats.total_events as f64
            } else {
                0.0
            },
            average_response_time_ms: performance_report.average_response_time_ms,
            throughput_rps: performance_report.throughput_requests_per_second,
            memory_usage_bytes: performance_report.memory_usage_bytes,
            cache_hit_rate: performance_report.cache_hit_rate,
            uptime_seconds: telemetry_report.uptime_seconds,
        }
    }

    /// Get health status
    pub async fn get_health_status(&self) -> ProductionHealthStatus {
        let metrics = self.collect_system_metrics().await;

        let status = if metrics.error_rate > 0.1 {
            "unhealthy"
        } else if metrics.error_rate > 0.05 {
            "degraded"
        } else {
            "healthy"
        };

        ProductionHealthStatus {
            status: status.to_string(),
            timestamp: Utc::now(),
            metrics: Some(metrics),
        }
    }
}

/// System metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub total_requests: u64,
    pub error_rate: f64,
    pub average_response_time_ms: f64,
    pub throughput_rps: f64,
    pub memory_usage_bytes: usize,
    pub cache_hit_rate: f64,
    pub uptime_seconds: f64,
}

/// Production health status for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionHealthStatus {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub metrics: Option<SystemMetrics>,
}

/// Production telemetry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionTelemetryConfig {
    pub enabled: bool,
    pub prometheus_enabled: bool,
    pub jaeger_enabled: bool,
    pub jaeger_endpoint: Option<String>,
    pub prometheus_port: Option<u16>,
    pub service_name: String,
}
