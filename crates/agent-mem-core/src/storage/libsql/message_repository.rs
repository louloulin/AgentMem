//! LibSQL Message Repository
//!
//! Provides LibSQL implementation of MessageRepositoryTrait

use agent_mem_traits::{AgentMemError, Result};
use async_trait::async_trait;
use libsql::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::storage::models::Message;
use crate::storage::traits::MessageRepositoryTrait;

/// LibSQL implementation of Message repository
pub struct LibSqlMessageRepository {
    conn: Arc<Mutex<Connection>>,
}

impl LibSqlMessageRepository {
    /// Create a new LibSQL message repository
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Helper function to serialize JSON fields
    fn serialize_json(value: &Option<serde_json::Value>) -> Option<String> {
        value.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default())
    }

    /// Helper function to deserialize JSON fields
    fn deserialize_json(value: Option<String>) -> Option<serde_json::Value> {
        value.and_then(|s| serde_json::from_str(&s).ok())
    }
}

#[async_trait]
impl MessageRepositoryTrait for LibSqlMessageRepository {
    async fn create(&self, message: &Message) -> Result<Message> {
        let conn = self.conn.lock().await;

        conn.execute(
            "INSERT INTO messages (
                id, organization_id, user_id, agent_id, role, text, content, model, name,
                tool_calls, tool_call_id, step_id, otid, tool_returns, group_id, sender_id,
                created_at, updated_at, is_deleted, created_by_id, last_updated_by_id
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            libsql::params![
                message.id.clone(),
                message.organization_id.clone(),
                message.user_id.clone(),
                message.agent_id.clone(),
                message.role.clone(),
                message.text.clone(),
                Self::serialize_json(&message.content),
                message.model.clone(),
                message.name.clone(),
                Self::serialize_json(&message.tool_calls),
                message.tool_call_id.clone(),
                message.step_id.clone(),
                message.otid.clone(),
                Self::serialize_json(&message.tool_returns),
                message.group_id.clone(),
                message.sender_id.clone(),
                message.created_at.timestamp(),
                message.updated_at.timestamp(),
                if message.is_deleted { 1 } else { 0 },
                message.created_by_id.clone(),
                message.last_updated_by_id.clone(),
            ],
        )
        .await
        .map_err(|e| AgentMemError::StorageError(format!("Failed to create message: {}", e)))?;

        Ok(message.clone())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<Message>> {
        let conn = self.conn.lock().await;

        let mut stmt = conn
            .prepare(
                "SELECT id, organization_id, user_id, agent_id, role, text, content, model, name,
                        tool_calls, tool_call_id, step_id, otid, tool_returns, group_id, sender_id,
                        created_at, updated_at, is_deleted, created_by_id, last_updated_by_id
                 FROM messages WHERE id = ? AND is_deleted = 0"
            )
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to prepare statement: {}", e)))?;

        let mut rows = stmt
            .query(libsql::params![id])
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to query message: {}", e)))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to fetch row: {}", e)))?
        {
            let message = Message {
                id: row.get::<String>(0).map_err(|e| AgentMemError::StorageError(format!("Failed to get id: {}", e)))?,
                organization_id: row.get::<String>(1).map_err(|e| AgentMemError::StorageError(format!("Failed to get organization_id: {}", e)))?,
                user_id: row.get::<String>(2).map_err(|e| AgentMemError::StorageError(format!("Failed to get user_id: {}", e)))?,
                agent_id: row.get::<String>(3).map_err(|e| AgentMemError::StorageError(format!("Failed to get agent_id: {}", e)))?,
                role: row.get::<String>(4).map_err(|e| AgentMemError::StorageError(format!("Failed to get role: {}", e)))?,
                text: row.get::<Option<String>>(5).map_err(|e| AgentMemError::StorageError(format!("Failed to get text: {}", e)))?,
                content: Self::deserialize_json(row.get::<Option<String>>(6).map_err(|e| AgentMemError::StorageError(format!("Failed to get content: {}", e)))?),
                model: row.get::<Option<String>>(7).map_err(|e| AgentMemError::StorageError(format!("Failed to get model: {}", e)))?,
                name: row.get::<Option<String>>(8).map_err(|e| AgentMemError::StorageError(format!("Failed to get name: {}", e)))?,
                tool_calls: Self::deserialize_json(row.get::<Option<String>>(9).map_err(|e| AgentMemError::StorageError(format!("Failed to get tool_calls: {}", e)))?),
                tool_call_id: row.get::<Option<String>>(10).map_err(|e| AgentMemError::StorageError(format!("Failed to get tool_call_id: {}", e)))?,
                step_id: row.get::<Option<String>>(11).map_err(|e| AgentMemError::StorageError(format!("Failed to get step_id: {}", e)))?,
                otid: row.get::<Option<String>>(12).map_err(|e| AgentMemError::StorageError(format!("Failed to get otid: {}", e)))?,
                tool_returns: Self::deserialize_json(row.get::<Option<String>>(13).map_err(|e| AgentMemError::StorageError(format!("Failed to get tool_returns: {}", e)))?),
                group_id: row.get::<Option<String>>(14).map_err(|e| AgentMemError::StorageError(format!("Failed to get group_id: {}", e)))?,
                sender_id: row.get::<Option<String>>(15).map_err(|e| AgentMemError::StorageError(format!("Failed to get sender_id: {}", e)))?,
                created_at: chrono::DateTime::from_timestamp(
                    row.get::<i64>(16).map_err(|e| AgentMemError::StorageError(format!("Failed to get created_at: {}", e)))?,
                    0,
                ).ok_or_else(|| AgentMemError::StorageError("Invalid created_at timestamp".to_string()))?,
                updated_at: chrono::DateTime::from_timestamp(
                    row.get::<i64>(17).map_err(|e| AgentMemError::StorageError(format!("Failed to get updated_at: {}", e)))?,
                    0,
                ).ok_or_else(|| AgentMemError::StorageError("Invalid updated_at timestamp".to_string()))?,
                is_deleted: row.get::<i64>(18).map_err(|e| AgentMemError::StorageError(format!("Failed to get is_deleted: {}", e)))? != 0,
                created_by_id: row.get::<Option<String>>(19).map_err(|e| AgentMemError::StorageError(format!("Failed to get created_by_id: {}", e)))?,
                last_updated_by_id: row.get::<Option<String>>(20).map_err(|e| AgentMemError::StorageError(format!("Failed to get last_updated_by_id: {}", e)))?,
            };
            Ok(Some(message))
        } else {
            Ok(None)
        }
    }

    async fn find_by_agent_id(&self, agent_id: &str, limit: i64) -> Result<Vec<Message>> {
        let conn = self.conn.lock().await;

        let mut stmt = conn
            .prepare(
                "SELECT id, organization_id, user_id, agent_id, role, text, content, model, name,
                        tool_calls, tool_call_id, step_id, otid, tool_returns, group_id, sender_id,
                        created_at, updated_at, is_deleted, created_by_id, last_updated_by_id
                 FROM messages WHERE agent_id = ? AND is_deleted = 0 ORDER BY created_at DESC LIMIT ?"
            )
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to prepare statement: {}", e)))?;

        let mut rows = stmt
            .query(libsql::params![agent_id, limit])
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to query messages: {}", e)))?;

        let mut messages = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to fetch row: {}", e)))?
        {
            let message = Message {
                id: row.get::<String>(0).map_err(|e| AgentMemError::StorageError(format!("Failed to get id: {}", e)))?,
                organization_id: row.get::<String>(1).map_err(|e| AgentMemError::StorageError(format!("Failed to get organization_id: {}", e)))?,
                user_id: row.get::<String>(2).map_err(|e| AgentMemError::StorageError(format!("Failed to get user_id: {}", e)))?,
                agent_id: row.get::<String>(3).map_err(|e| AgentMemError::StorageError(format!("Failed to get agent_id: {}", e)))?,
                role: row.get::<String>(4).map_err(|e| AgentMemError::StorageError(format!("Failed to get role: {}", e)))?,
                text: row.get::<Option<String>>(5).map_err(|e| AgentMemError::StorageError(format!("Failed to get text: {}", e)))?,
                content: Self::deserialize_json(row.get::<Option<String>>(6).map_err(|e| AgentMemError::StorageError(format!("Failed to get content: {}", e)))?),
                model: row.get::<Option<String>>(7).map_err(|e| AgentMemError::StorageError(format!("Failed to get model: {}", e)))?,
                name: row.get::<Option<String>>(8).map_err(|e| AgentMemError::StorageError(format!("Failed to get name: {}", e)))?,
                tool_calls: Self::deserialize_json(row.get::<Option<String>>(9).map_err(|e| AgentMemError::StorageError(format!("Failed to get tool_calls: {}", e)))?),
                tool_call_id: row.get::<Option<String>>(10).map_err(|e| AgentMemError::StorageError(format!("Failed to get tool_call_id: {}", e)))?,
                step_id: row.get::<Option<String>>(11).map_err(|e| AgentMemError::StorageError(format!("Failed to get step_id: {}", e)))?,
                otid: row.get::<Option<String>>(12).map_err(|e| AgentMemError::StorageError(format!("Failed to get otid: {}", e)))?,
                tool_returns: Self::deserialize_json(row.get::<Option<String>>(13).map_err(|e| AgentMemError::StorageError(format!("Failed to get tool_returns: {}", e)))?),
                group_id: row.get::<Option<String>>(14).map_err(|e| AgentMemError::StorageError(format!("Failed to get group_id: {}", e)))?,
                sender_id: row.get::<Option<String>>(15).map_err(|e| AgentMemError::StorageError(format!("Failed to get sender_id: {}", e)))?,
                created_at: chrono::DateTime::from_timestamp(
                    row.get::<i64>(16).map_err(|e| AgentMemError::StorageError(format!("Failed to get created_at: {}", e)))?,
                    0,
                ).ok_or_else(|| AgentMemError::StorageError("Invalid created_at timestamp".to_string()))?,
                updated_at: chrono::DateTime::from_timestamp(
                    row.get::<i64>(17).map_err(|e| AgentMemError::StorageError(format!("Failed to get updated_at: {}", e)))?,
                    0,
                ).ok_or_else(|| AgentMemError::StorageError("Invalid updated_at timestamp".to_string()))?,
                is_deleted: row.get::<i64>(18).map_err(|e| AgentMemError::StorageError(format!("Failed to get is_deleted: {}", e)))? != 0,
                created_by_id: row.get::<Option<String>>(19).map_err(|e| AgentMemError::StorageError(format!("Failed to get created_by_id: {}", e)))?,
                last_updated_by_id: row.get::<Option<String>>(20).map_err(|e| AgentMemError::StorageError(format!("Failed to get last_updated_by_id: {}", e)))?,
            };
            messages.push(message);
        }

        Ok(messages)
    }

    async fn find_by_user_id(&self, user_id: &str, limit: i64) -> Result<Vec<Message>> {
        let conn = self.conn.lock().await;

        let mut stmt = conn
            .prepare(
                "SELECT id, organization_id, user_id, agent_id, role, text, content, model, name,
                        tool_calls, tool_call_id, step_id, otid, tool_returns, group_id, sender_id,
                        created_at, updated_at, is_deleted, created_by_id, last_updated_by_id
                 FROM messages WHERE user_id = ? AND is_deleted = 0 ORDER BY created_at DESC LIMIT ?"
            )
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to prepare statement: {}", e)))?;

        let mut rows = stmt
            .query(libsql::params![user_id, limit])
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to query messages: {}", e)))?;

        let mut messages = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to fetch row: {}", e)))?
        {
            let message = Message {
                id: row.get::<String>(0).map_err(|e| AgentMemError::StorageError(format!("Failed to get id: {}", e)))?,
                organization_id: row.get::<String>(1).map_err(|e| AgentMemError::StorageError(format!("Failed to get organization_id: {}", e)))?,
                user_id: row.get::<String>(2).map_err(|e| AgentMemError::StorageError(format!("Failed to get user_id: {}", e)))?,
                agent_id: row.get::<String>(3).map_err(|e| AgentMemError::StorageError(format!("Failed to get agent_id: {}", e)))?,
                role: row.get::<String>(4).map_err(|e| AgentMemError::StorageError(format!("Failed to get role: {}", e)))?,
                text: row.get::<Option<String>>(5).map_err(|e| AgentMemError::StorageError(format!("Failed to get text: {}", e)))?,
                content: Self::deserialize_json(row.get::<Option<String>>(6).map_err(|e| AgentMemError::StorageError(format!("Failed to get content: {}", e)))?),
                model: row.get::<Option<String>>(7).map_err(|e| AgentMemError::StorageError(format!("Failed to get model: {}", e)))?,
                name: row.get::<Option<String>>(8).map_err(|e| AgentMemError::StorageError(format!("Failed to get name: {}", e)))?,
                tool_calls: Self::deserialize_json(row.get::<Option<String>>(9).map_err(|e| AgentMemError::StorageError(format!("Failed to get tool_calls: {}", e)))?),
                tool_call_id: row.get::<Option<String>>(10).map_err(|e| AgentMemError::StorageError(format!("Failed to get tool_call_id: {}", e)))?,
                step_id: row.get::<Option<String>>(11).map_err(|e| AgentMemError::StorageError(format!("Failed to get step_id: {}", e)))?,
                otid: row.get::<Option<String>>(12).map_err(|e| AgentMemError::StorageError(format!("Failed to get otid: {}", e)))?,
                tool_returns: Self::deserialize_json(row.get::<Option<String>>(13).map_err(|e| AgentMemError::StorageError(format!("Failed to get tool_returns: {}", e)))?),
                group_id: row.get::<Option<String>>(14).map_err(|e| AgentMemError::StorageError(format!("Failed to get group_id: {}", e)))?,
                sender_id: row.get::<Option<String>>(15).map_err(|e| AgentMemError::StorageError(format!("Failed to get sender_id: {}", e)))?,
                created_at: chrono::DateTime::from_timestamp(
                    row.get::<i64>(16).map_err(|e| AgentMemError::StorageError(format!("Failed to get created_at: {}", e)))?,
                    0,
                ).ok_or_else(|| AgentMemError::StorageError("Invalid created_at timestamp".to_string()))?,
                updated_at: chrono::DateTime::from_timestamp(
                    row.get::<i64>(17).map_err(|e| AgentMemError::StorageError(format!("Failed to get updated_at: {}", e)))?,
                    0,
                ).ok_or_else(|| AgentMemError::StorageError("Invalid updated_at timestamp".to_string()))?,
                is_deleted: row.get::<i64>(18).map_err(|e| AgentMemError::StorageError(format!("Failed to get is_deleted: {}", e)))? != 0,
                created_by_id: row.get::<Option<String>>(19).map_err(|e| AgentMemError::StorageError(format!("Failed to get created_by_id: {}", e)))?,
                last_updated_by_id: row.get::<Option<String>>(20).map_err(|e| AgentMemError::StorageError(format!("Failed to get last_updated_by_id: {}", e)))?,
            };
            messages.push(message);
        }

        Ok(messages)
    }

    async fn update(&self, message: &Message) -> Result<Message> {
        let conn = self.conn.lock().await;

        conn.execute(
            "UPDATE messages SET 
                role = ?, text = ?, content = ?, model = ?, name = ?,
                tool_calls = ?, tool_call_id = ?, step_id = ?, otid = ?,
                tool_returns = ?, group_id = ?, sender_id = ?,
                updated_at = ?, last_updated_by_id = ?
             WHERE id = ? AND is_deleted = 0",
            libsql::params![
                message.role.clone(),
                message.text.clone(),
                Self::serialize_json(&message.content),
                message.model.clone(),
                message.name.clone(),
                Self::serialize_json(&message.tool_calls),
                message.tool_call_id.clone(),
                message.step_id.clone(),
                message.otid.clone(),
                Self::serialize_json(&message.tool_returns),
                message.group_id.clone(),
                message.sender_id.clone(),
                message.updated_at.timestamp(),
                message.last_updated_by_id.clone(),
                message.id.clone(),
            ],
        )
        .await
        .map_err(|e| AgentMemError::StorageError(format!("Failed to update message: {}", e)))?;

        Ok(message.clone())
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().await;

        conn.execute(
            "UPDATE messages SET is_deleted = 1 WHERE id = ?",
            libsql::params![id],
        )
        .await
        .map_err(|e| AgentMemError::StorageError(format!("Failed to delete message: {}", e)))?;

        Ok(())
    }

    async fn delete_by_agent_id(&self, agent_id: &str) -> Result<u64> {
        let conn = self.conn.lock().await;

        let rows_affected = conn.execute(
            "UPDATE messages SET is_deleted = 1 WHERE agent_id = ?",
            libsql::params![agent_id],
        )
        .await
        .map_err(|e| AgentMemError::StorageError(format!("Failed to delete messages by agent_id: {}", e)))?;

        Ok(rows_affected)
    }
}

