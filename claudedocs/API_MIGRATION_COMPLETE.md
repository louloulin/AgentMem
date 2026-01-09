# AgentMem 2.6 API 迁移指南

**版本**: 2.6.0
**发布日期**: 2025-01-08
**状态**: 📘 正式发布

---

## 📊 快速参考：旧 API → 新 API

### 添加记忆

| 旧 API | 新 API | 说明 |
|--------|--------|------|
| `add_memory_fast(...)` | `add(content)` | ✨ 简化参数 |
| `add_memory(...)` | `add(content)` | ✨ 统一入口 |
| `add_memory_v2(...)` | `add(content)` | ✨ 智能处理 |
| `add_memories_batch(...)` | `add_batch(contents)` | ✨ 简化参数 |
| `add_memory_batch_optimized(...)` | `batch_add()...` | 🆕 Builder 模式 |

### 搜索记忆

| 旧 API | 新 API | 说明 |
|--------|--------|------|
| `search_memories(...)` | `search(query)` | ✨ 简化参数 |
| `search_memories_hybrid(...)` | `search_builder(query)...` | 🆕 Builder 模式 |
| `context_aware_rerank(...)` | `search_builder(query).with_rerank(true)` | 🆕 Builder 模式 |

### 其他操作

| 旧 API | 新 API | 说明 |
|--------|--------|------|
| `get_memory(id)` | `get(id)` | ✨ 简化名称 |
| `get_all_memories(...)` | `get_all()` | ✨ 无参数 |
| `update_memory(...)` | `update(id, content)` | ✨ 简化参数 |
| `delete_memory(id)` | `delete(id)` | ✨ 简化名称 |
| `delete_all_memories(...)` | `delete_all()` | ✨ 无参数 |
| `get_stats(...)` | `stats()` | ✨ 简化参数 |

---

## 🔄 迁移示例

### 场景 1: 添加记忆

#### ❌ 旧代码
```rust
let id = orchestrator
    .add_memory_fast(content, agent_id, user_id, None, None)
    .await?;
```

#### ✅ 新代码
```rust
let id = orchestrator.add(content).await?;
```

---

### 场景 2: 搜索记忆

#### ❌ 旧代码
```rust
let results = orchestrator
    .search_memories_hybrid(query, user_id, 10, None, None)
    .await?;

let results = orchestrator
    .context_aware_rerank(results, query, user_id)
    .await?;
```

#### ✅ 新代码（简单）
```rust
let results = orchestrator.search(query).await?;
```

#### ✅ 新代码（高级配置）
```rust
let results = orchestrator
    .search_builder(query)
    .limit(20)
    .with_rerank(true)
    .with_hybrid(true)
    .with_threshold(0.7)
    .with_time_range(start_ts, end_ts)
    .with_filter("category".to_string(), "urgent".to_string())
    .await?;
```

---

### 场景 3: 批量添加

#### ❌ 旧代码
```rust
let ids = orchestrator
    .add_memories_batch(
        contents.iter().map(|c| {
            (c.clone(), agent_id.clone(), Some(user_id.clone()), None, None)
        }).collect()
    )
    .await?;
```

#### ✅ 新代码（简单）
```rust
let ids = orchestrator.add_batch(contents).await?;
```

#### ✅ 新代码（高级配置）
```rust
let ids = orchestrator
    .batch_add()
    .add_all(contents)
    .with_agent_id("agent1".to_string())
    .with_user_id("user1".to_string())
    .batch_size(50)
    .await?;
```

---

## 🏗️ Builder 模式详解

### SearchBuilder

#### 创建方式
```rust
// 方式 1: 简单搜索
let results = orchestrator.search("query").await?;

// 方式 2: Builder 模式
let results = orchestrator
    .search_builder("query")
    .limit(20)
    .await?;

// 方式 3: 显式 execute
let results = orchestrator
    .search_builder("query")
    .limit(20)
    .execute()
    .await?;
```

#### 可用方法

