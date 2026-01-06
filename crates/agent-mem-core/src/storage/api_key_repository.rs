//! API Key repository for authentication
//!
//! This module provides database operations for API keys:
//! - CRUD operations
//! - Key validation
//! - Scope management
//! - Usage tracking

use crate::{CoreError, CoreResult};
use chrono::{DateTime, Utc};
use sqlx::PgPool;

/// API Key model
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ApiKeyModel {
    pub id: String,
    pub key_hash: String, // Store hash, not the actual key
    pub name: String,
    pub user_id: String,
    pub organization_id: String,
    pub scopes: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub is_deleted: bool,
}

/// API Key repository
#[derive(Clone)]
pub struct ApiKeyRepository {
    pool: PgPool,
}

impl ApiKeyRepository {
    /// Create a new API key repository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new API key
    pub async fn create(
        &self,
        key_hash: &str,
        name: &str,
        user_id: &str,
        organization_id: &str,
        scopes: Vec<String>,
        expires_at: Option<DateTime<Utc>>,
    ) -> CoreResult<ApiKeyModel> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        let api_key = sqlx::query_as::<_, ApiKeyModel>(
            r#"
            INSERT INTO api_keys (
                id, key_hash, name, user_id, organization_id, scopes,
                created_at, expires_at, last_used_at, is_active, is_deleted
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
        )
        .bind(&id)
        .bind(key_hash)
        .bind(name)
        .bind(user_id)
        .bind(organization_id)
        .bind(&scopes)
        .bind(now)
        .bind(expires_at)
        .bind(None::<DateTime<Utc>>)
        .bind(true)
        .bind(false)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create API key: {}", e)))?;

        Ok(api_key)
    }

    /// Find API key by ID
    pub async fn find_by_id(&self, id: &str) -> CoreResult<Option<ApiKeyModel>> {
        let api_key = sqlx::query_as::<_, ApiKeyModel>(
            r#"
            SELECT * FROM api_keys
            WHERE id = $1 AND is_deleted = false
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to find API key: {}", e)))?;

        Ok(api_key)
    }

    /// Find API key by key hash
    pub async fn find_by_key_hash(&self, key_hash: &str) -> CoreResult<Option<ApiKeyModel>> {
        let api_key = sqlx::query_as::<_, ApiKeyModel>(
            r#"
            SELECT * FROM api_keys
            WHERE key_hash = $1 AND is_deleted = false AND is_active = true
            "#,
        )
        .bind(key_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to find API key: {}", e)))?;

        Ok(api_key)
    }

    /// List API keys by user
    pub async fn list_by_user(&self, user_id: &str) -> CoreResult<Vec<ApiKeyModel>> {
        let api_keys = sqlx::query_as::<_, ApiKeyModel>(
            r#"
            SELECT * FROM api_keys
            WHERE user_id = $1 AND is_deleted = false
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to list API keys: {}", e)))?;

        Ok(api_keys)
    }

    /// List API keys by organization
    pub async fn list_by_organization(
        &self,
        organization_id: &str,
    ) -> CoreResult<Vec<ApiKeyModel>> {
        let api_keys = sqlx::query_as::<_, ApiKeyModel>(
            r#"
            SELECT * FROM api_keys
            WHERE organization_id = $1 AND is_deleted = false
            ORDER BY created_at DESC
            "#,
        )
        .bind(organization_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to list API keys: {}", e)))?;

        Ok(api_keys)
    }

    /// Update API key
    pub async fn update(
        &self,
        id: &str,
        name: Option<&str>,
        scopes: Option<Vec<String>>,
        is_active: Option<bool>,
    ) -> CoreResult<ApiKeyModel> {
        // Build dynamic update query
        let mut query = String::from("UPDATE api_keys SET");
        let mut updates = vec![];
        let mut param_count = 1;

        if name.is_some() {
            updates.push(format!(" name = ${}", param_count));
            param_count += 1;
        }
        if scopes.is_some() {
            updates.push(format!(" scopes = ${}", param_count));
            param_count += 1;
        }
        if is_active.is_some() {
            updates.push(format!(" is_active = ${}", param_count));
            param_count += 1;
        }

        if updates.is_empty() {
            return Err(CoreError::InvalidInput("No fields to update".to_string()));
        }

        query.push_str(&updates.join(","));
        query.push_str(&format!(
            " WHERE id = ${} AND is_deleted = false RETURNING *",
            param_count
        ));

        let mut q = sqlx::query_as::<_, ApiKeyModel>(&query);

        if let Some(n) = name {
            q = q.bind(n);
        }
        if let Some(s) = scopes {
            q = q.bind(s);
        }
        if let Some(a) = is_active {
            q = q.bind(a);
        }

        q = q.bind(id);

        let api_key = q
            .fetch_one(&self.pool)
            .await
            .map_err(|e| CoreError::Database(format!("Failed to update API key: {}", e)))?;

        Ok(api_key)
    }

    /// Update last used timestamp
    pub async fn update_last_used(&self, id: &str) -> CoreResult<()> {
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE api_keys
            SET last_used_at = $1
            WHERE id = $2
            "#,
        )
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to update last used: {}", e)))?;

        Ok(())
    }

    /// Revoke API key (soft delete)
    pub async fn revoke(&self, id: &str) -> CoreResult<()> {
        sqlx::query(
            r#"
            UPDATE api_keys
            SET is_active = false, is_deleted = true
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to revoke API key: {}", e)))?;

        Ok(())
    }

    /// Hard delete API key (for testing)
    pub async fn hard_delete(&self, id: &str) -> CoreResult<()> {
        sqlx::query("DELETE FROM api_keys WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| CoreError::Database(format!("Failed to hard delete API key: {}", e)))?;

        Ok(())
    }

    /// Validate API key (check if active and not expired)
    pub async fn validate(&self, key_hash: &str) -> CoreResult<Option<ApiKeyModel>> {
        let api_key = self.find_by_key_hash(key_hash).await?;

        if let Some(key) = api_key {
            // Check if expired
            if let Some(expires_at) = key.expires_at {
                if Utc::now() > expires_at {
                    return Ok(None);
                }
            }

            // Update last used
            self.update_last_used(&key.id).await?;

            Ok(Some(key))
        } else {
            Ok(None)
        }
    }

    /// Check if API key has scope
    pub fn has_scope(api_key: &ApiKeyModel, scope: &str) -> bool {
        api_key.scopes.contains(&scope.to_string()) || api_key.scopes.contains(&"*".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires database
    async fn test_api_key_repository() {
        // Placeholder for integration tests
    }
}
