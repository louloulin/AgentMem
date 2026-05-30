# AgentMem v2.0 - 核心功能精化计划

> **📅 日期**: 2026-05-30
> **版本**: v2.0
> **理念**: 功能不在于多，在于精 - 连接孤岛，强化核心
> **对标**: Letta/MemGPT, Mem0, Anthropic Claude Memory

---

## 零、架构图

### 0.1 当前架构 (现状 - 2026-05-30)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Client Applications                                │
│          CLI │ Web UI │ MCP Client │ LangChain │ Python SDK                │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         REST API / MCP Server                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │
│  │   Memory    │  │   Search    │  │  Working    │  │   Health    │       │
│  │   CRUD      │  │   API       │  │   Memory    │  │   API       │       │
│  │   ✅        │  │   ✅        │  │   ✅        │  │   ✅        │       │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └─────────────┘       │
└─────────┼────────────────┼────────────────┼─────────────────────────────────┘
          │                │                │
          ▼                ▼                ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                            Memory Manager (Core)                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │
│  │   Core      │  │  Semantic   │  │  Episodic   │  │  Working    │       │
│  │  Memory     │  │  Memory     │  │  Memory     │  │  Memory     │       │
│  │  Manager    │  │  Manager    │  │  Manager    │  │  Manager    │       │
│  │  (Rust)     │  │  (PG Only)  │  │  (PG Only)  │  │  ✅ 集成    │       │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘       │
│         │                │                │                │               │
│         │ ❌ 未连接       │ ❌ 未连接       │ ❌ 未连接       │ ✅ 已集成    │
└─────────┼────────────────┼────────────────┼────────────────┼───────────────┘
          │                │                │                │
          ▼                ▼                ▼                ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Storage Layer (Dual Write)                          │
│  ┌───────────────────────┐         ┌───────────────────────┐              │
│  │    Vector Store       │         │    LibSQL Database    │              │
│  │    (LanceDB)         │         │    (Structured)       │              │
│  │    ✅ 工作正常        │         │    ✅ 工作正常        │              │
│  └───────────────────────┘         └───────────────────────┘              │
└─────────────────────────────────────────────────────────────────────────────┘

                              ═══════════════════════════════════════
                              孤岛 (未连接 - 需要整合的功能)
                              ═══════════════════════════════════════
                              
