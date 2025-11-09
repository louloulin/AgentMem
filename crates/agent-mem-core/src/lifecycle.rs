//! Memory lifecycle management

use agent_mem_traits::MemoryType;
use agent_mem_traits::{AgentMemError, Result};
use std::collections::HashMap;

/// Memory lifecycle states
#[derive(Debug, Clone, PartialEq)]
pub enum MemoryState {
    /// Memory is newly created
    Created,
    /// Memory is active and being used
    Active,
    /// Memory is archived but still accessible
    Archived,
    /// Memory is marked for deletion
    Deprecated,
    /// Memory is permanently deleted
    Deleted,
}

/// Memory lifecycle event
#[derive(Debug, Clone)]
pub struct LifecycleEvent {
    pub memory_id: String,
    pub event_type: LifecycleEventType,
    pub timestamp: i64,
    pub metadata: HashMap<String, String>,
}

/// Types of lifecycle events
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LifecycleEventType {
    Created,
    Accessed,
    Updated,
    Archived,
    Restored,
    Deprecated,
    Deleted,
    ImportanceChanged,
    /// Memory expiration time has been set
    ExpirationSet,
}

/// Memory lifecycle manager
pub struct MemoryLifecycle {
    /// Current state of memories
    memory_states: HashMap<String, MemoryState>,
    /// Lifecycle events history
    events: Vec<LifecycleEvent>,
    /// Configuration for lifecycle policies
    config: LifecycleConfig,
}

/// Configuration for memory lifecycle policies
#[derive(Debug, Clone)]
pub struct LifecycleConfig {
    /// Auto-archive memories older than this (seconds)
    pub auto_archive_age: Option<i64>,
    /// Auto-delete memories older than this (seconds)
    pub auto_delete_age: Option<i64>,
    /// Minimum importance to prevent auto-archiving
    pub archive_importance_threshold: f32,
    /// Minimum importance to prevent auto-deletion
    pub delete_importance_threshold: f32,
    /// Maximum number of events to keep in history
    pub max_events_history: usize,
    /// Auto-expire working memories after this duration (seconds)
    pub working_memory_ttl: i64,
}

impl Default for LifecycleConfig {
    fn default() -> Self {
        Self {
            auto_archive_age: Some(30 * 24 * 3600), // 30 days
            auto_delete_age: Some(365 * 24 * 3600), // 1 year
            archive_importance_threshold: 0.3,
            delete_importance_threshold: 0.1,
            max_events_history: 10000,
            working_memory_ttl: 24 * 3600, // 1 day
        }
    }
}

impl MemoryLifecycle {
    /// Create a new lifecycle manager
    pub fn new(config: LifecycleConfig) -> Self {
        Self {
            memory_states: HashMap::new(),
            events: Vec::new(),
            config,
        }
    }

    /// Create a new lifecycle manager with default config
    pub fn with_default_config() -> Self {
        Self::new(LifecycleConfig::default())
    }

    /// Register a new memory
    pub fn register_memory(&mut self, memory: &agent_mem_traits::MemoryItem) -> Result<()> {
        self.memory_states
            .insert(memory.id.clone(), MemoryState::Created);

        let event = LifecycleEvent {
            memory_id: memory.id.clone(),
            event_type: LifecycleEventType::Created,
            timestamp: chrono::Utc::now().timestamp(),
            metadata: HashMap::new(),
        };

        self.add_event(event);

        // Set expiration for working memories
        if memory.memory_type == MemoryType::Working {
            let expiration = chrono::Utc::now().timestamp() + self.config.working_memory_ttl;
            self.set_expiration(&memory.id, expiration)?;
        }

        Ok(())
    }

    /// Record memory access
    pub fn record_access(&mut self, memory_id: &str) -> Result<()> {
        if let Some(state) = self.memory_states.get(memory_id) {
            if *state == MemoryState::Deleted {
                return Err(AgentMemError::memory_error("Memory is deleted"));
            }
        }

        // Update state to active if it was archived
        if let Some(state) = self.memory_states.get_mut(memory_id) {
            if *state == MemoryState::Archived {
                *state = MemoryState::Active;

                let event = LifecycleEvent {
                    memory_id: memory_id.to_string(),
                    event_type: LifecycleEventType::Restored,
                    timestamp: chrono::Utc::now().timestamp(),
                    metadata: HashMap::new(),
                };
                self.add_event(event);
            }
        }

        let event = LifecycleEvent {
            memory_id: memory_id.to_string(),
            event_type: LifecycleEventType::Accessed,
            timestamp: chrono::Utc::now().timestamp(),
            metadata: HashMap::new(),
        };

        self.add_event(event);
        Ok(())
    }

    /// Record memory update
    pub fn record_update(
        &mut self,
        memory_id: &str,
        old_version: u32,
        new_version: u32,
    ) -> Result<()> {
        let mut metadata = HashMap::new();
        metadata.insert("old_version".to_string(), old_version.to_string());
        metadata.insert("new_version".to_string(), new_version.to_string());

        let event = LifecycleEvent {
            memory_id: memory_id.to_string(),
            event_type: LifecycleEventType::Updated,
            timestamp: chrono::Utc::now().timestamp(),
            metadata,
        };

        self.add_event(event);
        Ok(())
    }

    /// Archive a memory
    pub fn archive_memory(&mut self, memory_id: &str) -> Result<()> {
        if let Some(state) = self.memory_states.get_mut(memory_id) {
            if *state == MemoryState::Deleted {
                return Err(AgentMemError::memory_error("Cannot archive deleted memory"));
            }
            *state = MemoryState::Archived;
        } else {
            return Err(AgentMemError::memory_error("Memory not found"));
        }

        let event = LifecycleEvent {
            memory_id: memory_id.to_string(),
            event_type: LifecycleEventType::Archived,
            timestamp: chrono::Utc::now().timestamp(),
            metadata: HashMap::new(),
        };

        self.add_event(event);
        Ok(())
    }

