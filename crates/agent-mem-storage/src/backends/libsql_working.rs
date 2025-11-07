//! LibSQL implementation of WorkingMemoryStore
//!
//! This implementation uses the unified memories table with memory_type='working'
//! This follows AgentMem's unified memory model design philosophy.

use agent_mem_traits::{AgentMemError, Result, WorkingMemoryItem, WorkingMemoryStore};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use libsql::{params, Connection, Row};
use std::sync::Arc;
use tokio::sync::Mutex;

/// LibSQL implementation of WorkingMemoryStore
/// Uses the unified memories table with memory_type='working'
pub struct LibSqlWorkingStore {
    conn: Arc<Mutex<Connection>>,
}

impl LibSqlWorkingStore {
    /// Create a new LibSQL working memory store
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
}

/// Convert LibSQL row to WorkingMemoryItem
/// Maps from memories table columns to WorkingMemoryItem fields
fn row_to_item(row: &Row) -> Result<WorkingMemoryItem> {
    // Memories table schema:
    // id, organization_id, user_id, agent_id, content, hash, metadata, score,
    // memory_type, scope, level, importance, access_count, last_accessed,
    // embedding, expires_at, version, created_at, updated_at, is_deleted, created_by_id, last_updated_by_id, session_id

    let id: String = row
        .get(0)
        .map_err(|e| AgentMemError::storage_error(format!("Failed to get id: {e}")))?;

    let user_id: String = row
        .get(2)
        .map_err(|e| AgentMemError::storage_error(format!("Failed to get user_id: {e}")))?;

    let agent_id: String = row
        .get(3)
        .map_err(|e| AgentMemError::storage_error(format!("Failed to get agent_id: {e}")))?;

    let content: String = row
        .get(4)
        .map_err(|e| AgentMemError::storage_error(format!("Failed to get content: {e}")))?;

    let metadata_json: Option<String> = row.get(6).ok();
    let metadata: serde_json::Value = metadata_json
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

    // importance maps to priority
    let importance: f64 = row.get(11).unwrap_or(1.0);
    let priority = importance as i32;

    // expires_at is INTEGER (timestamp)
    let expires_at_ts: Option<i64> = row.get(15).ok();
    let expires_at =
        expires_at_ts.map(|ts| DateTime::from_timestamp(ts, 0).unwrap_or_else(|| Utc::now()));

    // created_at is INTEGER (timestamp)
    let created_at_ts: i64 = row
        .get(17)
        .map_err(|e| AgentMemError::storage_error(format!("Failed to get created_at: {e}")))?;
    let created_at = DateTime::from_timestamp(created_at_ts, 0).unwrap_or_else(|| Utc::now());

    // session_id (new column, index 22)
    let session_id: String = row
        .get(22)
        .map_err(|e| AgentMemError::storage_error(format!("Failed to get session_id: {e}")))?;

    Ok(WorkingMemoryItem {
        id,
        user_id,
        agent_id,
        session_id,
        content,
        priority,
        expires_at,
        metadata,
        created_at,
    })
}

#[async_trait]
impl WorkingMemoryStore for LibSqlWorkingStore {
    async fn add_item(&self, item: WorkingMemoryItem) -> Result<WorkingMemoryItem> {
        let conn = self.conn.lock().await;

        let metadata_json = serde_json::to_string(&item.metadata).map_err(|e| {
            AgentMemError::storage_error(format!("Failed to serialize metadata: {e}"))
        })?;

        let expires_at_ts = item.expires_at.map(|dt| dt.timestamp());
        let created_at_ts = item.created_at.timestamp();

        // Default organization_id for working memory (can be enhanced later)
        let organization_id = "default-org";

        conn.execute(
            r#"
            INSERT INTO memories (
                id, organization_id, user_id, agent_id, content,
                metadata, memory_type, scope, level, importance,
                expires_at, created_at, updated_at, is_deleted, session_id
            )
            VALUES (?, ?, ?, ?, ?, ?, 'working', 'session', 'temporary', ?, ?, ?, ?, 0, ?)
            "#,
            params![
                item.id.clone(),
                organization_id,
                item.user_id.clone(),
                item.agent_id.clone(),
                item.content.clone(),
                metadata_json,
                item.priority as f64, // importance
                expires_at_ts,
                created_at_ts,
                created_at_ts, // updated_at
                item.session_id.clone(),
            ],
        )
        .await
        .map_err(|e| {
            AgentMemError::storage_error(format!("Failed to add working memory item: {e}"))
        })?;

        Ok(item)
    }

