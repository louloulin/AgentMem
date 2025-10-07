-- Create lifecycle_events table for memory lifecycle tracking
-- 生命周期事件表 - 追踪记忆的生命周期变化

CREATE TABLE IF NOT EXISTS lifecycle_events (
    -- 主键
    id SERIAL PRIMARY KEY,
    
    -- 租户隔离
    organization_id VARCHAR(255) NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    agent_id VARCHAR(255) NOT NULL,
    
    -- 记忆引用
    memory_id VARCHAR(255) NOT NULL,
    
    -- 事件信息
    event_type VARCHAR(50) NOT NULL CHECK (event_type IN (
        'created', 'accessed', 'updated', 'archived', 'restored', 
        'deprecated', 'deleted', 'importance_changed', 'expiration_set'
    )),
    
    -- 状态信息
    old_state VARCHAR(50),
    new_state VARCHAR(50) CHECK (new_state IN (
        'created', 'active', 'archived', 'deprecated', 'deleted'
    )),
    
    -- 元数据
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    
    -- 时间戳
    event_timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- 索引约束
    CONSTRAINT fk_organization FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE,
    CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_agent FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE
);

-- 创建索引以优化查询性能
CREATE INDEX IF NOT EXISTS idx_lifecycle_events_memory_id ON lifecycle_events(memory_id);
CREATE INDEX IF NOT EXISTS idx_lifecycle_events_user_id ON lifecycle_events(user_id);
CREATE INDEX IF NOT EXISTS idx_lifecycle_events_agent_id ON lifecycle_events(agent_id);
CREATE INDEX IF NOT EXISTS idx_lifecycle_events_event_type ON lifecycle_events(event_type);
CREATE INDEX IF NOT EXISTS idx_lifecycle_events_event_timestamp ON lifecycle_events(event_timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_lifecycle_events_new_state ON lifecycle_events(new_state);

-- 创建复合索引
CREATE INDEX IF NOT EXISTS idx_lifecycle_events_memory_timestamp ON lifecycle_events(memory_id, event_timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_lifecycle_events_user_timestamp ON lifecycle_events(user_id, event_timestamp DESC);

-- 创建 GIN 索引以支持 JSONB 元数据查询
CREATE INDEX IF NOT EXISTS idx_lifecycle_events_metadata ON lifecycle_events USING GIN (metadata);

-- 创建记忆状态表
CREATE TABLE IF NOT EXISTS memory_states (
    -- 主键
    memory_id VARCHAR(255) PRIMARY KEY,
    
    -- 租户隔离
    organization_id VARCHAR(255) NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    agent_id VARCHAR(255) NOT NULL,
    
    -- 当前状态
    current_state VARCHAR(50) NOT NULL CHECK (current_state IN (
        'created', 'active', 'archived', 'deprecated', 'deleted'
    )),
    
    -- 过期信息
    expires_at TIMESTAMPTZ,
    
    -- 统计信息
    access_count BIGINT NOT NULL DEFAULT 0,
    last_accessed TIMESTAMPTZ,
    
    -- 生命周期策略
    auto_archive_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    auto_delete_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    
    -- 元数据
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    
    -- 时间戳
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- 索引约束
    CONSTRAINT fk_organization FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE,
    CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_agent FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE
);

-- 创建索引以优化查询性能
CREATE INDEX IF NOT EXISTS idx_memory_states_user_id ON memory_states(user_id);
CREATE INDEX IF NOT EXISTS idx_memory_states_agent_id ON memory_states(agent_id);
CREATE INDEX IF NOT EXISTS idx_memory_states_current_state ON memory_states(current_state);
CREATE INDEX IF NOT EXISTS idx_memory_states_expires_at ON memory_states(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_memory_states_last_accessed ON memory_states(last_accessed DESC);

-- 创建 GIN 索引以支持 JSONB 元数据查询
CREATE INDEX IF NOT EXISTS idx_memory_states_metadata ON memory_states USING GIN (metadata);

-- 添加注释
COMMENT ON TABLE lifecycle_events IS '生命周期事件表 - 追踪记忆的生命周期变化';
COMMENT ON COLUMN lifecycle_events.id IS '事件唯一标识符';
COMMENT ON COLUMN lifecycle_events.memory_id IS '记忆 ID';
COMMENT ON COLUMN lifecycle_events.event_type IS '事件类型';
COMMENT ON COLUMN lifecycle_events.old_state IS '旧状态';
COMMENT ON COLUMN lifecycle_events.new_state IS '新状态';
COMMENT ON COLUMN lifecycle_events.metadata IS '事件元数据（JSONB 格式）';
COMMENT ON COLUMN lifecycle_events.event_timestamp IS '事件发生时间';

COMMENT ON TABLE memory_states IS '记忆状态表 - 存储记忆的当前状态';
COMMENT ON COLUMN memory_states.memory_id IS '记忆 ID（主键）';
COMMENT ON COLUMN memory_states.current_state IS '当前状态';
COMMENT ON COLUMN memory_states.expires_at IS '过期时间';
COMMENT ON COLUMN memory_states.access_count IS '访问次数';
COMMENT ON COLUMN memory_states.last_accessed IS '最后访问时间';
COMMENT ON COLUMN memory_states.auto_archive_enabled IS '是否启用自动归档';
COMMENT ON COLUMN memory_states.auto_delete_enabled IS '是否启用自动删除';

