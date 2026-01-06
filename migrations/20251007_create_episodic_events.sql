-- Create episodic_events table for episodic memory management
-- 情景记忆事件表 - 存储基于时间的事件和经历

CREATE TABLE IF NOT EXISTS episodic_events (
    -- 主键
    id VARCHAR(255) PRIMARY KEY,
    
    -- 租户隔离
    organization_id VARCHAR(255) NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    agent_id VARCHAR(255) NOT NULL,
    
    -- 时间信息
    occurred_at TIMESTAMPTZ NOT NULL,  -- 事件发生时间
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- 事件信息
    event_type VARCHAR(100) NOT NULL,  -- 事件类型：conversation, action, observation
    actor VARCHAR(255),  -- 参与者
    summary TEXT NOT NULL,  -- 事件摘要
    details TEXT,  -- 事件详情
    
    -- 重要性评分
    importance_score REAL NOT NULL DEFAULT 0.5 CHECK (importance_score >= 0.0 AND importance_score <= 1.0),
    
    -- 元数据
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    
    -- 索引约束
    CONSTRAINT fk_organization FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE,
    CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_agent FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE
);

-- 创建索引以优化查询性能
CREATE INDEX IF NOT EXISTS idx_episodic_events_user_id ON episodic_events(user_id);
CREATE INDEX IF NOT EXISTS idx_episodic_events_agent_id ON episodic_events(agent_id);
CREATE INDEX IF NOT EXISTS idx_episodic_events_occurred_at ON episodic_events(occurred_at DESC);
CREATE INDEX IF NOT EXISTS idx_episodic_events_event_type ON episodic_events(event_type);
CREATE INDEX IF NOT EXISTS idx_episodic_events_importance ON episodic_events(importance_score DESC);
CREATE INDEX IF NOT EXISTS idx_episodic_events_user_occurred ON episodic_events(user_id, occurred_at DESC);

-- 创建 GIN 索引以支持 JSONB 元数据查询
CREATE INDEX IF NOT EXISTS idx_episodic_events_metadata ON episodic_events USING GIN (metadata);

-- 添加注释
COMMENT ON TABLE episodic_events IS '情景记忆事件表 - 存储基于时间的事件和经历';
COMMENT ON COLUMN episodic_events.id IS '事件唯一标识符';
COMMENT ON COLUMN episodic_events.organization_id IS '组织 ID（多租户隔离）';
COMMENT ON COLUMN episodic_events.user_id IS '用户 ID';
COMMENT ON COLUMN episodic_events.agent_id IS 'Agent ID';
COMMENT ON COLUMN episodic_events.occurred_at IS '事件发生时间';
COMMENT ON COLUMN episodic_events.event_type IS '事件类型（conversation, action, observation 等）';
COMMENT ON COLUMN episodic_events.actor IS '事件参与者';
COMMENT ON COLUMN episodic_events.summary IS '事件摘要';
COMMENT ON COLUMN episodic_events.details IS '事件详细信息';
COMMENT ON COLUMN episodic_events.importance_score IS '重要性评分（0.0-1.0）';
COMMENT ON COLUMN episodic_events.metadata IS '事件元数据（JSONB 格式）';

