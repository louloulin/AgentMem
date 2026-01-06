//! LibSQL User Repository Implementation
//!
//! 使用 LibSQL 实现用户存储

use crate::storage::models::User;
use crate::storage::traits::UserRepositoryTrait;
use agent_mem_traits::{AgentMemError, Result};
use async_trait::async_trait;
use libsql::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;

/// LibSQL User Repository
pub struct LibSqlUserRepository {
    conn: Arc<Mutex<Connection>>,
}

impl LibSqlUserRepository {
    /// Create a new LibSQL user repository
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Initialize the users table (匹配 User 模型)
    pub async fn init_table(&self) -> Result<()> {
        let conn = self.conn.lock().await;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                organization_id TEXT NOT NULL,
                name TEXT NOT NULL,
                status TEXT NOT NULL,
                timezone TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                is_deleted INTEGER NOT NULL DEFAULT 0,
                created_by_id TEXT,
                last_updated_by_id TEXT
            )",
            (),
        )
        .await
        .map_err(|e| AgentMemError::StorageError(format!("Failed to create users table: {}", e)))?;

        Ok(())
    }
}

#[async_trait]
impl UserRepositoryTrait for LibSqlUserRepository {
    async fn create(&self, user: &User) -> Result<User> {
        let conn = self.conn.lock().await;

        conn.execute(
            "INSERT INTO users (id, organization_id, name, status, timezone, created_at, updated_at, is_deleted, created_by_id, last_updated_by_id)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            libsql::params![
                &user.id,
                &user.organization_id,
                &user.name,
                &user.status,
                &user.timezone,
                user.created_at.timestamp(),
                user.updated_at.timestamp(),
                if user.is_deleted { 1 } else { 0 },
                &user.created_by_id,
                &user.last_updated_by_id,
            ],
        )
        .await
        .map_err(|e| AgentMemError::StorageError(format!("Failed to create user: {}", e)))?;

        Ok(user.clone())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<User>> {
        let conn = self.conn.lock().await;
        
        let mut stmt = conn
            .prepare("SELECT id, email, name, created_at, updated_at FROM users WHERE id = ?")
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to prepare statement: {}", e)))?;

        let mut rows = stmt
            .query(libsql::params![id.to_string()])
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to query user: {}", e)))?;

        if let Some(row) = rows.next().await.map_err(|e| AgentMemError::StorageError(format!("Failed to fetch row: {}", e)))? {
            let id_str: String = row.get(0).map_err(|e| AgentMemError::StorageError(format!("Failed to get id: {}", e)))?;
            let email: String = row.get(1).map_err(|e| AgentMemError::StorageError(format!("Failed to get email: {}", e)))?;
            let name: Option<String> = row.get(2).map_err(|e| AgentMemError::StorageError(format!("Failed to get name: {}", e)))?;
            let created_at: i64 = row.get(3).map_err(|e| AgentMemError::StorageError(format!("Failed to get created_at: {}", e)))?;
            let updated_at: i64 = row.get(4).map_err(|e| AgentMemError::StorageError(format!("Failed to get updated_at: {}", e)))?;

            Ok(Some(User {
                id: Uuid::parse_str(&id_str).map_err(|e| AgentMemError::StorageError(format!("Invalid UUID: {}", e)))?,
                email,
                name,
                created_at: chrono::DateTime::from_timestamp(created_at, 0)
                    .ok_or_else(|| AgentMemError::StorageError("Invalid timestamp".to_string()))?,
                updated_at: chrono::DateTime::from_timestamp(updated_at, 0)
                    .ok_or_else(|| AgentMemError::StorageError("Invalid timestamp".to_string()))?,
            }))
        } else {
            Ok(None)
        }
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let conn = self.conn.lock().await;
        
        let mut stmt = conn
            .prepare("SELECT id, email, name, created_at, updated_at FROM users WHERE email = ?")
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to prepare statement: {}", e)))?;

        let mut rows = stmt
            .query(libsql::params![email])
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to query user: {}", e)))?;

        if let Some(row) = rows.next().await.map_err(|e| AgentMemError::StorageError(format!("Failed to fetch row: {}", e)))? {
            let id_str: String = row.get(0).map_err(|e| AgentMemError::StorageError(format!("Failed to get id: {}", e)))?;
            let email: String = row.get(1).map_err(|e| AgentMemError::StorageError(format!("Failed to get email: {}", e)))?;
            let name: Option<String> = row.get(2).map_err(|e| AgentMemError::StorageError(format!("Failed to get name: {}", e)))?;
            let created_at: i64 = row.get(3).map_err(|e| AgentMemError::StorageError(format!("Failed to get created_at: {}", e)))?;
            let updated_at: i64 = row.get(4).map_err(|e| AgentMemError::StorageError(format!("Failed to get updated_at: {}", e)))?;

            Ok(Some(User {
                id: Uuid::parse_str(&id_str).map_err(|e| AgentMemError::StorageError(format!("Invalid UUID: {}", e)))?,
                email,
                name,
                created_at: chrono::DateTime::from_timestamp(created_at, 0)
                    .ok_or_else(|| AgentMemError::StorageError("Invalid timestamp".to_string()))?,
                updated_at: chrono::DateTime::from_timestamp(updated_at, 0)
                    .ok_or_else(|| AgentMemError::StorageError("Invalid timestamp".to_string()))?,
            }))
        } else {
            Ok(None)
        }
    }

