//! Database models for AgentMem
//!
//! This module defines the database schema and models for AgentMem,
//! inspired by MIRIX's design but implemented in Rust with SQLx.
//!
//! Key design principles:
//! - Multi-tenancy through organization_id
//! - Soft deletes with is_deleted flag
//! - Audit trail with created_by_id and last_updated_by_id
//! - Timestamps for created_at and updated_at
//! - Foreign key relationships for data integrity

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
#[cfg(feature = "postgres")]
use sqlx::FromRow;
use uuid::Uuid;

/// Organization model - the highest level of the object tree
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "postgres", derive(FromRow))]
pub struct Organization {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

/// User model - represents a user within an organization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "postgres", derive(FromRow))]
pub struct User {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub email: String,
    pub password_hash: String,
    pub roles: Option<Vec<String>>, // JSON array of roles
    pub status: String,             // "active" or "inactive"
    pub timezone: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
    pub created_by_id: Option<String>,
    pub last_updated_by_id: Option<String>,
}

/// Agent model - represents an AI agent
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "postgres", derive(FromRow))]
pub struct Agent {
    pub id: String,
    pub organization_id: String,
    pub agent_type: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub system: Option<String>, // System prompt
    pub topic: Option<String>,  // Current topic
    #[cfg_attr(feature = "postgres", sqlx(json))]
    pub message_ids: Option<Vec<String>>, // In-context message IDs
    #[cfg_attr(feature = "postgres", sqlx(json))]
    pub metadata_: Option<JsonValue>,
    #[cfg_attr(feature = "postgres", sqlx(json))]
    pub llm_config: Option<JsonValue>,
    #[cfg_attr(feature = "postgres", sqlx(json))]
    pub embedding_config: Option<JsonValue>,
    #[cfg_attr(feature = "postgres", sqlx(json))]
    pub tool_rules: Option<JsonValue>,
    #[cfg_attr(feature = "postgres", sqlx(json))]
    pub mcp_tools: Option<Vec<String>>, // MCP server names
    pub state: Option<String>, // Agent state: idle, thinking, executing, waiting, error
    pub last_active_at: Option<DateTime<Utc>>, // Last activity timestamp
    pub error_message: Option<String>, // Error message if state is error
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
    pub created_by_id: Option<String>,
    pub last_updated_by_id: Option<String>,
}

/// Message model - represents a message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "postgres", derive(FromRow))]
pub struct Message {
    pub id: String,
    pub organization_id: String,
    pub user_id: String,
    pub agent_id: String,
    pub role: String, // "user", "assistant", "system", "tool"
    pub text: Option<String>,
    #[cfg_attr(feature = "postgres", sqlx(json))]
    pub content: Option<JsonValue>, // Message content parts
    pub model: Option<String>,
    pub name: Option<String>,
    #[cfg_attr(feature = "postgres", sqlx(json))]
    pub tool_calls: Option<JsonValue>,
    pub tool_call_id: Option<String>,
    pub step_id: Option<String>,
    pub otid: Option<String>, // Offline threading ID
    #[cfg_attr(feature = "postgres", sqlx(json))]
    pub tool_returns: Option<JsonValue>,
    pub group_id: Option<String>,
    pub sender_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
    pub created_by_id: Option<String>,
    pub last_updated_by_id: Option<String>,
}

/// Block model - represents a section of core memory
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "postgres", derive(FromRow))]
pub struct Block {
    pub id: String,
    pub organization_id: String,
    pub user_id: String,
    pub template_name: Option<String>,
    pub description: Option<String>,
    pub label: String, // "human", "persona", "system"
    pub is_template: bool,
    pub value: String,
    pub limit: i64, // Character limit
    #[cfg_attr(feature = "postgres", sqlx(json))]
    pub metadata_: Option<JsonValue>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
    pub created_by_id: Option<String>,
    pub last_updated_by_id: Option<String>,
}

/// Tool model - represents a tool that can be used by agents
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "postgres", derive(FromRow))]
pub struct Tool {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub description: Option<String>,
    #[cfg_attr(feature = "postgres", sqlx(json))]
    pub json_schema: Option<JsonValue>,
    pub source_type: Option<String>,
    pub source_code: Option<String>,
    #[cfg_attr(feature = "postgres", sqlx(json))]
    pub tags: Option<Vec<String>>,
    #[cfg_attr(feature = "postgres", sqlx(json))]
    pub metadata_: Option<JsonValue>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
    pub created_by_id: Option<String>,
    pub last_updated_by_id: Option<String>,
}

