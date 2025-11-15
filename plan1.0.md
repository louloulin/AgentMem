# AgentMem V4 架构完善计划 - 基于Mem0功能复用

## 1. 执行摘要

本文档全面分析了 AgentMem V4 架构和 Mem0 的核心功能，制定了基于现有代码最小改动的实现计划，**删除 SimpleMemory，仅保留 V4 架构**，通过功能复用完善整个记忆平台。

**关键决策：**
- ✅ **删除 SimpleMemory**：统一使用 Memory V4 架构
- ✅ **保留 V4 架构**：多模态内容、开放属性系统、关系图谱、强类型查询
- ✅ **复用 Mem0 功能**：元数据过滤、重排序、图记忆等
- ✅ **保持高内聚低耦合**：模块化设计，清晰职责分离

**理论基础：**
- 认知心理学：Episodic, Semantic, Procedural 记忆模型
- 向量数据库：语义检索和近似最近邻搜索
- 知识图谱：关系推理和图遍历
- 稀疏分布式存储器（Sparse Distributed Memory）

---

## 2. Memory V4 架构深度分析

### 2.1 V4 架构核心设计

**Memory V4 结构：**
```rust
pub struct Memory {
    pub id: MemoryId,              // 唯一标识符
    pub content: Content,           // 多模态内容
    pub attributes: AttributeSet,   // 开放属性系统（命名空间）
    pub relations: RelationGraph,   // 关系图谱
    pub metadata: Metadata,         // 系统元数据
}
```

**核心优势：**
1. **多模态内容支持**：Text, Structured, Vector, Binary, Multimodal
2. **开放属性系统**：命名空间隔离（core, system, user, agent, domain）
3. **关系图谱**：内置关系管理，支持复杂记忆网络
4. **强类型查询**：Query V4 支持语义化查询

### 2.2 V4 架构与存储层集成

**转换层：**
- `storage/conversion.rs`: Memory V4 ↔ DbMemory 转换
- `v4_migration.rs`: Legacy MemoryItem ↔ Memory V4 迁移

**存储流程：**
```rust
Memory V4 → memory_to_db() → DbMemory → PostgreSQL/LibSQL
DbMemory → db_to_memory() → Memory V4
```

**关键代码位置：**
- `crates/agent-mem-core/src/storage/conversion.rs:15-185`
- `crates/agent-mem-core/src/v4_migration.rs:42-122`

### 2.3 V4 架构与搜索层集成

**查询抽象：**
```rust
pub struct Query {
    pub intent: QueryIntent,        // NaturalLanguage, Structured, Vector, Hybrid
    pub constraints: Vec<Constraint>,  // Attribute, Relation, Temporal, Spatial
    pub preferences: Vec<Preference>, // Temporal, Relevance, Diversity
    pub context: QueryContext,
}
```

**检索引擎：**
- `RetrievalEngine` trait：可组合的检索引擎模式
- `RetrievalResult`：包含解释和评分分解
- `ScoredMemory`：评分记忆结果

**关键代码位置：**
- `crates/agent-mem-traits/src/abstractions.rs:300-650`
- `crates/agent-mem-core/src/search/mod.rs`

---

## 3. Mem0 核心功能分析

### 3.1 Mem0 存储实现

**核心类：`Memory` (main.py:172-2326行)**

**存储流程：**
```python
Memory.add(messages, user_id, agent_id, run_id, infer=True)
├── _build_filters_and_metadata()  # 构建过滤器和元数据
├── 并行执行：
│   ├── _add_to_vector_store()  # 向量存储
│   │   ├── infer=True → 智能提取
│   │   │   ├── get_fact_retrieval_messages()  # 构建提示词
│   │   │   ├── llm.generate()  # LLM提取事实
│   │   │   ├── 查找相似记忆
│   │   │   ├── 决策（ADD/UPDATE/DELETE）
│   │   │   └── 执行操作
│   │   └── infer=False → 直接存储
│   └── _add_to_graph()  # 图存储（可选）
└── 返回结果
```

