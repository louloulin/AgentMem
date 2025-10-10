//! PostgreSQL implementation of WorkingMemoryStore

use agent_mem_traits::{AgentMemError, Result, WorkingMemoryItem, WorkingMemoryStore};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use std::sync::Arc;

/// PostgreSQL implementation of WorkingMemoryStore
pub struct PostgresWorkingStore {
    pool: Arc<PgPool>,
}

impl PostgresWorkingStore {
    /// Create a new PostgreSQL working memory store
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

/// Database row representation
#[derive(Debug, sqlx::FromRow)]
struct WorkingMemoryItemRow {
    id: String,
    user_id: String,
    agent_id: String,
    session_id: String,
    content: String,
    priority: i32,
    expires_at: Option<DateTime<Utc>>,
    metadata: JsonValue,
    created_at: DateTime<Utc>,
}

impl From<WorkingMemoryItemRow> for WorkingMemoryItem {
    fn from(row: WorkingMemoryItemRow) -> Self {
        Self {
            id: row.id,
            user_id: row.user_id,
            agent_id: row.agent_id,
            session_id: row.session_id,
            content: row.content,
            priority: row.priority,
            expires_at: row.expires_at,
            metadata: row.metadata,
            created_at: row.created_at,
        }
    }
}

#[async_trait]
impl WorkingMemoryStore for PostgresWorkingStore {
    async fn add_item(&self, item: WorkingMemoryItem) -> Result<WorkingMemoryItem> {
        let result = sqlx::query_as::<_, WorkingMemoryItemRow>(
            r#"
            INSERT INTO working_memory (
                id, user_id, agent_id, session_id, content,
                priority, expires_at, metadata, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(&item.id)
        .bind(&item.user_id)
        .bind(&item.agent_id)
        .bind(&item.session_id)
        .bind(&item.content)
        .bind(item.priority)
        .bind(item.expires_at)
        .bind(&item.metadata)
        .bind(item.created_at)
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to add working memory item: {}", e)))?;

        Ok(result.into())
    }

    async fn get_session_items(&self, session_id: &str) -> Result<Vec<WorkingMemoryItem>> {
        let results = sqlx::query_as::<_, WorkingMemoryItemRow>(
            r#"
            SELECT * FROM working_memory
            WHERE session_id = $1
            AND (expires_at IS NULL OR expires_at > NOW())
            ORDER BY priority DESC, created_at ASC
            "#,
        )
        .bind(session_id)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to get session items: {}", e)))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn remove_item(&self, item_id: &str) -> Result<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM working_memory
            WHERE id = $1
            "#,
        )
        .bind(item_id)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to remove working memory item: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    async fn clear_expired(&self) -> Result<i64> {
        let result = sqlx::query(
            r#"
            DELETE FROM working_memory
            WHERE expires_at IS NOT NULL AND expires_at <= NOW()
            "#,
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to clear expired items: {}", e)))?;

        Ok(result.rows_affected() as i64)
    }

    async fn clear_session(&self, session_id: &str) -> Result<i64> {
        let result = sqlx::query(
            r#"
            DELETE FROM working_memory
            WHERE session_id = $1
            "#,
        )
        .bind(session_id)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to clear session: {}", e)))?;

        Ok(result.rows_affected() as i64)
    }

    async fn get_by_priority(
        &self,
        session_id: &str,
        min_priority: i32,
    ) -> Result<Vec<WorkingMemoryItem>> {
        let results = sqlx::query_as::<_, WorkingMemoryItemRow>(
            r#"
            SELECT * FROM working_memory
            WHERE session_id = $1
            AND priority >= $2
            AND (expires_at IS NULL OR expires_at > NOW())
            ORDER BY priority DESC, created_at ASC
            "#,
        )
        .bind(session_id)
        .bind(min_priority)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to get items by priority: {}", e)))?;

        Ok(results.into_iter().map(Into::into).collect())
    }
}

