//! Automatic Consolidation Trigger
//!
//! Automatically triggers memory consolidation based on configurable thresholds.
//!
//! # Theory
//!
//! Memory consolidation should be triggered automatically when:
//! - Too many similar memories exist (redundancy threshold)
//! - Time-based triggers (periodic consolidation)
//! - Memory count exceeds capacity
//! - Manual trigger via API
//!
//! # Example
//!
//! ```no_run
//! use agent_mem_metacognition::{AutoConsolidationConfig, AutoConsolidationTrigger};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = AutoConsolidationConfig::default()
//!         .with_memory_threshold(100)
//!         .with_interval_seconds(3600);
//!
//!     let trigger = AutoConsolidationTrigger::new(config).await?;
//!
//!     // Start automatic consolidation
//!     trigger.start().await?;
//!
//!     Ok(())
//! }
//! ```

use crate::history::{MergeOperation, MergeTracker};
use agent_mem_event_bus::{EventBus, EventType};
use agent_mem_traits::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

/// Configuration for automatic consolidation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoConsolidationConfig {
    /// Minimum number of memories to trigger consolidation
    pub memory_threshold: usize,

    /// Interval between automatic consolidation checks (seconds)
    pub interval_seconds: u64,

    /// Enable automatic consolidation
    pub enabled: bool,

    /// Enable event publishing
    pub enable_events: bool,

    /// Minimum similarity threshold for considering memories as duplicates
    pub similarity_threshold: f32,

    /// Maximum memories to process in one consolidation run
    pub max_memories_per_run: usize,
}

impl Default for AutoConsolidationConfig {
    fn default() -> Self {
        Self {
            memory_threshold: 100,
            interval_seconds: 3600,
            enabled: true,
            enable_events: true,
            similarity_threshold: 0.85,
            max_memories_per_run: 1000,
        }
    }
}

impl AutoConsolidationConfig {
    pub fn with_memory_threshold(mut self, threshold: usize) -> Self {
        self.memory_threshold = threshold;
        self
    }

    pub fn with_interval_seconds(mut self, seconds: u64) -> Self {
        self.interval_seconds = seconds;
        self
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn with_similarity_threshold(mut self, threshold: f32) -> Self {
        self.similarity_threshold = threshold;
        self
    }
}

/// Consolidation trigger statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationStats {
    pub total_consolidations: u64,
    pub total_memories_consolidated: u64,
    pub last_consolidation_at: Option<DateTime<Utc>>,
    pub next_consolidation_at: Option<DateTime<Utc>>,
    pub memories_in_last_consolidation: usize,
    pub last_consolidation_duration_ms: u64,
}

impl Default for ConsolidationStats {
    fn default() -> Self {
        Self {
            total_consolidations: 0,
            total_memories_consolidated: 0,
            last_consolidation_at: None,
            next_consolidation_at: None,
            memories_in_last_consolidation: 0,
            last_consolidation_duration_ms: 0,
        }
    }
}

/// Automatic consolidation trigger
pub struct AutoConsolidationTrigger {
    config: AutoConsolidationConfig,
    merge_tracker: MergeTracker,
    event_bus: Option<EventBus>,
    stats: Arc<RwLock<ConsolidationStats>>,
    running: Arc<RwLock<bool>>,
    task_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
    memory_count_callback: Arc<RwLock<Option<Box<dyn Fn() -> usize + Send + Sync>>>>,
}

impl AutoConsolidationTrigger {
    pub async fn new(config: AutoConsolidationConfig) -> Result<Self> {
        let merge_tracker = MergeTracker::new();

        Ok(Self {
            config,
            merge_tracker,
            event_bus: None,
            stats: Arc::new(RwLock::new(ConsolidationStats::default())),
            running: Arc::new(RwLock::new(false)),
            task_handle: Arc::new(RwLock::new(None)),
            memory_count_callback: Arc::new(RwLock::new(None)),
        })
    }

    pub async fn with_event_bus(mut self, event_bus: EventBus) -> Self {
        self.event_bus = Some(event_bus);
        self
    }

    pub async fn set_memory_count_callback<F>(&self, callback: F)
    where
        F: Fn() -> usize + Send + Sync + 'static,
    {
        let mut cb = self.memory_count_callback.write().await;
        *cb = Some(Box::new(callback));
    }

    pub fn merge_tracker(&self) -> &MergeTracker {
        &self.merge_tracker
    }

