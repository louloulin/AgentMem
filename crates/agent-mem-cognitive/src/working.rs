//! Working Memory Module
//! 
//! Working memory handles temporary active processing information.

use std::collections::VecDeque;
use serde::{Deserialize, Serialize};

/// Working memory item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingItem {
    /// Content
    pub content: String,
    /// Importance
    pub importance: f32,
    /// Expiry timestamp
    pub expires_at: Option<i64>,
}

impl WorkingItem {
    pub fn new(content: String) -> Self {
        Self {
            content,
            importance: 0.5,
            expires_at: None,
        }
    }
}

/// Working memory buffer with limited capacity
pub struct WorkingMemory {
    /// Items in working memory
    items: VecDeque<WorkingItem>,
    /// Maximum capacity (Miller's law default: 7)
    capacity: usize,
}

impl WorkingMemory {
    pub fn new(capacity: usize) -> Self {
        Self {
            items: VecDeque::new(),
            capacity,
        }
    }

    /// Add item (evicts oldest if at capacity)
    pub fn push(&mut self, item: WorkingItem) {
        if self.items.len() >= self.capacity {
            self.items.pop_front();
        }
        self.items.push_back(item);
    }

    /// Get recent items
    pub fn recent(&self, n: usize) -> Vec<&WorkingItem> {
        self.items.iter().rev().take(n).collect()
    }

    /// Clear all
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Item count
    pub fn len(&self) -> usize {
        self.items.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_working_memory() {
        let mut wm = WorkingMemory::new(3);
        wm.push(WorkingItem::new("item1".to_string()));
        wm.push(WorkingItem::new("item2".to_string()));
        wm.push(WorkingItem::new("item3".to_string()));
        assert_eq!(wm.len(), 3);
        wm.push(WorkingItem::new("item4".to_string())); // Evicts item1
        assert_eq!(wm.len(), 3);
    }
}
