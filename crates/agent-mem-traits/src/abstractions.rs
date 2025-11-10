//! Core Abstractions for AgentMem V4.0
//!
//! This module defines the foundational abstractions for the radical refactoring:
//! - Memory as open AttributeSet
//! - Query as intent + constraints + preferences
//! - Retrieval as composable engines

use crate::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Memory Abstraction
// ============================================================================

/// Memory = Content + Attributes + Relations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    /// Unique identifier
    pub id: MemoryId,
    
    /// Core content (multi-modal)
    pub content: Content,
    
    /// Open attribute set (completely extensible)
    pub attributes: AttributeSet,
    
    /// Relations with other memories/entities
    pub relations: RelationGraph,
    
    /// System metadata (maintained by the system)
    pub metadata: Metadata,
}

/// Memory identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MemoryId(pub String);

impl MemoryId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
    
    pub fn from_string(id: String) -> Self {
        Self(id)
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for MemoryId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for MemoryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Content abstraction (multi-modal)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Content {
    /// Text content
    Text(String),
    
    /// Structured data
    Structured(serde_json::Value),
    
    /// Vector embedding
    Vector(Vec<f32>),
    
    /// Multi-modal combination
    Multimodal(Vec<Content>),
    
    /// Binary data
    Binary(Vec<u8>),
}

impl Content {
    pub fn text(s: impl Into<String>) -> Self {
        Content::Text(s.into())
    }
    
    pub fn structured(v: serde_json::Value) -> Self {
        Content::Structured(v)
    }
    
    pub fn vector(v: Vec<f32>) -> Self {
        Content::Vector(v)
    }
    
    /// Check if content contains a substring (for text content)
    pub fn contains(&self, pattern: &str) -> bool {
        match self {
            Content::Text(t) => t.contains(pattern),
            Content::Structured(v) => v.to_string().contains(pattern),
            _ => false,
        }
    }
    
    /// Get text representation
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Content::Text(t) => Some(t),
            _ => None,
        }
    }
}

/// Attribute set (completely open)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AttributeSet {
    /// Attributes dictionary (type-safe)
    pub attributes: HashMap<AttributeKey, AttributeValue>,
    
    /// Optional schema for validation
    pub schema: Option<AttributeSchema>,
}

impl AttributeSet {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_attribute(mut self, key: AttributeKey, value: AttributeValue) -> Self {
        self.attributes.insert(key, value);
        self
    }
    
    pub fn get(&self, key: &AttributeKey) -> Option<&AttributeValue> {
        self.attributes.get(key)
    }
    
    pub fn set(&mut self, key: AttributeKey, value: AttributeValue) {
        self.attributes.insert(key, value);
    }
    
    /// Alias for set() for backward compatibility
    pub fn insert(&mut self, key: AttributeKey, value: AttributeValue) {
        self.set(key, value);
    }
    
    pub fn remove(&mut self, key: &AttributeKey) -> Option<AttributeValue> {
        self.attributes.remove(key)
    }
}

/// Attribute key (strong-typed, with namespace)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AttributeKey {
    /// Namespace (to avoid conflicts)
    pub namespace: String,
    
    /// Key name
    pub name: String,
}

impl AttributeKey {
    pub fn new(namespace: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            name: name.into(),
        }
    }
    
    /// Create a core attribute (system namespace)
    pub fn core(name: impl Into<String>) -> Self {
        Self::new("core", name)
    }
    
    /// Create a system attribute (for system-managed metadata)
    pub fn system(name: impl Into<String>) -> Self {
        Self::new("system", name)
    }
    
    /// Create a user attribute
    pub fn user(name: impl Into<String>) -> Self {
        Self::new("user", name)
    }
    
    /// Create an agent attribute
    pub fn agent(name: impl Into<String>) -> Self {
        Self::new("agent", name)
    }
    
    pub fn full_name(&self) -> String {
        format!("{}::{}", self.namespace, self.name)
    }
}

