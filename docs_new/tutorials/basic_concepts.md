# 基础概念

理解 AgentMem 的核心概念，帮助你更好地使用记忆管理系统。

## 目录

- [什么是 AgentMem?](#什么是-agentmem)
- [核心概念](#核心概念)
- [记忆类型](#记忆类型)
- [智能功能](#智能功能)
- [存储架构](#存储架构)

## 什么是 AgentMem?

AgentMem 是一个为 AI Agent 设计的记忆管理系统。就像人类拥有短期记忆、长期记忆、工作记忆一样，AgentMem 为 AI 提供了类似的记忆结构。

### 类比理解

| 人类记忆 | AgentMem | 用途 |
|---------|----------|------|
| 感知记忆 | CoreAgent (核心记忆) | 临时缓冲区 |
| 工作记忆 | WorkingAgent (工作记忆) | 当前任务相关 |
| 短期记忆 | EpisodicAgent (情节记忆) | 近期事件 |
| 长期记忆 | SemanticAgent (语义记忆) | 持久知识 |
| 程序记忆 | ProceduralAgent (程序记忆) | 技能和操作 |

## 核心概念

### 1. Memory (记忆)

Memory 是 AgentMem 中的核心数据结构，代表一条记忆记录。

```rust
use agentmem::{Memory, MemoryType};

// 创建一条记忆
let memory = Memory::builder()
    .content("用户喜欢咖啡")
    .memory_type(MemoryType::SEMANTIC)
    .metadata(Map::from([("preference", "drink")]))
    .build();
```

**Memory 结构**:

```rust
pub struct Memory {
    pub id: String,           // 唯一标识
    pub content: String,      // 记忆内容
    pub memory_type: MemoryType,  // 记忆类型
    pub agent_id: String,     // 所属 Agent
    pub session_id: Option<String>,  // 会话 ID
    pub metadata: Metadata,   // 元数据
    pub embedding: Vec<f32>,  // 向量嵌入
    pub importance: f32,      // 重要性评分 (0.0 - 1.0)
    pub created_at: DateTime,
    pub updated_at: DateTime,
}
```

### 2. Session (会话)

Session 代表一个对话或任务的上下文，用于组织相关记忆。

```rust
use agentmem::Memory;

// 创建会话
let session_id = "user_123_conversation_2024_01_15";

// 在会话中添加记忆
memory.add(
    "用户询问了产品价格",
    Some(session_id),
    MemoryType::EPISODIC
).await?;

// 获取会话中的所有记忆
let session_memories = memory.get_all(
    GetAllOptions::builder()
        .session_id(session_id)
        .build()
).await?;
```

**使用场景**:

```rust
// 多轮对话
let session = memory.create_session("user_123").await?;

// 第一轮
memory.add("用户: 你好", Some(session.id), MemoryType::EPISODIC).await?;
memory.add("助手: 你好！有什么我可以帮助你的？", Some(session.id), MemoryType::EPISODIC).await?;

// 第二轮
memory.add("用户: 今天天气怎么样？", Some(session.id), MemoryType::EPISODIC).await?;

// 获取对话历史
let history = memory.get_session_history(session.id).await?;
```

### 3. Agent (智能体)

Agent 代表一个独立的 AI 实体，拥有自己的记忆空间。

```rust
use agentmem::Memory;

// 为不同 Agent 创建独立的记忆空间
let agent1 = Memory::builder()
    .agent_id("data_collector")
    .build()
    .await?;

let agent2 = Memory::builder()
    .agent_id("data_processor")
    .build()
    .await?;

// Agent 1 收集数据
agent1.add("用户提供了新需求").await?;

// Agent 2 处理数据（可以访问 Agent 1 的记忆）
let data = agent2.get_all(
    GetAllOptions::builder()
        .agent_id("data_collector")
        .build()
).await?;
```

## 记忆类型

AgentMem 支持多种记忆类型，每种类型都有特定的用途和优化策略。

### 1. Core Memory (核心记忆)

**用途**: 临时存储和快速访问的基本信息

**特点**:
- 极快的读写速度
- 自动过期机制
- 用于存储临时状态

```rust
// 存储临时状态
memory.add(
    "用户当前正在浏览产品页面",
    None,
    MemoryType::CORE
).await?;
```

### 2. Working Memory (工作记忆)

**用途**: 当前任务相关的临时信息

**特点**:
- 任务完成后自动清理
- 支持优先级排序
- 用于上下文管理

```rust
// 存储当前任务上下文
memory.add(
    "用户正在比较产品 A 和产品 B 的价格",
    Some(current_task_id),
    MemoryType::WORKING
).await?;
```

### 3. Episodic Memory (情节记忆)

**用途**: 存储具体事件和经历

**特点**:
- 时间序列组织
- 支持时序查询
- 用于对话历史

```rust
// 记录对话历史
memory.add(
    "用户在 2024-01-15 询问了产品价格",
    Some(session_id),
    MemoryType::EPISODIC
).await?;

// 查询特定时间段的记忆
let memories = memory.get_all(
    GetAllOptions::builder()
        .memory_type(MemoryType::EPISODIC)
        .after(DateTime::from_str("2024-01-01")?)
        .before(DateTime::from_str("2024-01-31")?)
        .build()
).await?;
```

### 4. Semantic Memory (语义记忆)

**用途**: 存储长期知识和概念

**特点**:
- 永久存储（除非手动删除）
- 语义搜索优化
- 用于知识库

```rust
// 存储用户偏好（长期知识）
memory.add(
    "用户喜欢科幻电影，尤其是《星际穿越》和《银翼杀手》",
    None,
    MemoryType::SEMANTIC
).await?;

// 语义搜索
let results = memory.search("用户喜欢什么样的电影？").await?;
```

### 5. Procedural Memory (程序记忆)

**用途**: 存储操作流程和技能

**特点**:
- 结构化存储
- 支持步骤序列
- 用于自动化任务

```rust
// 存储操作流程
memory.add(
    "用户反馈处理流程：1. 接收反馈 2. 分类处理 3. 回复用户 4. 记录统计",
    None,
    MemoryType::PROCEDURAL
).await?;
```

### 6. Resource Memory (资源记忆)

**用途**: 管理外部资源的引用

**特点**:
- 存储文件路径、URL 等
- 支持资源生命周期管理
- 用于文档管理

```rust
// 存储资源引用
memory.add(
    "用户上传了需求文档",
    None,
    MemoryType::RESOURCE
).await?;
```

## 智能功能

AgentMem 提供多种智能功能，帮助你更好地管理记忆。

### 1. 自动事实提取 (Fact Extraction)

自动从文本中提取关键信息。

```rust
// 原始文本
let text = "张三，30岁，软件工程师，住在上海，喜欢科幻电影";

// AgentMem 自动提取：
// - 姓名: 张三
// - 年龄: 30岁
// - 职业: 软件工程师
// - 城市: 上海
// - 兴趣: 科幻电影

memory.add(text, None, MemoryType::SEMANTIC).await?;
```

### 2. 智能去重 (Deduplication)

自动识别和合并重复内容。

```rust
// 添加第一条记忆
memory.add("用户喜欢咖啡").await?;

// 添加相似内容（会被识别为重复）
memory.add("用户爱喝咖啡").await?;

// 结果：只会保存一条记忆
```

**去重策略**:

1. **精确匹配**: 内容完全相同
2. **语义相似**: 向量相似度 > 阈值（默认 0.95）
3. **包含关系**: 一个内容完全包含另一个

### 3. 冲突解决 (Conflict Resolution)

智能处理矛盾信息。

```rust
// 添加原始信息
memory.add("用户今年30岁").await?;

// 添加更新信息（自动识别为更新）
memory.add("用户今年31岁，刚过生日").await?;

// 结果：保留最新信息，并记录更新历史
```

### 4. 重要性评分 (Importance Scoring)

自动评估记忆的重要性。

```rust
let result = memory.add("这是一个非常重要的信息").await?;

println!("重要性评分: {:.2}", result.importance);

// 评分基于：
// - 信息的新颖性
// - 与现有记忆的相关性
// - 情感强度
// - 内容长度和复杂性
```

## 存储架构

AgentMem 采用分层存储架构，优化性能和成本。

```
┌─────────────────────────────────────┐
│         Application Layer           │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│         Memory API                  │
│  (统一接口，隐藏复杂性)               │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│      Memory Orchestrator            │
│  • 智能路由  • 事务管理  • 缓存      │
└──┬───┬───┬───┬───┬───┬───┬───┬───┬──┘
   │   │   │   │   │   │   │   │   │
┌──▼─┐▼───▼───▼─┐▼───▼───▼───▼───▼──┐
│   Hot Storage   │   Cold Storage   │
│  (Redis/Memory) │  (Disk/Cloud)    │
│  • 快速访问      │  • 长期存储       │
│  • 高成本       │  • 低成本         │
└─────────────────┴──────────────────┘
```

### 存储策略

| 记忆类型 | 默认存储 | 保留策略 | 访问速度 |
|---------|---------|---------|---------|
| CORE | 内存 | 自动过期（1小时） | < 1ms |
| WORKING | Redis | 任务完成后清理 | < 5ms |
| EPISODIC | PostgreSQL | 30天 | < 10ms |
| SEMANTIC | PostgreSQL + 向量数据库 | 永久 | < 50ms |
| PROCEDURAL | PostgreSQL | 永久 | < 10ms |
| RESOURCE | 对象存储 (S3) | 永久 | < 100ms |

### 自动分层

AgentMem 会自动根据记忆的访问频率和重要性进行存储分层：

```rust
// 频繁访问的记忆会自动移到热存储
for _ in 0..100 {
    memory.search("常用查询").await?;  // 自动提升到 Redis
}

// 长期未访问的记忆会自动移到冷存储
// 节省成本，不影响可用性
```

## 元数据系统

使用元数据为记忆添加结构化信息。

```rust
use std::collections::HashMap;
use agentmem::{Memory, MemoryType};

let mut metadata = HashMap::new();
metadata.insert("category".to_string(), "preference".into());
metadata.insert("source".to_string(), "user_survey".into());
metadata.insert("confidence".to_string(), 0.9.into());

memory.add(
    "用户喜欢咖啡",
    None,
    MemoryType::SEMANTIC
).await?;

// 或者使用 builder
let memory = Memory::builder()
    .content("用户喜欢咖啡")
    .memory_type(MemoryType::SEMANTIC)
    .metadata("category", "preference")
    .metadata("source", "user_survey")
    .metadata("confidence", 0.9)
    .build();

memory.add_memory(memory).await?;

// 基于元数据搜索
let results = memory.search(
    SearchOptions::builder()
        .metadata_filter(Map::from([("category", "preference")]))
        .build()
).await?;
```

## 最佳实践

### 1. 选择合适的记忆类型

```rust
// ✅ 正确：用户偏好使用 SEMANTIC
memory.add("用户喜欢咖啡", None, MemoryType::SEMANTIC).await?;

// ❌ 错误：用户偏好不应该用 EPISODIC
memory.add("用户喜欢咖啡", None, MemoryType::EPISODIC).await?;
```

### 2. 使用 Session 组织对话

```rust
// ✅ 正确：使用 Session 管理对话
let session = memory.create_session(user_id).await?;
memory.add("用户说: ...", Some(session.id), MemoryType::EPISODIC).await?;

// ❌ 错误：所有对话都混在一起
memory.add("用户说: ...", None, MemoryType::EPISODIC).await?;
```

### 3. 合理使用元数据

```rust
// ✅ 正确：使用元数据标记重要信息
memory.add(
    "用户喜欢咖啡",
    None,
    MemoryType::SEMANTIC
).await?;

// ❌ 错误：将结构化数据放在内容中
memory.add(
    "category:preference, source:survey, 用户喜欢咖啡",
    None,
    MemoryType::SEMANTIC
).await?;
```

### 4. 定期清理

```rust
// 定期清理过期记忆
memory.cleanup(
    CleanupOptions::builder()
        .before(DateTime::now() - Duration::days(30))
        .memory_type(MemoryType::EPISODIC)
        .build()
).await?;
```

## 下一步

现在你已经理解了 AgentMem 的核心概念，可以继续学习：

- [记忆管理](memory_management.md) - 学习如何添加、更新、删除记忆
- [语义搜索](semantic_search.md) - 深入了解搜索功能
- [多模态处理](multimodal.md) - 处理图像、音频等多媒体内容
