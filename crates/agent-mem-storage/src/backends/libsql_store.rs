//! LibSQL embedded database storage implementation
//!
//! Provides a zero-configuration, embedded SQL database for AgentMem.
//! LibSQL is a fork of SQLite with additional features like embedded replicas.

use agent_mem_traits::{AgentMemError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info, warn};

#[cfg(feature = "libsql")]
use libsql::{Builder, Connection, Database};

/// Memory record stored in LibSQL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub id: String,
    pub agent_id: String,
    pub user_id: Option<String>,
    pub content: String,
    pub memory_type: String,
    pub importance: f32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

/// LibSQL storage backend
#[cfg(feature = "libsql")]
pub struct LibSQLStore {
    db: Database,
    conn: Connection,
}

#[cfg(feature = "libsql")]
impl LibSQLStore {
    /// Create a new LibSQL store
    ///
    /// # Arguments
    /// * `path` - Path to the database file (use ":memory:" for in-memory)
    ///
    /// # Example
    /// ```no_run
    /// use agent_mem_storage::backends::libsql_store::LibSQLStore;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // File-based storage
    /// let store = LibSQLStore::new("~/.agentmem/data.db").await?;
    ///
    /// // In-memory storage (for testing)
    /// let store = LibSQLStore::new(":memory:").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(path: &str) -> Result<Self> {
        info!("Initializing LibSQL store at: {}", path);

        // Expand home directory
        let expanded_path = if path.starts_with("~/") {
            let home = std::env::var("HOME").map_err(|e| {
                AgentMemError::StorageError(format!("Failed to get HOME directory: {e}"))
            })?;
            path.replace("~", &home)
        } else {
            path.to_string()
        };

