//! Memory Hierarchy Module
//! 
//! 实现层级记忆管理，参考 MemGPT 的层级架构

use serde::{Deserialize, Serialize};

/// 记忆层级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryTier {
    Working,  // 工作记忆
    Core,     // 核心记忆
    Archive,  // 归档记忆
}

impl MemoryTier {
    pub fn description(&self) -> &'static str {
        match self {
            MemoryTier::Working => "短期记忆",
            MemoryTier::Core => "核心记忆",
            MemoryTier::Archive => "归档记忆",
        }
    }
}

/// 记忆层级项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TieredMemoryItem {
    pub id: String,
    pub content: String,
    pub tier: MemoryTier,
    pub importance: f32,
    pub access_count: u32,
    pub last_accessed: i64,
}

impl TieredMemoryItem {
    pub fn new(id: String, content: String, tier: MemoryTier) -> Self {
        Self {
            id,
            content,
            tier,
            importance: 0.5,
            access_count: 0,
            last_accessed: chrono::Utc::now().timestamp(),
        }
    }
    
    pub fn access(&mut self) {
        self.access_count += 1;
        self.last_accessed = chrono::Utc::now().timestamp();
    }
}

/// 层级记忆管理器
pub struct MemoryHierarchy {
    working: Vec<TieredMemoryItem>,
    core: Vec<TieredMemoryItem>,
    archive: Vec<TieredMemoryItem>,
    working_capacity: usize,
    core_capacity: usize,
}

impl MemoryHierarchy {
    pub fn new(working_cap: usize, core_cap: usize) -> Self {
        Self {
            working: Vec::new(),
            core: Vec::new(),
            archive: Vec::new(),
            working_capacity: working_cap,
            core_capacity: core_cap,
        }
    }
    
    pub fn with_defaults() -> Self {
        Self::new(100, 1000)
    }
    
    pub fn add(&mut self, item: TieredMemoryItem) {
        match item.tier {
            MemoryTier::Working => {
                if self.working.len() >= self.working_capacity
                    && !self.working.is_empty() {
                        let mut evicted = self.working.remove(0);
                        evicted.tier = MemoryTier::Archive;
                        self.archive.push(evicted);
                    }
                self.working.push(item);
            }
            MemoryTier::Core => {
                if self.core.len() >= self.core_capacity
                    && !self.core.is_empty() {
                        let mut evicted = self.core.remove(0);
                        evicted.tier = MemoryTier::Archive;
                        self.archive.push(evicted);
                    }
                self.core.push(item);
            }
            MemoryTier::Archive => {
                self.archive.push(item);
            }
        }
    }
    
    pub fn access(&mut self, id: &str) -> Option<&mut TieredMemoryItem> {
        if let Some(item) = self.working.iter_mut().find(|i| i.id == id) {
            item.access();
            return Some(item);
        }
        if let Some(item) = self.core.iter_mut().find(|i| i.id == id) {
            item.access();
            return Some(item);
        }
        self.archive.iter_mut().find(|i| i.id == id)
    }
    
    pub fn get_tier(&self, tier: MemoryTier) -> &[TieredMemoryItem] {
        match tier {
            MemoryTier::Working => &self.working,
            MemoryTier::Core => &self.core,
            MemoryTier::Archive => &self.archive,
        }
    }
    
    pub fn stats(&self) -> MemoryHierarchyStats {
        MemoryHierarchyStats {
            working_count: self.working.len(),
            working_capacity: self.working_capacity,
            core_count: self.core.len(),
            core_capacity: self.core_capacity,
            archive_count: self.archive.len(),
        }
    }
    
    /// Get capacity limits
    pub fn working_capacity(&self) -> usize {
        self.working_capacity
    }
    
    pub fn core_capacity(&self) -> usize {
        self.core_capacity
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryHierarchyStats {
    pub working_count: usize,
    pub working_capacity: usize,
    pub core_count: usize,
    pub core_capacity: usize,
    pub archive_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let mut h = MemoryHierarchy::with_defaults();
        h.add(TieredMemoryItem::new("1".to_string(), "test".to_string(), MemoryTier::Working));
        assert_eq!(h.stats().working_count, 1);
    }
    
    #[test]
    fn test_capacity() {
        let mut h = MemoryHierarchy::new(2, 2);
        h.add(TieredMemoryItem::new("1".to_string(), "a".to_string(), MemoryTier::Working));
        h.add(TieredMemoryItem::new("2".to_string(), "b".to_string(), MemoryTier::Working));
        h.add(TieredMemoryItem::new("3".to_string(), "c".to_string(), MemoryTier::Working));
        assert!(h.stats().working_count <= 2);
        assert!(h.stats().archive_count >= 1);
    }
}
