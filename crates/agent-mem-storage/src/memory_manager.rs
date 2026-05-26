//! Memory Management Module for AgentMem
//!
//! Provides intelligent memory management features:
//! - Automatic importance scoring
//! - Memory consolidation
//! - Memory decay mechanisms
//! - Priority queues

use agent_mem_traits::{AgentMemError, MemoryV4, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

/// Configuration for memory importance scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportanceConfig {
    /// Enable automatic importance calculation
    pub auto_calculate: bool,
    /// Base importance for new memories
    pub base_importance: f32,
    /// Maximum importance score
    pub max_importance: f32,
    /// Minimum importance score
    pub min_importance: f32,
    /// Access boost factor
    pub access_boost: f32,
    /// Access boost decay rate
    pub access_boost_decay: f32,
    /// Importance update interval (seconds)
    pub update_interval_seconds: u64,
}

impl Default for ImportanceConfig {
    fn default() -> Self {
        Self {
            auto_calculate: true,
            base_importance: 0.5,
            max_importance: 1.0,
            min_importance: 0.0,
            access_boost: 0.1,
            access_boost_decay: 0.95,
            update_interval_seconds: 300,
        }
    }
}

/// Configuration for memory decay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecayConfig {
    /// Enable automatic decay
    pub enabled: bool,
    /// Decay rate per day (0.0 - 1.0)
    pub decay_rate_per_day: f32,
    /// Minimum decay threshold
    pub min_decay_threshold: f32,
    /// Decay check interval (seconds)
    pub check_interval_seconds: u64,
}

impl Default for DecayConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            decay_rate_per_day: 0.05, // 5% decay per day
            min_decay_threshold: 0.1,
            check_interval_seconds: 3600,
        }
    }
}

/// Configuration for memory consolidation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationConfig {
    /// Enable automatic consolidation
    pub enabled: bool,
    /// Maximum memories before consolidation
    pub max_memories: usize,
    /// Minimum importance to keep
    pub min_importance_threshold: f32,
    /// Consolidation batch size
    pub batch_size: usize,
    /// Consolidation interval (seconds)
    pub interval_seconds: u64,
}

impl Default for ConsolidationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_memories: 100000,
            min_importance_threshold: 0.2,
            batch_size: 100,
            interval_seconds: 86400, // 1 day
        }
    }
}

/// Memory with calculated importance
#[derive(Debug, Clone)]
pub struct MemoryWithImportance {
    pub id: String,
    pub content: String,
    pub importance: f32,
    pub access_count: u64,
    pub created_at: u64,
    pub last_accessed: u64,
}

/// Importance calculation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportanceFactors {
    pub base: f32,
    pub recency: f32,
    pub access: f32,
    pub content: f32,
    pub total: f32,
}

/// Memory manager for intelligent memory handling
pub struct MemoryManager {
    config: MemoryManagementConfig,
    importance_calculator: ImportanceCalculator,
    stats: Arc<RwLock<MemoryManagerStats>>,
}

/// Statistics for memory management
#[derive(Debug, Clone, Default)]
pub struct MemoryManagerStats {
    pub memories_processed: u64,
    pub importance_updates: u64,
    pub decayed_memories: u64,
    pub consolidated_memories: u64,
    pub total_importance_changes: f32,
}

/// Complete memory management configuration
#[derive(Debug, Clone)]
pub struct MemoryManagementConfig {
    pub importance: ImportanceConfig,
    pub decay: DecayConfig,
    pub consolidation: ConsolidationConfig,
}

impl Default for MemoryManagementConfig {
    fn default() -> Self {
        Self {
            importance: ImportanceConfig::default(),
            decay: DecayConfig::default(),
            consolidation: ConsolidationConfig::default(),
        }
    }
}

/// Importance calculator
#[derive(Debug, Clone)]
pub struct ImportanceCalculator {
    config: ImportanceConfig,
}

impl ImportanceCalculator {
    pub fn new(config: ImportanceConfig) -> Self {
        Self { config }
    }
    
    /// Calculate importance based on various factors
    pub fn calculate(&self, memory: &MemoryWithImportance) -> ImportanceFactors {
        // Base importance
        let base = self.config.base_importance;
        
        // Recency factor (exponential decay based on age)
        let age_hours = Self::hours_since(memory.created_at);
        let recency_decay_rate = 0.01_f32;
        let recency = (-recency_decay_rate * age_hours as f32).exp().max(0.0).min(1.0);
        
        // Access factor (boost based on recent access)
        let last_access_hours = Self::hours_since(memory.last_accessed);
        let access_decay_rate = 0.1_f32;
        let access_boost = if memory.access_count > 0 {
            let boost = memory.access_count as f32 * self.config.access_boost;
            let decay = (-access_decay_rate * last_access_hours as f32).exp();
            boost * decay
        } else {
            0.0
        };
        
        // Content factor (based on content length - longer content slightly more important)
        let content_factor = (memory.content.len() as f32 / 1000.0).min(1.0) * 0.1;
        
        // Calculate total
        let total = (base + recency * 0.3 + access_boost + content_factor)
            .clamp(self.config.min_importance, self.config.max_importance);
        
        ImportanceFactors {
            base,
            recency,
            access: access_boost,
            content: content_factor,
            total,
        }
    }
    
