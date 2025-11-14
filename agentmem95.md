# AgentMem 核心改造计划 v95

## 整体架构图

### 1. 当前架构（顺序执行，300ms）

```
用户请求
  ↓
Orchestrator（协调层）
  ↓
[顺序执行 - 瓶颈]
  ├─ Step 1: extract_facts (LLM调用 50ms)
  ├─ Step 2-3: extract_structured_facts (LLM调用 50ms)
  ├─ Step 4: evaluate_importance (LLM调用 50ms)
  ├─ Step 5: search_similar (数据库 20ms)
  ├─ Step 6: detect_conflicts (CPU 30ms)
  ├─ Step 7: make_decisions (LLM调用 50ms)
  └─ Step 8: execute_decisions (数据库 50ms)
  ↓
直接调用Managers（绕过Agent层）
  ├─ EpisodicMemoryManager
  ├─ SemanticMemoryManager
  └─ CoreMemoryManager
  ↓
存储层（LibSQL + LanceDB）
```

**问题**:
- ❌ 4次LLM调用顺序执行（200ms，占67%）
- ❌ Agent层完全未使用
- ❌ 数据库操作顺序执行（70ms）

---

### 2. 优化后架构（并行执行，支持智能/非智能模式）

```
用户请求
  ↓
Orchestrator（协调层）
  ↓
[模式选择]
  ├─ 快速模式 (infer=False)
  │   ├─ 直接生成嵌入 (5ms)
  │   └─ 并行Agent执行 (20ms)
  │       ├─ EpisodicAgent (数据库操作)
  │       ├─ SemanticAgent (数据库操作)
  │       └─ CoreAgent (数据库操作)
  │   → 总延迟: 25ms, 吞吐量: 10,000+ ops/s
  │
  ├─ 智能模式 (infer=True)
  │   ├─ 并行LLM调用 (50ms)
  │   │   ├─ extract_facts
  │   │   ├─ extract_structured_facts
  │   │   └─ evaluate_importance
  │   ├─ 并行数据库查询 (20ms)
  │   │   └─ search_similar
  │   ├─ CPU计算 (30ms)
  │   │   └─ detect_conflicts
  │   ├─ LLM决策 (50ms)
  │   │   └─ make_decisions
  │   └─ 并行Agent执行 (20ms)
  │       ├─ EpisodicAgent
  │       ├─ SemanticAgent
  │       └─ CoreAgent
  │   → 总延迟: 170ms, 吞吐量: 1,000 ops/s
  │
  └─ 批量模式 (batch)
      ├─ 批量LLM调用 (50ms/100条)
      └─ 批量Agent执行 (20ms/100条)
      → 平均延迟: 0.7ms/条, 吞吐量: 5,000 ops/s
  ↓
Agent层（数据处理，无LLM调用）
  ├─ EpisodicAgent → EpisodicMemoryManager
  ├─ SemanticAgent → SemanticMemoryManager
  ├─ ProceduralAgent → ProceduralMemoryManager
  ├─ WorkingAgent → WorkingMemoryManager
  ├─ CoreAgent → CoreMemoryManager
  ├─ ResourceAgent → ResourceMemoryManager
  ├─ KnowledgeAgent → KnowledgeMemoryManager
  └─ ContextualAgent → ContextualMemoryManager
  ↓
存储层（LibSQL + LanceDB）
```

**优化点**:
- ✅ LLM调用并行执行（200ms → 50ms，4x提升）
- ✅ Agent层并行执行（70ms → 20ms，3.5x提升）
- ✅ 支持快速模式（无LLM，10,000+ ops/s）
- ✅ 支持批量模式（批量LLM，5,000 ops/s）

---

### 3. 核心能力集成架构

