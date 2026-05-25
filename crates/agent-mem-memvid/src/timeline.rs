//! Time travel functionality for MemVid

use crate::error::Result;
use crate::store::MemvidStore;
use agent_mem_traits::{Memory, MemoryId, MetadataV4};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Version information for a memory
#[derive(Debug, Clone)]
pub struct VersionInfo {
    /// Memory ID
    pub memory_id: MemoryId,

    /// Version number
    pub version: u64,

    /// Timestamp of this version
    pub timestamp: DateTime<Utc>,

    /// Change description
    pub change: VersionChange,
}

/// Type of version change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VersionChange {
    /// Initial creation
    Created,

    /// Content updated
    Updated {
        /// What changed
        field: String,
        /// Old value (if available)
        old_value: Option<String>,
        /// New value
        new_value: String,
    },

    /// Memory deleted
    Deleted,

    /// Memory merged
    Merged {
        /// IDs of merged memories
        merged_ids: Vec<MemoryId>,
    },
}

/// Time travel interface for MemVid
///
/// Allows querying historical versions of memories,
/// rolling back to previous states, and auditing changes.
pub struct TimeTravel {
    store: Arc<MemvidStore>,
}

impl TimeTravel {
    /// Create a new time travel interface
    pub fn new(store: Arc<MemvidStore>) -> Self {
        Self { store }
    }

    /// Get a specific version of a memory at a given timestamp
    pub async fn get_version(
        &self,
        id: &MemoryId,
        timestamp: DateTime<Utc>,
    ) -> Result<Option<Memory>> {
        tracing::debug!("Getting version: memory_id={}, timestamp={}", id, timestamp);

        // TODO: Integrate with memvid-core time travel API
        // For now, return current version
        self.store.get(id).await
    }

    /// List all versions of a memory
    pub async fn list_versions(&self, id: &MemoryId) -> Result<Vec<VersionInfo>> {
        tracing::debug!("Listing versions for memory: {}", id);

        // TODO: Integrate with memvid-core version history API
        // For now, return current version only
        if let Some(memory) = self.store.get(id).await? {
            Ok(vec![VersionInfo {
                memory_id: id.clone(),
                version: 1,
                timestamp: memory.metadata.created_at,
                change: VersionChange::Created,
            }])
        } else {
            Ok(Vec::new())
        }
    }

    /// Rollback a memory to a specific version
    pub async fn rollback(&self, id: &MemoryId, to_timestamp: DateTime<Utc>) -> Result<Memory> {
        tracing::debug!("Rolling back: memory_id={}, timestamp={}", id, to_timestamp);

        // Get the version to rollback to
        let version = self
            .get_version(id, to_timestamp)
            .await?
            .ok_or_else(|| crate::error::MemvidError::VersionNotFound(id.to_string()))?;

        // Create a new version with the old content
        let mut rollback_memory = version.clone();
        rollback_memory.id = MemoryId::new(); // New ID for the rollback
        let mut metadata = MetadataV4::default();
        metadata.created_at = Utc::now();
        metadata.updated_at = Utc::now();
        rollback_memory.metadata = metadata;

        self.store.add(&rollback_memory).await?;

        tracing::info!("Rolled back memory: {} to {}", id, to_timestamp);
        Ok(rollback_memory)
    }

    /// Get timeline of changes between two timestamps
    pub async fn timeline(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<VersionInfo>> {
        tracing::debug!("Getting timeline: from={}, to={}", from, to);

        // TODO: Integrate with memvid-core timeline API
        // For now, return empty
        Ok(Vec::new())
    }

    /// Compare two versions of a memory
    pub async fn compare_versions(
        &self,
        id: &MemoryId,
        version1: u64,
        version2: u64,
    ) -> Result<MemoryDiff> {
        tracing::debug!(
            "Comparing versions: memory_id={}, v1={}, v2={}",
            id,
            version1,
            version2
        );

        // TODO: Implement actual comparison
        Ok(MemoryDiff {
            memory_id: id.clone(),
            version1,
            version2,
            changes: Vec::new(),
        })
    }

    /// Get change history for a memory
    pub async fn get_history(&self, id: &MemoryId) -> Result<Vec<HistoryEntry>> {
        tracing::debug!("Getting history for memory: {}", id);

        let versions = self.list_versions(id).await?;

        let history = versions
            .into_iter()
            .map(|v| {
                let description = Self::describe_change(&v.change);
                HistoryEntry {
                    timestamp: v.timestamp,
                    version: v.version,
                    change: v.change,
                    description,
                }
            })
            .collect();

        Ok(history)
    }

    /// Describe a change in human-readable form
    fn describe_change(change: &VersionChange) -> String {
        match change {
            VersionChange::Created => "Memory created".to_string(),
            VersionChange::Updated { field, .. } => {
                format!("Updated field: {}", field)
            }
            VersionChange::Deleted => "Memory deleted".to_string(),
            VersionChange::Merged { merged_ids } => {
                format!("Merged {} memories", merged_ids.len())
            }
        }
    }
}

/// Difference between two memory versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDiff {
    /// Memory ID
    pub memory_id: MemoryId,

    /// First version number
    pub version1: u64,

    /// Second version number
    pub version2: u64,

    /// List of changes
    pub changes: Vec<FieldChange>,
}

/// Change in a specific field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldChange {
    /// Field name
    pub field: String,

    /// Type of change
    pub change_type: ChangeType,

    /// Old value (if available)
    pub old_value: Option<String>,

    /// New value (if available)
    pub new_value: Option<String>,
}

/// Type of field change
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeType {
    /// Field was added
    Added,

    /// Field was removed
    Removed,

    /// Field was modified
    Modified,

    /// Field type changed
    TypeChanged,
}

/// History entry for a memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Timestamp of the change
    pub timestamp: DateTime<Utc>,

    /// Version number
    pub version: u64,

    /// Type of change
    pub change: VersionChange,

    /// Human-readable description
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MemvidConfig, store::MemvidStore};
    use agent_mem_traits::{AttributeSet, Content, MetadataV4};

    #[tokio::test]
    async fn test_time_travel() {
        let store = Arc::new(MemvidStore::create(MemvidConfig::new("test_timeline.mv2")).await.unwrap());
        let tt = TimeTravel::new(store.clone());

        let memory = Memory {
            id: MemoryId::from_string("test-timeline".to_string()),
            content: Content::text("Original content"),
            attributes: AttributeSet::new(),
            relations: Default::default(),
            metadata: MetadataV4::default(),
        };

        store.add(&memory).await.unwrap();

        // List versions
        let versions = tt.list_versions(&memory.id).await.unwrap();
        assert_eq!(versions.len(), 1);
        assert!(matches!(versions[0].change, VersionChange::Created));

        // Get history
        let history = tt.get_history(&memory.id).await.unwrap();
        assert_eq!(history.len(), 1);

        // Cleanup
        let _ = tokio::fs::remove_file("test_timeline.mv2").await;
    }
}
