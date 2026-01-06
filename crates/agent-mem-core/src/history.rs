//! Memory history tracking and versioning

use agent_mem_traits::{AgentMemError, MemoryV4, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Memory history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryHistoryEntry {
    /// Memory ID
    pub memory_id: String,
    /// Version number
    pub version: u32,
    /// Memory content at this version
    pub content: String,
    /// Importance at this version
    pub importance: f32,
    /// Metadata at this version
    pub metadata: HashMap<String, String>,
    /// Timestamp of this version
    pub timestamp: i64,
    /// Type of change
    pub change_type: ChangeType,
    /// Optional change description
    pub change_description: Option<String>,
}

/// Type of change made to memory
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ChangeType {
    Created,
    ContentUpdated,
    ImportanceChanged,
    MetadataUpdated,
    Accessed,
    Archived,
    Restored,
    Deprecated,
}

impl ChangeType {
    pub fn to_string(&self) -> String {
        match self {
            ChangeType::Created => "created".to_string(),
            ChangeType::ContentUpdated => "content_updated".to_string(),
            ChangeType::ImportanceChanged => "importance_changed".to_string(),
            ChangeType::MetadataUpdated => "metadata_updated".to_string(),
            ChangeType::Accessed => "accessed".to_string(),
            ChangeType::Archived => "archived".to_string(),
            ChangeType::Restored => "restored".to_string(),
            ChangeType::Deprecated => "deprecated".to_string(),
        }
    }
}

/// Memory history manager
pub struct MemoryHistory {
    /// History entries indexed by memory ID
    history: HashMap<String, Vec<MemoryHistoryEntry>>,
    /// Configuration for history retention
    config: HistoryConfig,
}

/// Configuration for memory history
#[derive(Debug, Clone)]
pub struct HistoryConfig {
    /// Maximum number of versions to keep per memory
    pub max_versions_per_memory: usize,
    /// Maximum age of history entries in seconds
    pub max_history_age_seconds: i64,
    /// Whether to track access events
    pub track_access_events: bool,
    /// Whether to compress old history entries
    pub compress_old_entries: bool,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            max_versions_per_memory: 50,
            max_history_age_seconds: 365 * 24 * 3600, // 1 year
            track_access_events: false,               // Usually too noisy
            compress_old_entries: true,
        }
    }
}

impl MemoryHistory {
    /// Create a new memory history manager
    pub fn new(config: HistoryConfig) -> Self {
        Self {
            history: HashMap::new(),
            config,
        }
    }

    /// Create with default configuration
    pub fn with_default_config() -> Self {
        Self::new(HistoryConfig::default())
    }

    /// Record memory creation
    pub fn record_creation(&mut self, memory: &MemoryV4) -> Result<()> {
        let entry = MemoryHistoryEntry {
            memory_id: memory.id.0.clone(),
            version: memory.version(),
            content: memory.content.to_string(),
            importance: memory.importance().unwrap_or(0.5) as f32,
            metadata: memory.metadata.to_hashmap(),
            timestamp: memory.created_at().timestamp(),
            change_type: ChangeType::Created,
            change_description: Some("Memory created".to_string()),
        };

        self.add_history_entry(entry);
        Ok(())
    }

    /// Record memory content update
    pub fn record_content_update(
        &mut self,
        memory: &MemoryV4,
        old_content: &str,
        change_description: Option<String>,
    ) -> Result<()> {
        let entry = MemoryHistoryEntry {
            memory_id: memory.id.0.clone(),
            version: memory.version(),
            content: memory.content.to_string(),
            importance: memory.importance().unwrap_or(0.5) as f32,
            metadata: memory.metadata.to_hashmap(),
            timestamp: chrono::Utc::now().timestamp(),
            change_type: ChangeType::ContentUpdated,
            change_description: change_description.or_else(|| {
                Some(format!(
                    "Content updated from '{}' to '{}'",
                    self.truncate_text(old_content, 50),
                    self.truncate_text(&memory.content.to_string(), 50)
                ))
            }),
        };

        self.add_history_entry(entry);
        Ok(())
    }

