//! Query type definitions
//! 
//! Defines query structures for searching and filtering memories

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::*;

/// Query intent classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum QueryIntent {
    /// Retrieve specific facts
    FactRetrieval,
    /// Get procedure/instruction
    ProcedureRetrieval,
    /// Find similar experiences
    ExperienceSearch,
    /// Get current context
    ContextRetrieval,
    /// General exploration
    Exploration,
    /// Unknown intent
    Unknown,
}

/// Aggregation operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationOp {
    Sum,
    Avg,
    Count,
    Max,
    Min,
    GroupBy,
}

/// Query constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    /// Field to constrain
    pub field: String,
    /// Comparison operator
    pub operator: ComparisonOperator,
    /// Value to compare against
    pub value: AttributeValue,
}

/// Comparison operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonOperator {
    Eq,      // Equal
    Ne,      // Not equal
    Gt,      // Greater than
    Ge,      // Greater or equal
    Lt,      // Less than
    Le,      // Less or equal
    Like,    // Like (pattern match)
    In,      // In list
    NotIn,   // Not in list
}

impl ComparisonOperator {
    /// Apply comparison
    pub fn apply<T: PartialOrd>(&self, left: &T, right: &T) -> bool {
        match self {
            ComparisonOperator::Eq => left == right,
            ComparisonOperator::Ne => left != right,
            ComparisonOperator::Gt => left > right,
            ComparisonOperator::Ge => left >= right,
            ComparisonOperator::Lt => left < right,
            ComparisonOperator::Le => left <= right,
            _ => false,
        }
    }
}

/// Preference types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PreferenceType {
    Temporal,
    Relevance,
    Diversity,
}

/// Temporal preference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalPreference {
    /// Recent bias (0.0-1.0)
    pub recency_weight: f32,
    /// Time window in seconds
    pub time_window: Option<u64>,
}

/// Relevance preference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelevancePreference {
    /// Minimum relevance score (0.0-1.0)
    pub min_score: f32,
    /// Maximum results
    pub max_results: Option<usize>,
}

/// Diversity preference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiversityPreference {
    /// Diversity weight (0.0-1.0)
    pub diversity_weight: f32,
    /// Minimum unique sources
    pub min_unique_sources: Option<usize>,
}

/// Preference configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preference {
    /// Preference type
    pub preference_type: PreferenceType,
    /// JSON configuration
    pub config: serde_json::Value,
}

impl Preference {
    /// Create temporal preference
    pub fn temporal(recency_weight: f32, time_window: Option<u64>) -> Self {
        Self {
            preference_type: PreferenceType::Temporal,
            config: serde_json::json!({
                "recency_weight": recency_weight,
                "time_window": time_window
            }),
        }
    }

    /// Create relevance preference
    pub fn relevance(min_score: f32, max_results: Option<usize>) -> Self {
        Self {
            preference_type: PreferenceType::Relevance,
            config: serde_json::json!({
                "min_score": min_score,
                "max_results": max_results
            }),
        }
    }

    /// Create diversity preference
    pub fn diversity(diversity_weight: f32, min_unique_sources: Option<usize>) -> Self {
        Self {
            preference_type: PreferenceType::Diversity,
            config: serde_json::json!({
                "diversity_weight": diversity_weight,
                "min_unique_sources": min_unique_sources
            }),
        }
    }
}

/// Query context for adaptive search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryContext {
    /// Current user query
    pub query: String,
    /// Conversation history
    pub history: Vec<String>,
    /// User preferences
    pub preferences: Vec<Preference>,
    /// Session metadata
    pub session_meta: HashMap<String, serde_json::Value>,
}

impl QueryContext {
    /// Create new query context
    pub fn new(query: String) -> Self {
        Self {
            query,
            history: Vec::new(),
            preferences: Vec::new(),
            session_meta: HashMap::new(),
        }
    }

    /// Add to history
    pub fn add_to_history(&mut self, text: String) {
        self.history.push(text);
    }

    /// Add preference
    pub fn add_preference(&mut self, pref: Preference) {
        self.preferences.push(pref);
    }
}

/// Core query structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    /// Query text
    pub text: String,
    /// Memory types to search
    pub memory_types: Vec<MemoryType>,
    /// Constraints
    pub constraints: Vec<Constraint>,
    /// Query intent (optional)
    pub intent: Option<QueryIntent>,
    /// Context for adaptive search
    pub context: Option<QueryContext>,
    /// Preferences
    pub preferences: Vec<Preference>,
    /// Limit results
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
}

impl Query {
    /// Create new query
    pub fn new<S: Into<String>>(text: S) -> Self {
        Self {
            text: text.into(),
            memory_types: Vec::new(),
            constraints: Vec::new(),
            intent: None,
            context: None,
            preferences: Vec::new(),
            limit: None,
            offset: None,
        }
    }

    /// Add memory type filter
    pub fn with_memory_type(mut self, mt: MemoryType) -> Self {
        self.memory_types.push(mt);
        self
    }

    /// Add constraint
    pub fn with_constraint(mut self, constraint: Constraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    /// Set limit
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set offset
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }
}

/// Match type for search results
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MatchType {
    Exact,
    Partial,
    Fuzzy,
    Semantic,
}

/// Memory search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchResult {
    /// Memory ID
    pub id: String,
    /// Content preview
    pub content: String,
    /// Relevance score (0.0-1.0)
    pub score: f32,
    /// Match type
    pub match_type: MatchType,
    /// Memory type
    pub memory_type: MemoryType,
    /// Metadata
    pub metadata: Option<Metadata>,
}

impl MemorySearchResult {
    /// Create new result
    pub fn new(id: String, content: String, score: f32, memory_type: MemoryType) -> Self {
        Self {
            id,
            content,
            score,
            match_type: MatchType::Semantic,
            memory_type,
            metadata: None,
        }
    }
}

/// Memory statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Total memories
    pub total: u64,
    /// By memory type
    pub by_type: HashMap<String, u64>,
    /// Total size in bytes
    pub total_size: u64,
    /// Average importance
    pub avg_importance: f32,
}

impl Default for MemoryStats {
    fn default() -> Self {
        Self {
            total: 0,
            by_type: HashMap::new(),
            total_size: 0,
            avg_importance: 0.0,
        }
    }
}
