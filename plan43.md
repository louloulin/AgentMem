# AgentMem v2.0 - 核心功能精化计划

> **📅 日期**: 2026-05-30
> **版本**: v2.0
> **理念**: 功能不在于多，在于精 - 连接孤岛，强化核心
> **对标**: Letta/MemGPT, Mem0, Anthropic Claude Memory

---

## 零、架构图

### 0.1 当前架构 (现状)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Client Applications                              │
│   (CLI, Web UI, MCP Client, LangChain, etc.)                               │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                          REST API / MCP Server                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │
│  │   Memory    │  │   Search    │  │  Working    │  │   Health    │       │
│  │   CRUD      │  │   API       │  │   Memory     │  │   API       │       │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └─────────────┘       │
└─────────┼────────────────┼────────────────┼─────────────────────────────────┘
          │                │                │
          ▼                ▼                ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                            Memory Manager (Core)                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │
│  │   Core      │  │  Semantic   │  │  Episodic   │  │  Working    │       │
│  │  Memory     │  │  Memory     │  │  Memory     │  │  Memory     │       │
│  │  Manager    │  │  Manager    │  │  Manager    │  │  Manager    │       │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └─────────────┘       │
│         │                │                │                                │
│         │ ⚠️ 未集成       │                │ ✅ 已集成                       │
└─────────┼────────────────┼────────────────┼─────────────────────────────────┘
          │                │                │
          ▼                ▼                ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Storage Layer (Dual Write)                            │
│  ┌───────────────────────┐         ┌───────────────────────┐              │
│  │    Vector Store       │         │    LibSQL Database    │              │
│  │    (LanceDB)          │         │    (Structured)       │              │
│  └───────────────────────┘         └───────────────────────┘              │
└─────────────────────────────────────────────────────────────────────────────┘

                              孤岛 (未连接)
                              ┌─────────────┐
                              │  Forgetting │
                              │  Scheduler  │
                              │     ❌      │
                              └─────────────┘
                              ┌─────────────┐
                              │ Summarizer  │
                              │     ❌      │
                              └─────────────┘
                              ┌─────────────┐
                              │   Scope     │
                              │  Middleware │
                              │     ❌      │
                              └─────────────┘
```

### 0.2 目标架构 (改造后)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Client Applications                              │
│   (CLI, Web UI, MCP Client, LangChain, etc.)                               │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                          REST API / MCP Server                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │
│  │   Memory    │  │   Search    │  │  Working    │  │  Core       │       │
│  │   CRUD      │  │   API       │  │   Memory    │  │  Memory API │       │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘       │
│         │                │                │                │               │
│         └────────────────┴────────────────┴────────────────┘               │
│                                    │                                        │
│                    ┌───────────────┴───────────────┐                      │
│                    │    Scope Middleware (✅ 新增)   │                      │
│                    │    权限验证 + 访问控制          │                      │
│                    └───────────────┬───────────────┘                      │
└────────────────────────────────────┼───────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                            Memory Manager (Core)                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │
│  │   Core      │  │  Semantic   │  │  Episodic   │  │  Working    │       │
│  │  Memory     │  │  Memory     │  │  Memory     │  │  Memory     │       │
│  │  Manager    │  │  Manager    │  │  Manager    │  │  Manager    │       │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘       │
│         │                │                │                │               │
│         │ ✅ 已集成       │                │                │               │
└─────────┼────────────────┼────────────────┼────────────────┼───────────────────┘
          │                │                │                │
          ▼                ▼                ▼                ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Storage Layer (Dual Write)                            │
│  ┌───────────────────────┐         ┌───────────────────────┐              │
│  │    Vector Store       │         │    LibSQL Database    │              │
│  │    (LanceDB)          │         │    (Structured)       │              │
│  └───────────────────────┘         └───────────────────────┘              │
└─────────────────────────────────────────────────────────────────────────────┘

                              后台任务 (✅ 已连接)
                              ┌─────────────┐
                              │  Forgetting │
                              │  Scheduler  │
                              │  ✅ 已集成   │
                              └──────┬──────┘
                                     │
                              ┌──────┴──────┐
                              │ Consolidation │
                              │   Endpoint   │
                              │  ✅ 已集成   │
                              └─────────────┘
```

