//! Database migrations for AgentMem
//!
//! This module contains all database migration logic, creating tables and indexes
//! based on MIRIX's schema design.

use sqlx::PgPool;

use crate::{CoreError, CoreResult};
use super::memory_tables_migration;

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

    // Run memory-specific table migrations
    memory_tables_migration::run_memory_migrations(pool).await?;

    // Run migration to add missing fields (embedding, expires_at, version)
    migrate_add_missing_fields(pool).await?;

    // Run migration to create memory_history table and triggers
    create_memory_history_table(pool).await?;

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
            embedding JSONB,
            expires_at TIMESTAMPTZ,
            version INTEGER NOT NULL DEFAULT 1,
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

    // Index for expires_at (用于过期记忆查询)
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_memories_expires_at ON memories(expires_at) WHERE expires_at IS NOT NULL")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create expires_at index: {}", e)))?;

    // Index for version (用于乐观锁定)
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_memories_version ON memories(version)")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create version index: {}", e)))?;

    Ok(())
}

/// 迁移：添加缺失的字段到 memories 表
///
/// 此迁移添加以下字段：
/// - embedding: JSONB - 向量嵌入（用于语义搜索）
/// - expires_at: TIMESTAMPTZ - 过期时间（用于工作记忆）
/// - version: INTEGER - 版本号（用于乐观锁定）
///
/// 注意：此迁移是幂等的，可以安全地多次运行
pub async fn migrate_add_missing_fields(pool: &PgPool) -> CoreResult<()> {
    // 添加 embedding 字段（如果不存在）
    sqlx::query(
        r#"
        DO $$
        BEGIN
            IF NOT EXISTS (
                SELECT 1 FROM information_schema.columns
                WHERE table_name = 'memories' AND column_name = 'embedding'
            ) THEN
                ALTER TABLE memories ADD COLUMN embedding JSONB;
            END IF;
        END $$;
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| CoreError::Database(format!("Failed to add embedding column: {}", e)))?;

    // 添加 expires_at 字段（如果不存在）
    sqlx::query(
        r#"
        DO $$
        BEGIN
            IF NOT EXISTS (
                SELECT 1 FROM information_schema.columns
                WHERE table_name = 'memories' AND column_name = 'expires_at'
            ) THEN
                ALTER TABLE memories ADD COLUMN expires_at TIMESTAMPTZ;
            END IF;
        END $$;
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| CoreError::Database(format!("Failed to add expires_at column: {}", e)))?;

    // 添加 version 字段（如果不存在）
    sqlx::query(
        r#"
        DO $$
        BEGIN
            IF NOT EXISTS (
                SELECT 1 FROM information_schema.columns
                WHERE table_name = 'memories' AND column_name = 'version'
            ) THEN
                ALTER TABLE memories ADD COLUMN version INTEGER NOT NULL DEFAULT 1;
            END IF;
        END $$;
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| CoreError::Database(format!("Failed to add version column: {}", e)))?;

    // 创建索引（如果不存在）
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_memories_expires_at ON memories(expires_at) WHERE expires_at IS NOT NULL")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create expires_at index: {}", e)))?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_memories_version ON memories(version)")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create version index: {}", e)))?;

    Ok(())
}