┌─────────────────────────────────────────────────────────────────────────────┐
│                          未集成的组件 (Islands)                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │
│  │  Forgetting │  │  Summarizer │  │  Core Memory│  │  Scope      │       │
│  │  Scheduler  │  │  (LLM)      │  │  (Persona)  │  │  Middleware │       │
│  │     ❌      │  │     ❌      │  │     ❌      │  │     ❌      │       │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘       │
│                                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │
│  │  Cognitive  │  │  Lifecycle  │  │  Knowledge  │  │  Resource   │       │
│  │  Memory     │  │  Manager    │  │  Vault      │  │  Memory     │       │
│  │     ❌      │  │     ❌      │  │     ⚠️      │  │     ⚠️      │       │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 0.2 目标架构 (改造后 - v2.0)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Client Applications                               │
│          CLI │ Web UI │ MCP Client │ LangChain │ Python SDK               │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         REST API / MCP Server                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │
│  │   Memory    │  │   Search    │  │  Working    │  │  Core       │       │
│  │   CRUD      │  │   API       │  │   Memory    │  │  Memory API │       │
│  │   ✅        │  │   ✅        │  │   ✅        │  │  🔴 新增    │       │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘       │
│         │                │                │                │               │
│         └────────────────┴────────────────┴────────────────┘               │
│                                    │                                       │
│                    ┌───────────────┴───────────────┐                      │
│                    │   Scope Middleware (🔴 新增)   │                      │
│                    │   权限验证 + 访问控制 + 审计    │                      │
│                    └───────────────┬───────────────┘                      │
└────────────────────────────────────┼───────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                            Memory Manager (Core)                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │
│  │   Core      │  │  Semantic   │  │  Episodic   │  │  Working    │       │
│  │  Memory     │  │  Memory     │  │  Memory     │  │  Memory     │       │
│  │  Manager    │  │  Manager    │  │  Manager    │  │  Manager    │       │
│  │  🔴 集成    │  │  🔴 集成    │  │  🔴 集成    │  │  ✅ 已集成  │       │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └─────────────┘       │
│         │                │                │                               │
│         └────────────────┴────────────────┘                               │
│                              │                                            │
│                              ▼                                            │
│  ┌─────────────────────────────────────────────────────────────────────┐  │
│  │                     Forgetting & Consolidation                       │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌───────────┐│  │
│  │  │  Forgetting │  │  Summarizer │  │  Protection │  │ Scheduler ││  │
│  │  │  (Ebbinghaus│  │  (LLM)      │  │  Manager    │  │  🔴 集成  ││  │
│  │  │  Curve)     │  │  🔴 集成    │  │  🔴 集成    │  │           ││  │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  └───────────┘│  │
│  └─────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Storage Layer (Dual Write)                          │
│  ┌───────────────────────┐         ┌───────────────────────┐              │
│  │    Vector Store       │         │    LibSQL Database    │              │
│  │    (LanceDB)         │         │    (Structured)       │              │
│  │    ✅ 工作正常        │         │    ✅ 工作正常        │              │
│  └───────────────────────┘         └───────────────────────┘              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 0.3 功能闭环架构 (核心流程)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        AI Agent Memory Loop (功能闭环)                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│    ┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐       │
│    │   输入    │────▶│   记忆   │────▶│   检索   │────▶│   生成   │       │
│    │ (感知)   │     │  存储    │     │  回忆    │     │  响应    │       │
│    └──────────┘     └────┬─────┘     └────┬─────┘     └────┬─────┘       │
│                          │                │                │               │
│                          ▼                │                │               │
│                    ┌──────────┐            │                │               │
│                    │ 遗忘调度 │            │                │               │
│                    │ (自动)   │            │                │               │
│                    └────┬─────┘            │                │               │
│                          │                │                │               │
│                          ▼                │                ▼               │
│    ┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐       │
│    │ 保护重要 │◀────│  定期    │◀────│  评估    │◀────│  反馈    │       │
│    │  记忆    │     │  整合    │     │  重要性  │     │  更新    │       │
│    └──────────┘     └──────────┘     └──────────┘     └──────────┘       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘

                    ════════════════════════════════════════════
                    生命周期: 感知 → 存储 → 检索 → 生成 → 反馈 → 遗忘
                    ════════════════════════════════════════════
