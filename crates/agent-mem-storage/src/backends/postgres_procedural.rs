//! PostgreSQL implementation of ProceduralMemoryStore

use agent_mem_traits::{
    AgentMemError, ProceduralMemoryItem, ProceduralMemoryStore, ProceduralQuery, Result,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use std::sync::Arc;

/// PostgreSQL implementation of ProceduralMemoryStore
pub struct PostgresProceduralStore {
    pool: Arc<PgPool>,
}

impl PostgresProceduralStore {
    /// Create a new PostgreSQL procedural memory store
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

/// Database row representation
#[derive(Debug, sqlx::FromRow)]
struct ProceduralMemoryItemRow {
    id: String,
    organization_id: String,
    user_id: String,
    agent_id: String,
    skill_name: String,
    description: String,
    steps: JsonValue,
    success_rate: f32,
    execution_count: i32,
    metadata: JsonValue,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<ProceduralMemoryItemRow> for ProceduralMemoryItem {
    fn from(row: ProceduralMemoryItemRow) -> Self {
        let steps: Vec<String> = serde_json::from_value(row.steps).unwrap_or_default();

        Self {
            id: row.id,
            organization_id: row.organization_id,
            user_id: row.user_id,
            agent_id: row.agent_id,
            skill_name: row.skill_name,
            description: row.description,
            steps,
            success_rate: row.success_rate,
            execution_count: row.execution_count,
            metadata: row.metadata,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[async_trait]
impl ProceduralMemoryStore for PostgresProceduralStore {
    async fn create_item(&self, item: ProceduralMemoryItem) -> Result<ProceduralMemoryItem> {
        let steps_json = serde_json::to_value(&item.steps).map_err(|e| {
            AgentMemError::storage_error(&format!("Failed to serialize steps: {}", e))
        })?;

        let result = sqlx::query_as::<_, ProceduralMemoryItemRow>(
            r#"
            INSERT INTO procedural_memory (
                id, organization_id, user_id, agent_id, skill_name,
                description, steps, success_rate, execution_count, metadata,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#,
        )
        .bind(&item.id)
        .bind(&item.organization_id)
        .bind(&item.user_id)
        .bind(&item.agent_id)
        .bind(&item.skill_name)
        .bind(&item.description)
        .bind(steps_json)
        .bind(item.success_rate)
        .bind(item.execution_count)
        .bind(&item.metadata)
        .bind(item.created_at)
        .bind(item.updated_at)
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| {
            AgentMemError::storage_error(&format!("Failed to create procedural item: {}", e))
        })?;

        Ok(result.into())
    }

    async fn get_item(&self, item_id: &str, user_id: &str) -> Result<Option<ProceduralMemoryItem>> {
        let result = sqlx::query_as::<_, ProceduralMemoryItemRow>(
            r#"
            SELECT * FROM procedural_memory
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(item_id)
        .bind(user_id)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| {
            AgentMemError::storage_error(&format!("Failed to get procedural item: {}", e))
        })?;

        Ok(result.map(Into::into))
    }

    async fn query_items(
        &self,
        user_id: &str,
        query: ProceduralQuery,
    ) -> Result<Vec<ProceduralMemoryItem>> {
        let mut sql = String::from("SELECT * FROM procedural_memory WHERE user_id = $1");
        let mut param_count = 1;

        if query.skill_name_pattern.is_some() {
            param_count += 1;
            sql.push_str(&format!(" AND skill_name ILIKE ${}", param_count));
        }

        if let Some(min_success_rate) = query.min_success_rate {
            param_count += 1;
            sql.push_str(&format!(" AND success_rate >= ${}", param_count));
        }

        sql.push_str(" ORDER BY updated_at DESC");

        if let Some(limit) = query.limit {
            param_count += 1;
            sql.push_str(&format!(" LIMIT ${}", param_count));
        }

        let mut query_builder = sqlx::query_as::<_, ProceduralMemoryItemRow>(&sql);
        query_builder = query_builder.bind(user_id);

        if let Some(skill_name_pattern) = query.skill_name_pattern {
            query_builder = query_builder.bind(format!("%{}%", skill_name_pattern));
        }

        if let Some(min_success_rate) = query.min_success_rate {
            query_builder = query_builder.bind(min_success_rate);
        }

        if let Some(limit) = query.limit {
            query_builder = query_builder.bind(limit);
        }

        let results = query_builder
            .fetch_all(self.pool.as_ref())
            .await
            .map_err(|e| {
                AgentMemError::storage_error(&format!("Failed to query procedural items: {}", e))
            })?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn update_item(&self, item: ProceduralMemoryItem) -> Result<bool> {
        let steps_json = serde_json::to_value(&item.steps).map_err(|e| {
            AgentMemError::storage_error(&format!("Failed to serialize steps: {}", e))
        })?;

        let result = sqlx::query(
            r#"
            UPDATE procedural_memory
            SET skill_name = $1, description = $2, steps = $3,
                success_rate = $4, execution_count = $5, metadata = $6, updated_at = $7
            WHERE id = $8 AND user_id = $9
            "#,
        )
        .bind(&item.skill_name)
        .bind(&item.description)
        .bind(steps_json)
        .bind(item.success_rate)
        .bind(item.execution_count)
        .bind(&item.metadata)
        .bind(item.updated_at)
        .bind(&item.id)
        .bind(&item.user_id)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| {
            AgentMemError::storage_error(&format!("Failed to update procedural item: {}", e))
        })?;

        Ok(result.rows_affected() > 0)
    }

    async fn delete_item(&self, item_id: &str, user_id: &str) -> Result<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM procedural_memory
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(item_id)
        .bind(user_id)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| {
            AgentMemError::storage_error(&format!("Failed to delete procedural item: {}", e))
        })?;

        Ok(result.rows_affected() > 0)
    }

    async fn update_execution_stats(
        &self,
        item_id: &str,
        user_id: &str,
        success: bool,
    ) -> Result<bool> {
        let result = sqlx::query(
            r#"
            UPDATE procedural_memory
            SET execution_count = execution_count + 1,
                success_rate = CASE
                    WHEN $3 THEN (success_rate * execution_count + 1.0) / (execution_count + 1)
                    ELSE (success_rate * execution_count) / (execution_count + 1)
                END,
                updated_at = NOW()
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(item_id)
        .bind(user_id)
        .bind(success)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| {
            AgentMemError::storage_error(&format!("Failed to update execution stats: {}", e))
        })?;

        Ok(result.rows_affected() > 0)
    }

    async fn get_top_skills(&self, user_id: &str, limit: i64) -> Result<Vec<ProceduralMemoryItem>> {
        let results = sqlx::query_as::<_, ProceduralMemoryItemRow>(
            r#"
            SELECT * FROM procedural_memory
            WHERE user_id = $1
            ORDER BY success_rate DESC, execution_count DESC
            LIMIT $2
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to get top skills: {}", e)))?;

        Ok(results.into_iter().map(Into::into).collect())
    }
}
