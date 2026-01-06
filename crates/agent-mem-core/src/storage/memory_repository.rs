//! Memory repository implementation with multi-tenancy support

use async_trait::async_trait;
use chrono::Utc;
use sqlx::{PgPool, Row};

use super::models::DbMemory;
use super::repository::Repository;
use crate::search::metadata_filter::{LogicalOperator, MetadataFilterSystem};
use crate::{CoreError, CoreResult};

/// Memory repository with enhanced multi-tenant support
pub struct MemoryRepository {
    pool: PgPool,
}

impl MemoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// List memories by agent
    pub async fn list_by_agent(
        &self,
        agent_id: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> CoreResult<Vec<DbMemory>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        let results = sqlx::query_as::<_, DbMemory>(
            r#"
            SELECT * FROM memories
            WHERE agent_id = $1 AND is_deleted = FALSE
            ORDER BY importance DESC, created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(agent_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to list memories by agent: {}", e)))?;

        Ok(results)
    }

    /// List memories by user
    pub async fn list_by_user(
        &self,
        user_id: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> CoreResult<Vec<DbMemory>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        let results = sqlx::query_as::<_, DbMemory>(
            r#"
            SELECT * FROM memories
            WHERE user_id = $1 AND is_deleted = FALSE
            ORDER BY importance DESC, created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to list memories by user: {}", e)))?;

        Ok(results)
    }

    /// List memories by type
    pub async fn list_by_type(
        &self,
        agent_id: &str,
        memory_type: &str,
        limit: Option<i64>,
    ) -> CoreResult<Vec<DbMemory>> {
        let limit = limit.unwrap_or(50);

        let results = sqlx::query_as::<_, DbMemory>(
            r#"
            SELECT * FROM memories
            WHERE agent_id = $1 AND memory_type = $2 AND is_deleted = FALSE
            ORDER BY importance DESC, created_at DESC
            LIMIT $3
            "#,
        )
        .bind(agent_id)
        .bind(memory_type)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to list memories by type: {}", e)))?;

        Ok(results)
    }

    /// List memories by scope
    pub async fn list_by_scope(
        &self,
        agent_id: &str,
        scope: &str,
        limit: Option<i64>,
    ) -> CoreResult<Vec<DbMemory>> {
        let limit = limit.unwrap_or(50);

        let results = sqlx::query_as::<_, DbMemory>(
            r#"
            SELECT * FROM memories
            WHERE agent_id = $1 AND scope = $2 AND is_deleted = FALSE
            ORDER BY importance DESC, created_at DESC
            LIMIT $3
            "#,
        )
        .bind(agent_id)
        .bind(scope)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to list memories by scope: {}", e)))?;

        Ok(results)
    }

    /// List memories by level
    pub async fn list_by_level(
        &self,
        agent_id: &str,
        level: &str,
        limit: Option<i64>,
    ) -> CoreResult<Vec<DbMemory>> {
        let limit = limit.unwrap_or(50);

        let results = sqlx::query_as::<_, DbMemory>(
            r#"
            SELECT * FROM memories
            WHERE agent_id = $1 AND level = $2 AND is_deleted = FALSE
            ORDER BY importance DESC, created_at DESC
            LIMIT $3
            "#,
        )
        .bind(agent_id)
        .bind(level)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to list memories by level: {}", e)))?;

        Ok(results)
    }

    /// Full-text search memories
    pub async fn search_fulltext(
        &self,
        agent_id: &str,
        query: &str,
        limit: Option<i64>,
    ) -> CoreResult<Vec<DbMemory>> {
        let limit = limit.unwrap_or(50);

        let results = sqlx::query_as::<_, DbMemory>(
            r#"
            SELECT * FROM memories
            WHERE agent_id = $1 
              AND is_deleted = FALSE
              AND to_tsvector('english', content) @@ plainto_tsquery('english', $2)
            ORDER BY importance DESC, created_at DESC
            LIMIT $3
            "#,
        )
        .bind(agent_id)
        .bind(query)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to search memories: {}", e)))?;

        Ok(results)
    }

    /// Get most important memories
    pub async fn get_most_important(
        &self,
        agent_id: &str,
        limit: i64,
    ) -> CoreResult<Vec<DbMemory>> {
        let results = sqlx::query_as::<_, DbMemory>(
            r#"
            SELECT * FROM memories
            WHERE agent_id = $1 AND is_deleted = FALSE
            ORDER BY importance DESC, access_count DESC, created_at DESC
            LIMIT $2
            "#,
        )
        .bind(agent_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            CoreError::Database(format!("Failed to get most important memories: {}", e))
        })?;

        Ok(results)
    }

    /// Get recently accessed memories
    pub async fn get_recently_accessed(
        &self,
        agent_id: &str,
        limit: i64,
    ) -> CoreResult<Vec<DbMemory>> {
        let results = sqlx::query_as::<_, DbMemory>(
            r#"
            SELECT * FROM memories
            WHERE agent_id = $1 AND is_deleted = FALSE AND last_accessed IS NOT NULL
            ORDER BY last_accessed DESC
            LIMIT $2
            "#,
        )
        .bind(agent_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            CoreError::Database(format!("Failed to get recently accessed memories: {}", e))
        })?;

        Ok(results)
    }

    /// Update memory access
    pub async fn update_access(&self, memory_id: &str) -> CoreResult<()> {
        sqlx::query(
            r#"
            UPDATE memories
            SET access_count = access_count + 1,
                last_accessed = $2,
                updated_at = $2
            WHERE id = $1 AND is_deleted = FALSE
            "#,
        )
        .bind(memory_id)
        .bind(Utc::now())
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to update memory access: {}", e)))?;

        Ok(())
    }

    /// Batch create memories
    pub async fn batch_create(&self, memories: &[DbMemory]) -> CoreResult<Vec<DbMemory>> {
        let mut created_memories = Vec::new();

        for memory in memories {
            let created = self.create(memory).await?;
            created_memories.push(created);
        }

        Ok(created_memories)
    }

    /// Delete old memories (keep only N most recent)
    pub async fn delete_old_memories(&self, agent_id: &str, keep_count: i64) -> CoreResult<i64> {
        // Get IDs of memories to keep
        let keep_ids: Vec<String> = sqlx::query_scalar(
            r#"
            SELECT id FROM memories
            WHERE agent_id = $1 AND is_deleted = FALSE
            ORDER BY importance DESC, created_at DESC
            LIMIT $2
            "#,
        )
        .bind(agent_id)
        .bind(keep_count)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to get memory IDs: {}", e)))?;

        if keep_ids.is_empty() {
            return Ok(0);
        }

        // Soft delete memories not in keep list
        let result = sqlx::query(
            r#"
            UPDATE memories
            SET is_deleted = TRUE, updated_at = $3
            WHERE agent_id = $1 AND id != ALL($2) AND is_deleted = FALSE
            "#,
        )
        .bind(agent_id)
        .bind(&keep_ids)
        .bind(Utc::now())
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to delete old memories: {}", e)))?;

        Ok(result.rows_affected() as i64)
    }

    /// Count memories by agent
    pub async fn count_by_agent(&self, agent_id: &str) -> CoreResult<i64> {
        let result = sqlx::query(
            r#"
            SELECT COUNT(*) as count FROM memories
            WHERE agent_id = $1 AND is_deleted = FALSE
            "#,
        )
        .bind(agent_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to count memories: {}", e)))?;

        Ok(result.try_get("count").unwrap_or(0))
    }
}

#[async_trait]
impl Repository<DbMemory> for MemoryRepository {
    async fn create(&self, memory: &DbMemory) -> CoreResult<DbMemory> {
        let result = sqlx::query_as::<_, DbMemory>(
            r#"
            INSERT INTO memories (
                id, organization_id, user_id, agent_id, content,
                hash, metadata, score, memory_type, scope, level,
                importance, access_count, last_accessed,
                created_at, updated_at, is_deleted,
                created_by_id, last_updated_by_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
            RETURNING *
            "#,
        )
        .bind(&memory.id)
        .bind(&memory.organization_id)
        .bind(&memory.user_id)
        .bind(&memory.agent_id)
        .bind(&memory.content)
        .bind(&memory.hash)
        .bind(&memory.metadata)
        .bind(&memory.score)
        .bind(&memory.memory_type)
        .bind(&memory.scope)
        .bind(&memory.level)
        .bind(&memory.importance)
        .bind(&memory.access_count)
        .bind(&memory.last_accessed)
        .bind(&memory.created_at)
        .bind(&memory.updated_at)
        .bind(&memory.is_deleted)
        .bind(&memory.created_by_id)
        .bind(&memory.last_updated_by_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create memory: {}", e)))?;

        Ok(result)
    }

    async fn read(&self, id: &str) -> CoreResult<Option<DbMemory>> {
        let result = sqlx::query_as::<_, DbMemory>(
            r#"
            SELECT * FROM memories
            WHERE id = $1 AND is_deleted = FALSE
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to read memory: {}", e)))?;

        Ok(result)
    }

    async fn update(&self, memory: &DbMemory) -> CoreResult<DbMemory> {
        let result = sqlx::query_as::<_, DbMemory>(
            r#"
            UPDATE memories
            SET content = $2, hash = $3, metadata = $4, score = $5,
                memory_type = $6, scope = $7, level = $8, importance = $9,
                updated_at = $10, last_updated_by_id = $11
            WHERE id = $1 AND is_deleted = FALSE
            RETURNING *
            "#,
        )
        .bind(&memory.id)
        .bind(&memory.content)
        .bind(&memory.hash)
        .bind(&memory.metadata)
        .bind(&memory.score)
        .bind(&memory.memory_type)
        .bind(&memory.scope)
        .bind(&memory.level)
        .bind(&memory.importance)
        .bind(Utc::now())
        .bind(&memory.last_updated_by_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to update memory: {}", e)))?;

        Ok(result)
    }

    async fn delete(&self, id: &str) -> CoreResult<bool> {
        let result = sqlx::query(
            r#"
            UPDATE memories
            SET is_deleted = TRUE, updated_at = $2
            WHERE id = $1 AND is_deleted = FALSE
            "#,
        )
        .bind(id)
        .bind(Utc::now())
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to delete memory: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    async fn hard_delete(&self, id: &str) -> CoreResult<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM memories WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to hard delete memory: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    async fn list(&self, limit: Option<i64>, offset: Option<i64>) -> CoreResult<Vec<DbMemory>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        let results = sqlx::query_as::<_, DbMemory>(
            r#"
            SELECT * FROM memories
            WHERE is_deleted = FALSE
            ORDER BY importance DESC, created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to list memories: {}", e)))?;

        Ok(results)
    }

    async fn count(&self) -> CoreResult<i64> {
        let result = sqlx::query(
            r#"
            SELECT COUNT(*) as count FROM memories
            WHERE is_deleted = FALSE
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to count memories: {}", e)))?;

        Ok(result.try_get("count").unwrap_or(0))
    }
}
