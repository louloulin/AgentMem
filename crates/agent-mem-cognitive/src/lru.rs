//! LRU Cache for AgentMem Cognitive
//! 
//! Provides LRU cache for working memory optimization

use std::collections::{HashMap, VecDeque};
use std::hash::Hash;

/// LRU Cache implementation
#[derive(Debug, Clone)]
pub struct LruCache<K, V> {
    capacity: usize,
    map: HashMap<K, V>,
    order: VecDeque<K>,
}

impl<K: Clone + Hash + Eq, V: Clone> LruCache<K, V> {
    /// Create new LRU cache
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            map: HashMap::with_capacity(capacity),
            order: VecDeque::with_capacity(capacity),
        }
    }
    
    /// Get value by key
    pub fn get(&mut self, key: &K) -> Option<&V> {
        if self.map.contains_key(key) {
            // Move to back (most recently used)
            self.order.retain(|k| k != key);
            self.order.push_back(key.clone());
            self.map.get(key)
        } else {
            None
        }
    }
    
    /// Put value
    pub fn put(&mut self, key: K, value: V) {
        if self.map.contains_key(&key) {
            // Update existing
            self.order.retain(|k| k != &key);
            self.order.push_back(key.clone());
            self.map.insert(key, value);
        } else {
            // Insert new
            if self.map.len() >= self.capacity {
                // Evict LRU
                if let Some(lru_key) = self.order.pop_front() {
                    self.map.remove(&lru_key);
                }
            }
            self.order.push_back(key.clone());
            self.map.insert(key, value);
        }
    }
    
    /// Check if contains key
    pub fn contains(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }
    
    /// Remove key
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.order.retain(|k| k != key);
        self.map.remove(key)
    }
    
    /// Clear cache
    pub fn clear(&mut self) {
        self.map.clear();
        self.order.clear();
    }
    
    /// Get current size
    pub fn len(&self) -> usize {
        self.map.len()
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
    
    /// Get capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    
    /// Get all keys
    pub fn keys(&self) -> Vec<&K> {
        self.order.iter().collect()
    }
}

impl<K: Clone + Hash + Eq, V: Clone> Default for LruCache<K, V> {
    fn default() -> Self {
        Self::new(100)
    }
}

/// Memory tier with LRU cache
#[derive(Debug, Clone)]
pub struct LruTier<V> {
    cache: LruCache<String, V>,
}

impl<V: Clone> LruTier<V> {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: LruCache::new(capacity),
        }
    }
    
    pub fn get(&mut self, id: &str) -> Option<&V> {
        self.cache.get(&id.to_string())
    }
    
    pub fn put(&mut self, id: String, value: V) {
        self.cache.put(id, value);
    }
    
    pub fn remove(&mut self, id: &str) -> Option<V> {
        self.cache.remove(&id.to_string())
    }
    
    pub fn len(&self) -> usize {
        self.cache.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

impl<V: Clone> Default for LruTier<V> {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_basic() {
        let mut cache = LruCache::new(3);
        
        cache.put(1, "a");
        cache.put(2, "b");
        cache.put(3, "c");
        
        assert_eq!(cache.get(&1), Some(&"a"));
        assert_eq!(cache.get(&2), Some(&"b"));
        assert_eq!(cache.get(&3), Some(&"c"));
    }
    
    #[test]
    fn test_lru_eviction() {
        let mut cache = LruCache::new(3);
        
        cache.put(1, "a");
        cache.put(2, "b");
        cache.put(3, "c");
        cache.put(4, "d"); // Should evict 1
        
        assert_eq!(cache.get(&1), None);
        assert!(cache.contains(&2));
        assert!(cache.contains(&3));
        assert!(cache.contains(&4));
    }
    
    #[test]
    fn test_lru_access_order() {
        let mut cache = LruCache::new(3);
        
        cache.put(1, "a");
        cache.put(2, "b");
        cache.put(3, "c");
        cache.get(&1); // Access 1, making it most recent
        cache.put(4, "d"); // Should evict 2 (now LRU)
        
        assert_eq!(cache.get(&2), None);
        assert!(cache.contains(&1)); // Was accessed, so not evicted
        assert!(cache.contains(&3));
        assert!(cache.contains(&4));
    }
}