/// 创建 memory_history 表和触发器
///
/// 此迁移创建：
/// - memory_history 表：存储记忆的历史版本
/// - 索引：优化查询性能
/// - 触发器函数：自动追踪记忆变更
/// - 触发器：在 INSERT/UPDATE/DELETE 时自动记录历史
async fn create_memory_history_table(pool: &PgPool) -> CoreResult<()> {
    // 1. 创建 memory_history 表
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS memory_history (
            id VARCHAR(255) PRIMARY KEY,
            memory_id VARCHAR(255) NOT NULL,
            version INTEGER NOT NULL,
            change_type VARCHAR(50) NOT NULL CHECK (change_type IN ('created', 'updated', 'deleted', 'restored')),
            change_reason TEXT,
            content TEXT NOT NULL,
            metadata JSONB NOT NULL DEFAULT '{}',
            memory_type VARCHAR(50) NOT NULL,
            importance REAL NOT NULL DEFAULT 0.0,
            organization_id VARCHAR(255) NOT NULL,
            user_id VARCHAR(255) NOT NULL,
            agent_id VARCHAR(255) NOT NULL,
            changed_by_id VARCHAR(255),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            CONSTRAINT fk_memory_history_memory
                FOREIGN KEY (memory_id)
                REFERENCES memories(id)
                ON DELETE CASCADE,
            CONSTRAINT fk_memory_history_organization
                FOREIGN KEY (organization_id)
                REFERENCES organizations(id)
                ON DELETE CASCADE,
            CONSTRAINT fk_memory_history_user
                FOREIGN KEY (user_id)
                REFERENCES users(id)
                ON DELETE CASCADE,
            CONSTRAINT fk_memory_history_agent
                FOREIGN KEY (agent_id)
                REFERENCES agents(id)
                ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| CoreError::Database(format!("Failed to create memory_history table: {}", e)))?;

    // 2. 创建索引
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_memory_history_memory_id ON memory_history(memory_id)")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create memory_id index: {}", e)))?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_memory_history_memory_version ON memory_history(memory_id, version DESC)")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create memory_version index: {}", e)))?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_memory_history_change_type ON memory_history(change_type)")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create change_type index: {}", e)))?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_memory_history_created_at ON memory_history(created_at DESC)")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create created_at index: {}", e)))?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_memory_history_tenant ON memory_history(organization_id, user_id, agent_id)")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create tenant index: {}", e)))?;

    // 3. 创建触发器函数
    sqlx::query(
        r#"
        CREATE OR REPLACE FUNCTION track_memory_changes()
        RETURNS TRIGGER AS $$
        DECLARE
            next_version INTEGER;
            change_type_val VARCHAR(50);
        BEGIN
            IF TG_OP = 'INSERT' THEN
                change_type_val := 'created';
                next_version := 1;
            ELSIF TG_OP = 'UPDATE' THEN
                change_type_val := 'updated';
                SELECT COALESCE(MAX(version), 0) + 1 INTO next_version
                FROM memory_history
                WHERE memory_id = NEW.id;
            ELSIF TG_OP = 'DELETE' THEN
                change_type_val := 'deleted';
                SELECT COALESCE(MAX(version), 0) + 1 INTO next_version
                FROM memory_history
                WHERE memory_id = OLD.id;

                INSERT INTO memory_history (
                    id, memory_id, version, change_type, change_reason,
                    content, metadata, memory_type, importance,
                    organization_id, user_id, agent_id, changed_by_id, created_at
                ) VALUES (
                    gen_random_uuid()::TEXT, OLD.id, next_version, change_type_val, 'Memory deleted',
                    OLD.content, OLD.metadata, OLD.memory_type, OLD.importance,
                    OLD.organization_id, OLD.user_id, OLD.agent_id, OLD.last_updated_by_id, NOW()
                );

                RETURN OLD;
            END IF;

            INSERT INTO memory_history (
                id, memory_id, version, change_type, change_reason,
                content, metadata, memory_type, importance,
                organization_id, user_id, agent_id, changed_by_id, created_at
            ) VALUES (
                gen_random_uuid()::TEXT, NEW.id, next_version, change_type_val,
                CASE
                    WHEN TG_OP = 'INSERT' THEN 'Initial version'
                    WHEN TG_OP = 'UPDATE' THEN 'Memory updated'
                    ELSE NULL
                END,
                NEW.content, NEW.metadata, NEW.memory_type, NEW.importance,
                NEW.organization_id, NEW.user_id, NEW.agent_id, NEW.last_updated_by_id, NOW()
            );

            RETURN NEW;
        END;
        $$ LANGUAGE plpgsql;
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| CoreError::Database(format!("Failed to create trigger function: {}", e)))?;

    // 4. 创建触发器
    sqlx::query("DROP TRIGGER IF EXISTS trigger_memory_insert ON memories")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to drop old insert trigger: {}", e)))?;

    sqlx::query("DROP TRIGGER IF EXISTS trigger_memory_update ON memories")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to drop old update trigger: {}", e)))?;

    sqlx::query("DROP TRIGGER IF EXISTS trigger_memory_delete ON memories")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to drop old delete trigger: {}", e)))?;

    sqlx::query(
        "CREATE TRIGGER trigger_memory_insert AFTER INSERT ON memories FOR EACH ROW EXECUTE FUNCTION track_memory_changes()"
    )
    .execute(pool)
    .await
    .map_err(|e| CoreError::Database(format!("Failed to create insert trigger: {}", e)))?;

    sqlx::query(
        "CREATE TRIGGER trigger_memory_update AFTER UPDATE ON memories FOR EACH ROW WHEN (OLD.* IS DISTINCT FROM NEW.*) EXECUTE FUNCTION track_memory_changes()"
    )
    .execute(pool)
    .await
    .map_err(|e| CoreError::Database(format!("Failed to create update trigger: {}", e)))?;

    sqlx::query(
        "CREATE TRIGGER trigger_memory_delete BEFORE DELETE ON memories FOR EACH ROW EXECUTE FUNCTION track_memory_changes()"
    )
    .execute(pool)
    .await
    .map_err(|e| CoreError::Database(format!("Failed to create delete trigger: {}", e)))?;

    Ok(())
}
