//! LibSQL User Repository Implementation

use crate::storage::models::User;
use crate::storage::traits::UserRepositoryTrait;
use agent_mem_traits::{AgentMemError, Result};
use async_trait::async_trait;
use libsql::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;

/// LibSQL implementation of UserRepository
pub struct LibSqlUserRepository {
    conn: Arc<Mutex<Connection>>,
}

impl LibSqlUserRepository {
    /// Create a new LibSQL user repository
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl UserRepositoryTrait for LibSqlUserRepository {
    async fn create(&self, user: &User) -> Result<User> {
        let conn = self.conn.lock().await;

        // Serialize roles to JSON
        let roles_json = user.roles.as_ref()
            .map(|r| serde_json::to_string(r).unwrap_or_else(|_| "[]".to_string()))
            .unwrap_or_else(|| "[]".to_string());

        conn.execute(
            "INSERT INTO users (id, organization_id, name, email, password_hash, roles, status, timezone, created_at, updated_at, is_deleted, created_by_id, last_updated_by_id)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            libsql::params![
                user.id.clone(),
                user.organization_id.clone(),
                user.name.clone(),
                user.email.clone(),
                user.password_hash.clone(),
                roles_json,
                user.status.clone(),
                user.timezone.clone(),
                user.created_at.timestamp(),
                user.updated_at.timestamp(),
                if user.is_deleted { 1 } else { 0 },
                user.created_by_id.clone(),
                user.last_updated_by_id.clone(),
            ],
        )
        .await
        .map_err(|e| AgentMemError::StorageError(format!("Failed to create user: {}", e)))?;

        Ok(user.clone())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<User>> {
        let conn = self.conn.lock().await;

        let mut rows = conn
            .query(
                "SELECT id, organization_id, name, email, password_hash, roles, status, timezone, created_at, updated_at, is_deleted, created_by_id, last_updated_by_id
                 FROM users WHERE id = ? AND is_deleted = 0",
                libsql::params![id],
            )
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to query user: {}", e)))?;

        if let Some(row) = rows.next().await.map_err(|e| AgentMemError::StorageError(e.to_string()))? {
            let created_at_ts: i64 = row.get(8).map_err(|e| AgentMemError::StorageError(e.to_string()))?;
            let updated_at_ts: i64 = row.get(9).map_err(|e| AgentMemError::StorageError(e.to_string()))?;
            let is_deleted_int: i64 = row.get(10).map_err(|e| AgentMemError::StorageError(e.to_string()))?;

            // Deserialize roles from JSON
            let roles_str: String = row.get(5).map_err(|e| AgentMemError::StorageError(e.to_string()))?;
            let roles: Option<Vec<String>> = serde_json::from_str(&roles_str).ok();

            Ok(Some(User {
                id: row.get(0).map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                organization_id: row.get(1).map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                name: row.get(2).map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                email: row.get(3).map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                password_hash: row.get(4).map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                roles,
                status: row.get(6).map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                timezone: row.get(7).map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                created_at: chrono::DateTime::from_timestamp(created_at_ts, 0)
                    .ok_or_else(|| AgentMemError::StorageError("Invalid timestamp".to_string()))?,
                updated_at: chrono::DateTime::from_timestamp(updated_at_ts, 0)
                    .ok_or_else(|| AgentMemError::StorageError("Invalid timestamp".to_string()))?,
                is_deleted: is_deleted_int != 0,
                created_by_id: row.get(11).map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                last_updated_by_id: row.get(12).map_err(|e| AgentMemError::StorageError(e.to_string()))?,
            }))
        } else {
            Ok(None)
        }
    }

    async fn find_by_email(&self, email: &str, org_id: &str) -> Result<Option<User>> {
        let conn = self.conn.lock().await;

        let mut rows = conn
            .query(
                "SELECT id, organization_id, name, email, password_hash, roles, status, timezone, created_at, updated_at, is_deleted, created_by_id, last_updated_by_id
                 FROM users WHERE email = ? AND organization_id = ? AND is_deleted = 0",
                libsql::params![email, org_id],
            )
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to query user by email: {}", e)))?;

        if let Some(row) = rows.next().await.map_err(|e| AgentMemError::StorageError(e.to_string()))? {
            let created_at_ts: i64 = row.get(8).map_err(|e| AgentMemError::StorageError(e.to_string()))?;
            let updated_at_ts: i64 = row.get(9).map_err(|e| AgentMemError::StorageError(e.to_string()))?;
            let is_deleted_int: i64 = row.get(10).map_err(|e| AgentMemError::StorageError(e.to_string()))?;

            // Deserialize roles from JSON
            let roles_str: String = row.get(5).map_err(|e| AgentMemError::StorageError(e.to_string()))?;
            let roles: Option<Vec<String>> = serde_json::from_str(&roles_str).ok();

            Ok(Some(User {
                id: row.get(0).map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                organization_id: row.get(1).map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                name: row.get(2).map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                email: row.get(3).map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                password_hash: row.get(4).map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                roles,
                status: row.get(6).map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                timezone: row.get(7).map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                created_at: chrono::DateTime::from_timestamp(created_at_ts, 0)
                    .ok_or_else(|| AgentMemError::StorageError("Invalid timestamp".to_string()))?,
                updated_at: chrono::DateTime::from_timestamp(updated_at_ts, 0)
                    .ok_or_else(|| AgentMemError::StorageError("Invalid timestamp".to_string()))?,
                is_deleted: is_deleted_int != 0,
                created_by_id: row.get(11).map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                last_updated_by_id: row.get(12).map_err(|e| AgentMemError::StorageError(e.to_string()))?,
            }))
        } else {
            Ok(None)
        }
    }

    async fn email_exists(&self, email: &str, org_id: &str) -> Result<bool> {
        let conn = self.conn.lock().await;

        let mut rows = conn
            .query(
                "SELECT COUNT(*) FROM users WHERE email = ? AND organization_id = ? AND is_deleted = 0",
                libsql::params![email, org_id],
            )
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to check email existence: {}", e)))?;

        if let Some(row) = rows.next().await.map_err(|e| AgentMemError::StorageError(e.to_string()))? {
            let count: i64 = row.get(0).map_err(|e| AgentMemError::StorageError(e.to_string()))?;
            Ok(count > 0)
        } else {
            Ok(false)
        }
    }

    async fn find_by_organization_id(&self, org_id: &str) -> Result<Vec<User>> {
        let conn = self.conn.lock().await;

        let mut rows = conn
            .query(
                "SELECT id, organization_id, name, email, password_hash, roles, status, timezone, created_at, updated_at, is_deleted, created_by_id, last_updated_by_id
                 FROM users WHERE organization_id = ? AND is_deleted = 0",
                libsql::params![org_id],
            )
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to query users: {}", e)))?;

        let mut users = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| AgentMemError::StorageError(e.to_string()))? {
            let created_at_ts: i64 = row.get(8).map_err(|e| AgentMemError::StorageError(e.to_string()))?;
            let updated_at_ts: i64 = row.get(9).map_err(|e| AgentMemError::StorageError(e.to_string()))?;
            let is_deleted_int: i64 = row.get(10).map_err(|e| AgentMemError::StorageError(e.to_string()))?;

            // Deserialize roles from JSON
            let roles_str: String = row.get(5).map_err(|e| AgentMemError::StorageError(e.to_string()))?;
            let roles: Option<Vec<String>> = serde_json::from_str(&roles_str).ok();

            users.push(User {
                id: row.get(0).map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                organization_id: row.get(1).map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                name: row.get(2).map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                email: row.get(3).map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                password_hash: row.get(4).map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                roles,
                status: row.get(6).map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                timezone: row.get(7).map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                created_at: chrono::DateTime::from_timestamp(created_at_ts, 0)
                    .ok_or_else(|| AgentMemError::StorageError("Invalid timestamp".to_string()))?,
                updated_at: chrono::DateTime::from_timestamp(updated_at_ts, 0)
                    .ok_or_else(|| AgentMemError::StorageError("Invalid timestamp".to_string()))?,
                is_deleted: is_deleted_int != 0,
                created_by_id: row.get(11).map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                last_updated_by_id: row.get(12).map_err(|e| AgentMemError::StorageError(e.to_string()))?,
            });
        }

        Ok(users)
    }

