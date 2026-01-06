//! Message repository implementation

use async_trait::async_trait;
use chrono::Utc;
use sqlx::{PgPool, Row};

use super::models::Message;
use super::repository::Repository;
use crate::{CoreError, CoreResult};

/// Message repository
pub struct MessageRepository {
    pool: PgPool,
}

impl MessageRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// List messages by agent
    pub async fn list_by_agent(
        &self,
        agent_id: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> CoreResult<Vec<Message>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        let results = sqlx::query_as::<_, Message>(
            r#"
            SELECT * FROM messages
            WHERE agent_id = $1 AND is_deleted = FALSE
            ORDER BY created_at ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(agent_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to list messages: {}", e)))?;

        Ok(results)
    }

    /// Get message count for an agent
    pub async fn count_by_agent(&self, agent_id: &str) -> CoreResult<i64> {
        let result = sqlx::query(
            r#"
            SELECT COUNT(*) as count FROM messages
            WHERE agent_id = $1 AND is_deleted = FALSE
            "#,
        )
        .bind(agent_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to count messages: {}", e)))?;

        Ok(result.try_get("count").unwrap_or(0))
    }

    /// Get recent messages for an agent (for context window)
    pub async fn get_recent_messages(
        &self,
        agent_id: &str,
        limit: i64,
    ) -> CoreResult<Vec<Message>> {
        let results = sqlx::query_as::<_, Message>(
            r#"
            SELECT * FROM messages
            WHERE agent_id = $1 AND is_deleted = FALSE
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(agent_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to get recent messages: {}", e)))?;

        // Reverse to get chronological order
        Ok(results.into_iter().rev().collect())
    }

    /// Delete old messages for an agent (for context window management)
    pub async fn delete_old_messages(&self, agent_id: &str, keep_count: i64) -> CoreResult<i64> {
        // Get IDs of messages to keep
        let keep_ids: Vec<String> = sqlx::query_scalar(
            r#"
            SELECT id FROM messages
            WHERE agent_id = $1 AND is_deleted = FALSE
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(agent_id)
        .bind(keep_count)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to get message IDs: {}", e)))?;

        if keep_ids.is_empty() {
            return Ok(0);
        }

        // Soft delete messages not in keep list
        let result = sqlx::query(
            r#"
            UPDATE messages
            SET is_deleted = TRUE, updated_at = $3
            WHERE agent_id = $1 AND id != ALL($2) AND is_deleted = FALSE
            "#,
        )
        .bind(agent_id)
        .bind(&keep_ids)
        .bind(Utc::now())
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to delete old messages: {}", e)))?;

        Ok(result.rows_affected() as i64)
    }
}

#[async_trait]
impl Repository<Message> for MessageRepository {
    async fn create(&self, message: &Message) -> CoreResult<Message> {
        let result = sqlx::query_as::<_, Message>(
            r#"
            INSERT INTO messages (
                id, organization_id, user_id, agent_id, role,
                text, content, model, name, tool_calls,
                tool_call_id, step_id, otid, tool_returns,
                group_id, sender_id,
                created_at, updated_at, is_deleted,
                created_by_id, last_updated_by_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
            RETURNING *
            "#,
        )
        .bind(&message.id)
        .bind(&message.organization_id)
        .bind(&message.user_id)
        .bind(&message.agent_id)
        .bind(&message.role)
        .bind(&message.text)
        .bind(&message.content)
        .bind(&message.model)
        .bind(&message.name)
        .bind(&message.tool_calls)
        .bind(&message.tool_call_id)
        .bind(&message.step_id)
        .bind(&message.otid)
        .bind(&message.tool_returns)
        .bind(&message.group_id)
        .bind(&message.sender_id)
        .bind(&message.created_at)
        .bind(&message.updated_at)
        .bind(&message.is_deleted)
        .bind(&message.created_by_id)
        .bind(&message.last_updated_by_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create message: {}", e)))?;

        Ok(result)
    }

    async fn read(&self, id: &str) -> CoreResult<Option<Message>> {
        let result = sqlx::query_as::<_, Message>(
            r#"
            SELECT * FROM messages
            WHERE id = $1 AND is_deleted = FALSE
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to read message: {}", e)))?;

        Ok(result)
    }

    async fn update(&self, message: &Message) -> CoreResult<Message> {
        let result = sqlx::query_as::<_, Message>(
            r#"
            UPDATE messages
            SET role = $2, text = $3, content = $4,
                model = $5, name = $6, tool_calls = $7,
                tool_call_id = $8, step_id = $9, otid = $10,
                tool_returns = $11, group_id = $12, sender_id = $13,
                updated_at = $14, last_updated_by_id = $15
            WHERE id = $1 AND is_deleted = FALSE
            RETURNING *
            "#,
        )
        .bind(&message.id)
        .bind(&message.role)
        .bind(&message.text)
        .bind(&message.content)
        .bind(&message.model)
        .bind(&message.name)
        .bind(&message.tool_calls)
        .bind(&message.tool_call_id)
        .bind(&message.step_id)
        .bind(&message.otid)
        .bind(&message.tool_returns)
        .bind(&message.group_id)
        .bind(&message.sender_id)
        .bind(Utc::now())
        .bind(&message.last_updated_by_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to update message: {}", e)))?;

        Ok(result)
    }

    async fn delete(&self, id: &str) -> CoreResult<bool> {
        let result = sqlx::query(
            r#"
            UPDATE messages
            SET is_deleted = TRUE, updated_at = $2
            WHERE id = $1 AND is_deleted = FALSE
            "#,
        )
        .bind(id)
        .bind(Utc::now())
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to delete message: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    async fn hard_delete(&self, id: &str) -> CoreResult<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM messages WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to hard delete message: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    async fn list(&self, limit: Option<i64>, offset: Option<i64>) -> CoreResult<Vec<Message>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        let results = sqlx::query_as::<_, Message>(
            r#"
            SELECT * FROM messages
            WHERE is_deleted = FALSE
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to list messages: {}", e)))?;

        Ok(results)
    }

    async fn count(&self) -> CoreResult<i64> {
        let result = sqlx::query(
            r#"
            SELECT COUNT(*) as count FROM messages
            WHERE is_deleted = FALSE
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to count messages: {}", e)))?;

        Ok(result.try_get("count").unwrap_or(0))
    }
}
