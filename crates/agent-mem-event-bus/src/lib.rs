//! AgentMem Event Bus
//!
//! Pub/Sub event system for AgentMem using tokio::sync::broadcast.
//!
//! # Features
//!
//! - Async event publishing and subscription
//! - Event filtering by type
//! - Multiple subscribers support
//! - Event history tracking
//! - Graceful shutdown
//!
//! # Example
//!
//! ```no_run
//! use agent_mem_event_bus::{EventBus, EventHandler};
//! use agent_mem_performance::telemetry::{MemoryEvent, EventType};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create event bus
//!     let bus = EventBus::new(1000);
//!
//!     // Subscribe to events
//!     let mut subscriber = bus.subscribe().await;
//!
//!     // Handle events
//!     tokio::spawn(async move {
//!         while let Some(event) = subscriber.recv().await {
//!             println!("Received event: {:?}", event.event_type);
//!         }
//!     });
//!
//!     // Publish events
//!     let event = MemoryEvent::new(EventType::MemoryCreated)
//!         .with_memory_id("mem-123".to_string());
//!     bus.publish(event).await?;
//!
//!     Ok(())
//! }
//! ```

pub mod bus;
pub mod handler;
pub mod stream;

pub use bus::EventBus;
pub use handler::{EventHandler, EventFilter};
pub use stream::EventStream;

// Re-exports from agent-mem-performance
pub use agent_mem_performance::telemetry::{MemoryEvent, EventType};

use agent_mem_traits::Result;

/// Event bus configuration
#[derive(Debug, Clone)]
pub struct EventBusConfig {
    /// Channel capacity (number of events buffered)
    pub channel_capacity: usize,

    /// Enable event history
    pub enable_history: bool,

    /// Maximum history size
    pub max_history_size: usize,

    /// Enable event filtering
    pub enable_filtering: bool,
}

impl Default for EventBusConfig {
    fn default() -> Self {
        Self {
            channel_capacity: 1000,
            enable_history: true,
            max_history_size: 10000,
            enable_filtering: true,
        }
    }
}

impl EventBusConfig {
    /// Create a new configuration with custom capacity
    pub fn with_capacity(mut self, capacity: usize) -> Self {
        self.channel_capacity = capacity;
        self
    }

    /// Enable event history with custom size
    pub fn with_history(mut self, max_size: usize) -> Self {
        self.enable_history = true;
        self.max_history_size = max_size;
        self
    }

    /// Disable event history
    pub fn without_history(mut self) -> Self {
        self.enable_history = false;
        self
    }

    /// Enable event filtering
    pub fn with_filtering(mut self) -> Self {
        self.enable_filtering = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn test_config_default() {
        let config = EventBusConfig::default();
        assert_eq!(config.channel_capacity, 1000);
        assert_eq!(config.max_history_size, 10000);
        assert!(config.enable_history);
        assert!(config.enable_filtering);
    }

    #[tokio::test]
    async fn test_config_builder() {
        let config = EventBusConfig::default()
            .with_capacity(500)
            .with_history(5000)
            .with_filtering();

        assert_eq!(config.channel_capacity, 500);
        assert_eq!(config.max_history_size, 5000);
        assert!(config.enable_history);
        assert!(config.enable_filtering);
    }

    #[tokio::test]
    async fn test_config_without_history() {
        let config = EventBusConfig::default().without_history();
        assert!(!config.enable_history);
    }
}
