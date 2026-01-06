# AgentMem V3 到 V4 迁移指南

## 概述

AgentMem V4 引入了全新的架构，提供更强大和灵活的记忆管理能力。本指南将帮助您从 V3 (使用 `MemoryItem`) 迁移到 V4 (使用 `Memory`)。

### 主要变化

| 特性 | V3 (MemoryItem) | V4 (Memory) |
|------|----------------|-------------|
| **内容类型** | 仅文本 (String) | 多模态 (Text, Structured, Vector, Binary, Multimodal) |
| **元数据** | 固定字段 (HashMap) | 开放属性系统 (AttributeSet with namespaces) |
| **关系** | 简单列表 (Vec<Relation>) | 关系图谱 (RelationGraph) |
| **查询** | 字符串查询 (&str) | 强类型查询 (Query V4) |
| **类型安全** | 运行时检查 | 编译时检查 |

### 迁移策略

AgentMem V4 采用**渐进式迁移**策略：

- ✅ **V3 API 仍然可用**：`MemoryItem` 已标记为 `deprecated`，但仍可使用
- ✅ **向后兼容**：现有代码无需立即修改
- ✅ **平滑过渡**：可以逐步迁移，新代码使用 V4，旧代码继续使用 V3
- ⚠️ **未来移除**：V3 API 将在下一个主版本 (v2.0) 中移除

## 快速开始

### 1. 更新依赖

```toml
[dependencies]
agent-mem = "1.0"  # 或最新版本
agent-mem-traits = "1.0"
```

### 2. 导入 V4 类型

```rust
// V3 (已废弃)
use agent_mem::MemoryItem;

// V4 (推荐)
use agent_mem::MemoryV4;  // 或直接使用 Memory
use agent_mem::{AttributeKey, AttributeSet, AttributeValue, Content};
```

### 3. 基本使用对比

#### V3 方式 (已废弃)

```rust
use agent_mem::{Memory, MemoryItem};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mem = Memory::new().await?;
    
    // V3: 使用 MemoryItem
    let item = MemoryItem {
        id: "mem_001".to_string(),
        content: "用户喜欢披萨".to_string(),
        metadata: HashMap::new(),
        // ... 其他字段
    };
    
    Ok(())
}
```

#### V4 方式 (推荐)

```rust
use agent_mem::{Memory, MemoryV4, Content, AttributeSet, AttributeKey, AttributeValue};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mem = Memory::new().await?;
    
    // V4: 使用 Memory
    let memory = MemoryV4 {
        id: "mem_001".into(),
        content: Content::Text("用户喜欢披萨".to_string()),
        attributes: AttributeSet::new()
            .with(AttributeKey::user("preference"), AttributeValue::String("pizza".to_string()))
            .with(AttributeKey::system("importance"), AttributeValue::Number(0.8)),
        relations: Default::default(),
        metadata: Default::default(),
    };
    
    Ok(())
}
```

## 详细迁移指南

### 1. 内容类型迁移

#### V3: 仅支持文本

```rust
let item = MemoryItem {
    content: "这是文本内容".to_string(),
    // ...
};
```

#### V4: 支持多模态内容

```rust
use agent_mem::Content;

// 文本内容
let memory = MemoryV4 {
    content: Content::Text("这是文本内容".to_string()),
    // ...
};

// 结构化数据
let memory = MemoryV4 {
    content: Content::Structured(serde_json::json!({
        "type": "user_profile",
        "name": "张三",
        "age": 30
    })),
    // ...
};

// 向量数据
let memory = MemoryV4 {
    content: Content::Vector(vec![0.1, 0.2, 0.3, /* ... */]),
    // ...
};

// 二进制数据
let memory = MemoryV4 {
    content: Content::Binary(vec![0x89, 0x50, 0x4E, 0x47, /* PNG header */]),
    // ...
};

// 多模态内容
let memory = MemoryV4 {
    content: Content::Multimodal(vec![
        Content::Text("图片描述".to_string()),
        Content::Binary(image_data),
    ]),
    // ...
};
```

### 2. 元数据迁移

#### V3: 固定字段 + HashMap

