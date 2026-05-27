//! Optimistic Locking Module for AgentMem
//!
//! This module provides optimistic locking capabilities for memory operations,
//! ensuring data consistency in concurrent environments.
//!
//! # Features
//!
//! - Version-based concurrency control
//! - Conflict detection and resolution
//! - Retry mechanisms with exponential backoff
//! - Atomic update operations
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │              OptimisticLockManager                         │
//! │  ┌─────────────────────────────────────────────────────┐   │
//! │  │  VersionedMemory: HashMap<MemoryId, VersionInfo>   │   │
//! │  └─────────────────────────────────────────────────────┘   │
//! └─────────────────────────────────────────────────────────────┘
//!                            │
//!                            ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │              Lock Operations                              │
//! │  read_version() │ update_with_lock() │ verify_and_commit() │
//! └─────────────────────────────────────────────────────────────┘
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Optimistic lock errors
#[derive(Debug, Error)]
pub enum OptimisticLockError {
    /// Version conflict detected
    #[error("Version conflict: expected {expected}, but found {actual}")]
    VersionConflict {
        /// Expected version
        expected: u64,
        /// Actual version
        actual: u64,
    },

    /// Memory not found
    #[error("Memory not found: {0}")]
    MemoryNotFound(String),

    /// Lock acquisition failed
    #[error("Failed to acquire lock: {0}")]
    LockAcquisitionFailed(String),

    /// Transaction timeout
    #[error("Transaction timeout after {0}ms")]
    TransactionTimeout(u64),

    /// Retry exhausted
    #[error("Retry exhausted after {0} attempts")]
    RetryExhausted(u32),

    /// Invalid version
    #[error("Invalid version number: {0}")]
    InvalidVersion(String),
}

/// Result type for optimistic lock operations
pub type LockResult<T> = Result<T, OptimisticLockError>;

/// Version information for a memory item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    /// Memory ID
    pub memory_id: String,
    /// Current version number
    pub version: u64,
    /// Last modified timestamp
    pub last_modified: DateTime<Utc>,
    /// Modified by (agent/user ID)
    pub modified_by: Option<String>,
    /// Change description
    pub change_description: Option<String>,
}

/// Versioned memory with optimistic locking support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedMemory {
    /// Memory ID
    pub id: String,
    /// Memory content
    pub content: String,
    /// Memory type
    pub memory_type: String,
    /// Current version number (incremented on each update)
    pub version: u64,
    /// Created at timestamp
    pub created_at: DateTime<Utc>,
    /// Last modified timestamp
    pub last_modified: DateTime<Utc>,
    /// Importance score
    pub importance: f32,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl VersionedMemory {
    /// Create a new versioned memory
    pub fn new(
        id: String,
        content: String,
        memory_type: String,
        importance: f32,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            content,
            memory_type,
            version: 1,
            created_at: now,
            last_modified: now,
            importance,
            metadata: HashMap::new(),
        }
    }

    /// Get the current version
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Check if the version matches expected
    pub fn check_version(&self, expected: u64) -> LockResult<()> {
        if self.version != expected {
            return Err(OptimisticLockError::VersionConflict {
                expected,
                actual: self.version,
            });
        }
        Ok(())
    }

    /// Update content with version increment
    pub fn update_content(&mut self, new_content: String) {
        self.content = new_content;
        self.version += 1;
        self.last_modified = Utc::now();
    }
}

/// Optimistic lock manager for concurrent memory operations
#[derive(Debug, Clone, Default)]
pub struct OptimisticLockManager {
    /// Version tracking by memory ID
    versions: HashMap<String, VersionInfo>,
    /// Configuration
    config: LockManagerConfig,
}

/// Configuration for the lock manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockManagerConfig {
    /// Maximum retry attempts
    pub max_retries: u32,
    /// Base delay between retries (ms)
    pub base_retry_delay_ms: u64,
    /// Maximum retry delay (ms)
    pub max_retry_delay_ms: u64,
    /// Transaction timeout (ms)
    pub transaction_timeout_ms: u64,
    /// Enable automatic version increment
    pub auto_increment_version: bool,
}

impl Default for LockManagerConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_retry_delay_ms: 100,
            max_retry_delay_ms: 5000,
            transaction_timeout_ms: 30000,
            auto_increment_version: true,
        }
    }
}

/// Lock statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LockStats {
    /// Total lock operations
    pub total_operations: u64,
    /// Successful operations
    pub successful_operations: u64,
    /// Failed operations (conflicts)
    pub failed_operations: u64,
    /// Retry count
    pub total_retries: u64,
    /// Average retry delay (ms)
    pub avg_retry_delay_ms: f64,
}

