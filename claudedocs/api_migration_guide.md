# AgentMem 2.6 API 迁移指南

**版本**: 2.6.0
**发布日期**: 2025-01-08
**状态**: ✅ 迁移指南

---

## 📋 目录

1. [概述](#概述)
2. [快速迁移](#快速迁移)
3. [详细映射](#详细映射)
4. [常见问题](#常见问题)
5. [最佳实践](#最佳实践)
6. [兼容性说明](#兼容性说明)

---

## 概述

### 为什么要迁移？

AgentMem 2.6 引入了统一的 API 设计，解决了旧 API 的以下问题：

- ❌ **功能重叠**: `add_memory`, `add_memory_fast`, `add_memory_v2` 做类似的事
- ❌ **命名混乱**: 没有统一的命名规范
- ❌ **参数复杂**: 相似功能的参数不一致
- ❌ **难以发现**: 103 个公共方法，用户不知道用哪个

### 新 API 的优势

- ✅ **简洁**: 核心方法从 103 个减少到 ~30 个（-71%）
- ✅ **直观**: 方法名称清晰明确
- ✅ **灵活**: Builder 模式支持高级配置
- ✅ **向后兼容**: 旧 API 标记废弃但仍可用

---

## 快速迁移

### 最常见的迁移模式

#### 1. 添加记忆

**旧 API**:
```rust
// ❌ 多种方法，不知道用哪个
let id = orchestrator.add_memory_fast(content, agent_id, user_id, None, None).await?;
let id = orchestrator.add_memory(content, agent_id, user_id, None, None).await?;
let id = orchestrator.add_memory_v2(content, agent_id, user_id, None, None, true, None, None).await?;
```

**新 API**:
```rust
// ✅ 统一的方法
let id = orchestrator.add(content).await?;
```

#### 2. 搜索记忆

**旧 API**:
```rust
// ❌ 复杂的参数和多个方法
let results = orchestrator.search_memories(query, agent_id, user_id, 10, None).await?;
let results = orchestrator.search_memories_hybrid(query, user_id, 10, None, None).await?;
let results = orchestrator.context_aware_rerank(results, query, user_id).await?;
```

**新 API**:
```rust
// ✅ 简单搜索
let results = orchestrator.search(query).await?;

// ✅ 高级搜索（Builder 模式）
let results = orchestrator
    .search_builder(query)
    .limit(20)
    .with_rerank(true)
    .execute()
    .await?;
```

#### 3. 批量添加

**旧 API**:
```rust
// ❌ 复杂的参数结构
let items = vec![
    (content1, agent_id, user_id, None, None),
    (content2, agent_id, user_id, None, None),
];
let ids = orchestrator.add_memories_batch(items).await?;

// 或者
let ids = orchestrator.add_memory_batch_optimized(contents, agent_id, user_id, metadata).await?;
```

**新 API**:
```rust
// ✅ 简单批量添加
let ids = orchestrator.add_batch(contents).await?;

// ✅ 高级批量操作（Builder 模式）
let ids = orchestrator
    .batch_add()
    .add_all(contents)
    .batch_size(50)
    .concurrency(5)
    .execute()
    .await?;
```

---

## 详细映射

### 记忆添加 API

| 旧 API | 新 API | 迁移说明 |
|--------|--------|---------|
| `add_memory_fast(content, agent_id, user_id, memory_type, metadata)` | `add(content)` | 使用默认参数 |
| `add_memory(content, agent_id, user_id, memory_type, metadata)` | `add(content)` | 使用默认参数 |
| `add_memory_v2(content, agent_id, user_id, run_id, metadata, infer, memory_type, prompt)` | `add_with_options(content, agent_id, user_id, memory_type, metadata)` | 需要显式指定参数 |
| `add_memories_batch(items)` | `add_batch(contents)` | 简化参数 |
| `add_memory_batch_optimized(contents, agent_id, user_id, metadata)` | `batch_add().execute()` | 使用 Builder 模式 |

#### 高级用法

```rust
// 旧 API - 复杂参数
let id = orchestrator.add_memory_v2(
    "Hello".to_string(),
    "agent1".to_string(),
    Some("user1".to_string()),
    Some("run1".to_string()),
    Some(metadata),
    true,
    Some("chat".to_string()),
    None,
).await?;

// 新 API - 清晰明确
let id = orchestrator.add_with_options(
    "Hello",
    "agent1",
    Some("user1"),
    Some(MemoryType::Chat),
    Some(metadata),
).await?;
```

### 记忆查询 API

| 旧 API | 新 API | 迁移说明 |
|--------|--------|---------|
| `get_memory(id)` | `get(id)` | 方法名简化 |
| `get_all_memories(agent_id, user_id)` | `get_all()` | 使用默认参数 |
| `get_all_memories_v2(agent_id, user_id, run_id, limit)` | `get_all()` | 使用默认参数 |

#### 高级用法

```rust
// 旧 API
let memories = orchestrator.get_all_memories_v2(
    "agent1".to_string(),
    Some("user1".to_string()),
    Some("run1".to_string()),
    Some(100),
).await?;

// 新 API - 更简洁
let memories = orchestrator.get_all().await?;
// 如果需要过滤，使用 Iterator
let memories: Vec<_> = memories.into_iter()
    .filter(|m| m.agent_id == "agent1")
    .take(100)
    .collect();
```

### 记忆更新 API

| 旧 API | 新 API | 迁移说明 |
|--------|--------|---------|
| `update_memory(id, data)` | `update(id, content)` | 简化参数 |

#### 迁移示例

```rust
// 旧 API
let mut data = HashMap::new();
data.insert("content".to_string(), serde_json::json!("new content"));
data.insert("metadata".to_string(), serde_json::json!(metadata));
let updated = orchestrator.update_memory(id, data).await?;

// 新 API
let updated = orchestrator.update(id, "new content").await?;
```

### 记忆删除 API

| 旧 API | 新 API | 迁移说明 |
|--------|--------|---------|
| `delete_memory(id)` | `delete(id)` | 方法名简化 |
| `delete_all_memories(agent_id, user_id, run_id)` | `delete_all()` | 使用默认参数 |

### 搜索 API

| 旧 API | 新 API | 迁移说明 |
|--------|--------|---------|
| `search_memories(query, agent_id, user_id, limit, memory_type)` | `search(query)` | 简单搜索 |
| `search_memories_hybrid(query, user_id, limit, threshold, filters)` | `search_builder(query)` | 高级搜索 |
| `context_aware_rerank(memories, query, user_id)` | `search_builder(query).with_rerank(true)` | 集成到 Builder |

#### 高级用法

```rust
// 旧 API - 多个步骤
let mut results = orchestrator.search_memories_hybrid(
    "query".to_string(),
    "user1".to_string(),
    20,
    Some(0.7),
    None,
).await?;
results = orchestrator.context_aware_rerank(results, "query", "user1").await?;

// 新 API - 链式调用
let results = orchestrator
    .search_builder("query")
    .limit(20)
    .with_threshold(0.7)
    .with_rerank(true)
    .execute()
    .await?;
```

### 多模态 API

| 旧 API | 新 API | 迁移说明 |
|--------|--------|---------|
| `add_image_memory(image_data, user_id, agent_id, metadata)` | `add_image(image_data, caption)` | 简化参数 |
| `add_audio_memory(audio_data, user_id, agent_id, metadata)` | `add_audio(audio_data, transcript)` | 简化参数 |
| `add_video_memory(video_data, user_id, agent_id, metadata)` | `add_video(video_data, description)` | 简化参数 |

#### 迁移示例

```rust
// 旧 API
let mut metadata = HashMap::new();
metadata.insert("caption".to_string(), "A beautiful sunset".to_string());
let result = orchestrator.add_image_memory(
    image_data,
    "user1".to_string(),
    "agent1".to_string(),
    Some(metadata),
).await?;

// 新 API
let id = orchestrator.add_image(
    image_data,
    Some("A beautiful sunset"),
).await?;
```

### 统计 API

| 旧 API | 新 API | 迁移说明 |
|--------|--------|---------|
| `get_stats(user_id)` | `stats()` | 使用默认参数 |
| `get_performance_stats()` | `performance_stats()` | 方法名一致 |
| `get_history(memory_id)` | `history(memory_id)` | 方法名简化 |

---

## 常见问题

### Q1: 旧 API 还能使用吗？

**A**: 是的！所有旧 API 都标记为 `#[deprecated]` 但仍然可用。编译器会显示警告，但代码不会中断。

```rust
// 仍然可以工作，但会有警告
let id = orchestrator.add_memory_fast(content, agent_id, user_id, None, None).await?;
// ⚠️  warning: use of deprecated function
```

### Q2: 如何处理非默认的 agent_id 和 user_id？

**A**: 新 API 使用默认值 `"default"`，如果需要自定义：

```rust
// 方法 1: 使用 `add_with_options`
let id = orchestrator.add_with_options(
    content,
    "custom_agent",
    Some("custom_user"),
    None,
    None,
).await?;

// 方法 2: 使用 BatchBuilder 设置默认值
let ids = orchestrator
    .batch_add()
    .with_agent_id("custom_agent".to_string())
    .with_user_id("custom_user".to_string())
    .add_all(contents)
    .execute()
    .await?;
```

### Q3: Builder 模式的性能开销？

**A**: Builder 模式是零成本抽象，编译后与直接调用相同。Builder 只在编译时存在，运行时没有额外开销。

### Q4: 如何迁移复杂的批量操作？

**A**: 使用 BatchBuilder 的链式调用：

```rust
// 旧 API
let items = vec![
    (content1, agent1.clone(), user1.clone(), Some(type1), meta1),
    (content2, agent2.clone(), user2.clone(), Some(type2), meta2),
    // ...
];
let ids = orchestrator.add_memories_batch(items).await?;

// 新 API - 方案 1: 如果参数相同
let ids = orchestrator
    .batch_add()
    .with_agent_id(agent_id)
    .add_all(contents)
    .execute()
    .await?;

// 新 API - 方案 2: 如果参数不同，分批处理
let mut all_ids = Vec::new();
for (content, agent_id, user_id, memory_type, metadata) in items {
    let id = orchestrator.add_with_options(
        &content,
        &agent_id,
        user_id.as_deref(),
        memory_type,
        metadata,
    ).await?;
    all_ids.push(id);
}
```

### Q5: 搜索过滤器的迁移？

**A**: 使用 Builder 的 `.with_filter()` 方法：

```rust
// 旧 API
let mut filters = HashMap::new();
filters.insert("category".to_string(), "important".to_string());
filters.insert("date".to_string(), "2025-01-08".to_string());
let results = orchestrator.search_memories_hybrid(
    query,
    user_id,
    10,
    None,
    Some(filters),
).await?;

// 新 API
let results = orchestrator
    .search_builder(query)
    .with_filter("category".to_string(), "important".to_string())
    .with_filter("date".to_string(), "2025-01-08".to_string())
    .execute()
    .await?;
```

---

## 最佳实践

### 1. 优先使用新 API

新 API 设计更加清晰和一致，优先使用：

```rust
// ✅ 推荐
let id = orchestrator.add(content).await?;

// ❌ 不推荐（会产生警告）
let id = orchestrator.add_memory_fast(content, agent_id, user_id, None, None).await?;
```

### 2. 使用 Builder 模式处理复杂配置

Builder 模式让代码更清晰：

```rust
// ✅ 推荐 - 清晰的链式调用
let results = orchestrator
    .search_builder(query)
    .limit(20)
    .with_rerank(true)
    .with_threshold(0.7)
    .execute()
    .await?;

// ❌ 不推荐 - 难以阅读
let results = orchestrator.search_memories_hybrid(
    query,
    user_id,
    20,
    Some(0.7),
    Some(filters),
).await?;
let results = orchestrator.context_aware_rerank(results, query, user_id).await?;
```

### 3. 利用类型推断

新 API 利用 Rust 类型推断减少代码：

```rust
// ✅ 推荐 - 类型推断
let id: Result<String> = orchestrator.add(content).await;

// ❌ 不推荐 - 冗余的类型标注
let id: Result<String> = orchestrator.add_with_options(
    content.to_string(),
    "default".to_string(),
    None,
    None,
    None,
).await;
```

### 4. 错误处理

新 API 返回统一的 `Result<T>`：

```rust
// ✅ 推荐 - 使用 `?` 操作符
match orchestrator.add(content).await {
    Ok(id) => println!("Added: {}", id),
    Err(e) => eprintln!("Error: {}", e),
}

// 或者
let id = orchestrator.add(content).await?;
```

---

## 兼容性说明

### 废弃时间表

- **2.6.0** (当前): 旧 API 标记为 `#[deprecated]`，仍然可用
- **2.7.0** (计划): 旧 API 仍可用，但文档将移除
- **3.0.0** (未来): 旧 API 可能被完全移除

### 迁移策略

#### 阶段 1: 立即迁移（推荐）

```rust
// 使用编译器警告找到所有废弃的 API
cargo build --workspace 2>&1 | grep "deprecated"

// 逐个替换为新 API
```

#### 阶段 2: 渐进迁移

如果代码量大，可以分批迁移：

1. 第 1 批: 核心功能（add, search, get）
2. 第 2 批: 批量操作（add_batch, batch_add）
3. 第 3 批: 多模态功能（add_image, add_audio, add_video）
4. 第 4 批: 统计功能（stats, history）

#### 阶段 3: 允许警告过渡期

暂时允许编译警告，但设置截止日期：

```toml
# Cargo.toml
[workspace.metadata.compat]
# 设置迁移截止日期
migration_deadline = "2025-06-01"
```

---

## 示例代码

### 完整的迁移示例

#### 旧代码

```rust
use agent_mem::MemoryOrchestrator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let orchestrator = MemoryOrchestrator::new_with_auto_config().await?;

    // 添加记忆
    let id = orchestrator.add_memory_fast(
        "Hello, world!".to_string(),
        "agent1".to_string(),
        Some("user1".to_string()),
        None,
        None,
    ).await?;

    // 搜索记忆
    let results = orchestrator.search_memories_hybrid(
        "Hello".to_string(),
        "user1".to_string(),
        10,
        None,
        None,
    ).await?;

    // 批量添加
    let contents = vec
!["Memory 1".to_string(), "Memory 2".to_string()];
    let items: Vec<_> = contents.iter().map(|c| {
        (c.clone()
, "agent1".to_string(), Some("user1".to_string()), None, None)
    }).collect();
    let ids = orchestrator.add_memories_batch(items).await?;

    Ok(())
}
```

#### 新代码

```rust
use agent_mem::MemoryOrchestrator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let orchestrator = MemoryOrchestrator::new_with_auto_config().await?;

    // 添加记忆 - 更简洁
    let id = orchestrator.add("Hello, world!").await?;

    // 搜索记忆 - 更清晰
    let results = orchestrator.search("Hello").await?;

    // 批量添加 - 更直观
    let ids = orchestrator.add_batch(vec
!["Memory 1", "Memory 2"]).await?;

    Ok(())
}
```

---

## 需要帮助？

### 文档资源

- [完整重构计划](./api1.md)
- [改造总结](./api_refactoring_summary.md)
- [API 文档](https://docs.rs/agent_mem)

### 社区支持

- GitHub Issues: https://github.com/your-org/agentmem/issues
- Discord: https://discord.gg/agentmem
- 邮件列表: agentmem@googlegroups.com

---

**文档版本**: 1.0
**最后更新**: 2025-01-08
**维护者**: AgentMem 开发团队
