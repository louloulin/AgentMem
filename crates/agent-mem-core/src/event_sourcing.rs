//! Event Sourcing Module for AgentMem
//!
//! This module provides event sourcing capabilities for memory audit trails,
//! state reconstruction, and historical tracking.
//!
//! # Features
//!
//! - Memory event tracking (Created, Updated, Deleted, Accessed, Promoted, Merged)
//! - Event replay for state reconstruction
//! - Snapshot mechanism for performance optimization
//! - Event filtering and querying
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    EventStore                              │
//! │  ┌─────────────────────────────────────────────────┐   │
//! │  │  Events: Vec<MemoryEvent>                       │   │
//! │  │  Snapshots: HashMap<MemoryId, Vec<u8>>        │   │
//! │  └─────────────────────────────────────────────────┘   │
//! └─────────────────────────────────────────────────────────────┘
//!                           │
//!                           ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                 Event Operations                           │
//! │  append() │ replay() │ rebuild() │ snapshot()             │
//! └─────────────────────────────────────────────────────────────┘
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Memory event types for event sourcing
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", content = "payload")]
pub enum MemoryEvent {
    /// Memory was created
    Created {
        /// Memory ID
        memory_id: String,
        /// Memory content
        content: String,
        /// Memory type
        memory_type: String,
        /// Importance score
        importance: f32,
        /// Timestamp
        timestamp: DateTime<Utc>,
        /// Additional metadata
        metadata: HashMap<String, serde_json::Value>,
    },
    /// Memory was updated
    Updated {
        /// Memory ID
        memory_id: String,
        /// Old content
        old_content: String,
        /// New content
        new_content: String,
        /// Version number
        version: u64,
        /// Timestamp
        timestamp: DateTime<Utc>,
        /// Update reason
        reason: Option<String>,
    },
    /// Memory was deleted
    Deleted {
        /// Memory ID
        memory_id: String,
        /// Timestamp
        timestamp: DateTime<Utc>,
        /// Deletion reason
        reason: Option<String>,
    },
    /// Memory was accessed
    Accessed {
        /// Memory ID
        memory_id: String,
        /// Access type (read, search, etc.)
        access_type: String,
        /// Timestamp
        timestamp: DateTime<Utc>,
    },
    /// Memory was promoted to different level
    Promoted {
        /// Memory ID
        memory_id: String,
        /// Source level
        from_level: String,
        /// Target level
        to_level: String,
        /// Timestamp
        timestamp: DateTime<Utc>,
        /// Promotion reason
        reason: Option<String>,
    },
    /// Memories were merged
    Merged {
        /// Source memory IDs
        source_ids: Vec<String>,
        /// Target memory ID
        target_id: String,
        /// Timestamp
        timestamp: DateTime<Utc>,
    },
}

/// Event store errors
#[derive(Debug, Error)]
pub enum EventStoreError {
    /// Event not found
    #[error("Event not found for memory: {0}")]
    EventNotFound(String),

    /// Invalid event sequence
    #[error("Invalid event sequence: {0}")]
    InvalidSequence(String),

    /// Storage error
    #[error("Storage error: {0}")]
    StorageError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Rebuild error
    #[error("Failed to rebuild state: {0}")]
    RebuildError(String),
}

/// Result type for event store operations
pub type EventStoreResult<T> = Result<T, EventStoreError>;

/// Rebuilt memory state from events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebuiltMemory {
    /// Memory ID
    pub id: String,
    /// Memory content
    pub content: String,
    /// Memory type
    pub memory_type: String,
    /// Importance score
    pub importance: f32,
    /// Version
    pub version: u64,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Last modified
    pub last_modified: DateTime<Utc>,
    /// Event count
    pub event_count: usize,
}

impl RebuiltMemory {
    /// Create a new rebuilt memory from initial creation event
    fn from_created(event: &MemoryEvent) -> Option<Self> {
        match event {
            MemoryEvent::Created {
                memory_id,
                content,
                memory_type,
                importance,
                timestamp,
                ..
            } => Some(Self {
                id: memory_id.clone(),
                content: content.clone(),
                memory_type: memory_type.clone(),
                importance: *importance,
                version: 1,
                created_at: *timestamp,
                last_modified: *timestamp,
                event_count: 1,
            }),
            _ => None,
        }
    }

    /// Apply an update event to the current state
    fn apply_update(&mut self, event: &MemoryEvent) -> EventStoreResult<()> {
        match event {
            MemoryEvent::Updated {
                new_content,
                version,
                timestamp,
                ..
            } => {
                self.content = new_content.clone();
                self.version = *version;
                self.last_modified = *timestamp;
                self.event_count += 1;
                Ok(())
            }
            _ => Err(EventStoreError::InvalidSequence(
                "Expected Updated event".to_string(),
            )),
        }
    }
}

