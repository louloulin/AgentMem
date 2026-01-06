# Memory V4 最佳实践指南

## 概述

本指南提供 AgentMem V4 架构的最佳实践，帮助您充分利用 Memory V4 的强大功能。

## 核心概念

### Memory V4 结构

```rust
pub struct Memory {
    pub id: MemoryId,              // 唯一标识符
    pub content: Content,          // 多模态内容
    pub attributes: AttributeSet,  // 开放属性系统
    pub relations: RelationGraph,  // 关系图谱
    pub metadata: Metadata,        // 系统元数据
}
```

### 设计原则

1. **内容与元数据分离**: `content` 存储核心数据，`attributes` 存储描述性信息
2. **命名空间隔离**: 使用 `user`, `system`, `domain` 命名空间组织属性
3. **类型安全**: 使用强类型而非字符串
4. **关系优先**: 使用 `RelationGraph` 而非嵌入式引用

## 内容类型选择

### 1. 文本内容 (Text)

**适用场景**:
- 用户消息
- 文档片段
- 日志记录
- 简单描述

**示例**:

```rust
use agent_mem::{MemoryV4, Content};

let memory = MemoryV4 {
    id: Default::default(),
    content: Content::Text("用户询问产品价格".to_string()),
    attributes: Default::default(),
    relations: Default::default(),
    metadata: Default::default(),
};
```

**最佳实践**:
- ✅ 保持文本简洁，避免过长
- ✅ 使用 UTF-8 编码
- ❌ 不要在文本中嵌入 JSON 或 XML（使用 Structured 类型）

### 2. 结构化数据 (Structured)

**适用场景**:
- 用户配置
- 产品信息
- 复杂对象
- API 响应

**示例**:

```rust
use serde_json::json;

let memory = MemoryV4 {
    content: Content::Structured(json!({
        "type": "user_profile",
        "name": "张三",
        "age": 30,
        "preferences": {
            "food": ["pizza", "sushi"],
            "music": ["rock", "jazz"]
        }
    })),
    // ...
};
```

**最佳实践**:
- ✅ 使用标准 JSON 格式
- ✅ 定义清晰的 schema
- ✅ 避免深层嵌套（建议 ≤ 3 层）
- ❌ 不要存储二进制数据（使用 Binary 类型）

### 3. 向量数据 (Vector)

**适用场景**:
- 文本 embedding
- 图像特征
- 音频特征
- 语义表示

**示例**:

```rust
let memory = MemoryV4 {
    content: Content::Vector(vec![0.1, 0.2, 0.3, /* ... 1536 dimensions */]),
    attributes: AttributeSet::new()
        .with(AttributeKey::system("embedding_model"), AttributeValue::String("text-embedding-3-small".to_string()))
        .with(AttributeKey::system("dimension"), AttributeValue::Number(1536.0)),
    // ...
};
```

**最佳实践**:
- ✅ 记录 embedding 模型和维度
- ✅ 使用一致的向量维度
- ✅ 归一化向量（如需要）
- ❌ 不要混用不同模型的向量

### 4. 二进制数据 (Binary)

**适用场景**:
- 图片
- 音频
- 视频
- 文件

**示例**:

```rust
let image_data = std::fs::read("image.png")?;

let memory = MemoryV4 {
    content: Content::Binary(image_data),
    attributes: AttributeSet::new()
        .with(AttributeKey::system("mime_type"), AttributeValue::String("image/png".to_string()))
        .with(AttributeKey::system("size"), AttributeValue::Number(1024.0)),
    // ...
};
```

**最佳实践**:
- ✅ 记录 MIME 类型
- ✅ 记录文件大小
- ✅ 考虑压缩大文件
- ❌ 避免存储超大文件（建议 < 10MB）

### 5. 多模态内容 (Multimodal)

**适用场景**:
- 图文混合
- 视频 + 字幕
- 音频 + 转录
- 富媒体内容

**示例**:

```rust
let memory = MemoryV4 {
    content: Content::Multimodal(vec![
        Content::Text("这是一张美丽的风景照".to_string()),
        Content::Binary(image_data),
        Content::Structured(json!({
            "location": "杭州西湖",
            "date": "2025-11-13"
        })),
    ]),
    // ...
};
```