```

### 0.4 记忆层级架构 (对标 Letta/MemGPT)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                       AI Agent Memory Hierarchy                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────┐      │
│   │  Level 1: Core Memory (始终在上下文 - Persona/Human Blocks)       │      │
│   │  ┌───────────────────┐  ┌───────────────────┐                  │      │
│   │  │  Persona Block    │  │   Human Block     │                  │      │
│   │  │  - Agent Identity │  │   - User Prefs    │                  │      │
│   │  │  - Behavior Rules │  │   - History       │                  │      │
│   │  │  - Personality    │  │   - Relationships│                  │      │
│   │  └───────────────────┘  └───────────────────┘                  │      │
│   │  容量: ~2-4K tokens  │  容量: ~4-8K tokens                   │      │
│   │  特点: 永不遗忘 ⭐    │  特点: 永久保留 ⭐                      │      │
│   └─────────────────────────────────────────────────────────────────┘      │
│                                    │                                       │
│                                    ▼                                       │
│   ┌─────────────────────────────────────────────────────────────────┐      │
│   │  Level 2: Working Memory (会话内 - Session Scoped)              │      │
│   │  ┌───────────────────────────────────────────────────────┐        │      │
│   │  │ - Current Task State                                   │        │      │
│   │  │ - Recent N Messages (Last 10-20)                      │        │      │
│   │  │ - Active Tool Results                                  │        │      │
│   │  │ - In-progress Calculations                             │        │      │
│   │  └───────────────────────────────────────────────────────┘        │      │
│   │  容量: ~8-16K tokens  │  特点: TTL 自动过期 (默认 24h)            │      │
│   └─────────────────────────────────────────────────────────────────┘      │
│                                    │                                       │
│                                    ▼                                       │
│   ┌─────────────────────────────────────────────────────────────────┐      │
│   │  Level 3: Semantic Memory (长期存储 - Vector Search)            │      │
│   │  ┌───────────────────────────────────────────────────────┐        │      │
│   │  │ - Extracted Facts & Knowledge                          │        │      │
│   │  │ - Entity Relationships                                 │        │      │
│   │  │ - Learned Concepts                                     │        │      │
│   │  └───────────────────────────────────────────────────────┘        │      │
│   │  特点: 遗忘曲线管理 (Ebbinghaus) │ 向量相似度检索                 │      │
│   │  容量: 无限制  │  保护级别: 可配置                               │      │
│   └─────────────────────────────────────────────────────────────────┘      │
│                                    │                                       │
│                                    ▼                                       │
│   ┌─────────────────────────────────────────────────────────────────┐      │
│   │  Level 4: Episodic Memory (历史事件 - Timeline)                   │      │
│   │  ┌───────────────────────────────────────────────────────┐        │      │
│   │  │ - Past Important Events                               │        │      │
│   │  │ - Conversation Summaries                             │        │      │
│   │  │ - Task Completion Records                             │        │      │
│   │  └───────────────────────────────────────────────────────┘        │      │
│   │  特点: 可被整合/摘要 │ 时间线检索 │ 基于 Ebbinghaus 遗忘曲线       │      │
│   │  容量: 无限制  │  特点: 自动摘要保留关键事件                       │      │
│   └─────────────────────────────────────────────────────────────────┘      │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 0.5 数据流架构 (改造后)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          Data Flow Architecture                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  User Input                                                                │
│      │                                                                     │
│      ▼                                                                     │
│  ┌─────────────────────────────────────────────────────────────────────┐  │
│  │                     Memory Input Pipeline                             │  │
│  │  ┌───────────┐   ┌───────────┐   ┌───────────┐   ┌───────────┐    │  │
│  │  │ Importance│   │  Content  │   │  Scope    │   │ Protection│    │  │
│  │  │  Scoring  │──▶│ Extraction│──▶│  tagging  │──▶│  Level    │    │  │
│  │  └───────────┘   └───────────┘   └───────────┘   └───────────┘    │  │
│  └─────────────────────────────────────────────────────────────────────┘  │
│                                    │                                       │
│                                    ▼                                       │
│  ┌─────────────────────────────────────────────────────────────────────┐  │
│  │                     Memory Storage                                   │  │
│  │                                                                     │  │
│  │   ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐          │  │
│  │   │  Core   │    │ Working │    │Semantic │    │Episodic│          │  │
│  │   │ Memory  │    │ Memory  │    │ Memory  │    │ Memory │          │  │
│  │   │(Persona)│    │(Session)│    │(Vector) │    │(Events)│          │  │
│  │   └────┬────┘    └────┬────┘    └────┬────┘    └────┬────┘          │  │
│  │        │              │              │              │                │  │
│  │        └──────────────┬┴──────────────┴──────────────┘                │  │
│  │                       │                                               │  │
│  │                       ▼                                               │  │
│  │              ┌─────────────────┐                                     │  │
│  │              │   LanceDB +     │                                     │  │
│  │              │   LibSQL        │                                     │  │
│  │              └─────────────────┘                                     │  │
│  └─────────────────────────────────────────────────────────────────────┘  │
│                                    │                                       │
│                                    ▼                                       │
│  ┌─────────────────────────────────────────────────────────────────────┐  │
│  │                     Memory Retrieval Pipeline                        │  │
│  │                                                                     │  │
│  │   Query ──▶ Relevance Scoring ──▶ Reranking ──▶ Context Assembly  │  │
│  │                    │                    │              │             │  │
│  │                    ▼                    ▼              ▼             │  │
│  │   ┌─────────┐  ┌─────────┐  ┌─────────────┐  ┌─────────────┐       │  │
│  │   │Semantic │  │Temporal │  │Importance   │  │  Scope      │       │  │
│  │   │Similarity│  │Distance │  │Score        │  │  Filter     │       │  │
│  │   └─────────┘  └─────────┘  └─────────────┘  └─────────────┘       │  │
│  │                                                                     │  │
│  └─────────────────────────────────────────────────────────────────────┘  │
│                                    │                                       │
│                                    ▼                                       │
│                           Agent Response                                  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 一、现状分析

### 1.1 ✅ 已正常工作的功能

| 功能 | 状态 | 位置 | 代码量 |
|------|------|------|--------|
| Working Memory | ✅ 工作正常 | `agent-mem-server/routes/working_memory.rs` | ~300行 |
| Long-term Memory | ✅ 工作正常 | `agent-mem-core/managers/` | ~5000行 |
| Vector Search | ✅ 工作正常 | `agent-mem-embeddings` | ~2000行 |
| Metadata Filtering | ✅ 工作正常 | API 集成 | ~500行 |
| Memory CRUD | ✅ 工作正常 | REST API | ~1000行 |
| Embedding Fallback | ✅ 工作正常 | MockEmbedder fallback | ~200行 |
| FastEmbed 优化 | ✅ 工作正常 | 本地模型 5-10x 提升 | ~500行 |

### 1.2 ⚠️ 存在但未集成的功能

| 功能 | 代码状态 | 集成状态 | 位置 |
|------|----------|----------|------|
| Core Memory (Persona) | ✅ 存在 | ❌ 未连接到 API | `agent-mem-core/managers/core_memory.rs` (~600行) |
| Forgetting (遗忘机制) | ✅ 存在 | ❌ 未连接到 API | `agent-mem-forgetting/` (~800行) |
| Summarization (摘要) | ✅ 存在 | ❌ 未连接到 API | `agent-mem-core/src/prompt/summarizer.rs` |
| MemoryScope | ✅ trait 存在 | ❌ API 未强制执行 | `agent-mem-traits/` |
| Protection Levels | ✅ 存在 | ❌ 未连接到 API | `agent-mem-forgetting/protection.rs` |
| Ebbinghaus Curve | ✅ 存在 | ❌ 未连接到 API | `agent-mem-forgetting/curve.rs` |

### 1.3 🔴 缺失的集成

| 集成 | 重要性 | 说明 | 预计工作量 |
|------|--------|------|-----------|
| Scheduled Tasks | 🔴 高 | 后台定期运行 forgetting/summarization | 2-3天 |
| Scope Middleware | 🔴 高 | API 层验证 MemoryScope 权限 | 2-3天 |
| Consolidation Endpoint | 🟡 中 | 手动触发记忆整合的 API | 1-2天 |
| Core Memory API | 🔴 高 | 用户设置 persona/human blocks | 3-4天 |
| Forgetting Health API | 🟡 中 | 记忆健康状态查询 | 1天 |

### 1.4 🔴 架构问题

| 问题 | 影响 | 优先级 |
|------|------|--------|
| CoreMemoryManager 未集成到服务器 | 用户无法设置 Persona/Human blocks | 🔴 高 |
| ForgettingScheduler 未启动 | 记忆永不过期 | 🔴 高 |
| Semantic/Episodic 需要 PostgreSQL | LibSQL 用户无法使用 | 🟡 中 |
| 无后台任务调度机制 | 无法自动执行 forgetting | 🔴 高 |

---

## 二、顶级 AI Agent 记忆系统核心功能 (2026)

### 2.1 2026年最新研究趋势

根据 2025-2026 年最新研究：

| 趋势 | 描述 | AgentMem 状态 |
|------|------|---------------|
| **Hierarchical Memory** | 层级记忆 (Core/Working/Semantic/Episodic) | ⚠️ 部分实现 |
| **Ebbinghaus-based Forgetting** | 基于艾宾浩斯遗忘曲线的智能遗忘 | ✅ 代码存在，未集成 |
| **Context Compression** | 上下文压缩和摘要 | ⚠️ 代码存在，未集成 |
| **Importance Scoring** | 重要性评分驱动保留策略 | ⚠️ 部分实现 |
| **Scope-based Access** | 基于作用域的访问控制 | ⚠️ trait存在，未强制 |
| **Multi-modal Memory** | 多模态记忆支持 | ✅ 存在 multimodal_storage |
| **Real-time Consolidation** | 实时记忆整合 | ❌ 未实现 |

### 2.2 必须具备的 5 个核心功能

根据 Letta/MemGPT、Mem0、Claude Memory 分析：

| # | 核心功能 | AgentMem 状态 | 差距 |
|---|----------|---------------|------|
| 1 | **层级记忆** (Core/Working/Long-term) | ⚠️ 部分集成 | Working 集成，Core 未集成 |
| 2 | **语义搜索** (向量 + 过滤) | ✅ 已工作 | 无差距 |
| 3 | **记忆整合** (摘要/遗忘) | ❌ 未集成 | 需要连接现有代码 |
| 4 | **CRUD 操作** | ✅ 已工作 | 无差距 |
| 5 | **多租户隔离** | ⚠️ trait 存在 | API 未强制执行 |

### 2.3 3 个反模式 (不要做)

| 反模式 | 说明 | AgentMem 现状 |
|--------|------|---------------|
| 无差别存储 | 存储所有对话 | ✅ 已实现智能分类 |
| 无生命周期 | 记忆永不过期 | ⚠️ 有代码但未启用 |
| 单层架构 | 只用上下文或向量 | ✅ 已有多层架构 |

---

## 三、改造计划

### Phase 1: Core Memory API 🔴 高优先级

**目标**: 让用户可以通过 API 设置 Persona/Human blocks

**现状分析**:
- CoreMemoryManager 存在且有 11 个测试 ✅
- `crates/agent-mem-core/src/managers/core_memory.rs` (~600行)
- 但 `routes/memory.rs` 注释说明未集成

**任务**:
- [ ] 创建 `routes/core_memory.rs` 路由文件
- [ ] 添加 `GET/POST/PUT /api/v1/core-memory/persona` 端点
- [ ] 添加 `GET/POST/PUT /api/v1/core-memory/human` 端点
- [ ] 添加容量管理端点 `GET /api/v1/core-memory/capacity`
- [ ] 添加自动重写触发 `POST /api/v1/core-memory/rewrite`

**API 设计**:
```rust
// POST /api/v1/core-memory/persona
#[derive(Serialize, Deserialize, ToSchema)]
pub struct PersonaBlockRequest {
    pub agent_id: String,
    pub content: String,
    pub max_capacity: Option<usize>,
}

