-- AgentMem Database Schema Initialization
-- Based on DATABASE_SCHEMA.md

-- 1. Organizations Table
CREATE TABLE IF NOT EXISTS organizations (
    id VARCHAR(255) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE
);

-- 2. Users Table
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
);

-- 3. Agents Table
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
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_by_id VARCHAR(255),
    last_updated_by_id VARCHAR(255)
);

-- 4. Messages Table
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
);

-- 5. Blocks Table (Core Memory)
CREATE TABLE IF NOT EXISTS blocks (
    id VARCHAR(255) PRIMARY KEY,
    organization_id VARCHAR(255) NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id VARCHAR(255) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    template_name VARCHAR(255),
    description TEXT,
    label VARCHAR(100) NOT NULL,
    is_template BOOLEAN NOT NULL DEFAULT FALSE,
    value TEXT NOT NULL,
    "limit" BIGINT NOT NULL DEFAULT 2000,
    metadata_ JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_by_id VARCHAR(255),
    last_updated_by_id VARCHAR(255)
);

-- 6. Tools Table
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
);

-- 7. Memories Table
CREATE TABLE IF NOT EXISTS memories (
    id VARCHAR(255) PRIMARY KEY,
    organization_id VARCHAR(255) NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id VARCHAR(255) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    agent_id VARCHAR(255) NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    hash VARCHAR(255),
    metadata JSONB NOT NULL DEFAULT '{}',
    score REAL,
    memory_type VARCHAR(100) NOT NULL,
    scope VARCHAR(100) NOT NULL,
    level VARCHAR(100) NOT NULL,
    importance REAL NOT NULL DEFAULT 0.5,
    access_count BIGINT NOT NULL DEFAULT 0,
    last_accessed TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_by_id VARCHAR(255),
    last_updated_by_id VARCHAR(255)
);

-- 8. Agent-Block Associations
CREATE TABLE IF NOT EXISTS agent_blocks (
    agent_id VARCHAR(255) NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    block_id VARCHAR(255) NOT NULL REFERENCES blocks(id) ON DELETE CASCADE,
    block_label VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (agent_id, block_id)
);

-- 9. Agent-Tool Associations
CREATE TABLE IF NOT EXISTS agent_tools (
    agent_id VARCHAR(255) NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    tool_id VARCHAR(255) NOT NULL REFERENCES tools(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (agent_id, tool_id)
);

-- Create Indexes
CREATE INDEX IF NOT EXISTS idx_users_organization_id ON users(organization_id);
CREATE INDEX IF NOT EXISTS idx_agents_organization_id ON agents(organization_id);
CREATE INDEX IF NOT EXISTS idx_messages_agent_id ON messages(agent_id);
CREATE INDEX IF NOT EXISTS idx_messages_user_id ON messages(user_id);
CREATE INDEX IF NOT EXISTS idx_messages_created_at ON messages(created_at);
CREATE INDEX IF NOT EXISTS idx_blocks_user_id ON blocks(user_id);
CREATE INDEX IF NOT EXISTS idx_tools_organization_id ON tools(organization_id);
CREATE INDEX IF NOT EXISTS idx_memories_agent_id ON memories(agent_id);
CREATE INDEX IF NOT EXISTS idx_memories_user_id ON memories(user_id);
CREATE INDEX IF NOT EXISTS idx_memories_memory_type ON memories(memory_type);
CREATE INDEX IF NOT EXISTS idx_memories_created_at ON memories(created_at);

-- Grant permissions to agentmem user
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO agentmem;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO agentmem;

