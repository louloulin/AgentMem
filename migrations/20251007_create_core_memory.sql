-- Create core_memory_blocks table for core memory management
-- 核心记忆块表 - 存储 persona 和 human 块

CREATE TABLE IF NOT EXISTS core_memory_blocks (
    -- 主键
    id VARCHAR(255) PRIMARY KEY,
    
    -- 租户隔离
    organization_id VARCHAR(255) NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    agent_id VARCHAR(255) NOT NULL,
    
    -- 块信息
    block_type VARCHAR(50) NOT NULL CHECK (block_type IN ('persona', 'human')),  -- 块类型
    content TEXT NOT NULL,  -- 块内容
    importance REAL NOT NULL DEFAULT 0.5 CHECK (importance >= 0.0 AND importance <= 1.0),  -- 重要性评分
    
    -- 容量管理
    max_capacity INTEGER NOT NULL DEFAULT 2000,  -- 最大容量（字符数）
    current_size INTEGER NOT NULL DEFAULT 0,  -- 当前使用量（字符数）
    
    -- 访问统计
    access_count BIGINT NOT NULL DEFAULT 0,  -- 访问次数
    last_accessed TIMESTAMPTZ NOT NULL DEFAULT NOW(),  -- 最后访问时间
    
    -- 元数据
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    
    -- 时间戳
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- 索引约束
    CONSTRAINT fk_organization FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE,
    CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_agent FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE,
    
    -- 确保每个 agent 只有一个 persona 块和一个 human 块
    CONSTRAINT unique_block_per_agent UNIQUE (agent_id, block_type)
);

-- 创建索引以优化查询性能
CREATE INDEX IF NOT EXISTS idx_core_memory_blocks_user_id ON core_memory_blocks(user_id);
CREATE INDEX IF NOT EXISTS idx_core_memory_blocks_agent_id ON core_memory_blocks(agent_id);
CREATE INDEX IF NOT EXISTS idx_core_memory_blocks_block_type ON core_memory_blocks(block_type);
CREATE INDEX IF NOT EXISTS idx_core_memory_blocks_updated_at ON core_memory_blocks(updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_core_memory_blocks_importance ON core_memory_blocks(importance DESC);

-- 创建 GIN 索引以支持 JSONB 元数据查询
CREATE INDEX IF NOT EXISTS idx_core_memory_blocks_metadata ON core_memory_blocks USING GIN (metadata);

-- 创建全文搜索索引
CREATE INDEX IF NOT EXISTS idx_core_memory_blocks_content_trgm ON core_memory_blocks USING GIN (content gin_trgm_ops);

-- 添加注释
COMMENT ON TABLE core_memory_blocks IS '核心记忆块表 - 存储 persona 和 human 块';
COMMENT ON COLUMN core_memory_blocks.id IS '块唯一标识符';
COMMENT ON COLUMN core_memory_blocks.organization_id IS '组织 ID（多租户隔离）';
COMMENT ON COLUMN core_memory_blocks.user_id IS '用户 ID';
COMMENT ON COLUMN core_memory_blocks.agent_id IS 'Agent ID';
COMMENT ON COLUMN core_memory_blocks.block_type IS '块类型（persona 或 human）';
COMMENT ON COLUMN core_memory_blocks.content IS '块内容';
COMMENT ON COLUMN core_memory_blocks.importance IS '重要性评分（0.0-1.0）';
COMMENT ON COLUMN core_memory_blocks.max_capacity IS '最大容量（字符数）';
COMMENT ON COLUMN core_memory_blocks.current_size IS '当前使用量（字符数）';
COMMENT ON COLUMN core_memory_blocks.access_count IS '访问次数';
COMMENT ON COLUMN core_memory_blocks.last_accessed IS '最后访问时间';
COMMENT ON COLUMN core_memory_blocks.metadata IS '元数据（JSONB 格式）';

