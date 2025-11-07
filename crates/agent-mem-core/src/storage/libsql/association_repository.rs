//! LibSQL implementation of AssociationRepository

use crate::storage::traits::{AssociationRepositoryTrait, MemoryAssociation};
use agent_mem_traits::{AgentMemError, Result};
use async_trait::async_trait;
use libsql::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

pub struct LibSqlAssociationRepository {
    conn: Arc<Mutex<Connection>>,
}

impl LibSqlAssociationRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl AssociationRepositoryTrait for LibSqlAssociationRepository {
    async fn create(&self, association: &MemoryAssociation) -> Result<MemoryAssociation> {
        debug!(
            "Creating association: {} -> {}",
            association.from_memory_id, association.to_memory_id
        );

        let metadata_json = serde_json::to_string(&association.metadata).map_err(|e| {
            AgentMemError::StorageError(format!("Failed to serialize metadata: {e}"))
        })?;

        let conn = self.conn.lock().await;

        conn.execute(
            "INSERT INTO memory_associations (
                id, organization_id, user_id, agent_id,
                from_memory_id, to_memory_id, association_type,
                strength, confidence, metadata,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            (
                association.id.as_str(),
                association.organization_id.as_str(),
                association.user_id.as_str(),
                association.agent_id.as_str(),
                association.from_memory_id.as_str(),
                association.to_memory_id.as_str(),
                association.association_type.as_str(),
                association.strength as f64,
                association.confidence as f64,
                metadata_json.as_str(),
                association.created_at.timestamp(),
                association.updated_at.timestamp(),
            ),
        )
        .await
        .map_err(|e| AgentMemError::StorageError(format!("Failed to create association: {e}")))?;

        Ok(association.clone())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<MemoryAssociation>> {
        debug!("Finding association by ID: {}", id);

        let conn = self.conn.lock().await;

        let mut stmt = conn
            .prepare(
                "SELECT id, organization_id, user_id, agent_id,
                    from_memory_id, to_memory_id, association_type,
                    strength, confidence, metadata,
                    created_at, updated_at
             FROM memory_associations
             WHERE id = ?",
            )
            .await
            .map_err(|e| {
                AgentMemError::StorageError(format!("Failed to prepare statement: {e}"))
            })?;

        let mut rows = stmt.query([id]).await.map_err(|e| {
            AgentMemError::StorageError(format!("Failed to query association: {e}"))
        })?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to fetch row: {e}")))?
        {
            let metadata_str: String = row
                .get(9)
                .map_err(|e| AgentMemError::StorageError(format!("Failed to get metadata: {e}")))?;
            let metadata: serde_json::Value =
                serde_json::from_str(&metadata_str).unwrap_or(serde_json::Value::Null);

            let created_at_ts: i64 = row.get(10).map_err(|e| {
                AgentMemError::StorageError(format!("Failed to get created_at: {e}"))
            })?;
            let updated_at_ts: i64 = row.get(11).map_err(|e| {
                AgentMemError::StorageError(format!("Failed to get updated_at: {e}"))
            })?;

            let strength_f64: f64 = row
                .get(7)
                .map_err(|e| AgentMemError::StorageError(format!("Failed to get strength: {e}")))?;
            let confidence_f64: f64 = row.get(8).map_err(|e| {
                AgentMemError::StorageError(format!("Failed to get confidence: {e}"))
            })?;

            Ok(Some(MemoryAssociation {
                id: row
                    .get(0)
                    .map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                organization_id: row
                    .get(1)
                    .map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                user_id: row
                    .get(2)
                    .map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                agent_id: row
                    .get(3)
                    .map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                from_memory_id: row
                    .get(4)
                    .map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                to_memory_id: row
                    .get(5)
                    .map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                association_type: row
                    .get(6)
                    .map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                strength: strength_f64 as f32,
                confidence: confidence_f64 as f32,
                metadata,
                created_at: chrono::DateTime::from_timestamp(created_at_ts, 0)
                    .unwrap_or_else(chrono::Utc::now),
                updated_at: chrono::DateTime::from_timestamp(updated_at_ts, 0)
                    .unwrap_or_else(chrono::Utc::now),
            }))
        } else {
            Ok(None)
        }
    }

    async fn find_by_memory_id(
        &self,
        memory_id: &str,
        user_id: &str,
    ) -> Result<Vec<MemoryAssociation>> {
        debug!("Finding associations for memory: {}", memory_id);

        let conn = self.conn.lock().await;

        let mut stmt = conn
            .prepare(
                "SELECT id, organization_id, user_id, agent_id,
                    from_memory_id, to_memory_id, association_type,
                    strength, confidence, metadata,
                    created_at, updated_at
             FROM memory_associations
             WHERE (from_memory_id = ? OR to_memory_id = ?) AND user_id = ?
             ORDER BY strength DESC",
            )
            .await
            .map_err(|e| {
                AgentMemError::StorageError(format!("Failed to prepare statement: {e}"))
            })?;

        let mut rows = stmt
            .query([memory_id, memory_id, user_id])
            .await
            .map_err(|e| {
                AgentMemError::StorageError(format!("Failed to query associations: {e}"))
            })?;

        let mut associations = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to fetch row: {e}")))?
        {
            let metadata_str: String = row
                .get(9)
                .map_err(|e| AgentMemError::StorageError(format!("Failed to get metadata: {e}")))?;
            let metadata: serde_json::Value =
                serde_json::from_str(&metadata_str).unwrap_or(serde_json::Value::Null);

            let created_at_ts: i64 = row.get(10).map_err(|e| {
                AgentMemError::StorageError(format!("Failed to get created_at: {e}"))
            })?;
            let updated_at_ts: i64 = row.get(11).map_err(|e| {
                AgentMemError::StorageError(format!("Failed to get updated_at: {e}"))
            })?;

            let strength_f64: f64 = row
                .get(7)
                .map_err(|e| AgentMemError::StorageError(format!("Failed to get strength: {e}")))?;
            let confidence_f64: f64 = row.get(8).map_err(|e| {
                AgentMemError::StorageError(format!("Failed to get confidence: {e}"))
            })?;

            associations.push(MemoryAssociation {
                id: row
                    .get(0)
                    .map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                organization_id: row
                    .get(1)
                    .map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                user_id: row
                    .get(2)
                    .map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                agent_id: row
                    .get(3)
                    .map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                from_memory_id: row
                    .get(4)
                    .map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                to_memory_id: row
                    .get(5)
                    .map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                association_type: row
                    .get(6)
                    .map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                strength: strength_f64 as f32,
                confidence: confidence_f64 as f32,
                metadata,
                created_at: chrono::DateTime::from_timestamp(created_at_ts, 0)
                    .unwrap_or_else(chrono::Utc::now),
                updated_at: chrono::DateTime::from_timestamp(updated_at_ts, 0)
                    .unwrap_or_else(chrono::Utc::now),
            });
        }

        Ok(associations)
    }

    async fn find_by_type(
        &self,
        memory_id: &str,
        user_id: &str,
        association_type: &str,
    ) -> Result<Vec<MemoryAssociation>> {
        debug!(
            "Finding associations by type: {} for memory: {}",
            association_type, memory_id
        );

        let conn = self.conn.lock().await;

        let mut stmt = conn
            .prepare(
                "SELECT id, organization_id, user_id, agent_id,
                    from_memory_id, to_memory_id, association_type,
                    strength, confidence, metadata,
                    created_at, updated_at
             FROM memory_associations
             WHERE (from_memory_id = ? OR to_memory_id = ?) 
                   AND user_id = ? 
                   AND association_type = ?
             ORDER BY strength DESC",
            )
            .await
            .map_err(|e| {
                AgentMemError::StorageError(format!("Failed to prepare statement: {e}"))
            })?;

        let mut rows = stmt
            .query([memory_id, memory_id, user_id, association_type])
            .await
            .map_err(|e| {
                AgentMemError::StorageError(format!("Failed to query associations: {e}"))
            })?;

        let mut associations = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to fetch row: {e}")))?
        {
            let metadata_str: String = row
                .get(9)
                .map_err(|e| AgentMemError::StorageError(format!("Failed to get metadata: {e}")))?;
            let metadata: serde_json::Value =
                serde_json::from_str(&metadata_str).unwrap_or(serde_json::Value::Null);

            let created_at_ts: i64 = row.get(10).map_err(|e| {
                AgentMemError::StorageError(format!("Failed to get created_at: {e}"))
            })?;
            let updated_at_ts: i64 = row.get(11).map_err(|e| {
                AgentMemError::StorageError(format!("Failed to get updated_at: {e}"))
            })?;

            let strength_f64: f64 = row
                .get(7)
                .map_err(|e| AgentMemError::StorageError(format!("Failed to get strength: {e}")))?;
            let confidence_f64: f64 = row.get(8).map_err(|e| {
                AgentMemError::StorageError(format!("Failed to get confidence: {e}"))
            })?;

            associations.push(MemoryAssociation {
                id: row
                    .get(0)
                    .map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                organization_id: row
                    .get(1)
                    .map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                user_id: row
                    .get(2)
                    .map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                agent_id: row
                    .get(3)
                    .map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                from_memory_id: row
                    .get(4)
                    .map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                to_memory_id: row
                    .get(5)
                    .map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                association_type: row
                    .get(6)
                    .map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                strength: strength_f64 as f32,
                confidence: confidence_f64 as f32,
                metadata,
                created_at: chrono::DateTime::from_timestamp(created_at_ts, 0)
                    .unwrap_or_else(chrono::Utc::now),
                updated_at: chrono::DateTime::from_timestamp(updated_at_ts, 0)
                    .unwrap_or_else(chrono::Utc::now),
            });
        }

        Ok(associations)
    }

    async fn update_strength(&self, id: &str, strength: f32) -> Result<()> {
        debug!("Updating association strength: {} to {}", id, strength);

        let conn = self.conn.lock().await;
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "UPDATE memory_associations SET strength = ?, updated_at = ? WHERE id = ?",
            (strength as f64, now, id),
        )
        .await
        .map_err(|e| AgentMemError::StorageError(format!("Failed to update strength: {e}")))?;

        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<()> {
        debug!("Deleting association: {}", id);

        let conn = self.conn.lock().await;

        conn.execute("DELETE FROM memory_associations WHERE id = ?", [id])
            .await
            .map_err(|e| {
                AgentMemError::StorageError(format!("Failed to delete association: {e}"))
            })?;

        Ok(())
    }

    async fn count_by_user(&self, user_id: &str) -> Result<i64> {
        let conn = self.conn.lock().await;

        let mut stmt = conn
            .prepare("SELECT COUNT(*) FROM memory_associations WHERE user_id = ?")
            .await
            .map_err(|e| {
                AgentMemError::StorageError(format!("Failed to prepare statement: {e}"))
            })?;

        let mut rows = stmt
            .query([user_id])
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to query count: {e}")))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to fetch row: {e}")))?
        {
            let count: i64 = row
                .get(0)
                .map_err(|e| AgentMemError::StorageError(format!("Failed to get count: {e}")))?;
            Ok(count)
        } else {
            Ok(0)
        }
    }

    async fn count_by_type(&self, user_id: &str) -> Result<Vec<(String, i64)>> {
        let conn = self.conn.lock().await;

        let mut stmt = conn
            .prepare(
                "SELECT association_type, COUNT(*) as count 
             FROM memory_associations 
             WHERE user_id = ? 
             GROUP BY association_type",
            )
            .await
            .map_err(|e| {
                AgentMemError::StorageError(format!("Failed to prepare statement: {e}"))
            })?;

        let mut rows = stmt
            .query([user_id])
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to query counts: {e}")))?;

        let mut results = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to fetch row: {e}")))?
        {
            let assoc_type: String = row
                .get(0)
                .map_err(|e| AgentMemError::StorageError(format!("Failed to get type: {e}")))?;
            let count: i64 = row
                .get(1)
                .map_err(|e| AgentMemError::StorageError(format!("Failed to get count: {e}")))?;
            results.push((assoc_type, count));
        }

        Ok(results)
    }

    async fn avg_strength(&self, user_id: &str) -> Result<f32> {
        let conn = self.conn.lock().await;

        let mut stmt = conn
            .prepare("SELECT AVG(strength) FROM memory_associations WHERE user_id = ?")
            .await
            .map_err(|e| {
                AgentMemError::StorageError(format!("Failed to prepare statement: {e}"))
            })?;

        let mut rows = stmt
            .query([user_id])
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to query avg: {e}")))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to fetch row: {e}")))?
        {
            let avg: f64 = row
                .get(0)
                .map_err(|e| AgentMemError::StorageError(format!("Failed to get avg: {e}")))?;
            Ok(avg as f32)
        } else {
            Ok(0.0)
        }
    }

    async fn find_strongest(&self, user_id: &str, limit: i64) -> Result<Vec<MemoryAssociation>> {
        debug!("Finding strongest associations for user: {}", user_id);

        let conn = self.conn.lock().await;

        let mut stmt = conn
            .prepare(
                "SELECT id, organization_id, user_id, agent_id,
                    from_memory_id, to_memory_id, association_type,
                    strength, confidence, metadata,
                    created_at, updated_at
             FROM memory_associations
             WHERE user_id = ?
             ORDER BY strength DESC
             LIMIT ?",
            )
            .await
            .map_err(|e| {
                AgentMemError::StorageError(format!("Failed to prepare statement: {e}"))
            })?;

        let mut rows = stmt
            .query([user_id, &limit.to_string()])
            .await
            .map_err(|e| {
                AgentMemError::StorageError(format!("Failed to query associations: {e}"))
            })?;

        let mut associations = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to fetch row: {e}")))?
        {
            let metadata_str: String = row
                .get(9)
                .map_err(|e| AgentMemError::StorageError(format!("Failed to get metadata: {e}")))?;
            let metadata: serde_json::Value =
                serde_json::from_str(&metadata_str).unwrap_or(serde_json::Value::Null);

            let created_at_ts: i64 = row.get(10).map_err(|e| {
                AgentMemError::StorageError(format!("Failed to get created_at: {e}"))
            })?;
            let updated_at_ts: i64 = row.get(11).map_err(|e| {
                AgentMemError::StorageError(format!("Failed to get updated_at: {e}"))
            })?;

            let strength_f64: f64 = row
                .get(7)
                .map_err(|e| AgentMemError::StorageError(format!("Failed to get strength: {e}")))?;
            let confidence_f64: f64 = row.get(8).map_err(|e| {
                AgentMemError::StorageError(format!("Failed to get confidence: {e}"))
            })?;

            associations.push(MemoryAssociation {
                id: row
                    .get(0)
                    .map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                organization_id: row
                    .get(1)
                    .map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                user_id: row
                    .get(2)
                    .map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                agent_id: row
                    .get(3)
                    .map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                from_memory_id: row
                    .get(4)
                    .map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                to_memory_id: row
                    .get(5)
                    .map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                association_type: row
                    .get(6)
                    .map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                strength: strength_f64 as f32,
                confidence: confidence_f64 as f32,
                metadata,
                created_at: chrono::DateTime::from_timestamp(created_at_ts, 0)
                    .unwrap_or_else(chrono::Utc::now),
                updated_at: chrono::DateTime::from_timestamp(updated_at_ts, 0)
                    .unwrap_or_else(chrono::Utc::now),
            });
        }

        Ok(associations)
    }
}
