-- AgentMem Database Schema
-- This schema supports all SQLx queries in agent-mem-core

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";

-- Main memories table
CREATE TABLE IF NOT EXISTS memories (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    user_id TEXT,
    content TEXT NOT NULL,
    memory_type TEXT NOT NULL,
    importance REAL NOT NULL DEFAULT 0.5,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'::jsonb,
    search_vector tsvector,
    embedding vector(1536)  -- For vector search
);

-- Memory lifecycle tracking
CREATE TABLE IF NOT EXISTS memory_lifecycle (
    id TEXT PRIMARY KEY,
    memory_id TEXT NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
    state TEXT NOT NULL,
    event_type TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'::jsonb,
    previous_state TEXT,
    next_state TEXT
);

-- Memory associations (relationships between memories)
CREATE TABLE IF NOT EXISTS memory_associations (
    id TEXT PRIMARY KEY,
    organization_id TEXT,
    user_id TEXT,
    agent_id TEXT,
    memory_id_1 TEXT NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
    memory_id_2 TEXT NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
    association_type TEXT NOT NULL,
    strength REAL NOT NULL DEFAULT 0.5,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'::jsonb
);

-- Episodic memory (events and experiences)
CREATE TABLE IF NOT EXISTS episodic_memory (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    event TEXT NOT NULL,
    context TEXT,
    timestamp TIMESTAMPTZ NOT NULL,
    importance REAL NOT NULL DEFAULT 0.5,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'::jsonb
);

-- Semantic memory (facts and knowledge)
CREATE TABLE IF NOT EXISTS semantic_memory (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    summary TEXT,
    details TEXT,
    source TEXT,
    tree_path TEXT[],
    importance REAL NOT NULL DEFAULT 0.5,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'::jsonb
);

-- Procedural memory (how-to knowledge)
CREATE TABLE IF NOT EXISTS procedural_memory (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    steps TEXT[],
    conditions TEXT,
    expected_outcome TEXT,
    success_rate REAL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'::jsonb
);

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    organization_id TEXT,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    name TEXT,
    roles TEXT[] DEFAULT ARRAY['user']::TEXT[],
    is_active BOOLEAN DEFAULT true,
    is_deleted BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}'::jsonb
);

-- API keys table
CREATE TABLE IF NOT EXISTS api_keys (
    id TEXT PRIMARY KEY,
    key_hash TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    organization_id TEXT,
    scopes TEXT[] DEFAULT ARRAY[]::TEXT[],
    is_active BOOLEAN DEFAULT true,
    is_deleted BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ
);

-- Agents table
CREATE TABLE IF NOT EXISTS agents (
    id TEXT PRIMARY KEY,
    organization_id TEXT,
    user_id TEXT REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    config JSONB DEFAULT '{}'::jsonb,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Messages table
CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY,
    agent_id TEXT REFERENCES agents(id) ON DELETE CASCADE,
    user_id TEXT REFERENCES users(id) ON DELETE CASCADE,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'::jsonb
);