impl OptimisticLockManager {
    /// Create a new lock manager
    pub fn new() -> Self {
        Self {
            versions: HashMap::new(),
            config: LockManagerConfig::default(),
        }
    }

    /// Create a lock manager with configuration
    pub fn with_config(config: LockManagerConfig) -> Self {
        Self {
            versions: HashMap::new(),
            config,
        }
    }

    /// Initialize version for a new memory
    pub fn init_version(&mut self, memory_id: &str) -> LockResult<VersionInfo> {
        let info = VersionInfo {
            memory_id: memory_id.to_string(),
            version: 1,
            last_modified: Utc::now(),
            modified_by: None,
            change_description: Some("Initial creation".to_string()),
        };
        self.versions.insert(memory_id.to_string(), info.clone());
        Ok(info)
    }

    /// Get current version for a memory
    pub fn get_version(&self, memory_id: &str) -> LockResult<VersionInfo> {
        self.versions
            .get(memory_id)
            .cloned()
            .ok_or_else(|| OptimisticLockError::MemoryNotFound(memory_id.to_string()))
    }

    /// Verify and update version with optimistic locking
    pub fn verify_and_update(
        &mut self,
        memory_id: &str,
        expected_version: u64,
        new_content: &str,
    ) -> LockResult<VersionInfo> {
        let info = self.get_version(memory_id)?;

        // Check version match
        if info.version != expected_version {
            return Err(OptimisticLockError::VersionConflict {
                expected: expected_version,
                actual: info.version,
            });
        }

        // Update version
        let new_version = expected_version + 1;
        let new_info = VersionInfo {
            memory_id: memory_id.to_string(),
            version: new_version,
            last_modified: Utc::now(),
            modified_by: None,
            change_description: Some(format!("Updated content ({} chars)", new_content.len())),
        };

        self.versions.insert(memory_id.to_string(), new_info.clone());
        Ok(new_info)
    }

    /// Update with automatic retry on conflict
    pub fn update_with_retry<F>(
        &mut self,
        memory_id: &str,
        expected_version: u64,
        update_fn: F,
    ) -> LockResult<VersionInfo>
    where
        F: Fn(u64) -> LockResult<String>,
    {
        let mut attempts = 0;
        let mut current_version = expected_version;

        while attempts < self.config.max_retries {
            attempts += 1;

            match self.verify_and_update(memory_id, current_version, &update_fn(current_version)?) {
                Ok(info) => return Ok(info),
                Err(OptimisticLockError::VersionConflict { expected, actual }) => {
                    current_version = actual;
                    let delay = self.calculate_retry_delay(attempts);
                    std::thread::sleep(std::time::Duration::from_millis(delay));
                }
                Err(e) => return Err(e),
            }
        }

        Err(OptimisticLockError::RetryExhausted(attempts))
    }

    /// Calculate retry delay with exponential backoff
    fn calculate_retry_delay(&self, attempt: u32) -> u64 {
        let delay = self.config.base_retry_delay_ms * 2u64.pow(attempt.saturating_sub(1));
        delay.min(self.config.max_retry_delay_ms)
    }

    /// Delete version for a memory
    pub fn delete_version(&mut self, memory_id: &str) -> LockResult<()> {
        self.versions
            .remove(memory_id)
            .ok_or_else(|| OptimisticLockError::MemoryNotFound(memory_id.to_string()))?;
        Ok(())
    }

    /// Check if memory has version tracking
    pub fn has_version(&self, memory_id: &str) -> bool {
        self.versions.contains_key(memory_id)
    }

    /// Get all tracked memory IDs
    pub fn tracked_memories(&self) -> Vec<String> {
        self.versions.keys().cloned().collect()
    }

    /// Get statistics
    pub fn stats(&self) -> LockManagerStats {
        LockManagerStats {
            tracked_memories: self.versions.len(),
            config: self.config.clone(),
        }
    }
}

/// Lock manager statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockManagerStats {
    /// Number of tracked memories
    pub tracked_memories: usize,
    /// Current configuration
    pub config: LockManagerConfig,
}

/// Atomic update operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomicUpdateResult {
    /// Memory ID
    pub memory_id: String,
    /// New version
    pub new_version: u64,
    /// Previous version
    pub previous_version: u64,
    /// Update timestamp
    pub timestamp: DateTime<Utc>,
    /// Whether the update was successful
    pub success: bool,
    /// Error message if failed
    pub error_message: Option<String>,
}