**关键特性：**
- ✅ 支持多种向量数据库（Pinecone, Qdrant, Chroma, FAISS等）
- ✅ SQLite历史记录存储
- ✅ 可选图数据库（Memgraph, Kuzu）
- ✅ 智能事实提取（LLM驱动）

### 3.2 Mem0 检索实现

**搜索流程：**
```python
Memory.search(query, user_id, agent_id, run_id, limit=100, threshold=None, rerank=True)
├── _build_filters_and_metadata()  # 构建过滤器
├── 并行执行：
│   ├── _search_vector_store()  # 向量搜索
│   │   ├── embedding_model.embed(query)  # 生成查询向量
│   │   ├── vector_store.search(query, vectors, limit, filters)  # 向量搜索
│   │   └── 应用threshold过滤
│   └── graph.search()  # 图搜索（可选）
├── reranker.rerank()  # 重排序（可选）
└── 返回结果
```

**关键特性：**
- ✅ 高级元数据过滤（eq, ne, gt, in, contains, AND, OR, NOT）
- ✅ 可选重排序器（RerankerFactory）
- ✅ 图关系搜索（可选）
- ✅ 阈值过滤

### 3.3 Mem0 元数据过滤系统

**高级操作符：**
```python
# 比较操作符
{"key": {"eq": "value"}}    # 等于
{"key": {"ne": "value"}}    # 不等于
{"key": {"gt": 10}}         # 大于
{"key": {"gte": 10}}        # 大于等于
{"key": {"lt": 10}}         # 小于
{"key": {"lte": 10}}        # 小于等于

# 集合操作符
{"key": {"in": ["val1", "val2"]}}      # 在列表中
{"key": {"nin": ["val1", "val2"]}}     # 不在列表中

# 字符串操作符
{"key": {"contains": "text"}}          # 包含文本
{"key": {"icontains": "text"}}         # 不区分大小写包含

# 逻辑操作符
{"AND": [filter1, filter2]}            # 逻辑与
{"OR": [filter1, filter2]}             # 逻辑或
{"NOT": [filter1]}                      # 逻辑非
```

**关键代码位置：**
- `source/mem0/mem0/memory/main.py:858-952` (`_process_metadata_filters`)
- `source/mem0/mem0/memory/main.py:927-952` (`_has_advanced_operators`)

---

## 4. 功能对比与差距分析

### 4.1 存储功能对比

| 功能 | AgentMem V4 | Mem0 | 差距 | 优先级 |
|------|-------------|------|------|--------|
| **基础存储** | ✅ LibSQL/PostgreSQL | ✅ Vector Store + SQLite | 无 | - |
| **V4架构支持** | ✅ 完整实现 | ❌ 无 | AgentMem更优 | - |
| **多模态内容** | ✅ Text/Structured/Vector/Binary | ⚠️ 基础支持 | AgentMem更优 | - |
| **关系图谱** | ✅ RelationGraph | ✅ Graph Store | 无 | - |
| **批量操作** | ✅ batch_create/delete | ❌ 不支持 | AgentMem更优 | - |
| **事务支持** | ✅ PostgreSQL事务 | ❌ 无 | AgentMem更优 | - |
| **历史记录** | ✅ MemoryHistory | ✅ SQLiteManager | 无 | - |

### 4.2 检索功能对比