### 0.3 AI Agent 记忆层级 (对标 Letta/MemGPT)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        AI Agent Memory Hierarchy                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────┐      │
│   │  Level 1: Core Memory (始终在上下文)                            │      │
│   │  - Agent Persona (身份、性格、行为规则)                           │      │
│   │  - Human Info (用户偏好、历史关系)                                │      │
│   │  - 容量: ~4K tokens                                             │      │
│   │  - 特点: 永不遗忘，高优先级                                      │      │
│   └─────────────────────────────────────────────────────────────────┘      │
│                                    │                                       │
│                                    ▼                                       │
│   ┌─────────────────────────────────────────────────────────────────┐      │
│   │  Level 2: Working Memory (会话内)                                │      │
│   │  - 当前任务状态                                                  │      │
│   │  - 最近 N 条消息                                                 │      │
│   │  - 容量: ~8K tokens                                              │      │
│   │  - 特点: TTL 自动过期                                            │      │
│   └─────────────────────────────────────────────────────────────────┘      │
│                                    │                                       │
│                                    ▼                                       │
│   ┌─────────────────────────────────────────────────────────────────┐      │
│   │  Level 3: Semantic Memory (长期存储)                            │      │
│   │  - 提取的事实和知识                                              │      │
│   │  - 向量搜索检索                                                  │      │
│   │  - 特点: 遗忘曲线管理                                            │      │
│   └─────────────────────────────────────────────────────────────────┘      │
│                                    │                                       │
│                                    ▼                                       │
│   ┌─────────────────────────────────────────────────────────────────┐      │
│   │  Level 4: Episodic Memory (历史事件)                              │      │
│   │  - 过去的重要事件                                                │      │
│   │  - 时间线检索                                                     │      │
│   │  - 特点: 可被整合/摘要                                           │      │
│   └─────────────────────────────────────────────────────────────────┘      │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 一、现状分析

### 1.1 ✅ 已正常工作的功能

| 功能 | 状态 | 位置 |
|------|------|------|
| Working Memory | ✅ 工作正常 | `agent-mem-working-memory` |
| Long-term Memory | ✅ 工作正常 | `agent-mem-core/managers/` |
| Vector Search | ✅ 工作正常 | `agent-mem-embeddings` |
| Metadata Filtering | ✅ 工作正常 | API 集成 |
| Memory CRUD | ✅ 工作正常 | REST API |
| Embedding Fallback | ✅ 工作正常 | MockEmbedder fallback |

### 1.2 ⚠️ 存在但未集成的功能

| 功能 | 代码状态 | 集成状态 |
|------|----------|----------|
| Core Memory (Persona) | ✅ 存在 | ❌ 未连接到 API |
| Forgetting (遗忘机制) | ✅ 存在 | ❌ 未连接到 API |
| Summarization (摘要) | ✅ 存在 | ❌ 未连接到 API |
| MemoryScope | ✅ trait 存在 | ❌ API 未强制执行 |
| Cognitive Memory | ✅ 存在 | ❌ 未使用 |

### 1.3 🔴 缺失的集成

| 集成 | 重要性 | 说明 |
|------|--------|------|
| Scheduled Tasks | 🔴 高 | 后台定期运行 forgetting/summarization |
| Scope Middleware | 🔴 高 | API 层验证 MemoryScope 权限 |
| Consolidation Endpoint | 🟡 中 | 手动触发记忆整合的 API |
| Core Memory API | 🔴 高 | 用户设置 persona/human blocks |

---

## 二、顶级 AI Agent 记忆系统核心功能

### 2.1 必须具备的 5 个核心功能

根据 Letta/MemGPT、Mem0、Claude Memory 分析：

| # | 核心功能 | AgentMem 状态 | 差距 |
|---|----------|---------------|------|
| 1 | **层级记忆** (Core/Working/Long-term) | ⚠️ 部分集成 | Working 集成，Core 未集成 |
| 2 | **语义搜索** (向量 + 过滤) | ✅ 已工作 | 无差距 |
| 3 | **记忆整合** (摘要/遗忘) | ❌ 未集成 | 需要连接现有代码 |
| 4 | **CRUD 操作** | ✅ 已工作 | 无差距 |
| 5 | **多租户隔离** | ⚠️ trait 存在 | API 未强制执行 |

### 2.2 3 个反模式 (不要做)

