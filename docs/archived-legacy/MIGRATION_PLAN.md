# Feature-Paper 到 Core 迁移计划

## 🎯 目标

将 feature-paper 分支的核心功能迁移到 core 模块，mem 层只保留对外 API。

## 📐 新架构设计

```
┌─────────────────────────────────────────────────────────────┐
│  Layer 1: Memory API (agent-mem)                            │
│  - memory.rs (< 500 行)                                     │
│  - 7 个 mem0 兼容方法                                        │
│  - 零业务逻辑，纯粹的 API 封装                                │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│  Layer 2: Orchestrator (agent-mem)                          │
│  - orchestrator.rs (< 800 行)                               │
│  - 智能路由：根据内容类型路由到对应 Agent                      │
│  - 协调 core 模块的 Engines                                  │
│  - 不实现业务逻辑                                             │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│  Layer 3: Core Engines (agent-mem-core)                     │
│  - IntelligenceEngine (新增)                                │
│    ├─ FactExtractor                                         │
│    ├─ DecisionEngine                                        │
│    └─ ActionExecutor                                        │
│  - RetrievalEngine (已存在)                                 │
│    ├─ VectorSearch                                          │
│    ├─ KeywordSearch                                         │
│    └─ HybridSearch (RRF)                                    │
│  - StorageEngine (已存在)                                   │
│    ├─ LibSQL (结构化数据)                                    │
│    └─ LanceDB (向量数据)                                     │
│  - Agents (已存在)                                          │
│    ├─ SemanticAgent, EpisodicAgent, CoreAgent, etc.         │
│    └─ 每个 Agent 负责特定类型的记忆                           │
└─────────────────────────────────────────────────────────────┘
```

## 📋 迁移任务清单

### Phase 1: 创建 IntelligenceEngine (新增到 core)

**目标**: 将智能功能从 orchestrator 迁移到 core

**文件**: `crates/agent-mem-core/src/engines/intelligence_engine.rs`

**功能**:
```rust
pub struct IntelligenceEngine {
    fact_extractor: Arc<FactExtractor>,
    decision_engine: Arc<MemoryDecisionEngine>,
    llm_provider: Arc<dyn LLMProvider>,
}

impl IntelligenceEngine {
    /// 智能添加流程
    pub async fn process_intelligent_add(
        &self,
        content: String,
        existing_memories: Vec<MemoryItem>,
    ) -> Result<Vec<MemoryAction>> {
        // 1. 提取事实
        let facts = self.fact_extractor.extract_facts(&content).await?;
        
        // 2. 决策
        let actions = self.decision_engine.decide_actions(facts, existing_memories).await?;
        
        Ok(actions)
    }
}
```

**依赖**:
- `agent-mem-intelligence` (已存在)
- `agent-mem-llm` (已存在)

### Phase 2: 增强 RetrievalEngine (已存在，需增强)

**目标**: 确保 RetrievalEngine 支持混合搜索

**文件**: `crates/agent-mem-core/src/engines/retrieval_engine.rs`

**功能**:
```rust
pub struct RetrievalEngine {
    hybrid_storage: Arc<RwLock<HybridStorageManager>>,
    embedder: Arc<dyn Embedder>,
}

impl RetrievalEngine {
    /// 混合搜索（Vector + Keyword + RRF）
    pub async fn search_hybrid(
        &self,
        query: String,
        limit: usize,
        filters: Option<HashMap<String, Value>>,
    ) -> Result<Vec<MemoryItem>> {
        // 1. 并行执行向量搜索和关键词搜索
        let (vector_results, keyword_results) = tokio::join!(
            self.search_vector(&query, limit),
            self.search_keyword(&query, limit)
        );
        
        // 2. RRF 融合
        let merged = self.merge_with_rrf(vector_results?, keyword_results?);
        
        Ok(merged)
    }
}
```

### Phase 3: 简化 Orchestrator (agent-mem)

**目标**: Orchestrator 只负责路由和协调，不实现业务逻辑

**文件**: `crates/agent-mem/src/orchestrator.rs`

**修改**:
```rust
pub struct MemoryOrchestrator {
    // Agents (保留)
    core_agent: Option<Arc<RwLock<CoreAgent>>>,
    episodic_agent: Option<Arc<RwLock<EpisodicAgent>>>,
    semantic_agent: Option<Arc<RwLock<SemanticAgent>>>,
    procedural_agent: Option<Arc<RwLock<ProceduralAgent>>>,
    
    // Engines (新增，从 core 导入)
    intelligence_engine: Option<Arc<IntelligenceEngine>>,
    retrieval_engine: Option<Arc<RetrievalEngine>>,
    
    // 移除：fact_extractor, decision_engine, llm_provider, embedder
    // 移除：hybrid_storage, history_store (这些在 Engines 内部)
    
    config: OrchestratorConfig,
}

impl MemoryOrchestrator {
    /// 智能添加（调用 IntelligenceEngine）
    pub async fn add_memory_intelligent(
        &self,
        content: String,
        agent_id: String,
        user_id: Option<String>,
        metadata: Option<HashMap<String, Value>>,
    ) -> Result<AddMemoryResult> {
        // 1. 搜索相似记忆（调用 RetrievalEngine）
        let existing_memories = if let Some(engine) = &self.retrieval_engine {
            engine.search_hybrid(content.clone(), 10, None).await?
        } else {
            Vec::new()
        };
        
        // 2. 智能处理（调用 IntelligenceEngine）
        let actions = if let Some(engine) = &self.intelligence_engine {
            engine.process_intelligent_add(content, existing_memories).await?
        } else {
            // 降级：直接添加
            vec![MemoryAction::Add { content, metadata }]
        };
        
        // 3. 执行操作（路由到 Agents）
        let results = self.execute_memory_actions(actions).await?;
        
        Ok(AddMemoryResult { operations: results, ... })
    }
    
    /// 搜索记忆（调用 RetrievalEngine）
    pub async fn search_memories(
        &self,
        query: String,
        agent_id: String,
        user_id: Option<String>,
        limit: usize,
        memory_type: Option<MemoryType>,
    ) -> Result<Vec<MemoryItem>> {
        // 直接调用 RetrievalEngine
        if let Some(engine) = &self.retrieval_engine {
            engine.search_hybrid(query, limit, None).await
        } else {
            // 降级：调用 Agents 搜索
            self.search_via_agents(query, memory_type, limit).await
        }
    }
}
```