        // Create parent directory if needed
        if expanded_path != ":memory:" {
            if let Some(parent) = Path::new(&expanded_path).parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    AgentMemError::StorageError(format!("Failed to create directory: {e}"))
                })?;
            }
        }

        // Open database
        let db = Builder::new_local(&expanded_path)
            .build()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to open database: {e}")))?;

        let conn = db
            .connect()
            .map_err(|e| AgentMemError::StorageError(format!("Failed to connect to database: {e}")))?;

        let mut store = Self { db, conn };

        // Initialize schema
        store.init_schema().await?;

        info!("LibSQL store initialized successfully");
        Ok(store)
    }

    /// Initialize database schema
    async fn init_schema(&mut self) -> Result<()> {
        debug!("Initializing database schema");

        // Create memories table
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS memories (
                    id TEXT PRIMARY KEY,
                    agent_id TEXT NOT NULL,
                    user_id TEXT,
                    content TEXT NOT NULL,
                    memory_type TEXT NOT NULL,
                    importance REAL NOT NULL DEFAULT 0.5,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL,
                    metadata TEXT NOT NULL DEFAULT '{}'
                )",
                (),
            )
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to create memories table: {e}")))?;

        // Create indices
        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_memories_agent_id ON memories(agent_id)",
                (),
            )
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to create index: {e}")))?;

        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_memories_user_id ON memories(user_id)",
                (),
            )
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to create index: {e}")))?;

        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_memories_type ON memories(memory_type)",
                (),
            )
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to create index: {e}")))?;

        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_memories_created_at ON memories(created_at DESC)",
                (),
            )
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to create index: {e}")))?;

        debug!("Database schema initialized");
        Ok(())
    }

    /// Insert a memory record
    pub async fn insert(&self, record: &MemoryRecord) -> Result<()> {
        debug!("Inserting memory: {}", record.id);

        let metadata_json = serde_json::to_string(&record.metadata)
            .map_err(|e| AgentMemError::StorageError(format!("Failed to serialize metadata: {e}")))?;

        self.conn
            .execute(
                "INSERT INTO memories (id, agent_id, user_id, content, memory_type, importance, created_at, updated_at, metadata)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                libsql::params![
                    record.id.clone(),
                    record.agent_id.clone(),
                    record.user_id.clone(),
                    record.content.clone(),
                    record.memory_type.clone(),
                    record.importance,
                    record.created_at.timestamp(),
                    record.updated_at.timestamp(),
                    metadata_json.clone(),
                ],
            )
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to insert memory: {e}")))?;

        debug!("Memory inserted successfully");
        Ok(())
    }

    /// Get a memory by ID
    pub async fn get(&self, id: &str) -> Result<Option<MemoryRecord>> {
        debug!("Getting memory: {}", id);

        let mut rows = self
            .conn
            .query(
                "SELECT id, agent_id, user_id, content, memory_type, importance, created_at, updated_at, metadata
                 FROM memories WHERE id = ?",
                libsql::params![id],
            )
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to query memory: {e}")))?;

        if let Some(row) = rows.next().await.map_err(|e| AgentMemError::StorageError(format!("Failed to fetch row: {e}")))? {
            let record = self.row_to_record(row)?;
            Ok(Some(record))
        } else {
            Ok(None)
        }
    }

    /// Search memories
    pub async fn search(
        &self,
        agent_id: Option<&str>,
        user_id: Option<&str>,
        memory_type: Option<&str>,
        limit: usize,
    ) -> Result<Vec<MemoryRecord>> {
        debug!("Searching memories");

        let mut sql = "SELECT id, agent_id, user_id, content, memory_type, importance, created_at, updated_at, metadata FROM memories WHERE 1=1".to_string();
        let mut params = Vec::new();

        if let Some(aid) = agent_id {
            sql.push_str(" AND agent_id = ?");
            params.push(aid.to_string());
        }

        if let Some(uid) = user_id {
            sql.push_str(" AND user_id = ?");
            params.push(uid.to_string());
        }

        if let Some(mtype) = memory_type {
            sql.push_str(" AND memory_type = ?");
            params.push(mtype.to_string());
        }

        sql.push_str(" ORDER BY created_at DESC LIMIT ?");
        params.push(limit.to_string());

        let mut rows = self
            .conn
            .query(&sql, libsql::params_from_iter(params))
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to search memories: {e}")))?;

        let mut records = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| AgentMemError::StorageError(format!("Failed to fetch row: {e}")))? {
            records.push(self.row_to_record(row)?);
        }

        debug!("Found {} memories", records.len());
        Ok(records)
    }

    /// Update a memory
    pub async fn update(&self, record: &MemoryRecord) -> Result<()> {
        debug!("Updating memory: {}", record.id);

        let metadata_json = serde_json::to_string(&record.metadata)
            .map_err(|e| AgentMemError::StorageError(format!("Failed to serialize metadata: {e}")))?;

        self.conn
            .execute(
                "UPDATE memories SET content = ?, importance = ?, updated_at = ?, metadata = ? WHERE id = ?",
                libsql::params![
                    record.content.clone(),
                    record.importance,
                    record.updated_at.timestamp(),
                    metadata_json.clone(),
                    record.id.clone(),
                ],
            )
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to update memory: {e}")))?;

        debug!("Memory updated successfully");
        Ok(())
    }

    /// Delete a memory
    pub async fn delete(&self, id: &str) -> Result<bool> {
        debug!("Deleting memory: {}", id);

        let result = self
            .conn
            .execute("DELETE FROM memories WHERE id = ?", libsql::params![id])
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to delete memory: {e}")))?;

        let deleted = result > 0;
        debug!("Memory deleted: {}", deleted);
        Ok(deleted)
    }

    /// Count total memories
    pub async fn count(&self) -> Result<usize> {
        let mut rows = self
            .conn
            .query("SELECT COUNT(*) as count FROM memories", ())
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to count memories: {e}")))?;

        if let Some(row) = rows.next().await.map_err(|e| AgentMemError::StorageError(format!("Failed to fetch row: {e}")))? {
            let count: i64 = row.get(0).map_err(|e| AgentMemError::StorageError(format!("Failed to get count: {e}")))?;
            Ok(count as usize)
        } else {
            Ok(0)
        }
    }

    /// Clear all memories (for testing)
    pub async fn clear(&self) -> Result<()> {
        warn!("Clearing all memories");
        self.conn
            .execute("DELETE FROM memories", ())
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to clear memories: {e}")))?;
        Ok(())
    }

    /// Convert a row to a MemoryRecord
    fn row_to_record(&self, row: libsql::Row) -> Result<MemoryRecord> {
        let id: String = row.get(0).map_err(|e| AgentMemError::StorageError(format!("Failed to get id: {e}")))?;
        let agent_id: String = row.get(1).map_err(|e| AgentMemError::StorageError(format!("Failed to get agent_id: {e}")))?;
        let user_id: Option<String> = row.get(2).map_err(|e| AgentMemError::StorageError(format!("Failed to get user_id: {e}")))?;
        let content: String = row.get(3).map_err(|e| AgentMemError::StorageError(format!("Failed to get content: {e}")))?;
        let memory_type: String = row.get(4).map_err(|e| AgentMemError::StorageError(format!("Failed to get memory_type: {e}")))?;
        let importance: f64 = row.get(5).map_err(|e| AgentMemError::StorageError(format!("Failed to get importance: {e}")))?;
        let created_at: i64 = row.get(6).map_err(|e| AgentMemError::StorageError(format!("Failed to get created_at: {e}")))?;
        let updated_at: i64 = row.get(7).map_err(|e| AgentMemError::StorageError(format!("Failed to get updated_at: {e}")))?;
        let metadata_json: String = row.get(8).map_err(|e| AgentMemError::StorageError(format!("Failed to get metadata: {e}")))?;

        let metadata: HashMap<String, String> = serde_json::from_str(&metadata_json)
            .map_err(|e| AgentMemError::StorageError(format!("Failed to deserialize metadata: {e}")))?;

        Ok(MemoryRecord {
            id,
            agent_id,
            user_id,
            content,
            memory_type,
            importance: importance as f32,
            created_at: DateTime::from_timestamp(created_at, 0)
                .ok_or_else(|| AgentMemError::StorageError("Invalid created_at timestamp".to_string()))?,
            updated_at: DateTime::from_timestamp(updated_at, 0)
                .ok_or_else(|| AgentMemError::StorageError("Invalid updated_at timestamp".to_string()))?,
            metadata,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_libsql_create_and_get() {
        let store = LibSQLStore::new(":memory:").await.unwrap();

        let record = MemoryRecord {
            id: "test-1".to_string(),
            agent_id: "agent-1".to_string(),
            user_id: Some("user-1".to_string()),
            content: "Test memory".to_string(),
            memory_type: "episodic".to_string(),
            importance: 0.8,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: HashMap::new(),
        };

        store.insert(&record).await.unwrap();

        let retrieved = store.get("test-1").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().content, "Test memory");
    }

    #[tokio::test]
    async fn test_libsql_search() {
        let store = LibSQLStore::new(":memory:").await.unwrap();

        // Insert test records
        for i in 0..5 {
            let record = MemoryRecord {
                id: format!("test-{i}"),
                agent_id: "agent-1".to_string(),
                user_id: Some("user-1".to_string()),
                content: format!("Memory {i}"),
                memory_type: "episodic".to_string(),
                importance: 0.5,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                metadata: HashMap::new(),
            };
            store.insert(&record).await.unwrap();
        }

        let results = store.search(Some("agent-1"), None, None, 10).await.unwrap();
        assert_eq!(results.len(), 5);
    }
}