/// Attribute value (type-safe)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttributeValue {
    String(String),
    Number(f64),
    Integer(i64),
    Boolean(bool),
    DateTime(DateTime<Utc>),
    List(Vec<AttributeValue>),
    Map(HashMap<String, AttributeValue>),
    Null,
}

impl AttributeValue {
    pub fn as_string(&self) -> Option<&String> {
        match self {
            AttributeValue::String(s) => Some(s),
            _ => None,
        }
    }
    
    pub fn as_number(&self) -> Option<f64> {
        match self {
            AttributeValue::Number(n) => Some(*n),
            AttributeValue::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }
    
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            AttributeValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }
}

/// Attribute schema (optional, for validation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeSchema {
    pub required_attributes: Vec<AttributeKey>,
    pub attribute_types: HashMap<AttributeKey, AttributeType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttributeType {
    String,
    Number,
    Integer,
    Boolean,
    DateTime,
    List(Box<AttributeType>),
    Map,
}

/// Relation graph
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RelationGraph {
    /// Outgoing edges (this memory points to others)
    pub outgoing: Vec<RelationV4>,
    
    /// Incoming edges (others point to this)
    pub incoming: Vec<RelationV4>,
}

impl RelationGraph {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn add_outgoing(&mut self, relation: RelationV4) {
        self.outgoing.push(relation);
    }
    
    pub fn add_incoming(&mut self, relation: RelationV4) {
        self.incoming.push(relation);
    }
}

/// Relation in V4 abstraction (use types::Relation for compatibility)
pub type RelationV4 = crate::types::Relation;

/// System metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub accessed_at: DateTime<Utc>,
    pub access_count: u32,
    pub version: u32,
    pub hash: Option<String>,
}

impl Default for Metadata {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            created_at: now,
            updated_at: now,
            accessed_at: now,
            access_count: 0,
            version: 1,
            hash: None,
        }
    }
}

// ============================================================================
// Query Abstraction
// ============================================================================

/// Query = Intent + Constraints + Preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    /// Core: query intent (multiple expressions)
    pub intent: QueryIntent,
    
    /// Core: constraints (must satisfy)
    pub constraints: Vec<Constraint>,
    
    /// Core: preferences (soft requirements)
    pub preferences: Vec<Preference>,
    
    /// Context (affects understanding and retrieval)
    pub context: QueryContext,
}

impl Query {
    pub fn new(intent: QueryIntent) -> Self {
        Self {
            intent,
            constraints: vec![],
            preferences: vec![],
            context: QueryContext::default(),
        }
    }
    
    pub fn with_constraint(mut self, constraint: Constraint) -> Self {
        self.constraints.push(constraint);
        self
    }
    
    pub fn with_preference(mut self, preference: Preference) -> Self {
        self.preferences.push(preference);
        self
    }
    
    pub fn with_context(mut self, context: QueryContext) -> Self {
        self.context = context;
        self
    }
}

/// Query intent (multiple expression methods)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryIntent {
    /// Natural language
    NaturalLanguage {
        text: String,
        language: Option<String>,
    },
    
    /// Structured query
    Structured {
        predicates: Vec<Predicate>,
    },
    
    /// Vector similarity
    Vector {
        embedding: Vec<f32>,
    },
    
    /// Hybrid (combine multiple)
    Hybrid {
        intents: Vec<QueryIntent>,
        fusion: FusionStrategy,
    },
}

impl QueryIntent {
    pub fn natural_language(text: impl Into<String>) -> Self {
        QueryIntent::NaturalLanguage {
            text: text.into(),
            language: None,
        }
    }
    
    pub fn structured(predicates: Vec<Predicate>) -> Self {
        QueryIntent::Structured { predicates }
    }
    
    pub fn vector(embedding: Vec<f32>) -> Self {
        QueryIntent::Vector { embedding }
    }
}

/// Predicate for structured queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Predicate {
    pub key: AttributeKey,
    pub operator: ComparisonOperator,
    pub value: AttributeValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonOperator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    GreaterOrEqual,
    LessOrEqual,
    Contains,
    StartsWith,
    EndsWith,
    Matches,
}