| 反模式 | 说明 | AgentMem 现状 |
|--------|------|---------------|
| 无差别存储 | 存储所有对话 | ✅ 已实现智能分类 |
| 无生命周期 | 记忆永不过期 | ⚠️ 有代码但未启用 |
| 单层架构 | 只用上下文或向量 | ✅ 已有多层架构 |

---

## 三、精化计划

### Phase 1: 连接 Core Memory API 🔴 高优先级

**目标**: 让用户可以通过 API 设置 Persona/Human blocks

**现状**:
- CoreMemoryManager 存在且有 11 个测试
- 但 `crates/agent-mem-server/src/routes/memory.rs` 注释说明未集成

**任务**:
- [ ] 在 `routes/core_memory.rs` 创建新的路由文件
- [ ] 添加 `GET/POST/PUT /api/v1/core-memory/persona` 端点
- [ ] 添加 `GET/POST/PUT /api/v1/core-memory/human` 端点
- [ ] 添加容量管理端点 `GET /api/v1/core-memory/capacity`
- [ ] 添加自动重写触发 `POST /api/v1/core-memory/rewrite`

**实现文件**:
```rust
// crates/agent-mem-server/src/routes/core_memory.rs
pub async fn get_persona_block(agent_id: String) -> Result<CoreMemoryBlock>
pub async fn update_persona_block(agent_id: String, content: String) -> Result<CoreMemoryBlock>
pub async fn append_to_persona_block(agent_id: String, content: String) -> Result<CoreMemoryBlock>
pub async fn get_human_block(user_id: String) -> Result<CoreMemoryBlock>
pub async fn update_human_block(user_id: String, content: String) -> Result<CoreMemoryBlock>
```

---

### Phase 2: 连接 Forgetting API 🔴 高优先级

**目标**: 让记忆有生命周期，实现艾宾浩斯遗忘曲线

**现状**:
- `ForgettingScheduler` 存在于 `agent-mem-forgetting`
- `EbbinghausCurve` 遗忘曲线算法存在
- `MemoryProtection` 保护机制存在
- 但服务器启动时未启动这些服务

**任务**:
- [ ] 在服务器启动时初始化 `ForgettingScheduler`
- [ ] 添加清理任务定期执行 (默认: 每天凌晨 3 点)
- [ ] 添加 API 端点触发手动清理 `POST /api/v1/memories/cleanup`
- [ ] 添加 API 查询记忆健康状态 `GET /api/v1/memories/health`
- [ ] 保护重要记忆不被遗忘 (importance > 0.8)

**实现文件**:
```rust
// crates/agent-mem-server/src/background/forgetting_task.rs
pub struct ForgettingBackgroundTask {
    scheduler: ForgettingScheduler,
    interval: Duration,
}

impl BackgroundTask for ForgettingBackgroundTask {
    async fn run(&self) {
        self.scheduler.run_cleanup().await;
    }
}

// crates/agent-mem-server/src/routes/memory.rs 新增
POST /api/v1/memories/cleanup  // 手动触发清理
GET /api/v1/memories/health    // 查询记忆健康状态
```

---

### Phase 3: Scope Middleware 🟡 中优先级

**目标**: 在 API 层强制执行 MemoryScope 权限检查

**现状**:
- `MemoryScope` trait 有 `can_access()` 方法
- 但 API 路由未调用此方法检查权限

**任务**:
- [ ] 创建 scope middleware
- [ ] 在 add/search/update/delete 操作前检查 scope 权限
- [ ] 返回 403 如果越权访问

**实现文件**:
```rust
// crates/agent-mem-server/src/middleware/scope_check.rs
pub async fn scope_check_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let scope = extract_scope_from_request(&req);
    let target_scope = extract_target_scope(&req);

    if !scope.can_access(&target_scope) {
        return Err(StatusCode::FORBIDDEN);
    }

    next.run(req).await
}
```

---

### Phase 4: Consolidation Endpoint 🟡 中优先级

**目标**: 提供手动触发记忆整合的 API

**现状**:
- `MemorySummarizer` 存在于 `agent-mem-core/src/prompt/summarizer.rs`
- 但没有 API 触发

**任务**:
- [ ] 添加 `POST /api/v1/memories/consolidate` 端点
- [ ] 支持按 agent_id 或 user_id 整合
- [ ] 返回整合统计 (摘要数量、删除数量)

