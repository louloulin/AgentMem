//! Database migrations for AgentMem
//!
//! This module contains all database migration logic, creating tables and indexes
//! based on MIRIX's schema design.

use sqlx::PgPool;

use crate::{CoreError, CoreResult};

/// Run all database migrations
pub async fn run_migrations(pool: &PgPool) -> CoreResult<()> {
    // Run migrations in order
    create_organizations_table(pool).await?;
    create_users_table(pool).await?;
    create_agents_table(pool).await?;
    create_messages_table(pool).await?;
    create_blocks_table(pool).await?;
    create_tools_table(pool).await?;
    create_memories_table(pool).await?;
    create_junction_tables(pool).await?;
    create_indexes(pool).await?;

    Ok(())
}

/// Create organizations table
async fn create_organizations_table(pool: &PgPool) -> CoreResult<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS organizations (
            id VARCHAR(255) PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            is_deleted BOOLEAN NOT NULL DEFAULT FALSE
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| CoreError::Database(format!("Failed to create organizations table: {}", e)))?;

    Ok(())
}

/// Create users table
async fn create_users_table(pool: &PgPool) -> CoreResult<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id VARCHAR(255) PRIMARY KEY,
            organization_id VARCHAR(255) NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
            name VARCHAR(255) NOT NULL,
            status VARCHAR(50) NOT NULL DEFAULT 'active',
            timezone VARCHAR(100) NOT NULL DEFAULT 'UTC',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
            created_by_id VARCHAR(255),
            last_updated_by_id VARCHAR(255)
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| CoreError::Database(format!("Failed to create users table: {}", e)))?;

    Ok(())
}

/// Create agents table
async fn create_agents_table(pool: &PgPool) -> CoreResult<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS agents (
            id VARCHAR(255) PRIMARY KEY,
            organization_id VARCHAR(255) NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
            agent_type VARCHAR(100),
            name VARCHAR(255),
            description TEXT,
            system TEXT,
            topic TEXT,
            message_ids JSONB,
            metadata_ JSONB,
            llm_config JSONB,
            embedding_config JSONB,
            tool_rules JSONB,
            mcp_tools JSONB,
            state VARCHAR(50) DEFAULT 'idle',
            last_active_at TIMESTAMPTZ,
            error_message TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
            created_by_id VARCHAR(255),
            last_updated_by_id VARCHAR(255)
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| CoreError::Database(format!("Failed to create agents table: {}", e)))?;

    Ok(())
}

/// Create messages table
async fn create_messages_table(pool: &PgPool) -> CoreResult<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS messages (
            id VARCHAR(255) PRIMARY KEY,
            organization_id VARCHAR(255) NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
            user_id VARCHAR(255) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            agent_id VARCHAR(255) NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
            role VARCHAR(50) NOT NULL,
            text TEXT,
            content JSONB,
            model VARCHAR(255),
            name VARCHAR(255),
            tool_calls JSONB,
            tool_call_id VARCHAR(255),
            step_id VARCHAR(255),
            otid VARCHAR(255),
            tool_returns JSONB,
            group_id VARCHAR(255),
            sender_id VARCHAR(255),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
            created_by_id VARCHAR(255),
            last_updated_by_id VARCHAR(255)
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| CoreError::Database(format!("Failed to create messages table: {}", e)))?;

    Ok(())
}

/// Create blocks table
async fn create_blocks_table(pool: &PgPool) -> CoreResult<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS blocks (
            id VARCHAR(255) PRIMARY KEY,
            organization_id VARCHAR(255) NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
            user_id VARCHAR(255) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            template_name VARCHAR(255),
            description TEXT,
            label VARCHAR(50) NOT NULL,
            is_template BOOLEAN NOT NULL DEFAULT FALSE,
            value TEXT NOT NULL,
            "limit" BIGINT NOT NULL DEFAULT 2000,
            metadata_ JSONB,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
            created_by_id VARCHAR(255),
            last_updated_by_id VARCHAR(255),
            CONSTRAINT unique_block_id_label UNIQUE (id, label)
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| CoreError::Database(format!("Failed to create blocks table: {}", e)))?;

    Ok(())
}

