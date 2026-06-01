//! Contextual Memory Module
//! 
//! Contextual memories store environment-aware and situation-specific information.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextInfo {
    /// Context ID
    pub id: String,
    /// Environment type
    pub environment: String,
    /// Location
    pub location: Option<String>,
    /// Time of day
    pub time_of_day: String,
    /// Mood/state
    pub mood: Option<String>,
    /// Active tasks
    pub active_tasks: Vec<String>,
    /// Custom context
    pub custom: HashMap<String, String>,
}

impl ContextInfo {
    pub fn new(id: String, environment: &str) -> Self {
        Self {
            id,
            environment: environment.to_string(),
            location: None,
            time_of_day: "unknown".to_string(),
            mood: None,
            active_tasks: Vec::new(),
            custom: HashMap::new(),
        }
    }
}

/// Contextual memory store
pub struct ContextualStore {
    contexts: Vec<ContextInfo>,
    active_context: Option<String>,
}

impl ContextualStore {
    pub fn new() -> Self {
        Self {
            contexts: Vec::new(),
            active_context: None,
        }
    }

    pub fn add_context(&mut self, context: ContextInfo) {
        self.contexts.push(context);
    }

    pub fn set_active(&mut self, id: &str) {
        self.active_context = Some(id.to_string());
    }

    pub fn get_active(&self) -> Option<&ContextInfo> {
        self.active_context.as_ref()
            .and_then(|id| self.contexts.iter().find(|c| &c.id == id))
    }
}

impl Default for ContextualStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contextual_store() {
        let mut store = ContextualStore::new();
        store.add_context(ContextInfo::new("ctx-1".to_string(), "home"));
        store.set_active("ctx-1");
        assert!(store.get_active().is_some());
    }
}
