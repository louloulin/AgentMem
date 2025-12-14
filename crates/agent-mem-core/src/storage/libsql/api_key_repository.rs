//! LibSQL implementation of ApiKeyRepositoryTrait

use agent_mem_traits::{AgentMemError, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use libsql::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::storage::models::ApiKey;
use crate::storage::traits::ApiKeyRepositoryTrait;
use crate::storage::libsql::connection::LibSqlConnectionPool;

/// LibSQL implementation of the API key repository
pub struct LibSqlApiKeyRepository {
    /// Legacy single-connection mode (Arc<Mutex<Connection>>)
    conn: Option<Arc<Mutex<Connection>>>,
    /// Preferred pooled mode
    pool: Option<Arc<LibSqlConnectionPool>>,
}

impl LibSqlApiKeyRepository {
    /// Create a new LibSQL API key repository
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
}

#[async_trait]
impl ApiKeyRepositoryTrait for LibSqlApiKeyRepository {
    async fn create(&self, api_key: &ApiKey) -> Result<ApiKey> {
        let conn = self.get_conn().await?;
        let conn = conn.lock().await;

        conn.execute(
            "INSERT INTO api_keys (
                id, key_hash, name, user_id, organization_id,
                expires_at, last_used_at, created_at, updated_at, is_deleted
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            libsql::params![
                api_key.id.clone(),
                api_key.key_hash.clone(),
                api_key.name.clone(),
                api_key.user_id.clone(),
                api_key.organization_id.clone(),
                api_key.expires_at.map(|dt| dt.timestamp()),
                api_key.last_used_at.map(|dt| dt.timestamp()),
                api_key.created_at.timestamp(),
                api_key.updated_at.timestamp(),
                if api_key.is_deleted { 1 } else { 0 },
            ],
        )
        .await
        .map_err(|e| AgentMemError::StorageError(format!("Failed to create API key: {e}")))?;

        Ok(api_key.clone())
    }

    async fn find_by_key(&self, key_hash: &str) -> Result<Option<ApiKey>> {
        let conn = self.get_conn().await?;
        let conn = conn.lock().await;

        let mut stmt = conn
            .prepare(
                "SELECT id, key_hash, name, user_id, organization_id,
                        expires_at, last_used_at, created_at, updated_at, is_deleted
                 FROM api_keys
                 WHERE key_hash = ? AND is_deleted = 0",
            )
            .await
            .map_err(|e| {
                AgentMemError::StorageError(format!("Failed to prepare statement: {e}"))
            })?;

        let mut rows = stmt
            .query(libsql::params![key_hash.to_string()])
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to query API key: {e}")))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to fetch API key row: {e}")))?
        {
            let expires_at_ts: Option<i64> = row.get(5)
                .map_err(|e| AgentMemError::StorageError(format!("Failed to get expires_at: {e}")))?;
            let last_used_at_ts: Option<i64> = row.get(6)
                .map_err(|e| AgentMemError::StorageError(format!("Failed to get last_used_at: {e}")))?;
            let created_at_ts: i64 = row.get(7)
                .map_err(|e| AgentMemError::StorageError(format!("Failed to get created_at: {e}")))?;
            let updated_at_ts: i64 = row.get(8)
                .map_err(|e| AgentMemError::StorageError(format!("Failed to get updated_at: {e}")))?;
            let is_deleted: i64 = row.get(9)
                .map_err(|e| AgentMemError::StorageError(format!("Failed to get is_deleted: {e}")))?;

            Ok(Some(ApiKey {
                id: row.get(0)
                    .map_err(|e| AgentMemError::StorageError(format!("Failed to get id: {e}")))?,
                key_hash: row.get(1)
                    .map_err(|e| AgentMemError::StorageError(format!("Failed to get key_hash: {e}")))?,
                name: row.get(2)
                    .map_err(|e| AgentMemError::StorageError(format!("Failed to get name: {e}")))?,
                user_id: row.get(3)
                    .map_err(|e| AgentMemError::StorageError(format!("Failed to get user_id: {e}")))?,
                organization_id: row.get(4)
                    .map_err(|e| AgentMemError::StorageError(format!("Failed to get organization_id: {e}")))?,
                expires_at: expires_at_ts.and_then(|ts| DateTime::from_timestamp(ts, 0)),
                last_used_at: last_used_at_ts.and_then(|ts| DateTime::from_timestamp(ts, 0)),
                created_at: DateTime::from_timestamp(created_at_ts, 0).ok_or_else(|| {
                    AgentMemError::StorageError("Invalid created_at timestamp".to_string())
                })?,
                updated_at: DateTime::from_timestamp(updated_at_ts, 0).ok_or_else(|| {
                    AgentMemError::StorageError("Invalid updated_at timestamp".to_string())
                })?,
                is_deleted: is_deleted != 0,
            }))
        } else {
            Ok(None)
        }
    }

    async fn find_by_user_id(&self, user_id: &str) -> Result<Vec<ApiKey>> {
        let conn = self.get_conn().await?;
        let conn = conn.lock().await;

        let mut stmt = conn
            .prepare(
                "SELECT id, key_hash, name, user_id, organization_id,
                        expires_at, last_used_at, created_at, updated_at, is_deleted
                 FROM api_keys
                 WHERE user_id = ? AND is_deleted = 0
                 ORDER BY created_at DESC",
            )
            .await
            .map_err(|e| {
                AgentMemError::StorageError(format!("Failed to prepare statement: {e}"))
            })?;

        let mut rows = stmt
            .query(libsql::params![user_id.to_string()])
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to query API keys: {e}")))?;

        let mut api_keys = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to fetch API key row: {e}")))?
        {
            let expires_at_ts: Option<i64> = row.get(5).unwrap();
            let last_used_at_ts: Option<i64> = row.get(6).unwrap();
            let created_at_ts: i64 = row.get(7).unwrap();
            let updated_at_ts: i64 = row.get(8).unwrap();
            let is_deleted: i64 = row.get(9).unwrap();

            api_keys.push(ApiKey {
                id: row.get(0).unwrap(),
                key_hash: row.get(1).unwrap(),
                name: row.get(2).unwrap(),
                user_id: row.get(3).unwrap(),
                organization_id: row.get(4).unwrap(),
                expires_at: expires_at_ts.and_then(|ts| DateTime::from_timestamp(ts, 0)),
                last_used_at: last_used_at_ts.and_then(|ts| DateTime::from_timestamp(ts, 0)),
                created_at: DateTime::from_timestamp(created_at_ts, 0).ok_or_else(|| {
                    AgentMemError::StorageError("Invalid created_at timestamp".to_string())
                })?,
                updated_at: DateTime::from_timestamp(updated_at_ts, 0).ok_or_else(|| {
                    AgentMemError::StorageError("Invalid updated_at timestamp".to_string())
                })?,
                is_deleted: is_deleted != 0,
            });
        }

        Ok(api_keys)
    }

    async fn update(&self, api_key: &ApiKey) -> Result<ApiKey> {
        let api_key_id = api_key.id.clone();

        {
            let conn = self.get_conn().await?;
        let conn = conn.lock().await;

            let rows_affected = conn
                .execute(
                    "UPDATE api_keys SET
                        key_hash = ?, name = ?, user_id = ?, organization_id = ?,
                        expires_at = ?, last_used_at = ?, updated_at = ?
                    WHERE id = ? AND is_deleted = 0",
                    libsql::params![
                        api_key.key_hash.clone(),
                        api_key.name.clone(),
                        api_key.user_id.clone(),
                        api_key.organization_id.clone(),
                        api_key.expires_at.map(|dt| dt.timestamp()),
                        api_key.last_used_at.map(|dt| dt.timestamp()),
                        Utc::now().timestamp(),
                        api_key.id.clone(),
                    ],
                )
                .await
                .map_err(|e| {
                    AgentMemError::StorageError(format!("Failed to update API key: {e}"))
                })?;

            if rows_affected == 0 {
                return Err(AgentMemError::NotFound(format!(
                    "API key with id {} not found",
                    api_key.id
                )));
            }
        } // Release lock here

        // Fetch and return the updated API key
        self.find_by_key(&api_key.key_hash).await?.ok_or_else(|| {
            AgentMemError::NotFound(format!("API key with id {api_key_id} not found"))
        })
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let conn = self.get_conn().await?;
        let conn = conn.lock().await;

        let rows_affected = conn
            .execute(
                "DELETE FROM api_keys WHERE id = ?",
                libsql::params![id.to_string()],
            )
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to delete API key: {e}")))?;

        if rows_affected == 0 {
            return Err(AgentMemError::NotFound(format!(
                "API key with id {id} not found"
            )));
        }

        Ok(())
    }

    async fn revoke(&self, id: &str) -> Result<()> {
        let conn = self.get_conn().await?;
        let conn = conn.lock().await;

        let rows_affected = conn
            .execute(
                "UPDATE api_keys SET is_deleted = 1, updated_at = ? WHERE id = ?",
                libsql::params![Utc::now().timestamp(), id.to_string()],
            )
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to revoke API key: {e}")))?;

        if rows_affected == 0 {
            return Err(AgentMemError::NotFound(format!(
                "API key with id {id} not found"
            )));
        }

        Ok(())
    }

    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<ApiKey>> {
        let conn = self.get_conn().await?;
        let conn = conn.lock().await;

        let mut stmt = conn
            .prepare(
                "SELECT id, key_hash, name, user_id, organization_id,
                        expires_at, last_used_at, created_at, updated_at, is_deleted
                 FROM api_keys
                 WHERE is_deleted = 0
                 ORDER BY created_at DESC
                 LIMIT ? OFFSET ?",
            )
            .await
            .map_err(|e| {
                AgentMemError::StorageError(format!("Failed to prepare statement: {e}"))
            })?;

        let mut rows = stmt
            .query(libsql::params![limit, offset])
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to query API keys: {e}")))?;

        let mut api_keys = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to fetch API key row: {e}")))?
        {
            let expires_at_ts: Option<i64> = row.get(5).unwrap();
            let last_used_at_ts: Option<i64> = row.get(6).unwrap();
            let created_at_ts: i64 = row.get(7).unwrap();
            let updated_at_ts: i64 = row.get(8).unwrap();
            let is_deleted: i64 = row.get(9).unwrap();

            api_keys.push(ApiKey {
                id: row.get(0).unwrap(),
                key_hash: row.get(1).unwrap(),
                name: row.get(2).unwrap(),
                user_id: row.get(3).unwrap(),
                organization_id: row.get(4).unwrap(),
                expires_at: expires_at_ts.and_then(|ts| DateTime::from_timestamp(ts, 0)),
                last_used_at: last_used_at_ts.and_then(|ts| DateTime::from_timestamp(ts, 0)),
                created_at: DateTime::from_timestamp(created_at_ts, 0).ok_or_else(|| {
                    AgentMemError::StorageError("Invalid created_at timestamp".to_string())
                })?,
                updated_at: DateTime::from_timestamp(updated_at_ts, 0).ok_or_else(|| {
                    AgentMemError::StorageError("Invalid updated_at timestamp".to_string())
                })?,
                is_deleted: is_deleted != 0,
            });
        }

        Ok(api_keys)
    }
}