/// Create tools table
async fn create_tools_table(pool: &PgPool) -> CoreResult<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tools (
            id VARCHAR(255) PRIMARY KEY,
            organization_id VARCHAR(255) NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
            name VARCHAR(255) NOT NULL,
            description TEXT,
            json_schema JSONB,
            source_type VARCHAR(100),
            source_code TEXT,
            tags JSONB,
            metadata_ JSONB,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
            created_by_id VARCHAR(255),
            last_updated_by_id VARCHAR(255)
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| CoreError::Database(format!("Failed to create tools table: {}", e)))?;

    Ok(())
}

/// Create memories table (enhanced version)
async fn create_memories_table(pool: &PgPool) -> CoreResult<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS memories (
            id VARCHAR(255) PRIMARY KEY,
            organization_id VARCHAR(255) NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
            user_id VARCHAR(255) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            agent_id VARCHAR(255) NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
            content TEXT NOT NULL,
            hash VARCHAR(64),
            metadata JSONB NOT NULL DEFAULT '{}',
            score REAL,
            memory_type VARCHAR(50) NOT NULL,
            scope VARCHAR(50) NOT NULL,
            level VARCHAR(50) NOT NULL,
            importance REAL NOT NULL DEFAULT 0.0,
            access_count BIGINT NOT NULL DEFAULT 0,
            last_accessed TIMESTAMPTZ,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
            created_by_id VARCHAR(255),
            last_updated_by_id VARCHAR(255)
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| CoreError::Database(format!("Failed to create memories table: {}", e)))?;

    Ok(())
}

/// Create junction tables for many-to-many relationships
async fn create_junction_tables(pool: &PgPool) -> CoreResult<()> {
    // blocks_agents junction table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS blocks_agents (
            block_id VARCHAR(255) NOT NULL,
            block_label VARCHAR(50) NOT NULL,
            agent_id VARCHAR(255) NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
            PRIMARY KEY (block_id, block_label, agent_id),
            FOREIGN KEY (block_id, block_label) REFERENCES blocks(id, label) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| CoreError::Database(format!("Failed to create blocks_agents table: {}", e)))?;

    // tools_agents junction table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tools_agents (
            tool_id VARCHAR(255) NOT NULL REFERENCES tools(id) ON DELETE CASCADE,
            agent_id VARCHAR(255) NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
            PRIMARY KEY (tool_id, agent_id)
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| CoreError::Database(format!("Failed to create tools_agents table: {}", e)))?;

    Ok(())
}

/// Create indexes for better query performance
async fn create_indexes(pool: &PgPool) -> CoreResult<()> {
    // Organizations indexes
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_organizations_created_at ON organizations(created_at DESC)",
    )
    .execute(pool)
    .await
    .map_err(|e| CoreError::Database(format!("Failed to create index: {}", e)))?;

    // Users indexes
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_organization_id ON users(organization_id)")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create index: {}", e)))?;

    // Agents indexes
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_agents_organization_id ON agents(organization_id)")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create index: {}", e)))?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_agents_created_at ON agents(created_at DESC)")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create index: {}", e)))?;

    // Messages indexes (critical for performance)
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_messages_agent_created_at ON messages(agent_id, created_at)")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create index: {}", e)))?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_messages_created_at_id ON messages(created_at, id)",
    )
    .execute(pool)
    .await
    .map_err(|e| CoreError::Database(format!("Failed to create index: {}", e)))?;

    // Memories indexes
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_memories_agent_id ON memories(agent_id)")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create index: {}", e)))?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_memories_user_id ON memories(user_id)")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create index: {}", e)))?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_memories_scope ON memories(scope)")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create index: {}", e)))?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_memories_level ON memories(level)")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create index: {}", e)))?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_memories_importance ON memories(importance DESC)")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create index: {}", e)))?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_memories_created_at ON memories(created_at DESC)")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create index: {}", e)))?;

    // Full-text search index for memories
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_memories_content_fts ON memories USING gin(to_tsvector('english', content))")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create FTS index: {}", e)))?;

    Ok(())
}
