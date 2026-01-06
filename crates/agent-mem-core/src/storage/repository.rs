//! Repository pattern implementation for database access
//!
//! This module provides a clean abstraction layer for database operations,
//! following the Repository pattern from MIRIX's design.

use async_trait::async_trait;
use chrono::Utc;
use sqlx::{PgPool, Row};

use super::models::*;
use crate::{CoreError, CoreResult};

/// Repository trait for CRUD operations
#[async_trait]
pub trait Repository<T> {
    /// Create a new entity
    async fn create(&self, entity: &T) -> CoreResult<T>;

    /// Read an entity by ID
    async fn read(&self, id: &str) -> CoreResult<Option<T>>;

    /// Update an existing entity
    async fn update(&self, entity: &T) -> CoreResult<T>;

    /// Delete an entity by ID (soft delete)
    async fn delete(&self, id: &str) -> CoreResult<bool>;

    /// Hard delete an entity by ID
    async fn hard_delete(&self, id: &str) -> CoreResult<bool>;

    /// List entities with pagination
    async fn list(&self, limit: Option<i64>, offset: Option<i64>) -> CoreResult<Vec<T>>;

    /// Count total entities
    async fn count(&self) -> CoreResult<i64>;
}

/// Organization repository
#[derive(Clone)]
pub struct OrganizationRepository {
    pool: PgPool,
}

impl OrganizationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<Organization> for OrganizationRepository {
    async fn create(&self, org: &Organization) -> CoreResult<Organization> {
        let result = sqlx::query_as::<_, Organization>(
            r#"
            INSERT INTO organizations (id, name, created_at, updated_at, is_deleted)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(&org.id)
        .bind(&org.name)
        .bind(&org.created_at)
        .bind(&org.updated_at)
        .bind(&org.is_deleted)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create organization: {}", e)))?;

        Ok(result)
    }

    async fn read(&self, id: &str) -> CoreResult<Option<Organization>> {
        let result = sqlx::query_as::<_, Organization>(
            r#"
            SELECT * FROM organizations
            WHERE id = $1 AND is_deleted = FALSE
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to read organization: {}", e)))?;

        Ok(result)
    }

    async fn update(&self, org: &Organization) -> CoreResult<Organization> {
        let result = sqlx::query_as::<_, Organization>(
            r#"
            UPDATE organizations
            SET name = $2, updated_at = $3
            WHERE id = $1 AND is_deleted = FALSE
            RETURNING *
            "#,
        )
        .bind(&org.id)
        .bind(&org.name)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to update organization: {}", e)))?;

        Ok(result)
    }

    async fn delete(&self, id: &str) -> CoreResult<bool> {
        let result = sqlx::query(
            r#"
            UPDATE organizations
            SET is_deleted = TRUE, updated_at = $2
            WHERE id = $1 AND is_deleted = FALSE
            "#,
        )
        .bind(id)
        .bind(Utc::now())
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to delete organization: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    async fn hard_delete(&self, id: &str) -> CoreResult<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM organizations WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to hard delete organization: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    async fn list(&self, limit: Option<i64>, offset: Option<i64>) -> CoreResult<Vec<Organization>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        let results = sqlx::query_as::<_, Organization>(
            r#"
            SELECT * FROM organizations
            WHERE is_deleted = FALSE
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to list organizations: {}", e)))?;

        Ok(results)
    }

    async fn count(&self) -> CoreResult<i64> {
        let result = sqlx::query(
            r#"
            SELECT COUNT(*) as count FROM organizations
            WHERE is_deleted = FALSE
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to count organizations: {}", e)))?;

        Ok(result.get("count"))
    }
}

/// User repository
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// List users by organization
    pub async fn list_by_organization(
        &self,
        organization_id: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> CoreResult<Vec<User>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        let results = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users
            WHERE organization_id = $1 AND is_deleted = FALSE
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(organization_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to list users: {}", e)))?;

        Ok(results)
    }
}

#[async_trait]
impl Repository<User> for UserRepository {
    async fn create(&self, user: &User) -> CoreResult<User> {
        let result = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (
                id, organization_id, name, status, timezone,
                created_at, updated_at, is_deleted,
                created_by_id, last_updated_by_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(&user.id)
        .bind(&user.organization_id)
        .bind(&user.name)
        .bind(&user.status)
        .bind(&user.timezone)
        .bind(&user.created_at)
        .bind(&user.updated_at)
        .bind(&user.is_deleted)
        .bind(&user.created_by_id)
        .bind(&user.last_updated_by_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create user: {}", e)))?;

        Ok(result)
    }

    async fn read(&self, id: &str) -> CoreResult<Option<User>> {
        let result = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users
            WHERE id = $1 AND is_deleted = FALSE
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to read user: {}", e)))?;

        Ok(result)
    }

    async fn update(&self, user: &User) -> CoreResult<User> {
        let result = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET name = $2, status = $3, timezone = $4,
                updated_at = $5, last_updated_by_id = $6
            WHERE id = $1 AND is_deleted = FALSE
            RETURNING *
            "#,
        )
        .bind(&user.id)
        .bind(&user.name)
        .bind(&user.status)
        .bind(&user.timezone)
        .bind(Utc::now())
        .bind(&user.last_updated_by_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to update user: {}", e)))?;

        Ok(result)
    }

    async fn delete(&self, id: &str) -> CoreResult<bool> {
        let result = sqlx::query(
            r#"
            UPDATE users
            SET is_deleted = TRUE, updated_at = $2
            WHERE id = $1 AND is_deleted = FALSE
            "#,
        )
        .bind(id)
        .bind(Utc::now())
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to delete user: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    async fn hard_delete(&self, id: &str) -> CoreResult<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM users WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to hard delete user: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    async fn list(&self, limit: Option<i64>, offset: Option<i64>) -> CoreResult<Vec<User>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        let results = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users
            WHERE is_deleted = FALSE
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to list users: {}", e)))?;

        Ok(results)
    }

    async fn count(&self) -> CoreResult<i64> {
        let result = sqlx::query(
            r#"
            SELECT COUNT(*) as count FROM users
            WHERE is_deleted = FALSE
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to count users: {}", e)))?;

        Ok(result.get("count"))
    }
}
