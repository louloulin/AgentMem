//! Working Memory Service implementation
//!
//! High-performance in-memory working memory with:
//! - DashMap for concurrent access
//! - Session-based isolation
//! - Priority-based retrieval
//! - Automatic expiration
//! - EventBus integration

use super::{WorkingMemoryConfig, DEFAULT_CLEANUP_INTERVAL_SECONDS, DEFAULT_TTL_SECONDS};
use agent_mem_event_bus::EventBus;
use agent_mem_performance::telemetry::{EventType, MemoryEvent};
use agent_mem_traits::{Result, WorkingMemoryItem};
use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Working Memory Service - fast temporary context storage
pub struct WorkingMemoryService {
    /// In-memory storage: session_id -> (item_id -> item)
    storage: Arc<DashMap<String, DashMap<String, WorkingMemoryItem>>>,

    /// Configuration
    config: WorkingMemoryConfig,

    /// EventBus for event notifications (optional)
    event_bus: Option<EventBus>,

    /// Statistics
    stats: Arc<RwLock<WorkingMemoryStats>>,
}

/// Working memory statistics
#[derive(Debug, Clone, Default)]
pub struct WorkingMemoryStats {
    /// Total items stored
    pub total_items: u64,

    /// Total sessions
    pub total_sessions: u64,

    /// Items added
    pub items_added: u64,

    /// Items removed
    pub items_removed: u64,

    /// Expired items cleaned up
    pub expired_items_cleaned: u64,

    /// Last cleanup time
    pub last_cleanup_at: Option<DateTime<Utc>>,
}

impl WorkingMemoryService {
    /// Create a new working memory service
    pub async fn new(config: WorkingMemoryConfig) -> Result<Self> {
        info!("Creating WorkingMemoryService with config: {:?}", config);

        let event_bus = if config.enable_event_bus {
            Some(EventBus::new(100))
        } else {
            None
        };

        let service = Self {
            storage: Arc::new(DashMap::new()),
            config,
            event_bus,
            stats: Arc::new(RwLock::new(WorkingMemoryStats::default())),
        };

        // Start background cleanup if enabled
        if service.config.enable_auto_cleanup {
            service.start_cleanup_task().await;
        }

        info!("WorkingMemoryService created successfully");
        Ok(service)
    }

    /// Add an item to working memory
    pub async fn add_item(&self, mut item: WorkingMemoryItem) -> Result<WorkingMemoryItem> {
        // Validate and set defaults
        if item.id.is_empty() {
            item.id = Uuid::new_v4().to_string();
        }

        if item.created_at.timestamp() == 0 {
            item.created_at = Utc::now();
        }

        // Set default TTL if not specified
        if item.expires_at.is_none() && self.config.default_ttl_seconds > 0 {
            item.expires_at = Some(item.created_at + Duration::seconds(self.config.default_ttl_seconds));
        }

        // Check capacity
        let session_items = self.storage.entry(item.session_id.clone()).or_default();
        if session_items.len() >= self.config.max_items_per_session {
            // Remove lowest priority item
            if let Some((lowest_key, _)) = session_items
                .iter()
                .min_by_key(|(_, a)| a.priority)
            {
                session_items.remove(lowest_key);
                debug!("Removed lowest priority item due to capacity limit");
            }
        }

        // Add item
        session_items.insert(item.id.clone(), item.clone());

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.items_added += 1;
            stats.total_items = self.storage.iter().map(|m| m.len() as u64).sum();
        }

        // Publish event
        if let Some(ref bus) = self.event_bus {
            let event = MemoryEvent::new(EventType::MemoryCreated)
                .with_memory_id(item.id.clone())
                .with_agent_id(item.agent_id.clone())
                .with_user_id(item.user_id.clone());
            let _ = bus.publish(event).await;
        }