**最佳实践**:
- ✅ 按逻辑顺序组织内容
- ✅ 使用 attributes 描述整体
- ✅ 保持模态数量合理（建议 ≤ 5）
- ❌ 不要重复存储相同信息

## 属性系统使用

### 命名空间选择

#### 1. User 命名空间

**用途**: 用户相关的业务属性

```rust
let memory = MemoryV4 {
    attributes: AttributeSet::new()
        .with(AttributeKey::user("user_id"), AttributeValue::String("U123".to_string()))
        .with(AttributeKey::user("session_id"), AttributeValue::String("S456".to_string()))
        .with(AttributeKey::user("preference"), AttributeValue::String("pizza".to_string())),
    // ...
};
```

**最佳实践**:
- ✅ 存储用户 ID、会话 ID
- ✅ 存储用户偏好、设置
- ❌ 不要存储系统内部状态

#### 2. System 命名空间

**用途**: 系统内部属性

```rust
let memory = MemoryV4 {
    attributes: AttributeSet::new()
        .with(AttributeKey::system("importance"), AttributeValue::Number(0.8))
        .with(AttributeKey::system("created_at"), AttributeValue::String(Utc::now().to_rfc3339()))
        .with(AttributeKey::system("embedding_model"), AttributeValue::String("text-embedding-3-small".to_string())),
    // ...
};
```

**最佳实践**:
- ✅ 存储重要性、时间戳
- ✅ 存储模型信息、版本
- ❌ 不要存储业务逻辑数据

#### 3. Domain 命名空间

**用途**: 领域特定属性

```rust
let memory = MemoryV4 {
    attributes: AttributeSet::new()
        .with(AttributeKey::domain("product_id"), AttributeValue::String("P001".to_string()))
        .with(AttributeKey::domain("category"), AttributeValue::String("food".to_string()))
        .with(AttributeKey::domain("price"), AttributeValue::Number(29.99)),
    // ...
};
```

**最佳实践**:
- ✅ 存储业务实体 ID
- ✅ 存储分类、标签
- ✅ 存储业务指标
- ❌ 不要与 user 命名空间混淆

### 属性值类型

```rust
// 字符串
AttributeValue::String("value".to_string())

// 数字
AttributeValue::Number(42.0)

// 布尔
AttributeValue::Boolean(true)

// 数组
AttributeValue::Array(vec![
    AttributeValue::String("item1".to_string()),
    AttributeValue::String("item2".to_string()),
])

// 对象
AttributeValue::Object({
    let mut map = HashMap::new();
    map.insert("key".to_string(), AttributeValue::String("value".to_string()));
    map
})
```

**最佳实践**:
- ✅ 使用最简单的类型
- ✅ 避免深层嵌套
- ❌ 不要在属性中存储大量数据（使用 content）

## 关系图谱使用

### 基本关系

```rust
use agent_mem::{RelationGraph, Relation};

let memory = MemoryV4 {
    relations: RelationGraph::new()
        .add_relation(Relation {
            from: "user_123".into(),
            to: "product_456".into(),
            relation_type: "purchased".to_string(),
            weight: Some(1.0),
            metadata: Default::default(),
        })
        .add_relation(Relation {
            from: "user_123".into(),
            to: "user_789".into(),
            relation_type: "friend".to_string(),
            weight: Some(0.8),
            metadata: Default::default(),
        }),
    // ...
};
```

### 关系类型建议

**用户关系**:
- `friend`, `follower`, `colleague`
- `parent`, `child`, `sibling`

**业务关系**:
- `purchased`, `viewed`, `liked`
- `owns`, `manages`, `created`

**语义关系**:
- `similar_to`, `related_to`, `derived_from`
- `causes`, `requires`, `enables`

**最佳实践**:
- ✅ 使用标准化的关系类型
- ✅ 设置合理的权重 (0.0 - 1.0)
- ✅ 使用双向关系（如需要）
- ❌ 避免循环依赖

## 查询最佳实践

### 1. 简单查询

```rust
use agent_mem::Query;

// 适用于简单场景
let query = Query::from_string("查找披萨相关的记忆");
```

### 2. 结构化查询

