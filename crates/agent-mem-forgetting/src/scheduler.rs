//! Forgetting Scheduler
//!
//! Automatic scheduler for checking and forgetting memories based on retention rates.
//!
//! # Theory
//!
//! The forgetting scheduler periodically checks memories and determines which should be
//! forgotten based on:
//! - Time elapsed since creation/access
//! - Ebbinghaus forgetting curve retention rate
//! - Memory protection levels
//! - Configurable forgetting threshold
//!
//! # Example
//!
//! ```no_run
//! use agent_mem_forgetting::{ForgettingConfig, ForgettingScheduler};
//! use agent_mem_forgetting::curve::EbbinghausCurve;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = ForgettingConfig::default()
//!         .with_check_interval(3600); // Check every hour
//!
//!     let scheduler = ForgettingScheduler::new(config).await?;
//!
//!     // Start automatic forgetting
//!     scheduler.start().await?;
//!
//!     Ok(())
//! }
//! ```

use crate::curve::{EbbinghausCurve, ForgettingCurve};
use crate::protection::{MemoryProtection, ProtectionLevel};
use agent_mem_core::memories::Memory;
use agent_mem_event_bus::{EventBus, EventType};
use agent_mem_traits::{AgentMemError, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

/// Configuration for forgetting scheduler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgettingConfig {
    /// Interval between forgetting checks (seconds)
    pub check_interval_seconds: u64,

    /// Retention threshold below which memories are forgotten (0-1)
    pub forgetting_threshold: f64,

    /// Default memory strength (time units)
    pub default_strength: f64,

    /// Enable event publishing for forget operations
    pub enable_events: bool,

    /// Maximum memories to check per run
    pub max_memories_per_check: usize,
}

impl Default for ForgettingConfig {
    fn default() -> Self {
        Self {
            check_interval_seconds: 3600, // 1 hour
            forgetting_threshold: 0.1,   // 10% retention
            default_strength: 7.0,       // 7 days
            enable_events: true,
            max_memories_per_check: 1000,
        }
    }
}

impl ForgettingConfig {
    /// Set check interval
    pub fn with_check_interval(mut self, seconds: u64) -> Self {
        self.check_interval_seconds = seconds;
        self
    }

    /// Set forgetting threshold
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        assert!(threshold > 0.0 && threshold < 1.0, "Threshold must be in (0, 1)");
        self.forgetting_threshold = threshold;
        self
    }

    /// Set default memory strength
    pub fn with_strength(mut self, strength: f64) -> Self {
        assert!(strength > 0.0, "Strength must be positive");
        self.default_strength = strength;
        self
    }

    /// Enable/disable event publishing
    pub fn with_events(mut self, enable: bool) -> Self {
        self.enable_events = enable;
        self
    }

    /// Set max memories per check
    pub fn with_max_memories(mut self, max: usize) -> Self {
        self.max_memories_per_check = max;
        self
    }
}

/// Statistics for forgetting scheduler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgettingStats {
    /// Total forgetting checks performed
    pub total_checks: u64,

    /// Total memories forgotten
    pub total_forgotten: u64,

    /// Total memories checked
    pub total_checked: u64,

    /// Memories protected from forgetting
    pub total_protected: u64,

    /// Last check timestamp
    pub last_check_at: Option<DateTime<Utc>>,

    /// Next scheduled check
    pub next_check_at: Option<DateTime<Utc>>,
}

impl Default for ForgettingStats {
    fn default() -> Self {
        Self {
            total_checks: 0,
            total_forgotten: 0,
            total_checked: 0,
            total_protected: 0,
            last_check_at: None,
            next_check_at: None,
        }
    }
}

/// Forgetting scheduler
///
/// Periodically checks memories and forgets those below retention threshold.
pub struct ForgettingScheduler {
    config: ForgettingConfig,
    curve: EbbinghausCurve,
    protection: MemoryProtection,
    event_bus: Option<EventBus>,
    stats: Arc<RwLock<ForgettingStats>>,
    running: Arc<RwLock<bool>>,
    task_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
}

impl ForgettingScheduler {
    /// Create new forgetting scheduler
    ///
    /// # Parameters
    ///
    /// - `config`: Scheduler configuration
    pub async fn new(config: ForgettingConfig) -> Result<Self> {
        let curve = EbbinghausCurve::with_strength(config.default_strength);
        let protection = MemoryProtection::new();

        Ok(Self {
            config,
            curve,
            protection,
            event_bus: None,
            stats: Arc::new(RwLock::new(ForgettingStats::default())),
            running: Arc::new(RwLock::new(false)),
            task_handle: Arc::new(RwLock::new(None)),
        })
    }

