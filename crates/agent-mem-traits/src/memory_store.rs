//! Memory Store Traits
//!
//! Defines storage traits for different memory types (Episodic, Semantic, etc.)
//! These traits abstract the storage backend, allowing multiple implementations
//! (PostgreSQL, LibSQL, MongoDB, etc.)

use crate::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ============================================================================
// Episodic Memory Store
// ============================================================================

/// Episodic event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodicEvent {
    pub id: String,
    pub organization_id: String,
    pub user_id: String,
    pub agent_id: String,
    pub occurred_at: DateTime<Utc>,
    pub event_type: String,
    pub actor: Option<String>,
    pub summary: String,
    pub details: Option<String>,
    pub importance_score: f32,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Query parameters for episodic events
#[derive(Debug, Clone, Default)]
pub struct EpisodicQuery {
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub event_type: Option<String>,
    pub min_importance: Option<f32>,
    pub limit: Option<i64>,
}

/// Episodic memory storage trait
#[async_trait]
pub trait EpisodicMemoryStore: Send + Sync {
    /// Create a new episodic event
    async fn create_event(&self, event: EpisodicEvent) -> Result<EpisodicEvent>;

    /// Get an episodic event by ID
    async fn get_event(&self, event_id: &str, user_id: &str) -> Result<Option<EpisodicEvent>>;

    /// Query episodic events with filters
    async fn query_events(&self, user_id: &str, query: EpisodicQuery)
        -> Result<Vec<EpisodicEvent>>;

    /// Update an episodic event
    async fn update_event(&self, event: EpisodicEvent) -> Result<bool>;

    /// Delete an episodic event
    async fn delete_event(&self, event_id: &str, user_id: &str) -> Result<bool>;

    /// Update importance score
    async fn update_importance(
        &self,
        event_id: &str,
        user_id: &str,
        importance_score: f32,
    ) -> Result<bool>;

    /// Count events in time range
    async fn count_events_in_range(
        &self,
        user_id: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<i64>;

    /// Get recent events
    async fn get_recent_events(&self, user_id: &str, limit: i64) -> Result<Vec<EpisodicEvent>>;
}

// ============================================================================
// Semantic Memory Store
// ============================================================================

/// Semantic memory item structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticMemoryItem {
    pub id: String,
    pub organization_id: String,
    pub user_id: String,
    pub agent_id: String,
    pub name: String,
    pub summary: String,
    pub details: Option<String>,
    pub source: Option<String>,
    pub tree_path: Vec<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Query parameters for semantic memory
#[derive(Debug, Clone, Default)]
pub struct SemanticQuery {
    pub name_query: Option<String>,
    pub summary_query: Option<String>,
    pub tree_path_prefix: Option<Vec<String>>,
    pub limit: Option<i64>,
}

/// Semantic memory storage trait
#[async_trait]
pub trait SemanticMemoryStore: Send + Sync {
    /// Create a new semantic memory item
    async fn create_item(&self, item: SemanticMemoryItem) -> Result<SemanticMemoryItem>;

    /// Get a semantic memory item by ID
    async fn get_item(&self, item_id: &str, user_id: &str) -> Result<Option<SemanticMemoryItem>>;

    /// Query semantic memory items with filters
    async fn query_items(
        &self,
        user_id: &str,
        query: SemanticQuery,
    ) -> Result<Vec<SemanticMemoryItem>>;

    /// Update a semantic memory item
    async fn update_item(&self, item: SemanticMemoryItem) -> Result<bool>;

    /// Delete a semantic memory item
    async fn delete_item(&self, item_id: &str, user_id: &str) -> Result<bool>;

    /// Search by tree path
    async fn search_by_tree_path(
        &self,
        user_id: &str,
        tree_path: Vec<String>,
    ) -> Result<Vec<SemanticMemoryItem>>;