    /// Calculate hours since timestamp
    fn hours_since(timestamp: u64) -> f64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        ((now - timestamp) as f64) / 3600.0
    }
}

impl Default for ImportanceCalculator {
    fn default() -> Self {
        Self::new(ImportanceConfig::default())
    }
}

/// Decay calculator for memory importance
pub struct DecayCalculator {
    config: DecayConfig,
}

impl DecayCalculator {
    pub fn new(config: DecayConfig) -> Self {
        Self { config }
    }
    
    /// Calculate decayed importance
    pub fn calculate(&self, current_importance: f32, age_days: u64) -> f32 {
        if !self.config.enabled {
            return current_importance;
        }
        
        // Exponential decay
        let decay_factor = (self.config.decay_rate_per_day * age_days as f32).min(1.0);
        let new_importance = current_importance * (1.0 - decay_factor);
        
        // Don't go below threshold
        new_importance.max(self.config.min_decay_threshold)
    }
    
    /// Check if memory should be deleted based on decay
    pub fn should_delete(&self, importance: f32) -> bool {
        importance <= self.config.min_decay_threshold
    }
}

impl Default for DecayCalculator {
    fn default() -> Self {
        Self::new(DecayConfig::default())
    }
}

/// Memory priority queue for efficient retrieval
#[derive(Debug, Clone)]
pub struct MemoryPriorityQueue {
    memories: Vec<MemoryWithImportance>,
    capacity: usize,
}

impl MemoryPriorityQueue {
    pub fn new(capacity: usize) -> Self {
        Self {
            memories: Vec::with_capacity(capacity),
            capacity,
        }
    }
    
    /// Add memory to queue (maintains priority order)
    pub fn push(&mut self, memory: MemoryWithImportance) {
        // Find insertion position
        let pos = self.memories.iter()
            .position(|m| m.importance < memory.importance)
            .unwrap_or(self.memories.len());
        
        self.memories.insert(pos, memory);
        
        // Trim to capacity
        if self.memories.len() > self.capacity {
            self.memories.truncate(self.capacity);
        }
    }
    
    /// Get top N memories by importance
    pub fn top(&self, n: usize) -> Vec<&MemoryWithImportance> {
        self.memories.iter().take(n).collect()
    }
    
    /// Get memories below threshold
    pub fn below_threshold(&self, threshold: f32) -> Vec<&MemoryWithImportance> {
        self.memories.iter()
            .filter(|m| m.importance < threshold)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_importance_calculation() {
        let config = ImportanceConfig::default();
        let calculator = ImportanceCalculator::new(config);
        
        let memory = MemoryWithImportance {
            id: "test".to_string(),
            content: "Test content".to_string(),
            importance: 0.5,
            access_count: 5,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() - 86400, // 1 day ago
            last_accessed: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        let factors = calculator.calculate(&memory);
        assert!(factors.total >= 0.0);
        assert!(factors.total <= 1.0);
    }

    #[test]
    fn test_decay_calculation() {
        let config = DecayConfig::default();
        let calculator = DecayCalculator::new(config);
        
        // 1 day old memory with 0.5 importance
        let decayed = calculator.calculate(0.5, 1);
        assert!(decayed < 0.5);
        assert!(decayed >= 0.1);
    }

    #[test]
    fn test_priority_queue() {
        let mut queue = MemoryPriorityQueue::new(3);
        
        queue.push(MemoryWithImportance {
            id: "1".to_string(),
            content: "Low".to_string(),
            importance: 0.2,
            access_count: 0,
            created_at: 0,
            last_accessed: 0,
        });
        
        queue.push(MemoryWithImportance {
            id: "2".to_string(),
            content: "High".to_string(),
            importance: 0.8,
            access_count: 0,
            created_at: 0,
            last_accessed: 0,
        });
        
        queue.push(MemoryWithImportance {
            id: "3".to_string(),
            content: "Medium".to_string(),
            importance: 0.5,
            access_count: 0,
            created_at: 0,
            last_accessed: 0,
        });
        
        let top = queue.top(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].id, "2"); // High importance first
    }
}
