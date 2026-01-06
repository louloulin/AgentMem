-- Migration: Add missing fields (embedding, expires_at, version) to all memory tables
-- Date: 2025-01-10
-- Description: Adds support for vector search, memory expiration, and optimistic locking

-- ============================================================================
-- 1. Add fields to episodic_events table
-- ============================================================================

ALTER TABLE episodic_events 
ADD COLUMN IF NOT EXISTS embedding TEXT,
ADD COLUMN IF NOT EXISTS expires_at TIMESTAMPTZ,
ADD COLUMN IF NOT EXISTS version INTEGER NOT NULL DEFAULT 1;

-- Create index for expires_at (partial index for non-null values)
CREATE INDEX IF NOT EXISTS idx_episodic_expires 
ON episodic_events(expires_at) 
WHERE expires_at IS NOT NULL;

COMMENT ON COLUMN episodic_events.embedding IS '向量嵌入（JSON 格式存储，用于向量搜索）';
COMMENT ON COLUMN episodic_events.expires_at IS '过期时间（NULL 表示永不过期）';
COMMENT ON COLUMN episodic_events.version IS '版本号（用于乐观锁，每次更新递增）';

-- ============================================================================
-- 2. Add fields to semantic_memory table
-- ============================================================================

ALTER TABLE semantic_memory 
ADD COLUMN IF NOT EXISTS embedding TEXT,
ADD COLUMN IF NOT EXISTS expires_at TIMESTAMPTZ,
ADD COLUMN IF NOT EXISTS version INTEGER NOT NULL DEFAULT 1;

-- Create index for expires_at
CREATE INDEX IF NOT EXISTS idx_semantic_expires 
ON semantic_memory(expires_at) 
WHERE expires_at IS NOT NULL;

COMMENT ON COLUMN semantic_memory.embedding IS '向量嵌入（JSON 格式存储，用于向量搜索）';
COMMENT ON COLUMN semantic_memory.expires_at IS '过期时间（NULL 表示永不过期）';
COMMENT ON COLUMN semantic_memory.version IS '版本号（用于乐观锁，每次更新递增）';

-- ============================================================================
-- 3. Add fields to procedural_memory table
-- ============================================================================

ALTER TABLE procedural_memory 
ADD COLUMN IF NOT EXISTS embedding TEXT,
ADD COLUMN IF NOT EXISTS expires_at TIMESTAMPTZ,
ADD COLUMN IF NOT EXISTS version INTEGER NOT NULL DEFAULT 1;

-- Create index for expires_at
CREATE INDEX IF NOT EXISTS idx_procedural_expires 
ON procedural_memory(expires_at) 
WHERE expires_at IS NOT NULL;

COMMENT ON COLUMN procedural_memory.embedding IS '向量嵌入（JSON 格式存储，用于向量搜索）';
COMMENT ON COLUMN procedural_memory.expires_at IS '过期时间（NULL 表示永不过期）';
COMMENT ON COLUMN procedural_memory.version IS '版本号（用于乐观锁，每次更新递增）';

-- ============================================================================
-- 4. Add fields to core_memory table
-- ============================================================================

ALTER TABLE core_memory 
ADD COLUMN IF NOT EXISTS embedding TEXT,
ADD COLUMN IF NOT EXISTS expires_at TIMESTAMPTZ,
ADD COLUMN IF NOT EXISTS version INTEGER NOT NULL DEFAULT 1;

-- Create index for expires_at
CREATE INDEX IF NOT EXISTS idx_core_expires 
ON core_memory(expires_at) 
WHERE expires_at IS NOT NULL;

COMMENT ON COLUMN core_memory.embedding IS '向量嵌入（JSON 格式存储，用于向量搜索）';
COMMENT ON COLUMN core_memory.expires_at IS '过期时间（NULL 表示永不过期）';
COMMENT ON COLUMN core_memory.version IS '版本号（用于乐观锁，每次更新递增）';

-- ============================================================================
-- 5. Add fields to working_memory table
-- ============================================================================