    /// Restore an archived memory
    pub fn restore_memory(&mut self, memory_id: &str) -> Result<()> {
        if let Some(state) = self.memory_states.get_mut(memory_id) {
            if *state != MemoryState::Archived {
                return Err(AgentMemError::memory_error("Memory is not archived"));
            }
            *state = MemoryState::Active;
        } else {
            return Err(AgentMemError::memory_error("Memory not found"));
        }

        let event = LifecycleEvent {
            memory_id: memory_id.to_string(),
            event_type: LifecycleEventType::Restored,
            timestamp: chrono::Utc::now().timestamp(),
            metadata: HashMap::new(),
        };

        self.add_event(event);
        Ok(())
    }

    /// Mark memory as deprecated
    pub fn deprecate_memory(&mut self, memory_id: &str) -> Result<()> {
        if let Some(state) = self.memory_states.get_mut(memory_id) {
            if *state == MemoryState::Deleted {
                return Err(AgentMemError::memory_error("Memory is already deleted"));
            }
            *state = MemoryState::Deprecated;
        } else {
            return Err(AgentMemError::memory_error("Memory not found"));
        }

        let event = LifecycleEvent {
            memory_id: memory_id.to_string(),
            event_type: LifecycleEventType::Deprecated,
            timestamp: chrono::Utc::now().timestamp(),
            metadata: HashMap::new(),
        };

        self.add_event(event);
        Ok(())
    }

    /// Delete a memory
    pub fn delete_memory(&mut self, memory_id: &str) -> Result<()> {
        if let Some(state) = self.memory_states.get_mut(memory_id) {
            *state = MemoryState::Deleted;
        } else {
            return Err(AgentMemError::memory_error("Memory not found"));
        }

        let event = LifecycleEvent {
            memory_id: memory_id.to_string(),
            event_type: LifecycleEventType::Deleted,
            timestamp: chrono::Utc::now().timestamp(),
            metadata: HashMap::new(),
        };

        self.add_event(event);
        Ok(())
    }

    /// Set expiration for a memory
    pub fn set_expiration(&mut self, memory_id: &str, expires_at: i64) -> Result<()> {
        let mut metadata = HashMap::new();
        metadata.insert("expires_at".to_string(), expires_at.to_string());

        let event = LifecycleEvent {
            memory_id: memory_id.to_string(),
            event_type: LifecycleEventType::ExpirationSet,
            timestamp: chrono::Utc::now().timestamp(),
            metadata,
        };

        self.add_event(event);
        Ok(())
    }

    /// Get current state of a memory
    pub fn get_memory_state(&self, memory_id: &str) -> Option<&MemoryState> {
        self.memory_states.get(memory_id)
    }

    /// Check if memory is accessible (not deleted)
    pub fn is_accessible(&self, memory_id: &str) -> bool {
        match self.memory_states.get(memory_id) {
            Some(MemoryState::Deleted) => false,
            Some(_) => true,
            None => false,
        }
    }

    /// Get lifecycle events for a memory
    pub fn get_memory_events(&self, memory_id: &str) -> Vec<&LifecycleEvent> {
        self.events
            .iter()
            .filter(|event| event.memory_id == memory_id)
            .collect()
    }

    /// Apply automatic lifecycle policies
    pub fn apply_auto_policies(
        &mut self,
        memories: &[agent_mem_traits::MemoryItem],
    ) -> Result<Vec<String>> {
        let current_time = chrono::Utc::now().timestamp();
        let mut affected_memories = Vec::new();

        for memory in memories {
            // Skip if already deleted
            if let Some(MemoryState::Deleted) = self.memory_states.get(&memory.id) {
                continue;
            }

            let age = current_time - memory.created_at.timestamp();
            let current_importance = memory.score.unwrap_or(0.5);

            // Auto-delete policy
            if let Some(delete_age) = self.config.auto_delete_age {
                if age > delete_age && current_importance < self.config.delete_importance_threshold
                {
                    self.delete_memory(&memory.id)?;
                    affected_memories.push(memory.id.clone());
                    continue;
                }
            }

            // Auto-archive policy
            if let Some(archive_age) = self.config.auto_archive_age {
                if age > archive_age
                    && current_importance < self.config.archive_importance_threshold
                {
                    if let Some(state) = self.memory_states.get(&memory.id) {
                        if *state == MemoryState::Active || *state == MemoryState::Created {
                            self.archive_memory(&memory.id)?;
                            affected_memories.push(memory.id.clone());
                        }
                    }
                }
            }
        }

        Ok(affected_memories)
    }

    /// Add an event to the history
    fn add_event(&mut self, event: LifecycleEvent) {
        self.events.push(event);

        // Trim events if we exceed the maximum
        if self.events.len() > self.config.max_events_history {
            let excess = self.events.len() - self.config.max_events_history;
            self.events.drain(0..excess);
        }
    }

    /// Get statistics about lifecycle events
    pub fn get_lifecycle_stats(&self) -> HashMap<LifecycleEventType, usize> {
        let mut stats = HashMap::new();

        for event in &self.events {
            *stats.entry(event.event_type.clone()).or_insert(0) += 1;
        }

        stats
    }

    /// Clean up old events
    pub fn cleanup_old_events(&mut self, max_age_seconds: i64) {
        let cutoff_time = chrono::Utc::now().timestamp() - max_age_seconds;
        self.events.retain(|event| event.timestamp > cutoff_time);
    }
}