```rust
use std::collections::HashMap;

let mut metadata = HashMap::new();
metadata.insert("user_id".to_string(), json!("U123"));
metadata.insert("importance".to_string(), json!(0.8));

let item = MemoryItem {
    metadata,
    user_id: Some("U123".to_string()),
    importance: 0.8,
    // ...
};
```

#### V4: 开放属性系统

```rust
use agent_mem::{AttributeSet, AttributeKey, AttributeValue};

let memory = MemoryV4 {
    attributes: AttributeSet::new()
        // 用户命名空间
        .with(AttributeKey::user("user_id"), AttributeValue::String("U123".to_string()))
        .with(AttributeKey::user("preference"), AttributeValue::String("pizza".to_string()))
        
        // 系统命名空间
        .with(AttributeKey::system("importance"), AttributeValue::Number(0.8))
        .with(AttributeKey::system("created_at"), AttributeValue::String(Utc::now().to_rfc3339()))
        
        // 领域命名空间
        .with(AttributeKey::domain("product_id"), AttributeValue::String("P001".to_string()))
        .with(AttributeKey::domain("category"), AttributeValue::String("food".to_string())),
    // ...
};
```

### 3. 关系迁移

#### V3: 简单关系列表

```rust
use agent_mem_traits::{Relation, Entity};

let item = MemoryItem {
    relations: vec![
        Relation {
            source: Entity { id: "E1".to_string(), entity_type: "Person".to_string() },
            target: Entity { id: "E2".to_string(), entity_type: "Product".to_string() },
            relation_type: "likes".to_string(),
        }
    ],
    // ...
};
```

#### V4: 关系图谱

```rust
use agent_mem::{RelationGraph, Relation};

let memory = MemoryV4 {
    relations: RelationGraph::new()
        .add_relation(Relation {
            from: "E1".into(),
            to: "E2".into(),
            relation_type: "likes".to_string(),
            weight: Some(0.9),
            metadata: Default::default(),
        }),
    // ...
};
```

### 4. 查询迁移

#### V3: 字符串查询

```rust
// 搜索
let results = mem.search("查找披萨相关的记忆").await?;
```

#### V4: 强类型查询

```rust
use agent_mem::{Query, QueryIntent, Constraint, ComparisonOperator};

// 方式1: 简单查询（向后兼容）
let query = Query::from_string("查找披萨相关的记忆");
let results = search_engine.search(&query).await?;

// 方式2: 结构化查询
let query = Query::new(QueryIntent::natural_language("查找披萨相关的记忆"))
    .with_constraint(Constraint::Attribute {
        key: AttributeKey::user("preference"),
        operator: ComparisonOperator::Equals,
        value: AttributeValue::String("pizza".to_string()),
    })
    .with_limit(10);

let results = search_engine.search(&query).await?;
```

## 常见迁移场景

### 场景 1: 简单文本记忆

#### V3 代码

```rust
let mem = Memory::new().await?;
mem.add("用户喜欢披萨").await?;
let results = mem.search("披萨").await?;
```

#### V4 代码

```rust
// 高层 API 保持不变（内部已使用 V4）
let mem = Memory::new().await?;
mem.add("用户喜欢披萨").await?;
let results = mem.search("披萨").await?;

// 或使用 V4 类型
let memory = MemoryV4 {
    id: Default::default(),
    content: Content::Text("用户喜欢披萨".to_string()),
    attributes: AttributeSet::new(),
    relations: Default::default(),
    metadata: Default::default(),
};
```

### 场景 2: 带元数据的记忆

#### V3 代码

```rust
use agent_mem::AddMemoryOptions;

let options = AddMemoryOptions {
    user_id: Some("U123".to_string()),
    agent_id: Some("A456".to_string()),
    metadata: {
        let mut m = HashMap::new();
        m.insert("category".to_string(), json!("preference"));
        m
    },
    ..Default::default()
};

mem.add_with_options("用户喜欢披萨".to_string(), options).await?;
```

#### V4 代码

