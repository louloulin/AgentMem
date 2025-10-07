-- Create resource_memory table for resource memory management
-- 资源记忆表 - 存储多媒体文件元数据

CREATE TABLE IF NOT EXISTS resource_memory (
    -- 主键
    id VARCHAR(255) PRIMARY KEY,
    
    -- 租户隔离
    organization_id VARCHAR(255) NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    agent_id VARCHAR(255) NOT NULL,
    
    -- 文件信息
    original_filename VARCHAR(500) NOT NULL,  -- 原始文件名
    resource_type VARCHAR(50) NOT NULL CHECK (resource_type IN ('document', 'image', 'audio', 'video', 'other')),  -- 资源类型
    file_size BIGINT NOT NULL,  -- 文件大小（字节）
    file_hash VARCHAR(64) NOT NULL,  -- 文件哈希（SHA256）
    mime_type VARCHAR(100) NOT NULL,  -- MIME 类型
    storage_path TEXT NOT NULL,  -- 存储路径
    
    -- 压缩信息
    is_compressed BOOLEAN NOT NULL DEFAULT FALSE,  -- 是否压缩
    compressed_size BIGINT,  -- 压缩后大小
    
    -- 访问统计
    access_count BIGINT NOT NULL DEFAULT 0,  -- 访问次数
    last_accessed TIMESTAMPTZ NOT NULL DEFAULT NOW(),  -- 最后访问时间
    
    -- 标签和元数据
    tags TEXT[] NOT NULL DEFAULT '{}',  -- 标签数组
    custom_metadata JSONB NOT NULL DEFAULT '{}'::jsonb,  -- 自定义元数据
    
    -- 时间戳
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- 索引约束
    CONSTRAINT fk_organization FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE,
    CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_agent FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE,
    
    -- 确保文件哈希唯一（去重）
    CONSTRAINT unique_file_hash UNIQUE (file_hash, user_id)
);

-- 创建索引以优化查询性能
CREATE INDEX IF NOT EXISTS idx_resource_memory_user_id ON resource_memory(user_id);
CREATE INDEX IF NOT EXISTS idx_resource_memory_agent_id ON resource_memory(agent_id);
CREATE INDEX IF NOT EXISTS idx_resource_memory_resource_type ON resource_memory(resource_type);
CREATE INDEX IF NOT EXISTS idx_resource_memory_file_hash ON resource_memory(file_hash);
CREATE INDEX IF NOT EXISTS idx_resource_memory_created_at ON resource_memory(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_resource_memory_file_size ON resource_memory(file_size);

-- 创建 GIN 索引以支持数组和 JSONB 查询
CREATE INDEX IF NOT EXISTS idx_resource_memory_tags ON resource_memory USING GIN (tags);
CREATE INDEX IF NOT EXISTS idx_resource_memory_custom_metadata ON resource_memory USING GIN (custom_metadata);

-- 创建全文搜索索引
CREATE INDEX IF NOT EXISTS idx_resource_memory_filename_trgm ON resource_memory USING GIN (original_filename gin_trgm_ops);

-- 添加注释
COMMENT ON TABLE resource_memory IS '资源记忆表 - 存储多媒体文件元数据';
COMMENT ON COLUMN resource_memory.id IS '资源唯一标识符';
COMMENT ON COLUMN resource_memory.organization_id IS '组织 ID（多租户隔离）';
COMMENT ON COLUMN resource_memory.user_id IS '用户 ID';
COMMENT ON COLUMN resource_memory.agent_id IS 'Agent ID';
COMMENT ON COLUMN resource_memory.original_filename IS '原始文件名';
COMMENT ON COLUMN resource_memory.resource_type IS '资源类型（document, image, audio, video, other）';
COMMENT ON COLUMN resource_memory.file_size IS '文件大小（字节）';
COMMENT ON COLUMN resource_memory.file_hash IS '文件哈希（SHA256）';
COMMENT ON COLUMN resource_memory.mime_type IS 'MIME 类型';
COMMENT ON COLUMN resource_memory.storage_path IS '存储路径';
COMMENT ON COLUMN resource_memory.is_compressed IS '是否压缩';
COMMENT ON COLUMN resource_memory.compressed_size IS '压缩后大小（字节）';
COMMENT ON COLUMN resource_memory.access_count IS '访问次数';
COMMENT ON COLUMN resource_memory.last_accessed IS '最后访问时间';
COMMENT ON COLUMN resource_memory.tags IS '标签数组';
COMMENT ON COLUMN resource_memory.custom_metadata IS '自定义元数据（JSONB 格式）';

