-- Create semantic_memory table for semantic memory management
-- 语义记忆表 - 存储通用知识、概念、事实

CREATE TABLE IF NOT EXISTS semantic_memory (
    -- 主键
    id VARCHAR(255) PRIMARY KEY,
    
    -- 租户隔离
    organization_id VARCHAR(255) NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    agent_id VARCHAR(255) NOT NULL,
    
    -- 概念信息
    name VARCHAR(500) NOT NULL,  -- 概念名称
    summary TEXT NOT NULL,  -- 摘要
    details TEXT NOT NULL,  -- 详细信息
    source VARCHAR(500),  -- 来源（书籍、文章、电影等）
    
    -- 层级分类
    tree_path JSONB NOT NULL DEFAULT '[]'::jsonb,  -- 层级路径，如：["favorites", "pets", "dog"]
    
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
CREATE INDEX IF NOT EXISTS idx_semantic_memory_user_id ON semantic_memory(user_id);
CREATE INDEX IF NOT EXISTS idx_semantic_memory_agent_id ON semantic_memory(agent_id);
CREATE INDEX IF NOT EXISTS idx_semantic_memory_name ON semantic_memory(name);
CREATE INDEX IF NOT EXISTS idx_semantic_memory_updated_at ON semantic_memory(updated_at DESC);

-- 创建 GIN 索引以支持 JSONB 查询
CREATE INDEX IF NOT EXISTS idx_semantic_memory_tree_path ON semantic_memory USING GIN (tree_path);
CREATE INDEX IF NOT EXISTS idx_semantic_memory_metadata ON semantic_memory USING GIN (metadata);

-- 创建全文搜索索引
CREATE INDEX IF NOT EXISTS idx_semantic_memory_name_trgm ON semantic_memory USING GIN (name gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_semantic_memory_summary_trgm ON semantic_memory USING GIN (summary gin_trgm_ops);

-- 添加注释
COMMENT ON TABLE semantic_memory IS '语义记忆表 - 存储通用知识、概念、事实';
COMMENT ON COLUMN semantic_memory.id IS '记忆唯一标识符';
COMMENT ON COLUMN semantic_memory.organization_id IS '组织 ID（多租户隔离）';
COMMENT ON COLUMN semantic_memory.user_id IS '用户 ID';
COMMENT ON COLUMN semantic_memory.agent_id IS 'Agent ID';
COMMENT ON COLUMN semantic_memory.name IS '概念名称';
COMMENT ON COLUMN semantic_memory.summary IS '概念摘要';
COMMENT ON COLUMN semantic_memory.details IS '详细信息';
COMMENT ON COLUMN semantic_memory.source IS '信息来源（书籍、文章、电影等）';
COMMENT ON COLUMN semantic_memory.tree_path IS '层级分类路径（JSONB 数组）';
COMMENT ON COLUMN semantic_memory.metadata IS '元数据（JSONB 格式）';