| 功能 | AgentMem V4 | Mem0 | 差距 | 优先级 |
|------|-------------|------|------|--------|
| **向量搜索** | ✅ VectorSearchEngine | ✅ vector_store.search() | 无 | - |
| **BM25搜索** | ✅ 完整实现（315行） | ⚠️ 基础支持 | AgentMem更优 | - |
| **全文搜索** | ✅ PostgreSQL全文搜索 | ❌ 不支持 | AgentMem更优 | - |
| **模糊匹配** | ✅ FuzzyMatchEngine | ❌ 不支持 | AgentMem更优 | - |
| **混合搜索** | ✅ HybridSearchEngine + RRF | ⚠️ 基础实现 | AgentMem更优 | - |
| **元数据过滤** | ⚠️ 基础过滤 | ✅ 高级操作符 | **Mem0更优** | **P0** |
| **重排序** | ⚠️ 部分支持 | ✅ RerankerFactory | **Mem0更优** | **P1** |
| **Query V4** | ✅ 强类型查询 | ❌ 无 | AgentMem更优 | - |

### 4.3 API设计对比

| 特性 | AgentMem V4 | Mem0 | 差距 | 优先级 |
|------|-------------|------|------|--------|
| **API简洁性** | ⚠️ 需要理解V4架构 | ✅ 简洁直观 | Mem0更优 | - |
| **默认行为** | ⚠️ 需要显式配置 | ✅ 智能默认 | Mem0更优 | - |
| **infer参数** | ✅ 支持 | ✅ 默认True | 无 | - |
| **错误处理** | ✅ 强类型错误 | ⚠️ 异常处理 | AgentMem更优 | - |
| **异步支持** | ✅ 原生async | ⚠️ 部分async | AgentMem更优 | - |
| **类型安全** | ✅ Rust强类型 | ⚠️ Python动态类型 | AgentMem更优 | - |

---

## 5. 代码复用分析

### 5.1 可直接复用的代码

#### 5.1.1 存储层（100%复用）
- ✅ **`storage/memory_repository.rs`**: PostgreSQL存储实现完整
- ✅ **`storage/libsql/memory_repository.rs`**: LibSQL存储实现完整
- ✅ **`storage/conversion.rs`**: V4 ↔ DbMemory 转换完整
- ✅ **`storage/postgres.rs`**: PostgreSQL后端抽象完整

#### 5.1.2 搜索层（100%复用）
- ✅ **`search/vector_search.rs`**: 向量搜索实现完整
- ✅ **`search/bm25.rs`**: BM25搜索实现完整（315行）
- ✅ **`search/hybrid.rs`**: 混合搜索实现完整
- ✅ **`search/fulltext_search.rs`**: 全文搜索实现完整
- ✅ **`search/fuzzy.rs`**: 模糊匹配实现完整

#### 5.1.3 V4架构核心（100%复用）
- ✅ **`agent-mem-traits/src/abstractions.rs`**: V4抽象定义完整
- ✅ **`v4_migration.rs`**: 迁移工具完整
- ✅ **`storage/conversion.rs`**: 转换层完整

### 5.2 需要增强的代码

#### 5.2.1 元数据过滤系统（P0优先级）

**当前状态：**
- 只支持基础过滤（user_id, agent_id等）
- 缺少高级操作符

**需要实现：**
```rust
// 新建文件
crates/agent-mem-core/src/search/metadata_filter.rs
├── MetadataFilter 结构
├── FilterOperator 枚举（eq, ne, gt, in, contains等）
├── LogicalOperator 枚举（AND, OR, NOT）
└── 过滤逻辑实现

// 修改文件
crates/agent-mem-core/src/search/mod.rs
├── 集成MetadataFilter到SearchQuery
└── 更新SearchFilters结构

// 修改存储层
crates/agent-mem-core/src/storage/memory_repository.rs
├── 实现高级过滤逻辑
└── 支持操作符查询
```

**参考实现：**
- Mem0: `source/mem0/mem0/memory/main.py:858-952`

#### 5.2.2 重排序器集成（P1优先级）

**当前状态：**
- 无独立的重排序器模块
- 搜索中有部分重排序逻辑

**需要实现：**
```rust
// 新建文件
crates/agent-mem-core/src/search/reranker.rs
├── Reranker trait
├── CohereReranker
├── JinaReranker（可选）
└── 集成到搜索流程

// 修改文件
crates/agent-mem-core/src/manager.rs
├── 添加reranker字段
└── 在search_memories()中集成重排序

// 修改文件
crates/agent-mem/src/memory.rs
├── 添加with_reranker()方法
```