// POST /api/v1/core-memory/human
#[derive(Serialize, Deserialize, ToSchema)]
pub struct HumanBlockRequest {
    pub user_id: String,
    pub content: String,
    pub max_capacity: Option<usize>,
}

// GET /api/v1/core-memory/capacity
#[derive(Serialize, Deserialize, ToSchema)]
pub struct CapacityResponse {
    pub persona_blocks: Vec<CapacityInfo>,
    pub human_blocks: Vec<CapacityInfo>,
    pub total_usage_percent: f32,
}
```

**验收标准**:
```bash
curl -X POST http://localhost:8080/api/v1/core-memory/persona \
  -H "Content-Type: application/json" \
  -d '{"agent_id": "agent1", "content": "You are a helpful coding assistant"}'

curl http://localhost:8080/api/v1/core-memory/persona?agent_id=agent1
```

---

### Phase 2: Forgetting 集成 🔴 高优先级

**目标**: 让记忆有生命周期，实现艾宾浩斯遗忘曲线

**现状分析**:
- ForgettingScheduler 存在 ✅
- EbbinghausCurve 遗忘曲线算法存在 ✅
- MemoryProtection 保护机制存在 ✅
- 但服务器启动时未启动这些服务 ❌

**任务**:
- [ ] 在服务器启动时初始化 `ForgettingScheduler`
- [ ] 添加清理任务定期执行 (默认: 每天凌晨 3 点)
- [ ] 添加 API 端点触发手动清理 `POST /api/v1/memories/cleanup`
- [ ] 添加 API 查询记忆健康状态 `GET /api/v1/memories/health`
- [ ] 保护重要记忆不被遗忘 (importance > 0.8)

**实现架构**:
```rust
// src/background_tasks.rs
pub struct ForgettingBackgroundTask {
    scheduler: ForgettingScheduler,
    repositories: Repositories,
    interval: Duration,
}