```
Orchestrator（协调层）
  ↓
[智能模式专用能力]
  ├─ LLM调用层
  │   ├─ FactExtractor（事实提取）
  │   ├─ AdvancedFactExtractor（结构化提取）
  │   ├─ ImportanceEvaluator（重要性评估）
  │   └─ DecisionEngine（智能决策）
  │
  ├─ 图推理引擎（GraphMemoryEngine）
  │   ├─ 5种推理类型（演绎、归纳、溯因、类比、因果）
  │   ├─ 6种图算法（BFS、DFS、Dijkstra、社区检测、PageRank等）
  │   └─ 5种节点类型（Entity、Concept、Event、Relation、Context）
  │
  ├─ 高级推理引擎（AdvancedReasoner）
  │   ├─ 多跳因果推理
  │   ├─ 反事实推理
  │   ├─ 类比推理
  │   └─ 时序推理
  │
  ├─ 增强搜索引擎（EnhancedHybridSearchEngine）
  │   ├─ 查询分类（QueryClassifier）
  │   ├─ 自适应阈值（AdaptiveThresholdCalculator）
  │   ├─ 学习重排序（LearnedReranker）
  │   └─ 混合搜索（Vector + BM25 + Exact）
  │
  ├─ 聚类分析引擎（ClusteringEngine）
  │   ├─ DBSCAN聚类
  │   ├─ K-Means聚类
  │   └─ 层次聚类
  │
  └─ 批量处理引擎
      ├─ BatchEntityExtractor（批量实体提取）
      └─ BatchImportanceEvaluator（批量重要性评估）
  ↓
Agent层（数据处理）
  ↓
存储层
```

**集成策略**:
- ✅ 快速模式：不使用任何智能能力
- ✅ 智能模式：使用LLM调用层 + 增强搜索
- ✅ 高级模式：额外启用图推理 + 高级推理 + 聚类分析
- ✅ 批量模式：使用批量处理引擎

---

## TODO List

### Phase 0: 基准测试（1天）

**目标**: 建立性能基准，验证当前性能

- [/] **Task 0.1**: 运行压测获取基准数据（进行中 - 编译压测工具）
  ```bash
  cd tools/comprehensive-stress-test
  cargo build --release
  cargo run --release -- memory-creation --concurrency 100 --total 10000 --real true
  cargo run --release -- memory-retrieval --dataset-size 10000 --concurrency 50 --real true
  cargo run --release -- batch-operations --batch-size 100 --real true
  ```
  - 记录当前吞吐量：577 ops/s
  - 记录当前延迟：P95 24ms
  - 记录CPU利用率：15.76%

- [ ] **Task 0.2**: 保存基准结果
  ```bash
  mkdir -p stress-test-results/baseline
  cp stress-test-results/*.json stress-test-results/baseline/
  ```

---

### Phase 1: 实现快速模式（2天）

**目标**: 实现infer=False模式，达到10,000+ ops/s

- [ ] **Task 1.1**: 修改Orchestrator添加快速模式
  - 文件：`crates/agent-mem/src/orchestrator.rs`
  - 修改：`add_memory_v2`方法
  - 实现：
    ```rust
    pub async fn add_memory_v2(&self, content: String, infer: bool, ...) -> Result<AddResult> {
        if infer {
            // 智能模式（现有实现）
            self.add_memory_intelligent(content, ...).await
        } else {
            // 快速模式（新实现）
            self.add_memory_fast(content, ...).await
        }
    }
    
    async fn add_memory_fast(&self, content: String, ...) -> Result<AddResult> {
        // 1. 直接生成嵌入（无LLM调用）
        let embedding = self.embedder.embed(&content).await?;
        
        // 2. 直接插入数据库（无智能处理）
        let memory_id = uuid::Uuid::new_v4().to_string();
        
        // 3. 并行写入LibSQL和LanceDB
        tokio::try_join!(
            self.libsql_store.insert(memory_id, content, embedding),
            self.vector_store.add_vector(memory_id, embedding)
        )?;
        
        Ok(AddResult { id: memory_id, ... })
    }
    ```

- [ ] **Task 1.2**: 实现批量嵌入生成
  - 文件：`crates/agent-mem-embeddings/src/lib.rs`
  - 添加：`embed_batch`方法
  - 实现：
    ```rust
    pub async fn embed_batch(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        // 批量生成嵌入（FastEmbed支持批量）
        self.embedder.embed_batch(texts).await
    }
    ```

- [ ] **Task 1.3**: 压测验证快速模式
  ```bash
  # 测试快速模式性能
  cargo run --release -- memory-creation --concurrency 100 --total 10000 --infer false
  ```
  - 目标吞吐量：10,000+ ops/s
  - 目标延迟：P95 < 5ms

