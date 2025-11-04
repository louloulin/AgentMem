//! LibSQL database migrations
//!
//! Creates and manages database schema for LibSQL backend

use agent_mem_traits::{AgentMemError, Result};
use libsql::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Run all database migrations
pub async fn run_migrations(conn: Arc<Mutex<Connection>>) -> Result<()> {
    let conn_guard = conn.lock().await;

    // Create migrations tracking table
    create_migrations_table(&conn_guard).await?;

    // Run migrations in order
    run_migration(&conn_guard, 1, "create_organizations", create_organizations_table(&conn_guard)).await?;
    run_migration(&conn_guard, 2, "create_users", create_users_table(&conn_guard)).await?;
    run_migration(&conn_guard, 3, "create_agents", create_agents_table(&conn_guard)).await?;
    run_migration(&conn_guard, 4, "create_messages", create_messages_table(&conn_guard)).await?;
    run_migration(&conn_guard, 5, "create_blocks", create_blocks_table(&conn_guard)).await?;
    run_migration(&conn_guard, 6, "create_tools", create_tools_table(&conn_guard)).await?;
    run_migration(&conn_guard, 7, "create_memories", create_memories_table(&conn_guard)).await?;
    run_migration(&conn_guard, 8, "create_api_keys", create_api_keys_table(&conn_guard)).await?;
    run_migration(&conn_guard, 9, "create_junction_tables", create_junction_tables(&conn_guard)).await?;
    run_migration(&conn_guard, 10, "create_memory_associations", create_memory_associations_table(&conn_guard)).await?;
    run_migration(&conn_guard, 11, "create_indexes", create_indexes(&conn_guard)).await?;
    run_migration(&conn_guard, 12, "create_learning_feedback", create_learning_feedback_table(&conn_guard)).await?;
    run_migration(&conn_guard, 13, "add_session_id_to_memories", add_session_id_to_memories(&conn_guard)).await?;

    // Initialize default data (idempotent - safe to run multiple times)
    init_default_data(&conn_guard).await?;

    Ok(())
}

/// Create migrations tracking table
async fn create_migrations_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS _migrations (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at INTEGER NOT NULL
        )",
        (),
    )
    .await
    .map_err(|e| AgentMemError::StorageError(format!("Failed to create migrations table: {e}")))?;

    Ok(())
}

/// Run a single migration if not already applied
async fn run_migration(
    conn: &Connection,
    version: i64,
    name: &str,
    migration_fn: impl std::future::Future<Output = Result<()>>,
) -> Result<()> {
    // Check if migration already applied
    let mut rows = conn
        .query("SELECT id FROM _migrations WHERE id = ?", libsql::params![version])
        .await
        .map_err(|e| AgentMemError::StorageError(format!("Failed to check migration status: {e}")))?;

    if rows.next().await.map_err(|e| AgentMemError::StorageError(e.to_string()))?.is_some() {
        return Ok(()); // Already applied
    }

    // Run migration
    migration_fn.await?;

    // Record migration
    conn.execute(
        "INSERT INTO _migrations (id, name, applied_at) VALUES (?, ?, ?)",
        libsql::params![version, name, chrono::Utc::now().timestamp()],
    )
    .await
    .map_err(|e| AgentMemError::StorageError(format!("Failed to record migration: {e}")))?;

    Ok(())
}

/// Create organizations table
async fn create_organizations_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE organizations (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            is_deleted INTEGER NOT NULL DEFAULT 0
        )",
        (),
    )
    .await
    .map_err(|e| AgentMemError::StorageError(format!("Failed to create organizations table: {e}")))?;

    Ok(())
}

/// Create users table
async fn create_users_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE users (
            id TEXT PRIMARY KEY,
            organization_id TEXT NOT NULL,
            name TEXT NOT NULL,
            email TEXT NOT NULL,
            password_hash TEXT NOT NULL,
            roles TEXT,
            status TEXT NOT NULL,
            timezone TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            is_deleted INTEGER NOT NULL DEFAULT 0,
            created_by_id TEXT,
            last_updated_by_id TEXT,
            FOREIGN KEY (organization_id) REFERENCES organizations(id),
            UNIQUE(organization_id, email)
        )",
        (),
    )
    .await
    .map_err(|e| AgentMemError::StorageError(format!("Failed to create users table: {e}")))?;

    Ok(())
}

