#!/bin/bash
# 初始化 AgentMem 数据库

set -e

DB_PATH="${1:-agentmem.db}"

echo "🗄️  初始化 AgentMem 数据库: $DB_PATH"

# 创建 semantic_memory 表
sqlite3 "$DB_PATH" <<'EOF'
CREATE TABLE IF NOT EXISTS semantic_memory (
    id TEXT PRIMARY KEY,
    organization_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    agent_id TEXT NOT NULL,
    name TEXT NOT NULL,
    summary TEXT NOT NULL,
    details TEXT NOT NULL,
    source TEXT,
    tree_path TEXT NOT NULL DEFAULT '[]',
    metadata TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    embedding TEXT,
    expires_at TEXT,
    version INTEGER NOT NULL DEFAULT 1
);

CREATE INDEX IF NOT EXISTS idx_semantic_memory_user_id ON semantic_memory(user_id);
CREATE INDEX IF NOT EXISTS idx_semantic_memory_agent_id ON semantic_memory(agent_id);
CREATE INDEX IF NOT EXISTS idx_semantic_memory_name ON semantic_memory(name);
CREATE INDEX IF NOT EXISTS idx_semantic_memory_updated_at ON semantic_memory(updated_at DESC);
EOF

echo "✅ semantic_memory 表创建成功"

# 创建 episodic_events 表
sqlite3 "$DB_PATH" <<'EOF'
CREATE TABLE IF NOT EXISTS episodic_events (
    id TEXT PRIMARY KEY,
    organization_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    agent_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    content TEXT NOT NULL,
    context TEXT,
    importance REAL NOT NULL DEFAULT 0.5,
    timestamp TEXT NOT NULL,
    metadata TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    embedding TEXT,
    expires_at TEXT,
    version INTEGER NOT NULL DEFAULT 1
);

CREATE INDEX IF NOT EXISTS idx_episodic_events_user_id ON episodic_events(user_id);
CREATE INDEX IF NOT EXISTS idx_episodic_events_agent_id ON episodic_events(agent_id);
CREATE INDEX IF NOT EXISTS idx_episodic_events_timestamp ON episodic_events(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_episodic_events_importance ON episodic_events(importance DESC);
EOF

echo "✅ episodic_events 表创建成功"

# 创建 core_memory 表
sqlite3 "$DB_PATH" <<'EOF'
CREATE TABLE IF NOT EXISTS core_memory (
    id TEXT PRIMARY KEY,
    organization_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    agent_id TEXT NOT NULL,
    block_label TEXT NOT NULL,
    value TEXT NOT NULL,
    limit_value INTEGER NOT NULL DEFAULT 2000,
    metadata TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    embedding TEXT,
    expires_at TEXT,
    version INTEGER NOT NULL DEFAULT 1
);

CREATE INDEX IF NOT EXISTS idx_core_memory_user_id ON core_memory(user_id);
CREATE INDEX IF NOT EXISTS idx_core_memory_agent_id ON core_memory(agent_id);
CREATE INDEX IF NOT EXISTS idx_core_memory_block_label ON core_memory(block_label);
EOF

echo "✅ core_memory 表创建成功"

echo "🎉 数据库初始化完成！"