---

### Phase 2: 优化智能模式LLM调用（3天）

**目标**: 并行LLM调用，延迟从300ms降到170ms

- [ ] **Task 2.1**: 实现并行LLM调用
  - 文件：`crates/agent-mem/src/orchestrator.rs`
  - 修改：`add_memory_intelligent`方法
  - 实现：
    ```rust
    pub async fn add_memory_intelligent(&self, content: String, ...) -> Result<AddResult> {
        // Step 1-4: 并行LLM调用（原来顺序执行150ms → 现在并行50ms）
        let (facts, structured, importance) = tokio::try_join!(
            self.extract_facts(&content),
            self.extract_structured_facts(&content),
            self.evaluate_importance_preliminary(&content)
        )?;
        
        // Step 5: 搜索相似记忆（20ms）
        let existing = self.search_similar_memories(&facts).await?;
        
        // Step 6: 冲突检测（30ms）
        let conflicts = self.detect_conflicts(&facts, &existing).await?;
        
        // Step 7: 智能决策（50ms）
        let decisions = self.make_intelligent_decisions(&facts, &conflicts).await?;
        
        // Step 8: 并行执行决策（原来50ms → 现在20ms）
        self.execute_decisions_parallel(decisions).await?;
        
        Ok(AddResult { ... })
    }
    ```

- [ ] **Task 2.2**: 实现LLM结果缓存
  - 文件：`crates/agent-mem-llm/src/cache.rs`（新建）
  - 实现：
    ```rust
    pub struct LLMCache {
        cache: Arc<RwLock<HashMap<String, CachedResult>>>,
        ttl: Duration,
    }
    
    impl LLMCache {
        pub async fn get_or_compute<F, T>(&self, key: &str, compute: F) -> Result<T>
        where
            F: Future<Output = Result<T>>,
        {
            // 检查缓存
            if let Some(cached) = self.cache.read().await.get(key) {
                if !cached.is_expired() {
                    return Ok(cached.value.clone());
                }
            }
            
            // 计算并缓存
            let value = compute.await?;
            self.cache.write().await.insert(key.to_string(), CachedResult::new(value.clone()));
            Ok(value)
        }
    }
    ```

- [ ] **Task 2.3**: 集成缓存到Orchestrator
  - 修改：`extract_facts`、`extract_structured_facts`等方法
  - 添加缓存逻辑

- [ ] **Task 2.4**: 压测验证智能模式
  ```bash
  cargo run --release -- memory-creation --concurrency 100 --total 10000 --infer true
  ```
  - 目标吞吐量：1,000 ops/s
  - 目标延迟：P95 < 200ms

---

### Phase 3: 启用Agent并行执行（2天）

**目标**: Agent层并行执行，数据库操作从70ms降到20ms

- [ ] **Task 3.1**: 实现并行决策执行
  - 文件：`crates/agent-mem/src/orchestrator.rs`
  - 添加：`execute_decisions_parallel`方法
  - 实现：
    ```rust
    async fn execute_decisions_parallel(&self, decisions: Vec<MemoryDecision>) -> Result<()> {
        // 按类型分组决策
        let mut episodic_ops = Vec::new();
        let mut semantic_ops = Vec::new();
        let mut core_ops = Vec::new();
        
        for decision in decisions {
            match decision.memory_type {
                MemoryType::Episodic => episodic_ops.push(decision),
                MemoryType::Semantic => semantic_ops.push(decision),
                MemoryType::Core => core_ops.push(decision),
                _ => {}
            }
        }
        
        // 并行执行（原来顺序执行70ms → 现在并行20ms）
        tokio::try_join!(
            self.execute_episodic_ops(episodic_ops),
            self.execute_semantic_ops(semantic_ops),
            self.execute_core_ops(core_ops)
        )?;
        
        Ok(())
    }
    ```

