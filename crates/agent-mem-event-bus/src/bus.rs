//! Event bus implementation using tokio::sync::broadcast

use super::{EventBusConfig, Result};
use agent_mem_performance::telemetry::{MemoryEvent, EventType};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};

use crate::stream::EventStream;

/// Event bus for pub/sub messaging
pub struct EventBus {
    /// Broadcast channel for events
    tx: broadcast::Sender<MemoryEvent>,

    /// Event history (optional)
    history: Arc<RwLock<Vec<MemoryEvent>>>,

    /// Configuration
    config: EventBusConfig,

    /// Statistics
    stats: Arc<RwLock<EventBusStats>>,

    /// Start time
    start_time: Instant,
}

/// Event bus statistics
#[derive(Debug, Clone, Default)]
pub struct EventBusStats {
    /// Total events published
    pub events_published: u64,

    /// Total events received by subscribers
    pub events_received: u64,

    /// Total errors
    pub errors: u64,

    /// Current subscriber count
    pub subscriber_count: u64,

    /// Last publish time
    pub last_publish_at: Option<Instant>,
}

impl EventBus {
    /// Create a new event bus with default configuration
    pub fn new(capacity: usize) -> Self {
        Self::with_config(EventBusConfig {
            channel_capacity: capacity,
            ..Default::default()
        })
    }

    /// Create a new event bus with custom configuration
    pub fn with_config(config: EventBusConfig) -> Self {
        let (tx, _) = broadcast::channel(config.channel_capacity);

        info!(
            "EventBus created with capacity={}, history={}",
            config.channel_capacity, config.enable_history
        );

        Self {
            tx,
            history: Arc::new(RwLock::new(Vec::new())),
            config,
            stats: Arc::new(RwLock::new(EventBusStats::default())),
            start_time: Instant::now(),
        }
    }

    /// Publish an event to all subscribers
    ///
    /// # Errors
    ///
    /// Returns an error if there are no subscribers
    pub async fn publish(&self, event: MemoryEvent) -> Result<()> {
        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.events_published += 1;
            stats.last_publish_at = Some(Instant::now());
        }

        // Add to history if enabled
        if self.config.enable_history {
            let mut history = self.history.write().await;
            history.push(event.clone());

            // Trim history if needed
            if history.len() > self.config.max_history_size {
                let remove_count = history.len() - self.config.max_history_size;
                history.drain(0..remove_count);
                debug!("Trimmed {} events from history", remove_count);
            }
        }

        // Publish to all subscribers
        match self.tx.send(event.clone()) {
            Ok(receiver_count) => {
                debug!(
                    "Event published to {} subscribers: {:?}",
                    receiver_count,
                    event.event_type
                );
                Ok(())
            }
            Err(e) => {
                // No subscribers
                warn!("Failed to publish event (no receivers): {:?}", e.0);
                Err(agent_mem_traits::AgentMemError::other(
                    anyhow::anyhow!("No subscribers for event"),
                ))
            }
        }
    }

    /// Subscribe to events
    ///
    /// Returns a new event stream for receiving events
    pub async fn subscribe(&self) -> EventStream {
        let rx = self.tx.subscribe();

        // Update subscriber count
        {
            let mut stats = self.stats.write().await;
            stats.subscriber_count += 1;
        }

        info!("New subscriber added, total subscribers: {}", self.tx.receiver_count());

        EventStream::new(rx, self.stats.clone())
    }

    /// Subscribe to events with filtering
    ///
    /// Returns a filtered event stream that only receives matching events
    pub async fn subscribe_filtered(&self, filter: EventType) -> EventStream {
        let rx = self.tx.subscribe();

        // Update subscriber count
        {
            let mut stats = self.stats.write().await;
            stats.subscriber_count += 1;
        }

        info!(
            "New filtered subscriber added for {:?}, total subscribers: {}",
            filter,
            self.tx.receiver_count()
        );

        EventStream::with_filter(rx, self.stats.clone(), filter)
    }

    /// Get event history
    pub async fn get_history(&self) -> Vec<MemoryEvent> {
        if self.config.enable_history {
            self.history.read().await.clone()
        } else {
            Vec::new()
        }
    }

    /// Get events from history by type
    pub async fn get_history_by_type(&self, event_type: EventType) -> Vec<MemoryEvent> {
        if !self.config.enable_history {
            return Vec::new();
        }

        let history = self.history.read().await;
        history
            .iter()
            .filter(|e| e.event_type == event_type)
            .cloned()
            .collect()
    }

    /// Get event history in a time range
    pub async fn get_history_by_time_range(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Vec<MemoryEvent> {
        if !self.config.enable_history {
            return Vec::new();
        }

        let history = self.history.read().await;
        history
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .cloned()
            .collect()
    }

    /// Clear event history
    pub async fn clear_history(&self) {
        if self.config.enable_history {
            let mut history = self.history.write().await;
            let count = history.len();
            history.clear();
            info!("Cleared {} events from history", count);
        }
    }

    /// Get current statistics
    pub async fn get_stats(&self) -> EventBusStats {
        let mut stats = self.stats.read().await.clone();
        stats.subscriber_count = self.tx.receiver_count() as u64;
        stats
    }

    /// Get the number of active subscribers
    pub fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }

    /// Get the uptime of the event bus
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Shutdown the event bus gracefully
    pub async fn shutdown(&self) {
        info!("Shutting down EventBus...");

        // Wait for all subscribers to be dropped
        let timeout_duration = Duration::from_secs(5);
        let start = Instant::now();

        while self.tx.receiver_count() > 0 && start.elapsed() < timeout_duration {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        let remaining = self.tx.receiver_count();
        if remaining > 0 {
            warn!(
                "EventBus shutdown with {} remaining subscribers",
                remaining
            );
        } else {
            info!("EventBus shutdown gracefully");
        }
    }
}

