# AgentMem 最佳架构设计方案

> **基于主流论文研究和 mem0 深度对比**
> 
> 设计日期: 2025-10-21
> 
> 参考论文: MIRIX (2025), Grounded Memory (2025), HybridRAG (2024), Graphiti (2024)

---

## 🎓 主流论文研究总结

### 1. MIRIX: Multi-Agent Memory System (2025)

**论文**: [arXiv:2507.07957](https://arxiv.org/html/2507.07957v1)

**核心贡献**:
- ✅ **6 种记忆类型**: Core, Episodic, Semantic, Procedural, Working, Contextual
- ✅ **模块化设计**: 每种记忆类型独立管理
- ✅ **智能衰减**: Intelligent Decay 机制
- ✅ **混合存储**: 结构化 + 向量数据库

**对 agentmen 的启示**:
- ✅ agentmen 已实现 8 种记忆类型（比 MIRIX 更多）
- ❌ 缺少智能衰减机制
- ❌ 记忆类型之间缺少协作

### 2. Grounded Memory System (2025)

**论文**: [arXiv:2505.06328](https://arxiv.org/html/2505.06328v1)

**核心贡献**:
- ✅ **知识图谱 + 向量嵌入**: 混合表示
- ✅ **实体/关系提取**: 使用 LLM
- ✅ **混合检索**: 图遍历 + 向量搜索

**对 agentmen 的启示**:
- ❌ agentmen 没有知识图谱
- ❌ 没有实体/关系提取（虽然 extraction 模块存在）
- ⚠️ 只有向量搜索，没有图遍历

### 3. HybridRAG (2024)

**论文**: [arXiv:2408.04948](https://arxiv.org/html/2408.04948v1)

**核心贡献**:
- ✅ **GraphRAG + VectorRAG**: 混合检索
- ✅ **RRF 融合**: Reciprocal Rank Fusion
- ✅ **上下文增强**: 图结构提供额外上下文

**对 agentmen 的启示**:
- ✅ HybridSearchEngine 已实现（但未使用！）
- ✅ RRF 融合已实现
- ❌ 没有 GraphRAG

### 4. Graphiti - Temporal Knowledge Graph (2024)

**来源**: Zep AI

**核心贡献**:
- ✅ **时间感知**: 知识图谱包含时间维度
- ✅ **动态更新**: 增量更新和冲突解决
- ✅ **LLM 集成**: 使用 LLM 提取实体和关系

**对 agentmen 的启示**:
- ❌ 没有时间感知的知识图谱
- ❌ 没有动态更新机制
- ⚠️ extraction 模块存在但未集成

---

## 🏗️ 最佳架构设计

### 设计原则

1. **简洁优先**: 参考 mem0 的简洁设计
2. **模块化**: 参考 MIRIX 的模块化架构
3. **混合检索**: 参考 HybridRAG 的混合检索
4. **知识图谱**: 参考 Grounded Memory 的图增强
5. **时间感知**: 参考 Graphiti 的时间维度

### 推荐架构

```
┌─────────────────────────────────────────────────────────────┐
│                     Layer 1: API Interface                   │
│  Memory.add(infer=true), search(), get(), update(), delete() │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                  Layer 2: Processing Engine                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │FactExtractor│  │DecisionEngine│  │EntityExtractor│      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                   Layer 3: Orchestrator                      │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  Memory Type Router (8 types)                        │   │
│  │  ├─ Semantic  ├─ Episodic  ├─ Core  ├─ Procedural   │   │
│  │  ├─ Resource  ├─ Working   ├─ Knowledge  ├─ Context │   │
│  └──────────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  Hybrid Search Engine                                │   │
│  │  ├─ Vector Search (LanceDB, Qdrant, Chroma, etc.)   │   │
│  │  ├─ Fulltext Search (LibSQL)                         │   │
│  │  └─ RRF Fusion                                       │   │
│  └──────────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  Knowledge Graph Engine (Optional)                   │   │
│  │  ├─ Entity Storage (Neo4j, FalkorDB)                 │   │
│  │  ├─ Relation Storage                                 │   │
│  │  └─ Graph Traversal                                  │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                    Layer 4: Storage Layer                    │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ Vector Store │  │ Struct Store │  │ Graph Store  │      │
│  │ (Pluggable)  │  │ (LibSQL/PG)  │  │ (Optional)   │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└─────────────────────────────────────────────────────────────┘
```

### 核心改进点

#### 1. 简化 Orchestrator

**当前问题**:
- Orchestrator 太复杂（1,014 行）
- 通过 Agent 搜索效率低
- 没有使用 HybridSearchEngine

**改进方案**:
```rust
pub struct MemoryOrchestrator {
    // 智能组件
    fact_extractor: Option<Arc<FactExtractor>>,
    decision_engine: Option<Arc<MemoryDecisionEngine>>,
    entity_extractor: Option<Arc<EntityExtractor>>,
    
    // 搜索引擎
    hybrid_search_engine: Arc<HybridSearchEngine>,
    
    // 知识图谱（可选）
    graph_engine: Option<Arc<GraphEngine>>,
    
    // 8 个 Agents（保留）
    semantic_agent: Option<Arc<RwLock<SemanticAgent>>>,
    episodic_agent: Option<Arc<RwLock<EpisodicAgent>>>,
    // ...
    
    // 配置
    config: OrchestratorConfig,
}
```

#### 2. 智能添加流程

**参考 mem0 + MIRIX**:
```rust
pub async fn add_memory_intelligent(
    &self,
    content: String,
    options: AddOptions,
) -> Result<AddResult> {
    // Step 1: 提取事实
    let facts = self.fact_extractor
        .extract_facts(&content).await?;
    
    // Step 2: 提取实体和关系（如果启用图存储）
    let (entities, relations) = if self.graph_engine.is_some() {
        let entities = self.entity_extractor
            .extract_entities(&content).await?;
        let relations = self.entity_extractor
            .extract_relations(&content, &entities).await?;
        (entities, relations)
    } else {
        (Vec::new(), Vec::new())
    };
    
    // Step 3: 搜索相似记忆（使用混合搜索）
    let similar_memories = self.hybrid_search_engine
        .search_hybrid(content.clone(), 10, None).await?;
    
    // Step 4: 决策操作（ADD/UPDATE/DELETE/MERGE）
    let actions = self.decision_engine
        .decide_actions(facts, similar_memories).await?;
    
    // Step 5: 执行操作
    let mut results = Vec::new();
    for action in actions {
        match action {
            MemoryAction::Add { content, importance, metadata } => {
                // 推断记忆类型
                let memory_type = self.infer_memory_type(&content).await?;
                
                // 路由到对应 Agent
                let memory_id = self.route_to_agent(
                    memory_type,
                    content.clone(),
                    options.agent_id.clone(),
                    options.user_id.clone(),
                    Some(metadata),
                ).await?;
                
                results.push(MemoryOperation {
                    operation_type: "ADD".to_string(),
                    memory_id,
                    content,
                    old_content: None,
                });
            }
            MemoryAction::Update { memory_id, new_content, .. } => {
                // 更新记忆
                self.update_memory(&memory_id, new_content.clone()).await?;
                results.push(MemoryOperation {
                    operation_type: "UPDATE".to_string(),
                    memory_id,
                    content: new_content,
                    old_content: None,
                });
            }
            MemoryAction::Delete { memory_id, .. } => {
                // 删除记忆
                self.delete_memory(&memory_id).await?;
                results.push(MemoryOperation {
                    operation_type: "DELETE".to_string(),
                    memory_id,
                    content: String::new(),
                    old_content: None,
                });
            }
            MemoryAction::Merge { primary_memory_id, secondary_memory_ids, merged_content } => {
                // 合并记忆
                self.merge_memories(primary_memory_id, secondary_memory_ids, merged_content).await?;
            }
            MemoryAction::NoAction { .. } => {
                // 无操作
            }
        }
    }
    
    // Step 6: 存储到知识图谱（如果启用）
    if let Some(graph) = &self.graph_engine {
        graph.add_entities(entities).await?;
        graph.add_relations(relations).await?;
    }
    
    Ok(AddResult {
        operations: results,
        facts_extracted: facts.len(),
        entities_extracted: entities.len(),
        relations_extracted: relations.len(),
    })
}
```

#### 3. 混合搜索流程

**参考 HybridRAG**:
```rust
pub async fn search_memories(
    &self,
    query: String,
    options: SearchOptions,
) -> Result<Vec<MemoryItem>> {
    // 使用 HybridSearchEngine（已实现但未使用）
    let results = self.hybrid_search_engine.search_hybrid(
        query.clone(),
        options.limit,
        Some(build_filters(options)),
    ).await?;
    
    // 如果启用图存储，增强结果
    if let Some(graph) = &self.graph_engine {
        let graph_results = graph.search(query, options.limit).await?;
        // 合并向量搜索和图搜索结果
        return merge_results(results, graph_results);
    }
    
    Ok(results.into_iter()
        .map(|r| convert_to_memory_item(r))
        .collect())
}
```

#### 4. 向量存储抽象层

**参考 mem0 的设计**:
```rust
// Trait 定义
#[async_trait]
pub trait VectorStore: Send + Sync {
    async fn add(&self, id: String, vector: Vec<f32>, metadata: HashMap<String, Value>) -> Result<()>;
    async fn search(&self, query_vector: Vec<f32>, limit: usize, filters: Option<HashMap<String, Value>>, threshold: Option<f32>) -> Result<Vec<VectorSearchResult>>;
    async fn delete(&self, id: String) -> Result<()>;
    async fn update(&self, id: String, vector: Vec<f32>, metadata: HashMap<String, Value>) -> Result<()>;
}

// Factory 模式
pub struct VectorStoreFactory;

impl VectorStoreFactory {
    pub fn create(provider: &str, config: VectorStoreConfig) -> Result<Arc<dyn VectorStore>> {
        match provider {
            "lancedb" => Ok(Arc::new(LanceDBStore::new(config)?)),
            "qdrant" => Ok(Arc::new(QdrantStore::new(config)?)),
            "chroma" => Ok(Arc::new(ChromaStore::new(config)?)),
            "pgvector" => Ok(Arc::new(PGVectorStore::new(config)?)),
            "weaviate" => Ok(Arc::new(WeaviateStore::new(config)?)),
            _ => Err(Error::UnsupportedProvider(provider.to_string())),
        }
    }
}
```

#### 5. 知识图谱集成

**参考 Grounded Memory + Graphiti**:
```rust
pub struct GraphEngine {
    graph_store: Arc<dyn GraphStore>,
    entity_extractor: Arc<EntityExtractor>,
    relation_extractor: Arc<RelationExtractor>,
}

impl GraphEngine {
    pub async fn add_entities(&self, entities: Vec<Entity>) -> Result<()> {
        for entity in entities {
            self.graph_store.add_entity(entity).await?;
        }
        Ok(())
    }
    
    pub async fn add_relations(&self, relations: Vec<Relation>) -> Result<()> {
        for relation in relations {
            self.graph_store.add_relation(relation).await?;
        }
        Ok(())
    }
    
    pub async fn search(&self, query: String, limit: usize) -> Result<Vec<GraphSearchResult>> {
        // 1. 提取查询中的实体
        let query_entities = self.entity_extractor.extract(&query).await?;
        
        // 2. 在图中查找相关实体
        let mut results = Vec::new();
        for entity in query_entities {
            let related = self.graph_store.query_entities(
                hashmap!{"name" => entity.name}
            ).await?;
            results.extend(related);
        }
        
        Ok(results)
    }
}
```

---

## 📊 改造优先级

### 🔴 Phase 1: 核心功能真实化 (Week 1)

1. **删除所有 Mock 代码**
   - 清理 Agent Mock 代码
   - 实现 Hash 计算
   - 实现实体和关系提取

2. **集成智能组件**
   - 集成 FactExtractor
   - 集成 DecisionEngine
   - 实现智能添加方法

### 🟡 Phase 2: 搜索优化 (Week 2)

3. **使用 HybridSearchEngine**
   - 替换 Agent 搜索
   - 实现向量搜索
   - 实现相似度阈值

4. **向量存储抽象**
   - 创建 VectorStore trait
   - 实现 4+ 个向量存储
   - Factory 模式

### 🟢 Phase 3: 高级功能 (Week 3-4)

5. **知识图谱集成**
   - 创建 GraphStore trait
   - 实现 Neo4j GraphStore
   - 集成实体/关系提取

6. **历史记录和优化**
   - 实现 HistoryStore
   - 添加缓存机制
   - 性能优化

---

## ✅ 验收标准

### Phase 1
- ✅ 所有 mock 代码已删除
- ✅ FactExtractor 成功集成
- ✅ DecisionEngine 成功集成
- ✅ `add(content, infer=true)` 正常工作

### Phase 2
- ✅ HybridSearchEngine 成功使用
- ✅ 支持 4+ 个向量数据库
- ✅ 搜索性能提升 > 50%

### Phase 3
- ✅ 知识图谱功能正常
- ✅ 历史记录功能正常
- ✅ 所有测试通过

---

## 🎯 最终目标

**打造一个真实、完整、生产级的记忆管理系统，功能对标 mem0，性能超越 mem0！**

**关键指标**:
- Mock 代码: 0 处
- 智能功能: 100%
- 向量存储支持: 5+ 个
- 知识图谱: 完整支持
- 搜索性能: 提升 > 50%

