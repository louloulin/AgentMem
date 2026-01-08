//! LibSQL implementation of CoreMemoryStore
//!
//! Note: Statement caching removed due to libsql::Statement not implementing Clone

use agent_mem_traits::{AgentMemError, CoreMemoryItem, CoreMemoryStore, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use libsql::{params, Connection, Row};
use std::sync::Arc;
use tokio::sync::Mutex;

/// LibSQL implementation of CoreMemoryStore
pub struct LibSqlCoreStore {
    conn: Arc<Mutex<Connection>>,
}

impl LibSqlCoreStore {
    /// Create a new LibSQL core memory store
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
}

/// Convert LibSQL row to CoreMemoryItem
fn row_to_item(row: &Row) -> Result<CoreMemoryItem> {
    let metadata_json: String = row
        .get(7)
        .map_err(|e| AgentMemError::storage_error(format!("Failed to get metadata: {e}")))?;
    let metadata: serde_json::Value = serde_json::from_str(&metadata_json)
        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

    let created_at_str: String = row
        .get(8)
        .map_err(|e| AgentMemError::storage_error(format!("Failed to get created_at: {e}")))?;
    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map_err(|e| AgentMemError::storage_error(format!("Failed to parse created_at: {e}")))?
        .with_timezone(&Utc);

    let updated_at_str: String = row
        .get(9)
        .map_err(|e| AgentMemError::storage_error(format!("Failed to get updated_at: {e}")))?;
    let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
        .map_err(|e| AgentMemError::storage_error(format!("Failed to parse updated_at: {e}")))?
        .with_timezone(&Utc);

    let is_mutable_i64: i64 = row
        .get(6)
        .map_err(|e| AgentMemError::storage_error(format!("Failed to get is_mutable: {e}")))?;

    Ok(CoreMemoryItem {
        id: row
            .get(0)
            .map_err(|e| AgentMemError::storage_error(format!("Failed to get id: {e}")))?,
        user_id: row
            .get(1)
            .map_err(|e| AgentMemError::storage_error(format!("Failed to get user_id: {e}")))?,
        agent_id: row
            .get(2)
            .map_err(|e| AgentMemError::storage_error(format!("Failed to get agent_id: {e}")))?,
        key: row
            .get(3)
            .map_err(|e| AgentMemError::storage_error(format!("Failed to get key: {e}")))?,
        value: row
            .get(4)
            .map_err(|e| AgentMemError::storage_error(format!("Failed to get value: {e}")))?,
        category: row
            .get(5)
            .map_err(|e| AgentMemError::storage_error(format!("Failed to get category: {e}")))?,
        is_mutable: is_mutable_i64 != 0,
        metadata,
        created_at,
        updated_at,
    })
}

#[async_trait]
impl CoreMemoryStore for LibSqlCoreStore {
    async fn set_value(&self, item: CoreMemoryItem) -> Result<CoreMemoryItem> {
        let conn = self.conn.lock().await;

        let metadata_json = serde_json::to_string(&item.metadata).map_err(|e| {
            AgentMemError::storage_error(format!("Failed to serialize metadata: {e}"))
        })?;

        // Use INSERT OR REPLACE for UPSERT behavior
        conn.execute(
            r#"
            INSERT OR REPLACE INTO core_memory (
                id, user_id, agent_id, key, value, category,
                is_mutable, metadata, created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                item.id.clone(),
                item.user_id.clone(),
                item.agent_id.clone(),
                item.key.clone(),
                item.value.clone(),
                item.category.clone(),
                if item.is_mutable { 1 } else { 0 },
                metadata_json,
                item.created_at.to_rfc3339(),
                item.updated_at.to_rfc3339(),
            ],
        )
        .await
        .map_err(|e| {
            AgentMemError::storage_error(format!("Failed to set core memory value: {e}"))
        })?;

        Ok(item)
    }

    async fn get_value(&self, user_id: &str, key: &str) -> Result<Option<CoreMemoryItem>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare("SELECT * FROM core_memory WHERE user_id = ? AND key = ?")
            .await
            .map_err(|e| AgentMemError::storage_error(format!("Failed to prepare statement: {e}")))?;

        let mut rows = stmt
            .query(params![user_id, key])
            .await
            .map_err(|e| AgentMemError::storage_error(format!("Failed to execute query: {e}")))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::storage_error(format!("Failed to fetch row: {e}")))?
        {
            Ok(Some(row_to_item(&row)?))
        } else {
            Ok(None)
        }
    }

    async fn get_all(&self, user_id: &str) -> Result<Vec<CoreMemoryItem>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT * FROM core_memory WHERE user_id = ? ORDER BY category, key"
        )
        .await
        .map_err(|e| AgentMemError::storage_error(format!("Failed to prepare statement: {e}")))?;

        let mut rows = stmt
            .query(params![user_id])
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

    async fn get_by_category(&self, user_id: &str, category: &str) -> Result<Vec<CoreMemoryItem>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT * FROM core_memory WHERE user_id = ? AND category = ? ORDER BY key"
        )
        .await
        .map_err(|e| AgentMemError::storage_error(format!("Failed to prepare statement: {e}")))?;

        let mut rows = stmt
            .query(params![user_id, category])
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

    async fn delete_value(&self, user_id: &str, key: &str) -> Result<bool> {
        let conn = self.conn.lock().await;

        let result = conn
            .execute(
                "DELETE FROM core_memory WHERE user_id = ? AND key = ? AND is_mutable = 1",
                params![user_id, key],
            )
            .await
            .map_err(|e| {
                AgentMemError::storage_error(format!("Failed to delete core memory value: {e}"))
            })?;

        Ok(result > 0)
    }

    async fn update_value(&self, user_id: &str, key: &str, new_value: &str) -> Result<bool> {
        let conn = self.conn.lock().await;

        let result = conn
            .execute(
                r#"
                UPDATE core_memory
                SET value = ?, updated_at = datetime('now')
                WHERE user_id = ? AND key = ? AND is_mutable = 1
                "#,
                params![new_value, user_id, key],
            )
            .await
            .map_err(|e| {
                AgentMemError::storage_error(format!("Failed to update core memory value: {e}"))
            })?;

        Ok(result > 0)
    }
}