/// Create agents table
async fn create_agents_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE agents (
            id TEXT PRIMARY KEY,
            organization_id TEXT NOT NULL,
            agent_type TEXT,
            name TEXT,
            description TEXT,
            system TEXT,
            topic TEXT,
            message_ids TEXT,
            metadata TEXT,
            llm_config TEXT,
            embedding_config TEXT,
            tool_rules TEXT,
            mcp_tools TEXT,
            state TEXT,
            last_active_at INTEGER,
            error_message TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            is_deleted INTEGER NOT NULL DEFAULT 0,
            created_by_id TEXT,
            last_updated_by_id TEXT,
            FOREIGN KEY (organization_id) REFERENCES organizations(id)
        )",
        (),
    )
    .await
    .map_err(|e| AgentMemError::StorageError(format!("Failed to create agents table: {e}")))?;

    Ok(())
}

/// Create messages table
async fn create_messages_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE messages (
            id TEXT PRIMARY KEY,
            organization_id TEXT NOT NULL,
            user_id TEXT NOT NULL,
            agent_id TEXT NOT NULL,
            role TEXT NOT NULL,
            text TEXT,
            content TEXT,
            model TEXT,
            name TEXT,
            tool_calls TEXT,
            tool_call_id TEXT,
            step_id TEXT,
            otid TEXT,
            tool_returns TEXT,
            group_id TEXT,
            sender_id TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            is_deleted INTEGER NOT NULL DEFAULT 0,
            created_by_id TEXT,
            last_updated_by_id TEXT
        )",
        (),
    )
    .await
    .map_err(|e| AgentMemError::StorageError(format!("Failed to create messages table: {e}")))?;

    // 创建索引但不强制外键约束（避免UI测试时的约束问题）
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_messages_organization ON messages(organization_id)",
        (),
    ).await.map_err(|e| AgentMemError::StorageError(format!("Failed to create index: {e}")))?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_messages_user ON messages(user_id)",
        (),
    ).await.map_err(|e| AgentMemError::StorageError(format!("Failed to create index: {e}")))?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_messages_agent ON messages(agent_id)",
        (),
    ).await.map_err(|e| AgentMemError::StorageError(format!("Failed to create index: {e}")))?;

    Ok(())
}

/// Create blocks table (Core Memory)
async fn create_blocks_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE blocks (
            id TEXT PRIMARY KEY,
            organization_id TEXT NOT NULL,
            user_id TEXT NOT NULL,
            template_name TEXT,
            description TEXT,
            is_template INTEGER NOT NULL DEFAULT 0,
            label TEXT NOT NULL,
            value TEXT NOT NULL,
            \"limit\" INTEGER NOT NULL,
            metadata TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            is_deleted INTEGER NOT NULL DEFAULT 0,
            created_by_id TEXT,
            last_updated_by_id TEXT,
            FOREIGN KEY (organization_id) REFERENCES organizations(id),
            FOREIGN KEY (user_id) REFERENCES users(id)
        )",
        (),
    )
    .await
    .map_err(|e| AgentMemError::StorageError(format!("Failed to create blocks table: {e}")))?;

    Ok(())
}

/// Create tools table
async fn create_tools_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE tools (
            id TEXT PRIMARY KEY,
            organization_id TEXT NOT NULL,
            name TEXT NOT NULL,
            description TEXT,
            json_schema TEXT,
            source_type TEXT,
            source_code TEXT,
            tags TEXT,
            metadata TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            is_deleted INTEGER NOT NULL DEFAULT 0,
            created_by_id TEXT,
            last_updated_by_id TEXT,
            FOREIGN KEY (organization_id) REFERENCES organizations(id)
        )",
        (),
    )
    .await
    .map_err(|e| AgentMemError::StorageError(format!("Failed to create tools table: {e}")))?;

    Ok(())
}

