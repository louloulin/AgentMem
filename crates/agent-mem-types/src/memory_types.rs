//! Memory type definitions
//! 
//! Defines the 8 cognitive memory types for AgentMem

use serde::{Deserialize, Serialize};

/// Cognitive memory type classification (8 types for AgentMem 7.0)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MemoryType {
    // Basic cognitive memories (existing)
    /// Episodic memories - specific events and experiences with temporal context
    Episodic,
    /// Semantic memories - facts, concepts, and general knowledge
    Semantic,
    /// Procedural memories - skills, procedures, and how-to knowledge
    Procedural,
    /// Working memories - temporary information processing and active context
    Working,

    // Advanced cognitive memories (new in AgentMem 7.0)
    /// Core memories - persistent identity, preferences, and fundamental beliefs
    Core,
    /// Resource memories - multimedia content, documents, and external resources
    Resource,
    /// Knowledge memories - structured knowledge graphs and domain expertise
    Knowledge,
    /// Contextual memories - environment-aware and situation-specific information
    Contextual,
}

impl MemoryType {
    /// Convert memory type to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            // Basic cognitive memories
            MemoryType::Episodic => "episodic",
            MemoryType::Semantic => "semantic",
            MemoryType::Procedural => "procedural",
            MemoryType::Working => "working",
            // Advanced cognitive memories (AgentMem 7.0)
            MemoryType::Core => "core",
            MemoryType::Resource => "resource",
            MemoryType::Knowledge => "knowledge",
            MemoryType::Contextual => "contextual",
        }
    }

    /// Parse memory type from string representation
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            // Basic cognitive memories
            "episodic" => Some(MemoryType::Episodic),
            "semantic" => Some(MemoryType::Semantic),
            "procedural" => Some(MemoryType::Procedural),
            "working" => Some(MemoryType::Working),
            // Advanced cognitive memories (AgentMem 7.0)
            "core" => Some(MemoryType::Core),
            "resource" => Some(MemoryType::Resource),
            "knowledge" => Some(MemoryType::Knowledge),
            "contextual" => Some(MemoryType::Contextual),
            _ => None,
        }
    }

    /// Get all available memory types
    pub fn all_types() -> Vec<Self> {
        vec![
            MemoryType::Episodic,
            MemoryType::Semantic,
            MemoryType::Procedural,
            MemoryType::Working,
            MemoryType::Core,
            MemoryType::Resource,
            MemoryType::Knowledge,
            MemoryType::Contextual,
        ]
    }

    /// Check if this is a basic cognitive memory type
    pub fn is_basic_type(&self) -> bool {
        matches!(
            self,
            MemoryType::Episodic
                | MemoryType::Semantic
                | MemoryType::Procedural
                | MemoryType::Working
        )
    }

    /// Check if this is an advanced cognitive memory type (AgentMem 7.0)
    pub fn is_advanced_type(&self) -> bool {
        matches!(
            self,
            MemoryType::Core
                | MemoryType::Resource
                | MemoryType::Knowledge
                | MemoryType::Contextual
        )
    }

    /// Get the description of the memory type
    pub fn description(&self) -> &'static str {
        match self {
            MemoryType::Episodic => "Specific events and experiences with temporal context",
            MemoryType::Semantic => "Facts, concepts, and general knowledge",
            MemoryType::Procedural => "Skills, procedures, and how-to knowledge",
            MemoryType::Working => "Temporary information processing and active context",
            MemoryType::Core => "Persistent identity, preferences, and fundamental beliefs",
            MemoryType::Resource => "Multimedia content, documents, and external resources",
            MemoryType::Knowledge => "Structured knowledge graphs and domain expertise",
            MemoryType::Contextual => "Environment-aware and situation-specific information",
        }
    }
}

impl std::fmt::Display for MemoryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for MemoryType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str(s).ok_or_else(|| format!("Unknown memory type: {}", s))
    }
}

/// Memory importance level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Ord)]
pub enum ImportanceLevel {
    /// Low importance (score < 0.4)
    Low = 1,
    /// Medium importance (0.4 <= score < 0.6)
    Medium = 2,
    /// High importance (0.6 <= score < 0.8)
    High = 3,
    /// Critical importance (score >= 0.8)
    Critical = 4,
}

impl ImportanceLevel {
    /// Convert a numeric score to an importance level
    pub fn from_score(score: f32) -> Self {
        if score >= 0.8 {
            ImportanceLevel::Critical
        } else if score >= 0.6 {
            ImportanceLevel::High
        } else if score >= 0.4 {
            ImportanceLevel::Medium
        } else {
            ImportanceLevel::Low
        }
    }

    /// Convert importance level to a numeric score
    pub fn to_score(&self) -> f32 {
        match self {
            ImportanceLevel::Low => 0.25,
            ImportanceLevel::Medium => 0.5,
            ImportanceLevel::High => 0.75,
            ImportanceLevel::Critical => 1.0,
        }
    }
}
