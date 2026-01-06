-- Migration: Create memory_history table for tracking memory changes
-- Date: 2025-10-15
-- Description: Adds support for memory version tracking and history

-- ============================================================================
-- 1. Create memory_history table
-- ============================================================================

CREATE TABLE IF NOT EXISTS memory_history (
    -- 主键
    id VARCHAR(255) PRIMARY KEY,
    
    -- 关联的记忆 ID
    memory_id VARCHAR(255) NOT NULL,
    
    -- 版本信息
    version INTEGER NOT NULL,
    
    -- 变更类型
    change_type VARCHAR(50) NOT NULL CHECK (change_type IN ('created', 'updated', 'deleted', 'restored')),
    
    -- 变更原因
    change_reason TEXT,
    
    -- 快照数据
    content TEXT NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    memory_type VARCHAR(50) NOT NULL,
    importance REAL NOT NULL DEFAULT 0.0,
    
    -- 租户隔离
    organization_id VARCHAR(255) NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    agent_id VARCHAR(255) NOT NULL,
    
    -- 变更者信息
    changed_by_id VARCHAR(255),
    
    -- 时间戳
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- 外键约束
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
);

-- ============================================================================
-- 2. Create indexes for performance
-- ============================================================================

-- Index for querying history by memory_id
CREATE INDEX IF NOT EXISTS idx_memory_history_memory_id 
ON memory_history(memory_id);

-- Index for querying history by memory_id and version
CREATE INDEX IF NOT EXISTS idx_memory_history_memory_version 
ON memory_history(memory_id, version DESC);

-- Index for querying history by change_type
CREATE INDEX IF NOT EXISTS idx_memory_history_change_type 
ON memory_history(change_type);

-- Index for querying history by created_at
CREATE INDEX IF NOT EXISTS idx_memory_history_created_at 
ON memory_history(created_at DESC);

-- Composite index for tenant isolation
CREATE INDEX IF NOT EXISTS idx_memory_history_tenant 
ON memory_history(organization_id, user_id, agent_id);

-- ============================================================================
-- 3. Create trigger function for automatic history tracking
-- ============================================================================

CREATE OR REPLACE FUNCTION track_memory_changes()
RETURNS TRIGGER AS $$
DECLARE
    next_version INTEGER;
    change_type_val VARCHAR(50);
BEGIN
    -- Determine change type
    IF TG_OP = 'INSERT' THEN
        change_type_val := 'created';
        next_version := 1;
    ELSIF TG_OP = 'UPDATE' THEN
        change_type_val := 'updated';
        -- Get next version number
        SELECT COALESCE(MAX(version), 0) + 1 INTO next_version
        FROM memory_history
        WHERE memory_id = NEW.id;
    ELSIF TG_OP = 'DELETE' THEN
        change_type_val := 'deleted';
        -- Get next version number
        SELECT COALESCE(MAX(version), 0) + 1 INTO next_version
        FROM memory_history
        WHERE memory_id = OLD.id;
        
        -- Insert history record for deletion
        INSERT INTO memory_history (
            id,
            memory_id,
            version,
            change_type,
            change_reason,
            content,
            metadata,
            memory_type,
            importance,
            organization_id,
            user_id,
            agent_id,
            changed_by_id,
            created_at
        ) VALUES (
            gen_random_uuid()::TEXT,
            OLD.id,
            next_version,
            change_type_val,
            'Memory deleted',
            OLD.content,
            OLD.metadata,
            OLD.memory_type,
            OLD.importance,
            OLD.organization_id,
            OLD.user_id,
            OLD.agent_id,
            OLD.last_updated_by_id,
            NOW()
        );
        
        RETURN OLD;
    END IF;
    
    -- Insert history record for INSERT or UPDATE
    INSERT INTO memory_history (
        id,
        memory_id,
        version,
        change_type,
        change_reason,
        content,
        metadata,
        memory_type,
        importance,
        organization_id,
        user_id,
        agent_id,
        changed_by_id,
        created_at
    ) VALUES (
        gen_random_uuid()::TEXT,
        NEW.id,
        next_version,
        change_type_val,
        CASE 
            WHEN TG_OP = 'INSERT' THEN 'Initial version'
            WHEN TG_OP = 'UPDATE' THEN 'Memory updated'
            ELSE NULL
        END,
        NEW.content,
        NEW.metadata,
        NEW.memory_type,
        NEW.importance,
        NEW.organization_id,
        NEW.user_id,
        NEW.agent_id,
        NEW.last_updated_by_id,
        NOW()
    );
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- 4. Create triggers on memories table
-- ============================================================================

-- Drop existing triggers if they exist
DROP TRIGGER IF EXISTS trigger_memory_insert ON memories;
DROP TRIGGER IF EXISTS trigger_memory_update ON memories;
DROP TRIGGER IF EXISTS trigger_memory_delete ON memories;

-- Create trigger for INSERT
CREATE TRIGGER trigger_memory_insert
AFTER INSERT ON memories
FOR EACH ROW
EXECUTE FUNCTION track_memory_changes();

-- Create trigger for UPDATE
CREATE TRIGGER trigger_memory_update
AFTER UPDATE ON memories
FOR EACH ROW
WHEN (OLD.* IS DISTINCT FROM NEW.*)
EXECUTE FUNCTION track_memory_changes();

-- Create trigger for DELETE
CREATE TRIGGER trigger_memory_delete
BEFORE DELETE ON memories
FOR EACH ROW
EXECUTE FUNCTION track_memory_changes();

-- ============================================================================
-- 5. Add comments for documentation
-- ============================================================================

COMMENT ON TABLE memory_history IS 'Tracks all changes to memories for version control and audit trail';
COMMENT ON COLUMN memory_history.version IS 'Sequential version number for each memory';
COMMENT ON COLUMN memory_history.change_type IS 'Type of change: created, updated, deleted, restored';
COMMENT ON COLUMN memory_history.change_reason IS 'Optional reason for the change';
COMMENT ON FUNCTION track_memory_changes() IS 'Automatically tracks changes to memories table';

