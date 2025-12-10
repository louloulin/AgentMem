//! LibSQL implementation of ToolRepositoryTrait

use crate::storage::models::Tool;
use crate::storage::traits::ToolRepositoryTrait;
use agent_mem_traits::{AgentMemError, Result};
use async_trait::async_trait;
use chrono::Utc;
use libsql::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::storage::libsql::connection::LibSqlConnectionPool;

/// LibSQL implementation of Tool repository
pub struct LibSqlToolRepository {
    /// Legacy single-connection mode (Arc<Mutex<Connection>>)
    conn: Option<Arc<Mutex<Connection>>>,
    /// Preferred pooled mode
    pool: Option<Arc<LibSqlConnectionPool>>,
}

impl LibSqlToolRepository {
    /// Create a new LibSQL tool repository
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self {
            conn: Some(conn),
            pool: None,
        }
    }

    /// Create a new LibSQL repository backed by a connection pool
    pub fn new_with_pool(pool: Arc<LibSqlConnectionPool>) -> Self {
        Self {
            conn: None,
            pool: Some(pool),
        }
    }

    /// Helper to get a connection (from pool if available, otherwise the single conn)
    async fn get_conn(&self) -> Result<Arc<Mutex<Connection>>> {
        if let Some(pool) = &self.pool {
            return pool
                .get()
                .await
                .map_err(|e| AgentMemError::StorageError(format!("Failed to get pooled conn: {e}")));
        }
        if let Some(conn) = &self.conn {
            return Ok(conn.clone());
        }
        Err(AgentMemError::StorageError(
            "No connection or pool available".to_string(),
        ))
    }

    /// Helper function to serialize JSON fields
    fn serialize_json(value: &Option<serde_json::Value>) -> Option<String> {
        value
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap_or_default())
    }

    /// Helper function to deserialize JSON fields
    fn deserialize_json(value: Option<String>) -> Option<serde_json::Value> {
        value.and_then(|s| serde_json::from_str(&s).ok())
    }

    /// Helper function to serialize tags
    fn serialize_tags(tags: &Option<Vec<String>>) -> Option<String> {
        tags.as_ref()
            .map(|t| serde_json::to_string(t).unwrap_or_default())
    }

    /// Helper function to deserialize tags
    fn deserialize_tags(value: Option<String>) -> Option<Vec<String>> {
        value.and_then(|s| serde_json::from_str(&s).ok())
    }
}

