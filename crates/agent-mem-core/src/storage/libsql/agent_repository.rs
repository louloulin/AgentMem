//! LibSQL Agent Repository
//!
//! Provides LibSQL implementation of AgentRepositoryTrait

use agent_mem_traits::{AgentMemError, Result};
use async_trait::async_trait;
use libsql::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::storage::models::Agent;
use crate::storage::traits::AgentRepositoryTrait;

/// LibSQL implementation of Agent repository
pub struct LibSqlAgentRepository {
    conn: Arc<Mutex<Connection>>,
}

impl LibSqlAgentRepository {
    /// Create a new LibSQL agent repository
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Helper function to serialize JSON fields
    fn serialize_json(value: &Option<serde_json::Value>) -> Option<String> {
        value.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default())
    }

    /// Helper function to serialize Vec<String> to JSON
    fn serialize_vec(value: &Option<Vec<String>>) -> Option<String> {
        value.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default())
    }

    /// Helper function to deserialize JSON fields
    fn deserialize_json(value: Option<String>) -> Option<serde_json::Value> {
        value.and_then(|s| serde_json::from_str(&s).ok())
    }

    /// Helper function to deserialize Vec<String> from JSON
    fn deserialize_vec(value: Option<String>) -> Option<Vec<String>> {
        value.and_then(|s| serde_json::from_str(&s).ok())
    }
}

