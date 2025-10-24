//! Core memory types and data structures

use agent_mem_traits::{AgentMemError, MemoryItem, Result, Vector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

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

/// Core memory structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    /// Unique memory identifier
    pub id: String,
    /// Agent identifier
    pub agent_id: String,
    /// User identifier (optional)
    pub user_id: Option<String>,
    /// Memory type
    pub memory_type: MemoryType,
    /// Memory content
    pub content: String,
    /// Importance score (0.0 to 1.0)
    pub importance: f32,
    /// Vector embedding (optional)
    pub embedding: Option<Vector>,
    /// Creation timestamp
    pub created_at: i64,
    /// Last access timestamp
    pub last_accessed_at: i64,
    /// Access count
    pub access_count: u32,
    /// Expiration timestamp (optional)
    pub expires_at: Option<i64>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
    /// Memory version for conflict resolution
    pub version: u32,
}

impl Memory {
    /// Create a new memory
    pub fn new(
        agent_id: String,
        user_id: Option<String>,
        memory_type: MemoryType,
        content: String,
        importance: f32,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: Uuid::new_v4().to_string(),
            agent_id,
            user_id,
            memory_type,
            content,
            importance: importance.clamp(0.0, 1.0),
            embedding: None,
            created_at: now,
            last_accessed_at: now,
            access_count: 0,
            expires_at: None,
            metadata: HashMap::new(),
            version: 1,
        }
    }

    /// Record access to this memory
    pub fn access(&mut self) {
        self.access_count += 1;
        self.last_accessed_at = chrono::Utc::now().timestamp();
    }

    /// Calculate current importance with decay
    pub fn calculate_current_importance(&self) -> f32 {
        let current_time = chrono::Utc::now().timestamp();
        let time_decay = (current_time - self.created_at) as f32 / (24.0 * 3600.0); // days
        let access_factor = (self.access_count as f32 + 1.0).ln();

        // Apply time decay and access boost
        self.importance * (-time_decay * 0.01).exp() * (1.0 + access_factor * 0.1)
    }

    /// Check if memory is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            chrono::Utc::now().timestamp() > expires_at
        } else {
            false
        }
    }

    /// Set expiration time
    pub fn set_expiration(&mut self, expires_at: i64) {
        self.expires_at = Some(expires_at);
    }

    /// Add metadata
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Get metadata
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    /// Update content and increment version
    pub fn update_content(&mut self, new_content: String) {
        self.content = new_content;
        self.version += 1;
        self.last_accessed_at = chrono::Utc::now().timestamp();
    }
}

impl From<Memory> for MemoryItem {
    fn from(memory: Memory) -> Self {
        use agent_mem_traits::{MemoryType as TraitMemoryType, Session};
        use chrono::{DateTime, Utc};

        // Convert metadata from String to serde_json::Value
        let metadata: std::collections::HashMap<String, serde_json::Value> = memory
            .metadata
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::String(v)))
            .collect();

        // Create a session from memory data
        let mut session = Session::new();
        if let Some(ref user_id) = memory.user_id {
            session = session.with_user_id(Some(user_id.clone()));
        }
        session = session.with_agent_id(Some(memory.agent_id.clone()));

        MemoryItem {
            id: memory.id,
            content: memory.content,
            hash: None, // TODO: Calculate hash if needed
            metadata,
            score: Some(memory.importance),
            created_at: DateTime::from_timestamp(memory.created_at, 0)
                .unwrap_or_else(Utc::now),
            updated_at: Some(
                DateTime::from_timestamp(memory.last_accessed_at, 0).unwrap_or_else(Utc::now),
            ),
            session,
            memory_type: match memory.memory_type {
                // Basic cognitive memories
                MemoryType::Episodic => TraitMemoryType::Episodic,
                MemoryType::Semantic => TraitMemoryType::Semantic,
                MemoryType::Procedural => TraitMemoryType::Procedural,
                MemoryType::Working => TraitMemoryType::Working,
                // Advanced cognitive memories (AgentMem 7.0)
                MemoryType::Core => TraitMemoryType::Core,
                MemoryType::Resource => TraitMemoryType::Resource,
                MemoryType::Knowledge => TraitMemoryType::Knowledge,
                MemoryType::Contextual => TraitMemoryType::Contextual,
            },
            entities: Vec::new(),  // TODO: Extract entities if needed
            relations: Vec::new(), // TODO: Extract relations if needed
            // Additional fields for compatibility
            agent_id: memory.agent_id,
            user_id: memory.user_id,
            importance: memory.importance,
            embedding: memory.embedding.map(|v| v.values),
            last_accessed_at: DateTime::from_timestamp(memory.last_accessed_at, 0)
                .unwrap_or_else(Utc::now),
            access_count: memory.access_count,
            expires_at: memory
                .expires_at
                .map(|ts| DateTime::from_timestamp(ts, 0).unwrap_or_else(Utc::now)),
            version: memory.version,
        }
    }
}

impl TryFrom<MemoryItem> for Memory {
    type Error = AgentMemError;