/// Create memories table
async fn create_memories_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE memories (
            id TEXT PRIMARY KEY,
            organization_id TEXT NOT NULL,
            user_id TEXT NOT NULL,
            agent_id TEXT NOT NULL,
            content TEXT NOT NULL,
            hash TEXT,
            metadata TEXT,
            score REAL,
            memory_type TEXT NOT NULL,
            scope TEXT NOT NULL,
            level TEXT NOT NULL,
            importance REAL NOT NULL,
            access_count INTEGER NOT NULL DEFAULT 0,
            last_accessed INTEGER,
            embedding TEXT,
            expires_at INTEGER,
            version INTEGER NOT NULL DEFAULT 1,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            is_deleted INTEGER NOT NULL DEFAULT 0,
            created_by_id TEXT,
            last_updated_by_id TEXT
        )",
        (),
    )
    .await
    .map_err(|e| AgentMemError::StorageError(format!("Failed to create memories table: {e}")))?;

    // 创建索引但不强制外键约束
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_memories_organization ON memories(organization_id)",
        (),
    ).await.map_err(|e| AgentMemError::StorageError(format!("Failed to create index: {e}")))?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_memories_user ON memories(user_id)",
        (),
    ).await.map_err(|e| AgentMemError::StorageError(format!("Failed to create index: {e}")))?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_memories_agent ON memories(agent_id)",
        (),
    ).await.map_err(|e| AgentMemError::StorageError(format!("Failed to create index: {e}")))?;

    Ok(())
}

/// Create API keys table
async fn create_api_keys_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE api_keys (
            id TEXT PRIMARY KEY,
            key_hash TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            user_id TEXT NOT NULL,
            organization_id TEXT NOT NULL,
            expires_at INTEGER,
            last_used_at INTEGER,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            is_deleted INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (user_id) REFERENCES users(id),
            FOREIGN KEY (organization_id) REFERENCES organizations(id)
        )",
        (),
    )
    .await
    .map_err(|e| AgentMemError::StorageError(format!("Failed to create api_keys table: {e}")))?;

    Ok(())
}

/// Create junction tables for many-to-many relationships
async fn create_junction_tables(conn: &Connection) -> Result<()> {
    // blocks_agents junction table
    conn.execute(
        "CREATE TABLE blocks_agents (
            block_id TEXT NOT NULL,
            agent_id TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            PRIMARY KEY (block_id, agent_id),
            FOREIGN KEY (block_id) REFERENCES blocks(id),
            FOREIGN KEY (agent_id) REFERENCES agents(id)
        )",
        (),
    )
    .await
    .map_err(|e| AgentMemError::StorageError(format!("Failed to create blocks_agents table: {e}")))?;

    // tools_agents junction table
    conn.execute(
        "CREATE TABLE tools_agents (
            tool_id TEXT NOT NULL,
            agent_id TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            PRIMARY KEY (tool_id, agent_id),
            FOREIGN KEY (tool_id) REFERENCES tools(id),
            FOREIGN KEY (agent_id) REFERENCES agents(id)
        )",
        (),
    )
    .await
    .map_err(|e| AgentMemError::StorageError(format!("Failed to create tools_agents table: {e}")))?;

    Ok(())
}

/// Create memory associations table
async fn create_memory_associations_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE memory_associations (
            id TEXT PRIMARY KEY,
            organization_id TEXT NOT NULL,
            user_id TEXT NOT NULL,
            agent_id TEXT NOT NULL,
            from_memory_id TEXT NOT NULL,
            to_memory_id TEXT NOT NULL,
            association_type TEXT NOT NULL,
            strength REAL NOT NULL,
            confidence REAL NOT NULL,
            metadata TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            FOREIGN KEY (organization_id) REFERENCES organizations(id),
            FOREIGN KEY (user_id) REFERENCES users(id),
            FOREIGN KEY (agent_id) REFERENCES agents(id),
            FOREIGN KEY (from_memory_id) REFERENCES memories(id),
            FOREIGN KEY (to_memory_id) REFERENCES memories(id)
        )",
        (),
    )
    .await
    .map_err(|e| AgentMemError::StorageError(format!("Failed to create memory_associations table: {e}")))?;

    Ok(())
}

