-- Create memory_associations table for memory relationship tracking
-- 记忆关联表 - 追踪记忆之间的关系

CREATE TABLE IF NOT EXISTS memory_associations (
    -- 主键
    id VARCHAR(255) PRIMARY KEY,
    
    -- 租户隔离
    organization_id VARCHAR(255) NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    agent_id VARCHAR(255) NOT NULL,
    
    -- 关联的记忆
    from_memory_id VARCHAR(255) NOT NULL,
    to_memory_id VARCHAR(255) NOT NULL,
    
    -- 关联类型
    association_type VARCHAR(50) NOT NULL CHECK (association_type IN (
        'causal', 'temporal', 'similar', 'contrast', 
        'hierarchical', 'reference', 'custom'
    )),
    
    -- 关联强度和置信度
    strength REAL NOT NULL CHECK (strength >= 0.0 AND strength <= 1.0),
    confidence REAL NOT NULL CHECK (confidence >= 0.0 AND confidence <= 1.0),
    
    -- 元数据
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    
    -- 时间戳
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- 外键约束
    CONSTRAINT fk_organization FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE,
    CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_agent FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE,
    
    -- 唯一约束：同一对记忆之间的同类型关联只能有一个
    CONSTRAINT unique_association UNIQUE (from_memory_id, to_memory_id, association_type)
);

-- 创建索引以优化查询性能
CREATE INDEX IF NOT EXISTS idx_memory_associations_from ON memory_associations(from_memory_id);
CREATE INDEX IF NOT EXISTS idx_memory_associations_to ON memory_associations(to_memory_id);
CREATE INDEX IF NOT EXISTS idx_memory_associations_user_id ON memory_associations(user_id);
CREATE INDEX IF NOT EXISTS idx_memory_associations_agent_id ON memory_associations(agent_id);
CREATE INDEX IF NOT EXISTS idx_memory_associations_type ON memory_associations(association_type);
CREATE INDEX IF NOT EXISTS idx_memory_associations_strength ON memory_associations(strength DESC);

-- 创建复合索引
CREATE INDEX IF NOT EXISTS idx_memory_associations_from_type ON memory_associations(from_memory_id, association_type);
CREATE INDEX IF NOT EXISTS idx_memory_associations_to_type ON memory_associations(to_memory_id, association_type);
CREATE INDEX IF NOT EXISTS idx_memory_associations_user_type ON memory_associations(user_id, association_type);

-- 创建 GIN 索引以支持 JSONB 元数据查询
CREATE INDEX IF NOT EXISTS idx_memory_associations_metadata ON memory_associations USING GIN (metadata);

-- 添加注释
COMMENT ON TABLE memory_associations IS '记忆关联表 - 存储记忆之间的关系';
COMMENT ON COLUMN memory_associations.id IS '关联唯一标识符';
COMMENT ON COLUMN memory_associations.from_memory_id IS '源记忆 ID';
COMMENT ON COLUMN memory_associations.to_memory_id IS '目标记忆 ID';
COMMENT ON COLUMN memory_associations.association_type IS '关联类型';
COMMENT ON COLUMN memory_associations.strength IS '关联强度（0.0-1.0）';
COMMENT ON COLUMN memory_associations.confidence IS '置信度（0.0-1.0）';
COMMENT ON COLUMN memory_associations.metadata IS '关联元数据（JSONB 格式）';

