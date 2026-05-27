//! Integration Tests for AgentMem Core
//!
//! This module provides end-to-end integration tests for the core functionality,
//! including event sourcing and optimistic locking integration.

use agent_mem_core::event_sourcing::{
    EventStore, MemoryEvent, RebuiltMemory, Snapshot,
};
use agent_mem_core::optimistic_lock::{
    OptimisticLockManager, VersionedMemory, VersionInfo,
};
use chrono::Utc;
use std::collections::HashMap;

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Test: Full memory lifecycle with event sourcing and optimistic locking
    #[tokio::test]
    async fn test_memory_lifecycle_with_versioning() {
        let mut event_store = EventStore::new();
        let mut lock_manager = OptimisticLockManager::new();

        let memory_id = "test-lifecycle-memory";
        let initial_content = "Initial memory content";

        // Step 1: Initialize version tracking
        let version_info = lock_manager.init_version(memory_id).unwrap();
        assert_eq!(version_info.version, 1);

        // Step 2: Create memory event
        let created_event = MemoryEvent::Created {
            memory_id: memory_id.to_string(),
            content: initial_content.to_string(),
            memory_type: "semantic".to_string(),
            importance: 0.8,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };
        event_store.append(memory_id, created_event).await.unwrap();

        // Step 3: Verify version after creation
        let version = lock_manager.get_version(memory_id).unwrap();
        assert_eq!(version.version, 1);

        // Step 4: Update with optimistic locking
        let updated_content = "Updated memory content";
        let new_version = lock_manager
            .verify_and_update(memory_id, 1, updated_content)
            .unwrap();
        assert_eq!(new_version.version, 2);

        // Step 5: Record update event
        let updated_event = MemoryEvent::Updated {
            memory_id: memory_id.to_string(),
            old_content: initial_content.to_string(),
            new_content: updated_content.to_string(),
            version: 2,
            timestamp: Utc::now(),
            reason: Some("Test update".to_string()),
        };
        event_store.append(memory_id, updated_event).await.unwrap();

        // Step 6: Verify event count
        assert_eq!(event_store.event_count(memory_id), 2);

        // Step 7: Rebuild state from events
        let rebuilt = event_store.rebuild(memory_id).await.unwrap();
        assert_eq!(rebuilt.version, 2);
        assert_eq!(rebuilt.content, updated_content);
        assert_eq!(rebuilt.event_count, 2);
    }

    /// Test: Optimistic locking version conflict handling
    #[tokio::test]
    async fn test_version_conflict_detection() {
        let mut lock_manager = OptimisticLockManager::new();
        let memory_id = "conflict-test-memory";

        // Initialize with version 1
        lock_manager.init_version(memory_id).unwrap();

        // First update succeeds with version 1
        let result1 = lock_manager.verify_and_update(memory_id, 1, "content v1");
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap().version, 2);

        // Second update with stale version 1 fails
        let result2 = lock_manager.verify_and_update(memory_id, 1, "content v2");
        assert!(result2.is_err());
    }

    /// Test: Event sourcing replay with filtered events
    #[tokio::test]
    async fn test_event_filtered_replay() {
        let mut event_store = EventStore::new();
        let memory_id = "filter-test-memory";

        // Add multiple event types
        event_store
            .append(
                memory_id,
                MemoryEvent::Created {
                    memory_id: memory_id.to_string(),
                    content: "Initial".to_string(),
                    memory_type: "semantic".to_string(),
                    importance: 0.8,
                    timestamp: Utc::now(),
                    metadata: HashMap::new(),
                },
            )
            .await
            .unwrap();

        event_store
            .append(
                memory_id,
                MemoryEvent::Accessed {
                    memory_id: memory_id.to_string(),
                    access_type: "read".to_string(),
                    timestamp: Utc::now(),
                },
            )
            .await
            .unwrap();

        event_store
            .append(
                memory_id,
                MemoryEvent::Updated {
                    memory_id: memory_id.to_string(),
                    old_content: "Old".to_string(),
                    new_content: "New".to_string(),
                    version: 2,
                    timestamp: Utc::now(),
                    reason: Some("Test".to_string()),
                },
            )
            .await
            .unwrap();

        // Filter to only Updated events
        let filtered = event_store
            .replay_filtered(memory_id, &["Updated"])
            .await
            .unwrap();
        assert_eq!(filtered.len(), 1);

        // Filter to only Accessed events
        let accessed = event_store
            .replay_filtered(memory_id, &["Accessed"])
            .await
            .unwrap();
        assert_eq!(accessed.len(), 1);

        // Filter with multiple types
        let multi = event_store
            .replay_filtered(memory_id, &["Updated", "Accessed"])
            .await
            .unwrap();
        assert_eq!(multi.len(), 2);
    }

    /// Test: Snapshot creation and retrieval
    #[tokio::test]
    async fn test_snapshot_operations() {
        let mut event_store = EventStore::new();
        let memory_id = "snapshot-test-memory";

        // Create initial event
        event_store
            .append(
                memory_id,
                MemoryEvent::Created {
                    memory_id: memory_id.to_string(),
                    content: "Initial content".to_string(),
                    memory_type: "semantic".to_string(),
                    importance: 0.8,
                    timestamp: Utc::now(),
                    metadata: HashMap::new(),
                },
            )
            .await
            .unwrap();

        // Create snapshot
        let snapshot = event_store.snapshot(memory_id).await.unwrap();
        assert_eq!(snapshot.state.version, 1);
        assert_eq!(snapshot.state.content, "Initial content");

        // Verify snapshot exists
        let retrieved = event_store.get_snapshot(memory_id);
        assert!(retrieved.is_some());

        // Add more events after snapshot
        event_store
            .append(
                memory_id,
                MemoryEvent::Updated {
                    memory_id: memory_id.to_string(),
                    old_content: "Old".to_string(),
                    new_content: "Updated after snapshot".to_string(),
                    version: 2,
                    timestamp: Utc::now(),
                    reason: Some("Post-snapshot update".to_string()),
                },
            )
            .await
            .unwrap();

        // Verify new version
        let rebuilt = event_store.rebuild(memory_id).await.unwrap();
        assert_eq!(rebuilt.version, 2);
    }

    /// Test: VersionedMemory with auto-increment
    #[test]
    fn test_versioned_memory_auto_increment() {
        let mut memory = VersionedMemory::new(
            "test".to_string(),
            "initial content".to_string(),
            "semantic".to_string(),
            0.8,
        );

        // Verify initial version
        assert_eq!(memory.version(), 1);

        // Update content should increment version
        memory.update_content("updated content".to_string());
        assert_eq!(memory.version(), 2);
        assert_eq!(memory.content, "updated content");

        // Another update
        memory.update_content("another update".to_string());
        assert_eq!(memory.version(), 3);
    }

    /// Test: Event store stats tracking
    #[tokio::test]
    async fn test_event_store_stats() {
        let mut event_store = EventStore::new();

        // Add events for multiple memories
        event_store
            .append(
                "memory-1",
                MemoryEvent::Created {
                    memory_id: "memory-1".to_string(),
                    content: "Content 1".to_string(),
                    memory_type: "semantic".to_string(),
                    importance: 0.8,
                    timestamp: Utc::now(),
                    metadata: HashMap::new(),
                },
            )
            .await
            .unwrap();

        event_store
            .append(
                "memory-2",
                MemoryEvent::Created {
                    memory_id: "memory-2".to_string(),
                    content: "Content 2".to_string(),
                    memory_type: "episodic".to_string(),
                    importance: 0.6,
                    timestamp: Utc::now(),
                    metadata: HashMap::new(),
                },
            )
            .await
            .unwrap();

        let stats = event_store.stats();
        assert_eq!(stats.total_events, 2);
        assert_eq!(stats.total_memories, 2);
    }

    /// Test: Memory promotion tracking
    #[tokio::test]
    async fn test_memory_promotion_tracking() {
        let mut event_store = EventStore::new();
        let memory_id = "promotion-test-memory";

        // Create initial memory
        event_store
            .append(
                memory_id,
                MemoryEvent::Created {
                    memory_id: memory_id.to_string(),
                    content: "Working memory content".to_string(),
                    memory_type: "working".to_string(),
                    importance: 0.5,
                    timestamp: Utc::now(),
                    metadata: HashMap::new(),
                },
            )
            .await
            .unwrap();

        // Promote to episodic
        event_store
            .append(
                memory_id,
                MemoryEvent::Promoted {
                    memory_id: memory_id.to_string(),
                    from_level: "working".to_string(),
                    to_level: "episodic".to_string(),
                    timestamp: Utc::now(),
                    reason: Some("Memory consolidation".to_string()),
                },
            )
            .await
            .unwrap();

        // Verify events
        let events = event_store.replay(memory_id).await.unwrap();
        assert_eq!(events.len(), 2);

        // Rebuild should preserve all events
        let rebuilt = event_store.rebuild(memory_id).await.unwrap();
        assert_eq!(rebuilt.event_count, 2);
    }

    /// Test: Concurrent update simulation
    #[tokio::test]
    async fn test_concurrent_update_simulation() {
        let mut lock_manager = OptimisticLockManager::new();
        let memory_id = "concurrent-test-memory";

        // Initialize
        lock_manager.init_version(memory_id).unwrap();

        // Simulate two concurrent readers getting version 1
        let reader1_version = 1u64;
        let reader2_version = 1u64;

        // Reader 1 updates successfully
        let result1 = lock_manager.verify_and_update(memory_id, reader1_version, "reader1 update");
        assert!(result1.is_ok());
        let new_version = result1.unwrap().version;

        // Reader 2 should detect conflict
        let result2 = lock_manager.verify_and_update(memory_id, reader2_version, "reader2 update");
        assert!(result2.is_err());

        // Verify final state
        let final_version = lock_manager.get_version(memory_id).unwrap();
        assert_eq!(final_version.version, new_version);
    }

    /// Test: Lock manager with custom config
    #[test]
    fn test_custom_lock_config() {
        use agent_mem_core::optimistic_lock::LockManagerConfig;

        let config = LockManagerConfig {
            max_retries: 5,
            base_retry_delay_ms: 50,
            max_retry_delay_ms: 1000,
            transaction_timeout_ms: 60000,
            auto_increment_version: true,
        };

        let manager = OptimisticLockManager::with_config(config);
        let stats = manager.stats();

        assert_eq!(stats.config.max_retries, 5);
        assert_eq!(stats.config.base_retry_delay_ms, 50);
    }

    /// Test: RebuiltMemory state reconstruction
    #[tokio::test]
    async fn test_rebuilt_memory_state() {
        let mut event_store = EventStore::new();
        let memory_id = "state-test-memory";

        // Create memory
        event_store
            .append(
                memory_id,
                MemoryEvent::Created {
                    memory_id: memory_id.to_string(),
                    content: "Initial state".to_string(),
                    memory_type: "semantic".to_string(),
                    importance: 0.9,
                    timestamp: Utc::now(),
                    metadata: HashMap::new(),
                },
            )
            .await
            .unwrap();

        // Update
        event_store
            .append(
                memory_id,
                MemoryEvent::Updated {
                    memory_id: memory_id.to_string(),
                    old_content: "Initial state".to_string(),
                    new_content: "Modified state".to_string(),
                    version: 2,
                    timestamp: Utc::now(),
                    reason: Some("Enhancement".to_string()),
                },
            )
            .await
            .unwrap();

        // Rebuild
        let rebuilt = event_store.rebuild(memory_id).await.unwrap();

        // Verify rebuilt state
        assert_eq!(rebuilt.id, memory_id);
        assert_eq!(rebuilt.content, "Modified state");
        assert_eq!(rebuilt.version, 2);
        assert_eq!(rebuilt.memory_type, "semantic");
        assert_eq!(rebuilt.importance, 0.9);
        assert_eq!(rebuilt.event_count, 2);
    }

    /// Test: Event store clear operations
    #[tokio::test]
    async fn test_clear_operations() {
        let mut event_store = EventStore::new();
        let memory_id = "clear-test-memory";

        // Add events
        event_store
            .append(
                memory_id,
                MemoryEvent::Created {
                    memory_id: memory_id.to_string(),
                    content: "Content".to_string(),
                    memory_type: "semantic".to_string(),
                    importance: 0.8,
                    timestamp: Utc::now(),
                    metadata: HashMap::new(),
                },
            )
            .await
            .unwrap();

        assert_eq!(event_store.event_count(memory_id), 1);

        // Clear specific memory
        event_store.clear(memory_id);
        assert_eq!(event_store.event_count(memory_id), 0);

        // Verify rebuild fails
        let result = event_store.rebuild(memory_id).await;
        assert!(result.is_err());
    }

    /// Test: Version comparison utilities
    #[test]
    fn test_version_comparison() {
        use agent_mem_core::optimistic_lock::VersionComparison;

        let info1 = VersionInfo {
            memory_id: "test".to_string(),
            version: 1,
            last_modified: Utc::now(),
            modified_by: None,
            change_description: None,
        };

        let mut info2 = info1.clone();
        info2.version = 2;

        // Older comparison
        assert_eq!(info1.compare_versions(&info2), VersionComparison::Older);
        assert!(info1.is_stale(&info2));

        // Newer comparison
        assert_eq!(info2.compare_versions(&info1), VersionComparison::Newer);
        assert!(!info2.is_stale(&info1));

        // Equal comparison
        let mut info3 = info1.clone();
        info3.version = 1;
        assert_eq!(info1.compare_versions(&info3), VersionComparison::Equal);
        assert!(!info1.is_stale(&info3));
    }
}