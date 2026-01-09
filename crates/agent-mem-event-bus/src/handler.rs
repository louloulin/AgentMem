//! Event handler trait and implementations

use super::Result;
use agent_mem_performance::telemetry::{MemoryEvent, EventType};
use async_trait::async_trait;

/// Event handler trait for processing events
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Handle an event
    async fn handle(&self, event: &MemoryEvent) -> Result<()>;

    /// Get the event filter (None means handle all events)
    fn filter(&self) -> Option<EventType> {
        None
    }
}

/// Event filter for subscribing to specific event types
#[derive(Debug, Clone, PartialEq)]
pub enum EventFilter {
    /// Handle all events
    All,

    /// Handle specific event type
    Type(EventType),

    /// Handle multiple event types
    Types(Vec<EventType>),

    /// Custom filter function
    Custom(Box<dyn Fn(&MemoryEvent) -> bool + Send + Sync>),
}

impl EventFilter {
    /// Check if an event matches the filter
    pub fn matches(&self, event: &MemoryEvent) -> bool {
        match self {
            EventFilter::All => true,
            EventFilter::Type(event_type) => &event.event_type == event_type,
            EventFilter::Types(types) => types.contains(&event.event_type),
            EventFilter::Custom(f) => f(event),
        }
    }
}

/// Closure-based event handler
pub struct ClosureHandler<F>
where
    F: Fn(&MemoryEvent) -> Result<()> + Send + Sync,
{
    handler: F,
    filter: Option<EventType>,
}

impl<F> ClosureHandler<F>
where
    F: Fn(&MemoryEvent) -> Result<()> + Send + Sync,
{
    /// Create a new closure handler
    pub fn new(handler: F) -> Self {
        Self {
            handler,
            filter: None,
        }
    }

    /// Set the event filter
    pub fn with_filter(mut self, event_type: EventType) -> Self {
        self.filter = Some(event_type);
        self
    }
}

#[async_trait]
impl<F> EventHandler for ClosureHandler<F>
where
    F: Fn(&MemoryEvent) -> Result<()> + Send + Sync,
{
    async fn handle(&self, event: &MemoryEvent) -> Result<()> {
        // Call the closure
        (self.handler)(event)
    }

    fn filter(&self) -> Option<EventType> {
        self.filter.clone()
    }
}

/// Logging event handler - logs all events
pub struct LoggingHandler {
    filter: Option<EventType>,
}

impl LoggingHandler {
    /// Create a new logging handler
    pub fn new() -> Self {
        Self { filter: None }
    }

    /// Create a new logging handler with filter
    pub fn with_filter(event_type: EventType) -> Self {
        Self {
            filter: Some(event_type),
        }
    }
}

impl Default for LoggingHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventHandler for LoggingHandler {
    async fn handle(&self, event: &MemoryEvent) -> Result<()> {
        tracing::info!(
            "Event: {:?}, Memory: {:?}, User: {:?}, Success: {}",
            event.event_type,
            event.memory_id,
            event.user_id,
            event.success
        );
        Ok(())
    }

    fn filter(&self) -> Option<EventType> {
        self.filter.clone()
    }
}

/// Metrics event handler - tracks event statistics
#[cfg(feature = "metrics")]
pub struct MetricsHandler {
    filter: Option<EventType>,
}

#[cfg(feature = "metrics")]
impl MetricsHandler {
    /// Create a new metrics handler
    pub fn new() -> Self {
        Self { filter: None }
    }

    /// Create a new metrics handler with filter
    pub fn with_filter(event_type: EventType) -> Self {
        Self {
            filter: Some(event_type),
        }
    }
}

#[cfg(feature = "metrics")]
impl Default for MetricsHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "metrics")]
#[async_trait]
impl EventHandler for MetricsHandler {
    async fn handle(&self, event: &MemoryEvent) -> Result<()> {
        // Update metrics using counters
        tracing::debug!(
            "Metrics: event_type={:?}, success={}",
            event.event_type,
            event.success
        );

        if let Some(duration) = event.duration {
            tracing::debug!("Event duration: {:?}", duration);
        }

        Ok(())
    }

    fn filter(&self) -> Option<EventType> {
        self.filter.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_filter_all() {
        let filter = EventFilter::All;
        let event = MemoryEvent::new(EventType::MemoryCreated);
        assert!(filter.matches(&event));
    }

    #[test]
    fn test_event_filter_type() {
        let filter = EventFilter::Type(EventType::MemoryCreated);

        let event1 = MemoryEvent::new(EventType::MemoryCreated);
        assert!(filter.matches(&event1));

        let event2 = MemoryEvent::new(EventType::MemoryUpdated);
        assert!(!filter.matches(&event2));
    }

    #[test]
    fn test_event_filter_types() {
        let filter = EventFilter::Types(vec![
            EventType::MemoryCreated,
            EventType::MemoryUpdated,
        ]);

        let event1 = MemoryEvent::new(EventType::MemoryCreated);
        assert!(filter.matches(&event1));

        let event2 = MemoryEvent::new(EventType::MemoryUpdated);
        assert!(filter.matches(&event2));

        let event3 = MemoryEvent::new(EventType::MemoryDeleted);
        assert!(!filter.matches(&event3));
    }

    #[test]
    fn test_event_filter_custom() {
        let filter = EventFilter::Custom(Box::new(|event| {
            matches!(event.event_type, EventType::MemoryCreated | EventType::MemoryUpdated)
        }));

        let event1 = MemoryEvent::new(EventType::MemoryCreated);
        assert!(filter.matches(&event1));

        let event2 = MemoryEvent::new(EventType::MemoryDeleted);
        assert!(!filter.matches(&event2));
    }

    #[tokio::test]
    async fn test_closure_handler() {
        let called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let called_clone = called.clone();

        let handler = ClosureHandler::new(move |_event| {
            called_clone.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        });

        let event = MemoryEvent::new(EventType::MemoryCreated);
        handler.handle(&event).await.unwrap();

        assert!(called.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_logging_handler() {
        let handler = LoggingHandler::new();
        let event = MemoryEvent::new(EventType::MemoryCreated);
        let result = handler.handle(&event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handler_filter() {
        let handler = LoggingHandler::with_filter(EventType::MemoryCreated);

        assert_eq!(handler.filter(), Some(EventType::MemoryCreated));

        let handler_no_filter = LoggingHandler::new();
        assert_eq!(handler_no_filter.filter(), None);
    }
}
