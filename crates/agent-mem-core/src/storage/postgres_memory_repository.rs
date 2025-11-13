//! PostgreSQL Memory Repository
//!
//! 提供 PostgreSQL 实现的 MemoryRepositoryTrait

use agent_mem_traits::{AgentMemError, MemoryV4 as Memory, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use std::sync::Arc;

use crate::storage::models::DbMemory;
use crate::storage::traits::MemoryRepositoryTrait;
use crate::storage::conversion::{memory_to_db, db_to_memory};

/// PostgreSQL 实现的 Memory repository
pub struct PostgresMemoryRepository {
    pool: Arc<PgPool>,
}

impl PostgresMemoryRepository {
    /// 创建新的 PostgreSQL memory repository
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MemoryRepositoryTrait for PostgresMemoryRepository {
    async fn create(&self, memory: &Memory) -> Result<Memory> {
        // 1. 转换 V4 Memory 到 DbMemory
        let db_memory = memory_to_db(memory);

        // 2. 插入数据库
        let metadata_json = serde_json::to_string(&db_memory.metadata).map_err(|e| {
            AgentMemError::StorageError(format!("Failed to serialize metadata: {e}"))
        })?;

        sqlx::query(
            r#"
            INSERT INTO memories (
                id, organization_id, user_id, agent_id, content, hash, metadata,
                score, memory_type, scope, level, importance, access_count, last_accessed,
                created_at, updated_at, is_deleted, created_by_id, last_updated_by_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
            "#,
        )
        .bind(&db_memory.id)
        .bind(&db_memory.organization_id)
        .bind(&db_memory.user_id)
        .bind(&db_memory.agent_id)
        .bind(&db_memory.content)
        .bind(&db_memory.hash)
        .bind(metadata_json)
        .bind(db_memory.score)
        .bind(&db_memory.memory_type)
        .bind(&db_memory.scope)
        .bind(&db_memory.level)
        .bind(db_memory.importance)
        .bind(db_memory.access_count as i32)
        .bind(db_memory.last_accessed)
        .bind(db_memory.created_at)
        .bind(db_memory.updated_at)
        .bind(db_memory.is_deleted)
        .bind(&db_memory.created_by_id)
        .bind(&db_memory.last_updated_by_id)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| {
            AgentMemError::StorageError(format!("Failed to create memory: {e}"))
        })?;

        // 3. 返回 Memory V4
        Ok(memory.clone())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<Memory>> {
        // 1. 查询数据库
        let db_memory = sqlx::query_as::<_, DbMemory>(
            r#"
            SELECT id, organization_id, user_id, agent_id, content, hash, metadata,
                   score, memory_type, scope, level, importance, access_count, last_accessed,
                   created_at, updated_at, is_deleted, created_by_id, last_updated_by_id
            FROM memories
            WHERE id = $1 AND is_deleted = FALSE
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| {
            AgentMemError::StorageError(format!("Failed to find memory by id: {e}"))
        })?;

        // 2. 转换到 Memory V4
        match db_memory {
            Some(db) => Ok(Some(db_to_memory(&db)?)),
            None => Ok(None),
        }
    }

    async fn find_by_agent_id(&self, agent_id: &str, limit: i64) -> Result<Vec<Memory>> {
        // 1. 查询数据库
        let db_memories = sqlx::query_as::<_, DbMemory>(
            r#"
            SELECT id, organization_id, user_id, agent_id, content, hash, metadata,
                   score, memory_type, scope, level, importance, access_count, last_accessed,
                   created_at, updated_at, is_deleted, created_by_id, last_updated_by_id
            FROM memories
            WHERE agent_id = $1 AND is_deleted = FALSE
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(agent_id)
        .bind(limit)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| {
            AgentMemError::StorageError(format!("Failed to find memories by agent_id: {e}"))
        })?;

        // 2. 转换到 Memory V4
        let memories: Result<Vec<Memory>> = db_memories.iter().map(|db| db_to_memory(db)).collect();
        memories
    }

    async fn find_by_user_id(&self, user_id: &str, limit: i64) -> Result<Vec<Memory>> {
        // 1. 查询数据库
        let db_memories = sqlx::query_as::<_, DbMemory>(
            r#"
            SELECT id, organization_id, user_id, agent_id, content, hash, metadata,
                   score, memory_type, scope, level, importance, access_count, last_accessed,
                   created_at, updated_at, is_deleted, created_by_id, last_updated_by_id
            FROM memories
            WHERE user_id = $1 AND is_deleted = FALSE
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| {
            AgentMemError::StorageError(format!("Failed to find memories by user_id: {e}"))
        })?;

        // 2. 转换到 Memory V4
        let memories: Result<Vec<Memory>> = db_memories.iter().map(|db| db_to_memory(db)).collect();
        memories
    }

    async fn search(&self, query: &str, limit: i64) -> Result<Vec<Memory>> {
        let search_pattern = format!("%{query}%");

        // 1. 查询数据库
        let db_memories = sqlx::query_as::<_, DbMemory>(
            r#"
            SELECT id, organization_id, user_id, agent_id, content, hash, metadata,
                   score, memory_type, scope, level, importance, access_count, last_accessed,
                   created_at, updated_at, is_deleted, created_by_id, last_updated_by_id
            FROM memories
            WHERE content ILIKE $1 AND is_deleted = FALSE
            ORDER BY importance DESC, created_at DESC
            LIMIT $2
            "#,
        )
        .bind(&search_pattern)
        .bind(limit)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| {
            AgentMemError::StorageError(format!("Failed to search memories: {e}"))
        })?;

        // 2. 转换到 Memory V4
        let memories: Result<Vec<Memory>> = db_memories.iter().map(|db| db_to_memory(db)).collect();
        memories
    }

    async fn update(&self, memory: &Memory) -> Result<Memory> {
        // 1. 转换 V4 Memory 到 DbMemory
        let db_memory = memory_to_db(memory);

        // 2. 更新数据库
        let metadata_json = serde_json::to_string(&db_memory.metadata).map_err(|e| {
            AgentMemError::StorageError(format!("Failed to serialize metadata: {e}"))
        })?;

        let result = sqlx::query(
            r#"
            UPDATE memories
            SET content = $2, hash = $3, metadata = $4, score = $5,
                memory_type = $6, scope = $7, level = $8, importance = $9,
                access_count = $10, last_accessed = $11, updated_at = $12,
                last_updated_by_id = $13
            WHERE id = $1 AND is_deleted = FALSE
            "#,
        )
        .bind(&db_memory.id)
        .bind(&db_memory.content)
        .bind(&db_memory.hash)
        .bind(metadata_json)
        .bind(db_memory.score)
        .bind(&db_memory.memory_type)
        .bind(&db_memory.scope)
        .bind(&db_memory.level)
        .bind(db_memory.importance)
        .bind(db_memory.access_count as i32)
        .bind(db_memory.last_accessed)
        .bind(db_memory.updated_at)
        .bind(&db_memory.last_updated_by_id)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| {
            AgentMemError::StorageError(format!("Failed to update memory: {e}"))
        })?;

        if result.rows_affected() == 0 {
            return Err(AgentMemError::NotFound(format!(
                "Memory not found: {}",
                memory.id.as_str()
            )));
        }

        // 3. 返回 Memory V4
        Ok(memory.clone())
    }

    async fn delete(&self, id: &str) -> Result<()> {
        // 软删除：设置 is_deleted = TRUE
        let result = sqlx::query(
            r#"
            UPDATE memories
            SET is_deleted = TRUE, updated_at = $2
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(Utc::now())
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| {
            AgentMemError::StorageError(format!("Failed to delete memory: {e}"))
        })?;

        if result.rows_affected() == 0 {
            return Err(AgentMemError::NotFound(format!("Memory not found: {}", id)));
        }

        Ok(())
    }

    async fn delete_by_agent_id(&self, agent_id: &str) -> Result<u64> {
        // 软删除：设置 is_deleted = TRUE
        let result = sqlx::query(
            r#"
            UPDATE memories
            SET is_deleted = TRUE, updated_at = $2
            WHERE agent_id = $1 AND is_deleted = FALSE
            "#,
        )
        .bind(agent_id)
        .bind(Utc::now())
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| {
            AgentMemError::StorageError(format!("Failed to delete memories by agent_id: {e}"))
        })?;

        Ok(result.rows_affected())
    }

    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Memory>> {
        // 1. 查询数据库
        let db_memories = sqlx::query_as::<_, DbMemory>(
            r#"
            SELECT id, organization_id, user_id, agent_id, content, hash, metadata,
                   score, memory_type, scope, level, importance, access_count, last_accessed,
                   created_at, updated_at, is_deleted, created_by_id, last_updated_by_id
            FROM memories
            WHERE is_deleted = FALSE
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| {
            AgentMemError::StorageError(format!("Failed to list memories: {e}"))
        })?;

        // 2. 转换到 Memory V4
        let memories: Result<Vec<Memory>> = db_memories.iter().map(|db| db_to_memory(db)).collect();
        memories
    }
}

