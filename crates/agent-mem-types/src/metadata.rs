//! Metadata type definitions
//! 
//! Defines memory metadata structure

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metadata for memory items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    /// Custom key-value attributes
    pub attributes: HashMap<String, serde_json::Value>,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            attributes: HashMap::new(),
        }
    }
}

impl Metadata {
    /// Create new empty metadata
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a key-value pair
    pub fn set<K: Into<String>, V: Into<serde_json::Value>>(&mut self, key: K, value: V) {
        self.attributes.insert(key.into(), value.into());
    }

    /// Get a value by key
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.attributes.get(key)
    }

    /// Check if key exists
    pub fn contains(&self, key: &str) -> bool {
        self.attributes.contains_key(key)
    }

    /// Remove a key
    pub fn remove(&mut self, key: &str) -> Option<serde_json::Value> {
        self.attributes.remove(key)
    }
}

impl From<HashMap<String, serde_json::Value>> for Metadata {
    fn from(map: HashMap<String, serde_json::Value>) -> Self {
        Self { attributes: map }
    }
}