    fn try_from(item: MemoryItem) -> Result<Self> {
        // Convert metadata from serde_json::Value to String
        let metadata: std::collections::HashMap<String, String> = item
            .metadata
            .into_iter()
            .filter_map(|(k, v)| match v {
                serde_json::Value::String(s) => Some((k, s)),
                _ => Some((k, v.to_string())),
            })
            .collect();

        let agent_id = item
            .session
            .agent_id
            .ok_or_else(|| AgentMemError::memory_error("Missing agent_id in session"))?;

        let memory_type = match item.memory_type {
            // Basic cognitive memories
            agent_mem_traits::MemoryType::Episodic => MemoryType::Episodic,
            agent_mem_traits::MemoryType::Semantic => MemoryType::Semantic,
            agent_mem_traits::MemoryType::Procedural => MemoryType::Procedural,
            agent_mem_traits::MemoryType::Working => MemoryType::Working,
            // Advanced cognitive memories (AgentMem 7.0)
            agent_mem_traits::MemoryType::Core => MemoryType::Core,
            agent_mem_traits::MemoryType::Resource => MemoryType::Resource,
            agent_mem_traits::MemoryType::Knowledge => MemoryType::Knowledge,
            agent_mem_traits::MemoryType::Contextual => MemoryType::Contextual,
            // Legacy mapping
            agent_mem_traits::MemoryType::Factual => MemoryType::Semantic, // Map Factual to Semantic
        };

        Ok(Memory {
            id: item.id,
            agent_id,
            user_id: item.session.user_id,
            memory_type,
            content: item.content,
            importance: item.importance,
            embedding: None,
            created_at: item.created_at.timestamp(),
            last_accessed_at: item
                .updated_at
                .map(|dt| dt.timestamp())
                .unwrap_or_else(|| chrono::Utc::now().timestamp()),
            access_count: 0,  // Default value
            expires_at: None, // Default value
            metadata,
            version: 1,
        })
    }
}

/// Memory search query
#[derive(Debug, Clone)]
pub struct MemoryQuery {
    /// Agent ID to search within
    pub agent_id: String,
    /// User ID filter (optional)
    pub user_id: Option<String>,
    /// Memory type filter (optional)
    pub memory_type: Option<MemoryType>,
    /// Text query for content search
    pub text_query: Option<String>,
    /// Vector query for semantic search
    pub vector_query: Option<Vector>,
    /// Minimum importance threshold
    pub min_importance: Option<f32>,
    /// Maximum age in seconds
    pub max_age_seconds: Option<i64>,
    /// Maximum number of results
    pub limit: usize,
}

impl MemoryQuery {
    /// Create a new memory query for the specified agent
    pub fn new(agent_id: String) -> Self {
        Self {
            agent_id,
            user_id: None,
            memory_type: None,
            text_query: None,
            vector_query: None,
            min_importance: None,
            max_age_seconds: None,
            limit: 10,
        }
    }

    /// Set the user ID for the query
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Set the memory type filter
    pub fn with_memory_type(mut self, memory_type: MemoryType) -> Self {
        self.memory_type = Some(memory_type);
        self
    }

    /// Set the text query for searching
    pub fn with_text_query(mut self, query: String) -> Self {
        self.text_query = Some(query);
        self
    }

    /// Set the vector query for semantic search
    pub fn with_vector_query(mut self, vector: Vector) -> Self {
        self.vector_query = Some(vector);
        self
    }

    /// Set the minimum importance threshold
    pub fn with_min_importance(mut self, importance: f32) -> Self {
        self.min_importance = Some(importance);
        self
    }

    /// Set the maximum age filter in seconds
    pub fn with_max_age_seconds(mut self, seconds: i64) -> Self {
        self.max_age_seconds = Some(seconds);
        self
    }

    /// Set the maximum number of results to return
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }
}

/// Memory search result
#[derive(Debug, Clone)]
pub struct MemorySearchResult {
    /// The matched memory
    pub memory: Memory,
    /// Relevance score (0.0 to 1.0)
    pub score: f32,
    /// Type of match found
    pub match_type: MatchType,
}

/// Type of match found
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MatchType {
    /// Exact text match
    ExactText,
    /// Partial text match
    PartialText,
    /// Semantic similarity match
    Semantic,
    /// Metadata field match
    Metadata,
}

/// Memory statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Total number of memories
    pub total_memories: usize,
    /// Count of memories by type
    pub memories_by_type: HashMap<MemoryType, usize>,
    /// Count of memories by agent
    pub memories_by_agent: HashMap<String, usize>,
    /// Average importance score across all memories
    pub average_importance: f32,
    /// Age of the oldest memory in days
    pub oldest_memory_age_days: f32,
    /// ID of the most frequently accessed memory
    pub most_accessed_memory_id: Option<String>,
    /// Total number of memory accesses
    pub total_access_count: u64,
}