    /// Record importance change
    pub fn record_importance_change(
        &mut self,
        memory: &MemoryV4,
        old_importance: f32,
    ) -> Result<()> {
        let new_importance = memory.importance().unwrap_or(0.5) as f32;
        let entry = MemoryHistoryEntry {
            memory_id: memory.id.0.clone(),
            version: memory.version(),
            content: memory.content.to_string(),
            importance: new_importance,
            metadata: memory.metadata.to_hashmap(),
            timestamp: chrono::Utc::now().timestamp(),
            change_type: ChangeType::ImportanceChanged,
            change_description: Some(format!(
                "Importance changed from {old_importance:.2} to {new_importance:.2}"
            )),
        };

        self.add_history_entry(entry);
        Ok(())
    }

    /// Record metadata update
    pub fn record_metadata_update(
        &mut self,
        memory: &MemoryV4,
        changed_keys: Vec<String>,
    ) -> Result<()> {
        let entry = MemoryHistoryEntry {
            memory_id: memory.id.0.clone(),
            version: memory.version(),
            content: memory.content.to_string(),
            importance: memory.importance().unwrap_or(0.5) as f32,
            metadata: memory.metadata.to_hashmap(),
            timestamp: chrono::Utc::now().timestamp(),
            change_type: ChangeType::MetadataUpdated,
            change_description: Some(format!("Metadata updated: {}", changed_keys.join(", "))),
        };

        self.add_history_entry(entry);
        Ok(())
    }

    /// Record memory access (if enabled)
    pub fn record_access(&mut self, memory: &MemoryV4) -> Result<()> {
        if !self.config.track_access_events {
            return Ok(());
        }

        let entry = MemoryHistoryEntry {
            memory_id: memory.id.0.clone(),
            version: memory.version(),
            content: memory.content.to_string(),
            importance: memory.importance().unwrap_or(0.5) as f32,
            metadata: memory.metadata.to_hashmap(),
            timestamp: chrono::Utc::now().timestamp(),
            change_type: ChangeType::Accessed,
            change_description: Some(format!(
                "Memory accessed (count: {})",
                memory.metadata.access_count
            )),
        };

        self.add_history_entry(entry);
        Ok(())
    }

    /// Record memory archival
    pub fn record_archival(&mut self, memory: &MemoryV4) -> Result<()> {
        let entry = MemoryHistoryEntry {
            memory_id: memory.id.0.clone(),
            version: memory.version(),
            content: memory.content.to_string(),
            importance: memory.importance().unwrap_or(0.5) as f32,
            metadata: memory.metadata.to_hashmap(),
            timestamp: chrono::Utc::now().timestamp(),
            change_type: ChangeType::Archived,
            change_description: Some("Memory archived".to_string()),
        };

        self.add_history_entry(entry);
        Ok(())
    }

    /// Record memory restoration
    pub fn record_restoration(&mut self, memory: &MemoryV4) -> Result<()> {
        let entry = MemoryHistoryEntry {
            memory_id: memory.id.0.clone(),
            version: memory.version(),
            content: memory.content.to_string(),
            importance: memory.importance().unwrap_or(0.5) as f32,
            metadata: memory.metadata.to_hashmap(),
            timestamp: chrono::Utc::now().timestamp(),
            change_type: ChangeType::Restored,
            change_description: Some("Memory restored from archive".to_string()),
        };

        self.add_history_entry(entry);
        Ok(())
    }

    /// Get history for a specific memory
    pub fn get_memory_history(&self, memory_id: &str) -> Option<&Vec<MemoryHistoryEntry>> {
        self.history.get(memory_id)
    }

    /// Get specific version of a memory
    pub fn get_memory_version(&self, memory_id: &str, version: u32) -> Option<&MemoryHistoryEntry> {
        self.history
            .get(memory_id)?
            .iter()
            .find(|entry| entry.version == version)
    }

    /// Get latest version of a memory from history
    pub fn get_latest_version(&self, memory_id: &str) -> Option<&MemoryHistoryEntry> {
        self.history
            .get(memory_id)?
            .iter()
            .max_by_key(|entry| entry.version)
    }

