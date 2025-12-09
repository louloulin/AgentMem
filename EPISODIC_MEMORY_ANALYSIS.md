# AgentMem Episodic Memory 完整分析

## 📋 目录
1. [Episodic Memory 概念定义](#1-episodic-memory-概念定义)
2. [理论依据](#2-理论依据)
3. [代码实现架构](#3-代码实现架构)
4. [数据结构](#4-数据结构)
5. [检索策略](#5-检索策略)
6. [存储实现](#6-存储实现)
7. [使用场景](#7-使用场景)

---

## 1. Episodic Memory 概念定义

### 1.1 基本定义

**Episodic Memory（情景记忆/情节记忆）** 是 AgentMem 系统中的核心记忆类型之一，用于存储**基于时间的事件和经历**。

根据代码注释和文档：
- **中文名称**: 情景记忆、情节记忆
- **英文名称**: Episodic Memory
- **核心特征**: 基于时间的事件和经历
- **参考来源**: 参考 MIRIX 的 EpisodicMemoryManager 实现

### 1.2 在认知心理学中的含义

根据 `ag1.md` 文档：
```
记忆类型:
- episodic: 情节记忆（具体事件、对话）
- semantic: 语义记忆（知识、概念）
- procedural: 程序记忆（如何做某事）
```

**Episodic Memory** 对应认知心理学中的**情节记忆**，用于记录：
- 具体的事件（如："昨天用户说喜欢 pizza"）
- 对话片段（如："用户提到他的生日是 5 月 1 日"）
- 时间相关的经历（如："上周完成了项目里程碑"）

---

## 2. 理论依据

### 2.1 认知理论模型

AgentMem 的 Episodic Memory 实现基于以下认知理论：

#### 2.1.1 Atkinson-Shiffrin 模型
- **Long-term Memory 应该是主要检索源**
- Episodic Memory 属于 Long-term Memory 范畴
- 在检索策略中，Episodic Memory 被赋予**最高优先级**

#### 2.1.2 HCAM（Hierarchical Cognitive Architecture Model）
- **分层检索**: 粗略检索（Episodic）→ 精细检索（Working 补充）
- Episodic Memory 作为**主要来源**（90%）
- Working Memory 作为**补充上下文**（10%）

#### 2.1.3 Adaptive Framework
- **动态权重调整**:
  - Episodic Memory: 权重 **1.2**（提升主要来源）
  - Working Memory: 权重 **1.0**（正常，因为新鲜）
  - Semantic Memory: 权重 **0.9**（降低，因为范围更广）

### 2.2 检索优先级

根据 `docs/architecture/memory-architecture-analysis.md`:

```
检索策略（符合认知模型）:
1. Priority 1: Episodic Memory (Agent/User scope) - 主要来源（90%）
2. Priority 2: Working Memory (Session scope) - 补充上下文（10%）
3. Priority 3: Semantic Memory (Agent scope) - 备选
```

---

## 3. 代码实现架构

### 3.1 核心组件

AgentMem 中 Episodic Memory 的实现包含以下核心组件：

#### 3.1.1 EpisodicMemoryManager
**位置**: `crates/agent-mem-core/src/managers/episodic_memory.rs`

**职责**:
- 管理情景记忆事件的 CRUD 操作
- 提供基于时间范围的查询
- 管理重要性评分

**主要方法**:
```rust
pub struct EpisodicMemoryManager {
    pool: Arc<PgPool>,
}

impl EpisodicMemoryManager {
    pub async fn create_event(&self, event: EpisodicEvent) -> Result<EpisodicEvent>
    pub async fn get_event(&self, event_id: &str, user_id: &str) -> Result<Option<EpisodicEvent>>
    pub async fn query_events(&self, user_id: &str, query: EpisodicQuery) -> Result<Vec<EpisodicEvent>>
    pub async fn delete_event(&self, event_id: &str, user_id: &str) -> Result<bool>
    pub async fn update_importance(&self, event_id: &str, user_id: &str, importance_score: f32) -> Result<bool>
    pub async fn count_events_in_range(&self, user_id: &str, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Result<i64>
}
```

#### 3.1.2 EpisodicAgent
**位置**: `crates/agent-mem-core/src/agents/episodic_agent.rs`

**职责**:
- 专门处理 Episodic Memory 的 Agent
- 支持多种存储后端（PostgreSQL, LibSQL, MongoDB 等）
- 通过 trait-based 设计实现存储抽象

**特点**:
```rust
pub struct EpisodicAgent {
    base: BaseAgent,
    context: Arc<RwLock<AgentContext>>,
    episodic_store: Option<Arc<dyn EpisodicMemoryStore>>,  // trait-based
    initialized: bool,
}
```

**支持的操作**:
- `insert`: 插入情景记忆事件
- `search`: 搜索情景记忆
- `time_range_query`: 基于时间范围查询
- `update`: 更新事件（如重要性评分）
- `delete`: 删除事件

#### 3.1.3 MemoryIntegration (检索策略)
**位置**: `crates/agent-mem-core/src/orchestrator/memory_integration.rs`

**核心方法**: `retrieve_episodic_first()`

这是 **Episodic-first 检索策略**的核心实现，实现了基于认知理论的分层检索。

---

## 4. 数据结构

### 4.1 EpisodicEvent

**定义位置**: `crates/agent-mem-core/src/managers/episodic_memory.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodicEvent {
    /// 事件 ID
    pub id: String,
    
    /// 组织 ID（多租户隔离）
    pub organization_id: String,
    
    /// 用户 ID
    pub user_id: String,
    
    /// Agent ID
    pub agent_id: String,
    
    /// 事件发生时间（关键字段）
    pub occurred_at: DateTime<Utc>,
    
    /// 事件类型（如：conversation, action, observation）
    pub event_type: String,
    
    /// 参与者
    pub actor: Option<String>,
    
    /// 事件摘要
    pub summary: String,
    
    /// 事件详情
    pub details: Option<String>,
    
    /// 重要性评分（0.0-1.0）
    pub importance_score: f32,
    
    /// 元数据（JSONB 格式）
    pub metadata: serde_json::Value,
    
    /// 创建时间
    pub created_at: DateTime<Utc>,
    
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}
```

### 4.2 EpisodicQuery

**查询参数结构**:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodicQuery {
    /// 开始时间
    pub start_time: Option<DateTime<Utc>>,
    
    /// 结束时间
    pub end_time: Option<DateTime<Utc>>,
    
    /// 事件类型过滤
    pub event_type: Option<String>,
    
    /// 最小重要性评分
    pub min_importance: Option<f32>,
    
    /// 限制返回数量
    pub limit: Option<i64>,
}
```

### 4.3 数据库表结构

**位置**: `migrations/20251007_create_episodic_events.sql`

```sql
CREATE TABLE IF NOT EXISTS episodic_events (
    id VARCHAR(255) PRIMARY KEY,
    organization_id VARCHAR(255) NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    agent_id VARCHAR(255) NOT NULL,
    
    -- 时间信息（关键）
    occurred_at TIMESTAMPTZ NOT NULL,  -- 事件发生时间
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- 事件信息
    event_type VARCHAR(100) NOT NULL,  -- conversation, action, observation
    actor VARCHAR(255),
    summary TEXT NOT NULL,
    details TEXT,
    
    -- 重要性评分
    importance_score REAL NOT NULL DEFAULT 0.5 
        CHECK (importance_score >= 0.0 AND importance_score <= 1.0),
    
    -- 元数据
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    
    -- 外键约束
    CONSTRAINT fk_organization FOREIGN KEY (organization_id) REFERENCES organizations(id),
    CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES users(id),
    CONSTRAINT fk_agent FOREIGN KEY (agent_id) REFERENCES agents(id)
);
```

**索引优化**:
```sql
-- 性能优化索引
CREATE INDEX idx_episodic_events_user_id ON episodic_events(user_id);
CREATE INDEX idx_episodic_events_agent_id ON episodic_events(agent_id);
CREATE INDEX idx_episodic_events_occurred_at ON episodic_events(occurred_at DESC);
CREATE INDEX idx_episodic_events_event_type ON episodic_events(event_type);
CREATE INDEX idx_episodic_events_importance ON episodic_events(importance_score DESC);
CREATE INDEX idx_episodic_events_user_occurred ON episodic_events(user_id, occurred_at DESC);
CREATE INDEX idx_episodic_events_metadata ON episodic_events USING GIN (metadata);
```

### 4.4 MemoryType 枚举

**位置**: `crates/agent-mem-traits/src/types.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum MemoryType {
    #[default]
    Episodic,    // 情节性记忆 - specific events and experiences
    Procedural,  // 程序性记忆 - skills and procedures
    Semantic,    // 语义记忆 - facts and general knowledge
    Working,     // 工作记忆 - temporary information processing
    Core,        // 核心记忆 - persistent identity and preferences
    Resource,    // 资源记忆 - multimedia content and documents
    Knowledge,   // 知识记忆 - structured knowledge graphs
    Contextual,  // 上下文记忆 - environment-aware information
}
```

---

## 5. 检索策略

### 5.1 Episodic-First 检索策略

**核心方法**: `retrieve_episodic_first()`

**位置**: `crates/agent-mem-core/src/orchestrator/memory_integration.rs:324`

#### 5.1.1 检索优先级

```rust
pub async fn retrieve_episodic_first(
    &self,
    query: &str,
    agent_id: &str,
    user_id: Option<&str>,
    session_id: Option<&str>,
    max_count: usize,
) -> Result<Vec<Memory>> {
    // Priority 1: Episodic Memory (User scope) - 主要来源
    // Priority 2: Working Memory (Session scope) - 补充上下文
    // Priority 3: Semantic Memory (Agent scope) - 备选
    // Priority 4: Global Memory (Global scope) - 全局知识
}
```

#### 5.1.2 详细流程

**Step 1: 缓存检查**
```rust
let cache_key = self.normalize_cache_key(query, agent_id, user_id, session_id);
if let Some(cached) = self.get_cached(&cache_key) {
    return Ok(cached.into_iter().take(max_count).collect());
}
```

**Step 2: Priority 1 - Episodic Memory (User scope)**
```rust
if let Some(uid) = user_id {
    let episodic_scope = MemoryScope::User {
        agent_id: agent_id.to_string(),
        user_id: uid.to_string(),
    };
    
    // 查询 max_count * 2（因为是主要来源）
    let memories = self.memory_engine
        .search_memories(query, Some(episodic_scope), Some(max_count * 2))
        .await?;
    
    for mut memory in memories {
        if seen_ids.insert(memory.id.clone()) {
            // 权重：1.2（提升主要来源）
            if let Some(score) = memory.score() {
                memory.set_score(score * self.config.episodic_weight as f64);
            }
            all_memories.push(memory);
        }
    }
}
```

**Step 3: Priority 2 - Working Memory (Session scope)**
```rust
if let (Some(uid), Some(sid)) = (user_id, session_id) {
    let working_scope = MemoryScope::Session {
        agent_id: agent_id.to_string(),
        user_id: uid.to_string(),
        session_id: sid.to_string(),
    };
    
    // 查询 max_count / 2（只是补充）
    let memories = self.memory_engine
        .search_memories(query, Some(working_scope), Some(max_count / 2))
        .await?;
    
    for memory in memories {
        if seen_ids.insert(memory.id.clone()) {
            // 权重：1.0（正常，因为新鲜）
            all_memories.push(memory);
        }
    }
}
```

**Step 4: Priority 3 - Semantic Memory (Agent scope)**
```rust
if all_memories.len() < max_count {
    let semantic_scope = MemoryScope::Agent(agent_id.to_string());
    
    let remaining = max_count.saturating_sub(all_memories.len());
    let memories = self.memory_engine
        .search_memories(query, Some(semantic_scope), Some(remaining * 2))
        .await?;
    
    for mut memory in memories {
        if seen_ids.insert(memory.id.clone()) {
            // 权重：0.9（降低，范围更广）
            if let Some(score) = memory.score() {
                memory.set_score(score * self.config.semantic_weight as f64);
            }
            all_memories.push(memory);
        }
    }
}
```

**Step 5: 排序和去重**
```rust
// 按分数排序
all_memories.sort_by(|a, b| {
    b.score().unwrap_or(0.0)
        .partial_cmp(&a.score().unwrap_or(0.0))
        .unwrap_or(std::cmp::Ordering::Equal)
});

// 返回 Top-N
Ok(all_memories.into_iter().take(max_count).collect())
```

#### 5.1.3 权重配置

**配置位置**: `MemoryIntegrationConfig`

```rust
pub struct MemoryIntegrationConfig {
    /// Episodic Memory权重（Long-term Memory优先）
    pub episodic_weight: f32,  // 默认 1.2
    
    /// Working Memory权重
    pub working_weight: f32,    // 默认 1.0
    
    /// Semantic Memory权重
    pub semantic_weight: f32,    // 默认 0.9
}
```

### 5.2 在 Orchestrator 中的使用

**位置**: `crates/agent-mem-core/src/orchestrator/mod.rs:959`

```rust
async fn retrieve_memories(&self, request: &ChatRequest) -> Result<Vec<Memory>> {
    // 🆕 Phase 1: 使用 Episodic-first检索（基于认知理论）
    let memories = self
        .memory_integrator
        .retrieve_episodic_first(
            &request.message,
            &request.agent_id,
            Some(&request.user_id),
            Some(&request.session_id),
            max_count,
        )
        .await?;
    
    // Phase 2/3: 过滤和排序
    let memories = self.memory_integrator.filter_by_relevance(memories);
    let memories = self.memory_integrator.sort_memories(memories);
    
    // Phase 5: 去重和压缩
    let memories = self.memory_integrator.deduplicate_memories(memories);
    let memories = self.memory_integrator.compress_memories(memories);
    
    Ok(memories)
}
```

---

## 6. 存储实现

### 6.1 Trait-Based 设计

AgentMem 使用 **trait-based** 设计，支持多种存储后端：

**Trait 定义**: `crates/agent-mem-traits/src/memory_store.rs`

```rust
#[async_trait]
pub trait EpisodicMemoryStore: Send + Sync {
    async fn create_event(&self, event: EpisodicEvent) -> Result<EpisodicEvent>;
    async fn get_event(&self, event_id: &str, user_id: &str) -> Result<Option<EpisodicEvent>>;
    async fn query_events(&self, user_id: &str, query: EpisodicQuery) -> Result<Vec<EpisodicEvent>>;
    async fn update_event(&self, event: EpisodicEvent) -> Result<bool>;
    async fn delete_event(&self, event_id: &str, user_id: &str) -> Result<bool>;
    async fn update_importance(&self, event_id: &str, user_id: &str, importance_score: f32) -> Result<bool>;
    async fn count_events_in_range(&self, user_id: &str, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Result<i64>;
    async fn get_recent_events(&self, user_id: &str, limit: i64) -> Result<Vec<EpisodicEvent>>;
}
```

**支持的存储后端**:
- PostgreSQL (通过 `EpisodicMemoryManager`)
- LibSQL
- MongoDB
- 其他实现 `EpisodicMemoryStore` trait 的后端

### 6.2 存储范围（Memory Scope）

Episodic Memory 主要存储在以下范围：

1. **User Scope** (主要)
   ```rust
   MemoryScope::User {
       agent_id: String,
       user_id: String,
   }
   ```
   - 用户个人的长期记忆
   - 跨会话持久化

2. **Agent Scope** (备选)
   ```rust
   MemoryScope::Agent(agent_id: String)
   ```
   - Agent 级别的共享记忆

3. **Global Scope** (特殊场景)
   ```rust
   MemoryScope::Global
   ```
   - 全局知识（如商品信息）

---

## 7. 使用场景

### 7.1 典型使用场景

#### 场景 1: 存储用户偏好
```rust
let event = EpisodicEvent {
    id: generate_id(),
    organization_id: "org-123".to_string(),
    user_id: "user-456".to_string(),
    agent_id: "agent-789".to_string(),
    occurred_at: Utc::now(),
    event_type: "conversation".to_string(),
    actor: Some("user".to_string()),
    summary: "用户喜欢 pizza".to_string(),
    details: Some("在对话中提到喜欢意大利披萨".to_string()),
    importance_score: 0.8,
    metadata: json!({"category": "preference"}),
    created_at: Utc::now(),
    updated_at: Utc::now(),
};

manager.create_event(event).await?;
```

#### 场景 2: 检索用户历史对话
```rust
let query = EpisodicQuery {
    start_time: Some(Utc::now() - Duration::days(7)),
    end_time: Some(Utc::now()),
    event_type: Some("conversation".to_string()),
    min_importance: Some(0.5),
    limit: Some(10),
};

let events = manager.query_events("user-456", query).await?;
```

#### 场景 3: Episodic-First 检索
```rust
// 在聊天请求中自动使用
let memories = memory_integrator
    .retrieve_episodic_first(
        "用户喜欢什么食物？",
        "agent-789",
        Some("user-456"),
        Some("session-abc"),
        10,
    )
    .await?;
// 返回: 优先包含 Episodic Memory（用户偏好），补充 Working Memory（当前会话）
```

### 7.2 与其他记忆类型的关系

```
记忆层次结构:
┌─────────────────────────────────────┐
│  Working Memory (Session scope)     │  ← 临时，当前会话
│  - 快速访问 (<1ms)                   │
│  - 容量有限 (4K-128K tokens)         │
└─────────────────────────────────────┘
            ↕️ 数据交换
┌─────────────────────────────────────┐
│  Episodic Memory (User scope)       │  ← 长期，跨会话 ⭐
│  - 历史对话                          │
│  - 用户偏好                          │
│  - 事件记录                          │
│  - 需要检索 (~100ms)                 │
│  - 容量无限                           │
└─────────────────────────────────────┘
            ↕️ 数据交换
┌─────────────────────────────────────┐
│  Semantic Memory (Agent/Global)     │  ← 知识库
│  - 通用知识                          │
│  - 领域知识                          │
└─────────────────────────────────────┘
```

### 7.3 生命周期

- **TTL (Time To Live)**: 根据配置，Episodic Memory 的 TTL 通常为 **1小时** (3600秒)
- **持久化**: 存储在数据库中，跨会话持久化
- **重要性评分**: 通过 `importance_score` 字段管理，范围 0.0-1.0

---

## 8. 总结

### 8.1 核心要点

1. **Episodic Memory = 情景记忆/情节记忆**
   - 存储基于时间的事件和经历
   - 对应认知心理学中的情节记忆

2. **检索优先级最高**
   - 在 Episodic-first 策略中，优先级 1
   - 权重 1.2（提升主要来源）

3. **存储范围**
   - 主要在 User scope（用户长期记忆）
   - 跨会话持久化

4. **实现特点**
   - Trait-based 设计，支持多种存储后端
   - 基于认知理论（Atkinson-Shiffrin, HCAM）
   - 完整的 CRUD 操作支持

### 8.2 关键文件

- **管理器**: `crates/agent-mem-core/src/managers/episodic_memory.rs`
- **Agent**: `crates/agent-mem-core/src/agents/episodic_agent.rs`
- **检索策略**: `crates/agent-mem-core/src/orchestrator/memory_integration.rs`
- **Trait 定义**: `crates/agent-mem-traits/src/memory_store.rs`
- **数据库迁移**: `migrations/20251007_create_episodic_events.sql`
- **类型定义**: `crates/agent-mem-traits/src/types.rs`

---

**生成时间**: 2025-01-XX
**分析范围**: AgentMem 代码库完整分析
