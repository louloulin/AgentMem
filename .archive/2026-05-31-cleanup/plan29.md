# AgentMem v2.2 全面优化计划 (plan29.md)

## Context

基于 plan28.md 的 MVP 完成状态，结合 **AgentMem + EVIF**，对标 **OpenViking**，制定全面优化计划。

**参考项目**:
- [AgentMem](../contextengine/agentmen/) - AI Agent 记忆系统 (~137K 行 Rust)
- [EVIF](../evif/) - Agent 原生连接层 (~120K 行 Rust)
- [OpenViking](../OpenViking/) - AI Agent 上下文数据库

---

## 一、项目概览

### 1.1 三大系统核心数据

| 项目 | 代码行数 | Crates | 核心功能 |
|------|----------|--------|----------|
| **AgentMem** | ~137,000 | 30+ | 记忆管理、5种搜索引擎、Agent生命周期 |
| **EVIF** | ~120,000 | 13 | ContextFS、SkillFS、PipeFS、VectorFS、40+存储插件 |
| **OpenViking** | - | - | VikingFS、L0/L1/L2、目录递归检索 |

### 1.2 EVIF 核心架构 (ASCII)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            EVIF 架构                                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Access Layer (接入层)                                                    │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  CLI (60+ cmds) │ REST API (150 endpoints) │ MCP Server │ FUSE   │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                     │                                     │
│  ┌─────────────────────────────────┼─────────────────────────────────────┐│
│  │                         Core Layer (核心层)                           ││
│  │  ┌───────────────────────────────────────────────────────────────┐   ││
│  │  │         MountTable (Radix Tree) - 插件路由                      │   ││
│  │  │  /context → ContextFS                                      │   ││
│  │  │  /skills → SkillFS                                        │   ││
│  │  │  /pipes → PipeFS                                          │   ││
│  │  │  /memories → VectorFS                                      │   ││
│  │  │  /queue → QueueFS                                         │   ││
│  │  └───────────────────────────────────────────────────────────────┘   ││
│  │                                 │                                   ││
│  │  ┌─────────────────────────────┴─────────────────────────────────┐ ││
│  │  │                    Plugin Layer (插件层)                        │ ││
│  │  │                                                                 │ ││
│  │  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  │ ││
│  │  │  │ContextFS│  │ SkillFS │  │  PipeFS │  │VectorFS │  │ ││
│  │  │  │ L0/L1/L2│  │SKILL.md │  │多Agent协调│  │ 语义检索 │  │ ││
│  │  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘  │ ││
│  │  │                                                                 │ ││
│  │  │  ┌──────────────────────────────────────────────────────────┐   │ ││
│  │  │  │              Storage Plugins (存储插件 40+)              │   │ ││
│  │  │  │  S3 │ GCS │ Azure │ OSS │ Redis │ PostgreSQL │ ... │   │ ││
│  │  │  └──────────────────────────────────────────────────────────┘   │ ││
│  │  └───────────────────────────────────────────────────────────────┘ ││
│  └────────────────────────────────────────────────────────────────────┼─────┘│
└─────────────────────────────────────────────────────────────────────────────┼─────┘
```

### 1.3 AgentMem 核心架构 (ASCII)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          AgentMem 架构                                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Access Layer (接入层)                                                    │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  Python SDK (15方法) │ REST API (65端点) │ MCP Server │ CLI      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                     │                                     │
│  ┌─────────────────────────────────┼─────────────────────────────────────┐│
│  │                       Core Layer (核心层)                             ││
│  │  ┌───────────────────────────────────────────────────────────────┐   ││
│  │  │                     MemoryManager                              │   ││
│  │  │  ├─ MemoryOperations (操作)                                    │   ││
│  │  │  ├─ MemoryLifecycle (生命周期)                                │   ││
│  │  │  ├─ MemoryHistory (历史)                                     │   ││
│  │  │  └─ 智能组件 (FactExtractor, DecisionEngine, Deduplicator)     │   ││
│  │  └───────────────────────────────────────────────────────────────┘   ││
│  │                                 │                                   ││
│  │  ┌─────────────────────────────┴─────────────────────────────────┐ ││
│  │  │                   Search Layer (搜索层)                        │ ││
│  │  │                                                                 │ ││
│  │  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  │ ││
│  │  │  │Vector   │  │  BM25   │  │  Hybrid  │  │ Adaptive │  │ ││
│  │  │  │ Search  │  │  Search │  │  Search  │  │  Search │  │ ││
│  │  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘  │ ││
│  │  │                                                                 │ ││
│  │  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  │ ││
│  │  │  │  Fuzzy  │  │ Hierarch │  │ Reranker │  │ Learning │  │ ││
│  │  │  │  Search │  │  Search │  │          │  │  Engine  │  │ ││
│  │  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘  │ ││
│  │  └───────────────────────────────────────────────────────────────┘ ││
│  │                                 │                                   ││
│  │  ┌─────────────────────────────┴─────────────────────────────────┐ ││
│  │  │                Storage Layer (存储层)                           │ ││
│  │  │                                                                 │ ││
│  │  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  │ ││
│  │  │  │ LibSQL   │  │PostgreSQL│  │ LanceDB  │  │ Qdrant   │  │ ││
│  │  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘  │ ││
│  │  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  │ ││
│  │  │  │ Chroma   │  │Pinecone  │  │ Weaviate │  │ Milvus   │  │ ││
│  │  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘  │ ││
│  │  └───────────────────────────────────────────────────────────────┘ ││
│  └────────────────────────────────────────────────────────────────────┼─────┘│
└─────────────────────────────────────────────────────────────────────────────┼─────┘
```