**参考实现：**
- Mem0: `mem0/reranker/` 目录

#### 5.2.3 Memory API简化（P2优先级）

**当前状态：**
- `Memory::new()` 已实现零配置
- 但需要理解V4架构

**需要优化：**
```rust
// 优化 Memory::new()
Memory::new()
├── 自动检测环境变量
├── 默认启用智能功能（infer=true）
├── 默认使用持久化存储（LibSQL）
└── 简化配置流程

// 添加便捷方法
Memory::add_text(content)  // 文本内容
Memory::add_structured(data)  // 结构化数据
Memory::add_multimodal(contents)  // 多模态内容
```

### 5.3 需要删除的代码

#### 5.3.1 SimpleMemory模块（P0优先级）

**删除清单：**
- [ ] `crates/agent-mem-core/src/simple_memory.rs` (546行)
- [ ] `crates/agent-mem-core/src/lib.rs` 中的 `simple_memory` 导出
- [ ] 所有使用 `SimpleMemory` 的示例代码
- [ ] 文档中关于 `SimpleMemory` 的引用

**迁移路径：**
```rust
// 旧代码（删除）
use agent_mem_core::SimpleMemory;
let mem = SimpleMemory::new().await?;

// 新代码（V4架构）
use agent_mem::Memory;
let mem = Memory::new().await?;
```

**影响范围：**
- 71个文件引用 `SimpleMemory` 或 `simple_memory`
- 需要迁移所有示例代码到 V4 架构

#### 5.3.2 Legacy MemoryItem（可选，P3优先级）

**保留原因：**
- 向后兼容性
- 迁移工具需要

**处理方式：**
- 标记为 `deprecated`
- 保留迁移工具
- 新代码强制使用 V4

---

## 6. Mem0功能复用计划

### 6.1 元数据过滤系统复用（P0）

**Mem0实现分析：**
- `_process_metadata_filters()`: 处理高级操作符
- `_has_advanced_operators()`: 检测高级操作符
- 支持 eq, ne, gt, in, contains, AND, OR, NOT

**AgentMem实现计划：**
```rust
// 1. 新建 metadata_filter.rs
pub struct MetadataFilter {
    pub field: String,
    pub operator: FilterOperator,
    pub value: FilterValue,
}

pub enum FilterOperator {
    Eq, Ne, Gt, Gte, Lt, Lte,
    In, Nin,
    Contains, IContains,
}

pub enum LogicalOperator {
    And(Vec<MetadataFilter>),
    Or(Vec<MetadataFilter>),
    Not(Box<MetadataFilter>),
}

// 2. 集成到 SearchQuery
pub struct SearchQuery {
    // ... existing fields
    pub metadata_filters: Option<LogicalOperator>,
}

// 3. 实现过滤逻辑
impl MetadataFilter {
    pub fn matches(&self, memory: &Memory) -> bool {
        // 实现过滤逻辑
    }
}
```

**预计工作量：** 3-4天

### 6.2 重排序器复用（P1）

**Mem0实现分析：**
- `RerankerFactory`: 工厂模式创建重排序器
- 支持 Cohere, Jina 等
- `reranker.rerank(query, memories, limit)`

**AgentMem实现计划：**
```rust
// 1. 新建 reranker.rs
#[async_trait]
pub trait Reranker: Send + Sync {
    async fn rerank(
        &self,
        query: &str,
        memories: Vec<ScoredMemory>,
        limit: usize,
    ) -> Result<Vec<ScoredMemory>>;
}

pub struct CohereReranker {
    api_key: String,
    model: String,
}

pub struct JinaReranker {
    api_key: String,
}

// 2. 集成到 MemoryManager
pub struct MemoryManager {
    // ... existing fields
    reranker: Option<Arc<dyn Reranker>>,
}

// 3. 在搜索流程中集成
impl MemoryManager {
    pub async fn search_memories(&self, query: Query) -> Result<Vec<ScoredMemory>> {
        let results = self.vector_search(query).await?;
        
        if let Some(reranker) = &self.reranker {
            reranker.rerank(&query.text, results, query.limit).await
        } else {
            Ok(results)
        }
    }
}
```