    /// Get changes between two versions
    pub fn get_version_diff(
        &self,
        memory_id: &str,
        from_version: u32,
        to_version: u32,
    ) -> Result<VersionDiff> {
        let history = self
            .history
            .get(memory_id)
            .ok_or_else(|| AgentMemError::memory_error("Memory history not found"))?;

        let from_entry = history
            .iter()
            .find(|entry| entry.version == from_version)
            .ok_or_else(|| AgentMemError::memory_error("From version not found"))?;

        let to_entry = history
            .iter()
            .find(|entry| entry.version == to_version)
            .ok_or_else(|| AgentMemError::memory_error("To version not found"))?;

        Ok(VersionDiff {
            memory_id: memory_id.to_string(),
            from_version,
            to_version,
            content_changed: from_entry.content != to_entry.content,
            importance_changed: from_entry.importance != to_entry.importance,
            metadata_changed: from_entry.metadata != to_entry.metadata,
            time_diff_seconds: to_entry.timestamp - from_entry.timestamp,
        })
    }

    /// Clean up old history entries
    pub fn cleanup_old_entries(&mut self) -> usize {
        let current_time = chrono::Utc::now().timestamp();
        let cutoff_time = current_time - self.config.max_history_age_seconds;
        let mut removed_count = 0;

        for (_, entries) in self.history.iter_mut() {
            let original_len = entries.len();
            entries.retain(|entry| entry.timestamp > cutoff_time);
            removed_count += original_len - entries.len();

            // Also enforce max versions limit
            if entries.len() > self.config.max_versions_per_memory {
                // Keep the most recent versions
                entries.sort_by_key(|entry| entry.timestamp);
                let excess = entries.len() - self.config.max_versions_per_memory;
                entries.drain(0..excess);
                removed_count += excess;
            }
        }

        // Remove empty history entries
        self.history.retain(|_, entries| !entries.is_empty());

        removed_count
    }

    /// Get history statistics
    pub fn get_history_stats(&self) -> HistoryStats {
        let mut stats = HistoryStats::default();

        stats.total_memories_tracked = self.history.len();

        for entries in self.history.values() {
            stats.total_history_entries += entries.len();

            for entry in entries {
                *stats
                    .changes_by_type
                    .entry(entry.change_type.clone())
                    .or_insert(0) += 1;
            }
        }

        if !self.history.is_empty() {
            stats.average_versions_per_memory =
                stats.total_history_entries as f32 / stats.total_memories_tracked as f32;
        }

        stats
    }

    /// Add a history entry
    fn add_history_entry(&mut self, entry: MemoryHistoryEntry) {
        let memory_id = entry.memory_id.clone();
        self.history
            .entry(memory_id.clone())
            .or_default()
            .push(entry);

        // Enforce limits immediately
        if let Some(entries) = self.history.get_mut(&memory_id) {
            if entries.len() > self.config.max_versions_per_memory {
                // Remove oldest entries
                entries.sort_by_key(|e| e.timestamp);
                let excess = entries.len() - self.config.max_versions_per_memory;
                entries.drain(0..excess);
            }
        }
    }

    /// Truncate text for display
    fn truncate_text(&self, text: &str, max_len: usize) -> String {
        if text.len() <= max_len {
            text.to_string()
        } else {
            format!("{}...", &text[..max_len])
        }
    }
}

/// Difference between two memory versions
#[derive(Debug, Clone)]
pub struct VersionDiff {
    pub memory_id: String,
    pub from_version: u32,
    pub to_version: u32,
    pub content_changed: bool,
    pub importance_changed: bool,
    pub metadata_changed: bool,
    pub time_diff_seconds: i64,
}

/// History statistics
#[derive(Debug, Clone, Default)]
pub struct HistoryStats {
    pub total_memories_tracked: usize,
    pub total_history_entries: usize,
    pub average_versions_per_memory: f32,
    pub changes_by_type: HashMap<ChangeType, usize>,
}