---

## 二、融合架构 (AgentMem + EVIF → 对标 OpenViking)

### 2.1 融合总览 (ASCII)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                  AgentMem + EVIF 融合架构 → 对标 OpenViking                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌───────────────────────────────────────────────────────────────────────┐   │
│  │                        Access Layer (统一接入层)                      │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │   │
│  │  │  Python  │  │   REST   │  │   MCP    │  │   CLI    │        │   │
│  │  │   SDK    │  │   API    │  │  Server  │  │  60+ cmds│        │   │
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘        │   │
│  └───────────────────────────────────────────────────────────────────────┘   │
│                                     │                                     │
│  ┌─────────────────────────────────┼─────────────────────────────────────┐│
│  │                    Unified Core Layer (统一核心层)                       ││
│  │  ┌───────────────────────────────────────────────────────────────┐   ││
│  │  │              MountTable (Radix Tree) - 统一路由                │   ││
│  │  │                                                               │   ││
│  │  │  /context ──→ ContextFS (EVIF)     │ L0/L1/L2 分层上下文   │   ││
│  │  │  /skills ───→ SkillFS (EVIF)      │ SKILL.md 技能管理    │   ││
│  │  │  /pipes ────→ PipeFS (EVIF)      │ 多Agent协调        │   ││
│  │  │  /memories ─→ VectorFS (EVIF)     │ 向量检索          │   ││
│  │  │  /queue ────→ QueueFS (EVIF)     │ 任务队列          │   ││
│  │  │                                                               │   ││
│  │  │  /mem ──────→ MemoryManager (AgentMem) │ 记忆管理       │   ││
│  │  │  /search ───→ SearchEngine (AgentMem) │ 5种搜索引擎   │   ││
│  │  │  /agent ────→ AgentOrchestrator (AgentMem) │ Agent生命周期│   ││
│  │  └───────────────────────────────────────────────────────────────┘   ││
│  │                                 │                                   ││
│  │  ┌─────────────────────────────┴─────────────────────────────────┐ ││
│  │  │                   Plugin Layer (统一插件层)                     │ ││
│  │  │                                                                 │ ││
│  │  │  ┌───────────────────────────────────────────────────────┐   │ ││
│  │  │  │         Storage Plugins (存储插件 40+)                   │   │ ││
│  │  │  │  S3 │ GCS │ Azure │ OSS │ Redis │ PostgreSQL │ LibSQL │   │ ││
│  │  │  └───────────────────────────────────────────────────────┘   │ ││
│  │  │                                                                 │ ││
│  │  │  ┌───────────────────────────────────────────────────────┐   │ ││
│  │  │  │         AI Service Plugins (AI服务插件)                  │   │ ││
│  │  │  │  OpenAI │ Claude │ Gemini │ GPT │ Ollama │ LangChain │   │ ││
│  │  │  └───────────────────────────────────────────────────────┘   │ ││
│  │  │                                                                 │ ││
│  │  │  ┌───────────────────────────────────────────────────────┐   │ ││
│  │  │  │         AgentMem Intelligence (智能组件)                 │   │ ││
│  │  │  │  FactExtractor │ DecisionEngine │ Deduplicator │ LLM   │   │ ││
│  │  │  └───────────────────────────────────────────────────────┘   │ ││
│  │  └───────────────────────────────────────────────────────────────┘ ││
│  └────────────────────────────────────────────────────────────────────┼─────┘│
│                                                                     │     │
│  ┌─────────────────────────────────────────────────────────────────┼─────┐│
│  │                   OpenViking Compatibility Layer                    │     ││
│  │  ┌────────────────────────────────────────────────────────────┐ │     ││
│  │  │                    VikingFS 兼容层                            │ │     ││
│  │  │  ├─ URI 转换 (viking:// ↔ /local/)                       │ │     ││
│  │  │  ├─ 分层读取 (L0/L1/L2)                                   │ │     ││
│  │  │  ├─ 目录递归检索                                          │ │     ││
│  │  │  ├─ 意图分析器 (IntentAnalyzer)                          │ │     ││
│  │  │  └─ 可视化检索轨迹                                        │ │     ││
│  │  └────────────────────────────────────────────────────────────┘ │     ││
│  └─────────────────────────────────────────────────────────────────┼─────┘│
└─────────────────────────────────────────────────────────────────────────────┼─────┘
```

### 2.2 融合策略

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           融合策略                                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  EVIF 提供 (拿来即用):                                                      │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  ContextFS       → L0/L1/L2 分层上下文、LLM 摘要生成                 │   │
│  │  SkillFS         → SKILL.md 解析、技能发现、技能执行                  │   │
│  │  PipeFS          → 多Agent通信、任务协调、原子操作                     │   │
│  │  VectorFS        → 向量检索、嵌入管理                                 │   │
│  │  QueueFS         → 任务队列、异步处理                                │   │
│  │  40+ 存储插件   → S3/GCS/Azure/OSS/Redis/PostgreSQL/LibSQL         │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                     │                                     │
│                                     ▼                                     │
│  AgentMem 增强 (深度融合):                                                  │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  多搜索引擎       → 5种搜索引擎 (向量/BM25/混合/模糊/自适应)           │   │
│  │  记忆管理        → 重要性评估、去重、压缩、生命周期                    │   │
│  │  Agent 生命周期  → 状态机、任务编排、多Agent协作                      │   │
│  │  审计日志        → 完整审计系统、安全事件追踪                          │   │
│  │  可观测性        → Prometheus + OpenTelemetry + Grafana                │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                     │                                     │
│                                     ▼                                     │
│  新增功能 (对标 OpenViking):                                                │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  VikingFS 兼容层  → OpenViking API 兼容、URI 转换                     │   │
│  │  目录递归检索     → Hierarchical Retriever、分数传播                   │   │
│  │  意图分析器      → 查询分类、实体提取、查询计划                       │   │
│  │  可视化轨迹      → 检索过程可视化、分析报告                           │   │
│  │  记忆压缩 V2     → 会话压缩、长期记忆提取                            │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 三、功能差距与融合矩阵

### 3.1 功能对比表

| 功能 | OpenViking | AgentMem | EVIF | 融合方案 |
|------|------------|----------|------|----------|
| **VikingFS** | ✅ | ❌ | ❌ | **新增** |
| **L0/L1/L2 分层** | ✅ | ❌ | ✅ ContextFS | EVIF 已有 |
| **目录递归检索** | ✅ | ⚠️ | ❌ | **新增** |
| **意图分析** | ✅ | ❌ | ❌ | **新增** |
| **可视化检索** | ✅ | ❌ | ❌ | **新增** |
| **记忆压缩** | ✅ V2 | ⚠️ | ❌ | AgentMem 增强 |
| **SkillFS** | ❌ | ❌ | ✅ | EVIF 已有 |
| **PipeFS** | ❌ | ⚠️ | ✅ | EVIF 已有 |
| **多搜索引擎** | 1种 | 5种 | 1种 | AgentMem 已有 |
| **多存储后端** | ⚠️ | ✅ 27种 | ✅ 40+ | 合并领先 |
| **WASM 插件** | ❌ | ✅ | ❌ | AgentMem 已有 |
| **多Agent协调** | ❌ | ⚠️ | ✅ PipeFS | EVIF 已有 |
| **审计日志** | ❌ | ✅ | ❌ | AgentMem 已有 |
| **可观测性** | ❌ | ✅ | ⚠️ | AgentMem 已有 |

### 3.2 融合后的独特优势

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        融合后的独特优势                                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  1. 多搜索引擎 (AgentMem 独有)                                            │
│     ├─ 向量搜索 (HNSW/IVF)                                               │
│     ├─ BM25 全文搜索                                                      │
│     ├─ 混合搜索 (RRF 融合)                                               │
│     ├─ 模糊搜索 (编辑距离)                                                │
│     └─ 自适应搜索 (Thompson Sampling)                                       │
│                                                                             │
│  2. 多存储后端 (合并领先)                                                 │
│     ├─ AgentMem: 27 种 (LibSQL/PostgreSQL/LanceDB/Qdrant/Chroma/...)       │
│     └─ EVIF: 40+ 种 (S3/GCS/Azure/OSS/Redis/Slack/GitHub/Notion/...)     │
│                                                                             │
│  3. 多Agent协调 (EVIF PipeFS)                                            │
│     ├─ 原子 Claim 操作                                                    │
│     ├─ 等待结果 (wait_for_result)                                        │
│     └─ 广播订阅 (broadcast/subscribers)                                    │
│                                                                             │
│  4. 技能管理 (EVIF SkillFS)                                             │
│     ├─ SKILL.md 解析                                                     │
│     ├─ 技能发现                                                         │
│     └─ 技能执行                                                         │
│                                                                             │
│  5. Agent 生命周期 (AgentMem)                                             │
│     ├─ 状态机 (idle/thinking/executing/waiting/error)                      │
│     ├─ 任务编排                                                         │
│     └─ 协作                                                             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 四、核心模块设计

### 4.1 VikingFS 兼容层

```rust
// crates/agent-mem-evif/src/viking_fs.rs

/// VikingFS - OpenViking 兼容文件系统
///
/// 结合 AgentMem 记忆管理和 EVIF ContextFS
pub struct VikingFS {
    /// 存储后端
    storage: Arc<dyn MemoryStorage>,
    /// 向量检索
    vector_store: Arc<dyn VectorStore>,
    /// 嵌入生成
    embedder: Arc<dyn Embedder>,
    /// 重排序
    reranker: Option<Arc<dyn Reranker>>,
    /// LLM 提供商
    llm_provider: Option<Arc<dyn LLMProvider>>,
}

/// URI 转换
/// viking://memory/user/session → /local/memory/user/session
pub fn uri_to_path(uri: &str) -> PathBuf {
    uri.replace("viking://", "/local/")
}

/// 分层读取
pub async fn read_layered(
    &self,
    uri: &str,
    layer: ContextLayer,
) -> Result<LayerContent> {
    match layer {
        ContextLayer::Abstract => self.read_abstract(uri).await,
        ContextLayer::Overview => self.read_overview(uri).await,
        ContextLayer::Full => self.read_full(uri).await,
    }
}
```

### 4.2 分层上下文加载器

```rust
// crates/agent-mem-evif/src/layered_context.rs

/// 上下文层级
pub enum ContextLayer {
    /// L0: 抽象层 (~100 tokens)
    Abstract,
    /// L1: 概述层 (500-1000 tokens)
    Overview,
    /// L2: 完整层 (无限制)
    Full,
}

/// 分层上下文加载器 (基于 EVIF ContextFS)
pub struct LayeredContextLoader {
    /// LLM 提供商 (用于摘要生成)
    llm_provider: Arc<dyn LLMProvider>,
    /// 内容提取器
    extractor: Arc<dyn ContentExtractor>,
}

impl LayeredContextLoader {
    /// 按需加载指定层级
    pub async fn load(&self, uri: &str, layer: ContextLayer) -> Result<LayerContent> {
        match layer {
            ContextLayer::Abstract => self.generate_abstract(uri).await,
            ContextLayer::Overview => self.generate_overview(uri).await,
            ContextLayer::Full => self.load_full(uri).await,
        }
    }

    /// LLM 摘要生成 (基于 EVIF ContextFS)
    async fn summarize_llm(&self, content: &str, mode: &str) -> Result<String> {
        // 使用 OpenAI API 生成摘要
        // 降级方案: 头行截断
    }
}
```

### 4.3 目录递归检索器

```rust
// crates/agent-mem-evif/src/hierarchical_search.rs

/// 分层检索器 (对标 OpenViking HierarchicalRetriever)
pub struct HierarchicalRetriever {
    /// 存储后端
    storage: Arc<dyn MemoryStorage>,
    /// 嵌入器
    embedder: Arc<dyn Embedder>,
    /// 重排序
    reranker: Option<Arc<dyn Reranker>>,
    /// 最大深度
    max_depth: usize,
    /// 分数传播系数
    score_propagation_alpha: f32,
    /// 目录主导比率
    directory_dominance_ratio: f32,
}

impl HierarchicalRetriever {
    /// 分层检索
    pub async fn retrieve(
        &self,
        query: &str,
        root_uri: Option<&str>,
        limit: usize,
    ) -> Result<Vec<SearchHit>> {
        // 1. 全局搜索 (语义 + 目录)
        let global_hits = self.global_search(query).await?;

        // 2. 递归检索
        let hierarchical_hits = self.search_recursive(
            query,
            root_uri.unwrap_or("/"),
            0,
        ).await?;

        // 3. 分数传播
        let propagated = self.propagate_scores(hierarchical_hits);

        // 4. 目录主导过滤
        let filtered = self.filter_by_dominance(propagated);

        // 5. 重排序
        self.rerank(filtered, limit).await
    }

    /// 分数传播
    fn propagate_scores(&self, nodes: Vec<DirNode>) -> Vec<SearchHit> {
        nodes.into_iter().map(|node| {
            let child_max = node.children
                .iter()
                .map(|c| self.propagate_scores(c.clone()))
                .flatten()
                .map(|h| h.score)
                .fold(0.0f32, |a, b| a.max(b));

            let propagated = (node.self_score * (1.0 - self.score_propagation_alpha))
                .max(child_max * self.score_propagation_alpha);

            SearchHit { score: propagated, ..node.to_hit() }
        }).collect()
    }
}
```

### 4.4 意图分析器

```rust
// crates/agent-mem-evif/src/intent_analyzer.rs

/// 查询意图
#[derive(Debug, Clone)]
pub enum QueryIntent {
    /// 精确匹配
    ExactMatch,
    /// 概念查询
    Conceptual,
    /// 时序查询
    Temporal { start: DateTime, end: DateTime },
    /// 关系查询
    Relational { entity: String, relation: String },
    /// 列表查询
    List { category: String },
    /// 混合查询
    Mixed(Vec<QueryIntent>),
}

/// 意图分析器 (对标 OpenViking IntentAnalyzer)
pub struct IntentAnalyzer {
    /// LLM 提供商
    llm_provider: Arc<dyn LLMProvider>,
}

impl IntentAnalyzer {
    /// 分析查询意图
    pub async fn analyze(&self, query: &str) -> Result<QueryIntent> {
        // 1. 特征提取
        let features = self.extract_features(query);

        // 2. 模式匹配
        if let Some(intent) = self.match_patterns(&features) {
            return Ok(intent);
        }

        // 3. LLM 分类
        self.classify_with_llm(query).await
    }

    /// 生成查询计划
    pub fn plan(&self, intent: &QueryIntent) -> QueryPlan {
        match intent {
            QueryIntent::ExactMatch => QueryPlan::Direct,
            QueryIntent::Conceptual => QueryPlan::Semantic,
            QueryIntent::Temporal { .. } => QueryPlan::Filtered,
            QueryIntent::Relational { .. } => QueryPlan::Graph,
            QueryIntent::List { .. } => QueryPlan::Aggregate,
            QueryIntent::Mixed(intents) => QueryPlan::Composite,
        }
    }
}
```

---

## 五、实施计划

### 5.1 Timeline

```
Week 1: Phase 1 - EVIF 集成
├── 1.1 ContextFS 集成 (L0/L1/L2)
├── 1.2 SkillFS 集成 (SKILL.md)
├── 1.3 PipeFS 集成 (多Agent协调)
├── 1.4 VectorFS 集成 (向量检索)
└── 1.5 QueueFS 集成 (任务队列)

Week 2-3: Phase 2 - OpenViking 对标
├── 2.1 VikingFS 兼容层
├── 2.2 目录递归检索
├── 2.3 意图分析器
├── 2.4 可视化检索轨迹
└── 2.5 记忆压缩 V2

Week 4: Phase 3 - 功能增强
├── 3.1 AgentMem 搜索融合
├── 3.2 Feedback API
├── 3.3 实体链接
├── 3.4 中文分词
└── 3.5 编译警告清理

预计完成: 4 周
```

### 5.2 新增 Crate 结构

```
crates/
├── ...
└── agent-mem-evif (新增)
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        ├── viking_fs.rs          # VikingFS 兼容层
        ├── layered_context.rs      # 分层上下文
        ├── hierarchical_search.rs   # 目录递归检索
        ├── intent_analyzer.rs      # 意图分析器
        ├── retrieval_trace.rs      # 可视化轨迹
        ├── compressor_v2.rs        # 记忆压缩 V2
        └── integration/
            ├── mod.rs
            ├── test_contextfs.rs
            ├── test_skillfs.rs
            ├── test_pipefs.rs
            └── test_vikingfs.rs
```

---

## 六、验收标准

- [ ] EVIF ContextFS/SkillFS/PipeFS/VectorFS/QueueFS 集成完成
- [ ] L0/L1/L2 分层上下文正常工作
- [ ] SKILL.md 技能解析和执行
- [ ] 多Agent协调 (Claim/Wait/Broadcast)
- [ ] VikingFS 兼容层实现
- [ ] 目录递归检索实现
- [ ] 意图分析器实现
- [ ] 可视化检索轨迹
- [ ] AgentMem 5种搜索引擎融合
- [ ] 记忆压缩 V2
- [ ] 内部编译警告 < 100

---

## 七、代码质量指标

### 融合后

| 指标 | 数值 |
|------|------|
| AgentMem | ~137,000 行 |
| EVIF | ~120,000 行 |
| 新增 (agent-mem-evif) | ~20,000 行 |
| **融合后总计** | **~277,000 行** |
| Crate 数量 | 40+ |
| 编译错误 | 0 |
| 编译警告 | ~2,000 (外部为主) |
| 测试覆盖 | 600+ |

### 目标

| 指标 | 当前 | 目标 |
|------|------|------|
| 内部编译警告 | ~200 | < 100 |
| OpenViking 对齐 | 0% | > 80% |
| EVIF 集成 | 0% | 100% |

---

## 八、问题分析与修复计划

### 8.1 AgentMem 当前问题清单

#### 编译警告问题 (P1 - 中优先级)

| 文件 | 警告类型 | 数量 | 修复方案 |
|------|----------|------|----------|
| `agent-mem-llm/src/providers/*.rs` | dead_code (未使用字段) | ~15 | 添加 `#[allow(dead_code)]` 或删除 |
| `agent-mem-core/src/*.rs` | dead_code (未使用模块) | ~10 | 确认是否需要或添加 cfg |
| `agent-mem-storage/src/*.rs` | unused_variables | ~5 | 确认业务逻辑 |

#### HTTP 端点优化 (P2 - 低优先级)

| 当前端点 | 建议 | 理由 |
|----------|------|------|
| 65 端点 | 精简到 50 | 减少维护成本 |

#### 测试覆盖 (P2 - 低优先级)

| 模块 | 当前覆盖 | 目标 |
|------|----------|------|
| agent-mem-core | ~40% | > 70% |
| agent-mem-storage | ~50% | > 70% |
| agent-mem-server | ~30% | > 60% |

### 8.2 EVIF 当前问题清单

| 问题 | 影响 | 解决方案 |
|------|------|----------|
| Rust LS 不稳定 | 中 - 代码导航困难 | 使用简单文件读取 |
| 插件文档缺失 | 低 - 使用困难 | 补充文档 |

### 8.3 融合风险与缓解

| 风险 | 影响评估 | 缓解措施 |
|------|----------|----------|
| **命名冲突** | 高 | 统一使用 `/viking/memories` |
| **依赖冲突** | 中 | 使用 workspace.dependencies |
| **性能下降** | 中 | 性能基准测试 |
| **复杂度增加** | 高 | 保持模块独立性 |

---

## 九、详细实施路线图

### 9.1 Phase 1: 基础融合 (Week 1-2)

```
新增 Crate: agent-mem-vikingfs
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── mount/
    │   ├── mod.rs
    │   └── radix_tree.rs      # Radix Tree 实现
    ├── adapters/
    │   ├── mod.rs
    │   ├── contextfs.rs       # ContextFS 适配器
    │   ├── memoryfs.rs        # MemoryFS 适配器
    │   └── skillfs.rs         # SkillFS 适配器
    └── path/
        └── resolver.rs         # 路径解析
```

**关键文件变更**:

```toml
# 新增: crates/agent-mem-vikingfs/Cargo.toml
[package]
name = "agent-mem-vikingfs"
version = "0.1.0"
edition = "2021"

[dependencies]
agent-mem-core = { path = "../agent-mem-core" }
evif-core = { path = "../../evif/crates/evif-core" }
radix-trie = "0.2"
```

### 9.2 Phase 2: OpenViking 对标 (Week 3-4)

#### 目录递归检索

```rust
pub struct HierarchicalRetriever {
    storage: Arc<dyn MemoryStorage>,
    max_depth: usize,
    score_alpha: f32,
}

impl HierarchicalRetriever {
    pub async fn retrieve(
        &self,
        query: &str,
        root: Option<&str>,
        limit: usize,
    ) -> Result<Vec<SearchHit>> {
        // 1. 全局搜索
        let global = self.global_search(query).await?;
        // 2. 递归扩展
        let expanded = self.expand_recursive(global, root, 0).await?;
        // 3. 分数传播
        let propagated = self.propagate_scores(expanded);
        // 4. 重排序
        self.rerank(propagated, limit).await
    }
}
```

#### 意图分析器

```rust
pub enum QueryIntent {
    ExactMatch,
    Conceptual,
    Temporal { start: DateTime, end: DateTime },
    Relational { entity: String, relation: String },
    List { category: String },
    Mixed(Vec<QueryIntent>),
}

pub struct IntentAnalyzer {
    llm: Option<Arc<dyn LLM>>,
    rules: Vec<IntentRule>,
}
```

### 9.3 Phase 3: 功能增强 (Week 5-6)

#### 记忆压缩 V2

```rust
pub struct AdaptiveCompressor {
    strategy: CompressionStrategy,
    max_tokens: usize,
    preserve_entities: bool,
}

impl AdaptiveCompressor {
    pub async fn compress(
        &self,
        content: &str,
        context: &CompressionContext,
    ) -> Result<CompressedContent> {
        let importance = self.score_importance(content, context).await?;
        let strategy = self.select_strategy(importance);
        match strategy {
            CompressionStrategy::Keep => Ok(CompressedContent::Full(content.to_string())),
            CompressionStrategy::Truncate => self.truncate(content),
            CompressionStrategy::Summarize => self.summarize_llm(content).await,
            CompressionStrategy::Selective => self.selective_keep(content, importance).await,
        }
    }
}
```

#### 代码执行沙箱

```rust
pub struct CodeSandbox {
    wasm_runtime: Wasmtime,
    limits: ResourceLimits,
}

impl CodeSandbox {
    pub async fn execute(
        &self,
        code: &str,
        language: Language,
    ) -> Result<ExecutionResult> {
        match language {
            Language::Python => self.execute_python(code).await,
            Language::JavaScript => self.execute_js(code).await,
            Language::Rust => self.execute_wasm(code).await,
        }
    }
}
```

---

## 十、文件变更清单

### 10.1 新增文件

| 文件 | 说明 | 优先级 |
|------|------|--------|
| `crates/agent-mem-vikingfs/Cargo.toml` | 新 Crate | P0 |
| `crates/agent-mem-vikingfs/src/lib.rs` | 库入口 | P0 |
| `crates/agent-mem-vikingfs/src/mount/radix_tree.rs` | Radix Tree | P0 |
| `crates/agent-mem-vikingfs/src/adapters/contextfs.rs` | ContextFS 适配器 | P0 |
| `crates/agent-mem-vikingfs/src/adapters/memoryfs.rs` | MemoryFS 适配器 | P0 |
| `crates/agent-mem-vikingfs/src/intent.rs` | 意图分析器 | P1 |
| `crates/agent-mem-vikingfs/src/hierarchical.rs` | 目录递归检索 | P1 |
| `crates/agent-mem-vikingfs/src/compression_v2.rs` | 记忆压缩 V2 | P2 |
| `crates/agent-mem-vikingfs/src/sandbox.rs` | 代码沙箱 | P3 |

### 10.2 修改文件

| 文件 | 变更 | 优先级 |
|------|------|--------|
| `Cargo.toml` (workspace) | 添加 agent-mem-vikingfs | P0 |
| `crates/agent-mem-core/src/lib.rs` | 导出 VikingFS | P1 |
| `crates/agent-mem-server/src/routes/mod.rs` | 添加 VikingFS 路由 | P2 |
| `crates/agent-mem-mcp/src/lib.rs` | 集成 VikingFS MCP | P2 |

---

## 十一、测试计划

### 11.1 单元测试

```bash
# 测试 VikingFS 核心
cargo test -p agent-mem-vikingfs

# 测试适配器
cargo test -p agent-mem-vikingfs adapters

# 测试意图分析器
cargo test -p agent-mem-vikingfs intent
```

### 11.2 集成测试

```bash
# EVIF 集成测试
cargo test -p agent-mem-vikingfs evif_integration

# AgentMem 集成测试
cargo test -p agent-mem-vikingfs agentmem_integration
```

### 11.3 性能测试

```bash
# 路径解析性能
cargo bench -p agent-mem-vikingfs path_resolution

# 检索性能
cargo bench -p agent-mem-vikingfs retrieval
```

---

*文档版本: 2.0*
*最后更新: 2026-05-21*
*更新内容: 添加详细实施路线图、问题分析、文件变更清单、测试计划*