    /// Get items by name pattern
    async fn search_by_name(
        &self,
        user_id: &str,
        name_pattern: &str,
        limit: i64,
    ) -> Result<Vec<SemanticMemoryItem>>;
}

// ============================================================================
// Procedural Memory Store
// ============================================================================

/// Procedural memory item structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProceduralMemoryItem {
    pub id: String,
    pub organization_id: String,
    pub user_id: String,
    pub agent_id: String,
    pub skill_name: String,
    pub description: String,
    pub steps: Vec<String>,
    pub success_rate: f32,
    pub execution_count: i32,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Query parameters for procedural memory
#[derive(Debug, Clone, Default)]
pub struct ProceduralQuery {
    pub skill_name_pattern: Option<String>,
    pub min_success_rate: Option<f32>,
    pub limit: Option<i64>,
}

/// Procedural memory storage trait
#[async_trait]
pub trait ProceduralMemoryStore: Send + Sync {
    /// Create a new procedural memory item
    async fn create_item(&self, item: ProceduralMemoryItem) -> Result<ProceduralMemoryItem>;

    /// Get a procedural memory item by ID
    async fn get_item(&self, item_id: &str, user_id: &str) -> Result<Option<ProceduralMemoryItem>>;

    /// Query procedural memory items with filters
    async fn query_items(
        &self,
        user_id: &str,
        query: ProceduralQuery,
    ) -> Result<Vec<ProceduralMemoryItem>>;

    /// Update a procedural memory item
    async fn update_item(&self, item: ProceduralMemoryItem) -> Result<bool>;

    /// Delete a procedural memory item
    async fn delete_item(&self, item_id: &str, user_id: &str) -> Result<bool>;

    /// Update execution statistics
    async fn update_execution_stats(
        &self,
        item_id: &str,
        user_id: &str,
        success: bool,
    ) -> Result<bool>;

    /// Get top performing skills
    async fn get_top_skills(&self, user_id: &str, limit: i64) -> Result<Vec<ProceduralMemoryItem>>;
}

// ============================================================================
// Working Memory Store
// ============================================================================

/// Working memory item structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingMemoryItem {
    pub id: String,
    pub user_id: String,
    pub agent_id: String,
    pub session_id: String,
    pub content: String,
    pub priority: i32,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Working memory storage trait
#[async_trait]
pub trait WorkingMemoryStore: Send + Sync {
    /// Add item to working memory
    async fn add_item(&self, item: WorkingMemoryItem) -> Result<WorkingMemoryItem>;

    /// Get working memory items for a session
    async fn get_session_items(&self, session_id: &str) -> Result<Vec<WorkingMemoryItem>>;

    /// Remove item from working memory
    async fn remove_item(&self, item_id: &str) -> Result<bool>;

    /// Clear expired items
    async fn clear_expired(&self) -> Result<i64>;

    /// Clear session working memory
    async fn clear_session(&self, session_id: &str) -> Result<i64>;

    /// Get items by priority
    async fn get_by_priority(
        &self,
        session_id: &str,
        min_priority: i32,
    ) -> Result<Vec<WorkingMemoryItem>>;
}

// ============================================================================
// Core Memory Store
// ============================================================================

/// Core memory item structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreMemoryItem {
    pub id: String,
    pub user_id: String,
    pub agent_id: String,
    pub key: String,
    pub value: String,
    pub category: String,
    pub is_mutable: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Core memory storage trait
#[async_trait]
pub trait CoreMemoryStore: Send + Sync {
    /// Set a core memory value
    async fn set_value(&self, item: CoreMemoryItem) -> Result<CoreMemoryItem>;

    /// Get a core memory value by key
    async fn get_value(&self, user_id: &str, key: &str) -> Result<Option<CoreMemoryItem>>;

    /// Get all core memory items for a user
    async fn get_all(&self, user_id: &str) -> Result<Vec<CoreMemoryItem>>;

    /// Get core memory items by category
    async fn get_by_category(&self, user_id: &str, category: &str) -> Result<Vec<CoreMemoryItem>>;

    /// Delete a core memory value
    async fn delete_value(&self, user_id: &str, key: &str) -> Result<bool>;

    /// Update a core memory value
    async fn update_value(&self, user_id: &str, key: &str, value: &str) -> Result<bool>;
}