### Phase 4: 简化 Memory API (agent-mem)

**目标**: Memory 只保留 7 个核心方法，移除冗余功能

**文件**: `crates/agent-mem/src/memory.rs`

**修改**:
```rust
pub struct Memory {
    orchestrator: Arc<RwLock<MemoryOrchestrator>>,
    default_user_id: Option<String>,
    default_agent_id: String,
    
    // 移除：batch_processor, query_cache, chat_engine, user_manager,
    //       multimodal_engine, tool_engine
}

impl Memory {
    // 保留 7 个核心方法
    pub async fn add(&self, content: impl Into<String>) -> Result<String>
    pub async fn search(&self, query: &str) -> Result<Vec<MemoryItem>>
    pub async fn get(&self, memory_id: &str) -> Result<MemoryItem>
    pub async fn get_all(&self) -> Result<Vec<MemoryItem>>
    pub async fn update(&self, memory_id: &str, data: HashMap<String, Value>) -> Result<MemoryItem>
    pub async fn delete(&self, memory_id: &str) -> Result<()>
    pub async fn delete_all(&self) -> Result<usize>
    
    // 保留智能添加（调用 orchestrator.add_memory_intelligent）
    pub async fn add_intelligent(&self, content: impl Into<String>) -> Result<AddMemoryResult>
    
    // 移除：chat, add_multimodal, backup, restore, create_user, register_tool, etc.
}
```

## 🔄 迁移步骤

### Step 1: 创建 IntelligenceEngine (2 小时)

1. 创建 `crates/agent-mem-core/src/engines/intelligence_engine.rs`
2. 从 feature-paper 的 orchestrator 中提取智能处理逻辑
3. 实现 `process_intelligent_add` 方法
4. 添加单元测试

### Step 2: 增强 RetrievalEngine (1 小时)

1. 检查现有的 `retrieval_engine.rs`
2. 确保支持 `search_hybrid` 方法
3. 实现 RRF 融合算法
4. 添加单元测试

### Step 3: 重构 Orchestrator (2 小时)

1. 移除冗余字段（fact_extractor, decision_engine, etc.）
2. 添加 `intelligence_engine` 和 `retrieval_engine` 字段
3. 修改 `add_memory_intelligent` 调用 IntelligenceEngine
4. 修改 `search_memories` 调用 RetrievalEngine
5. 保留 Agent 路由逻辑

### Step 4: 简化 Memory API (1 小时)

1. 移除冗余字段（chat_engine, multimodal_engine, etc.）
2. 移除冗余方法（chat, add_multimodal, backup, etc.）
3. 保留 7 个核心方法 + add_intelligent
4. 更新文档

### Step 5: 测试和验证 (1 小时)

1. 运行 `cargo test`
2. 运行 `mem0-api-demo`
3. 验证智能添加功能
4. 验证混合搜索功能

## 📊 预期结果

### 代码量对比

| 模块 | feature-paper | 迁移后 | 减少 |
|------|--------------|--------|------|
| memory.rs | 1,844 行 | < 500 行 | -73% |
| orchestrator.rs | 2,648 行 | < 800 行 | -70% |
| **总计** | **4,492 行** | **< 1,300 行** | **-71%** |

### 新增到 core

| 文件 | 行数 | 说明 |
|------|------|------|
| `intelligence_engine.rs` | ~300 行 | 智能处理引擎 |
| `retrieval_engine.rs` (增强) | +100 行 | 混合搜索增强 |
| **总计** | **~400 行** | 核心功能 |

### 净减少

- **总代码量**: 4,492 → 1,700 行 (-62%)
- **mem 层**: 4,492 → 1,300 行 (-71%)
- **core 层**: +400 行（新增核心功能）

## ✅ 验收标准

1. ✅ 所有 7 个 mem0 API 方法正常工作
2. ✅ 智能添加功能正常（FactExtractor + DecisionEngine）
3. ✅ 混合搜索功能正常（Vector + Keyword + RRF）
4. ✅ Agent 路由正常（根据 memory_type 路由）
5. ✅ 数据持久化正常（LibSQL + LanceDB）
6. ✅ 编译通过，测试通过
7. ✅ 代码量减少 > 60%

## 🚀 开始执行

准备好了吗？让我们开始迁移！