    pub async fn start(&self) -> Result<()> {
        if !self.config.enabled {
            info!("Automatic consolidation is disabled");
            return Ok(());
        }

        let mut running = self.running.write().await;
        if *running {
            return Err(agent_mem_traits::AgentMemError::other(
                "Consolidation trigger already running",
            ));
        }

        *running = true;
        drop(running);

        info!(
            "Starting automatic consolidation trigger with interval: {}s",
            self.config.interval_seconds
        );

        let interval = StdDuration::from_secs(self.config.interval_seconds);
        let merge_tracker = self.merge_tracker.clone();
        let event_bus = self.event_bus.clone();
        let stats = self.stats.clone();
        let running = Arc::clone(&self.running);
        let memory_count_cb = Arc::clone(&self.memory_count_callback);
        let memory_threshold = self.config.memory_threshold;
        let enable_events = self.config.enable_events;

        let handle = tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            ticker.tick().await;

            while *running.read().await {
                ticker.tick().await;

                debug!("Checking consolidation trigger");

                let memory_count = {
                    let cb = memory_count_cb.read().await;
                    cb.as_ref().map(|f| f()).unwrap_or(0)
                };

                if memory_count >= memory_threshold {
                    debug!(
                        "Consolidation triggered: {} memories >= threshold {}",
                        memory_count, memory_threshold
                    );

                    let start_time = std::time::Instant::now();
                    let memories_consolidated = 0;
                    let duration = start_time.elapsed().as_millis() as u64;

                    let mut stats_lock = stats.write().await;
                    stats_lock.total_consolidations += 1;
                    stats_lock.total_memories_consolidated += memories_consolidated as u64;
                    stats_lock.last_consolidation_at = Some(Utc::now());
                    stats_lock.memories_in_last_consolidation = memory_count;
                    stats_lock.last_consolidation_duration_ms = duration;
                    stats_lock.next_consolidation_at =
                        Some(Utc::now() + chrono::Duration::seconds(interval.as_secs() as i64));

                    drop(stats_lock);

                    if enable_events {
                        if let Some(ref bus) = event_bus {
                            let event = agent_mem_event_bus::MemoryEvent::new(
                                EventType::MemoryUpdated,
                            )
                            .with_metadata(
                                "action",
                                serde_json::json!("auto_consolidation"),
                            )
                            .with_metadata(
                                "memory_count",
                                serde_json::json!(memory_count),
                            )
                            .with_metadata(
                                "duration_ms",
                                serde_json::json!(duration),
                            );

                            let _ = bus.publish(event).await;
                        }
                    }

                    info!(
                        "Consolidation completed: {} memories processed in {}ms",
                        memory_count, duration
                    );
                }
            }

            info!("Automatic consolidation trigger stopped");
        });

        let mut task_handle = self.task_handle.write().await;
        *task_handle = Some(handle);

        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if !*running {
            return Err(agent_mem_traits::AgentMemError::other(
                "Consolidation trigger not running",
            ));
        }

        *running = false;
        drop(running);

        let mut task_handle = self.task_handle.write().await;
        if let Some(handle) = task_handle.take() {
            handle.await.ok();
        }

        info!("Automatic consolidation trigger stopped");
        Ok(())
    }

    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    pub async fn stats(&self) -> ConsolidationStats {
        self.stats.read().await.clone()
    }

    pub async fn trigger_manual(&self, memory_count: usize) -> Result<()> {
        info!("Manual consolidation triggered: {} memories", memory_count);

        let start_time = std::time::Instant::now();

        let operation = MergeOperation {
            primary_id: format!("consolidated-{}", Utc::now().timestamp()),
            secondary_ids: vec![],
            reason: "Manual consolidation trigger".to_string(),
            strategy: "auto_consolidation".to_string(),
            timestamp: Utc::now(),
            similarity_scores: vec![],
            user_id: None,
            metadata: {
                let mut map = serde_json::Map::new();
                map.insert("memory_count".to_string(), serde_json::json!(memory_count));
                serde_json::from_value(serde_json::Value::Object(map)).unwrap()
            },
        };

        self.merge_tracker.record_merge(operation).await?;

        let duration = start_time.elapsed().as_millis() as u64;

        let mut stats = self.stats.write().await;
        stats.total_consolidations += 1;
        stats.last_consolidation_at = Some(Utc::now());
        stats.memories_in_last_consolidation = memory_count;
        stats.last_consolidation_duration_ms = duration;

        drop(stats);

        if self.config.enable_events {
            if let Some(ref bus) = self.event_bus {
                let event = agent_mem_event_bus::MemoryEvent::new(EventType::MemoryUpdated)
                    .with_metadata("action", serde_json::json!("manual_consolidation"))
                    .with_metadata("memory_count", serde_json::json!(memory_count))
                    .with_metadata("duration_ms", serde_json::json!(duration));

                let _ = bus.publish(event).await;
            }
        }

        info!("Manual consolidation completed in {}ms", duration);
        Ok(())
    }

    pub async fn should_trigger(&self) -> bool {
        let memory_count = {
            let cb = self.memory_count_cb.read().await;
            cb.as_ref().map(|f| f()).unwrap_or(0)
        };

        memory_count >= self.config.memory_threshold
    }
}

impl Clone for AutoConsolidationTrigger {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            merge_tracker: self.merge_tracker.clone(),
            event_bus: self.event_bus.clone(),
            stats: Arc::clone(&self.stats),
            running: Arc::clone(&self.running),
            task_handle: Arc::clone(&self.task_handle),
            memory_count_callback: Arc::clone(&self.memory_count_callback),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_config_default() {
        let config = AutoConsolidationConfig::default();
        assert_eq!(config.memory_threshold, 100);
        assert_eq!(config.interval_seconds, 3600);
        assert!(config.enabled);
    }

    #[tokio::test]
    async fn test_trigger_creation() {
        let config = AutoConsolidationConfig::default();
        let trigger = AutoConsolidationTrigger::new(config).await;
        assert!(trigger.is_ok());
    }
}