impl Clone for EventBus {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            history: self.history.clone(),
            config: self.config.clone(),
            stats: self.stats.clone(),
            start_time: self.start_time,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_mem_performance::telemetry::EventType;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_event_bus_creation() {
        let bus = EventBus::new(100);
        assert_eq!(bus.subscriber_count(), 0);
    }

    #[tokio::test]
    async fn test_event_bus_with_config() {
        let config = EventBusConfig::default()
            .with_capacity(500)
            .with_history(1000);

        let bus = EventBus::with_config(config);
        assert_eq!(bus.subscriber_count(), 0);
    }

    #[tokio::test]
    async fn test_publish_no_subscribers() {
        let bus = EventBus::new(100);
        let event = MemoryEvent::new(EventType::MemoryCreated);

        // Should return error when no subscribers
        let result = bus.publish(event).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_publish_with_subscriber() {
        let bus = EventBus::new(100);
        let mut subscriber = bus.subscribe().await;

        let event = MemoryEvent::new(EventType::MemoryCreated)
            .with_memory_id("test-123".to_string());

        // Publish should succeed
        let result = bus.publish(event.clone()).await;
        assert!(result.is_ok());

        // Subscriber should receive the event
        let received = timeout(Duration::from_millis(100), subscriber.recv())
            .await
            .expect("Timeout waiting for event")
            .expect("No event received");

        assert_eq!(received.event_type, EventType::MemoryCreated);
        assert_eq!(received.memory_id, Some("test-123".to_string()));
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let bus = EventBus::new(100);
        let mut sub1 = bus.subscribe().await;
        let mut sub2 = bus.subscribe().await;

        let event = MemoryEvent::new(EventType::MemoryUpdated);

        bus.publish(event).await.unwrap();

        // Both subscribers should receive the event
        let recv1 = timeout(Duration::from_millis(100), sub1.recv())
            .await
            .unwrap()
            .unwrap();
        let recv2 = timeout(Duration::from_millis(100), sub2.recv())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(recv1.event_type, EventType::MemoryUpdated);
        assert_eq!(recv2.event_type, EventType::MemoryUpdated);
    }

    #[tokio::test]
    async fn test_event_history() {
        let bus = EventBus::new(100);

        // Publish some events
        for i in 0..5 {
            let event = MemoryEvent::new(EventType::MemoryCreated)
                .with_memory_id(format!("mem-{}", i));
            // Create a subscriber first
            if i == 0 {
                let _ = bus.subscribe().await;
            }
            bus.publish(event).await.unwrap();
        }

        // Get history
        let history = bus.get_history().await;
        assert_eq!(history.len(), 5);

        // Get by type
        let created_events = bus.get_history_by_type(EventType::MemoryCreated).await;
        assert_eq!(created_events.len(), 5);
    }

    #[tokio::test]
    async fn test_event_stats() {
        let bus = EventBus::new(100);
        let _subscriber = bus.subscribe().await;

        // Publish some events
        for _ in 0..3 {
            let event = MemoryEvent::new(EventType::MemoryCreated);
            bus.publish(event).await.unwrap();
        }

        // Get stats
        let stats = bus.get_stats().await;
        assert_eq!(stats.events_published, 3);
        assert_eq!(stats.subscriber_count, 1);
    }

    #[tokio::test]
    async fn test_clear_history() {
        let bus = EventBus::new(100);
        let _subscriber = bus.subscribe().await;

        // Publish some events
        for _ in 0..5 {
            let event = MemoryEvent::new(EventType::MemoryCreated);
            bus.publish(event).await.unwrap();
        }

        // Clear history
        bus.clear_history().await;

        let history = bus.get_history().await;
        assert_eq!(history.len(), 0);
    }
}