**预计工作量：** 2-3天

### 6.3 图记忆功能复用（P2）

**Mem0实现分析：**
- `graph_memory.py`: 图记忆抽象
- `memgraph_memory.py`: Memgraph实现
- `kuzu_memory.py`: Kuzu实现
- 支持 `add()`, `search()`, `delete_all()`, `get_all()`

**AgentMem现状：**
- `graph_memory.rs`: 已有基础实现
- 需要完善和集成

**实现计划：**
```rust
// 1. 完善 graph_memory.rs
pub struct GraphMemory {
    // ... existing fields
}

impl GraphMemory {
    pub async fn add(&self, memory: &Memory, filters: &Filters) -> Result<()>;
    pub async fn search(&self, query: &str, filters: &Filters, limit: usize) -> Result<Vec<Memory>>;
    pub async fn delete_all(&self, filters: &Filters) -> Result<()>;
    pub async fn get_all(&self, filters: &Filters, limit: usize) -> Result<Vec<Memory>>;
}

// 2. 集成到 MemoryManager
pub struct MemoryManager {
    // ... existing fields
    graph_memory: Option<Arc<GraphMemory>>,
}

// 3. 在add/search中集成
impl MemoryManager {
    pub async fn add_memory(&self, memory: Memory) -> Result<String> {
        // ... existing logic
        
        if let Some(graph) = &self.graph_memory {
            graph.add(&memory, &filters).await?;
        }
        
        Ok(memory_id)
    }
}
```

**预计工作量：** 3-5天

---

## 7. 最小改动实现计划

### 7.1 阶段1：删除SimpleMemory，统一V4架构（P0）

**目标：** 删除SimpleMemory，统一使用Memory V4架构

**任务清单：**

1. **删除SimpleMemory代码**
   - [ ] 删除 `crates/agent-mem-core/src/simple_memory.rs`
   - [ ] 从 `crates/agent-mem-core/src/lib.rs` 移除导出
   - [ ] 更新所有文档引用

2. **迁移示例代码**
   - [ ] 迁移 `examples/simple-memory-demo/` 到 V4
   - [ ] 迁移 `examples/simple-api-test/` 到 V4
   - [ ] 更新所有示例使用 `Memory::new()`

3. **优化Memory API**
   - [ ] 确保 `Memory::new()` 零配置可用
   - [ ] 添加便捷方法（`add_text`, `add_structured`等）
   - [ ] 完善文档和示例

**预计工作量：** 2-3天

### 7.2 阶段2：元数据过滤系统增强（P0）

**目标：** 实现Mem0级别的高级元数据过滤

**任务清单：**

1. **新建metadata_filter.rs**
   - [ ] 实现 `MetadataFilter` 结构
   - [ ] 实现 `FilterOperator` 枚举
   - [ ] 实现 `LogicalOperator` 枚举
   - [ ] 实现过滤逻辑

2. **集成到搜索系统**
   - [ ] 修改 `SearchQuery` 结构
   - [ ] 修改 `SearchFilters` 结构
   - [ ] 更新搜索流程

3. **存储层支持**
   - [ ] 在 `memory_repository.rs` 中实现过滤
   - [ ] 在 `libsql/memory_repository.rs` 中实现过滤
   - [ ] 支持SQL查询生成

4. **测试和文档**
   - [ ] 编写单元测试
   - [ ] 编写集成测试
   - [ ] 更新API文档

**预计工作量：** 3-4天

### 7.3 阶段3：重排序器集成（P1）