#[async_trait]
impl AgentRepositoryTrait for LibSqlAgentRepository {
    async fn create(&self, agent: &Agent) -> Result<Agent> {
        let conn = self.conn.lock().await;

        conn.execute(
            "INSERT INTO agents (
                id, organization_id, agent_type, name, description, system, topic,
                message_ids, metadata, llm_config, embedding_config, tool_rules, mcp_tools,
                state, last_active_at, error_message, created_at, updated_at, is_deleted,
                created_by_id, last_updated_by_id
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            libsql::params![
                agent.id.clone(),
                agent.organization_id.clone(),
                agent.agent_type.clone(),
                agent.name.clone(),
                agent.description.clone(),
                agent.system.clone(),
                agent.topic.clone(),
                Self::serialize_vec(&agent.message_ids),
                Self::serialize_json(&agent.metadata_),
                Self::serialize_json(&agent.llm_config),
                Self::serialize_json(&agent.embedding_config),
                Self::serialize_json(&agent.tool_rules),
                Self::serialize_vec(&agent.mcp_tools),
                agent.state.clone(),
                agent.last_active_at.map(|dt| dt.timestamp()),
                agent.error_message.clone(),
                agent.created_at.timestamp(),
                agent.updated_at.timestamp(),
                if agent.is_deleted { 1 } else { 0 },
                agent.created_by_id.clone(),
                agent.last_updated_by_id.clone(),
            ],
        )
        .await
        .map_err(|e| AgentMemError::StorageError(format!("Failed to create agent: {e}")))?;

        Ok(agent.clone())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<Agent>> {
        let conn = self.conn.lock().await;

        let mut stmt = conn
            .prepare(
                "SELECT id, organization_id, agent_type, name, description, system, topic,
                        message_ids, metadata, llm_config, embedding_config, tool_rules, mcp_tools,
                        state, last_active_at, error_message, created_at, updated_at, is_deleted,
                        created_by_id, last_updated_by_id
                 FROM agents WHERE id = ? AND is_deleted = 0"
            )
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to prepare statement: {e}")))?;

        let mut rows = stmt
            .query(libsql::params![id])
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to query agent: {e}")))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to fetch row: {e}")))?
        {
            let agent = Agent {
                id: row.get::<String>(0).map_err(|e| AgentMemError::StorageError(format!("Failed to get id: {e}")))?,
                organization_id: row.get::<String>(1).map_err(|e| AgentMemError::StorageError(format!("Failed to get organization_id: {e}")))?,
                agent_type: row.get::<Option<String>>(2).map_err(|e| AgentMemError::StorageError(format!("Failed to get agent_type: {e}")))?,
                name: row.get::<Option<String>>(3).map_err(|e| AgentMemError::StorageError(format!("Failed to get name: {e}")))?,
                description: row.get::<Option<String>>(4).map_err(|e| AgentMemError::StorageError(format!("Failed to get description: {e}")))?,
                system: row.get::<Option<String>>(5).map_err(|e| AgentMemError::StorageError(format!("Failed to get system: {e}")))?,
                topic: row.get::<Option<String>>(6).map_err(|e| AgentMemError::StorageError(format!("Failed to get topic: {e}")))?,
                message_ids: Self::deserialize_vec(row.get::<Option<String>>(7).map_err(|e| AgentMemError::StorageError(format!("Failed to get message_ids: {e}")))?),
                metadata_: Self::deserialize_json(row.get::<Option<String>>(8).map_err(|e| AgentMemError::StorageError(format!("Failed to get metadata_: {e}")))?),
                llm_config: Self::deserialize_json(row.get::<Option<String>>(9).map_err(|e| AgentMemError::StorageError(format!("Failed to get llm_config: {e}")))?),
                embedding_config: Self::deserialize_json(row.get::<Option<String>>(10).map_err(|e| AgentMemError::StorageError(format!("Failed to get embedding_config: {e}")))?),
                tool_rules: Self::deserialize_json(row.get::<Option<String>>(11).map_err(|e| AgentMemError::StorageError(format!("Failed to get tool_rules: {e}")))?),
                mcp_tools: Self::deserialize_vec(row.get::<Option<String>>(12).map_err(|e| AgentMemError::StorageError(format!("Failed to get mcp_tools: {e}")))?),
                state: row.get::<Option<String>>(13).map_err(|e| AgentMemError::StorageError(format!("Failed to get state: {e}")))?,
                last_active_at: row.get::<Option<i64>>(14).map_err(|e| AgentMemError::StorageError(format!("Failed to get last_active_at: {e}")))?.and_then(|ts| chrono::DateTime::from_timestamp(ts, 0)),
                error_message: row.get::<Option<String>>(15).map_err(|e| AgentMemError::StorageError(format!("Failed to get error_message: {e}")))?,
                created_at: chrono::DateTime::from_timestamp(
                    row.get::<i64>(16).map_err(|e| AgentMemError::StorageError(format!("Failed to get created_at: {e}")))?,
                    0,
                ).ok_or_else(|| AgentMemError::StorageError("Invalid created_at timestamp".to_string()))?,
                updated_at: chrono::DateTime::from_timestamp(
                    row.get::<i64>(17).map_err(|e| AgentMemError::StorageError(format!("Failed to get updated_at: {e}")))?,
                    0,
                ).ok_or_else(|| AgentMemError::StorageError("Invalid updated_at timestamp".to_string()))?,
                is_deleted: row.get::<i64>(18).map_err(|e| AgentMemError::StorageError(format!("Failed to get is_deleted: {e}")))? != 0,
                created_by_id: row.get::<Option<String>>(19).map_err(|e| AgentMemError::StorageError(format!("Failed to get created_by_id: {e}")))?,
                last_updated_by_id: row.get::<Option<String>>(20).map_err(|e| AgentMemError::StorageError(format!("Failed to get last_updated_by_id: {e}")))?,
            };
            Ok(Some(agent))
        } else {
            Ok(None)
        }
    }

    async fn find_by_organization_id(&self, org_id: &str) -> Result<Vec<Agent>> {
        let conn = self.conn.lock().await;

        let mut stmt = conn
            .prepare(
                "SELECT id, organization_id, agent_type, name, description, system, topic,
                        message_ids, metadata, llm_config, embedding_config, tool_rules, mcp_tools,
                        state, last_active_at, error_message, created_at, updated_at, is_deleted,
                        created_by_id, last_updated_by_id
                 FROM agents WHERE organization_id = ? AND is_deleted = 0 ORDER BY created_at DESC"
            )
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to prepare statement: {e}")))?;

        let mut rows = stmt
            .query(libsql::params![org_id])
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to query agents: {e}")))?;

        let mut agents = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to fetch row: {e}")))?
        {
            let agent = Agent {
                id: row.get::<String>(0).map_err(|e| AgentMemError::StorageError(format!("Failed to get id: {e}")))?,
                organization_id: row.get::<String>(1).map_err(|e| AgentMemError::StorageError(format!("Failed to get organization_id: {e}")))?,
                agent_type: row.get::<Option<String>>(2).map_err(|e| AgentMemError::StorageError(format!("Failed to get agent_type: {e}")))?,
                name: row.get::<Option<String>>(3).map_err(|e| AgentMemError::StorageError(format!("Failed to get name: {e}")))?,
                description: row.get::<Option<String>>(4).map_err(|e| AgentMemError::StorageError(format!("Failed to get description: {e}")))?,
                system: row.get::<Option<String>>(5).map_err(|e| AgentMemError::StorageError(format!("Failed to get system: {e}")))?,
                topic: row.get::<Option<String>>(6).map_err(|e| AgentMemError::StorageError(format!("Failed to get topic: {e}")))?,
                message_ids: Self::deserialize_vec(row.get::<Option<String>>(7).map_err(|e| AgentMemError::StorageError(format!("Failed to get message_ids: {e}")))?),
                metadata_: Self::deserialize_json(row.get::<Option<String>>(8).map_err(|e| AgentMemError::StorageError(format!("Failed to get metadata_: {e}")))?),
                llm_config: Self::deserialize_json(row.get::<Option<String>>(9).map_err(|e| AgentMemError::StorageError(format!("Failed to get llm_config: {e}")))?),
                embedding_config: Self::deserialize_json(row.get::<Option<String>>(10).map_err(|e| AgentMemError::StorageError(format!("Failed to get embedding_config: {e}")))?),
                tool_rules: Self::deserialize_json(row.get::<Option<String>>(11).map_err(|e| AgentMemError::StorageError(format!("Failed to get tool_rules: {e}")))?),
                mcp_tools: Self::deserialize_vec(row.get::<Option<String>>(12).map_err(|e| AgentMemError::StorageError(format!("Failed to get mcp_tools: {e}")))?),
                state: row.get::<Option<String>>(13).map_err(|e| AgentMemError::StorageError(format!("Failed to get state: {e}")))?,
                last_active_at: row.get::<Option<i64>>(14).map_err(|e| AgentMemError::StorageError(format!("Failed to get last_active_at: {e}")))?.and_then(|ts| chrono::DateTime::from_timestamp(ts, 0)),
                error_message: row.get::<Option<String>>(15).map_err(|e| AgentMemError::StorageError(format!("Failed to get error_message: {e}")))?,
                created_at: chrono::DateTime::from_timestamp(
                    row.get::<i64>(16).map_err(|e| AgentMemError::StorageError(format!("Failed to get created_at: {e}")))?,
                    0,
                ).ok_or_else(|| AgentMemError::StorageError("Invalid created_at timestamp".to_string()))?,
                updated_at: chrono::DateTime::from_timestamp(
                    row.get::<i64>(17).map_err(|e| AgentMemError::StorageError(format!("Failed to get updated_at: {e}")))?,
                    0,
                ).ok_or_else(|| AgentMemError::StorageError("Invalid updated_at timestamp".to_string()))?,
                is_deleted: row.get::<i64>(18).map_err(|e| AgentMemError::StorageError(format!("Failed to get is_deleted: {e}")))? != 0,
                created_by_id: row.get::<Option<String>>(19).map_err(|e| AgentMemError::StorageError(format!("Failed to get created_by_id: {e}")))?,
                last_updated_by_id: row.get::<Option<String>>(20).map_err(|e| AgentMemError::StorageError(format!("Failed to get last_updated_by_id: {e}")))?,
            };
            agents.push(agent);
        }

        Ok(agents)
    }

    async fn update(&self, agent: &Agent) -> Result<Agent> {
        let conn = self.conn.lock().await;

        conn.execute(
            "UPDATE agents SET 
                agent_type = ?, name = ?, description = ?, system = ?, topic = ?,
                message_ids = ?, metadata = ?, llm_config = ?, embedding_config = ?,
                tool_rules = ?, mcp_tools = ?, state = ?, last_active_at = ?,
                error_message = ?, updated_at = ?, last_updated_by_id = ?
             WHERE id = ? AND is_deleted = 0",
            libsql::params![
                agent.agent_type.clone(),
                agent.name.clone(),
                agent.description.clone(),
                agent.system.clone(),
                agent.topic.clone(),
                Self::serialize_vec(&agent.message_ids),
                Self::serialize_json(&agent.metadata_),
                Self::serialize_json(&agent.llm_config),
                Self::serialize_json(&agent.embedding_config),
                Self::serialize_json(&agent.tool_rules),
                Self::serialize_vec(&agent.mcp_tools),
                agent.state.clone(),
                agent.last_active_at.map(|dt| dt.timestamp()),
                agent.error_message.clone(),
                agent.updated_at.timestamp(),
                agent.last_updated_by_id.clone(),
                agent.id.clone(),
            ],
        )
        .await
        .map_err(|e| AgentMemError::StorageError(format!("Failed to update agent: {e}")))?;

        Ok(agent.clone())
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().await;

        conn.execute(
            "UPDATE agents SET is_deleted = 1 WHERE id = ?",
            libsql::params![id],
        )
        .await
        .map_err(|e| AgentMemError::StorageError(format!("Failed to delete agent: {e}")))?;

        Ok(())
    }

    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Agent>> {
        let conn = self.conn.lock().await;

        let mut stmt = conn
            .prepare(
                "SELECT id, organization_id, agent_type, name, description, system, topic,
                        message_ids, metadata, llm_config, embedding_config, tool_rules, mcp_tools,
                        state, last_active_at, error_message, created_at, updated_at, is_deleted,
                        created_by_id, last_updated_by_id
                 FROM agents WHERE is_deleted = 0 ORDER BY created_at DESC LIMIT ? OFFSET ?"
            )
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to prepare statement: {e}")))?;

        let mut rows = stmt
            .query(libsql::params![limit, offset])
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to query agents: {e}")))?;

        let mut agents = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to fetch row: {e}")))?
        {
            let agent = Agent {
                id: row.get::<String>(0).map_err(|e| AgentMemError::StorageError(format!("Failed to get id: {e}")))?,
                organization_id: row.get::<String>(1).map_err(|e| AgentMemError::StorageError(format!("Failed to get organization_id: {e}")))?,
                agent_type: row.get::<Option<String>>(2).map_err(|e| AgentMemError::StorageError(format!("Failed to get agent_type: {e}")))?,
                name: row.get::<Option<String>>(3).map_err(|e| AgentMemError::StorageError(format!("Failed to get name: {e}")))?,
                description: row.get::<Option<String>>(4).map_err(|e| AgentMemError::StorageError(format!("Failed to get description: {e}")))?,
                system: row.get::<Option<String>>(5).map_err(|e| AgentMemError::StorageError(format!("Failed to get system: {e}")))?,
                topic: row.get::<Option<String>>(6).map_err(|e| AgentMemError::StorageError(format!("Failed to get topic: {e}")))?,
                message_ids: Self::deserialize_vec(row.get::<Option<String>>(7).map_err(|e| AgentMemError::StorageError(format!("Failed to get message_ids: {e}")))?),
                metadata_: Self::deserialize_json(row.get::<Option<String>>(8).map_err(|e| AgentMemError::StorageError(format!("Failed to get metadata_: {e}")))?),
                llm_config: Self::deserialize_json(row.get::<Option<String>>(9).map_err(|e| AgentMemError::StorageError(format!("Failed to get llm_config: {e}")))?),
                embedding_config: Self::deserialize_json(row.get::<Option<String>>(10).map_err(|e| AgentMemError::StorageError(format!("Failed to get embedding_config: {e}")))?),
                tool_rules: Self::deserialize_json(row.get::<Option<String>>(11).map_err(|e| AgentMemError::StorageError(format!("Failed to get tool_rules: {e}")))?),
                mcp_tools: Self::deserialize_vec(row.get::<Option<String>>(12).map_err(|e| AgentMemError::StorageError(format!("Failed to get mcp_tools: {e}")))?),
                state: row.get::<Option<String>>(13).map_err(|e| AgentMemError::StorageError(format!("Failed to get state: {e}")))?,
                last_active_at: row.get::<Option<i64>>(14).map_err(|e| AgentMemError::StorageError(format!("Failed to get last_active_at: {e}")))?.and_then(|ts| chrono::DateTime::from_timestamp(ts, 0)),
                error_message: row.get::<Option<String>>(15).map_err(|e| AgentMemError::StorageError(format!("Failed to get error_message: {e}")))?,
                created_at: chrono::DateTime::from_timestamp(
                    row.get::<i64>(16).map_err(|e| AgentMemError::StorageError(format!("Failed to get created_at: {e}")))?,
                    0,
                ).ok_or_else(|| AgentMemError::StorageError("Invalid created_at timestamp".to_string()))?,
                updated_at: chrono::DateTime::from_timestamp(
                    row.get::<i64>(17).map_err(|e| AgentMemError::StorageError(format!("Failed to get updated_at: {e}")))?,
                    0,
                ).ok_or_else(|| AgentMemError::StorageError("Invalid updated_at timestamp".to_string()))?,
                is_deleted: row.get::<i64>(18).map_err(|e| AgentMemError::StorageError(format!("Failed to get is_deleted: {e}")))? != 0,
                created_by_id: row.get::<Option<String>>(19).map_err(|e| AgentMemError::StorageError(format!("Failed to get created_by_id: {e}")))?,
                last_updated_by_id: row.get::<Option<String>>(20).map_err(|e| AgentMemError::StorageError(format!("Failed to get last_updated_by_id: {e}")))?,
            };
            agents.push(agent);
        }

        Ok(agents)
    }
}

