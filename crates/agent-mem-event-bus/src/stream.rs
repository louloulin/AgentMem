//! Event stream implementation for receiving events

use super::Result;
use agent_mem_performance::telemetry::{MemoryEvent, EventType};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::debug;

use super::bus::EventBusStats;

/// Event stream for receiving events from the bus
pub struct EventStream {
    /// Broadcast receiver
    rx: broadcast::Receiver<MemoryEvent>,

    /// Event filter (optional)
    filter: Option<EventType>,

    /// Statistics reference
    stats: Arc<RwLock<EventBusStats>>,
}

impl EventStream {
    /// Create a new event stream
    pub(crate) fn new(rx: broadcast::Receiver<MemoryEvent>, stats: Arc<RwLock<EventBusStats>>) -> Self {
        Self {
            rx,
            filter: None,
            stats,
        }
    }

    /// Create a new filtered event stream
    pub(crate) fn with_filter(
        rx: broadcast::Receiver<MemoryEvent>,
        stats: Arc<RwLock<EventBusStats>>,
        filter: EventType,
    ) -> Self {
        Self {
            rx,
            filter: Some(filter),
            stats,
        }
    }

    /// Receive the next event
    ///
    /// This will wait until an event is available or the bus is closed.
    /// Returns None if the bus is closed.
    pub async fn recv(&mut self) -> Option<MemoryEvent> {
        loop {
            match self.rx.recv().await {
                Ok(event) => {
                    // Apply filter if set
                    if let Some(ref filter) = self.filter {
                        if event.event_type != *filter {
                            continue; // Skip non-matching events
                        }
                    }

                    // Update stats
                    let mut stats = self.stats.write().await;
                    stats.events_received += 1;

                    debug!("Received event: {:?}", event.event_type);
                    return Some(event);
                }
                Err(broadcast::error::RecvError::Lagged(count)) => {
                    debug!("Event stream lagged, skipped {} messages", count);
                    continue;
                }
                Err(broadcast::error::RecvError::Closed) => {
                    debug!("Event bus closed");
                    return None;
                }
            }
        }
    }

    /// Try to receive an event without waiting
    ///
    /// Returns immediately with either an event or None if no event is available
    pub fn try_recv(&mut self) -> Option<MemoryEvent> {
        loop {
            match self.rx.try_recv() {
                Ok(event) => {
                    // Apply filter if set
                    if let Some(ref filter) = self.filter {
                        if event.event_type != *filter {
                            continue; // Skip non-matching events
                        }
                    }
                    return Some(event);
                }
                Err(broadcast::error::TryRecvError::Empty) => return None,
                Err(broadcast::error::TryRecvError::Lagged(count)) => {
                    debug!("Event stream lagged, skipped {} messages", count);
                    continue;
                }
                Err(broadcast::error::TryRecvError::Closed) => return None,
            }
        }
    }

    /// Receive an event with timeout
    ///
    /// Returns None if no event is received within the timeout
    pub async fn recv_timeout(&mut self, timeout: Duration) -> Option<MemoryEvent> {
        match tokio::time::timeout(timeout, self.recv()).await {
            Ok(event) => event,
            Err(_) => None,
        }
    }

    /// Receive multiple events at once
    ///
    /// Returns up to `max_events` events that are immediately available
    pub fn recv_batch(&mut self, max_events: usize) -> Vec<MemoryEvent> {
        let mut events = Vec::new();

        while events.len() < max_events {
            match self.try_recv() {
                Some(event) => events.push(event),
                None => break,
            }
        }

        events
    }

    /// Create a stream using async-stream
    ///
    /// This allows using the event stream with StreamExt
    #[cfg(feature = "stream")]
    pub fn into_stream(self) -> impl futures::Stream<Item = MemoryEvent> {
        use futures::stream::{self, StreamExt};
        stream::unfold(self, |mut rx| async move {
            let event = rx.recv().await;
            event.map(|e| (e, rx))
        })
    }

    /// Set event filter
    pub fn set_filter(&mut self, filter: EventType) {
        self.filter = Some(filter);
    }

    /// Clear event filter
    pub fn clear_filter(&mut self) {
        self.filter = None;
    }

    /// Get the current filter
    pub fn filter(&self) -> Option<&EventType> {
        self.filter.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EventBus;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_event_stream_recv() {
        let bus = EventBus::new(100);
        let mut stream = bus.subscribe().await;

        // Publish an event
        let event = MemoryEvent::new(EventType::MemoryCreated);
        bus.publish(event).await.unwrap();

        // Receive it
        let received = stream.recv().await;
        assert!(received.is_some());
        assert_eq!(received.unwrap().event_type, EventType::MemoryCreated);
    }

    #[tokio::test]
    async fn test_event_stream_try_recv() {
        let bus = EventBus::new(100);
        let mut stream = bus.subscribe().await;

        // No event available
        let result = stream.try_recv();
        assert!(result.is_none());

        // Publish an event
        let event = MemoryEvent::new(EventType::MemoryCreated);
        bus.publish(event).await.unwrap();

        // Try recv should succeed
        let result = stream.try_recv();
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_event_stream_timeout() {
        let bus = EventBus::new(100);
        let mut stream = bus.subscribe().await;

        // Timeout with no event
        let result = stream.recv_timeout(Duration::from_millis(100)).await;
        assert!(result.is_none());

        // Publish an event
        tokio::spawn(async move {
            sleep(Duration::from_millis(50)).await;
            let event = MemoryEvent::new(EventType::MemoryCreated);
            bus.publish(event).await.unwrap();
        });

        // Should receive within timeout
        let result = stream.recv_timeout(Duration::from_millis(200)).await;
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_event_stream_batch() {
        let bus = EventBus::new(100);
        let mut stream = bus.subscribe().await;

        // Publish multiple events
        for _ in 0..5 {
            let event = MemoryEvent::new(EventType::MemoryCreated);
            bus.publish(event).await.unwrap();
        }

        // Receive batch
        let events = stream.recv_batch(3);
        assert_eq!(events.len(), 3);

        // Receive remaining
        let events = stream.recv_batch(10);
        assert_eq!(events.len(), 2);
    }

    #[tokio::test]
    async fn test_event_stream_filter() {
        let bus = EventBus::new(100);
        let mut stream = bus.subscribe_filtered(EventType::MemoryCreated).await;

        // Publish different event types
        let event1 = MemoryEvent::new(EventType::MemoryCreated);
        let event2 = MemoryEvent::new(EventType::MemoryUpdated);
        let event3 = MemoryEvent::new(EventType::MemoryCreated);

        bus.publish(event1).await.unwrap();
        bus.publish(event2).await.unwrap();
        bus.publish(event3).await.unwrap();

        // Should only receive MemoryCreated events
        let recv1 = stream.recv().await.unwrap();
        assert_eq!(recv1.event_type, EventType::MemoryCreated);

        let recv2 = stream.recv().await.unwrap();
        assert_eq!(recv2.event_type, EventType::MemoryCreated);
    }
}
