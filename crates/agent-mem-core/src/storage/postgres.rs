// PostgreSQL storage backend implementation

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json;
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use std::collections::HashMap;
use std::time::Instant;

use super::{migrations, HealthStatus, PostgresConfig, StorageBackend, StorageStatistics};
use crate::hierarchy::{HierarchicalMemory, MemoryLevel, MemoryScope};
use crate::{types::*, CoreError, CoreResult};

/// PostgreSQL storage backend
pub struct PostgresStorage {
    pool: PgPool,
    config: PostgresConfig,
}

impl PostgresStorage {
    /// Create new PostgreSQL storage backend
    pub async fn new(config: PostgresConfig) -> CoreResult<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .connect(&config.url)
            .await
            .map_err(|e| CoreError::Database(format!("Failed to connect to PostgreSQL: {}", e)))?;

        Ok(Self { pool, config })
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Run database migrations
    ///
    /// This now uses the comprehensive migration system that creates all tables:
    /// - organizations
    /// - users
    /// - agents
    /// - messages
    /// - blocks
    /// - tools
    /// - memories (enhanced with foreign keys)
    /// - junction tables (blocks_agents, tools_agents)
    /// - all necessary indexes
    pub async fn migrate(&self) -> CoreResult<()> {
        migrations::run_migrations(&self.pool).await
    }

    /// Convert database row to HierarchicalMemory
    fn row_to_memory(&self, row: &sqlx::postgres::PgRow) -> CoreResult<HierarchicalMemory> {
        let metadata_json: serde_json::Value = row
            .try_get("metadata")
            .map_err(|e| CoreError::Database(format!("Failed to get metadata: {}", e)))?;

        let metadata: HashMap<String, serde_json::Value> = serde_json::from_value(metadata_json)
            .map_err(|e| {
                CoreError::SerializationError(format!("Failed to deserialize metadata: {}", e))
            })?;

        let memory_type_str: String = row
            .try_get("memory_type")
            .map_err(|e| CoreError::Database(format!("Failed to get memory_type: {}", e)))?;

        let scope_str: String = row
            .try_get("scope")
            .map_err(|e| CoreError::Database(format!("Failed to get scope: {}", e)))?;

        let level_str: String = row
            .try_get("level")
            .map_err(|e| CoreError::Database(format!("Failed to get level: {}", e)))?;

        let memory_type = MemoryType::from_str(&memory_type_str).ok_or_else(|| {
            CoreError::ValidationError(format!("Invalid memory type: {}", memory_type_str))
        })?;

        let scope = MemoryScope::from_str(&scope_str)
            .ok_or_else(|| CoreError::ValidationError(format!("Invalid scope: {}", scope_str)))?;

        let level = MemoryLevel::from_str(&level_str)
            .ok_or_else(|| CoreError::ValidationError(format!("Invalid level: {}", level_str)))?;

        let created_at: chrono::DateTime<chrono::Utc> = row
            .try_get("created_at")
            .map_err(|e| CoreError::Database(format!("Failed to get created_at: {}", e)))?;
        let last_accessed: Option<chrono::DateTime<chrono::Utc>> =
            row.try_get("last_accessed").ok();

        // Convert metadata from JSONB to HashMap<String, String>
        let metadata_map: HashMap<String, String> = metadata
            .into_iter()
            .filter_map(|(k, v)| match v {
                serde_json::Value::String(s) => Some((k, s)),
                _ => Some((k, v.to_string())),
            })
            .collect();

        let memory = crate::types::Memory {
            id: row
                .try_get("id")
                .map_err(|e| CoreError::Database(format!("Failed to get id: {}", e)))?,

            // ✅ Read agent_id from database, fallback to "default"
            agent_id: row
                .try_get("agent_id")
                .unwrap_or_else(|_| "default".to_string()),

            // ✅ Read user_id from database (optional field)
            user_id: row.try_get("user_id").ok(),

            memory_type,
            content: row
                .try_get("content")
                .map_err(|e| CoreError::Database(format!("Failed to get content: {}", e)))?,
            importance: row
                .try_get("importance")
                .map_err(|e| CoreError::Database(format!("Failed to get importance: {}", e)))?,

            // ✅ Read embedding from database (JSONB format)
            embedding: row
                .try_get::<Option<serde_json::Value>, _>("embedding")
                .ok()
                .flatten()
                .and_then(|v| {
                    serde_json::from_value::<Vec<f32>>(v)
                        .ok()
                        .map(|values| crate::types::Vector { values })
                }),

            created_at: created_at.timestamp(),
            last_accessed_at: last_accessed
                .map(|dt| dt.timestamp())
                .unwrap_or_else(|| chrono::Utc::now().timestamp()),
            access_count: row
                .try_get::<i64, _>("access_count")
                .map(|v| v as u32)
                .unwrap_or(0),

            // ✅ Read expires_at from database, convert to timestamp
            expires_at: row
                .try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("expires_at")
                .ok()
                .flatten()
                .map(|dt| dt.timestamp()),

            metadata: metadata_map,

            // ✅ Read version from database, fallback to 1
            version: row.try_get("version").unwrap_or(1),
        };

        use crate::hierarchy::{HierarchyMetadata, MemoryInheritance, MemoryPermissions};

        Ok(HierarchicalMemory {
            memory: memory.into(),
            scope,
            level,
            hierarchy_metadata: HierarchyMetadata {
                level_assigned_at: chrono::Utc::now(),
                promotion_count: 0,
                demotion_count: 0,
                inheritance: MemoryInheritance::default(),
                permissions: MemoryPermissions::default(),
            },
        })
    }
}