- [ ] **Task 3.2**: 实现Agent池（可选，用于负载均衡）
  - 文件：`crates/agent-mem-core/src/agents/pool.rs`（新建）
  - 实现：
    ```rust
    pub struct AgentPool {
        episodic_agents: Vec<Arc<EpisodicAgent>>,
        semantic_agents: Vec<Arc<SemanticAgent>>,
        // ... 其他Agent
        load_balancer: LoadBalancingStrategy,
    }
    
    impl AgentPool {
        pub async fn execute_parallel(&self, tasks: Vec<Task>) -> Result<Vec<TaskResult>> {
            // 分发任务到不同Agent
            // 并行执行
            // 收集结果
        }
    }
    ```

- [ ] **Task 3.3**: 压测验证Agent并行
  ```bash
  cargo run --release -- concurrent-ops --users 1000 --duration 300
  ```
  - 目标CPU利用率：70%
  - 目标吞吐量：1,500+ ops/s

---

### Phase 4: 实现批量模式（2天）

**目标**: 批量处理，达到5,000 ops/s

- [ ] **Task 4.1**: 实现批量LLM调用
  - 文件：`crates/agent-mem/src/orchestrator.rs`
  - 添加：`add_memory_batch`方法
  - 实现：
    ```rust
    pub async fn add_memory_batch(&self, contents: Vec<String>, ...) -> Result<Vec<AddResult>> {
        // 1. 批量提取事实（100条/次，50ms）
        let facts_batch = self.extract_facts_batch(&contents).await?;
        
        // 2. 批量生成嵌入（100条/次，20ms）
        let embeddings = self.embedder.embed_batch(contents.clone()).await?;
        
        // 3. 批量插入数据库（100条/次，30ms）
        self.insert_batch(contents, embeddings, facts_batch).await?;
        
        Ok(results)
    }
    ```

- [ ] **Task 4.2**: 实现批量实体提取
  - 使用现有的`BatchEntityExtractor`
  - 集成到批量流程

- [ ] **Task 4.3**: 压测验证批量模式
  ```bash
  cargo run --release -- batch-operations --batch-size 100 --real true
  ```
  - 目标吞吐量：5,000 ops/s
  - 目标平均延迟：< 1ms/条

---

### Phase 5: 集成高级能力（3天，可选）

**目标**: 集成图推理、高级推理、聚类分析等能力

- [ ] **Task 5.1**: 集成GraphMemoryEngine
  - 文件：`crates/agent-mem/src/orchestrator.rs`
  - 添加：`graph_reasoning`方法
  - 暴露API：`memory.graph_reasoning(query, user_id)`

- [ ] **Task 5.2**: 集成AdvancedReasoner
  - 添加：`advanced_reasoning`方法
  - 暴露API：`memory.advanced_reasoning(query, user_id)`

- [ ] **Task 5.3**: 集成ClusteringEngine
  - 添加：`cluster_memories`方法
  - 暴露API：`memory.cluster_memories(user_id)`

- [ ] **Task 5.4**: 更新文档和示例
  - 编写使用指南
  - 添加代码示例

---

## 性能目标

| 模式 | 当前性能 | 目标性能 | 提升倍数 |
|------|---------|---------|---------|
| **快速模式** | 577 ops/s | 10,000+ ops/s | **17x** |
| **智能模式** | 577 ops/s | 1,000 ops/s | **1.7x** |
| **批量模式** | 36.66 ops/s | 5,000 ops/s | **136x** |
| **延迟（快速）** | 24ms | 5ms | **4.8x** |
| **延迟（智能）** | 300ms | 170ms | **1.8x** |
| **CPU利用率** | 15.76% | 70% | **4.4x** |

---

## 实施时间表

| Phase | 任务 | 工作量 | 完成标准 |
|-------|------|--------|---------|
| **Phase 0** | 基准测试 | 1天 | 获取基准数据 |
| **Phase 1** | 快速模式 | 2天 | 10,000+ ops/s |
| **Phase 2** | 优化LLM | 3天 | 1,000 ops/s |
| **Phase 3** | Agent并行 | 2天 | CPU 70% |
| **Phase 4** | 批量模式 | 2天 | 5,000 ops/s |
| **Phase 5** | 高级能力 | 3天 | API可用 |
| **总计** | - | **13天** | 全部完成 |

---

**文档版本**: 1.0  
**创建日期**: 2025-11-14  
**状态**: ✅ 计划完成，待执行