#[async_trait]
impl ToolRepositoryTrait for LibSqlToolRepository {
    async fn create(&self, tool: &Tool) -> Result<Tool> {
        let conn = self.get_conn().await?;
        let conn = conn.lock().await;

        conn.execute(
            "INSERT INTO tools (
                id, organization_id, name, description, json_schema,
                source_type, source_code, tags, metadata,
                created_at, updated_at, is_deleted, created_by_id, last_updated_by_id
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            libsql::params![
                tool.id.clone(),
                tool.organization_id.clone(),
                tool.name.clone(),
                tool.description.clone(),
                Self::serialize_json(&tool.json_schema),
                tool.source_type.clone(),
                tool.source_code.clone(),
                Self::serialize_tags(&tool.tags),
                Self::serialize_json(&tool.metadata_),
                tool.created_at.timestamp(),
                tool.updated_at.timestamp(),
                if tool.is_deleted { 1 } else { 0 },
                tool.created_by_id.clone(),
                tool.last_updated_by_id.clone(),
            ],
        )
        .await
        .map_err(|e| AgentMemError::StorageError(format!("Failed to create tool: {e}")))?;

        Ok(tool.clone())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<Tool>> {
        let conn = self.get_conn().await?;
        let conn = conn.lock().await;

        let query = "SELECT 
            id, organization_id, name, description, json_schema, 
            source_type, source_code, tags, metadata, 
            created_at, updated_at, is_deleted, created_by_id, last_updated_by_id
        FROM tools WHERE id = ? AND is_deleted = 0";

        let mut stmt = conn
            .prepare(query)
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to prepare query: {e}")))?;

        let mut rows = stmt
            .query([id])
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to execute query: {e}")))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to fetch row: {e}")))?
        {
            let created_at_ts: i64 = row.get(9).unwrap();
            let updated_at_ts: i64 = row.get(10).unwrap();
            let is_deleted_int: i64 = row.get(11).unwrap();

            let tool = Tool {
                id: row.get(0).unwrap(),
                organization_id: row.get(1).unwrap(),
                name: row.get(2).unwrap(),
                description: row.get(3).unwrap(),
                json_schema: Self::deserialize_json(row.get(4).unwrap()),
                source_type: row.get(5).unwrap(),
                source_code: row.get(6).unwrap(),
                tags: Self::deserialize_tags(row.get(7).unwrap()),
                metadata_: Self::deserialize_json(row.get(8).unwrap()),
                created_at: chrono::DateTime::from_timestamp(created_at_ts, 0)
                    .unwrap_or_else(Utc::now),
                updated_at: chrono::DateTime::from_timestamp(updated_at_ts, 0)
                    .unwrap_or_else(Utc::now),
                is_deleted: is_deleted_int != 0,
                created_by_id: row.get(12).unwrap(),
                last_updated_by_id: row.get(13).unwrap(),
            };

            Ok(Some(tool))
        } else {
            Ok(None)
        }
    }

    async fn find_by_organization_id(&self, org_id: &str) -> Result<Vec<Tool>> {
        let conn = self.get_conn().await?;
        let conn = conn.lock().await;

        let query = "SELECT
            id, organization_id, name, description, json_schema,
            source_type, source_code, tags, metadata,
            created_at, updated_at, is_deleted, created_by_id, last_updated_by_id
        FROM tools WHERE organization_id = ? AND is_deleted = 0 ORDER BY created_at DESC";

        let mut stmt = conn
            .prepare(query)
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to prepare query: {e}")))?;

        let mut rows = stmt
            .query([org_id])
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to execute query: {e}")))?;

        let mut tools = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to fetch row: {e}")))?
        {
            let created_at_ts: i64 = row.get(9).unwrap();
            let updated_at_ts: i64 = row.get(10).unwrap();
            let is_deleted_int: i64 = row.get(11).unwrap();

            let tool = Tool {
                id: row.get(0).unwrap(),
                organization_id: row.get(1).unwrap(),
                name: row.get(2).unwrap(),
                description: row.get(3).unwrap(),
                json_schema: Self::deserialize_json(row.get(4).unwrap()),
                source_type: row.get(5).unwrap(),
                source_code: row.get(6).unwrap(),
                tags: Self::deserialize_tags(row.get(7).unwrap()),
                metadata_: Self::deserialize_json(row.get(8).unwrap()),
                created_at: chrono::DateTime::from_timestamp(created_at_ts, 0)
                    .unwrap_or_else(Utc::now),
                updated_at: chrono::DateTime::from_timestamp(updated_at_ts, 0)
                    .unwrap_or_else(Utc::now),
                is_deleted: is_deleted_int != 0,
                created_by_id: row.get(12).unwrap(),
                last_updated_by_id: row.get(13).unwrap(),
            };

            tools.push(tool);
        }

        Ok(tools)
    }

    async fn find_by_tags(&self, org_id: &str, tags: &[String]) -> Result<Vec<Tool>> {
        if tags.is_empty() {
            return self.find_by_organization_id(org_id).await;
        }

        let conn = self.get_conn().await?;
        let conn = conn.lock().await;

        // Build query to find tools that have any of the specified tags
        // Tags are stored as JSON array, so we need to check if any tag matches
        let query = "SELECT
            id, organization_id, name, description, json_schema,
            source_type, source_code, tags, metadata,
            created_at, updated_at, is_deleted, created_by_id, last_updated_by_id
        FROM tools WHERE organization_id = ? AND is_deleted = 0 ORDER BY created_at DESC";

        let mut stmt = conn
            .prepare(query)
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to prepare query: {e}")))?;

        let mut rows = stmt
            .query([org_id])
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to execute query: {e}")))?;

        let mut tools = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to fetch row: {e}")))?
        {
            let created_at_ts: i64 = row.get(9).unwrap();
            let updated_at_ts: i64 = row.get(10).unwrap();
            let is_deleted_int: i64 = row.get(11).unwrap();
            let tool_tags = Self::deserialize_tags(row.get(7).unwrap());

            // Check if tool has any of the requested tags
            if let Some(ref tag_list) = tool_tags {
                let has_matching_tag = tags.iter().any(|tag| tag_list.contains(tag));
                if !has_matching_tag {
                    continue;
                }
            } else {
                // No tags, skip this tool
                continue;
            }

            let tool = Tool {
                id: row.get(0).unwrap(),
                organization_id: row.get(1).unwrap(),
                name: row.get(2).unwrap(),
                description: row.get(3).unwrap(),
                json_schema: Self::deserialize_json(row.get(4).unwrap()),
                source_type: row.get(5).unwrap(),
                source_code: row.get(6).unwrap(),
                tags: tool_tags,
                metadata_: Self::deserialize_json(row.get(8).unwrap()),
                created_at: chrono::DateTime::from_timestamp(created_at_ts, 0)
                    .unwrap_or_else(Utc::now),
                updated_at: chrono::DateTime::from_timestamp(updated_at_ts, 0)
                    .unwrap_or_else(Utc::now),
                is_deleted: is_deleted_int != 0,
                created_by_id: row.get(12).unwrap(),
                last_updated_by_id: row.get(13).unwrap(),
            };

            tools.push(tool);
        }

        Ok(tools)
    }

    async fn update(&self, tool: &Tool) -> Result<Tool> {
        let tool_id = tool.id.clone();

        {
            let conn = self.get_conn().await?;
        let conn = conn.lock().await;

            let rows_affected = conn
                .execute(
                    "UPDATE tools SET
                        organization_id = ?, name = ?, description = ?, json_schema = ?,
                        source_type = ?, source_code = ?, tags = ?, metadata = ?,
                        updated_at = ?, last_updated_by_id = ?
                    WHERE id = ? AND is_deleted = 0",
                    libsql::params![
                        tool.organization_id.clone(),
                        tool.name.clone(),
                        tool.description.clone(),
                        Self::serialize_json(&tool.json_schema),
                        tool.source_type.clone(),
                        tool.source_code.clone(),
                        Self::serialize_tags(&tool.tags),
                        Self::serialize_json(&tool.metadata_),
                        Utc::now().timestamp(),
                        tool.last_updated_by_id.clone(),
                        tool.id.clone(),
                    ],
                )
                .await
                .map_err(|e| AgentMemError::StorageError(format!("Failed to update tool: {e}")))?;

            if rows_affected == 0 {
                return Err(AgentMemError::NotFound(format!(
                    "Tool with id {} not found",
                    tool.id
                )));
            }
        } // Release lock here

        // Fetch and return the updated tool
        self.find_by_id(&tool_id)
            .await?
            .ok_or_else(|| AgentMemError::NotFound(format!("Tool with id {tool_id} not found")))
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let conn = self.get_conn().await?;
        let conn = conn.lock().await;

        let rows_affected = conn
            .execute(
                "UPDATE tools SET is_deleted = 1, updated_at = ? WHERE id = ?",
                libsql::params![Utc::now().timestamp(), id.to_string()],
            )
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to delete tool: {e}")))?;

        if rows_affected == 0 {
            return Err(AgentMemError::NotFound(format!(
                "Tool with id {id} not found"
            )));
        }

        Ok(())
    }

    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Tool>> {
        let conn = self.get_conn().await?;
        let conn = conn.lock().await;

        let query = "SELECT 
            id, organization_id, name, description, json_schema, 
            source_type, source_code, tags, metadata, 
            created_at, updated_at, is_deleted, created_by_id, last_updated_by_id
        FROM tools WHERE is_deleted = 0 ORDER BY created_at DESC LIMIT ? OFFSET ?";

        let mut stmt = conn
            .prepare(query)
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to prepare query: {e}")))?;

        let mut rows = stmt
            .query((limit, offset))
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to execute query: {e}")))?;

        let mut tools = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to fetch row: {e}")))?
        {
            let created_at_ts: i64 = row.get(9).unwrap();
            let updated_at_ts: i64 = row.get(10).unwrap();
            let is_deleted_int: i64 = row.get(11).unwrap();

            let tool = Tool {
                id: row.get(0).unwrap(),
                organization_id: row.get(1).unwrap(),
                name: row.get(2).unwrap(),
                description: row.get(3).unwrap(),
                json_schema: Self::deserialize_json(row.get(4).unwrap()),
                source_type: row.get(5).unwrap(),
                source_code: row.get(6).unwrap(),
                tags: Self::deserialize_tags(row.get(7).unwrap()),
                metadata_: Self::deserialize_json(row.get(8).unwrap()),
                created_at: chrono::DateTime::from_timestamp(created_at_ts, 0)
                    .unwrap_or_else(Utc::now),
                updated_at: chrono::DateTime::from_timestamp(updated_at_ts, 0)
                    .unwrap_or_else(Utc::now),
                is_deleted: is_deleted_int != 0,
                created_by_id: row.get(12).unwrap(),
                last_updated_by_id: row.get(13).unwrap(),
            };

            tools.push(tool);
        }

        Ok(tools)
    }
}