```rust
// 高层 API 仍然支持（向后兼容）
let options = AddMemoryOptions {
    user_id: Some("U123".to_string()),
    agent_id: Some("A456".to_string()),
    metadata: {
        let mut m = HashMap::new();
        m.insert("category".to_string(), json!("preference"));
        m
    },
    ..Default::default()
};

mem.add_with_options("用户喜欢披萨".to_string(), options).await?;

// 或使用 V4 属性系统
let memory = MemoryV4 {
    id: Default::default(),
    content: Content::Text("用户喜欢披萨".to_string()),
    attributes: AttributeSet::new()
        .with(AttributeKey::user("user_id"), AttributeValue::String("U123".to_string()))
        .with(AttributeKey::user("agent_id"), AttributeValue::String("A456".to_string()))
        .with(AttributeKey::domain("category"), AttributeValue::String("preference".to_string())),
    relations: Default::default(),
    metadata: Default::default(),
};
```

## 最佳实践

### 1. 使用命名空间组织属性

```rust
// ✅ 推荐：使用命名空间
let memory = MemoryV4 {
    attributes: AttributeSet::new()
        .with(AttributeKey::user("user_id"), AttributeValue::String("U123".to_string()))
        .with(AttributeKey::system("importance"), AttributeValue::Number(0.8))
        .with(AttributeKey::domain("product_id"), AttributeValue::String("P001".to_string())),
    // ...
};

// ❌ 不推荐：混乱的属性
let memory = MemoryV4 {
    attributes: AttributeSet::new()
        .with(AttributeKey::custom("user_id"), AttributeValue::String("U123".to_string()))
        .with(AttributeKey::custom("importance"), AttributeValue::Number(0.8))
        .with(AttributeKey::custom("product_id"), AttributeValue::String("P001".to_string())),
    // ...
};
```

### 2. 选择合适的内容类型

```rust
// ✅ 推荐：根据数据类型选择
let text_memory = MemoryV4 {
    content: Content::Text("纯文本".to_string()),
    // ...
};

let structured_memory = MemoryV4 {
    content: Content::Structured(json!({"key": "value"})),
    // ...
};

// ❌ 不推荐：将结构化数据序列化为文本
let bad_memory = MemoryV4 {
    content: Content::Text(r#"{"key": "value"}"#.to_string()),
    // ...
};
```

### 3. 使用 Query Builder

```rust
// ✅ 推荐：使用 Query Builder
let query = Query::new(QueryIntent::natural_language("查找披萨"))
    .with_constraint(Constraint::Attribute {
        key: AttributeKey::user("preference"),
        operator: ComparisonOperator::Equals,
        value: AttributeValue::String("pizza".to_string()),
    })
    .with_limit(10);

// ✅ 也可以：简单查询
let query = Query::from_string("查找披萨");
```

## 迁移检查清单

- [ ] 更新依赖到最新版本
- [ ] 导入 V4 类型 (`MemoryV4`, `Content`, `AttributeSet` 等)
- [ ] 将 `MemoryItem` 替换为 `MemoryV4`
- [ ] 将字符串内容替换为 `Content::Text`
- [ ] 将 HashMap 元数据替换为 `AttributeSet`
- [ ] 更新查询代码使用 `Query` 类型
- [ ] 运行测试确保功能正常
- [ ] 更新文档和注释

## 常见问题

### Q1: 我必须立即迁移吗？

**A**: 不需要。V3 API (`MemoryItem`) 仍然可用，只是标记为 `deprecated`。您可以：
- 继续使用 V3 API（会有编译警告）
- 逐步迁移新代码到 V4
- 在方便的时候完成完整迁移

### Q2: V3 和 V4 可以混用吗？

**A**: 可以。AgentMem 内部提供了转换层，可以在 V3 和 V4 之间自动转换。但建议新代码统一使用 V4。

### Q3: 迁移会影响现有数据吗？

**A**: 不会。数据库 schema 没有变化，V4 使用转换层与数据库交互，现有数据完全兼容。

### Q4: 性能会受影响吗？

**A**: 不会。V4 的转换层经过优化，性能与 V3 相当甚至更好。

### Q5: 如何获取帮助？

**A**: 
- 查看示例代码：`examples/` 目录
- 阅读 API 文档：`docs/api/`
- 提交 Issue：GitHub Issues
- 加入社区：Discord/Slack

## 参考资源

- [Memory V4 架构文档](../architecture/memory-v4.md)
- [Query V4 使用指南](../guides/query-v4.md)
- [API 参考文档](../api/memory-v4.md)
- [示例代码](../../examples/)

## 更新日志

- **2025-11-13**: 初始版本，覆盖基本迁移场景