    async fn update(&self, user: &User) -> Result<User> {
        let conn = self.conn.lock().await;

        conn.execute(
            "UPDATE users SET organization_id = ?, name = ?, status = ?, timezone = ?, updated_at = ?, last_updated_by_id = ?
             WHERE id = ? AND is_deleted = 0",
            libsql::params![
                user.organization_id.clone(),
                user.name.clone(),
                user.status.clone(),
                user.timezone.clone(),
                user.updated_at.timestamp(),
                user.last_updated_by_id.clone(),
                user.id.clone(),
            ],
        )
        .await
        .map_err(|e| AgentMemError::StorageError(format!("Failed to update user: {}", e)))?;

        Ok(user.clone())
    }

    async fn update_password(&self, user_id: &str, password_hash: &str) -> Result<()> {
        let conn = self.conn.lock().await;

        conn.execute(
            "UPDATE users SET password_hash = ?, updated_at = ? WHERE id = ? AND is_deleted = 0",
            libsql::params![password_hash, chrono::Utc::now().timestamp(), user_id],
        )
        .await
        .map_err(|e| AgentMemError::StorageError(format!("Failed to update password: {}", e)))?;

        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().await;
        
        conn.execute(
            "UPDATE users SET is_deleted = 1, updated_at = ? WHERE id = ?",
            libsql::params![chrono::Utc::now().timestamp(), id],
        )
        .await
        .map_err(|e| AgentMemError::StorageError(format!("Failed to delete user: {}", e)))?;

        Ok(())
    }

    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<User>> {
        let conn = self.conn.lock().await;

        let mut rows = conn
            .query(
                "SELECT id, organization_id, name, email, password_hash, roles, status, timezone, created_at, updated_at, is_deleted, created_by_id, last_updated_by_id
                 FROM users WHERE is_deleted = 0 ORDER BY created_at DESC LIMIT ? OFFSET ?",
                libsql::params![limit, offset],
            )
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to list users: {}", e)))?;

        let mut users = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| AgentMemError::StorageError(e.to_string()))? {
            let created_at_ts: i64 = row.get(8).map_err(|e| AgentMemError::StorageError(e.to_string()))?;
            let updated_at_ts: i64 = row.get(9).map_err(|e| AgentMemError::StorageError(e.to_string()))?;
            let is_deleted_int: i64 = row.get(10).map_err(|e| AgentMemError::StorageError(e.to_string()))?;

            // Deserialize roles from JSON
            let roles_str: String = row.get(5).map_err(|e| AgentMemError::StorageError(e.to_string()))?;
            let roles: Option<Vec<String>> = serde_json::from_str(&roles_str).ok();

            users.push(User {
                id: row.get(0).map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                organization_id: row.get(1).map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                name: row.get(2).map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                email: row.get(3).map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                password_hash: row.get(4).map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                roles,
                status: row.get(6).map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                timezone: row.get(7).map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                created_at: chrono::DateTime::from_timestamp(created_at_ts, 0)
                    .ok_or_else(|| AgentMemError::StorageError("Invalid timestamp".to_string()))?,
                updated_at: chrono::DateTime::from_timestamp(updated_at_ts, 0)
                    .ok_or_else(|| AgentMemError::StorageError("Invalid timestamp".to_string()))?,
                is_deleted: is_deleted_int != 0,
                created_by_id: row.get(11).map_err(|e| AgentMemError::StorageError(e.to_string()))?,
                last_updated_by_id: row.get(12).map_err(|e| AgentMemError::StorageError(e.to_string()))?,
            });
        }

        Ok(users)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::libsql::{connection::create_libsql_pool, migrations::run_migrations};
    use crate::storage::models::generate_id;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_user_repository_crud() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = create_libsql_pool(db_path.to_str().unwrap()).await.unwrap();
        run_migrations(conn.clone()).await.unwrap();

        let repo = LibSqlUserRepository::new(conn);

        // Create
        let user = User::new("org-123".to_string(), "Test User".to_string(), "UTC".to_string());
        let created = repo.create(&user).await.unwrap();
        assert_eq!(created.name, "Test User");

        // Find by ID
        let found = repo.find_by_id(&user.id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Test User");

        // Update
        let mut updated_user = user.clone();
        updated_user.name = "Updated User".to_string();
        repo.update(&updated_user).await.unwrap();
        let found = repo.find_by_id(&user.id).await.unwrap().unwrap();
        assert_eq!(found.name, "Updated User");

        // Delete
        repo.delete(&user.id).await.unwrap();
        let found = repo.find_by_id(&user.id).await.unwrap();
        assert!(found.is_none());
    }
}