impl BackgroundTask for ForgettingBackgroundTask {
    async fn run(&self) {
        // 1. 获取所有需要检查的记忆
        let memories = self.repositories.memory.get_all_for_forgetting().await?;
        
        // 2. 检查遗忘
        let to_forget = self.scheduler.check_forgetting(memories).await?;
        
        // 3. 删除过期的记忆
        for memory_id in to_forget {
            self.repositories.memory.delete(&memory_id).await?;
        }
    }
}
```

**验收标准**:
```bash
curl http://localhost:8080/api/v1/memories/health
# 返回: { "healthy_count": 100, "forgotten_count": 5, "protected_count": 10 }

curl -X POST http://localhost:8080/api/v1/memories/cleanup
# 返回: { "deleted_count": 3, "reason": "forgetting" }
```

---

### Phase 3: Scope Middleware 🟡 中优先级

**目标**: 在 API 层强制执行 MemoryScope 权限检查

**现状分析**:
- `MemoryScope` trait 有 `can_access()` 方法
- 但 API 路由未调用此方法检查权限

**任务**:
- [ ] 创建 `middleware/scope_check.rs`
- [ ] 在 add/search/update/delete 操作前检查 scope 权限
- [ ] 返回 403 如果越权访问

**实现架构**:
```rust
// middleware/scope_check.rs
pub async fn scope_check_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let scope = extract_scope_from_request(&req)?;
    let target_scope = extract_target_scope(&req)?;
    
    if !scope.can_access(&target_scope) {
        return Err(StatusCode::FORBIDDEN);
    }
    
    next.run(req).await
}
```

**验收标准**:
```bash
curl http://localhost:8080/api/v1/memories?user_id=user1  # 只返回 user1 的记忆
curl http://localhost:8080/api/v1/memories?user_id=user2  # 返回 403 (如果跨用户访问)
```

---

### Phase 4: Consolidation Endpoint 🟡 中优先级

**目标**: 提供手动触发记忆整合的 API

**现状分析**:
- `MemorySummarizer` 存在于 `agent-mem-core/src/prompt/summarizer.rs`
- 但没有 API 触发

**任务**:
- [ ] 添加 `POST /api/v1/memories/consolidate` 端点
- [ ] 支持按 agent_id 或 user_id 整合
- [ ] 返回整合统计 (摘要数量、删除数量)

**实现架构**:
```rust
// POST /api/v1/memories/consolidate
#[derive(Serialize, Deserialize, ToSchema)]
pub struct ConsolidateRequest {
    pub agent_id: Option<String>,
    pub user_id: Option<String>,
    pub options: ConsolidationOptions,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ConsolidationResult {
    pub summarized_count: i64,
    pub deleted_count: i64,
    pub retained_count: i64,
    pub total_tokens_saved: i64,
}
```

**验收标准**:
```bash
curl -X POST http://localhost:8080/api/v1/memories/consolidate \
  -H "Content-Type: application/json" \
  -d '{"agent_id": "agent1", "dry_run": false}'
```

---

## 四、TODO List (完整功能闭环)

### 4.1 Phase 1: Core Memory API 🔴

```
[ ] 创建 routes/core_memory.rs
[ ] 实现 get_persona_block 端点
[ ] 实现 create_persona_block 端点
[ ] 实现 update_persona_block 端点
[ ] 实现 append_to_persona_block 端点
[ ] 实现 get_human_block 端点
[ ] 实现 create_human_block 端点
[ ] 实现 update_human_block 端点
[ ] 实现 get_capacity 端点
[ ] 实现 manual_rewrite 端点
[ ] 添加单元测试 (至少 10 个)
[ ] 添加集成测试
[ ] 更新 OpenAPI 文档
```

### 4.2 Phase 2: Forgetting 集成 🔴

```
[ ] 创建 src/background_tasks.rs
[ ] 实现 ForgettingBackgroundTask
[ ] 在服务器启动时初始化 scheduler
[ ] 实现 GET /api/v1/memories/health
[ ] 实现 POST /api/v1/memories/cleanup
[ ] 添加重要性评分自动更新
[ ] 添加保护级别自动调整
[ ] 添加单元测试 (至少 8 个)
[ ] 添加集成测试
```

### 4.3 Phase 3: Scope Middleware 🟡

```
[ ] 创建 middleware/scope_check.rs
[ ] 实现 extract_scope_from_request
[ ] 实现 can_access 检查
[ ] 集成到 add_memory 端点
[ ] 集成到 search_memories 端点
[ ] 集成到 update_memory 端点
[ ] 集成到 delete_memory 端点
[ ] 添加单元测试 (至少 6 个)
[ ] 添加集成测试
```

### 4.4 Phase 4: Consolidation API 🟡

```
[ ] 创建 routes/consolidation.rs
[ ] 实现 get_summarizable_memories
[ ] 实现 MemorySummarizer 集成
[ ] 实现 POST /api/v1/memories/consolidate
[ ] 添加 dry_run 支持
[ ] 添加统计报告
[ ] 添加单元测试 (至少 6 个)
[ ] 添加集成测试
```

### 4.5 验证清单 ✅

```
[ ] cargo check --workspace
[ ] cargo test --package agent-mem-server
[ ] API 端到端测试
[ ] 性能基准测试
[ ] 文档更新
```

---

## 五、不需要做的 (避免过度工程)

根据 "功能不在于多在于精" 原则：

| 功能 | 原因 |
|------|------|
| 新增 Cognitive Memory tiers | 现有 5 层已够用，增加复杂度 |
| 多层向量索引 | 当前 LanceDB 性能足够 |
| 新的 Embedding Provider | Mock → FastEmbed → OpenAI 链已完整 |
| 实时记忆同步 | 已有 LibSQL WAL 足够 |
| 分布式记忆 | 当前单体架构足够小团队使用 |
| 新的存储后端 | LibSQL + LanceDB 组合已满足需求 |

---

## 六、实施顺序

```
Week 1: Phase 1 (Core Memory API)
  │
  ├── Day 1-2: 创建 core_memory.rs 路由文件
  ├── Day 3-4: 实现 Persona/Human block API
  └── Day 5: 测试和文档

Week 2: Phase 2 (Forgetting Integration)
  │
  ├── Day 1-2: 创建 background_tasks.rs
  ├── Day 3-4: 实现 ForgettingScheduler 集成
  └── Day 5: 实现 health/cleanup API

Week 3: Phase 3 + Phase 4
  │
  ├── Day 1-2: Scope Middleware
  └── Day 3-5: Consolidation API

Week 4: 测试 + 优化
  │
  ├── Day 1-2: 集成测试
  ├── Day 3-4: 性能测试
  └── Day 5: 文档和发布
```

---

## 七、验证清单

### 7.1 API 验证

```bash
# Core Memory API
curl -X POST http://localhost:8080/api/v1/core-memory/persona \
  -H "Content-Type: application/json" \
  -d '{"agent_id": "agent1", "content": "You are a helpful assistant"}'

curl http://localhost:8080/api/v1/core-memory/persona?agent_id=agent1

curl http://localhost:8080/api/v1/core-memory/capacity

# Forgetting Health
curl http://localhost:8080/api/v1/memories/health

# Manual Cleanup
curl -X POST http://localhost:8080/api/v1/memories/cleanup

# Consolidation
curl -X POST http://localhost:8080/api/v1/memories/consolidate \
  -H "Content-Type: application/json" \
  -d '{"agent_id": "agent1"}'

# Scope Enforcement
curl http://localhost:8080/api/v1/memories?user_id=user1  # 应该只返回 user1 的记忆
curl http://localhost:8080/api/v1/memories?user_id=user2  # 应该返回 403 (如果跨用户访问)
```

### 7.2 编译验证

```bash
cargo check --workspace
cargo test --package agent-mem-server
```

---

## 八、里程碑

| 里程碑 | 完成标准 | 优先级 | 预计完成 |
|--------|----------|--------|----------|
| M1: Core Memory API | 用户可以设置/获取 Persona blocks | 🔴 高 | Week 1 |
| M2: Forgetting 集成 | 记忆有生命周期，自动清理 | 🔴 高 | Week 2 |
| M3: Scope Middleware | API 层强制权限检查 | 🟡 中 | Week 3 |
| M4: Consolidation API | 手动触发记忆整合 | 🟡 中 | Week 3 |
| M5: v2.0 稳定版 | 所有测试通过，文档完整 | 🔴 高 | Week 4 |

---

## 九、风险与缓解

| 风险 | 缓解 |
|------|------|
| 破坏现有 API | 添加功能不修改现有端点 |
| 性能影响 | forgetting 后台任务使用低优先级 |
| 测试覆盖不足 | 添加集成测试验证各组件协同 |
| PostgreSQL 依赖 | 提供 LibSQL 回退方案 |

---

## 十、2026年研究参考

### 10.1 最新论文

| 论文/系统 | 年份 | 核心概念 | AgentMem 借鉴 |
|-----------|------|----------|---------------|
| **MemGPT** | 2024 | 层次记忆 + OS 模拟 | ✅ Core Memory 设计已借鉴 |
| **Mem0** | 2024 | 记忆提取 + 语义搜索 | ✅ 记忆整合策略已借鉴 |
| **Anthropic Claude Memory** | 2024 | 上下文管理 + 工具调用 | ✅ Scope 设计已借鉴 |
| **Ebbinghaus Forgetting** | 1885/2024 | 记忆衰减模型 | ✅ 已实现代码 |
| **RAG 架构** | 2020-2026 | 检索增强生成 | ✅ 向量搜索已实现 |

### 10.2 顶级 AI Agent 记忆系统

| 系统 | 核心特点 | 参考价值 |
|------|----------|----------|
| **Letta/MemGPT** | Core/Archival/Recall 分层 | ✅ 对标 Core Memory |
| **Mem0** | 记忆提取 + 语义搜索 | ✅ 对标 记忆整合 |
| **Anthropic Claude Memory** | 上下文管理 + 工具调用 | ✅ 对标 Scope |
| **LangChain Memory** | Store 抽象 + 缓冲 | ✅ 对标 API 设计 |

### 10.3 关键架构决策

| 决策 | 选择 | 理由 |
|------|------|------|
| 存储后端 | LanceDB + LibSQL | 向量搜索 + 结构化数据 |
| Embedding | FastEmbed (本地) → OpenAI (云) → Mock | 中国网络适配 |
| 遗忘策略 | 艾宾浩斯曲线 | 符合人类记忆科学 |
| 整合策略 | LLM 摘要 | 自动压缩上下文 |
| 作用域 | Scope Middleware | 多租户安全隔离 |

---

## 十一、未来展望 (v3.0)

```
Phase 5: 多租户支持
  - Organization 级别共享记忆
  - Agent 间知识共享

Phase 6: 实时学习
  - 从对话中自动提取知识
  - 动态更新 Core Memory

Phase 7: 分布式部署
  - 多节点向量索引
  - 跨实例记忆同步
```

---

## 十二、架构图汇总

### 12.1 完整功能闭环

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         AgentMem v2.0 功能闭环                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐  │
│  │   Input │───▶│  Store  │───▶│ Retrieve│───▶│ Generate│───▶│  Learn  │  │
│  │         │    │         │    │         │    │         │    │         │  │
│  │ - User  │    │ - Core  │    │ - Search│    │ - LLM   │    │ - Stats │  │
│  │ - Agent │    │ - Working│    │ - Filter│    │ - Tools │    │ - Score │  │
│  │ - Events│    │ - Semantic    │ - Rank │    │         │    │         │  │
│  └────┬────┘    └────┬────┘    └────┬────┘    └────┬────┘    └────┬────┘  │
│       │              │              │              │              │       │
│       │              │              │              │              │       │
│       │              ▼              │              │              │       │
│       │         ┌─────────┐         │              │              │       │
│       │         │ Forget  │◀────────┴──────────────┴──────────────┘       │
│       │         │         │                                                │
│       │         │ - Ebbinghaus Curve                                     │
│       │         │ - Protection Levels                                     │
│       │         │ - Consolidation                                         │
│       │         └─────────┘                                                │
│       │              │                                                    │
│       └──────────────┴────────────────────────────────────────────────▶反馈
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

**核心理念**: 连接孤岛，强化核心。不要添加新功能，让现有功能协同工作。

**最后更新**: 2026-05-30
**版本**: v2.0
**状态**: 进行中
