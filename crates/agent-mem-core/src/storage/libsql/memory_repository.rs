//! LibSQL Memory Repository
//!
//! Provides LibSQL implementation of MemoryRepositoryTrait

use agent_mem_traits::{AgentMemError, MemoryV4 as Memory, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use libsql::Connection;
use serde_json::Value as JsonValue;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::storage::models::DbMemory;
use crate::storage::traits::MemoryRepositoryTrait;
use crate::storage::conversion::{memory_to_db, db_to_memory};
use crate::search::metadata_filter::{LogicalOperator, MetadataFilterSystem};

/// LibSQL implementation of Memory repository
pub struct LibSqlMemoryRepository {
    conn: Arc<Mutex<Connection>>,
}

impl LibSqlMemoryRepository {
    /// Create a new LibSQL memory repository
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Batch create memories (optimized for performance)
    ///
    /// Uses a transaction to insert multiple memories efficiently.
    /// Performance: ~10-20x faster than individual inserts for large batches.
    pub async fn batch_create(&self, memories: &[&Memory]) -> Result<Vec<Memory>> {
        if memories.is_empty() {
            return Ok(Vec::new());
        }

        let conn = self.conn.lock().await;

        // Start transaction
        conn.execute("BEGIN TRANSACTION", libsql::params![])
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to begin transaction: {e}")))?;

        let mut created_memories = Vec::new();

        for memory in memories {
            let db_memory = memory_to_db(memory);

            let metadata_json = serde_json::to_string(&db_memory.metadata).map_err(|e| {
                AgentMemError::StorageError(format!("Failed to serialize metadata: {e}"))
            })?;

            match conn.execute(
                "INSERT INTO memories (
                    id, organization_id, user_id, agent_id, content, hash, metadata,
                    score, memory_type, scope, level, importance, access_count, last_accessed,
                    created_at, updated_at, is_deleted, created_by_id, last_updated_by_id
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                libsql::params![
                    db_memory.id,
                    db_memory.organization_id,
                    db_memory.user_id,
                    db_memory.agent_id,
                    db_memory.content,
                    db_memory.hash,
                    metadata_json,
                    db_memory.score,
                    db_memory.memory_type,
                    db_memory.scope,
                    db_memory.level,
                    db_memory.importance,
                    db_memory.access_count,
                    db_memory.last_accessed.map(|dt| dt.timestamp()),
                    db_memory.created_at.timestamp(),
                    db_memory.updated_at.timestamp(),
                    if db_memory.is_deleted { 1 } else { 0 },
                    db_memory.created_by_id,
                    db_memory.last_updated_by_id,
                ],
            )
            .await
            {
                Ok(_) => created_memories.push((*memory).clone()),
                Err(e) => {
                    // Rollback on error
                    let _ = conn.execute("ROLLBACK", libsql::params![]).await;
                    return Err(AgentMemError::StorageError(format!(
                        "Failed to insert memory in batch: {e}"
                    )));
                }
            }
        }

        // Commit transaction
        conn.execute("COMMIT", libsql::params![])
            .await
            .map_err(|e| {
                AgentMemError::StorageError(format!("Failed to commit transaction: {e}"))
            })?;

        Ok(created_memories)
    }

    /// Helper function to convert row to DbMemory
    fn row_to_memory(row: &libsql::Row) -> Result<DbMemory> {
        // Column order from SELECT:
        // 0: id, 1: organization_id, 2: user_id, 3: agent_id, 4: content, 5: hash, 6: metadata,
        // 7: score, 8: memory_type, 9: scope, 10: level, 11: importance, 12: access_count, 13: last_accessed,
        // 14: created_at, 15: updated_at, 16: is_deleted, 17: created_by_id, 18: last_updated_by_id

        let metadata_str: Option<String> = row.get(6).ok();
        let metadata: JsonValue = metadata_str
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or(JsonValue::Null);

        let last_accessed_ts: Option<i64> = row.get(13).ok();
        let last_accessed = last_accessed_ts.and_then(|ts| DateTime::from_timestamp(ts, 0));

        let created_at_ts: i64 = row
            .get(14)
            .map_err(|e| AgentMemError::StorageError(format!("Failed to get created_at: {e}")))?;
        let created_at = DateTime::from_timestamp(created_at_ts, 0).ok_or_else(|| {
            AgentMemError::StorageError("Invalid created_at timestamp".to_string())
        })?;

        let updated_at_ts: i64 = row
            .get(15)
            .map_err(|e| AgentMemError::StorageError(format!("Failed to get updated_at: {e}")))?;
        let updated_at = DateTime::from_timestamp(updated_at_ts, 0).ok_or_else(|| {
            AgentMemError::StorageError("Invalid updated_at timestamp".to_string())
        })?;

        let is_deleted_int: i64 = row.get(16).unwrap_or(0);

        let score_f64: Option<f64> = row.get(7).ok();
        let importance_f64: f64 = row
            .get(11)
            .map_err(|e| AgentMemError::StorageError(format!("Failed to get importance: {e}")))?;

        Ok(DbMemory {
            id: row
                .get(0)
                .map_err(|e| AgentMemError::StorageError(format!("Failed to get id: {e}")))?,
            organization_id: row.get(1).map_err(|e| {
                AgentMemError::StorageError(format!("Failed to get organization_id: {e}"))
            })?,
            user_id: row
                .get(2)
                .map_err(|e| AgentMemError::StorageError(format!("Failed to get user_id: {e}")))?,
            agent_id: row
                .get(3)
                .map_err(|e| AgentMemError::StorageError(format!("Failed to get agent_id: {e}")))?,
            content: row
                .get(4)
                .map_err(|e| AgentMemError::StorageError(format!("Failed to get content: {e}")))?,
            hash: row.get(5).ok(),
            metadata,
            score: score_f64.map(|v| v as f32),
            memory_type: row.get(8).map_err(|e| {
                AgentMemError::StorageError(format!("Failed to get memory_type: {e}"))
            })?,
            scope: row
                .get(9)
                .map_err(|e| AgentMemError::StorageError(format!("Failed to get scope: {e}")))?,
            level: row
                .get(10)
                .map_err(|e| AgentMemError::StorageError(format!("Failed to get level: {e}")))?,
            importance: importance_f64 as f32,
            access_count: row.get(12).map_err(|e| {
                AgentMemError::StorageError(format!("Failed to get access_count: {e}"))
            })?,
            last_accessed,
            created_at,
            updated_at,
            is_deleted: is_deleted_int != 0,
            created_by_id: row.get(17).ok(),
            last_updated_by_id: row.get(18).ok(),
        })
    }
}

#[async_trait]
impl MemoryRepositoryTrait for LibSqlMemoryRepository {
    async fn create(&self, memory: &Memory) -> Result<Memory> {
        let conn = self.conn.lock().await;

        // Convert V4 Memory to DbMemory
        let db_memory = memory_to_db(memory);

        let metadata_json = serde_json::to_string(&db_memory.metadata).map_err(|e| {
            AgentMemError::StorageError(format!("Failed to serialize metadata: {e}"))
        })?;

        conn.execute(
            "INSERT INTO memories (
                id, organization_id, user_id, agent_id, content, hash, metadata,
                score, memory_type, scope, level, importance, access_count, last_accessed,
                created_at, updated_at, is_deleted, created_by_id, last_updated_by_id
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            libsql::params![
                db_memory.id,
                db_memory.organization_id,
                db_memory.user_id,
                db_memory.agent_id,
                db_memory.content,
                db_memory.hash,
                metadata_json,
                db_memory.score,
                db_memory.memory_type,
                db_memory.scope,
                db_memory.level,
                db_memory.importance,
                db_memory.access_count,
                db_memory.last_accessed.map(|dt| dt.timestamp()),
                db_memory.created_at.timestamp(),
                db_memory.updated_at.timestamp(),
                if db_memory.is_deleted { 1 } else { 0 },
                db_memory.created_by_id,
                db_memory.last_updated_by_id,
            ],
        )
        .await
        .map_err(|e| AgentMemError::StorageError(format!("Failed to create memory: {e}")))?;

        Ok(memory.clone())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<Memory>> {
        let conn = self.conn.lock().await;

        let mut stmt = conn
            .prepare(
                "SELECT id, organization_id, user_id, agent_id, content, hash, metadata,
                        score, memory_type, scope, level, importance, access_count, last_accessed,
                        created_at, updated_at, is_deleted, created_by_id, last_updated_by_id
                 FROM memories WHERE id = ? AND is_deleted = 0",
            )
            .await
            .map_err(|e| {
                AgentMemError::StorageError(format!("Failed to prepare statement: {e}"))
            })?;

        let mut rows = stmt
            .query(libsql::params![id])
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to query memory: {e}")))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to fetch row: {e}")))?
        {
            let db_memory = Self::row_to_memory(&row)?;
            let memory = db_to_memory(&db_memory)?;
            Ok(Some(memory))
        } else {
            Ok(None)
        }
    }

    async fn find_by_agent_id(&self, agent_id: &str, limit: i64) -> Result<Vec<Memory>> {
        let conn = self.conn.lock().await;

        let mut stmt = conn
            .prepare(
                "SELECT id, organization_id, user_id, agent_id, content, hash, metadata,
                        score, memory_type, scope, level, importance, access_count, last_accessed,
                        created_at, updated_at, is_deleted, created_by_id, last_updated_by_id
                 FROM memories WHERE agent_id = ? AND is_deleted = 0 
                 ORDER BY created_at DESC LIMIT ?",
            )
            .await
            .map_err(|e| {
                AgentMemError::StorageError(format!("Failed to prepare statement: {e}"))
            })?;

        let mut rows = stmt
            .query(libsql::params![agent_id, limit])
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to query memories: {e}")))?;

        let mut db_memories = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to fetch row: {e}")))?
        {
            db_memories.push(Self::row_to_memory(&row)?);
        }

        // Convert DbMemory to Memory V4
        let memories: Result<Vec<Memory>> = db_memories.iter().map(|db| db_to_memory(db)).collect();
        memories
    }

    async fn find_by_user_id(&self, user_id: &str, limit: i64) -> Result<Vec<Memory>> {
        let conn = self.conn.lock().await;

        let mut stmt = conn
            .prepare(
                "SELECT id, organization_id, user_id, agent_id, content, hash, metadata,
                        score, memory_type, scope, level, importance, access_count, last_accessed,
                        created_at, updated_at, is_deleted, created_by_id, last_updated_by_id
                 FROM memories WHERE user_id = ? AND is_deleted = 0 
                 ORDER BY created_at DESC LIMIT ?",
            )
            .await
            .map_err(|e| {
                AgentMemError::StorageError(format!("Failed to prepare statement: {e}"))
            })?;

        let mut rows = stmt
            .query(libsql::params![user_id, limit])
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to query memories: {e}")))?;

        let mut db_memories = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to fetch row: {e}")))?
        {
            db_memories.push(Self::row_to_memory(&row)?);
        }

        // Convert DbMemory to Memory V4
        let memories: Result<Vec<Memory>> = db_memories.iter().map(|db| db_to_memory(db)).collect();
        memories
    }

    async fn search(&self, query: &str, limit: i64) -> Result<Vec<Memory>> {
        let conn = self.conn.lock().await;

        let search_pattern = format!("%{query}%");

        let mut stmt = conn
            .prepare(
                "SELECT id, organization_id, user_id, agent_id, content, hash, metadata,
                        score, memory_type, scope, level, importance, access_count, last_accessed,
                        created_at, updated_at, is_deleted, created_by_id, last_updated_by_id
                 FROM memories WHERE content LIKE ? AND is_deleted = 0 
                 ORDER BY importance DESC, created_at DESC LIMIT ?",
            )
            .await
            .map_err(|e| {
                AgentMemError::StorageError(format!("Failed to prepare statement: {e}"))
            })?;

        let mut rows = stmt
            .query(libsql::params![search_pattern, limit])
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to search memories: {e}")))?;

        let mut db_memories = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to fetch row: {e}")))?
        {
            db_memories.push(Self::row_to_memory(&row)?);
        }

        // Convert DbMemory to Memory V4
        let memories: Result<Vec<Memory>> = db_memories.iter().map(|db| db_to_memory(db)).collect();
        memories
    }

    /// 使用元数据过滤搜索记忆（阶段2：高级过滤）
    ///
    /// 支持高级操作符和逻辑操作符的元数据过滤（LibSQL版本）
    /// 注意：这是一个辅助方法，不在trait中定义
    pub async fn search_with_metadata_filters(
        &self,
        agent_id: &str,
        query: &str,
        filters: &LogicalOperator,
        limit: i64,
    ) -> Result<Vec<Memory>> {
        // 简化实现：先获取所有结果，然后在内存中过滤
        // TODO: 优化为SQL级别的过滤
        let conn = self.conn.lock().await;

        let search_pattern = if query.is_empty() {
            String::new()
        } else {
            format!("%{}%", query)
        };

        let mut sql = String::from(
            "SELECT id, organization_id, user_id, agent_id, content, hash, metadata,
                    score, memory_type, scope, level, importance, access_count, last_accessed,
                    created_at, updated_at, is_deleted, created_by_id, last_updated_by_id
             FROM memories WHERE agent_id = ? AND is_deleted = 0",
        );

        if !query.is_empty() {
            sql.push_str(" AND content LIKE ?");
        }

        sql.push_str(" ORDER BY importance DESC, created_at DESC LIMIT ?");

        let mut stmt = conn
            .prepare(&sql)
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to prepare statement: {e}")))?;

        let mut rows = if query.is_empty() {
            stmt.query(libsql::params![agent_id, limit])
                .await
        } else {
            stmt.query(libsql::params![agent_id, search_pattern, limit])
                .await
        }
        .map_err(|e| AgentMemError::StorageError(format!("Failed to search memories: {e}")))?;

        let mut db_memories = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to fetch row: {e}")))?
        {
            db_memories.push(Self::row_to_memory(&row)?);
        }

        // Convert DbMemory to Memory V4
        let memories: Result<Vec<Memory>> = db_memories.iter().map(|db| db_to_memory(db)).collect();
        let mut memories = memories?;

        // 在内存中应用元数据过滤
        let mut filtered = Vec::new();
        for memory in memories {
            let metadata: std::collections::HashMap<String, serde_json::Value> = 
                memory.metadata().iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
            
            if MetadataFilterSystem::matches(filters, &metadata) {
                filtered.push(memory);
            }
        }

        Ok(filtered)
    }

    async fn update(&self, memory: &Memory) -> Result<Memory> {
        let conn = self.conn.lock().await;

        // Convert V4 Memory to DbMemory
        let db_memory = memory_to_db(memory);

        let metadata_json = serde_json::to_string(&db_memory.metadata).map_err(|e| {
            AgentMemError::StorageError(format!("Failed to serialize metadata: {e}"))
        })?;

        conn.execute(
            "UPDATE memories SET
                organization_id = ?, user_id = ?, agent_id = ?, content = ?, hash = ?,
                metadata = ?, score = ?, memory_type = ?, scope = ?, level = ?,
                importance = ?, access_count = ?, last_accessed = ?, updated_at = ?,
                last_updated_by_id = ?
             WHERE id = ? AND is_deleted = 0",
            libsql::params![
                db_memory.organization_id,
                db_memory.user_id,
                db_memory.agent_id,
                db_memory.content,
                db_memory.hash,
                metadata_json,
                db_memory.score,
                db_memory.memory_type,
                db_memory.scope,
                db_memory.level,
                db_memory.importance,
                db_memory.access_count,
                db_memory.last_accessed.map(|dt| dt.timestamp()),
                db_memory.updated_at.timestamp(),
                db_memory.last_updated_by_id,
                db_memory.id,
            ],
        )
        .await
        .map_err(|e| AgentMemError::StorageError(format!("Failed to update memory: {e}")))?;

        Ok(memory.clone())
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().await;

        conn.execute(
            "UPDATE memories SET is_deleted = 1, updated_at = ? WHERE id = ?",
            libsql::params![Utc::now().timestamp(), id],
        )
        .await
        .map_err(|e| AgentMemError::StorageError(format!("Failed to delete memory: {e}")))?;

        Ok(())
    }

    async fn delete_by_agent_id(&self, agent_id: &str) -> Result<u64> {
        let conn = self.conn.lock().await;

        let result = conn.execute(
            "UPDATE memories SET is_deleted = 1, updated_at = ? WHERE agent_id = ? AND is_deleted = 0",
            libsql::params![Utc::now().timestamp(), agent_id],
        )
        .await
        .map_err(|e| AgentMemError::StorageError(format!("Failed to delete memories by agent_id: {e}")))?;

        Ok(result as u64)
    }

    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Memory>> {
        let conn = self.conn.lock().await;

        let mut stmt = conn
            .prepare(
                "SELECT id, organization_id, user_id, agent_id, content, hash, metadata,
                        score, memory_type, scope, level, importance, access_count, last_accessed,
                        created_at, updated_at, is_deleted, created_by_id, last_updated_by_id
                 FROM memories WHERE is_deleted = 0 
                 ORDER BY created_at DESC LIMIT ? OFFSET ?",
            )
            .await
            .map_err(|e| {
                AgentMemError::StorageError(format!("Failed to prepare statement: {e}"))
            })?;

        let mut rows = stmt
            .query(libsql::params![limit, offset])
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to list memories: {e}")))?;

        let mut db_memories = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to fetch row: {e}")))?
        {
            db_memories.push(Self::row_to_memory(&row)?);
        }

        // Convert DbMemory to Memory V4
        let memories: Result<Vec<Memory>> = db_memories.iter().map(|db| db_to_memory(db)).collect();
        memories
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde_json::json;

    async fn setup_test_db() -> Arc<Mutex<Connection>> {
        let db = libsql::Database::open(":memory:").expect("Failed to create in-memory database");
        let conn = db.connect().expect("Failed to connect to database");

        // Create memories table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS memories (
                id TEXT PRIMARY KEY,
                organization_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                agent_id TEXT NOT NULL,
                content TEXT NOT NULL,
                hash TEXT,
                metadata TEXT,
                score REAL,
                memory_type TEXT NOT NULL,
                scope TEXT NOT NULL,
                level TEXT NOT NULL,
                importance REAL NOT NULL,
                access_count INTEGER NOT NULL DEFAULT 0,
                last_accessed INTEGER,
                embedding TEXT,
                expires_at INTEGER,
                version INTEGER NOT NULL DEFAULT 1,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                is_deleted INTEGER NOT NULL DEFAULT 0,
                created_by_id TEXT,
                last_updated_by_id TEXT
            )",
            (),
        )
        .await
        .expect("Failed to create memories table");

        Arc::new(Mutex::new(conn))
    }

    fn create_test_memory(id: &str) -> Memory {
        use agent_mem_traits::{
            MemoryId, Content, AttributeKey, AttributeValue, AttributeSet, 
            MetadataV4, RelationGraph
        };
        
        let mut attributes = AttributeSet::new();
        attributes.insert(AttributeKey::core("organization_id"), AttributeValue::String("org1".to_string()));
        attributes.insert(AttributeKey::core("user_id"), AttributeValue::String("user1".to_string()));
        attributes.insert(AttributeKey::core("agent_id"), AttributeValue::String("agent1".to_string()));
        attributes.insert(AttributeKey::core("memory_type"), AttributeValue::String("episodic".to_string()));
        attributes.insert(AttributeKey::core("scope"), AttributeValue::String("session".to_string()));
        attributes.insert(AttributeKey::core("level"), AttributeValue::String("high".to_string()));
        attributes.insert(AttributeKey::system("hash"), AttributeValue::String("hash123".to_string()));
        attributes.insert(AttributeKey::system("score"), AttributeValue::Number(0.95));
        attributes.insert(AttributeKey::system("importance"), AttributeValue::Number(0.8));
        attributes.insert(AttributeKey::system("created_by_id"), AttributeValue::String("user1".to_string()));
        
        Memory {
            id: MemoryId::from_string(id.to_string()),
            content: Content::Text(format!("Test memory content {id}")),
            attributes,
            relations: RelationGraph::new(),
            metadata: MetadataV4 {
            created_at: Utc::now(),
            updated_at: Utc::now(),
                accessed_at: Utc::now(),
                access_count: 0,
                version: 1,
                hash: None,
            },
        }
    }

    #[tokio::test]
    async fn test_create_memory() {
        let conn = setup_test_db().await;
        let repo = LibSqlMemoryRepository::new(conn);

        let memory = create_test_memory("mem1");
        let result = repo.create(&memory).await;

        assert!(result.is_ok());
        let created = result.unwrap();
        assert_eq!(created.id.as_str(), "mem1");
        if let agent_mem_traits::Content::Text(text) = &created.content {
            assert_eq!(text, "Test memory content mem1");
        } else {
            panic!("Expected text content");
        }
    }

    #[tokio::test]
    async fn test_find_by_id() {
        let conn = setup_test_db().await;
        let repo = LibSqlMemoryRepository::new(conn);

        let memory = create_test_memory("mem2");
        repo.create(&memory).await.unwrap();

        let result = repo.find_by_id("mem2").await;
        assert!(result.is_ok());
        let found = result.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id.as_str(), "mem2");
    }

    #[tokio::test]
    async fn test_find_by_agent_id() {
        let conn = setup_test_db().await;
        let repo = LibSqlMemoryRepository::new(conn);

        let memory1 = create_test_memory("mem3");
        let memory2 = create_test_memory("mem4");
        repo.create(&memory1).await.unwrap();
        repo.create(&memory2).await.unwrap();

        let result = repo.find_by_agent_id("agent1", 10).await;
        assert!(result.is_ok());
        let memories = result.unwrap();
        assert_eq!(memories.len(), 2);
    }

    #[tokio::test]
    async fn test_find_by_user_id() {
        let conn = setup_test_db().await;
        let repo = LibSqlMemoryRepository::new(conn);

        let memory = create_test_memory("mem5");
        repo.create(&memory).await.unwrap();

        let result = repo.find_by_user_id("user1", 10).await;
        assert!(result.is_ok());
        let memories = result.unwrap();
        assert_eq!(memories.len(), 1);
        assert_eq!(memories[0].id.as_str(), "mem5");
    }

    #[tokio::test]
    async fn test_search() {
        let conn = setup_test_db().await;
        let repo = LibSqlMemoryRepository::new(conn);

        let memory = create_test_memory("mem6");
        repo.create(&memory).await.unwrap();

        let result = repo.search("Test memory", 10).await;
        assert!(result.is_ok());
        let memories = result.unwrap();
        assert_eq!(memories.len(), 1);
    }

    #[tokio::test]
    async fn test_update() {
        let conn = setup_test_db().await;
        let repo = LibSqlMemoryRepository::new(conn);

        let mut memory = create_test_memory("mem7");
        repo.create(&memory).await.unwrap();

        memory.content = agent_mem_traits::Content::Text("Updated content".to_string());
        memory.set_importance(0.9);
        let result = repo.update(&memory).await;

        assert!(result.is_ok());
        let updated = repo.find_by_id("mem7").await.unwrap().unwrap();
        if let agent_mem_traits::Content::Text(text) = &updated.content {
            assert_eq!(text, "Updated content");
        } else {
            panic!("Expected text content");
        }
        assert!((updated.importance().unwrap() - 0.9).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_delete() {
        let conn = setup_test_db().await;
        let repo = LibSqlMemoryRepository::new(conn);

        let memory = create_test_memory("mem8");
        repo.create(&memory).await.unwrap();

        let result = repo.delete("mem8").await;
        assert!(result.is_ok());

        let found = repo.find_by_id("mem8").await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_delete_by_agent_id() {
        let conn = setup_test_db().await;
        let repo = LibSqlMemoryRepository::new(conn);

        let memory1 = create_test_memory("mem9");
        let memory2 = create_test_memory("mem10");
        repo.create(&memory1).await.unwrap();
        repo.create(&memory2).await.unwrap();

        let result = repo.delete_by_agent_id("agent1").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);

        let memories = repo.find_by_agent_id("agent1", 10).await.unwrap();
        assert_eq!(memories.len(), 0);
    }

    #[tokio::test]
    async fn test_list() {
        let conn = setup_test_db().await;
        let repo = LibSqlMemoryRepository::new(conn);

        let memory1 = create_test_memory("mem11");
        let memory2 = create_test_memory("mem12");
        repo.create(&memory1).await.unwrap();
        repo.create(&memory2).await.unwrap();

        let result = repo.list(10, 0).await;
        assert!(result.is_ok());
        let memories = result.unwrap();
        assert_eq!(memories.len(), 2);
    }
}
