//! LibSQL implementation of WorkingMemoryStore

use agent_mem_traits::{AgentMemError, Result, WorkingMemoryItem, WorkingMemoryStore};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use libsql::{params, Connection, Row};
use std::sync::Arc;
use tokio::sync::Mutex;

/// LibSQL implementation of WorkingMemoryStore
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
fn row_to_item(row: &Row) -> Result<WorkingMemoryItem> {
    let metadata_json: String = row.get(7).map_err(|e| {
        AgentMemError::storage_error(&format!("Failed to get metadata: {}", e))
    })?;
    let metadata: serde_json::Value = serde_json::from_str(&metadata_json)
        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

    let created_at_str: String = row.get(8).map_err(|e| {
        AgentMemError::storage_error(&format!("Failed to get created_at: {}", e))
    })?;
    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to parse created_at: {}", e)))?
        .with_timezone(&Utc);

    let expires_at: Option<DateTime<Utc>> = row
        .get::<Option<String>>(6)
        .ok()
        .flatten()
        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let priority_i64: i64 = row.get(5).map_err(|e| {
        AgentMemError::storage_error(&format!("Failed to get priority: {}", e))
    })?;

    Ok(WorkingMemoryItem {
        id: row.get(0).map_err(|e| {
            AgentMemError::storage_error(&format!("Failed to get id: {}", e))
        })?,
        user_id: row.get(1).map_err(|e| {
            AgentMemError::storage_error(&format!("Failed to get user_id: {}", e))
        })?,
        agent_id: row.get(2).map_err(|e| {
            AgentMemError::storage_error(&format!("Failed to get agent_id: {}", e))
        })?,
        session_id: row.get(3).map_err(|e| {
            AgentMemError::storage_error(&format!("Failed to get session_id: {}", e))
        })?,
        content: row.get(4).map_err(|e| {
            AgentMemError::storage_error(&format!("Failed to get content: {}", e))
        })?,
        priority: priority_i64 as i32,
        expires_at,
        metadata,
        created_at,
    })
}

#[async_trait]
impl WorkingMemoryStore for LibSqlWorkingStore {
    async fn add_item(&self, item: WorkingMemoryItem) -> Result<WorkingMemoryItem> {
        let conn = self.conn.lock().await;

        let metadata_json = serde_json::to_string(&item.metadata)
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to serialize metadata: {}", e)))?;

        let expires_at_str = item.expires_at.map(|dt| dt.to_rfc3339());

        conn.execute(
            r#"
            INSERT INTO working_memory (
                id, user_id, agent_id, session_id, content,
                priority, expires_at, metadata, created_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                item.id.clone(),
                item.user_id.clone(),
                item.agent_id.clone(),
                item.session_id.clone(),
                item.content.clone(),
                item.priority as i64,
                expires_at_str,
                metadata_json,
                item.created_at.to_rfc3339(),
            ],
        )
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to add working memory item: {}", e)))?;

        Ok(item)
    }

    async fn get_session_items(&self, session_id: &str) -> Result<Vec<WorkingMemoryItem>> {
        let conn = self.conn.lock().await;

        let mut stmt = conn
            .prepare(
                r#"
                SELECT * FROM working_memory
                WHERE session_id = ?
                AND (expires_at IS NULL OR expires_at > datetime('now'))
                ORDER BY priority DESC, created_at ASC
                "#,
            )
            .await
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to prepare statement: {}", e)))?;

        let mut rows = stmt
            .query(params![session_id])
            .await
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to execute query: {}", e)))?;

        let mut results = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| {
            AgentMemError::storage_error(&format!("Failed to fetch row: {}", e))
        })? {
            results.push(row_to_item(&row)?);
        }

        Ok(results)
    }

    async fn remove_item(&self, item_id: &str) -> Result<bool> {
        let conn = self.conn.lock().await;

        let result = conn
            .execute("DELETE FROM working_memory WHERE id = ?", params![item_id])
            .await
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to remove working memory item: {}", e)))?;

        Ok(result > 0)
    }

    async fn clear_expired(&self) -> Result<i64> {
        let conn = self.conn.lock().await;

        let result = conn
            .execute(
                "DELETE FROM working_memory WHERE expires_at IS NOT NULL AND expires_at <= datetime('now')",
                params![],
            )
            .await
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to clear expired items: {}", e)))?;

        Ok(result as i64)
    }

    async fn clear_session(&self, session_id: &str) -> Result<i64> {
        let conn = self.conn.lock().await;

        let result = conn
            .execute(
                "DELETE FROM working_memory WHERE session_id = ?",
                params![session_id],
            )
            .await
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to clear session: {}", e)))?;

        Ok(result as i64)
    }

    async fn get_by_priority(
        &self,
        session_id: &str,
        min_priority: i32,
    ) -> Result<Vec<WorkingMemoryItem>> {
        let conn = self.conn.lock().await;

        let mut stmt = conn
            .prepare(
                r#"
                SELECT * FROM working_memory
                WHERE session_id = ?
                AND priority >= ?
                AND (expires_at IS NULL OR expires_at > datetime('now'))
                ORDER BY priority DESC, created_at ASC
                "#,
            )
            .await
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to prepare statement: {}", e)))?;

        let mut rows = stmt
            .query(params![session_id, min_priority as i64])
            .await
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to execute query: {}", e)))?;

        let mut results = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| {
            AgentMemError::storage_error(&format!("Failed to fetch row: {}", e))
        })? {
            results.push(row_to_item(&row)?);
        }

        Ok(results)
    }
}

