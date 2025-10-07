-- Create knowledge graph tables for entity and relation storage
-- 知识图谱表 - 存储实体和关系

-- 实体表
CREATE TABLE IF NOT EXISTS knowledge_entities (
    -- 主键
    id VARCHAR(255) PRIMARY KEY,
    
    -- 租户隔离
    organization_id VARCHAR(255) NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    
    -- 实体信息
    name VARCHAR(500) NOT NULL,
    entity_type VARCHAR(50) NOT NULL CHECK (entity_type IN (
        'person', 'organization', 'location', 'event', 
        'concept', 'object', 'custom'
    )),
    
    -- 属性和置信度
    properties JSONB NOT NULL DEFAULT '{}'::jsonb,
    confidence REAL NOT NULL CHECK (confidence >= 0.0 AND confidence <= 1.0),
    
    -- 时间戳
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- 外键约束
    CONSTRAINT fk_entity_organization FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE,
    CONSTRAINT fk_entity_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- 关系表
CREATE TABLE IF NOT EXISTS knowledge_relations (
    -- 主键
    id VARCHAR(255) PRIMARY KEY,
    
    -- 租户隔离
    organization_id VARCHAR(255) NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    
    -- 关系信息
    from_entity_id VARCHAR(255) NOT NULL,
    to_entity_id VARCHAR(255) NOT NULL,
    relation_type VARCHAR(50) NOT NULL CHECK (relation_type IN (
        'is_a', 'part_of', 'related_to', 'caused_by', 'leads',
        'similar_to', 'opposite_of', 'located_in', 'works_for', 'owns', 'custom'
    )),
    
    -- 属性和置信度
    properties JSONB NOT NULL DEFAULT '{}'::jsonb,
    confidence REAL NOT NULL CHECK (confidence >= 0.0 AND confidence <= 1.0),
    
    -- 时间戳
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- 外键约束
    CONSTRAINT fk_relation_organization FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE,
    CONSTRAINT fk_relation_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_relation_from_entity FOREIGN KEY (from_entity_id) REFERENCES knowledge_entities(id) ON DELETE CASCADE,
    CONSTRAINT fk_relation_to_entity FOREIGN KEY (to_entity_id) REFERENCES knowledge_entities(id) ON DELETE CASCADE,
    
    -- 唯一约束：同一对实体之间的同类型关系只能有一个
    CONSTRAINT unique_relation UNIQUE (from_entity_id, to_entity_id, relation_type)
);

-- 创建索引以优化查询性能

-- 实体表索引
CREATE INDEX IF NOT EXISTS idx_knowledge_entities_user_id ON knowledge_entities(user_id);
CREATE INDEX IF NOT EXISTS idx_knowledge_entities_org_id ON knowledge_entities(organization_id);
CREATE INDEX IF NOT EXISTS idx_knowledge_entities_type ON knowledge_entities(entity_type);
CREATE INDEX IF NOT EXISTS idx_knowledge_entities_name ON knowledge_entities(name);
CREATE INDEX IF NOT EXISTS idx_knowledge_entities_confidence ON knowledge_entities(confidence DESC);

-- 实体表复合索引
CREATE INDEX IF NOT EXISTS idx_knowledge_entities_user_type ON knowledge_entities(user_id, entity_type);
CREATE INDEX IF NOT EXISTS idx_knowledge_entities_user_name ON knowledge_entities(user_id, name);

-- 实体表 GIN 索引
CREATE INDEX IF NOT EXISTS idx_knowledge_entities_properties ON knowledge_entities USING GIN (properties);

-- 关系表索引
CREATE INDEX IF NOT EXISTS idx_knowledge_relations_user_id ON knowledge_relations(user_id);
CREATE INDEX IF NOT EXISTS idx_knowledge_relations_org_id ON knowledge_relations(organization_id);
CREATE INDEX IF NOT EXISTS idx_knowledge_relations_from ON knowledge_relations(from_entity_id);
CREATE INDEX IF NOT EXISTS idx_knowledge_relations_to ON knowledge_relations(to_entity_id);
CREATE INDEX IF NOT EXISTS idx_knowledge_relations_type ON knowledge_relations(relation_type);
CREATE INDEX IF NOT EXISTS idx_knowledge_relations_confidence ON knowledge_relations(confidence DESC);

-- 关系表复合索引
CREATE INDEX IF NOT EXISTS idx_knowledge_relations_from_type ON knowledge_relations(from_entity_id, relation_type);
CREATE INDEX IF NOT EXISTS idx_knowledge_relations_to_type ON knowledge_relations(to_entity_id, relation_type);
CREATE INDEX IF NOT EXISTS idx_knowledge_relations_user_type ON knowledge_relations(user_id, relation_type);

-- 关系表 GIN 索引
CREATE INDEX IF NOT EXISTS idx_knowledge_relations_properties ON knowledge_relations USING GIN (properties);

-- 添加注释
COMMENT ON TABLE knowledge_entities IS '知识图谱实体表 - 存储提取的实体';
COMMENT ON COLUMN knowledge_entities.id IS '实体唯一标识符';
COMMENT ON COLUMN knowledge_entities.name IS '实体名称';
COMMENT ON COLUMN knowledge_entities.entity_type IS '实体类型';
COMMENT ON COLUMN knowledge_entities.properties IS '实体属性（JSONB 格式）';
COMMENT ON COLUMN knowledge_entities.confidence IS '置信度（0.0-1.0）';

COMMENT ON TABLE knowledge_relations IS '知识图谱关系表 - 存储实体之间的关系';
COMMENT ON COLUMN knowledge_relations.id IS '关系唯一标识符';
COMMENT ON COLUMN knowledge_relations.from_entity_id IS '源实体 ID';
COMMENT ON COLUMN knowledge_relations.to_entity_id IS '目标实体 ID';
COMMENT ON COLUMN knowledge_relations.relation_type IS '关系类型';
COMMENT ON COLUMN knowledge_relations.properties IS '关系属性（JSONB 格式）';
COMMENT ON COLUMN knowledge_relations.confidence IS '置信度（0.0-1.0）';