#[async_trait]
impl StorageBackend for PostgresStorage {
    async fn initialize(&self) -> CoreResult<()> {
        self.migrate().await
    }

    async fn store_memory(&self, memory: &HierarchicalMemory) -> CoreResult<()> {
        let metadata_json = serde_json::to_value(&memory.memory.metadata).map_err(|e| {
            CoreError::SerializationError(format!("Failed to serialize metadata: {}", e))
        })?;

        // 序列化 embedding 为 JSON（如果存在）
        let embedding_json = memory
            .memory
            .embedding
            .as_ref()
            .map(|emb| serde_json::to_value(&emb.values).ok())
            .flatten();

        // 转换 expires_at 为 DateTime
        let expires_at = memory.memory.expires_at.map(|ts| {
            chrono::DateTime::from_timestamp(ts, 0).unwrap_or_else(|| chrono::Utc::now())
        });

        sqlx::query(
            r#"
            INSERT INTO memories (
                id, organization_id, user_id, agent_id, content, hash, metadata, score,
                memory_type, scope, level, importance, access_count, last_accessed,
                embedding, expires_at, version, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
            ON CONFLICT (id) DO UPDATE SET
                content = EXCLUDED.content,
                hash = EXCLUDED.hash,
                metadata = EXCLUDED.metadata,
                score = EXCLUDED.score,
                memory_type = EXCLUDED.memory_type,
                scope = EXCLUDED.scope,
                level = EXCLUDED.level,
                importance = EXCLUDED.importance,
                access_count = EXCLUDED.access_count,
                last_accessed = EXCLUDED.last_accessed,
                embedding = EXCLUDED.embedding,
                expires_at = EXCLUDED.expires_at,
                version = EXCLUDED.version,
                updated_at = NOW()
            "#,
        )
        .bind(&memory.memory.id)
        .bind("default_org") // TODO: 从上下文获取 organization_id
        .bind(&memory.memory.user_id().unwrap_or_else(|| "default_user".to_string()))
        .bind(&memory.memory.agent_id())
        .bind(&memory.memory.content)
        .bind(None::<String>) // hash - not available in Memory struct
        .bind(&metadata_json)
        .bind(Some(memory.memory.importance())) // score mapped from importance
        .bind(memory.memory.memory_type().as_str())
        .bind(memory.scope.as_str())
        .bind(memory.level.as_str())
        .bind(&memory.memory.importance())
        .bind(memory.memory.metadata.access_count as i64)
        .bind(&memory.memory.last_accessed_at())
        .bind(&embedding_json)
        .bind(&expires_at)
        .bind(memory.memory.version() as i32)
        .bind(&memory.memory.created_at())
        .bind(&memory.memory.last_accessed_at()) // updated_at mapped from last_accessed_at
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to store memory: {}", e)))?;

        Ok(())
    }

    async fn get_memory(&self, id: &str) -> CoreResult<Option<HierarchicalMemory>> {
        let row = sqlx::query("SELECT * FROM memories WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| CoreError::Database(format!("Failed to get memory: {}", e)))?;

        match row {
            Some(row) => {
                let memory = self.row_to_memory(&row)?;

                // Update access count and last accessed time
                let _ = sqlx::query(
                    "UPDATE memories SET access_count = access_count + 1, last_accessed = NOW() WHERE id = $1"
                )
                .bind(id)
                .execute(&self.pool)
                .await;

                Ok(Some(memory))
            }
            None => Ok(None),
        }
    }

    async fn update_memory(&self, memory: &HierarchicalMemory) -> CoreResult<()> {
        let metadata_json = serde_json::to_value(&memory.memory.metadata).map_err(|e| {
            CoreError::SerializationError(format!("Failed to serialize metadata: {}", e))
        })?;

        // 序列化 embedding 为 JSON（如果存在）
        let embedding_json = memory
            .memory
            .embedding
            .as_ref()
            .map(|emb| serde_json::to_value(&emb.values).ok())
            .flatten();

        // 转换 expires_at 为 DateTime
        let expires_at = memory.memory.expires_at.map(|ts| {
            chrono::DateTime::from_timestamp(ts, 0).unwrap_or_else(|| chrono::Utc::now())
        });

        let result = sqlx::query(
            r#"
            UPDATE memories SET
                content = $2,
                hash = $3,
                metadata = $4,
                score = $5,
                memory_type = $6,
                scope = $7,
                level = $8,
                importance = $9,
                embedding = $10,
                expires_at = $11,
                version = $12,
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(&memory.memory.id)
        .bind(&memory.memory.content)
        .bind(None::<String>) // hash - not available in Memory struct
        .bind(&metadata_json)
        .bind(Some(memory.memory.importance)) // score mapped from importance
        .bind(memory.memory.memory_type.as_str())
        .bind(memory.scope.as_str())
        .bind(memory.level.as_str())
        .bind(&memory.memory.importance)
        .bind(&embedding_json)
        .bind(&expires_at)
        .bind(memory.memory.version as i32)
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to update memory: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound(format!(
                "Memory with id {} not found",
                memory.memory.id
            )));
        }

        Ok(())
    }

    async fn delete_memory(&self, id: &str) -> CoreResult<bool> {
        let result = sqlx::query("DELETE FROM memories WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| CoreError::Database(format!("Failed to delete memory: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    async fn search_memories(
        &self,
        query: &str,
        scope: Option<MemoryScope>,
        level: Option<MemoryLevel>,
        limit: Option<usize>,
    ) -> CoreResult<Vec<HierarchicalMemory>> {
        let mut sql = String::from(
            "SELECT * FROM memories WHERE to_tsvector('english', content) @@ plainto_tsquery('english', $1)"
        );
        let mut param_count = 1;

        if scope.is_some() {
            param_count += 1;
            sql.push_str(&format!(" AND scope = ${}", param_count));
        }

        if level.is_some() {
            param_count += 1;
            sql.push_str(&format!(" AND level = ${}", param_count));
        }

        sql.push_str(" ORDER BY importance DESC, created_at DESC");

        if let Some(limit) = limit {
            param_count += 1;
            sql.push_str(&format!(" LIMIT ${}", param_count));
        }

        let mut query_builder = sqlx::query(&sql).bind(query);

        if let Some(scope) = scope {
            query_builder = query_builder.bind(scope.as_str());
        }

        if let Some(level) = level {
            query_builder = query_builder.bind(level.as_str());
        }

        if let Some(limit) = limit {
            query_builder = query_builder.bind(limit as i64);
        }

        let rows = query_builder
            .fetch_all(&self.pool)
            .await
            .map_err(|e| CoreError::Database(format!("Failed to search memories: {}", e)))?;

        let mut memories = Vec::new();
        for row in rows {
            memories.push(self.row_to_memory(&row)?);
        }

        Ok(memories)
    }

    async fn get_memories_by_scope(
        &self,
        scope: MemoryScope,
        limit: Option<usize>,
    ) -> CoreResult<Vec<HierarchicalMemory>> {
        let mut sql = String::from(
            "SELECT * FROM memories WHERE scope = $1 ORDER BY importance DESC, created_at DESC",
        );

        if limit.is_some() {
            sql.push_str(" LIMIT $2");
        }

        let mut query_builder = sqlx::query(&sql).bind(scope.as_str());

        if let Some(limit) = limit {
            query_builder = query_builder.bind(limit as i64);
        }

        let rows = query_builder
            .fetch_all(&self.pool)
            .await
            .map_err(|e| CoreError::Database(format!("Failed to get memories by scope: {}", e)))?;

        let mut memories = Vec::new();
        for row in rows {
            memories.push(self.row_to_memory(&row)?);
        }

        Ok(memories)
    }

    async fn get_memories_by_level(
        &self,
        level: MemoryLevel,
        limit: Option<usize>,
    ) -> CoreResult<Vec<HierarchicalMemory>> {
        let mut sql = String::from(
            "SELECT * FROM memories WHERE level = $1 ORDER BY importance DESC, created_at DESC",
        );

        if limit.is_some() {
            sql.push_str(" LIMIT $2");
        }

        let mut query_builder = sqlx::query(&sql).bind(level.as_str());

        if let Some(limit) = limit {
            query_builder = query_builder.bind(limit as i64);
        }

        let rows = query_builder
            .fetch_all(&self.pool)
            .await
            .map_err(|e| CoreError::Database(format!("Failed to get memories by level: {}", e)))?;

        let mut memories = Vec::new();
        for row in rows {
            memories.push(self.row_to_memory(&row)?);
        }

        Ok(memories)
    }

    async fn get_statistics(&self) -> CoreResult<StorageStatistics> {
        // Get total count and size
        let total_row = sqlx::query(
            "SELECT COUNT(*) as total_memories, COALESCE(SUM(LENGTH(content)), 0) as storage_size FROM memories"
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to get total statistics: {}", e)))?;

        let total_memories: i64 = total_row
            .try_get("total_memories")
            .map_err(|e| CoreError::Database(format!("Failed to get total_memories: {}", e)))?;
        let storage_size: i64 = total_row
            .try_get("storage_size")
            .map_err(|e| CoreError::Database(format!("Failed to get storage_size: {}", e)))?;

        // Get statistics by level
        let level_rows =
            sqlx::query("SELECT level, COUNT(*) as count FROM memories GROUP BY level")
                .fetch_all(&self.pool)
                .await
                .map_err(|e| {
                    CoreError::Database(format!("Failed to get level statistics: {}", e))
                })?;

        let mut memories_by_level = HashMap::new();
        for row in level_rows {
            let level_str: String = row
                .try_get("level")
                .map_err(|e| CoreError::Database(format!("Failed to get level: {}", e)))?;
            let count: i64 = row
                .try_get("count")
                .map_err(|e| CoreError::Database(format!("Failed to get count: {}", e)))?;

            if let Some(level) = MemoryLevel::from_str(&level_str) {
                memories_by_level.insert(level, count as u64);
            }
        }

        // Get statistics by scope
        let scope_rows =
            sqlx::query("SELECT scope, COUNT(*) as count FROM memories GROUP BY scope")
                .fetch_all(&self.pool)
                .await
                .map_err(|e| {
                    CoreError::Database(format!("Failed to get scope statistics: {}", e))
                })?;

        let mut memories_by_scope = HashMap::new();
        for row in scope_rows {
            let scope_str: String = row
                .try_get("scope")
                .map_err(|e| CoreError::Database(format!("Failed to get scope: {}", e)))?;
            let count: i64 = row
                .try_get("count")
                .map_err(|e| CoreError::Database(format!("Failed to get count: {}", e)))?;

            if let Some(scope) = MemoryScope::from_str(&scope_str) {
                memories_by_scope.insert(scope, count as u64);
            }
        }

        let average_memory_size = if total_memories > 0 {
            storage_size as f64 / total_memories as f64
        } else {
            0.0
        };

        Ok(StorageStatistics {
            total_memories: total_memories as u64,
            storage_size: storage_size as u64,
            memories_by_level,
            memories_by_scope,
            average_memory_size,
            last_updated: Utc::now(),
        })
    }

    async fn health_check(&self) -> CoreResult<HealthStatus> {
        let start = Instant::now();

        match sqlx::query("SELECT 1").fetch_one(&self.pool).await {
            Ok(_) => Ok(HealthStatus {
                healthy: true,
                message: "PostgreSQL connection is healthy".to_string(),
                response_time_ms: start.elapsed().as_millis() as u64,
                last_check: Utc::now(),
            }),
            Err(e) => Ok(HealthStatus {
                healthy: false,
                message: format!("PostgreSQL connection failed: {}", e),
                response_time_ms: start.elapsed().as_millis() as u64,
                last_check: Utc::now(),
            }),
        }
    }
}