```rust
use agent_mem::{Query, QueryIntent, Constraint, ComparisonOperator};

let query = Query::new(QueryIntent::natural_language("查找披萨"))
    .with_constraint(Constraint::Attribute {
        key: AttributeKey::user("user_id"),
        operator: ComparisonOperator::Equals,
        value: AttributeValue::String("U123".to_string()),
    })
    .with_constraint(Constraint::Attribute {
        key: AttributeKey::system("importance"),
        operator: ComparisonOperator::GreaterThan,
        value: AttributeValue::Number(0.5),
    })
    .with_limit(10);
```

### 3. 复杂查询

```rust
use agent_mem::{Preference, PreferenceType, TemporalPreference};

let query = Query::new(QueryIntent::natural_language("查找最近的重要记忆"))
    .with_constraint(Constraint::Attribute {
        key: AttributeKey::system("importance"),
        operator: ComparisonOperator::GreaterThan,
        value: AttributeValue::Number(0.7),
    })
    .with_preference(Preference {
        preference_type: PreferenceType::Temporal(TemporalPreference {
            prefer_recent: true,
            decay_factor: 0.1,
        }),
        weight: 0.8,
    })
    .with_limit(20);
```

**最佳实践**:
- ✅ 使用约束过滤结果
- ✅ 使用偏好排序结果
- ✅ 设置合理的 limit
- ❌ 避免过于复杂的查询

## 性能优化

### 1. 批量操作

```rust
// ✅ 推荐：批量插入
let memories = vec![memory1, memory2, memory3];
repository.batch_create(&memories).await?;

// ❌ 不推荐：逐个插入
for memory in memories {
    repository.create(&memory).await?;
}
```

### 2. 索引优化

```rust
// 为常用查询字段添加索引
let memory = MemoryV4 {
    attributes: AttributeSet::new()
        // 这些字段应该有索引
        .with(AttributeKey::user("user_id"), AttributeValue::String("U123".to_string()))
        .with(AttributeKey::system("created_at"), AttributeValue::String(Utc::now().to_rfc3339()))
        .with(AttributeKey::domain("category"), AttributeValue::String("food".to_string())),
    // ...
};
```

### 3. 缓存策略

```rust
// 缓存频繁访问的记忆
let cache = Arc::new(RwLock::new(HashMap::new()));

async fn get_memory_cached(id: &str, cache: &Arc<RwLock<HashMap<String, MemoryV4>>>) -> Result<MemoryV4> {
    // 先查缓存
    if let Some(memory) = cache.read().await.get(id) {
        return Ok(memory.clone());
    }
    
    // 缓存未命中，从数据库加载
    let memory = repository.find_by_id(id).await?;
    cache.write().await.insert(id.to_string(), memory.clone());
    
    Ok(memory)
}
```

## 错误处理

```rust
use agent_mem_traits::{AgentMemError, Result};

async fn create_memory_safe(memory: &MemoryV4) -> Result<MemoryV4> {
    // 验证输入
    if memory.content == Content::Text("".to_string()) {
        return Err(AgentMemError::validation_error("内容不能为空"));
    }
    
    // 执行操作
    match repository.create(memory).await {
        Ok(m) => Ok(m),
        Err(e) => {
            tracing::error!("创建记忆失败: {:?}", e);
            Err(e)
        }
    }
}
```

## 测试建议

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_memory() {
        let memory = MemoryV4 {
            id: Default::default(),
            content: Content::Text("测试内容".to_string()),
            attributes: AttributeSet::new()
                .with(AttributeKey::user("user_id"), AttributeValue::String("U123".to_string())),
            relations: Default::default(),
            metadata: Default::default(),
        };
        
        let result = repository.create(&memory).await;
        assert!(result.is_ok());
    }
}
```

## 总结

Memory V4 提供了强大而灵活的记忆管理能力。遵循本指南的最佳实践，您可以：

- ✅ 充分利用多模态内容
- ✅ 使用命名空间组织属性
- ✅ 构建复杂的关系网络
- ✅ 编写高效的查询
- ✅ 优化性能和可维护性

## 参考资源

- [V3 到 V4 迁移指南](../migration/v3_to_v4.md)
- [Memory V4 架构文档](../architecture/memory-v4.md)
- [Query V4 使用指南](query-v4.md)
- [API 参考文档](../api/memory-v4.md)

