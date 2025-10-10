//! PostgreSQL implementation of EpisodicMemoryStore

use agent_mem_traits::{AgentMemError, EpisodicEvent, EpisodicMemoryStore, EpisodicQuery, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{debug, info};

/// PostgreSQL-based episodic memory store
pub struct PostgresEpisodicStore {
    pool: Arc<PgPool>,
}

impl PostgresEpisodicStore {
    /// Create a new PostgreSQL episodic store
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EpisodicMemoryStore for PostgresEpisodicStore {
    async fn create_event(&self, event: EpisodicEvent) -> Result<EpisodicEvent> {
        info!("Creating episodic event: {}", event.id);

        let result = sqlx::query_as!(
            EpisodicEventRow,
            r#"
            INSERT INTO episodic_events (
                id, organization_id, user_id, agent_id, occurred_at,
                event_type, actor, summary, details, importance_score,
                metadata, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING *
            "#,
            event.id,
            event.organization_id,
            event.user_id,
            event.agent_id,
            event.occurred_at,
            event.event_type,
            event.actor,
            event.summary,
            event.details,
            event.importance_score,
            event.metadata,
            event.created_at,
            event.updated_at,
        )
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to create episodic event: {}", e)))?;

        Ok(result.into())
    }

    async fn get_event(&self, event_id: &str, user_id: &str) -> Result<Option<EpisodicEvent>> {
        debug!("Getting episodic event: {}", event_id);

        let result = sqlx::query_as!(
            EpisodicEventRow,
            r#"
            SELECT * FROM episodic_events
            WHERE id = $1 AND user_id = $2
            "#,
            event_id,
            user_id,
        )
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to get episodic event: {}", e)))?;

        Ok(result.map(Into::into))
    }

    async fn query_events(&self, user_id: &str, query: EpisodicQuery) -> Result<Vec<EpisodicEvent>> {
        info!("Querying episodic events for user: {}", user_id);

        let mut sql = String::from("SELECT * FROM episodic_events WHERE user_id = $1");
        let mut param_count = 1;

        // Build dynamic query
        if query.start_time.is_some() {
            param_count += 1;
            sql.push_str(&format!(" AND occurred_at >= ${}", param_count));
        }

        if query.end_time.is_some() {
            param_count += 1;
            sql.push_str(&format!(" AND occurred_at <= ${}", param_count));
        }

        if query.event_type.is_some() {
            param_count += 1;
            sql.push_str(&format!(" AND event_type = ${}", param_count));
        }

        if query.min_importance.is_some() {
            param_count += 1;
            sql.push_str(&format!(" AND importance_score >= ${}", param_count));
        }

        sql.push_str(" ORDER BY occurred_at DESC");

        if let Some(limit) = query.limit {
            param_count += 1;
            sql.push_str(&format!(" LIMIT ${}", param_count));
        }

        // Execute query with parameters
        let mut query_builder = sqlx::query_as::<_, EpisodicEventRow>(&sql);
        query_builder = query_builder.bind(user_id);

        if let Some(start_time) = query.start_time {
            query_builder = query_builder.bind(start_time);
        }
        if let Some(end_time) = query.end_time {
            query_builder = query_builder.bind(end_time);
        }
        if let Some(event_type) = query.event_type {
            query_builder = query_builder.bind(event_type);
        }
        if let Some(min_importance) = query.min_importance {
            query_builder = query_builder.bind(min_importance);
        }
        if let Some(limit) = query.limit {
            query_builder = query_builder.bind(limit);
        }

        let results = query_builder
            .fetch_all(self.pool.as_ref())
            .await
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to query events: {}", e)))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn update_event(&self, event: EpisodicEvent) -> Result<bool> {
        debug!("Updating episodic event: {}", event.id);

        let result = sqlx::query!(
            r#"
            UPDATE episodic_events
            SET event_type = $1, actor = $2, summary = $3, details = $4,
                importance_score = $5, metadata = $6, updated_at = $7
            WHERE id = $8 AND user_id = $9
            "#,
            event.event_type,
            event.actor,
            event.summary,
            event.details,
            event.importance_score,
            event.metadata,
            Utc::now(),
            event.id,
            event.user_id,
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to update event: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    async fn delete_event(&self, event_id: &str, user_id: &str) -> Result<bool> {
        debug!("Deleting episodic event: {}", event_id);

        let result = sqlx::query!(
            r#"
            DELETE FROM episodic_events
            WHERE id = $1 AND user_id = $2
            "#,
            event_id,
            user_id,
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to delete episodic event: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    async fn update_importance(&self, event_id: &str, user_id: &str, importance_score: f32) -> Result<bool> {
        debug!("Updating importance for event: {}", event_id);

        let result = sqlx::query!(
            r#"
            UPDATE episodic_events
            SET importance_score = $1, updated_at = $2
            WHERE id = $3 AND user_id = $4
            "#,
            importance_score,
            Utc::now(),
            event_id,
            user_id,
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to update importance: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    async fn count_events_in_range(
        &self,
        user_id: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<i64> {
        let result = sqlx::query!(
            r#"
            SELECT COUNT(*) as count FROM episodic_events
            WHERE user_id = $1 AND occurred_at >= $2 AND occurred_at <= $3
            "#,
            user_id,
            start_time,
            end_time,
        )
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to count events: {}", e)))?;

        Ok(result.count.unwrap_or(0))
    }

    async fn get_recent_events(&self, user_id: &str, limit: i64) -> Result<Vec<EpisodicEvent>> {
        let results = sqlx::query_as!(
            EpisodicEventRow,
            r#"
            SELECT * FROM episodic_events
            WHERE user_id = $1
            ORDER BY occurred_at DESC
            LIMIT $2
            "#,
            user_id,
            limit,
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to get recent events: {}", e)))?;

        Ok(results.into_iter().map(Into::into).collect())
    }
}

/// Database row structure
#[derive(Debug, sqlx::FromRow)]
struct EpisodicEventRow {
    id: String,
    organization_id: String,
    user_id: String,
    agent_id: String,
    occurred_at: DateTime<Utc>,
    event_type: String,
    actor: Option<String>,
    summary: String,
    details: Option<String>,
    importance_score: f32,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<EpisodicEventRow> for EpisodicEvent {
    fn from(row: EpisodicEventRow) -> Self {
        Self {
            id: row.id,
            organization_id: row.organization_id,
            user_id: row.user_id,
            agent_id: row.agent_id,
            occurred_at: row.occurred_at,
            event_type: row.event_type,
            actor: row.actor,
            summary: row.summary,
            details: row.details,
            importance_score: row.importance_score,
            metadata: row.metadata,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