    /// Create with event bus
    pub async fn with_event_bus(mut self, event_bus: EventBus) -> Self {
        self.event_bus = Some(event_bus);
        self
    }

    /// Get memory protection manager
    pub fn protection(&self) -> &MemoryProtection {
        &self.protection
    }

    /// Start automatic forgetting scheduler
    ///
    /// Returns error if already running.
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Err(AgentMemError::other("Scheduler already running"));
        }

        *running = true;
        drop(running);

        info!(
            "Starting forgetting scheduler with interval: {}s",
            self.config.check_interval_seconds
        );

        let interval = StdDuration::from_secs(self.config.check_interval_seconds);
        let curve = self.curve.clone();
        let protection = self.protection.clone();
        let event_bus = self.event_bus.clone();
        let stats = self.stats.clone();
        let running = Arc::clone(&self.running);
        let threshold = self.config.forgetting_threshold;
        let enable_events = self.config.enable_events;

        let handle = tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            ticker.tick().await; // Skip first immediate tick

            while *running.read().await {
                ticker.tick().await;

                debug!("Running forgetting check");
                let mut stats_lock = stats.write().await;
                stats_lock.total_checks += 1;
                stats_lock.last_check_at = Some(Utc::now());
                stats_lock.next_check_at = Some(Utc::now() + Duration::seconds(interval.as_secs() as i64));
                drop(stats_lock);

                // Note: In real implementation, this would query from storage
                // For now, this is a placeholder for the forgetting logic
                debug!("Forgetting check completed");
            }

            info!("Forgetting scheduler stopped");
        });

        let mut task_handle = self.task_handle.write().await;
        *task_handle = Some(handle);

        Ok(())
    }

    /// Stop automatic forgetting scheduler
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if !*running {
            return Err(AgentMemError::other("Scheduler not running"));
        }

        *running = false;
        drop(running);

        // Wait for task to complete
        let mut task_handle = self.task_handle.write().await;
        if let Some(handle) = task_handle.take() {
            handle.await.ok();
        }

        info!("Forgetting scheduler stopped");
        Ok(())
    }

    /// Check if scheduler is running
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// Get statistics
    pub async fn stats(&self) -> ForgettingStats {
        self.stats.read().await.clone()
    }

    /// Manually trigger forgetting check
    ///
    /// # Parameters
    ///
    /// - `memories`: Memories to check
    ///
    /// # Returns
    ///
    /// List of memory IDs that were forgotten
    pub async fn check_forgetting(&self, memories: Vec<Memory>) -> Result<Vec<String>> {
        let mut forgotten = Vec::new();
        let now = Utc::now();
        let threshold = self.config.forgetting_threshold;
        let enable_events = self.config.enable_events;
        let event_bus = self.event_bus.clone();

        let mut stats = self.stats.write().await;

        for memory in memories.iter().take(self.config.max_memories_per_check) {
            stats.total_checked += 1;

            // Check protection
            let memory_id = memory.id();
            if self.protection.is_permanently_protected(memory_id).await {
                stats.total_protected += 1;
                continue;
            }

            // Calculate time elapsed
            let created_at = memory.created_at();
            let elapsed_days = (now - *created_at).num_days() as f64;

            // Apply protection multiplier
            let effective_time = self
                .protection
                .effective_forgetting_time(memory_id, elapsed_days)
                .await;

            // Check retention
            let retention = self.curve.retention(effective_time);

            if retention < threshold {
                // Check protection again (might be protected)
                if self.protection.is_protected(memory_id).await {
                    stats.total_protected += 1;
                    continue;
                }

                debug!(
                    "Forgetting memory {} with retention {:.2}",
                    memory_id, retention
                );

                forgotten.push(memory_id.clone());
                stats.total_forgotten += 1;

                // Publish event
                if enable_events {
                    if let Some(ref bus) = event_bus {
                        let event = agent_mem_event_bus::MemoryEvent::new(EventType::MemoryDeleted)
                            .with_memory_id(memory_id.clone())
                            .with_metadata("retention", serde_json::json!(retention))
                            .with_metadata("reason", serde_json::json!("forgetting"));
                        let _ = bus.publish(event).await;
                    }
                }
            }
        }

        Ok(forgotten)
    }

    /// Estimate when memory will be forgotten
    ///
    /// # Parameters
    ///
    /// - `memory_id`: Memory ID to check
    /// - `created_at`: Memory creation timestamp
    ///
    /// # Returns
    ///
    /// Estimated forgetting timestamp, or None if permanently protected
    pub async fn estimate_forgetting(
        &self,
        memory_id: &str,
        created_at: DateTime<Utc>,
    ) -> Option<DateTime<Utc>> {
        if self.protection.is_permanently_protected(memory_id).await {
            return None;
        }

        let protection_level = self.protection.get_protection(memory_id).await;
        let base_time = self.curve.time_to_threshold(self.config.forgetting_threshold);
        let protected_time = base_time * protection_level.multiplier();

        Some(created_at + Duration::days(protected_time as i64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DEFAULT_CHECK_INTERVAL_SECONDS;
    use crate::DEFAULT_FORGETTING_THRESHOLD;

    #[test]
    fn test_config_default() {
        let config = ForgettingConfig::default();
        assert_eq!(config.check_interval_seconds, DEFAULT_CHECK_INTERVAL_SECONDS);
        assert_eq!(config.forgetting_threshold, DEFAULT_FORGETTING_THRESHOLD);
        assert_eq!(config.default_strength, 7.0);
    }

    #[test]
    fn test_config_builder() {
        let config = ForgettingConfig::default()
            .with_check_interval(1800)
            .with_threshold(0.05)
            .with_strength(14.0)
            .with_events(false)
            .with_max_memories(500);

        assert_eq!(config.check_interval_seconds, 1800);
        assert_eq!(config.forgetting_threshold, 0.05);
        assert_eq!(config.default_strength, 14.0);
        assert_eq!(config.enable_events, false);
        assert_eq!(config.max_memories_per_check, 500);
    }

    #[test]
    #[should_panic(expected = "Threshold must be in (0, 1)")]
    fn test_config_invalid_threshold_high() {
        ForgettingConfig::default().with_threshold(1.0);
    }

    #[test]
    #[should_panic(expected = "Threshold must be in (0, 1)")]
    fn test_config_invalid_threshold_zero() {
        ForgettingConfig::default().with_threshold(0.0);
    }

    #[test]
    #[should_panic(expected = "Strength must be positive")]
    fn test_config_invalid_strength() {
        ForgettingConfig::default().with_strength(0.0);
    }

    #[tokio::test]
    async fn test_scheduler_creation() {
        let config = ForgettingConfig::default();
        let scheduler = ForgettingScheduler::new(config).await;
        assert!(scheduler.is_ok());
    }

    #[tokio::test]
    async fn test_protection_access() {
        let config = ForgettingConfig::default();
        let scheduler = ForgettingScheduler::new(config).await.unwrap();

        scheduler
            .protection()
            .set_protection("mem-1".to_string(), ProtectionLevel::High)
            .await;

        assert!(scheduler.protection().is_protected("mem-1").await);
    }

    #[tokio::test]
    async fn test_scheduler_stats() {
        let config = ForgettingConfig::default();
        let scheduler = ForgettingScheduler::new(config).await.unwrap();

        let stats = scheduler.stats().await;
        assert_eq!(stats.total_checks, 0);
        assert_eq!(stats.total_forgotten, 0);
    }

    #[tokio::test]
    async fn test_estimate_forgetting() {
        let config = ForgettingConfig::default().with_strength(1.0);
        let scheduler = ForgettingScheduler::new(config).await.unwrap();

        let created_at = Utc::now();
        let estimate = scheduler
            .estimate_forgetting("mem-1", created_at)
            .await;

        assert!(estimate.is_some());

        // With strength=1.0 and threshold=0.1, should forget in ~2.3 days
        let forgetting_time = estimate.unwrap();
        let days_until = (forgetting_time - created_at).num_days();
        assert!(days_until >= 2 && days_until <= 3);
    }

    #[tokio::test]
    async fn test_estimate_permanent_protection() {
        let config = ForgettingConfig::default();
        let scheduler = ForgettingScheduler::new(config).await.unwrap();

        scheduler
            .protection()
            .set_protection("mem-1".to_string(), ProtectionLevel::Critical)
            .await;

        let created_at = Utc::now();
        let estimate = scheduler
            .estimate_forgetting("mem-1", created_at)
            .await;

        assert!(estimate.is_none());
    }
}