impl Default for MemoryStats {
    fn default() -> Self {
        Self {
            total_memories: 0,
            memories_by_type: HashMap::new(),
            memories_by_agent: HashMap::new(),
            average_importance: 0.0,
            oldest_memory_age_days: 0.0,
            most_accessed_memory_id: None,
            total_access_count: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_type_string_conversion() {
        // Test basic cognitive memory types
        assert_eq!(MemoryType::Episodic.as_str(), "episodic");
        assert_eq!(MemoryType::Semantic.as_str(), "semantic");
        assert_eq!(MemoryType::Procedural.as_str(), "procedural");
        assert_eq!(MemoryType::Working.as_str(), "working");

        // Test advanced cognitive memory types (AgentMem 7.0)
        assert_eq!(MemoryType::Core.as_str(), "core");
        assert_eq!(MemoryType::Resource.as_str(), "resource");
        assert_eq!(MemoryType::Knowledge.as_str(), "knowledge");
        assert_eq!(MemoryType::Contextual.as_str(), "contextual");
    }

    #[test]
    fn test_memory_type_from_string() {
        // Test basic cognitive memory types
        assert_eq!(MemoryType::from_str("episodic"), Some(MemoryType::Episodic));
        assert_eq!(MemoryType::from_str("semantic"), Some(MemoryType::Semantic));
        assert_eq!(
            MemoryType::from_str("procedural"),
            Some(MemoryType::Procedural)
        );
        assert_eq!(MemoryType::from_str("working"), Some(MemoryType::Working));

        // Test advanced cognitive memory types (AgentMem 7.0)
        assert_eq!(MemoryType::from_str("core"), Some(MemoryType::Core));
        assert_eq!(MemoryType::from_str("resource"), Some(MemoryType::Resource));
        assert_eq!(
            MemoryType::from_str("knowledge"),
            Some(MemoryType::Knowledge)
        );
        assert_eq!(
            MemoryType::from_str("contextual"),
            Some(MemoryType::Contextual)
        );

        // Test invalid type
        assert_eq!(MemoryType::from_str("invalid"), None);
    }

    #[test]
    fn test_memory_type_classification() {
        // Test basic type classification
        assert!(MemoryType::Episodic.is_basic_type());
        assert!(MemoryType::Semantic.is_basic_type());
        assert!(MemoryType::Procedural.is_basic_type());
        assert!(MemoryType::Working.is_basic_type());

        assert!(!MemoryType::Episodic.is_advanced_type());
        assert!(!MemoryType::Semantic.is_advanced_type());
        assert!(!MemoryType::Procedural.is_advanced_type());
        assert!(!MemoryType::Working.is_advanced_type());

        // Test advanced type classification
        assert!(MemoryType::Core.is_advanced_type());
        assert!(MemoryType::Resource.is_advanced_type());
        assert!(MemoryType::Knowledge.is_advanced_type());
        assert!(MemoryType::Contextual.is_advanced_type());

        assert!(!MemoryType::Core.is_basic_type());
        assert!(!MemoryType::Resource.is_basic_type());
        assert!(!MemoryType::Knowledge.is_basic_type());
        assert!(!MemoryType::Contextual.is_basic_type());
    }

    #[test]
    fn test_memory_type_all_types() {
        let all_types = MemoryType::all_types();
        assert_eq!(all_types.len(), 8);

        // Verify all types are included
        assert!(all_types.contains(&MemoryType::Episodic));
        assert!(all_types.contains(&MemoryType::Semantic));
        assert!(all_types.contains(&MemoryType::Procedural));
        assert!(all_types.contains(&MemoryType::Working));
        assert!(all_types.contains(&MemoryType::Core));
        assert!(all_types.contains(&MemoryType::Resource));
        assert!(all_types.contains(&MemoryType::Knowledge));
        assert!(all_types.contains(&MemoryType::Contextual));
    }

    #[test]
    fn test_memory_type_descriptions() {
        // Test that all memory types have descriptions
        for memory_type in MemoryType::all_types() {
            let description = memory_type.description();
            assert!(
                !description.is_empty(),
                "Memory type {:?} should have a description",
                memory_type
            );
        }
    }

    #[test]
    fn test_memory_creation_with_new_types() {
        // Test creating memories with new cognitive types
        let core_memory = Memory::new(
            "agent_1".to_string(),
            Some("user_1".to_string()),
            MemoryType::Core,
            "User prefers dark mode interface".to_string(),
            0.9,
        );
        assert_eq!(core_memory.memory_type, MemoryType::Core);
        assert_eq!(core_memory.importance, 0.9);

        let resource_memory = Memory::new(
            "agent_1".to_string(),
            Some("user_1".to_string()),
            MemoryType::Resource,
            "Document: project_plan.pdf".to_string(),
            0.7,
        );
        assert_eq!(resource_memory.memory_type, MemoryType::Resource);

        let knowledge_memory = Memory::new(
            "agent_1".to_string(),
            Some("user_1".to_string()),
            MemoryType::Knowledge,
            "Python is a programming language".to_string(),
            0.8,
        );
        assert_eq!(knowledge_memory.memory_type, MemoryType::Knowledge);

        let contextual_memory = Memory::new(
            "agent_1".to_string(),
            Some("user_1".to_string()),
            MemoryType::Contextual,
            "Currently in meeting room A".to_string(),
            0.6,
        );
        assert_eq!(contextual_memory.memory_type, MemoryType::Contextual);
    }
}