-- Tools table
CREATE TABLE IF NOT EXISTS tools (
    id TEXT PRIMARY KEY,
    user_id TEXT REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    tool_type TEXT NOT NULL,
    config JSONB DEFAULT '{}'::jsonb,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Blocks table (for core memory)
CREATE TABLE IF NOT EXISTS blocks (
    id TEXT PRIMARY KEY,
    agent_id TEXT REFERENCES agents(id) ON DELETE CASCADE,
    user_id TEXT REFERENCES users(id) ON DELETE CASCADE,
    block_type TEXT NOT NULL,
    content TEXT NOT NULL,
    position INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indices for performance

-- Memories indices
CREATE INDEX IF NOT EXISTS idx_memories_agent_id ON memories(agent_id);
CREATE INDEX IF NOT EXISTS idx_memories_user_id ON memories(user_id);
CREATE INDEX IF NOT EXISTS idx_memories_type ON memories(memory_type);
CREATE INDEX IF NOT EXISTS idx_memories_created_at ON memories(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_memories_importance ON memories(importance DESC);
CREATE INDEX IF NOT EXISTS idx_memories_search ON memories USING gin(search_vector);
CREATE INDEX IF NOT EXISTS idx_memories_metadata ON memories USING gin(metadata);

-- Lifecycle indices
CREATE INDEX IF NOT EXISTS idx_lifecycle_memory_id ON memory_lifecycle(memory_id);
CREATE INDEX IF NOT EXISTS idx_lifecycle_state ON memory_lifecycle(state);
CREATE INDEX IF NOT EXISTS idx_lifecycle_event_type ON memory_lifecycle(event_type);
CREATE INDEX IF NOT EXISTS idx_lifecycle_created_at ON memory_lifecycle(created_at DESC);

-- Associations indices
CREATE INDEX IF NOT EXISTS idx_associations_memory_1 ON memory_associations(memory_id_1);
CREATE INDEX IF NOT EXISTS idx_associations_memory_2 ON memory_associations(memory_id_2);
CREATE INDEX IF NOT EXISTS idx_associations_type ON memory_associations(association_type);
CREATE INDEX IF NOT EXISTS idx_associations_user_id ON memory_associations(user_id);
CREATE INDEX IF NOT EXISTS idx_associations_agent_id ON memory_associations(agent_id);

-- Episodic memory indices
CREATE INDEX IF NOT EXISTS idx_episodic_user_id ON episodic_memory(user_id);
CREATE INDEX IF NOT EXISTS idx_episodic_timestamp ON episodic_memory(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_episodic_importance ON episodic_memory(importance DESC);

-- Semantic memory indices
CREATE INDEX IF NOT EXISTS idx_semantic_user_id ON semantic_memory(user_id);
CREATE INDEX IF NOT EXISTS idx_semantic_name ON semantic_memory(name);
CREATE INDEX IF NOT EXISTS idx_semantic_tree_path ON semantic_memory USING gin(tree_path);

-- Procedural memory indices
CREATE INDEX IF NOT EXISTS idx_procedural_user_id ON procedural_memory(user_id);
CREATE INDEX IF NOT EXISTS idx_procedural_name ON procedural_memory(name);

-- Users indices
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_organization_id ON users(organization_id);
CREATE INDEX IF NOT EXISTS idx_users_is_active ON users(is_active) WHERE is_deleted = false;

-- API keys indices
CREATE INDEX IF NOT EXISTS idx_api_keys_user_id ON api_keys(user_id);
CREATE INDEX IF NOT EXISTS idx_api_keys_key_hash ON api_keys(key_hash);
CREATE INDEX IF NOT EXISTS idx_api_keys_is_active ON api_keys(is_active) WHERE is_deleted = false;

-- Agents indices
CREATE INDEX IF NOT EXISTS idx_agents_user_id ON agents(user_id);
CREATE INDEX IF NOT EXISTS idx_agents_organization_id ON agents(organization_id);

-- Messages indices
CREATE INDEX IF NOT EXISTS idx_messages_agent_id ON messages(agent_id);
CREATE INDEX IF NOT EXISTS idx_messages_user_id ON messages(user_id);
CREATE INDEX IF NOT EXISTS idx_messages_created_at ON messages(created_at DESC);

-- Tools indices
CREATE INDEX IF NOT EXISTS idx_tools_user_id ON tools(user_id);
CREATE INDEX IF NOT EXISTS idx_tools_tool_type ON tools(tool_type);

-- Blocks indices
CREATE INDEX IF NOT EXISTS idx_blocks_agent_id ON blocks(agent_id);
CREATE INDEX IF NOT EXISTS idx_blocks_user_id ON blocks(user_id);
CREATE INDEX IF NOT EXISTS idx_blocks_position ON blocks(position);

-- Triggers for updated_at

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_memories_updated_at BEFORE UPDATE ON memories
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_associations_updated_at BEFORE UPDATE ON memory_associations
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_episodic_updated_at BEFORE UPDATE ON episodic_memory
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_semantic_updated_at BEFORE UPDATE ON semantic_memory
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_procedural_updated_at BEFORE UPDATE ON procedural_memory
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_api_keys_updated_at BEFORE UPDATE ON api_keys
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_agents_updated_at BEFORE UPDATE ON agents
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_tools_updated_at BEFORE UPDATE ON tools
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_blocks_updated_at BEFORE UPDATE ON blocks
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Function to update search_vector
CREATE OR REPLACE FUNCTION update_search_vector()
RETURNS TRIGGER AS $$
BEGIN
    NEW.search_vector = to_tsvector('english', COALESCE(NEW.content, ''));
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_memories_search_vector BEFORE INSERT OR UPDATE ON memories
    FOR EACH ROW EXECUTE FUNCTION update_search_vector();

-- Comments
COMMENT ON TABLE memories IS 'Main memory storage table';
COMMENT ON TABLE memory_lifecycle IS 'Tracks memory state changes and lifecycle events';
COMMENT ON TABLE memory_associations IS 'Stores relationships between memories';
COMMENT ON TABLE episodic_memory IS 'Stores episodic memories (events and experiences)';
COMMENT ON TABLE semantic_memory IS 'Stores semantic memories (facts and knowledge)';
COMMENT ON TABLE procedural_memory IS 'Stores procedural memories (how-to knowledge)';

