//! Attribute type definitions
//! 
//! Defines attribute keys, values, patterns, and sets for memory organization

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Attribute key
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AttributeKey {
    /// Key name (e.g., "priority", "category", "status")
    pub name: String,
    /// Optional namespace for disambiguation
    pub namespace: Option<String>,
}

impl AttributeKey {
    /// Create a new attribute key
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            namespace: None,
        }
    }

    /// Create a namespaced attribute key
    pub fn namespaced<S: Into<String>>(namespace: S, name: S) -> Self {
        Self {
            name: name.into(),
            namespace: Some(namespace.into()),
        }
    }
}

impl std::fmt::Display for AttributeKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.namespace {
            Some(ns) => write!(f, "{}:{}", ns, self.name),
            None => write!(f, "{}", self.name),
        }
    }
}

/// Attribute value types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AttributeValue {
    /// String value
    String(String),
    /// Integer value
    Integer(i64),
    /// Float value
    Float(f64),
    /// Boolean value
    Boolean(bool),
    /// String array
    StringArray(Vec<String>),
    /// Integer array
    IntegerArray(Vec<i64>),
    /// Nested object (arbitrary JSON)
    Object(serde_json::Value),
}

impl AttributeValue {
    /// Convert to string representation
    pub fn as_string(&self) -> String {
        match self {
            AttributeValue::String(s) => s.clone(),
            AttributeValue::Integer(i) => i.to_string(),
            AttributeValue::Float(f) => f.to_string(),
            AttributeValue::Boolean(b) => b.to_string(),
            AttributeValue::StringArray(arr) => arr.join(", "),
            AttributeValue::IntegerArray(arr) => arr.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(", "),
            AttributeValue::Object(v) => serde_json::to_string(v).unwrap_or_default(),
        }
    }

    /// Create from string
    pub fn from_string(s: &str) -> Self {
        AttributeValue::String(s.to_string())
    }
}

impl PartialEq for AttributeValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AttributeValue::String(a), AttributeValue::String(b)) => a == b,
            (AttributeValue::Integer(a), AttributeValue::Integer(b)) => a == b,
            (AttributeValue::Float(a), AttributeValue::Float(b)) => (a - b).abs() < f64::EPSILON,
            (AttributeValue::Boolean(a), AttributeValue::Boolean(b)) => a == b,
            _ => false,
        }
    }
}

/// Attribute pattern for matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttributePattern {
    /// Exact match
    Exact(String),
    /// Prefix match
    Prefix(String),
    /// Suffix match
    Suffix(String),
    /// Regex match
    Regex(String),
    /// Contains match
    Contains(String),
}

impl AttributePattern {
    /// Check if value matches pattern
    pub fn matches(&self, value: &str) -> bool {
        match self {
            AttributePattern::Exact(s) => value == s,
            AttributePattern::Prefix(p) => value.starts_with(p),
            AttributePattern::Suffix(s) => value.ends_with(s),
            AttributePattern::Regex(r) => {
                regex::Regex::new(r)
                    .map(|re| re.is_match(value))
                    .unwrap_or(false)
            }
            AttributePattern::Contains(c) => value.contains(c),
        }
    }
}

/// Attribute set for organizing and filtering memories
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AttributeSet {
    /// Internal storage
    attrs: HashMap<String, AttributeValue>,
}

impl AttributeSet {
    /// Create a new empty attribute set
    pub fn new() -> Self {
        Self::default()
    }

    /// Set an attribute
    pub fn set<K: Into<String>, V: Into<AttributeValue>>(&mut self, key: K, value: V) {
        self.attrs.insert(key.into(), value.into());
    }

    /// Get an attribute
    pub fn get(&self, key: &str) -> Option<&AttributeValue> {
        self.attrs.get(key)
    }

    /// Check if attribute exists
    pub fn contains(&self, key: &str) -> bool {
        self.attrs.contains_key(key)
    }

    /// Remove an attribute
    pub fn remove(&mut self, key: &str) -> Option<AttributeValue> {
        self.attrs.remove(key)
    }

    /// Get all keys
    pub fn keys(&self) -> Vec<&String> {
        self.attrs.keys().collect()
    }

    /// Get all attributes as a map
    pub fn all(&self) -> &HashMap<String, AttributeValue> {
        &self.attrs
    }

    /// Merge another attribute set into this one
    pub fn merge(&mut self, other: &AttributeSet) {
        for (k, v) in &other.attrs {
            self.attrs.insert(k.clone(), v.clone());
        }
    }

    // Scope-related methods (extracted from original types.rs)
    
    /// Get agent_id from attributes
    pub fn get_agent_id(&self) -> Option<String> {
        self.attrs.get("agent_id")
            .and_then(|v| match v {
                AttributeValue::String(s) => Some(s.clone()),
                _ => None,
            })
    }

    /// Get user_id from attributes
    pub fn get_user_id(&self) -> Option<String> {
        self.attrs.get("user_id")
            .and_then(|v| match v {
                AttributeValue::String(s) => Some(s.clone()),
                _ => None,
            })
    }

    /// Get session_id from attributes
    pub fn get_session_id(&self) -> Option<String> {
        self.attrs.get("session_id")
            .and_then(|v| match v {
                AttributeValue::String(s) => Some(s.clone()),
                _ => None,
            })
    }

    /// Check if this is a global scope
    pub fn is_global_scope(&self) -> bool {
        self.get_agent_id().is_none() && self.get_user_id().is_none()
    }

    /// Set session scope (agent_id + user_id + session_id)
    pub fn set_session_scope(&mut self, agent_id: &str, user_id: &str, session_id: &str) {
        self.set("agent_id", AttributeValue::String(agent_id.to_string()));
        self.set("user_id", AttributeValue::String(user_id.to_string()));
        self.set("session_id", AttributeValue::String(session_id.to_string()));
    }
}

impl From<HashMap<String, AttributeValue>> for AttributeSet {
    fn from(map: HashMap<String, AttributeValue>) -> Self {
        Self { attrs: map }
    }
}