    async fn update(&self, user: &User) -> Result<User> {
        let conn = self.conn.lock().await;
        
        conn.execute(
            "UPDATE users SET email = ?, name = ?, updated_at = ? WHERE id = ?",
            libsql::params![
                &user.email,
                &user.name,
                user.updated_at.timestamp(),
                user.id.to_string(),
            ],
        )
        .await
        .map_err(|e| AgentMemError::StorageError(format!("Failed to update user: {}", e)))?;

        Ok(user.clone())
    }

    async fn delete(&self, id: &Uuid) -> Result<()> {
        let conn = self.conn.lock().await;
        
        conn.execute(
            "DELETE FROM users WHERE id = ?",
            libsql::params![id.to_string()],
        )
        .await
        .map_err(|e| AgentMemError::StorageError(format!("Failed to delete user: {}", e)))?;

        Ok(())
    }

    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<User>> {
        let conn = self.conn.lock().await;
        
        let mut stmt = conn
            .prepare("SELECT id, email, name, created_at, updated_at FROM users ORDER BY created_at DESC LIMIT ? OFFSET ?")
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to prepare statement: {}", e)))?;

        let mut rows = stmt
            .query(libsql::params![limit, offset])
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to query users: {}", e)))?;

        let mut users = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| AgentMemError::StorageError(format!("Failed to fetch row: {}", e)))? {
            let id_str: String = row.get(0).map_err(|e| AgentMemError::StorageError(format!("Failed to get id: {}", e)))?;
            let email: String = row.get(1).map_err(|e| AgentMemError::StorageError(format!("Failed to get email: {}", e)))?;
            let name: Option<String> = row.get(2).map_err(|e| AgentMemError::StorageError(format!("Failed to get name: {}", e)))?;
            let created_at: i64 = row.get(3).map_err(|e| AgentMemError::StorageError(format!("Failed to get created_at: {}", e)))?;
            let updated_at: i64 = row.get(4).map_err(|e| AgentMemError::StorageError(format!("Failed to get updated_at: {}", e)))?;

            users.push(User {
                id: Uuid::parse_str(&id_str).map_err(|e| AgentMemError::StorageError(format!("Invalid UUID: {}", e)))?,
                email,
                name,
                created_at: chrono::DateTime::from_timestamp(created_at, 0)
                    .ok_or_else(|| AgentMemError::StorageError("Invalid timestamp".to_string()))?,
                updated_at: chrono::DateTime::from_timestamp(updated_at, 0)
                    .ok_or_else(|| AgentMemError::StorageError("Invalid timestamp".to_string()))?,
            });
        }

        Ok(users)
    }
}