impl Tool {
    /// Create a new tool
    pub fn new(organization_id: String, name: String) -> Self {
        let now = Utc::now();
        Self {
            id: generate_id("tool"),
            organization_id,
            name,
            description: None,
            json_schema: None,
            source_type: None,
            source_code: None,
            tags: None,
            metadata_: None,
            created_at: now,
            updated_at: now,
            is_deleted: false,
            created_by_id: None,
            last_updated_by_id: None,
        }
    }
}

/// Memory model - enhanced version with agent and user relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "postgres", derive(FromRow))]
pub struct Memory {
    pub id: String,
    pub organization_id: String,
    pub user_id: String,
    pub agent_id: String,
    pub content: String,
    pub hash: Option<String>,
    #[cfg_attr(feature = "postgres", sqlx(json))]
    pub metadata: JsonValue,
    pub score: Option<f32>,
    pub memory_type: String,
    pub scope: String,
    pub level: String,
    pub importance: f32,
    pub access_count: i64,
    pub last_accessed: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
    pub created_by_id: Option<String>,
    pub last_updated_by_id: Option<String>,
}

/// API Key model - represents an API key for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "postgres", derive(FromRow))]
pub struct ApiKey {
    pub id: String,
    pub key_hash: String, // Store hash, not the actual key
    pub name: String,
    pub user_id: String,
    pub organization_id: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

/// Junction table for blocks and agents (many-to-many)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "postgres", derive(FromRow))]
pub struct BlocksAgents {
    pub block_id: String,
    pub block_label: String,
    pub agent_id: String,
}

/// Junction table for tools and agents (many-to-many)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "postgres", derive(FromRow))]
pub struct ToolsAgents {
    pub tool_id: String,
    pub agent_id: String,
}

/// Helper function to generate a new ID with prefix
pub fn generate_id(prefix: &str) -> String {
    format!("{}-{}", prefix, Uuid::new_v4())
}

impl Organization {
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: generate_id("org"),
            name,
            created_at: now,
            updated_at: now,
            is_deleted: false,
        }
    }
}

impl User {
    pub fn new(
        organization_id: String,
        name: String,
        email: String,
        password_hash: String,
        timezone: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: generate_id("user"),
            organization_id,
            name,
            email,
            password_hash,
            roles: Some(vec!["user".to_string()]), // Default role
            status: "active".to_string(),
            timezone,
            created_at: now,
            updated_at: now,
            is_deleted: false,
            created_by_id: None,
            last_updated_by_id: None,
        }
    }
}

impl Agent {
    pub fn new(organization_id: String, name: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: generate_id("agent"),
            organization_id,
            agent_type: None,
            name,
            description: None,
            system: None,
            topic: None,
            message_ids: None,
            metadata_: None,
            llm_config: None,
            embedding_config: None,
            tool_rules: None,
            mcp_tools: None,
            state: Some("idle".to_string()),
            last_active_at: None,
            error_message: None,
            created_at: now,
            updated_at: now,
            is_deleted: false,
            created_by_id: None,
            last_updated_by_id: None,
        }
    }
}

impl Message {
    pub fn new(
        organization_id: String,
        user_id: String,
        agent_id: String,
        role: String,
        text: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: generate_id("message"),
            organization_id,
            user_id,
            agent_id,
            role,
            text,
            content: None,
            model: None,
            name: None,
            tool_calls: None,
            tool_call_id: None,
            step_id: None,
            otid: None,
            tool_returns: None,
            group_id: None,
            sender_id: None,
            created_at: now,
            updated_at: now,
            is_deleted: false,
            created_by_id: None,
            last_updated_by_id: None,
        }
    }
}

impl Block {
    pub fn new(
        organization_id: String,
        user_id: String,
        label: String,
        value: String,
        limit: i64,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: generate_id("block"),
            organization_id,
            user_id,
            template_name: None,
            description: None,
            label,
            is_template: false,
            value,
            limit,
            metadata_: None,
            created_at: now,
            updated_at: now,
            is_deleted: false,
            created_by_id: None,
            last_updated_by_id: None,
        }
    }
}

impl ApiKey {
    pub fn new(
        organization_id: String,
        user_id: String,
        name: String,
        key_hash: String,
        expires_at: Option<DateTime<Utc>>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: generate_id("apikey"),
            key_hash,
            name,
            user_id,
            organization_id,
            expires_at,
            last_used_at: None,
            created_at: now,
            updated_at: now,
            is_deleted: false,
        }
    }
}