| 方法 | 参数 | 说明 | 默认值 |
|------|------|------|--------|
| `limit(usize)` | 返回数量 | 设置返回结果数量 | `10` |
| `with_hybrid(bool)` | 是否启用 | 启用混合搜索 | `true` |
| `with_rerank(bool)` | 是否启用 | 启用重排序 | `true` |
| `with_threshold(f32)` | 阈值 | 设置相似度阈值 | `None` |
| `with_time_range(i64, i64)` | 起始, 结束 | 时间范围过滤 | `None` |
| `with_filter(String, String)` | 键, 值 | 自定义过滤器 | 空 |
| `execute()` | - | 执行搜索 | 可省略 |

#### 完整示例
```rust
use agent_mem::MemoryOrchestrator;

let orchestrator = MemoryOrchestrator::new_with_auto_config().await?;

// 基础搜索
let results = orchestrator
    .search_builder("important document")
    .await?;

// 高级配置
let results = orchestrator
    .search_builder("project update")
    .limit(20)
    .with_hybrid(true)
    .with_rerank(true)
    .with_threshold(0.7)
    .with_time_range(1704067200, 1706745600)
    .with_filter("category".to_string(), "work".to_string())
    .with_filter("priority".to_string(), "high".to_string())
    .await?;
```

---

### BatchBuilder

#### 创建方式
```rust
// 方式 1: 简单批量
let ids = orchestrator.add_batch(contents).await?;

// 方式 2: Builder 模式
let ids = orchestrator
    .batch_add()
    .add_all(contents)
    .await?;

// 方式 3: 逐个添加
let ids = orchestrator
    .batch_add()
    .add("Memory 1")
    .add("Memory 2")
    .add("Memory 3")
    .await?;
```

#### 可用方法

| 方法 | 参数 | 说明 | 默认值 |
|------|------|------|--------|
| `add(&str)` | 内容 | 添加单个内容 | - |
| `add_all(Vec<String>)` | 内容列表 | 批量添加 | - |
| `with_agent_id(String)` | ID | 设置 agent_id | `"default"` |
| `with_user_id(String)` | ID | 设置 user_id | `None` |
| `with_memory_type(MemoryType)` | 类型 | 设置记忆类型 | `None` |
| `batch_size(usize)` | 大小 | 批量大小 | `100` |
| `execute()` | - | 执行批量添加 | 可省略 |

#### 完整示例
```rust
use agent_mem::MemoryOrchestrator;
use agent_mem_core::types::MemoryType;

let orchestrator = MemoryOrchestrator::new_with_auto_config().await?;

// 简单批量
let ids = orchestrator
    .batch_add()
    .add_all(vec
!["M1", "M2", "M3"])
    .await?;

// 高级配置
let ids = orchestrator
    .batch_add()
    .add("First memory")
    .add("Second memory")
    .add_all(vec
!["Third", "Fourth"])
    .with_agent_id("agent1".to_string())
    .with_user_id("user1".to_string())
    .with_memory_type(MemoryType::Conversation)
    .batch_size(50)
    .await?;
```

---

## ❓ 常见问题

### Q1: 为什么要移除旧 API？

**A**: 旧 API 存在严重问题：
- 🔴 **命名混乱**: `add_memory_fast`, `add_memory_v2`, `add_memory_intelligent`
- 🔴 **功能重叠**: 多个方法做同样的事
- 🔴 **参数复杂**: 大量可选参数，不知道传什么

新 API 解决了所有这些问题：
- ✅ 统一命名：`add()`, `search()`, `get()`, `update()`, `delete()`
- ✅ 简化参数：合理的默认值
- ✅ Builder 模式：复杂场景提供灵活配置

### Q2: 性能会下降吗？

**A**: 不会！新 API 性能与旧 API 相同或更好：

```rust
// 旧 API
let ids = orchestrator
    .add_memory_batch_optimized(contents, agent_id, user_id, None, 100, 10)
    .await?;

// 新 API（相同性能）
let ids = orchestrator.add_batch(contents).await?;
```

### Q3: 如何迁移？

**A**: 分步进行：

1. **查找所有旧 API 调用**
   ```bash
   grep -r "add_memory_fast\|search_memories_hybrid" src/
   ```

2. **使用查找替换**
   - `add_memory_fast(...)` → `add(content)`
   - `search_memories(...)` → `search(query)`
   - `get_memory(id)` → `get(id)`

