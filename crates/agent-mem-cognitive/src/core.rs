//! Core Memory Module
//! 
//! Core memories store persistent identity, preferences, and fundamental beliefs.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Core memory - persistent identity and preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreMemory {
    /// Identity traits
    pub identity_traits: HashMap<String, String>,
    /// Preferences
    pub preferences: HashMap<String, Preference>,
    /// Beliefs
    pub beliefs: Vec<Belief>,
    /// Goals
    pub goals: Vec<Goal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preference {
    pub key: String,
    pub value: String,
    pub strength: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Belief {
    pub id: String,
    pub statement: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub description: String,
    pub priority: usize,
    pub completed: bool,
}

impl CoreMemory {
    pub fn new() -> Self {
        Self {
            identity_traits: HashMap::new(),
            preferences: HashMap::new(),
            beliefs: Vec::new(),
            goals: Vec::new(),
        }
    }
}

impl Default for CoreMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_memory() {
        let mut core = CoreMemory::new();
        core.identity_traits.insert("personality".to_string(), "analytical".to_string());
        assert_eq!(core.identity_traits.get("personality"), Some(&"analytical".to_string()));
    }
}
