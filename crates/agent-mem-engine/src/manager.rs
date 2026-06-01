//! Memory Manager - Core memory management operations
//! 
//! Provides the MemoryManager for handling memory operations.

use agent_mem_types::{MemoryType, Result};
use std::collections::HashMap;
use std::sync::RwLock;

/// MemoryManager handles core memory management operations
/// 
/// This includes:
/// - Memory CRUD operations
/// - Memory type management
/// - Memory statistics
/// - Memory lifecycle management
pub struct MemoryManager {
    memories: RwLock<HashMap<String, MemoryItem>>,
    stats: RwLock<MemoryStats>,
}

/// Simple memory item representation
#[derive(Debug, Clone)]
pub struct MemoryItem {
    pub id: String,
    pub memory_type: String,
    pub content: String,
}

#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    pub total_memories: usize,
    pub by_type: HashMap<String, usize>,
}

impl MemoryManager {
    /// Create new memory manager
    pub fn new() -> Self {
        Self {
            memories: RwLock::new(HashMap::new()),
            stats: RwLock::new(MemoryStats::default()),
        }
    }

    /// Get total memory count
    pub fn total_count(&self) -> usize {
        self.memories.read().unwrap().len()
    }

    /// Get memory count by type
    pub fn count_by_type(&self, memory_type: &str) -> usize {
        self.memories
            .read()
            .unwrap()
            .values()
            .filter(|m| m.memory_type == memory_type)
            .count()
    }

    /// Get all memory types
    pub fn available_types(&self) -> Vec<MemoryType> {
        MemoryType::all_types()
    }

    /// Clear all memories
    pub fn clear(&self) {
        self.memories.write().unwrap().clear();
        self.stats.write().unwrap().total_memories = 0;
        self.stats.write().unwrap().by_type.clear();
    }

    /// Get statistics
    pub fn stats(&self) -> MemoryStats {
        self.stats.read().unwrap().clone()
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_creation() {
        let manager = MemoryManager::new();
        assert_eq!(manager.total_count(), 0);
    }

    #[test]
    fn test_manager_types() {
        let manager = MemoryManager::new();
        let types = manager.available_types();
        assert_eq!(types.len(), 8);
    }
}
