//! Block repository implementation for Core Memory

use async_trait::async_trait;
use chrono::Utc;
use sqlx::{PgPool, Row};

use super::models::Block;
use super::repository::Repository;
use crate::{CoreError, CoreResult};

/// Block repository for Core Memory management
pub struct BlockRepository {
    pool: PgPool,
}

impl BlockRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// List blocks by user
    pub async fn list_by_user(
        &self,
        user_id: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> CoreResult<Vec<Block>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        let results = sqlx::query_as::<_, Block>(
            r#"
            SELECT * FROM blocks
            WHERE user_id = $1 AND is_deleted = FALSE
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to list blocks: {}", e)))?;

        Ok(results)
    }

    /// List blocks by label (human, persona, system)
    pub async fn list_by_label(&self, user_id: &str, label: &str) -> CoreResult<Vec<Block>> {
        let results = sqlx::query_as::<_, Block>(
            r#"
            SELECT * FROM blocks
            WHERE user_id = $1 AND label = $2 AND is_deleted = FALSE
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .bind(label)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to list blocks by label: {}", e)))?;

        Ok(results)
    }

    /// Get template blocks
    pub async fn list_templates(&self) -> CoreResult<Vec<Block>> {
        let results = sqlx::query_as::<_, Block>(
            r#"
            SELECT * FROM blocks
            WHERE is_template = TRUE AND is_deleted = FALSE
            ORDER BY template_name, label
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to list template blocks: {}", e)))?;

        Ok(results)
    }

    /// Get blocks by template name
    pub async fn list_by_template(&self, template_name: &str) -> CoreResult<Vec<Block>> {
        let results = sqlx::query_as::<_, Block>(
            r#"
            SELECT * FROM blocks
            WHERE template_name = $1 AND is_template = TRUE AND is_deleted = FALSE
            ORDER BY label
            "#,
        )
        .bind(template_name)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to list blocks by template: {}", e)))?;

        Ok(results)
    }

    /// List blocks associated with an agent
    pub async fn list_by_agent(&self, agent_id: &str) -> CoreResult<Vec<Block>> {
        let results = sqlx::query_as::<_, Block>(
            r#"
            SELECT b.* FROM blocks b
            INNER JOIN blocks_agents ba ON b.id = ba.block_id
            WHERE ba.agent_id = $1 AND b.is_deleted = FALSE
            ORDER BY ba.block_label, b.created_at DESC
            "#,
        )
        .bind(agent_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to list blocks by agent: {}", e)))?;

        Ok(results)
    }

    /// Validate block value length before insert/update
    fn validate_value_length(value: &str, limit: i64) -> CoreResult<()> {
        if value.len() as i64 > limit {
            return Err(CoreError::ValidationError(format!(
                "Block value length ({}) exceeds limit ({})",
                value.len(),
                limit
            )));
        }
        Ok(())
    }

    /// Create block with validation
    pub async fn create_validated(&self, block: &Block) -> CoreResult<Block> {
        // Validate value length
        Self::validate_value_length(&block.value, block.limit)?;

        // Create block
        self.create(block).await
    }

    /// Update block with validation
    pub async fn update_validated(&self, block: &Block) -> CoreResult<Block> {
        // Validate value length
        Self::validate_value_length(&block.value, block.limit)?;

        // Update block
        self.update(block).await
    }
}

#[async_trait]
impl Repository<Block> for BlockRepository {
    async fn create(&self, block: &Block) -> CoreResult<Block> {
        let result = sqlx::query_as::<_, Block>(
            r#"
            INSERT INTO blocks (
                id, organization_id, user_id, template_name,
                description, label, is_template, value, "limit",
                metadata_, created_at, updated_at, is_deleted,
                created_by_id, last_updated_by_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING *
            "#,
        )
        .bind(&block.id)
        .bind(&block.organization_id)
        .bind(&block.user_id)
        .bind(&block.template_name)
        .bind(&block.description)
        .bind(&block.label)
        .bind(&block.is_template)
        .bind(&block.value)
        .bind(&block.limit)
        .bind(&block.metadata_)
        .bind(&block.created_at)
        .bind(&block.updated_at)
        .bind(&block.is_deleted)
        .bind(&block.created_by_id)
        .bind(&block.last_updated_by_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create block: {}", e)))?;

        Ok(result)
    }

    async fn read(&self, id: &str) -> CoreResult<Option<Block>> {
        let result = sqlx::query_as::<_, Block>(
            r#"
            SELECT * FROM blocks
            WHERE id = $1 AND is_deleted = FALSE
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to read block: {}", e)))?;

        Ok(result)
    }

    async fn update(&self, block: &Block) -> CoreResult<Block> {
        let result = sqlx::query_as::<_, Block>(
            r#"
            UPDATE blocks
            SET template_name = $2, description = $3, label = $4,
                is_template = $5, value = $6, "limit" = $7,
                metadata_ = $8, updated_at = $9, last_updated_by_id = $10
            WHERE id = $1 AND is_deleted = FALSE
            RETURNING *
            "#,
        )
        .bind(&block.id)
        .bind(&block.template_name)
        .bind(&block.description)
        .bind(&block.label)
        .bind(&block.is_template)
        .bind(&block.value)
        .bind(&block.limit)
        .bind(&block.metadata_)
        .bind(Utc::now())
        .bind(&block.last_updated_by_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to update block: {}", e)))?;

        Ok(result)
    }

    async fn delete(&self, id: &str) -> CoreResult<bool> {
        let result = sqlx::query(
            r#"
            UPDATE blocks
            SET is_deleted = TRUE, updated_at = $2
            WHERE id = $1 AND is_deleted = FALSE
            "#,
        )
        .bind(id)
        .bind(Utc::now())
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to delete block: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    async fn hard_delete(&self, id: &str) -> CoreResult<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM blocks WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to hard delete block: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    async fn list(&self, limit: Option<i64>, offset: Option<i64>) -> CoreResult<Vec<Block>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        let results = sqlx::query_as::<_, Block>(
            r#"
            SELECT * FROM blocks
            WHERE is_deleted = FALSE
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to list blocks: {}", e)))?;

        Ok(results)
    }

    async fn count(&self) -> CoreResult<i64> {
        let result = sqlx::query(
            r#"
            SELECT COUNT(*) as count FROM blocks
            WHERE is_deleted = FALSE
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to count blocks: {}", e)))?;

        Ok(result.try_get("count").unwrap_or(0))
    }
}
