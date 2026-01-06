//! Database migration version management system
//!
//! This module provides a production-grade migration management system with:
//! - Version tracking
//! - Up/down migrations
//! - Migration history
//! - Rollback support

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};

use crate::{CoreError, CoreResult};

/// Migration version information
#[derive(Debug, Clone)]
pub struct MigrationVersion {
    pub version: i32,
    pub name: String,
    pub applied_at: DateTime<Utc>,
    pub checksum: String,
}

/// Migration manager for version control
pub struct MigrationManager {
    pool: PgPool,
}

impl MigrationManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Initialize migration tracking table
    pub async fn init(&self) -> CoreResult<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                checksum VARCHAR(64) NOT NULL,
                execution_time_ms BIGINT
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create migrations table: {}", e)))?;

        Ok(())
    }

    /// Get current schema version
    pub async fn current_version(&self) -> CoreResult<Option<i32>> {
        let row = sqlx::query("SELECT MAX(version) as version FROM schema_migrations")
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| CoreError::Database(format!("Failed to get current version: {}", e)))?;

        Ok(row.and_then(|r| r.try_get("version").ok()))
    }

    /// Get all applied migrations
    pub async fn applied_migrations(&self) -> CoreResult<Vec<MigrationVersion>> {
        let rows = sqlx::query(
            r#"
            SELECT version, name, applied_at, checksum
            FROM schema_migrations
            ORDER BY version ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to get applied migrations: {}", e)))?;

        let migrations = rows
            .into_iter()
            .map(|row| MigrationVersion {
                version: row.get("version"),
                name: row.get("name"),
                applied_at: row.get("applied_at"),
                checksum: row.get("checksum"),
            })
            .collect();

        Ok(migrations)
    }

    /// Check if a migration version is applied
    pub async fn is_applied(&self, version: i32) -> CoreResult<bool> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM schema_migrations WHERE version = $1")
            .bind(version)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| CoreError::Database(format!("Failed to check migration: {}", e)))?;

        let count: i64 = row.get("count");
        Ok(count > 0)
    }

    /// Record a migration as applied
    pub async fn record_migration(
        &self,
        version: i32,
        name: &str,
        checksum: &str,
        execution_time_ms: i64,
    ) -> CoreResult<()> {
        sqlx::query(
            r#"
            INSERT INTO schema_migrations (version, name, checksum, execution_time_ms)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(version)
        .bind(name)
        .bind(checksum)
        .bind(execution_time_ms)
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to record migration: {}", e)))?;

        Ok(())
    }

    /// Remove a migration record (for rollback)
    pub async fn remove_migration(&self, version: i32) -> CoreResult<()> {
        sqlx::query("DELETE FROM schema_migrations WHERE version = $1")
            .bind(version)
            .execute(&self.pool)
            .await
            .map_err(|e| CoreError::Database(format!("Failed to remove migration: {}", e)))?;

        Ok(())
    }

    /// Get pending migrations
    pub async fn pending_migrations(&self) -> CoreResult<Vec<i32>> {
        let current = self.current_version().await?.unwrap_or(0);
        let all_versions = self.all_migration_versions();

        let pending: Vec<i32> = all_versions.into_iter().filter(|v| *v > current).collect();

        Ok(pending)
    }

    /// Get all available migration versions
    fn all_migration_versions(&self) -> Vec<i32> {
        // Return all migration versions in order
        // Version 1: Initial schema (organizations, users, agents, messages, blocks, tools, memories)
        vec![1]
    }

    /// Run a specific migration
    pub async fn run_migration(&self, version: i32) -> CoreResult<()> {
        if self.is_applied(version).await? {
            tracing::info!("Migration {} already applied, skipping", version);
            return Ok(());
        }

        let start = std::time::Instant::now();

        match version {
            1 => self.run_migration_v1().await?,
            _ => {
                return Err(CoreError::MigrationError(format!(
                    "Unknown migration version: {}",
                    version
                )))
            }
        }

        let execution_time = start.elapsed().as_millis() as i64;
        let checksum = self.calculate_checksum(version);

        self.record_migration(
            version,
            &self.migration_name(version),
            &checksum,
            execution_time,
        )
        .await?;

        tracing::info!(
            "Migration {} applied successfully in {}ms",
            version,
            execution_time
        );

        Ok(())
    }

    /// Rollback a specific migration
    pub async fn rollback_migration(&self, version: i32) -> CoreResult<()> {
        if !self.is_applied(version).await? {
            tracing::info!("Migration {} not applied, skipping rollback", version);
            return Ok(());
        }

        match version {
            1 => self.rollback_migration_v1().await?,
            _ => {
                return Err(CoreError::MigrationError(format!(
                    "Unknown migration version: {}",
                    version
                )))
            }
        }

        self.remove_migration(version).await?;

        tracing::info!("Migration {} rolled back successfully", version);

        Ok(())
    }

    /// Run all pending migrations
    pub async fn migrate_up(&self) -> CoreResult<()> {
        self.init().await?;

        let pending = self.pending_migrations().await?;

        if pending.is_empty() {
            tracing::info!("No pending migrations");
            return Ok(());
        }

        tracing::info!("Running {} pending migrations", pending.len());

        for version in pending {
            self.run_migration(version).await?;
        }

        Ok(())
    }

    /// Rollback N migrations
    pub async fn migrate_down(&self, steps: usize) -> CoreResult<()> {
        let applied = self.applied_migrations().await?;

        if applied.is_empty() {
            tracing::info!("No migrations to rollback");
            return Ok(());
        }

        let to_rollback: Vec<i32> = applied
            .iter()
            .rev()
            .take(steps)
            .map(|m| m.version)
            .collect();

        tracing::info!("Rolling back {} migrations", to_rollback.len());

        for version in to_rollback {
            self.rollback_migration(version).await?;
        }

        Ok(())
    }

    /// Get migration name
    fn migration_name(&self, version: i32) -> String {
        match version {
            1 => "initial_schema".to_string(),
            _ => format!("migration_{}", version),
        }
    }

    /// Calculate migration checksum
    fn calculate_checksum(&self, version: i32) -> String {
        // Simple checksum based on version and name
        // In production, this should hash the actual migration SQL
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        version.hash(&mut hasher);
        self.migration_name(version).hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Migration V1: Initial schema
    async fn run_migration_v1(&self) -> CoreResult<()> {
        // Use the existing migrations module
        super::migrations::run_migrations(&self.pool).await
    }

    /// Rollback Migration V1
    async fn rollback_migration_v1(&self) -> CoreResult<()> {
        // Drop all tables in reverse order
        let tables = vec![
            "tools_agents",
            "blocks_agents",
            "memories",
            "tools",
            "blocks",
            "messages",
            "agents",
            "users",
            "organizations",
        ];

        for table in tables {
            sqlx::query(&format!("DROP TABLE IF EXISTS {} CASCADE", table))
                .execute(&self.pool)
                .await
                .map_err(|e| {
                    CoreError::Database(format!("Failed to drop table {}: {}", table, e))
                })?;
        }

        Ok(())
    }
}