/// Version comparison result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionComparison {
    /// Current version is newer
    Newer,
    /// Current version is older
    Older,
    /// Versions are equal
    Equal,
}

impl VersionInfo {
    /// Compare versions
    pub fn compare_versions(&self, other: &VersionInfo) -> VersionComparison {
        if self.version > other.version {
            VersionComparison::Newer
        } else if self.version < other.version {
            VersionComparison::Older
        } else {
            VersionComparison::Equal
        }
    }

    /// Check if this version is stale compared to another
    pub fn is_stale(&self, other: &VersionInfo) -> bool {
        self.version < other.version
    }

    /// Get time since last modification
    pub fn time_since_modified(&self) -> chrono::Duration {
        Utc::now() - self.last_modified
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_manager() -> OptimisticLockManager {
        OptimisticLockManager::new()
    }

    #[test]
    fn test_init_version() {
        let mut manager = create_test_manager();
        let memory_id = "test-memory-1";

        let info = manager.init_version(memory_id).unwrap();
        assert_eq!(info.version, 1);
        assert_eq!(info.memory_id, memory_id);
    }

    #[test]
    fn test_get_version_not_found() {
        let manager = create_test_manager();
        let result = manager.get_version("nonexistent");
        assert!(matches!(result, Err(OptimisticLockError::MemoryNotFound(_))));
    }

    #[test]
    fn test_verify_and_update_success() {
        let mut manager = create_test_manager();
        let memory_id = "test-memory-2";

        manager.init_version(memory_id).unwrap();
        let result = manager.verify_and_update(memory_id, 1, "new content");

        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.version, 2);
    }

    #[test]
    fn test_verify_and_update_version_conflict() {
        let mut manager = create_test_manager();
        let memory_id = "test-memory-3";

        manager.init_version(memory_id).unwrap();
        let result = manager.verify_and_update(memory_id, 2, "new content");

        assert!(matches!(
            result,
            Err(OptimisticLockError::VersionConflict { .. })
        ));
    }

    #[test]
    fn test_delete_version() {
        let mut manager = create_test_manager();
        let memory_id = "test-memory-4";

        manager.init_version(memory_id).unwrap();
        assert!(manager.has_version(memory_id));

        manager.delete_version(memory_id).unwrap();
        assert!(!manager.has_version(memory_id));
    }

    #[test]
    fn test_delete_version_not_found() {
        let mut manager = create_test_manager();
        let result = manager.delete_version("nonexistent");
        assert!(matches!(result, Err(OptimisticLockError::MemoryNotFound(_))));
    }

    #[test]
    fn test_versioned_memory_check_version() {
        let memory = VersionedMemory::new(
            "test".to_string(),
            "content".to_string(),
            "semantic".to_string(),
            0.8,
        );

        assert!(memory.check_version(1).is_ok());
        assert!(memory.check_version(2).is_err());
    }

    #[test]
    fn test_versioned_memory_update_content() {
        let mut memory = VersionedMemory::new(
            "test".to_string(),
            "old content".to_string(),
            "semantic".to_string(),
            0.8,
        );

        assert_eq!(memory.version, 1);
        memory.update_content("new content".to_string());
        assert_eq!(memory.content, "new content");
        assert_eq!(memory.version, 2);
    }

    #[test]
    fn test_version_info_comparison() {
        let info1 = VersionInfo {
            memory_id: "test".to_string(),
            version: 1,
            last_modified: Utc::now(),
            modified_by: None,
            change_description: None,
        };

        let mut info2 = info1.clone();
        info2.version = 2;

        assert_eq!(info1.compare_versions(&info2), VersionComparison::Older);
        assert_eq!(info2.compare_versions(&info1), VersionComparison::Newer);
        assert_eq!(info1.compare_versions(&info1), VersionComparison::Equal);
    }

    #[test]
    fn test_has_version() {
        let mut manager = create_test_manager();
        let memory_id = "test-memory-5";

        assert!(!manager.has_version(memory_id));
        manager.init_version(memory_id).unwrap();
        assert!(manager.has_version(memory_id));
    }

    #[test]
    fn test_tracked_memories() {
        let mut manager = create_test_manager();

        manager.init_version("memory-1").unwrap();
        manager.init_version("memory-2").unwrap();
        manager.init_version("memory-3").unwrap();

        let tracked = manager.tracked_memories();
        assert_eq!(tracked.len(), 3);
    }

    #[test]
    fn test_stats() {
        let mut manager = create_test_manager();
        manager.init_version("memory-1").unwrap();

        let stats = manager.stats();
        assert_eq!(stats.tracked_memories, 1);
    }
}