-- working_memory already has expires_at, only add embedding and version
ALTER TABLE working_memory 
ADD COLUMN IF NOT EXISTS embedding TEXT,
ADD COLUMN IF NOT EXISTS version INTEGER NOT NULL DEFAULT 1;

COMMENT ON COLUMN working_memory.embedding IS '向量嵌入（JSON 格式存储，用于向量搜索）';
COMMENT ON COLUMN working_memory.version IS '版本号（用于乐观锁，每次更新递增）';

-- ============================================================================
-- 6. Verification queries
-- ============================================================================

-- Verify all columns exist
DO $$
DECLARE
    missing_columns TEXT := '';
BEGIN
    -- Check episodic_events
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'episodic_events' AND column_name = 'embedding'
    ) THEN
        missing_columns := missing_columns || 'episodic_events.embedding, ';
    END IF;
    
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'episodic_events' AND column_name = 'expires_at'
    ) THEN
        missing_columns := missing_columns || 'episodic_events.expires_at, ';
    END IF;
    
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'episodic_events' AND column_name = 'version'
    ) THEN
        missing_columns := missing_columns || 'episodic_events.version, ';
    END IF;
    
    -- Check semantic_memory
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'semantic_memory' AND column_name = 'embedding'
    ) THEN
        missing_columns := missing_columns || 'semantic_memory.embedding, ';
    END IF;
    
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'semantic_memory' AND column_name = 'expires_at'
    ) THEN
        missing_columns := missing_columns || 'semantic_memory.expires_at, ';
    END IF;
    
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'semantic_memory' AND column_name = 'version'
    ) THEN
        missing_columns := missing_columns || 'semantic_memory.version, ';
    END IF;
    
    -- Check procedural_memory
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'procedural_memory' AND column_name = 'embedding'
    ) THEN
        missing_columns := missing_columns || 'procedural_memory.embedding, ';
    END IF;
    
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'procedural_memory' AND column_name = 'expires_at'
    ) THEN
        missing_columns := missing_columns || 'procedural_memory.expires_at, ';
    END IF;
    
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'procedural_memory' AND column_name = 'version'
    ) THEN
        missing_columns := missing_columns || 'procedural_memory.version, ';
    END IF;
    
    -- Check core_memory
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'core_memory' AND column_name = 'embedding'
    ) THEN
        missing_columns := missing_columns || 'core_memory.embedding, ';
    END IF;
    
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'core_memory' AND column_name = 'expires_at'
    ) THEN
        missing_columns := missing_columns || 'core_memory.expires_at, ';
    END IF;
    
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'core_memory' AND column_name = 'version'
    ) THEN
        missing_columns := missing_columns || 'core_memory.version, ';
    END IF;
    
    -- Check working_memory
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'working_memory' AND column_name = 'embedding'
    ) THEN
        missing_columns := missing_columns || 'working_memory.embedding, ';
    END IF;
    
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'working_memory' AND column_name = 'version'
    ) THEN
        missing_columns := missing_columns || 'working_memory.version, ';
    END IF;
    
    IF missing_columns != '' THEN
        RAISE EXCEPTION 'Migration incomplete. Missing columns: %', missing_columns;
    ELSE
        RAISE NOTICE 'Migration successful! All columns added.';
    END IF;
END $$;

-- ============================================================================
-- 7. Usage examples
-- ============================================================================

-- Example 1: Insert with embedding
-- INSERT INTO episodic_events (..., embedding, expires_at, version)
-- VALUES (..., '[0.1, 0.2, 0.3, ...]', NOW() + INTERVAL '7 days', 1);

-- Example 2: Update with optimistic locking
-- UPDATE episodic_events 
-- SET summary = 'Updated summary', version = version + 1
-- WHERE id = 'some-id' AND version = 1;  -- Will fail if version changed

-- Example 3: Query non-expired memories
-- SELECT * FROM episodic_events 
-- WHERE (expires_at IS NULL OR expires_at > NOW());

-- Example 4: Clean up expired memories
-- DELETE FROM episodic_events WHERE expires_at < NOW();

