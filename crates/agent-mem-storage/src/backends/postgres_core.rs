//! PostgreSQL implementation of CoreMemoryStore

use agent_mem_traits::{AgentMemError, CoreMemoryItem, CoreMemoryStore, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use std::sync::Arc;

/// PostgreSQL implementation of CoreMemoryStore
pub struct PostgresCoreStore {
    pool: Arc<PgPool>,
}

impl PostgresCoreStore {
    /// Create a new PostgreSQL core memory store
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

/// Database row representation
#[derive(Debug, sqlx::FromRow)]
struct CoreMemoryItemRow {
    id: String,
    user_id: String,
    agent_id: String,
    key: String,
    value: String,
    category: String,
    is_mutable: bool,
    metadata: JsonValue,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<CoreMemoryItemRow> for CoreMemoryItem {
    fn from(row: CoreMemoryItemRow) -> Self {
        Self {
            id: row.id,
            user_id: row.user_id,
            agent_id: row.agent_id,
            key: row.key,
            value: row.value,
            category: row.category,
            is_mutable: row.is_mutable,
            metadata: row.metadata,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[async_trait]
impl CoreMemoryStore for PostgresCoreStore {
    async fn set_value(&self, item: CoreMemoryItem) -> Result<CoreMemoryItem> {
        // Use UPSERT (INSERT ... ON CONFLICT) to handle both create and update
        let result = sqlx::query_as::<_, CoreMemoryItemRow>(
            r#"
            INSERT INTO core_memory (
                id, user_id, agent_id, key, value, category,
                is_mutable, metadata, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (user_id, agent_id, key)
            DO UPDATE SET
                value = EXCLUDED.value,
                category = EXCLUDED.category,
                metadata = EXCLUDED.metadata,
                updated_at = EXCLUDED.updated_at
            WHERE core_memory.is_mutable = true
            RETURNING *
            "#,
        )
        .bind(&item.id)
        .bind(&item.user_id)
        .bind(&item.agent_id)
        .bind(&item.key)
        .bind(&item.value)
        .bind(&item.category)
        .bind(item.is_mutable)
        .bind(&item.metadata)
        .bind(item.created_at)
        .bind(item.updated_at)
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| {
            AgentMemError::storage_error(&format!("Failed to set core memory value: {}", e))
        })?;

        Ok(result.into())
    }

    async fn get_value(&self, user_id: &str, key: &str) -> Result<Option<CoreMemoryItem>> {
        let result = sqlx::query_as::<_, CoreMemoryItemRow>(
            r#"
            SELECT * FROM core_memory
            WHERE user_id = $1 AND key = $2
            "#,
        )
        .bind(user_id)
        .bind(key)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| {
            AgentMemError::storage_error(&format!("Failed to get core memory value: {}", e))
        })?;

        Ok(result.map(Into::into))
    }

    async fn get_all(&self, user_id: &str) -> Result<Vec<CoreMemoryItem>> {
        let results = sqlx::query_as::<_, CoreMemoryItemRow>(
            r#"
            SELECT * FROM core_memory
            WHERE user_id = $1
            ORDER BY category, key
            "#,
        )
        .bind(user_id)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| {
            AgentMemError::storage_error(&format!("Failed to get all core memory: {}", e))
        })?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn get_by_category(&self, user_id: &str, category: &str) -> Result<Vec<CoreMemoryItem>> {
        let results = sqlx::query_as::<_, CoreMemoryItemRow>(
            r#"
            SELECT * FROM core_memory
            WHERE user_id = $1 AND category = $2
            ORDER BY key
            "#,
        )
        .bind(user_id)
        .bind(category)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| {
            AgentMemError::storage_error(&format!("Failed to get core memory by category: {}", e))
        })?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn delete_value(&self, user_id: &str, key: &str) -> Result<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM core_memory
            WHERE user_id = $1 AND key = $2 AND is_mutable = true
            "#,
        )
        .bind(user_id)
        .bind(key)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| {
            AgentMemError::storage_error(&format!("Failed to delete core memory value: {}", e))
        })?;

        Ok(result.rows_affected() > 0)
    }

    async fn update_value(&self, user_id: &str, key: &str, new_value: &str) -> Result<bool> {
        let result = sqlx::query(
            r#"
            UPDATE core_memory
            SET value = $1, updated_at = NOW()
            WHERE user_id = $2 AND key = $3 AND is_mutable = true
            "#,
        )
        .bind(new_value)
        .bind(user_id)
        .bind(key)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| {
            AgentMemError::storage_error(&format!("Failed to update core memory value: {}", e))
        })?;

        Ok(result.rows_affected() > 0)
    }
}