**目标：** 添加可选的重排序功能

**任务清单：**

1. **新建reranker.rs**
   - [ ] 定义 `Reranker` trait
   - [ ] 实现 `CohereReranker`
   - [ ] 实现 `JinaReranker`（可选）

2. **集成到MemoryManager**
   - [ ] 添加 `reranker` 字段
   - [ ] 在 `search_memories()` 中集成
   - [ ] 支持配置启用/禁用

3. **集成到Memory API**
   - [ ] 添加 `with_reranker()` 方法
   - [ ] 支持Builder模式配置

4. **测试和文档**
   - [ ] 编写单元测试
   - [ ] 编写集成测试
   - [ ] 更新API文档

**预计工作量：** 2-3天

### 7.4 阶段4：图记忆完善（P2）

**目标：** 完善图记忆功能，对标Mem0

**任务清单：**

1. **完善graph_memory.rs**
   - [ ] 实现完整的图操作API
   - [ ] 支持图搜索
   - [ ] 支持关系查询

2. **集成到MemoryManager**
   - [ ] 添加 `graph_memory` 字段
   - [ ] 在 `add_memory()` 中集成
   - [ ] 在 `search_memories()` 中集成

3. **测试和文档**
   - [ ] 编写单元测试
   - [ ] 编写集成测试
   - [ ] 更新API文档

**预计工作量：** 3-5天

### 7.5 阶段5：代码清理和优化（P2）

**目标：** 删除不需要的代码，保持高内聚低耦合

**任务清单：**

1. **代码审查**
   - [ ] 标记未使用的模块
   - [ ] 标记重复实现
   - [ ] 标记过时代码

2. **删除冗余代码**
   - [ ] 删除未使用的agent实现
   - [ ] 合并重复的存储实现
   - [ ] 统一搜索接口

3. **优化代码结构**
   - [ ] 确保高内聚低耦合
   - [ ] 优化模块依赖
   - [ ] 更新文档

**预计工作量：** 2-3天

---

## 8. 实施优先级和时间表

### 8.1 优先级排序

1. **P0（必须）**: 阶段1 - 删除SimpleMemory，统一V4架构
2. **P0（必须）**: 阶段2 - 元数据过滤系统增强
3. **P1（重要）**: 阶段3 - 重排序器集成
4. **P2（可选）**: 阶段5 - 代码清理和优化
5. **P2（可选）**: 阶段4 - 图记忆完善

### 8.2 时间表

| 阶段 | 优先级 | 预计时间 | 开始时间 | 完成时间 |
|------|--------|----------|----------|----------|
| 阶段1 | P0 | 2-3天 | Day 1 | Day 3 |
| 阶段2 | P0 | 3-4天 | Day 4 | Day 7 |
| 阶段3 | P1 | 2-3天 | Day 8 | Day 10 |
| 阶段5 | P2 | 2-3天 | Day 11 | Day 13 |
| 阶段4 | P2 | 3-5天 | 未来 | 未来 |

**总计：** 9-13天（P0+P1+P2）

---

## 9. 理论基础和参考文献

### 9.1 认知心理学理论

**记忆模型：**
- **Episodic Memory（情景记忆）**: 特定时间和地点的事件记忆
- **Semantic Memory（语义记忆）**: 概念和事实知识
- **Procedural Memory（程序记忆）**: 技能和程序性知识
- **Working Memory（工作记忆）**: 短期信息保持和处理

**参考论文：**
1. Atkinson, R. C., & Shiffrin, R. M. (1968). "Human memory: A proposed system and its control processes"
2. Tulving, E. (1972). "Episodic and semantic memory"
3. Baddeley, A. D. (2000). "The episodic buffer: a new component of working memory?"

### 9.2 向量数据库和语义检索

**技术原理：**
- **向量嵌入（Embedding）**: 将文本转换为高维向量
- **近似最近邻搜索（ANN）**: 快速查找相似向量
- **余弦相似度（Cosine Similarity）**: 衡量向量相似性