    async fn get_session_items(&self, session_id: &str) -> Result<Vec<WorkingMemoryItem>> {
        let conn = self.conn.lock().await;

        let now_ts = Utc::now().timestamp();

        let mut stmt = conn
            .prepare(
                r#"
                SELECT * FROM memories
                WHERE session_id = ?
                AND memory_type = 'working'
                AND is_deleted = 0
                AND (expires_at IS NULL OR expires_at > ?)
                ORDER BY importance DESC, created_at ASC
                "#,
            )
            .await
            .map_err(|e| {
                AgentMemError::storage_error(format!("Failed to prepare statement: {e}"))
            })?;

        let mut rows = stmt
            .query(params![session_id, now_ts])
            .await
            .map_err(|e| AgentMemError::storage_error(format!("Failed to execute query: {e}")))?;

        let mut results = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::storage_error(format!("Failed to fetch row: {e}")))?
        {
            results.push(row_to_item(&row)?);
        }

        Ok(results)
    }

    async fn remove_item(&self, item_id: &str) -> Result<bool> {
        let conn = self.conn.lock().await;

        let result = conn
            .execute(
                "UPDATE memories SET is_deleted = 1 WHERE id = ? AND memory_type = 'working'",
                params![item_id],
            )
            .await
            .map_err(|e| {
                AgentMemError::storage_error(format!("Failed to remove working memory item: {e}"))
            })?;

        Ok(result > 0)
    }

    async fn clear_expired(&self) -> Result<i64> {
        let conn = self.conn.lock().await;

        let now_ts = Utc::now().timestamp();

        let result = conn
            .execute(
                "UPDATE memories SET is_deleted = 1 WHERE memory_type = 'working' AND expires_at IS NOT NULL AND expires_at <= ?",
                params![now_ts],
            )
            .await
            .map_err(|e| AgentMemError::storage_error(format!("Failed to clear expired items: {e}")))?;

        Ok(result as i64)
    }

    async fn clear_session(&self, session_id: &str) -> Result<i64> {
        let conn = self.conn.lock().await;

        let result = conn
            .execute(
                "UPDATE memories SET is_deleted = 1 WHERE memory_type = 'working' AND session_id = ?",
                params![session_id],
            )
            .await
            .map_err(|e| AgentMemError::storage_error(format!("Failed to clear session: {e}")))?;

        Ok(result as i64)
    }

    async fn get_by_priority(
        &self,
        session_id: &str,
        min_priority: i32,
    ) -> Result<Vec<WorkingMemoryItem>> {
        let conn = self.conn.lock().await;

        let now_ts = Utc::now().timestamp();

        let mut stmt = conn
            .prepare(
                r#"
                SELECT * FROM memories
                WHERE session_id = ?
                AND memory_type = 'working'
                AND importance >= ?
                AND is_deleted = 0
                AND (expires_at IS NULL OR expires_at > ?)
                ORDER BY importance DESC, created_at ASC
                "#,
            )
            .await
            .map_err(|e| {
                AgentMemError::storage_error(format!("Failed to prepare statement: {e}"))
            })?;

        let mut rows = stmt
            .query(params![session_id, min_priority as f64, now_ts])
            .await
            .map_err(|e| AgentMemError::storage_error(format!("Failed to execute query: {e}")))?;

        let mut results = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::storage_error(format!("Failed to fetch row: {e}")))?
        {
            results.push(row_to_item(&row)?);
        }

        Ok(results)
    }
}