/// Event store for storing and replaying memory events
#[derive(Debug, Clone, Default)]
pub struct EventStore {
    /// Events by memory ID
    events: HashMap<String, Vec<MemoryEvent>>,
    /// Snapshots by memory ID
    snapshots: HashMap<String, Snapshot>,
    /// Event count statistics
    stats: EventStoreStats,
}

/// Event store statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventStoreStats {
    /// Total events stored
    pub total_events: usize,
    /// Total memories tracked
    pub total_memories: usize,
    /// Total snapshots
    pub total_snapshots: usize,
}

/// Snapshot for faster state reconstruction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// Memory ID
    pub memory_id: String,
    /// Rebuilt memory state
    pub state: RebuiltMemory,
    /// Events since snapshot
    pub events_since_snapshot: usize,
    /// Snapshot timestamp
    pub timestamp: DateTime<Utc>,
}

impl EventStore {
    /// Create a new event store
    pub fn new() -> Self {
        Self {
            events: HashMap::new(),
            snapshots: HashMap::new(),
            stats: EventStoreStats::default(),
        }
    }

    /// Create event store with capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            events: HashMap::with_capacity(capacity),
            snapshots: HashMap::with_capacity(capacity / 10),
            stats: EventStoreStats::default(),
        }
    }

    /// Append an event to the store
    pub async fn append(&mut self, memory_id: &str, event: MemoryEvent) -> EventStoreResult<()> {
        let events = self.events.entry(memory_id.to_string()).or_insert_with(Vec::new);
        events.push(event);
        self.stats.total_events += 1;
        self.stats.total_memories = self.events.len();
        Ok(())
    }

    /// Append multiple events
    pub async fn append_many(
        &mut self,
        events: Vec<(String, MemoryEvent)>,
    ) -> EventStoreResult<()> {
        for (memory_id, event) in events {
            self.append(&memory_id, event).await?;
        }
        Ok(())
    }

    /// Replay events for a specific memory
    pub async fn replay(&self, memory_id: &str) -> EventStoreResult<Vec<MemoryEvent>> {
        self.events
            .get(memory_id)
            .cloned()
            .ok_or_else(|| EventStoreError::EventNotFound(memory_id.to_string()))
    }

    /// Rebuild memory state from events
    pub async fn rebuild(&self, memory_id: &str) -> EventStoreResult<RebuiltMemory> {
        let events = self.replay(memory_id).await?;

        if events.is_empty() {
            return Err(EventStoreError::EventNotFound(memory_id.to_string()));
        }

        // Find creation event
        let created_event = events
            .first()
            .ok_or_else(|| EventStoreError::RebuildError("No events found".to_string()))?;

        let mut state = RebuiltMemory::from_created(created_event)
            .ok_or_else(|| EventStoreError::RebuildError("First event is not Created".to_string()))?;

        // Apply subsequent events
        for event in events.iter().skip(1) {
            match event {
                MemoryEvent::Updated { .. } => {
                    state.apply_update(event)?;
                }
                _ => {
                    state.event_count += 1;
                }
            }
        }

        Ok(state)
    }

    /// Take a snapshot of current state
    pub async fn snapshot(&mut self, memory_id: &str) -> EventStoreResult<Snapshot> {
        let state = self.rebuild(memory_id).await?;
        let events = self.replay(memory_id).await?;

        let snapshot = Snapshot {
            memory_id: memory_id.to_string(),
            state,
            events_since_snapshot: 0,
            timestamp: Utc::now(),
        };

        self.snapshots
            .insert(memory_id.to_string(), snapshot.clone());
        self.stats.total_snapshots = self.snapshots.len();

        Ok(snapshot)
    }

    /// Get snapshot for a memory
    pub fn get_snapshot(&self, memory_id: &str) -> Option<&Snapshot> {
        self.snapshots.get(memory_id)
    }

    /// Get events filtered by type
    pub async fn replay_filtered(
        &self,
        memory_id: &str,
        event_types: &[&str],
    ) -> EventStoreResult<Vec<MemoryEvent>> {
        let events = self.replay(memory_id).await?;

        let filtered: Vec<MemoryEvent> = events
            .into_iter()
            .filter(|e| {
                let type_name = match e {
                    MemoryEvent::Created { .. } => "Created",
                    MemoryEvent::Updated { .. } => "Updated",
                    MemoryEvent::Deleted { .. } => "Deleted",
                    MemoryEvent::Accessed { .. } => "Accessed",
                    MemoryEvent::Promoted { .. } => "Promoted",
                    MemoryEvent::Merged { .. } => "Merged",
                };
                event_types.contains(&type_name)
            })
            .collect();

        Ok(filtered)
    }

    /// Get events in time range
    pub async fn replay_in_range(
        &self,
        memory_id: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> EventStoreResult<Vec<MemoryEvent>> {
        let events = self.replay(memory_id).await?;

        let filtered: Vec<MemoryEvent> = events
            .into_iter()
            .filter(|e| {
                let timestamp = match e {
                    MemoryEvent::Created { timestamp, .. } => *timestamp,
                    MemoryEvent::Updated { timestamp, .. } => *timestamp,
                    MemoryEvent::Deleted { timestamp, .. } => *timestamp,
                    MemoryEvent::Accessed { timestamp, .. } => *timestamp,
                    MemoryEvent::Promoted { timestamp, .. } => *timestamp,
                    MemoryEvent::Merged { timestamp, .. } => *timestamp,
                };
                timestamp >= start && timestamp <= end
            })
            .collect();

        Ok(filtered)
    }

    /// Get all memory IDs with events
    pub fn memory_ids(&self) -> Vec<String> {
        self.events.keys().cloned().collect()
    }

    /// Get event count for a memory
    pub fn event_count(&self, memory_id: &str) -> usize {
        self.events.get(memory_id).map(|e| e.len()).unwrap_or(0)
    }

    /// Get statistics
    pub fn stats(&self) -> &EventStoreStats {
        &self.stats
    }

    /// Clear all events for a memory
    pub fn clear(&mut self, memory_id: &str) {
        if let Some(events) = self.events.remove(memory_id) {
            self.stats.total_events -= events.len();
        }
        self.snapshots.remove(memory_id);
        self.stats.total_memories = self.events.len();
        self.stats.total_snapshots = self.snapshots.len();
    }

    /// Clear all events
    pub fn clear_all(&mut self) {
        self.events.clear();
        self.snapshots.clear();
        self.stats = EventStoreStats::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_event_store() -> EventStore {
        EventStore::new()
    }

    fn create_created_event(memory_id: &str) -> MemoryEvent {
        MemoryEvent::Created {
            memory_id: memory_id.to_string(),
            content: "Initial content".to_string(),
            memory_type: "semantic".to_string(),
            importance: 0.8,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    fn create_updated_event(memory_id: &str, version: u64) -> MemoryEvent {
        MemoryEvent::Updated {
            memory_id: memory_id.to_string(),
            old_content: "Old content".to_string(),
            new_content: "New content".to_string(),
            version,
            timestamp: Utc::now(),
            reason: Some("Test update".to_string()),
        }
    }

    #[tokio::test]
    async fn test_append_and_replay() {
        let mut store = create_test_event_store();
        let memory_id = "test-memory-1";
        let event = create_created_event(memory_id);

        store.append(memory_id, event).await.unwrap();

        let events = store.replay(memory_id).await.unwrap();
        assert_eq!(events.len(), 1);
    }

    #[tokio::test]
    async fn test_rebuild_from_created() {
        let mut store = create_test_event_store();
        let memory_id = "test-memory-2";
        let event = create_created_event(memory_id);

        store.append(memory_id, event).await.unwrap();

        let state = store.rebuild(memory_id).await.unwrap();
        assert_eq!(state.id, memory_id);
        assert_eq!(state.content, "Initial content");
        assert_eq!(state.version, 1);
    }

    #[tokio::test]
    async fn test_rebuild_with_updates() {
        let mut store = create_test_event_store();
        let memory_id = "test-memory-3";

        // First event: Created
        store
            .append(memory_id, create_created_event(memory_id))
            .await
            .unwrap();

        // Second event: Updated
        store
            .append(memory_id, create_updated_event(memory_id, 2))
            .await
            .unwrap();

        let state = store.rebuild(memory_id).await.unwrap();
        assert_eq!(state.version, 2);
        assert_eq!(state.content, "New content");
        assert_eq!(state.event_count, 2);
    }

    #[tokio::test]
    async fn test_snapshot() {
        let mut store = create_test_event_store();
        let memory_id = "test-memory-4";

        store
            .append(memory_id, create_created_event(memory_id))
            .await
            .unwrap();

        let snapshot = store.snapshot(memory_id).await.unwrap();
        assert_eq!(snapshot.memory_id, memory_id);
        assert_eq!(snapshot.state.version, 1);
    }

    #[tokio::test]
    async fn test_replay_not_found() {
        let store = create_test_event_store();
        let result = store.replay("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_clear() {
        let mut store = create_test_event_store();
        let memory_id = "test-memory-5";

        store
            .append(memory_id, create_created_event(memory_id))
            .await
            .unwrap();

        assert_eq!(store.event_count(memory_id), 1);

        store.clear(memory_id);

        assert_eq!(store.event_count(memory_id), 0);
    }

    #[tokio::test]
    async fn test_stats() {
        let mut store = create_test_event_store();

        store
            .append("memory-1", create_created_event("memory-1"))
            .await
            .unwrap();
        store
            .append("memory-2", create_created_event("memory-2"))
            .await
            .unwrap();

        let stats = store.stats();
        assert_eq!(stats.total_events, 2);
        assert_eq!(stats.total_memories, 2);
    }
}