**参考论文：**
1. Mikolov, T., et al. (2013). "Efficient estimation of word representations in vector space"
2. Johnson, J., et al. (2019). "Billion-scale similarity search with GPUs"
3. Douze, M., et al. (2024). "The Faiss library"

### 9.3 知识图谱和关系推理

**技术原理：**
- **图数据库**: 存储实体和关系
- **图遍历算法**: 查找路径和关系
- **关系推理**: 基于图结构进行推理

**参考论文：**
1. Auer, S., et al. (2007). "DBpedia: A nucleus for a web of open data"
2. Hogan, A., et al. (2021). "Knowledge graphs"
3. Hamilton, W. L. (2020). "Graph representation learning"

### 9.4 稀疏分布式存储器

**技术原理：**
- **稀疏编码**: 使用稀疏向量表示记忆
- **分布式存储**: 记忆分布在多个位置
- **内容寻址**: 基于内容而非地址检索

**参考论文：**
1. Kanerva, P. (1988). "Sparse distributed memory"
2. Gallant, S. I., & Okaywe, T. W. (2013). "Representing objects, relations, and sequences"

---

## 10. 技术债务和风险

### 10.1 技术债务

1. **API不一致**
   - SimpleMemory和Memory API不一致
   - **解决方案**: 删除SimpleMemory，统一V4架构

2. **配置复杂**
   - 多层配置系统
   - **解决方案**: 简化配置，自动检测环境变量

3. **文档不足**
   - V4架构文档不完整
   - **解决方案**: 补充完整文档和使用示例

### 10.2 风险

1. **向后兼容性**
   - 删除SimpleMemory可能破坏现有代码
   - **缓解措施**: 提供迁移指南和工具

2. **性能影响**
   - 高级过滤可能影响性能
   - **缓解措施**: 性能测试和优化

3. **测试覆盖**
   - 新功能需要完整测试
   - **缓解措施**: 确保测试覆盖率 > 80%

---

## 11. 成功标准

### 11.1 功能完整性

- [ ] SimpleMemory已删除，统一使用V4架构
- [ ] 元数据过滤支持所有Mem0操作符
- [ ] 重排序器集成完成
- [ ] 图记忆功能完善（可选）

### 11.2 代码质量

- [ ] 所有新代码通过clippy检查
- [ ] 测试覆盖率 > 80%
- [ ] 文档完整，包含使用示例
- [ ] 高内聚低耦合，模块职责清晰

### 11.3 性能指标

- [ ] 搜索延迟 < 100ms（P95）
- [ ] 存储延迟 < 50ms（P95）
- [ ] 内存占用 < 2GB（idle）

---

## 12. 附录

### 12.1 关键文件清单

**AgentMem V4核心文件：**
- `crates/agent-mem-traits/src/abstractions.rs` - V4抽象定义
- `crates/agent-mem-core/src/storage/conversion.rs` - V4 ↔ DbMemory转换
- `crates/agent-mem-core/src/v4_migration.rs` - 迁移工具
- `crates/agent-mem/src/memory.rs` - Memory API
- `crates/agent-mem-core/src/manager.rs` - 核心管理器

**Mem0核心文件：**
- `source/mem0/mem0/memory/main.py` - 核心Memory类
- `source/mem0/mem0/memory/storage.py` - SQLite存储
- `source/mem0/mem0/reranker/` - 重排序器

### 12.2 需要删除的文件

- `crates/agent-mem-core/src/simple_memory.rs` (546行)
- 所有使用SimpleMemory的示例代码

### 12.3 需要新建的文件

- `crates/agent-mem-core/src/search/metadata_filter.rs` - 元数据过滤
- `crates/agent-mem-core/src/search/reranker.rs` - 重排序器

---

**文档版本：** 2.0  
**创建日期：** 2024-12-19  
**最后更新：** 2024-12-19  
**状态：** 待实施
