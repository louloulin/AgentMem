//! LibSQL Organization Repository
//!
//! Provides LibSQL implementation of OrganizationRepositoryTrait

use agent_mem_traits::{AgentMemError, Result};
use async_trait::async_trait;
use libsql::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::storage::models::Organization;
use crate::storage::traits::OrganizationRepositoryTrait;

/// LibSQL implementation of Organization repository
pub struct LibSqlOrganizationRepository {
    conn: Arc<Mutex<Connection>>,
}

impl LibSqlOrganizationRepository {
    /// Create a new LibSQL organization repository
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl OrganizationRepositoryTrait for LibSqlOrganizationRepository {
    async fn create(&self, org: &Organization) -> Result<Organization> {
        let conn = self.conn.lock().await;

        conn.execute(
            "INSERT INTO organizations (id, name, created_at, updated_at, is_deleted) 
             VALUES (?, ?, ?, ?, ?)",
            libsql::params![
                org.id.clone(),
                org.name.clone(),
                org.created_at.timestamp(),
                org.updated_at.timestamp(),
                if org.is_deleted { 1 } else { 0 },
            ],
        )
        .await
        .map_err(|e| AgentMemError::StorageError(format!("Failed to create organization: {e}")))?;

        Ok(org.clone())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<Organization>> {
        let conn = self.conn.lock().await;

        let mut stmt = conn
            .prepare("SELECT id, name, created_at, updated_at, is_deleted FROM organizations WHERE id = ? AND is_deleted = 0")
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to prepare statement: {e}")))?;

        let mut rows = stmt
            .query(libsql::params![id])
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to query organization: {e}")))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to fetch row: {e}")))?
        {
            let org = Organization {
                id: row
                    .get::<String>(0)
                    .map_err(|e| AgentMemError::StorageError(format!("Failed to get id: {e}")))?,
                name: row
                    .get::<String>(1)
                    .map_err(|e| AgentMemError::StorageError(format!("Failed to get name: {e}")))?,
                created_at: chrono::DateTime::from_timestamp(
                    row.get::<i64>(2)
                        .map_err(|e| AgentMemError::StorageError(format!("Failed to get created_at: {e}")))?,
                    0,
                )
                .ok_or_else(|| AgentMemError::StorageError("Invalid created_at timestamp".to_string()))?,
                updated_at: chrono::DateTime::from_timestamp(
                    row.get::<i64>(3)
                        .map_err(|e| AgentMemError::StorageError(format!("Failed to get updated_at: {e}")))?,
                    0,
                )
                .ok_or_else(|| AgentMemError::StorageError("Invalid updated_at timestamp".to_string()))?,
                is_deleted: row
                    .get::<i64>(4)
                    .map_err(|e| AgentMemError::StorageError(format!("Failed to get is_deleted: {e}")))?
                    != 0,
            };
            Ok(Some(org))
        } else {
            Ok(None)
        }
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<Organization>> {
        let conn = self.conn.lock().await;

        let mut stmt = conn
            .prepare("SELECT id, name, created_at, updated_at, is_deleted FROM organizations WHERE name = ? AND is_deleted = 0")
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to prepare statement: {e}")))?;

        let mut rows = stmt
            .query(libsql::params![name])
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to query organization: {e}")))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to fetch row: {e}")))?
        {
            let org = Organization {
                id: row
                    .get::<String>(0)
                    .map_err(|e| AgentMemError::StorageError(format!("Failed to get id: {e}")))?,
                name: row
                    .get::<String>(1)
                    .map_err(|e| AgentMemError::StorageError(format!("Failed to get name: {e}")))?,
                created_at: chrono::DateTime::from_timestamp(
                    row.get::<i64>(2)
                        .map_err(|e| AgentMemError::StorageError(format!("Failed to get created_at: {e}")))?,
                    0,
                )
                .ok_or_else(|| AgentMemError::StorageError("Invalid created_at timestamp".to_string()))?,
                updated_at: chrono::DateTime::from_timestamp(
                    row.get::<i64>(3)
                        .map_err(|e| AgentMemError::StorageError(format!("Failed to get updated_at: {e}")))?,
                    0,
                )
                .ok_or_else(|| AgentMemError::StorageError("Invalid updated_at timestamp".to_string()))?,
                is_deleted: row
                    .get::<i64>(4)
                    .map_err(|e| AgentMemError::StorageError(format!("Failed to get is_deleted: {e}")))?
                    != 0,
            };
            Ok(Some(org))
        } else {
            Ok(None)
        }
    }

    async fn update(&self, org: &Organization) -> Result<Organization> {
        let conn = self.conn.lock().await;

        conn.execute(
            "UPDATE organizations SET name = ?, updated_at = ? WHERE id = ? AND is_deleted = 0",
            libsql::params![
                org.name.clone(),
                org.updated_at.timestamp(),
                org.id.clone(),
            ],
        )
        .await
        .map_err(|e| AgentMemError::StorageError(format!("Failed to update organization: {e}")))?;

        Ok(org.clone())
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().await;

        conn.execute(
            "UPDATE organizations SET is_deleted = 1 WHERE id = ?",
            libsql::params![id],
        )
        .await
        .map_err(|e| AgentMemError::StorageError(format!("Failed to delete organization: {e}")))?;

        Ok(())
    }

    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Organization>> {
        let conn = self.conn.lock().await;

        let mut stmt = conn
            .prepare("SELECT id, name, created_at, updated_at, is_deleted FROM organizations WHERE is_deleted = 0 ORDER BY created_at DESC LIMIT ? OFFSET ?")
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to prepare statement: {e}")))?;

        let mut rows = stmt
            .query(libsql::params![limit, offset])
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to query organizations: {e}")))?;

        let mut organizations = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to fetch row: {e}")))?
        {
            let org = Organization {
                id: row
                    .get::<String>(0)
                    .map_err(|e| AgentMemError::StorageError(format!("Failed to get id: {e}")))?,
                name: row
                    .get::<String>(1)
                    .map_err(|e| AgentMemError::StorageError(format!("Failed to get name: {e}")))?,
                created_at: chrono::DateTime::from_timestamp(
                    row.get::<i64>(2)
                        .map_err(|e| AgentMemError::StorageError(format!("Failed to get created_at: {e}")))?,
                    0,
                )
                .ok_or_else(|| AgentMemError::StorageError("Invalid created_at timestamp".to_string()))?,
                updated_at: chrono::DateTime::from_timestamp(
                    row.get::<i64>(3)
                        .map_err(|e| AgentMemError::StorageError(format!("Failed to get updated_at: {e}")))?,
                    0,
                )
                .ok_or_else(|| AgentMemError::StorageError("Invalid updated_at timestamp".to_string()))?,
                is_deleted: row
                    .get::<i64>(4)
                    .map_err(|e| AgentMemError::StorageError(format!("Failed to get is_deleted: {e}")))?
                    != 0,
            };
            organizations.push(org);
        }

        Ok(organizations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::libsql::{create_libsql_pool, run_migrations};
    use crate::storage::models::Organization;
    use tempfile::TempDir;

    async fn setup_test_db() -> (TempDir, Arc<Mutex<Connection>>) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = create_libsql_pool(db_path.to_str().unwrap())
            .await
            .unwrap();
        run_migrations(conn.clone()).await.unwrap();
        (temp_dir, conn)
    }

    #[tokio::test]
    async fn test_organization_crud() {
        let (_temp_dir, conn) = setup_test_db().await;
        let repo = LibSqlOrganizationRepository::new(conn);

        // Create
        let org = Organization::new("Test Org".to_string());
        let created = repo.create(&org).await.unwrap();
        assert_eq!(created.name, "Test Org");

        // Find by ID
        let found = repo.find_by_id(&created.id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Test Org");

        // Find by name
        let found_by_name = repo.find_by_name("Test Org").await.unwrap();
        assert!(found_by_name.is_some());

        // Update
        let mut updated_org = created.clone();
        updated_org.name = "Updated Org".to_string();
        let updated = repo.update(&updated_org).await.unwrap();
        assert_eq!(updated.name, "Updated Org");

        // List
        let orgs = repo.list(10, 0).await.unwrap();
        assert_eq!(orgs.len(), 1);

        // Delete
        repo.delete(&created.id).await.unwrap();
        let deleted = repo.find_by_id(&created.id).await.unwrap();
        assert!(deleted.is_none());
    }
}