        debug!("Added working memory item: {}", item.id);
        Ok(item)
    }

    /// Get all items for a session
    pub async fn get_session_items(&self, session_id: &str) -> Result<Vec<WorkingMemoryItem>> {
        let items = self
            .storage
            .get(session_id)
            .map(|map| map.iter().map(|(_, v)| v.clone()).collect())
            .unwrap_or_default();

        debug!(
            "Retrieved {} items for session {}",
            items.len(),
            session_id
        );
        Ok(items)
    }

    /// Get an item by ID
    pub async fn get_item(&self, session_id: &str, item_id: &str) -> Result<Option<WorkingMemoryItem>> {
        let item = self
            .storage
            .get(session_id)
            .and_then(|map| map.get(item_id).map(|v| v.clone()));

        Ok(item)
    }

    /// Get items by priority (minimum priority)
    pub async fn get_by_priority(
        &self,
        session_id: &str,
        min_priority: i32,
    ) -> Result<Vec<WorkingMemoryItem>> {
        let items = self
            .storage
            .get(session_id)
            .map(|map| {
                map.iter()
                    .filter(|(_, v)| v.priority >= min_priority)
                    .map(|(_, v)| v.clone())
                    .collect()
            })
            .unwrap_or_default();

        debug!(
            "Retrieved {} items with priority >= {} for session {}",
            items.len(),
            min_priority,
            session_id
        );
        Ok(items)
    }

    /// Remove an item
    pub async fn remove_item(&self, session_id: &str, item_id: &str) -> Result<bool> {
        let removed = self
            .storage
            .get(session_id)
            .map(|map| map.remove(item_id).is_some())
            .unwrap_or(false);

        if removed {
            let mut stats = self.stats.write().await;
            stats.items_removed += 1;
            stats.total_items = self.storage.iter().map(|m| m.len() as u64).sum();

            // Publish event
            if let Some(ref bus) = self.event_bus {
                let event = MemoryEvent::new(EventType::MemoryDeleted)
                    .with_memory_id(item_id.to_string());
                let _ = bus.publish(event).await;
            }

            debug!("Removed working memory item: {}", item_id);
        }

        Ok(removed)
    }

    /// Clear all items for a session
    pub async fn clear_session(&self, session_id: &str) -> Result<i64> {
        let count = self
            .storage
            .remove(session_id)
            .map(|map| map.len() as i64)
            .unwrap_or(0);

        if count > 0 {
            let mut stats = self.stats.write().await;
            stats.items_removed += count as u64;
            stats.total_items = self.storage.iter().map(|m| m.len() as u64).sum();

            // Publish event
            if let Some(ref bus) = self.event_bus {
                let event = MemoryEvent::new(EventType::MemoryDeleted)
                    .with_metadata("session_id".to_string(), serde_json::json!(session_id));
                let _ = bus.publish(event).await;
            }

            info!("Cleared {} items for session {}", count, session_id);
        }

        Ok(count)
    }

    /// Clear expired items across all sessions
    pub async fn clear_expired(&self) -> Result<i64> {
        let now = Utc::now();
        let mut total_removed = 0i64;

        // Iterate over all sessions
        for session_entry in self.storage.iter() {
            let session_id = session_entry.key().clone();
            let session_map = session_entry.value();

            // Find expired items
            let expired_ids: Vec<String> = session_map
                .iter()
                .filter(|(_, item)| {
                    item
                        .expires_at
                        .map(|exp| exp < now)
                        .unwrap_or(false)
                })
                .map(|(id, _)| id.clone())
                .collect();

            // Remove expired items
            for id in expired_ids {
                session_map.remove(&id);
                total_removed += 1;
            }

            // Remove empty sessions
            if session_map.is_empty() {
                self.storage.remove(&session_id);
            }
        }

        if total_removed > 0 {
            let mut stats = self.stats.write().await;
            stats.expired_items_cleaned += total_removed as u64;
            stats.total_items = self.storage.iter().map(|m| m.len() as u64).sum();
            stats.last_cleanup_at = Some(now);

            info!("Cleared {} expired items", total_removed);
        }

        Ok(total_removed)
    }

    /// Get statistics
    pub async fn get_stats(&self) -> WorkingMemoryStats {
        let mut stats = self.stats.write().await.clone();
        stats.total_sessions = self.storage.len() as u64;
        stats.total_items = self.storage.iter().map(|m| m.len() as u64).sum();
        stats
    }

    /// Get session count
    pub fn session_count(&self) -> usize {
        self.storage.len()
    }

    /// Start background cleanup task
    async fn start_cleanup_task(&self) {
        let storage = self.storage.clone();
        let interval_seconds = self.config.cleanup_interval_seconds;
        let stats = self.stats.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(interval_seconds));
            loop {
                interval.tick().await;

                let now = Utc::now();
                let mut total_removed = 0i64;

                for session_entry in storage.iter() {
                    let session_map = session_entry.value();

                    let expired_ids: Vec<String> = session_map
                        .iter()
                        .filter(|(_, item)| {
                            item
                                .expires_at
                                .map(|exp| exp < now)
                                .unwrap_or(false)
                        })
                        .map(|(id, _)| id.clone())
                        .collect();

                    for id in expired_ids {
                        session_map.remove(&id);
                        total_removed += 1;
                    }
                }

                if total_removed > 0 {
                    let mut s = stats.write().await;
                    s.expired_items_cleaned += total_removed as u64;
                    s.total_items = storage.iter().map(|m| m.len() as u64).sum();
                    s.last_cleanup_at = Some(now);

                    debug!("Auto-cleanup: removed {} expired items", total_removed);
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DEFAULT_CAPACITY;

    #[tokio::test]
    async fn test_service_creation() {
        let config = WorkingMemoryConfig::default();
        let service = WorkingMemoryService::new(config).await.unwrap();
        assert_eq!(service.session_count(), 0);
    }

    #[tokio::test]
    async fn test_add_and_get_item() {
        let service = WorkingMemoryService::new(WorkingMemoryConfig::default())
            .await
            .unwrap();

        let item = WorkingMemoryItem {
            id: "item-1".to_string(),
            session_id: "session-1".to_string(),
            content: "Test content".to_string(),
            priority: 5,
            expires_at: None,
            created_at: Utc::now(),
            user_id: "user-1".to_string(),
            agent_id: "agent-1".to_string(),
            metadata: serde_json::json!({}),
        };

        let added = service.add_item(item.clone()).await.unwrap();
        assert_eq!(added.id, "item-1");

        let retrieved = service
            .get_item("session-1", "item-1")
            .await
            .unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().content, "Test content");
    }

    #[tokio::test]
    async fn test_get_session_items() {
        let service = WorkingMemoryService::new(WorkingMemoryConfig::default())
            .await
            .unwrap();

        for i in 1..=3 {
            let item = WorkingMemoryItem {
                id: format!("item-{}", i),
                session_id: "session-1".to_string(),
                content: format!("Content {}", i),
                priority: i,
                expires_at: None,
                created_at: Utc::now(),
                user_id: "user-1".to_string(),
                agent_id: "agent-1".to_string(),
                metadata: serde_json::json!({}),
            };
            service.add_item(item).await.unwrap();
        }

        let items = service.get_session_items("session-1").await.unwrap();
        assert_eq!(items.len(), 3);
    }

    #[tokio::test]
    async fn test_get_by_priority() {
        let service = WorkingMemoryService::new(WorkingMemoryConfig::default())
            .await
            .unwrap();

        for i in 1..=5 {
            let item = WorkingMemoryItem {
                id: format!("item-{}", i),
                session_id: "session-1".to_string(),
                content: format!("Content {}", i),
                priority: i,
                expires_at: None,
                created_at: Utc::now(),
                user_id: "user-1".to_string(),
                agent_id: "agent-1".to_string(),
                metadata: serde_json::json!({}),
            };
            service.add_item(item).await.unwrap();
        }

        let items = service.get_by_priority("session-1", 3).await.unwrap();
        assert_eq!(items.len(), 3); // priorities 3, 4, 5
    }

    #[tokio::test]
    async fn test_remove_item() {
        let service = WorkingMemoryService::new(WorkingMemoryConfig::default())
            .await
            .unwrap();

        let item = WorkingMemoryItem {
            id: "item-1".to_string(),
            session_id: "session-1".to_string(),
            content: "Test".to_string(),
            priority: 1,
            expires_at: None,
            created_at: Utc::now(),
            user_id: "user-1".to_string(),
            agent_id: "agent-1".to_string(),
            metadata: serde_json::json!({}),
        };

        service.add_item(item).await.unwrap();

        let removed = service.remove_item("session-1", "item-1").await.unwrap();
        assert!(removed);

        let removed_again = service
            .remove_item("session-1", "item-1")
            .await
            .unwrap();
        assert!(!removed_again);
    }

    #[tokio::test]
    async fn test_clear_session() {
        let service = WorkingMemoryService::new(WorkingMemoryConfig::default())
            .await
            .unwrap();

        for i in 1..=3 {
            let item = WorkingMemoryItem {
                id: format!("item-{}", i),
                session_id: "session-1".to_string(),
                content: format!("Content {}", i),
                priority: i,
                expires_at: None,
                created_at: Utc::now(),
                user_id: "user-1".to_string(),
                agent_id: "agent-1".to_string(),
                metadata: serde_json::json!({}),
            };
            service.add_item(item).await.unwrap();
        }

        let count = service.clear_session("session-1").await.unwrap();
        assert_eq!(count, 3);

        let items = service.get_session_items("session-1").await.unwrap();
        assert_eq!(items.len(), 0);
    }

    #[tokio::test]
    async fn test_auto_expiration() {
        let config = WorkingMemoryConfig::default()
            .with_ttl(1) // 1 second TTL
            .without_cleanup(); // Disable auto cleanup for test

        let service = WorkingMemoryService::new(config).await.unwrap();

        let item = WorkingMemoryItem {
            id: "item-1".to_string(),
            session_id: "session-1".to_string(),
            content: "Test".to_string(),
            priority: 1,
            expires_at: None, // Will be set to 1 second from now
            created_at: Utc::now(),
            user_id: "user-1".to_string(),
            agent_id: "agent-1".to_string(),
            metadata: serde_json::json!({}),
        };

        service.add_item(item).await.unwrap();

        // Wait for expiration
        tokio::time::sleep(tokio::time::Duration::from_millis(1100)).await;

        let cleared = service.clear_expired().await.unwrap();
        assert_eq!(cleared, 1);

        let items = service.get_session_items("session-1").await.unwrap();
        assert_eq!(items.len(), 0);
    }

    #[tokio::test]
    async fn test_stats() {
        let service = WorkingMemoryService::new(WorkingMemoryConfig::default())
            .await
            .unwrap();

        // Add some items
        for i in 1..=3 {
            let item = WorkingMemoryItem {
                id: format!("item-{}", i),
                session_id: "session-1".to_string(),
                content: format!("Content {}", i),
                priority: i,
                expires_at: None,
                created_at: Utc::now(),
                user_id: "user-1".to_string(),
                agent_id: "agent-1".to_string(),
                metadata: serde_json::json!({}),
            };
            service.add_item(item).await.unwrap();
        }

        let stats = service.get_stats().await;
        assert_eq!(stats.items_added, 3);
        assert_eq!(stats.total_items, 3);
        assert_eq!(stats.total_sessions, 1);
    }
}
