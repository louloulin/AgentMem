-- Create procedural_memory table for procedural memory management
-- 程序记忆表 - 存储工作流、步骤、方法

CREATE TABLE IF NOT EXISTS procedural_memory (
    -- 主键
    id VARCHAR(255) PRIMARY KEY,
    
    -- 租户隔离
    organization_id VARCHAR(255) NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    agent_id VARCHAR(255) NOT NULL,
    
    -- 程序信息
    entry_type VARCHAR(100) NOT NULL,  -- 条目类型：workflow, guide, script
    summary TEXT NOT NULL,  -- 摘要
    steps JSONB NOT NULL DEFAULT '[]'::jsonb,  -- 步骤列表
    
    -- 层级分类
    tree_path JSONB NOT NULL DEFAULT '[]'::jsonb,  -- 层级路径，如：["workflows", "development", "testing"]
    
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
CREATE INDEX IF NOT EXISTS idx_procedural_memory_user_id ON procedural_memory(user_id);
CREATE INDEX IF NOT EXISTS idx_procedural_memory_agent_id ON procedural_memory(agent_id);
CREATE INDEX IF NOT EXISTS idx_procedural_memory_entry_type ON procedural_memory(entry_type);
CREATE INDEX IF NOT EXISTS idx_procedural_memory_updated_at ON procedural_memory(updated_at DESC);

-- 创建 GIN 索引以支持 JSONB 查询
CREATE INDEX IF NOT EXISTS idx_procedural_memory_steps ON procedural_memory USING GIN (steps);
CREATE INDEX IF NOT EXISTS idx_procedural_memory_tree_path ON procedural_memory USING GIN (tree_path);
CREATE INDEX IF NOT EXISTS idx_procedural_memory_metadata ON procedural_memory USING GIN (metadata);

-- 创建全文搜索索引
CREATE INDEX IF NOT EXISTS idx_procedural_memory_summary_trgm ON procedural_memory USING GIN (summary gin_trgm_ops);

-- 添加注释
COMMENT ON TABLE procedural_memory IS '程序记忆表 - 存储工作流、步骤、方法';
COMMENT ON COLUMN procedural_memory.id IS '记忆唯一标识符';
COMMENT ON COLUMN procedural_memory.organization_id IS '组织 ID（多租户隔离）';
COMMENT ON COLUMN procedural_memory.user_id IS '用户 ID';
COMMENT ON COLUMN procedural_memory.agent_id IS 'Agent ID';
COMMENT ON COLUMN procedural_memory.entry_type IS '条目类型（workflow, guide, script）';
COMMENT ON COLUMN procedural_memory.summary IS '程序摘要';
COMMENT ON COLUMN procedural_memory.steps IS '步骤列表（JSONB 数组）';
COMMENT ON COLUMN procedural_memory.tree_path IS '层级分类路径（JSONB 数组）';
COMMENT ON COLUMN procedural_memory.metadata IS '元数据（JSONB 格式）';