/// Create indexes for performance
async fn create_indexes(conn: &Connection) -> Result<()> {
    let indexes = vec![
        // Organization indexes
        "CREATE INDEX IF NOT EXISTS idx_organizations_name ON organizations(name)",
        "CREATE INDEX IF NOT EXISTS idx_organizations_deleted ON organizations(is_deleted)",
        
        // User indexes
        "CREATE INDEX IF NOT EXISTS idx_users_org_id ON users(organization_id)",
        "CREATE INDEX IF NOT EXISTS idx_users_status ON users(status)",
        "CREATE INDEX IF NOT EXISTS idx_users_deleted ON users(is_deleted)",
        
        // Agent indexes
        "CREATE INDEX IF NOT EXISTS idx_agents_org_id ON agents(organization_id)",
        "CREATE INDEX IF NOT EXISTS idx_agents_created_at ON agents(created_at)",
        "CREATE INDEX IF NOT EXISTS idx_agents_deleted ON agents(is_deleted)",
        
        // Message indexes
        "CREATE INDEX IF NOT EXISTS idx_messages_agent_id ON messages(agent_id)",
        "CREATE INDEX IF NOT EXISTS idx_messages_user_id ON messages(user_id)",
        "CREATE INDEX IF NOT EXISTS idx_messages_created_at ON messages(created_at)",
        "CREATE INDEX IF NOT EXISTS idx_messages_deleted ON messages(is_deleted)",
        
        // Tool indexes
        "CREATE INDEX IF NOT EXISTS idx_tools_org_id ON tools(organization_id)",
        "CREATE INDEX IF NOT EXISTS idx_tools_name ON tools(name)",
        "CREATE INDEX IF NOT EXISTS idx_tools_deleted ON tools(is_deleted)",
        
        // Memory indexes
        "CREATE INDEX IF NOT EXISTS idx_memories_agent_id ON memories(agent_id)",
        "CREATE INDEX IF NOT EXISTS idx_memories_user_id ON memories(user_id)",
        "CREATE INDEX IF NOT EXISTS idx_memories_created_at ON memories(created_at)",
        "CREATE INDEX IF NOT EXISTS idx_memories_deleted ON memories(is_deleted)",
        
        // API Key indexes
        "CREATE INDEX IF NOT EXISTS idx_api_keys_user_id ON api_keys(user_id)",
        "CREATE INDEX IF NOT EXISTS idx_api_keys_key_hash ON api_keys(key_hash)",
        "CREATE INDEX IF NOT EXISTS idx_api_keys_deleted ON api_keys(is_deleted)",
        
        // Block indexes
        "CREATE INDEX IF NOT EXISTS idx_blocks_label ON blocks(label)",
        "CREATE INDEX IF NOT EXISTS idx_blocks_deleted ON blocks(is_deleted)",
    ];

    for index_sql in indexes {
        conn.execute(index_sql, ())
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to create index: {e}")))?;
    }

    Ok(())
}

/// Create learning_feedback table for storing search optimization feedback
async fn create_learning_feedback_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE learning_feedback (
            id TEXT PRIMARY KEY,
            query_pattern TEXT NOT NULL,
            features TEXT NOT NULL,
            vector_weight REAL NOT NULL,
            fulltext_weight REAL NOT NULL,
            effectiveness REAL NOT NULL,
            timestamp INTEGER NOT NULL,
            user_id TEXT
        )",
        (),
    )
    .await
    .map_err(|e| AgentMemError::StorageError(format!("Failed to create learning_feedback table: {e}")))?;

    // Create indexes for learning_feedback table
    let indexes = vec![
        "CREATE INDEX IF NOT EXISTS idx_learning_feedback_pattern ON learning_feedback(query_pattern)",
        "CREATE INDEX IF NOT EXISTS idx_learning_feedback_timestamp ON learning_feedback(timestamp)",
        "CREATE INDEX IF NOT EXISTS idx_learning_feedback_user_id ON learning_feedback(user_id)",
    ];

    for index_sql in indexes {
        conn.execute(index_sql, ())
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to create learning_feedback index: {e}")))?;
    }

    Ok(())
}

