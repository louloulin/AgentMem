//! LibSQL Memory Repository
//!
//! Provides LibSQL implementation of MemoryRepositoryTrait

use agent_mem_traits::{AgentMemError, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use libsql::Connection;
use serde_json::Value as JsonValue;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::storage::models::Memory;
use crate::storage::traits::MemoryRepositoryTrait;

/// LibSQL implementation of Memory repository
pub struct LibSqlMemoryRepository {
    conn: Arc<Mutex<Connection>>,
}

impl LibSqlMemoryRepository {
    /// Create a new LibSQL memory repository
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Helper function to convert row to Memory
    fn row_to_memory(row: &libsql::Row) -> Result<Memory> {
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

        let created_at_ts: i64 = row.get(14).map_err(|e| {
            AgentMemError::StorageError(format!("Failed to get created_at: {e}"))
        })?;
        let created_at = DateTime::from_timestamp(created_at_ts, 0)
            .ok_or_else(|| AgentMemError::StorageError("Invalid created_at timestamp".to_string()))?;

        let updated_at_ts: i64 = row.get(15).map_err(|e| {
            AgentMemError::StorageError(format!("Failed to get updated_at: {e}"))
        })?;
        let updated_at = DateTime::from_timestamp(updated_at_ts, 0)
            .ok_or_else(|| AgentMemError::StorageError("Invalid updated_at timestamp".to_string()))?;

        let is_deleted_int: i64 = row.get(16).unwrap_or(0);

        let score_f64: Option<f64> = row.get(7).ok();
        let importance_f64: f64 = row.get(11).map_err(|e| {
            AgentMemError::StorageError(format!("Failed to get importance: {e}"))
        })?;

        Ok(Memory {
            id: row.get(0).map_err(|e| AgentMemError::StorageError(format!("Failed to get id: {e}")))?,
            organization_id: row.get(1).map_err(|e| AgentMemError::StorageError(format!("Failed to get organization_id: {e}")))?,
            user_id: row.get(2).map_err(|e| AgentMemError::StorageError(format!("Failed to get user_id: {e}")))?,
            agent_id: row.get(3).map_err(|e| AgentMemError::StorageError(format!("Failed to get agent_id: {e}")))?,
            content: row.get(4).map_err(|e| AgentMemError::StorageError(format!("Failed to get content: {e}")))?,
            hash: row.get(5).ok(),
            metadata,
            score: score_f64.map(|v| v as f32),
            memory_type: row.get(8).map_err(|e| AgentMemError::StorageError(format!("Failed to get memory_type: {e}")))?,
            scope: row.get(9).map_err(|e| AgentMemError::StorageError(format!("Failed to get scope: {e}")))?,
            level: row.get(10).map_err(|e| AgentMemError::StorageError(format!("Failed to get level: {e}")))?,
            importance: importance_f64 as f32,
            access_count: row.get(12).map_err(|e| AgentMemError::StorageError(format!("Failed to get access_count: {e}")))?,
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

        let metadata_json = serde_json::to_string(&memory.metadata)
            .map_err(|e| AgentMemError::StorageError(format!("Failed to serialize metadata: {e}")))?;

        conn.execute(
            "INSERT INTO memories (
                id, organization_id, user_id, agent_id, content, hash, metadata,
                score, memory_type, scope, level, importance, access_count, last_accessed,
                created_at, updated_at, is_deleted, created_by_id, last_updated_by_id
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            libsql::params![
                memory.id.clone(),
                memory.organization_id.clone(),
                memory.user_id.clone(),
                memory.agent_id.clone(),
                memory.content.clone(),
                memory.hash.clone(),
                metadata_json,
                memory.score,
                memory.memory_type.clone(),
                memory.scope.clone(),
                memory.level.clone(),
                memory.importance,
                memory.access_count,
                memory.last_accessed.map(|dt| dt.timestamp()),
                memory.created_at.timestamp(),
                memory.updated_at.timestamp(),
                if memory.is_deleted { 1 } else { 0 },
                memory.created_by_id.clone(),
                memory.last_updated_by_id.clone(),
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
                 FROM memories WHERE id = ? AND is_deleted = 0"
            )
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to prepare statement: {e}")))?;

        let mut rows = stmt
            .query(libsql::params![id])
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to query memory: {e}")))?;

        if let Some(row) = rows.next().await.map_err(|e| {
            AgentMemError::StorageError(format!("Failed to fetch row: {e}"))
        })? {
            Ok(Some(Self::row_to_memory(&row)?))
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
                 ORDER BY created_at DESC LIMIT ?"
            )
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to prepare statement: {e}")))?;

        let mut rows = stmt
            .query(libsql::params![agent_id, limit])
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to query memories: {e}")))?;

        let mut memories = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| {
            AgentMemError::StorageError(format!("Failed to fetch row: {e}"))
        })? {
            memories.push(Self::row_to_memory(&row)?);
        }

        Ok(memories)
    }

    async fn find_by_user_id(&self, user_id: &str, limit: i64) -> Result<Vec<Memory>> {
        let conn = self.conn.lock().await;

        let mut stmt = conn
            .prepare(
                "SELECT id, organization_id, user_id, agent_id, content, hash, metadata,
                        score, memory_type, scope, level, importance, access_count, last_accessed,
                        created_at, updated_at, is_deleted, created_by_id, last_updated_by_id
                 FROM memories WHERE user_id = ? AND is_deleted = 0 
                 ORDER BY created_at DESC LIMIT ?"
            )
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to prepare statement: {e}")))?;

        let mut rows = stmt
            .query(libsql::params![user_id, limit])
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to query memories: {e}")))?;

        let mut memories = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| {
            AgentMemError::StorageError(format!("Failed to fetch row: {e}"))
        })? {
            memories.push(Self::row_to_memory(&row)?);
        }

        Ok(memories)
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
                 ORDER BY importance DESC, created_at DESC LIMIT ?"
            )
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to prepare statement: {e}")))?;

        let mut rows = stmt
            .query(libsql::params![search_pattern, limit])
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to search memories: {e}")))?;

        let mut memories = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| {
            AgentMemError::StorageError(format!("Failed to fetch row: {e}"))
        })? {
            memories.push(Self::row_to_memory(&row)?);
        }

        Ok(memories)
    }

    async fn update(&self, memory: &Memory) -> Result<Memory> {
        let conn = self.conn.lock().await;

        let metadata_json = serde_json::to_string(&memory.metadata)
            .map_err(|e| AgentMemError::StorageError(format!("Failed to serialize metadata: {e}")))?;

        conn.execute(
            "UPDATE memories SET 
                organization_id = ?, user_id = ?, agent_id = ?, content = ?, hash = ?,
                metadata = ?, score = ?, memory_type = ?, scope = ?, level = ?,
                importance = ?, access_count = ?, last_accessed = ?, updated_at = ?,
                last_updated_by_id = ?
             WHERE id = ? AND is_deleted = 0",
            libsql::params![
                memory.organization_id.clone(),
                memory.user_id.clone(),
                memory.agent_id.clone(),
                memory.content.clone(),
                memory.hash.clone(),
                metadata_json,
                memory.score,
                memory.memory_type.clone(),
                memory.scope.clone(),
                memory.level.clone(),
                memory.importance,
                memory.access_count,
                memory.last_accessed.map(|dt| dt.timestamp()),
                memory.updated_at.timestamp(),
                memory.last_updated_by_id.clone(),
                memory.id.clone(),
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
                 ORDER BY created_at DESC LIMIT ? OFFSET ?"
            )
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to prepare statement: {e}")))?;

        let mut rows = stmt
            .query(libsql::params![limit, offset])
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to list memories: {e}")))?;

        let mut memories = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| {
            AgentMemError::StorageError(format!("Failed to fetch row: {e}"))
        })? {
            memories.push(Self::row_to_memory(&row)?);
        }

        Ok(memories)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde_json::json;

    async fn setup_test_db() -> Arc<Mutex<Connection>> {
        let db = libsql::Database::open(":memory:")
            .expect("Failed to create in-memory database");
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
        Memory {
            id: id.to_string(),
            organization_id: "org1".to_string(),
            user_id: "user1".to_string(),
            agent_id: "agent1".to_string(),
            content: format!("Test memory content {}", id),
            hash: Some("hash123".to_string()),
            metadata: json!({"key": "value"}),
            score: Some(0.95),
            memory_type: "episodic".to_string(),
            scope: "session".to_string(),
            level: "high".to_string(),
            importance: 0.8,
            access_count: 0,
            last_accessed: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            is_deleted: false,
            created_by_id: Some("user1".to_string()),
            last_updated_by_id: None,
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
        assert_eq!(created.id, "mem1");
        assert_eq!(created.content, "Test memory content mem1");
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
        assert_eq!(found.unwrap().id, "mem2");
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
        assert_eq!(memories[0].id, "mem5");
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

        memory.content = "Updated content".to_string();
        memory.importance = 0.9;
        let result = repo.update(&memory).await;

        assert!(result.is_ok());
        let updated = repo.find_by_id("mem7").await.unwrap().unwrap();
        assert_eq!(updated.content, "Updated content");
        assert_eq!(updated.importance, 0.9);
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