**实现文件**:
```rust
// crates/agent-mem-server/src/routes/consolidation.rs
pub async fn consolidate_memories(
    agent_id: Option<String>,
    user_id: Option<String>,
    options: ConsolidationOptions,
) -> Result<ConsolidationResult>
```

---

## 四、不需要做的 (避免过度工程)

根据 "功能不在于多在于精" 原则：

| 功能 | 原因 |
|------|------|
| 新增 Cognitive Memory tiers | 现有 5 层已够用，增加复杂度 |
| 多层向量索引 | 当前 LanceDB 性能足够 |
| 新的 Embedding Provider | Mock → FastEmbed → OpenAI 链已完整 |
| 实时记忆同步 | 已有 LibSQL WAL 足够 |
| 分布式记忆 | 当前单体架构足够小团队使用 |

---

## 五、实施顺序

```
Week 1: Phase 1 (Core Memory API)
  ↓
Week 2: Phase 2 (Forgetting Integration)
  ↓
Week 3: Phase 3 (Scope Middleware) + Phase 4 (Consolidation)
  ↓
Week 4: 测试 + 优化
```

---

## 六、验证清单

### 6.1 API 验证

```bash
# Core Memory API
curl -X POST http://localhost:8080/api/v1/core-memory/persona \
  -H "Content-Type: application/json" \
  -d '{"agent_id": "agent1", "content": "You are a helpful assistant"}'

curl http://localhost:8080/api/v1/core-memory/persona?agent_id=agent1

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

### 6.2 编译验证

```bash
cargo check --workspace
cargo test --package agent-mem-server
```

---

## 七、里程碑

| 里程碑 | 完成标准 | 优先级 |
|--------|----------|--------|
| M1: Core Memory API | 用户可以设置/获取 Persona blocks | 🔴 高 |
| M2: Forgetting 集成 | 记忆有生命周期，自动清理 | 🔴 高 |
| M3: Scope Middleware | API 层强制权限检查 | 🟡 中 |
| M4: Consolidation API | 手动触发记忆整合 | 🟡 中 |

---

## 八、风险与缓解

| 风险 | 缓解 |
|------|------|
| 破坏现有 API | 添加功能不修改现有端点 |
| 性能影响 | forgetting 后台任务使用低优先级 |
| 测试覆盖不足 | 添加集成测试验证各组件协同 |

---

**核心理念**: 连接孤岛，强化核心。不要添加新功能，让现有功能协同工作。

---

## 九、研究参考

### 9.1 顶级 AI Agent 记忆系统

| 系统 | 核心特点 | 参考价值 |
|------|----------|----------|
| **Letta/MemGPT** | Core/Archival/Recall 分层 | ✅ 对标 Core Memory |
| **Mem0** | 记忆提取 + 语义搜索 | ✅ 对标 记忆整合 |
| **Anthropic Claude Memory** | 上下文管理 + 工具调用 | ✅ 对标 Scope |
| **LangChain Memory** | Store 抽象 + 缓冲 | ✅ 对标 API 设计 |

### 9.2 论文与架构模式

| 论文/系统 | 核心概念 | 适用场景 |
|-----------|----------|----------|
| **MemGPT** (2024) | 层次记忆 + OS 模拟 | Core Memory 设计 |
| **Ebbinghaus遗忘曲线** | 记忆衰减模型 | Forgetting 机制 |
| **RAG 架构** | 检索增强生成 | 向量搜索 |
| **Actor Model** | 消息传递隔离 | Scope 权限 |

### 9.3 关键架构决策

| 决策 | 选择 | 理由 |
|------|------|------|
| 存储后端 | LanceDB + LibSQL | 向量搜索 + 结构化数据 |
| Embedding | FastEmbed (本地) → OpenAI (云) → Mock | 中国网络适配 |
| 遗忘策略 | 艾宾浩斯曲线 | 符合人类记忆科学 |
| 整合策略 | LLM 摘要 | 自动压缩上下文 |

---

## 十、未来展望 (v3.0)

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

**参考来源**:
- [MemGPT GitHub](https://github.com/letta-ai/letta)
- [Mem0 Documentation](https://docs.mem0.ai)
- [LangChain Memory](https://docs.langchain.com/docs/how-to/memory/)
- [Claude Memory (Anthropic)](https://docs.anthropic.com/claude/docs/introducing-memory-to-claude-code-agent)