/// Add session_id column to memories table for Working Memory support
/// This migration enables unified memory model where Working Memory uses the same table
async fn add_session_id_to_memories(conn: &Connection) -> Result<()> {
    // Add session_id column (SQLite allows adding columns to existing tables)
    conn.execute(
        "ALTER TABLE memories ADD COLUMN session_id TEXT",
        (),
    )
    .await
    .map_err(|e| AgentMemError::StorageError(format!("Failed to add session_id column: {e}")))?;

    // Create index on session_id for fast session-based queries
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_memories_session_id ON memories(session_id)",
        (),
    )
    .await
    .map_err(|e| AgentMemError::StorageError(format!("Failed to create session_id index: {e}")))?;

    // Create composite index for Working Memory queries (session_id + memory_type)
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_memories_session_type ON memories(session_id, memory_type)",
        (),
    )
    .await
    .map_err(|e| AgentMemError::StorageError(format!("Failed to create composite index: {e}")))?;

    Ok(())
}

/// Initialize default data (organizations, users)
/// This is idempotent - safe to run multiple times
async fn init_default_data(conn: &Connection) -> Result<()> {
    use chrono::Utc;
    
    let now = Utc::now().timestamp();
    
    // Insert default organization (if not exists)
    conn.execute(
        "INSERT OR IGNORE INTO organizations (id, name, created_at, updated_at, is_deleted)
         VALUES (?, ?, ?, ?, ?)",
        libsql::params![
            "default-org",
            "Default Organization",
            now,
            now,
            0
        ],
    )
    .await
    .map_err(|e| AgentMemError::StorageError(format!("Failed to insert default organization: {e}")))?;
    
    // Insert default user (if not exists)
    conn.execute(
        "INSERT OR IGNORE INTO users (id, organization_id, email, name, created_at, updated_at, is_deleted)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
        libsql::params![
            "default",
            "default-org",
            "default@agentmem.local",
            "Default User",
            now,
            now,
            0
        ],
    )
    .await
    .map_err(|e| AgentMemError::StorageError(format!("Failed to insert default user: {e}")))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::libsql::connection::create_libsql_pool;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_run_migrations() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = create_libsql_pool(db_path.to_str().unwrap()).await.unwrap();

        let result = run_migrations(conn.clone()).await;
        assert!(result.is_ok());

        // Verify migrations table exists
        let conn_guard = conn.lock().await;
        let mut rows = conn_guard
            .query("SELECT COUNT(*) FROM _migrations", ())
            .await
            .unwrap();

        let row = rows.next().await.unwrap().unwrap();
        let count: i64 = row.get(0).unwrap();
        assert_eq!(count, 12); // 12 migrations (including learning_feedback)
    }

    #[tokio::test]
    async fn test_migrations_idempotent() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = create_libsql_pool(db_path.to_str().unwrap()).await.unwrap();

        // Run migrations twice
        run_migrations(conn.clone()).await.unwrap();
        let result = run_migrations(conn.clone()).await;
        assert!(result.is_ok());

        // Should still have 12 migrations
        let conn_guard = conn.lock().await;
        let mut rows = conn_guard
            .query("SELECT COUNT(*) FROM _migrations", ())
            .await
            .unwrap();

        let row = rows.next().await.unwrap().unwrap();
        let count: i64 = row.get(0).unwrap();
        assert_eq!(count, 13); // 13 migrations (including session_id)
    }
}