3. **复杂场景使用 Builder**
   - 多参数搜索 → `search_builder()...`
   - 批量操作配置 → `batch_add()...`

4. **编译测试**
   ```bash
   cargo build
   cargo test
   ```

### Q4: 旧 API 完全消失了吗？

**A**: 不，旧实现仍作为内部方法保留：

```rust
// crates/agent-mem/src/orchestrator/core.rs

#[allow(dead_code)]
pub(crate) async fn add_memory_fast(...) { ... }

#[allow(dead_code)]
pub(crate) async fn search_memories_hybrid(...) { ... }
```

- ✅ 内部代码仍可使用
- ✅ 新 API 调用旧实现
- ❌ 用户代码无法直接调用

---

## 📚 完整 API 映射表

### 记忆管理

| 旧 API | 新 API |
|--------|--------|
| `add_memory_fast(c, a, u, m, md)` | `add(c)` |
| `add_memory(c, a, u, m, md)` | `add(c)` |
| `add_memory_v2(c, a, u, m, md, i, opt)` | `add(c)` |
| `add_memory_intelligent(c, a, u, m, md)` | `add(c)` |
| `add_memories_batch(items)` | `add_batch(contents)` |
| `add_image_memory(img, cap, a, u, md)` | `add_image(img, cap)` |
| `add_audio_memory(aud, tr, a, u, md)` | `add_audio(aud, tr)` |
| `add_video_memory(vid, desc, a, u, md)` | `add_video(vid, desc)` |

### 记忆查询

| 旧 API | 新 API |
|--------|--------|
| `get_memory(id)` | `get(id)` |
| `get_all_memories(a, u, lim, off)` | `get_all()` |
| `get_all_memories_v2(a, u, m, lim, off, sort)` | `get_all()` |

### 记忆更新

| 旧 API | 新 API |
|--------|--------|
| `update_memory(id, c, a, u)` | `update(id, c)` |

### 记忆删除

| 旧 API | 新 API |
|--------|--------|
| `delete_memory(id)` | `delete(id)` |
| `delete_all_memories(a, u)` | `delete_all()` |
| `reset_system()` | `delete_all()` |

### 搜索功能

| 旧 API | 新 API |
|--------|--------|
| `search_memories(q, a, u, lim, f)` | `search(q)` |
| `search_memories_hybrid(q, u, lim, th, f)` | `search_builder(q)...` |
| `context_aware_rerank(r, q, u)` | `search_builder(q).with_rerank(true)` |
| `cached_search(q, a, u, lim, ttl)` | `search(q)` |

### 统计功能

| 旧 API | 新 API |
|--------|--------|
| `get_stats(a, u)` | `stats()` |
| `get_performance_stats()` | `performance_stats()` |
| `get_history(id)` | `history(id)` |

---

## 🎓 最佳实践

### ✅ DO: 简单场景使用简单 API

```rust
// 推荐
let id = orchestrator.add("content").await?;
let results = orchestrator.search("query").await?;
```

### ✅ DO: 复杂场景使用 Builder

```rust
// 推荐
let results = orchestrator
    .search_builder("query")
    .limit(20)
    .with_rerank(true)
    .with_threshold(0.7)
    .await?;
```

### ❌ DON'T: 过度使用 Builder

```rust
// 不推荐：简单场景使用 Builder（过度设计）
let id = orchestrator
    .batch_add()
    .add("content")
    .await?;
```

### ❌ DON'T: 放弃 Builder 的优势

```rust
// 不推荐：复杂场景不使用 Builder
let results = orchestrator.search("query").await?;
// 然后手动过滤、排序...
```

---

## 📞 获取帮助

- 📘 [API 文档](https://docs.rs/agent-mem)
- 📗 [用户指南](https://github.com/agent-mem/agent-mem)
- 💬 [Discord 社区](https://discord.gg/agent-mem)
- 🐛 [问题追踪](https://github.com/agent-mem/agent-mem/issues)

---

**最后更新**: 2025-01-08  
**文档版本**: 1.0  
**维护者**: AgentMem 团队