/// Fusion strategy for hybrid queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FusionStrategy {
    /// Weighted average
    Weighted(Vec<f32>),
    
    /// Reciprocal rank fusion
    ReciprocalRank,
    
    /// Custom fusion
    Custom(String),
}

/// Constraint (hard requirements)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Constraint {
    /// Attribute constraint
    Attribute {
        key: AttributeKey,
        operator: ComparisonOperator,
        value: AttributeValue,
    },
    
    /// Relation constraint
    Relation {
        relation_type: String,
        target: String,
    },
    
    /// Temporal constraint
    Temporal {
        time_range: TimeRange,
    },
    
    /// Spatial constraint (Scope)
    Spatial {
        scope: ScopeConstraint,
    },
    
    /// Logical combination
    Logical {
        operator: LogicalOperator,
        constraints: Vec<Constraint>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogicalOperator {
    And,
    Or,
    Not,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
}

/// Scope constraint (abstracted, not limited to User/Agent/Global)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScopeConstraint {
    /// Attribute match
    AttributeMatch {
        key: AttributeKey,
        value: AttributeValue,
    },
    
    /// Relation match
    RelationMatch {
        relation_type: String,
        target: String,
    },
    
    /// Any (no constraint)
    Any,
}

/// Preference (soft requirements)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preference {
    /// Preference type
    pub preference_type: PreferenceType,
    
    /// Weight (adjustable)
    pub weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PreferenceType {
    /// Temporal preference (freshness)
    Temporal(TemporalPreference),
    
    /// Relevance preference
    Relevance(RelevancePreference),
    
    /// Diversity preference
    Diversity(DiversityPreference),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalPreference {
    pub prefer_recent: bool,
    pub decay_factor: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelevancePreference {
    pub min_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiversityPreference {
    pub max_similar: usize,
}

/// Query context
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QueryContext {
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub session_id: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

// ============================================================================
// Retrieval Abstraction
// ============================================================================

/// Retrieval engine (composable)
#[async_trait]
pub trait RetrievalEngine: Send + Sync {
    /// Retrieve memories
    async fn retrieve(
        &self,
        query: &Query,
        context: &RetrievalContext,
    ) -> Result<RetrievalResult>;
    
    /// Engine name
    fn name(&self) -> &str;
    
    /// Supported query types
    fn supported_intents(&self) -> Vec<QueryIntentType>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryIntentType {
    NaturalLanguage,
    Structured,
    Vector,
    Hybrid,
}

/// Retrieval context
#[derive(Debug, Clone, Default)]
pub struct RetrievalContext {
    pub max_results: Option<usize>,
    pub timeout: Option<std::time::Duration>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Retrieval result
#[derive(Debug, Clone)]
pub struct RetrievalResult {
    /// Memories list
    pub memories: Vec<ScoredMemory>,
    
    /// Explanation (optional, for debugging)
    pub explanation: Option<RetrievalExplanation>,
    
    /// Performance metrics
    pub metrics: RetrievalMetrics,
}

/// Scored memory
#[derive(Debug, Clone)]
pub struct ScoredMemory {
    /// Memory
    pub memory: Memory,
    
    /// Total score
    pub score: f32,
    
    /// Score breakdown (explainability)
    pub score_breakdown: HashMap<String, f32>,
}

/// Retrieval explanation
#[derive(Debug, Clone)]
pub struct RetrievalExplanation {
    /// Why these memories were selected
    pub reasoning: Vec<ReasoningStep>,
    
    /// Engines used
    pub engines_used: Vec<String>,
    
    /// Fusion strategy
    pub fusion_strategy: String,
}

#[derive(Debug, Clone)]
pub struct ReasoningStep {
    pub step: String,
    pub description: String,
}

/// Retrieval metrics
#[derive(Debug, Clone, Default)]
pub struct RetrievalMetrics {
    pub query_time_ms: u64,
    pub total_candidates: usize,
    pub returned_results: usize,
}

// ============================================================================
// Backward Compatibility Helpers
// ============================================================================

impl Memory {
    /// Create memory from old MemoryItem format
    pub fn from_legacy_item(item: &crate::types::MemoryItem) -> Self {
        let mut attributes = AttributeSet::new();
        
        // Map legacy fields to attributes
        attributes.set(
            AttributeKey::core("user_id"),
            item.user_id.as_ref()
                .map(|s| AttributeValue::String(s.clone()))
                .unwrap_or(AttributeValue::Null),
        );
        
        attributes.set(
            AttributeKey::core("agent_id"),
            AttributeValue::String(item.agent_id.clone()),
        );
        
        attributes.set(
            AttributeKey::core("memory_type"),
            AttributeValue::String(item.memory_type.as_str().to_string()),
        );
        
        attributes.set(
            AttributeKey::core("importance"),
            AttributeValue::Number(item.importance as f64),
        );
        
        // Add metadata fields
        for (k, v) in &item.metadata {
            attributes.set(
                AttributeKey::new("metadata", k),
                AttributeValue::String(v.to_string()),
            );
        }
        
        Self {
            id: MemoryId::from_string(item.id.clone()),
            content: Content::text(item.content.clone()),
            attributes,
            relations: RelationGraph::new(),
            metadata: Metadata {
                created_at: item.created_at,
                updated_at: item.updated_at.unwrap_or(item.created_at),
                accessed_at: item.last_accessed_at,
                access_count: item.access_count,
                version: item.version,
                hash: item.hash.clone(),
            },
        }
    }
    
    /// Convert to legacy MemoryItem format
    pub fn to_legacy_item(&self) -> crate::types::MemoryItem {
        use crate::types::MemoryType;
        
        let user_id = self.attributes.get(&AttributeKey::core("user_id"))
            .and_then(|v| v.as_string())
            .cloned();
        
        let agent_id = self.attributes.get(&AttributeKey::core("agent_id"))
            .and_then(|v| v.as_string())
            .cloned()
            .unwrap_or_else(|| "default".to_string());
        
        let importance = self.attributes.get(&AttributeKey::core("importance"))
            .and_then(|v| v.as_number())
            .unwrap_or(0.5) as f32;
        
        let memory_type_str = self.attributes.get(&AttributeKey::core("memory_type"))
            .and_then(|v| v.as_string())
            .map(|s| s.as_str())
            .unwrap_or("episodic");
        
        let memory_type = MemoryType::parse_type(memory_type_str)
            .unwrap_or(MemoryType::Episodic);
        
        let content = match &self.content {
            Content::Text(s) => s.clone(),
            Content::Structured(v) => v.to_string(),
            _ => "".to_string(),
        };
        
        crate::types::MemoryItem {
            id: self.id.0.clone(),
            content,
            hash: self.metadata.hash.clone(),
            metadata: HashMap::new(),
            score: None,
            created_at: self.metadata.created_at,
            updated_at: Some(self.metadata.updated_at),
            session: crate::types::Session::new(),
            memory_type,
            entities: vec![],
            relations: vec![],
            agent_id,
            user_id,
            importance,
            embedding: None,
            last_accessed_at: self.metadata.accessed_at,
            access_count: self.metadata.access_count,
            expires_at: None,
            version: self.metadata.version,
        }
    }
}

impl Query {
    /// Create query from simple string (backward compatibility)
    pub fn from_string(s: impl Into<String>) -> Self {
        Self::new(QueryIntent::natural_language(s))
    }
}

// Memory Extension Methods for Legacy Compatibility
impl Memory {
    /// Get agent_id from attributes
    pub fn agent_id(&self) -> Option<String> {
        self.attributes
            .get(&AttributeKey::core("agent_id"))
            .and_then(|v| v.as_string())
            .cloned()
    }
    
    /// Get user_id from attributes
    pub fn user_id(&self) -> Option<String> {
        self.attributes
            .get(&AttributeKey::core("user_id"))
            .and_then(|v| v.as_string())
            .cloned()
    }
    
    /// Get memory_type from attributes
    pub fn memory_type(&self) -> Option<String> {
        self.attributes
            .get(&AttributeKey::core("memory_type"))
            .and_then(|v| v.as_string())
            .cloned()
    }
    
    /// Get importance from attributes
    pub fn importance(&self) -> Option<f64> {
        self.attributes
            .get(&AttributeKey::system("importance"))
            .and_then(|v| v.as_number())
    }
    
    /// Get score from attributes
    pub fn score(&self) -> Option<f64> {
        self.attributes
            .get(&AttributeKey::system("score"))
            .and_then(|v| v.as_number())
    }
    
    /// Get hash from attributes
    pub fn hash(&self) -> Option<String> {
        self.attributes
            .get(&AttributeKey::system("hash"))
            .and_then(|v| v.as_string())
            .cloned()
    }
    
    /// Get access_count from metadata
    pub fn access_count(&self) -> u32 {
        self.metadata.access_count
    }
    
    /// Get created_at from metadata
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.metadata.created_at
    }
    
    /// Get updated_at from metadata
    pub fn updated_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.metadata.updated_at
    }
    
    /// Get last_accessed from metadata
    pub fn last_accessed(&self) -> chrono::DateTime<chrono::Utc> {
        self.metadata.accessed_at
    }
    
    /// Set agent_id attribute
    pub fn set_agent_id(&mut self, agent_id: impl Into<String>) {
        self.attributes.insert(
            AttributeKey::core("agent_id"),
            AttributeValue::String(agent_id.into())
        );
    }
    
    /// Set user_id attribute
    pub fn set_user_id(&mut self, user_id: impl Into<String>) {
        self.attributes.insert(
            AttributeKey::core("user_id"),
            AttributeValue::String(user_id.into())
        );
    }
    
    /// Set memory_type attribute
    pub fn set_memory_type(&mut self, memory_type: impl Into<String>) {
        self.attributes.insert(
            AttributeKey::core("memory_type"),
            AttributeValue::String(memory_type.into())
        );
    }
    
    /// Set importance attribute
    pub fn set_importance(&mut self, importance: f64) {
        self.attributes.insert(
            AttributeKey::system("importance"),
            AttributeValue::Number(importance)
        );
    }
    
    /// Set score attribute
    pub fn set_score(&mut self, score: f64) {
        self.attributes.insert(
            AttributeKey::system("score"),
            AttributeValue::Number(score)
        );
    }
    
    /// Get organization_id from attributes
    pub fn organization_id(&self) -> Option<String> {
        self.attributes
            .get(&AttributeKey::core("organization_id"))
            .and_then(|v| v.as_string())
            .cloned()
    }
    
    /// Get scope from attributes
    pub fn scope(&self) -> Option<String> {
        self.attributes
            .get(&AttributeKey::core("scope"))
            .and_then(|v| v.as_string())
            .cloned()
    }
    
    /// Get level from attributes
    pub fn level(&self) -> Option<String> {
        self.attributes
            .get(&AttributeKey::core("level"))
            .and_then(|v| v.as_string())
            .cloned()
    }
    
    /// Get is_deleted from attributes
    pub fn is_deleted(&self) -> bool {
        self.attributes
            .get(&AttributeKey::system("is_deleted"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }
    
    /// Get created_by_id from attributes
    pub fn created_by_id(&self) -> Option<String> {
        self.attributes
            .get(&AttributeKey::system("created_by_id"))
            .and_then(|v| v.as_string())
            .cloned()
    }
    
    /// Get last_updated_by_id from attributes
    pub fn last_updated_by_id(&self) -> Option<String> {
        self.attributes
            .get(&AttributeKey::system("last_updated_by_id"))
            .and_then(|v| v.as_string())
            .cloned()
    }
}

