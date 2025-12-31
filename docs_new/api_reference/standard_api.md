# Level 2: 标准 API

AgentMem 的标准 API，提供更多功能和选项。

## 核心 API

### 带选项的添加

```rust
use agentmem::{Memory, AddMemoryOptions, MemoryType};
use std::collections::HashMap;

let mut metadata = HashMap::new();
metadata.insert("category".to_string(), "preference".into());
metadata.insert("source".to_string(), "survey".into());

let options = AddMemoryOptions::builder()
    .content("用户喜欢科幻电影")
    .memory_type(MemoryType::SEMANTIC)
    .agent_id("agent_1")
    .session_id(Some("session_123"))
    .importance(0.8)
    .metadata(metadata)
    .build();

let result = memory.addWithOptions(options).await?;
```

### 带选项的搜索

```rust
use agentmem::{SearchOptions, MemoryType};

let options = SearchOptions::builder()
    .query("电影")
    .memory_type(Some(MemoryType::SEMANTIC))
    .agent_id(Some("agent_1"))
    .session_id(Some("session_123"))
    .limit(10)
    .threshold(0.7)
    .build();

let results = memory.searchWithOptions(options).await?;
```

## 会话管理

### create_session()

创建新会话。

```rust
let session = memory.create_session("user_123").await?;
println!("会话 ID: {}", session.id);
```

### get_session_history()

获取会话历史。

```rust
let history = memory.get_session_history("session_id").await?;

for mem in history {
    println!("{}", mem.content);
}
```

### create_sessionWithOptions()

创建带选项的会话。

```rust
use agentmem::SessionOptions;

let options = SessionOptions::builder()
    .agent_id("agent_1")
    .user_id("user_123")
    .metadata(Map::from([("title", "产品咨询")]))
    .build();

let session = memory.create_sessionWithOptions(options).await?;
```

## 批量操作

### add_batch()

批量添加记忆。

```rust
let items = vec![
    "记忆 1",
    "记忆 2",
    "记忆 3",
];

let results = memory.add_batch(
    items,
    None,
    MemoryType::SEMANTIC
).await?;

println!("成功添加 {} 条记忆", results.len());
```

### delete_batch()

批量删除记忆。

```rust
let ids = vec!["id1", "id2", "id3"];
memory.delete_batch(&ids).await?;
```

### get_batch()

批量获取记忆。

```rust
let ids = vec!["id1", "id2", "id3"];
let memories = memory.get_batch(&ids).await?;
```

## 更新操作

### update()

更新记忆。

```rust
use agentmem::UpdateMemoryOptions;

let options = UpdateMemoryOptions::builder()
    .content("更新后的内容")
    .importance(0.9)
    .build();

memory.update("memory_id", options).await?;
```

### update_batch()

批量更新。

```rust
let updates = vec![
    ("id1", UpdateMemoryOptions::builder().content("新1").build()),
    ("id2", UpdateMemoryOptions::builder().content("新2").build()),
];

memory.update_batch(updates).await?;
```

## 高级搜索

### 时间范围搜索

```rust
use chrono::{Utc, Duration};

let options = SearchOptions::builder()
    .query("项目进展")
    .after(Some(Utc::now() - Duration::days(7)))
    .before(Some(Utc::now()))
    .build();

let results = memory.searchWithOptions(options).await?;
```

### 元数据过滤

```rust
use std::collections::HashMap;

let mut filter = HashMap::new();
filter.insert("category".to_string(), "preference".into());

let options = SearchOptions::builder()
    .query("用户偏好")
    .metadata_filter(filter)
    .build();

let results = memory.searchWithOptions(options).await?;
```

### 分页搜索

```rust
// 第一页
let page1 = memory.search(
    SearchOptions::builder()
        .query("查询")
        .limit(10)
        .offset(0)
        .build()
).await?;

// 第二页
let page2 = memory.search(
    SearchOptions::builder()
        .query("查询")
        .limit(10)
        .offset(10)
        .build()
).await?;
```

## 统计和分析

### get_stats()

获取统计信息。

```rust
let stats = memory.get_stats().await?;

println!("总记忆数: {}", stats.total_memories);
println!("按类型分布:");
for (mem_type, count) in stats.by_type {
    println!("  {:?}: {}", mem_type, count);
}
```

### get_memory_stats()

获取详细统计。

```rust
use agentmem::MemoryStats;

let stats = memory.get_memory_stats(
    GetStatsOptions::builder()
        .agent_id(Some("agent_1"))
        .memory_type(Some(MemoryType::SEMANTIC))
        .build()
).await?;
```

## 完整示例

```rust
use agentmem::{
    Memory, MemoryType, AddMemoryOptions,
    SearchOptions, SessionOptions, GetAllOptions
};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let memory = Memory::new().await?;

    // 创建会话
    let session = memory.create_sessionWithOptions(
        SessionOptions::builder()
            .user_id("user_123")
            .metadata(Map::from([("title", "咨询对话")]))
            .build()
    ).await?;

    // 添加对话历史
    let mut metadata = HashMap::new();
    metadata.insert("role".to_string(), "user".into());

    memory.addWithOptions(
        AddMemoryOptions::builder()
            .content("你好，我想了解产品价格")
            .memory_type(MemoryType::EPISODIC)
            .session_id(Some(session.id.clone()))
            .metadata(metadata.clone())
            .build()
    ).await?;

    metadata.insert("role".to_string(), "assistant".into());

    memory.addWithOptions(
        AddMemoryOptions::builder()
            .content("我们的产品价格从 $99 到 $299")
            .memory_type(MemoryType::EPISODIC)
            .session_id(Some(session.id.clone()))
            .metadata(metadata)
            .build()
    ).await?;

    // 搜索相关内容
    let results = memory.searchWithOptions(
        SearchOptions::builder()
            .query("价格信息")
            .session_id(Some(session.id.clone()))
            .threshold(0.5)
            .limit(5)
            .build()
    ).await?;

    println!("找到 {} 条相关记忆:", results.len());
    for (i, result) in results.iter().enumerate() {
        println!("  {}. {}", i + 1, result.content);
    }

    // 获取会话历史
    let history = memory.get_session_history(&session.id).await?;
    println!("\n会话历史 ({} 条):", history.len());
    for mem in &history {
        println!("  {}", mem.content);
    }

    Ok(())
}
```

## Python SDK

### 标准API

```python
from agentmem import Memory, MemoryType, AddMemoryOptions, SearchOptions

# 创建实例
memory = await Memory.new()

# 创建会话
session = await memory.create_session(
    user_id="user_123",
    metadata={"title": "咨询对话"}
)

# 添加对话
await memory.addWithOptions(
    AddMemoryOptions(
        content="你好，我想了解产品价格",
        memory_type=MemoryType.EPISODIC,
        session_id=session.id,
        metadata={"role": "user"}
    )
)

# 搜索
results = await memory.searchWithOptions(
    SearchOptions(
        query="价格信息",
        session_id=session.id,
        threshold=0.5,
        limit=5
    )
)

# 获取会话历史
history = await memory.get_session_history(session.id)
```

## 下一步

- [高级 API](advanced_api.md) - 高级特性和自定义
- [教程](../tutorials/) - 完整的使用指南
