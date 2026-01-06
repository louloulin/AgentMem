//! Memory-specific table migrations
//!
//! This module creates specialized tables for different memory types
//! (Episodic, Semantic, Procedural, Core, Working) to support the trait-based
//! multi-backend architecture.

use crate::{CoreError, CoreResult};
use sqlx::PgPool;

/// Run all memory-specific table migrations
pub async fn run_memory_migrations(pool: &PgPool) -> CoreResult<()> {
    create_episodic_events_table(pool).await?;
    create_semantic_memory_table(pool).await?;
    create_procedural_memory_table(pool).await?;
    create_core_memory_table(pool).await?;
    create_working_memory_table(pool).await?;
    create_memory_indexes(pool).await?;

    Ok(())
}

/// Create episodic_events table for time-based events and experiences
async fn create_episodic_events_table(pool: &PgPool) -> CoreResult<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS episodic_events (
            id VARCHAR(255) PRIMARY KEY,
            organization_id VARCHAR(255) NOT NULL,
            user_id VARCHAR(255) NOT NULL,
            agent_id VARCHAR(255) NOT NULL,
            occurred_at TIMESTAMPTZ NOT NULL,
            event_type VARCHAR(100) NOT NULL,
            actor VARCHAR(255),
            summary TEXT NOT NULL,
            details TEXT,
            importance_score REAL NOT NULL DEFAULT 0.0,
            metadata JSONB NOT NULL DEFAULT '{}',
            embedding TEXT,
            expires_at TIMESTAMPTZ,
            version INTEGER NOT NULL DEFAULT 1,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| CoreError::Database(format!("Failed to create episodic_events table: {}", e)))?;

    // Add columns if they don't exist (for existing databases)
    let _ = sqlx::query("ALTER TABLE episodic_events ADD COLUMN IF NOT EXISTS embedding TEXT")
        .execute(pool)
        .await;

    let _ =
        sqlx::query("ALTER TABLE episodic_events ADD COLUMN IF NOT EXISTS expires_at TIMESTAMPTZ")
            .execute(pool)
            .await;

    let _ = sqlx::query(
        "ALTER TABLE episodic_events ADD COLUMN IF NOT EXISTS version INTEGER NOT NULL DEFAULT 1",
    )
    .execute(pool)
    .await;

    Ok(())
}

/// Create semantic_memory table for factual knowledge and concepts
async fn create_semantic_memory_table(pool: &PgPool) -> CoreResult<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS semantic_memory (
            id VARCHAR(255) PRIMARY KEY,
            organization_id VARCHAR(255) NOT NULL,
            user_id VARCHAR(255) NOT NULL,
            agent_id VARCHAR(255) NOT NULL,
            name VARCHAR(255) NOT NULL,
            summary TEXT NOT NULL,
            details TEXT,
            source VARCHAR(255),
            tree_path TEXT[] NOT NULL DEFAULT '{}',
            metadata JSONB NOT NULL DEFAULT '{}',
            embedding TEXT,
            expires_at TIMESTAMPTZ,
            version INTEGER NOT NULL DEFAULT 1,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| CoreError::Database(format!("Failed to create semantic_memory table: {}", e)))?;

    // Add columns if they don't exist (for existing databases)
    let _ = sqlx::query("ALTER TABLE semantic_memory ADD COLUMN IF NOT EXISTS embedding TEXT")
        .execute(pool)
        .await;

    let _ =
        sqlx::query("ALTER TABLE semantic_memory ADD COLUMN IF NOT EXISTS expires_at TIMESTAMPTZ")
            .execute(pool)
            .await;

    let _ = sqlx::query(
        "ALTER TABLE semantic_memory ADD COLUMN IF NOT EXISTS version INTEGER NOT NULL DEFAULT 1",
    )
    .execute(pool)
    .await;

    Ok(())
}

/// Create procedural_memory table for skills and procedures
async fn create_procedural_memory_table(pool: &PgPool) -> CoreResult<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS procedural_memory (
            id VARCHAR(255) PRIMARY KEY,
            organization_id VARCHAR(255) NOT NULL,
            user_id VARCHAR(255) NOT NULL,
            agent_id VARCHAR(255) NOT NULL,
            skill_name VARCHAR(255) NOT NULL,
            description TEXT NOT NULL,
            steps TEXT[] NOT NULL DEFAULT '{}',
            success_rate REAL NOT NULL DEFAULT 0.0,
            execution_count INTEGER NOT NULL DEFAULT 0,
            metadata JSONB NOT NULL DEFAULT '{}',
            embedding TEXT,
            expires_at TIMESTAMPTZ,
            version INTEGER NOT NULL DEFAULT 1,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| CoreError::Database(format!("Failed to create procedural_memory table: {}", e)))?;

    // Add columns if they don't exist (for existing databases)
    let _ = sqlx::query("ALTER TABLE procedural_memory ADD COLUMN IF NOT EXISTS embedding TEXT")
        .execute(pool)
        .await;

    let _ = sqlx::query(
        "ALTER TABLE procedural_memory ADD COLUMN IF NOT EXISTS expires_at TIMESTAMPTZ",
    )
    .execute(pool)
    .await;

    let _ = sqlx::query(
        "ALTER TABLE procedural_memory ADD COLUMN IF NOT EXISTS version INTEGER NOT NULL DEFAULT 1",
    )
    .execute(pool)
    .await;

    Ok(())
}

