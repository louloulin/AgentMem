//! User repository with authentication support
//!
//! This module provides database operations for users with authentication:
//! - CRUD operations
//! - Authentication (email + password)
//! - Role management
//! - Multi-tenancy support

use crate::storage::models::User;
use crate::{CoreError, CoreResult};
use chrono::Utc;
use sqlx::PgPool;

/// User with authentication information
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserAuth {
    pub id: String,
    pub organization_id: String,
    pub email: String,
    pub password_hash: String,
    pub name: String,
    pub roles: Vec<String>,
    pub status: String,
    pub timezone: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub is_deleted: bool,
    pub created_by_id: Option<String>,
    pub last_updated_by_id: Option<String>,
}

/// User repository for database operations
#[derive(Clone)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    /// Create a new user repository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new user with authentication
    pub async fn create(
        &self,
        organization_id: &str,
        email: &str,
        password_hash: &str,
        name: &str,
        roles: Vec<String>,
        created_by_id: Option<&str>,
    ) -> CoreResult<UserAuth> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        let user = sqlx::query_as::<_, UserAuth>(
            r#"
            INSERT INTO users (
                id, organization_id, email, password_hash, name, roles,
                status, timezone, created_at, updated_at, is_deleted,
                created_by_id, last_updated_by_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING *
            "#,
        )
        .bind(&id)
        .bind(organization_id)
        .bind(email)
        .bind(password_hash)
        .bind(name)
        .bind(&roles)
        .bind("active")
        .bind("UTC")
        .bind(now)
        .bind(now)
        .bind(false)
        .bind(created_by_id)
        .bind(created_by_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create user: {}", e)))?;

        Ok(user)
    }

    /// Find user by ID
    pub async fn find_by_id(&self, id: &str) -> CoreResult<Option<UserAuth>> {
        let user = sqlx::query_as::<_, UserAuth>(
            r#"
            SELECT * FROM users
            WHERE id = $1 AND is_deleted = false
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to find user: {}", e)))?;

        Ok(user)
    }

    /// Find user by email
    pub async fn find_by_email(&self, email: &str) -> CoreResult<Option<UserAuth>> {
        let user = sqlx::query_as::<_, UserAuth>(
            r#"
            SELECT * FROM users
            WHERE email = $1 AND is_deleted = false
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to find user by email: {}", e)))?;

        Ok(user)
    }

    /// Find user by email and organization
    pub async fn find_by_email_and_org(
        &self,
        email: &str,
        organization_id: &str,
    ) -> CoreResult<Option<UserAuth>> {
        let user = sqlx::query_as::<_, UserAuth>(
            r#"
            SELECT * FROM users
            WHERE email = $1 AND organization_id = $2 AND is_deleted = false
            "#,
        )
        .bind(email)
        .bind(organization_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to find user: {}", e)))?;

        Ok(user)
    }

    /// List users by organization
    pub async fn list_by_organization(&self, organization_id: &str) -> CoreResult<Vec<UserAuth>> {
        let users = sqlx::query_as::<_, UserAuth>(
            r#"
            SELECT * FROM users
            WHERE organization_id = $1 AND is_deleted = false
            ORDER BY created_at DESC
            "#,
        )
        .bind(organization_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to list users: {}", e)))?;

        Ok(users)
    }

    /// Update user
    pub async fn update(
        &self,
        id: &str,
        name: Option<&str>,
        email: Option<&str>,
        roles: Option<Vec<String>>,
        status: Option<&str>,
        updated_by_id: Option<&str>,
    ) -> CoreResult<UserAuth> {
        let now = Utc::now();

        // Build dynamic update query
        let mut query = String::from("UPDATE users SET updated_at = $1, last_updated_by_id = $2");
        let mut param_count = 3;
        let mut params: Vec<String> = vec![];

        if name.is_some() {
            query.push_str(&format!(", name = ${}", param_count));
            param_count += 1;
        }
        if email.is_some() {
            query.push_str(&format!(", email = ${}", param_count));
            param_count += 1;
        }
        if roles.is_some() {
            query.push_str(&format!(", roles = ${}", param_count));
            param_count += 1;
        }
        if status.is_some() {
            query.push_str(&format!(", status = ${}", param_count));
            param_count += 1;
        }

        query.push_str(&format!(
            " WHERE id = ${} AND is_deleted = false RETURNING *",
            param_count
        ));

        // Execute query with dynamic parameters
        let mut q = sqlx::query_as::<_, UserAuth>(&query)
            .bind(now)
            .bind(updated_by_id);

        if let Some(n) = name {
            q = q.bind(n);
        }
        if let Some(e) = email {
            q = q.bind(e);
        }
        if let Some(r) = roles {
            q = q.bind(r);
        }
        if let Some(s) = status {
            q = q.bind(s);
        }

        q = q.bind(id);

        let user = q
            .fetch_one(&self.pool)
            .await
            .map_err(|e| CoreError::Database(format!("Failed to update user: {}", e)))?;

        Ok(user)
    }

    /// Update password
    pub async fn update_password(
        &self,
        id: &str,
        password_hash: &str,
        updated_by_id: Option<&str>,
    ) -> CoreResult<()> {
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE users
            SET password_hash = $1, updated_at = $2, last_updated_by_id = $3
            WHERE id = $4 AND is_deleted = false
            "#,
        )
        .bind(password_hash)
        .bind(now)
        .bind(updated_by_id)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to update password: {}", e)))?;

        Ok(())
    }

    /// Soft delete user
    pub async fn delete(&self, id: &str, deleted_by_id: Option<&str>) -> CoreResult<()> {
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE users
            SET is_deleted = true, updated_at = $1, last_updated_by_id = $2
            WHERE id = $3
            "#,
        )
        .bind(now)
        .bind(deleted_by_id)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to delete user: {}", e)))?;

        Ok(())
    }

    /// Hard delete user (for testing)
    pub async fn hard_delete(&self, id: &str) -> CoreResult<()> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| CoreError::Database(format!("Failed to hard delete user: {}", e)))?;

        Ok(())
    }

    /// Check if email exists
    pub async fn email_exists(&self, email: &str, organization_id: &str) -> CoreResult<bool> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM users
            WHERE email = $1 AND organization_id = $2 AND is_deleted = false
            "#,
        )
        .bind(email)
        .bind(organization_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to check email: {}", e)))?;

        Ok(count.0 > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a running PostgreSQL database
    // Run with: cargo test --package agent-mem-core --lib storage::user_repository --features test-db

    #[tokio::test]
    #[ignore] // Requires database
    async fn test_user_repository() {
        // This is a placeholder for integration tests
        // Actual tests would require database setup
    }
}
