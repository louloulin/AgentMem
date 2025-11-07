//! PostgreSQL implementation of SemanticMemoryStore

use agent_mem_traits::{
    AgentMemError, Result, SemanticMemoryItem, SemanticMemoryStore, SemanticQuery,
};
use async_trait::async_trait;
use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{debug, info};

/// PostgreSQL-based semantic memory store
pub struct PostgresSemanticStore {
    pool: Arc<PgPool>,
}

impl PostgresSemanticStore {
    /// Create a new PostgreSQL semantic store
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SemanticMemoryStore for PostgresSemanticStore {
    async fn create_item(&self, item: SemanticMemoryItem) -> Result<SemanticMemoryItem> {
        info!("Creating semantic item: {}", item.id);

        let result = sqlx::query_as!(
            SemanticItemRow,
            r#"
            INSERT INTO semantic_memory (
                id, organization_id, user_id, agent_id, name,
                summary, details, source, tree_path,
                metadata, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#,
            item.id,
            item.organization_id,
            item.user_id,
            item.agent_id,
            item.name,
            item.summary,
            item.details,
            item.source,
            &item.tree_path,
            item.metadata,
            item.created_at,
            item.updated_at,
        )
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| {
            AgentMemError::storage_error(&format!("Failed to create semantic item: {}", e))
        })?;

        Ok(result.into())
    }

    async fn get_item(&self, item_id: &str, user_id: &str) -> Result<Option<SemanticMemoryItem>> {
        debug!("Getting semantic item: {}", item_id);

        let result = sqlx::query_as!(
            SemanticItemRow,
            r#"
            SELECT * FROM semantic_memory
            WHERE id = $1 AND user_id = $2
            "#,
            item_id,
            user_id,
        )
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| {
            AgentMemError::storage_error(&format!("Failed to get semantic item: {}", e))
        })?;

        Ok(result.map(Into::into))
    }

    async fn query_items(
        &self,
        user_id: &str,
        query: SemanticQuery,
    ) -> Result<Vec<SemanticMemoryItem>> {
        info!("Querying semantic items for user: {}", user_id);

        let mut sql = String::from("SELECT * FROM semantic_memory WHERE user_id = $1");
        let mut param_count = 1;

        if query.name_query.is_some() {
            param_count += 1;
            sql.push_str(&format!(" AND name ILIKE ${}", param_count));
        }

        if query.summary_query.is_some() {
            param_count += 1;
            sql.push_str(&format!(" AND summary ILIKE ${}", param_count));
        }

        if query.tree_path_prefix.is_some() {
            param_count += 1;
            sql.push_str(&format!(" AND tree_path @> ${}", param_count));
        }

        sql.push_str(" ORDER BY created_at DESC");

        if let Some(limit) = query.limit {
            param_count += 1;
            sql.push_str(&format!(" LIMIT ${}", param_count));
        }

        let mut query_builder = sqlx::query_as::<_, SemanticItemRow>(&sql);
        query_builder = query_builder.bind(user_id);

        if let Some(name_query) = query.name_query {
            query_builder = query_builder.bind(format!("%{}%", name_query));
        }
        if let Some(summary_query) = query.summary_query {
            query_builder = query_builder.bind(format!("%{}%", summary_query));
        }
        if let Some(tree_path_prefix) = query.tree_path_prefix {
            query_builder = query_builder.bind(tree_path_prefix);
        }
        if let Some(limit) = query.limit {
            query_builder = query_builder.bind(limit);
        }

        let results = query_builder
            .fetch_all(self.pool.as_ref())
            .await
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to query items: {}", e)))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn update_item(&self, item: SemanticMemoryItem) -> Result<bool> {
        debug!("Updating semantic item: {}", item.id);

        let result = sqlx::query!(
            r#"
            UPDATE semantic_memory
            SET name = $1, summary = $2, details = $3, source = $4,
                tree_path = $5, metadata = $6, updated_at = $7
            WHERE id = $8 AND user_id = $9
            "#,
            item.name,
            item.summary,
            item.details,
            item.source,
            &item.tree_path,
            item.metadata,
            Utc::now(),
            item.id,
            item.user_id,
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to update item: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    async fn delete_item(&self, item_id: &str, user_id: &str) -> Result<bool> {
        debug!("Deleting semantic item: {}", item_id);

        let result = sqlx::query!(
            r#"
            DELETE FROM semantic_memory
            WHERE id = $1 AND user_id = $2
            "#,
            item_id,
            user_id,
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to delete item: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    async fn search_by_tree_path(
        &self,
        user_id: &str,
        tree_path: Vec<String>,
    ) -> Result<Vec<SemanticMemoryItem>> {
        let results = sqlx::query_as!(
            SemanticItemRow,
            r#"
            SELECT * FROM semantic_memory
            WHERE user_id = $1 AND tree_path @> $2
            ORDER BY created_at DESC
            "#,
            user_id,
            &tree_path,
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| {
            AgentMemError::storage_error(&format!("Failed to search by tree path: {}", e))
        })?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn search_by_name(
        &self,
        user_id: &str,
        name_pattern: &str,
        limit: i64,
    ) -> Result<Vec<SemanticMemoryItem>> {
        let results = sqlx::query_as!(
            SemanticItemRow,
            r#"
            SELECT * FROM semantic_memory
            WHERE user_id = $1 AND name ILIKE $2
            ORDER BY created_at DESC
            LIMIT $3
            "#,
            user_id,
            format!("%{}%", name_pattern),
            limit,
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to search by name: {}", e)))?;

        Ok(results.into_iter().map(Into::into).collect())
    }
}

/// Database row structure
#[derive(Debug, sqlx::FromRow)]
struct SemanticItemRow {
    id: String,
    organization_id: String,
    user_id: String,
    agent_id: String,
    name: String,
    summary: String,
    details: Option<String>,
    source: Option<String>,
    tree_path: Vec<String>,
    metadata: serde_json::Value,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl From<SemanticItemRow> for SemanticMemoryItem {
    fn from(row: SemanticItemRow) -> Self {
        Self {
            id: row.id,
            organization_id: row.organization_id,
            user_id: row.user_id,
            agent_id: row.agent_id,
            name: row.name,
            summary: row.summary,
            details: row.details,
            source: row.source,
            tree_path: row.tree_path,
            metadata: row.metadata,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