/// Create core_memory table for persistent key-value pairs
async fn create_core_memory_table(pool: &PgPool) -> CoreResult<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS core_memory (
            id VARCHAR(255) PRIMARY KEY,
            user_id VARCHAR(255) NOT NULL,
            agent_id VARCHAR(255) NOT NULL,
            key VARCHAR(255) NOT NULL,
            value TEXT NOT NULL,
            category VARCHAR(100) NOT NULL,
            is_mutable BOOLEAN NOT NULL DEFAULT TRUE,
            metadata JSONB NOT NULL DEFAULT '{}',
            embedding TEXT,
            expires_at TIMESTAMPTZ,
            version INTEGER NOT NULL DEFAULT 1,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE(user_id, agent_id, key)
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| CoreError::Database(format!("Failed to create core_memory table: {}", e)))?;

    // Add columns if they don't exist (for existing databases)
    let _ = sqlx::query("ALTER TABLE core_memory ADD COLUMN IF NOT EXISTS embedding TEXT")
        .execute(pool)
        .await;

    let _ = sqlx::query("ALTER TABLE core_memory ADD COLUMN IF NOT EXISTS expires_at TIMESTAMPTZ")
        .execute(pool)
        .await;

    let _ = sqlx::query(
        "ALTER TABLE core_memory ADD COLUMN IF NOT EXISTS version INTEGER NOT NULL DEFAULT 1",
    )
    .execute(pool)
    .await;

    Ok(())
}

/// Create working_memory table for temporary session data
async fn create_working_memory_table(pool: &PgPool) -> CoreResult<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS working_memory (
            id VARCHAR(255) PRIMARY KEY,
            user_id VARCHAR(255) NOT NULL,
            agent_id VARCHAR(255) NOT NULL,
            session_id VARCHAR(255) NOT NULL,
            content TEXT NOT NULL,
            priority INTEGER NOT NULL DEFAULT 0,
            expires_at TIMESTAMPTZ,
            metadata JSONB NOT NULL DEFAULT '{}',
            embedding TEXT,
            version INTEGER NOT NULL DEFAULT 1,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| CoreError::Database(format!("Failed to create working_memory table: {}", e)))?;

    // Add columns if they don't exist (for existing databases)
    let _ = sqlx::query("ALTER TABLE working_memory ADD COLUMN IF NOT EXISTS embedding TEXT")
        .execute(pool)
        .await;

    let _ = sqlx::query(
        "ALTER TABLE working_memory ADD COLUMN IF NOT EXISTS version INTEGER NOT NULL DEFAULT 1",
    )
    .execute(pool)
    .await;

    Ok(())
}

/// Create indexes for memory tables
async fn create_memory_indexes(pool: &PgPool) -> CoreResult<()> {
    // Episodic events indexes
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_episodic_user_occurred ON episodic_events(user_id, occurred_at DESC)")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create index: {}", e)))?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_episodic_agent_occurred ON episodic_events(agent_id, occurred_at DESC)")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create index: {}", e)))?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_episodic_event_type ON episodic_events(event_type)",
    )
    .execute(pool)
    .await
    .map_err(|e| CoreError::Database(format!("Failed to create index: {}", e)))?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_episodic_importance ON episodic_events(importance_score DESC)")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create index: {}", e)))?;

    // Semantic memory indexes
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_semantic_user_id ON semantic_memory(user_id)")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create index: {}", e)))?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_semantic_name ON semantic_memory(name)")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create index: {}", e)))?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_semantic_tree_path ON semantic_memory USING GIN(tree_path)",
    )
    .execute(pool)
    .await
    .map_err(|e| CoreError::Database(format!("Failed to create index: {}", e)))?;

    // Procedural memory indexes
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_procedural_user_id ON procedural_memory(user_id)")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create index: {}", e)))?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_procedural_skill_name ON procedural_memory(skill_name)",
    )
    .execute(pool)
    .await
    .map_err(|e| CoreError::Database(format!("Failed to create index: {}", e)))?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_procedural_success_rate ON procedural_memory(success_rate DESC)")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create index: {}", e)))?;

    // Core memory indexes
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_core_user_agent ON core_memory(user_id, agent_id)")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create index: {}", e)))?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_core_category ON core_memory(category)")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create index: {}", e)))?;

    // Working memory indexes
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_working_session ON working_memory(session_id)")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create index: {}", e)))?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_working_expires ON working_memory(expires_at)")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create index: {}", e)))?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_working_priority ON working_memory(priority DESC)")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create index: {}", e)))?;

    // Indexes for new fields (expires_at)
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_episodic_expires ON episodic_events(expires_at) WHERE expires_at IS NOT NULL")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create index: {}", e)))?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_semantic_expires ON semantic_memory(expires_at) WHERE expires_at IS NOT NULL")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create index: {}", e)))?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_procedural_expires ON procedural_memory(expires_at) WHERE expires_at IS NOT NULL")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create index: {}", e)))?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_core_expires ON core_memory(expires_at) WHERE expires_at IS NOT NULL")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create index: {}", e)))?;

    Ok(())
}
