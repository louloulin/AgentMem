-- Performance Optimization Indexes for AgentMem
-- Task 4.3.1: Database Optimization
-- Created: 2025-10-07

-- ============================================================================
-- 1. Composite Indexes for Common Query Patterns
-- ============================================================================

-- Messages: Common query pattern (agent_id + created_at for timeline queries)
CREATE INDEX IF NOT EXISTS idx_messages_agent_created 
ON messages(agent_id, created_at DESC) 
WHERE is_deleted = FALSE;

-- Messages: User timeline queries
CREATE INDEX IF NOT EXISTS idx_messages_user_created 
ON messages(user_id, created_at DESC) 
WHERE is_deleted = FALSE;

-- Messages: Organization-wide queries
CREATE INDEX IF NOT EXISTS idx_messages_org_created 
ON messages(organization_id, created_at DESC) 
WHERE is_deleted = FALSE;

-- Memories: Most common query pattern (agent + type + importance)
CREATE INDEX IF NOT EXISTS idx_memories_agent_type_importance 
ON memories(agent_id, memory_type, importance DESC) 
WHERE is_deleted = FALSE;

-- Memories: User + type queries
CREATE INDEX IF NOT EXISTS idx_memories_user_type 
ON memories(user_id, memory_type) 
WHERE is_deleted = FALSE;

-- Memories: Recent access pattern (for cache warming)
CREATE INDEX IF NOT EXISTS idx_memories_last_accessed 
ON memories(last_accessed DESC NULLS LAST) 
WHERE is_deleted = FALSE;

-- Memories: Access count for popularity-based queries
CREATE INDEX IF NOT EXISTS idx_memories_access_count 
ON memories(access_count DESC) 
WHERE is_deleted = FALSE;

-- ============================================================================
-- 2. Partial Indexes for Soft Delete Pattern
-- ============================================================================

-- Only index non-deleted records (reduces index size by ~50% typically)
CREATE INDEX IF NOT EXISTS idx_users_active 
ON users(id, organization_id) 
WHERE is_deleted = FALSE;

CREATE INDEX IF NOT EXISTS idx_agents_active 
ON agents(id, organization_id) 
WHERE is_deleted = FALSE;

CREATE INDEX IF NOT EXISTS idx_blocks_active 
ON blocks(id, user_id) 
WHERE is_deleted = FALSE;

CREATE INDEX IF NOT EXISTS idx_tools_active 
ON tools(id, organization_id) 
WHERE is_deleted = FALSE;

-- ============================================================================
-- 3. JSONB Indexes for Metadata Queries
-- ============================================================================

-- GIN indexes for JSONB columns (enables fast containment queries)
CREATE INDEX IF NOT EXISTS idx_messages_content_gin 
ON messages USING GIN (content);

CREATE INDEX IF NOT EXISTS idx_memories_metadata_gin 
ON memories USING GIN (metadata);

CREATE INDEX IF NOT EXISTS idx_agents_metadata_gin 
ON agents USING GIN (metadata_);

CREATE INDEX IF NOT EXISTS idx_agents_llm_config_gin 
ON agents USING GIN (llm_config);

CREATE INDEX IF NOT EXISTS idx_blocks_metadata_gin 
ON blocks USING GIN (metadata_);

-- ============================================================================
-- 4. Text Search Indexes
-- ============================================================================

-- Full-text search on memory content
CREATE INDEX IF NOT EXISTS idx_memories_content_fts 
ON memories USING GIN (to_tsvector('english', content));

-- Full-text search on messages
CREATE INDEX IF NOT EXISTS idx_messages_text_fts 
ON messages USING GIN (to_tsvector('english', COALESCE(text, '')));

-- Full-text search on blocks
CREATE INDEX IF NOT EXISTS idx_blocks_value_fts 
ON blocks USING GIN (to_tsvector('english', value));

-- ============================================================================
-- 5. Hash Indexes for Exact Match Queries
-- ============================================================================

-- Hash index for memory deduplication
CREATE INDEX IF NOT EXISTS idx_memories_hash 
ON memories USING HASH (hash) 
WHERE hash IS NOT NULL;

-- ============================================================================
-- 6. Covering Indexes for Common Queries
-- ============================================================================

-- Covering index for memory list queries (includes commonly selected columns)
CREATE INDEX IF NOT EXISTS idx_memories_list_covering 
ON memories(agent_id, memory_type, created_at DESC) 
INCLUDE (content, importance, access_count, metadata) 
WHERE is_deleted = FALSE;

-- Covering index for message list queries
CREATE INDEX IF NOT EXISTS idx_messages_list_covering 
ON messages(agent_id, created_at DESC) 
INCLUDE (role, text, content, model) 
WHERE is_deleted = FALSE;

-- ============================================================================
-- 7. Indexes for Association Tables
-- ============================================================================

-- Reverse lookup for agent_blocks
CREATE INDEX IF NOT EXISTS idx_agent_blocks_block_id 
ON agent_blocks(block_id);

-- Reverse lookup for agent_tools
CREATE INDEX IF NOT EXISTS idx_agent_tools_tool_id 
ON agent_tools(tool_id);

-- ============================================================================
-- 8. Indexes for Time-based Queries
-- ============================================================================

-- Updated_at indexes for change tracking
CREATE INDEX IF NOT EXISTS idx_memories_updated_at 
ON memories(updated_at DESC) 
WHERE is_deleted = FALSE;

CREATE INDEX IF NOT EXISTS idx_messages_updated_at 
ON messages(updated_at DESC) 
WHERE is_deleted = FALSE;

CREATE INDEX IF NOT EXISTS idx_agents_updated_at 
ON agents(updated_at DESC) 
WHERE is_deleted = FALSE;

-- ============================================================================
-- 9. Statistics Update
-- ============================================================================

-- Update table statistics for better query planning
ANALYZE organizations;
ANALYZE users;
ANALYZE agents;
ANALYZE messages;
ANALYZE blocks;
ANALYZE tools;
ANALYZE memories;
ANALYZE agent_blocks;
ANALYZE agent_tools;

-- ============================================================================
-- 10. Index Usage Comments
-- ============================================================================

COMMENT ON INDEX idx_messages_agent_created IS 'Optimizes agent timeline queries with date filtering';
COMMENT ON INDEX idx_memories_agent_type_importance IS 'Optimizes memory retrieval by agent, type, and importance';
COMMENT ON INDEX idx_memories_content_fts IS 'Enables full-text search on memory content';
COMMENT ON INDEX idx_memories_metadata_gin IS 'Enables fast JSONB containment queries on metadata';
COMMENT ON INDEX idx_memories_list_covering IS 'Covering index to avoid table lookups for list queries';
COMMENT ON INDEX idx_memories_hash IS 'Optimizes memory deduplication checks';
COMMENT ON INDEX idx_memories_last_accessed IS 'Supports cache warming and LRU eviction';
COMMENT ON INDEX idx_memories_access_count IS 'Supports popularity-based queries';

-- ============================================================================
-- Performance Notes:
-- ============================================================================
-- 1. Composite indexes are ordered by selectivity (most selective first)
-- 2. Partial indexes reduce index size and improve write performance
-- 3. GIN indexes enable fast JSONB and full-text search queries
-- 4. Covering indexes eliminate table lookups for common queries
-- 5. Hash indexes are faster than B-tree for exact match queries
-- 6. Regular ANALYZE ensures query planner has up-to-date statistics
-- ============================================================================

