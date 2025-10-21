# AgentMem 详细改造计划

## 📋 Phase 1: 删除所有 Mock 代码 (优先级: 🔴 最高)

### 任务 1.1: 清理 Agent Mock 代码

**文件**: 
- `crates/agent-mem-core/src/agents/semantic_agent.rs`
- `crates/agent-mem-core/src/agents/core_agent.rs`
- `crates/agent-mem-core/src/agents/working_agent.rs`

**当前代码**:
```rust
// semantic_agent.rs line 450
// Fallback to mock response if store not available
let response = serde_json::json!({
    "success": true,
    "knowledge": []
});
return Ok(TaskResponse::success(Some(response)));
```

**改造后**:
```rust
// 如果 store 不可用，返回错误
if self.semantic_store.is_none() {
    return Err(AgentError::ConfigurationError(
        "Semantic store not configured. Please initialize the agent with a valid store.".to_string()
    ));
}

// 真实调用 store
let store = self.semantic_store.as_ref().unwrap();
let items = store.query_items(&user_id, &query).await?;

let response = serde_json::json!({
    "success": true,
    "knowledge": items
});
Ok(TaskResponse::success(Some(response)))
```

**验收标准**:
- ✅ 所有 "Fallback to mock" 代码已删除
- ✅ 所有操作都真实调用 store
- ✅ 如果 store 未配置，返回明确错误

### 任务 1.2: 实现 Hash 计算

**文件**: `crates/agent-mem-core/src/types.rs`

**当前代码**:
```rust
hash: None, // TODO: Calculate hash if needed
```

**改造后**:
```rust
use sha2::{Sha256, Digest};

impl SemanticMemoryItem {
    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.content.as_bytes());
        hasher.update(self.agent_id.as_bytes());
        hasher.update(self.user_id.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

// 使用
hash: Some(self.calculate_hash()),
```

**验收标准**:
- ✅ 所有 "TODO: Calculate hash" 已实现
- ✅ Hash 计算包含关键字段
- ✅ Hash 用于去重检测

### 任务 1.3: 实现实体和关系提取

**文件**: `crates/agent-mem-core/src/extraction/` (使用已有模块)

**当前代码**:
```rust
entities: Vec::new(),  // TODO: Extract entities if needed
relations: Vec::new(), // TODO: Extract relations if needed
```

**改造后**:
```rust
use crate::extraction::{EntityExtractor, RelationExtractor};

// 在 Orchestrator 初始化时创建
let entity_extractor = EntityExtractor::new(llm_provider.clone());
let relation_extractor = RelationExtractor::new(llm_provider.clone());

// 提取实体
let entities = entity_extractor.extract(&content).await?;

// 提取关系
let relations = relation_extractor.extract(&content, &entities).await?;
```

**验收标准**:
- ✅ 所有 "TODO: Extract entities" 已实现
- ✅ 使用已有的 extraction 模块
- ✅ 实体和关系正确提取

---

## 📋 Phase 2: 集成智能组件 (优先级: 🔴 最高)

### 任务 2.1: 集成 FactExtractor 到 Orchestrator

**文件**: `crates/agent-mem/src/orchestrator.rs`

**改造步骤**:

#### 步骤 1: 添加依赖

```rust
use agent_mem_intelligence::{
    FactExtractor, 
    ExtractedFact, 
    FactCategory,
    FactExtractionConfig
};
use agent_mem_llm::LLMProvider;
```

#### 步骤 2: 添加字段

```rust
pub struct MemoryOrchestrator {
    // 现有字段...
    
    // 新增智能组件
    fact_extractor: Option<Arc<FactExtractor>>,
    llm_provider: Option<Arc<dyn LLMProvider + Send + Sync>>,
}
```

#### 步骤 3: 初始化

```rust
impl MemoryOrchestrator {
    pub async fn new_with_config(config: OrchestratorConfig) -> Result<Self> {
        // 创建 LLM Provider
        let llm_provider = if let Some(provider_config) = &config.llm_provider {
            Some(create_llm_provider(provider_config)?)
        } else {
            None
        };
        
        // 创建 FactExtractor
        let fact_extractor = if let Some(llm) = &llm_provider {
            let extraction_config = FactExtractionConfig::default();
            Some(Arc::new(FactExtractor::new(
                llm.clone(),
                extraction_config
            )))
        } else {
            None
        };
        
        Ok(Self {
            fact_extractor,
            llm_provider,
            // ...
        })
    }
}
```

#### 步骤 4: 使用 FactExtractor

```rust
impl MemoryOrchestrator {
    /// 提取事实
    async fn extract_facts(&self, content: &str) -> Result<Vec<ExtractedFact>> {
        if let Some(extractor) = &self.fact_extractor {
            extractor.extract_facts(content).await
        } else {
            // 降级：创建简单事实
            Ok(vec![ExtractedFact {
                content: content.to_string(),
                confidence: 1.0,
                category: FactCategory::Knowledge,
                entities: Vec::new(),
                temporal_info: None,
                importance: 0.5,
            }])
        }
    }
}
```

**验收标准**:
- ✅ FactExtractor 成功集成
- ✅ 可以提取事实
- ✅ 支持降级模式

### 任务 2.2: 集成 DecisionEngine 到 Orchestrator

**文件**: `crates/agent-mem/src/orchestrator.rs`

**改造步骤**:

#### 步骤 1: 添加依赖

```rust
use agent_mem_intelligence::{
    MemoryDecisionEngine,
    MemoryAction,
    DecisionEngineConfig,
    MergeStrategy
};
```

#### 步骤 2: 添加字段

```rust
pub struct MemoryOrchestrator {
    // 新增
    decision_engine: Option<Arc<MemoryDecisionEngine>>,
}
```

#### 步骤 3: 初始化

```rust
let decision_engine = if let Some(llm) = &llm_provider {
    let decision_config = DecisionEngineConfig {
        similarity_threshold: 0.85,
        confidence_threshold: 0.7,
        conflict_detection_enabled: true,
        max_merge_candidates: 5,
    };
    Some(Arc::new(MemoryDecisionEngine::new(
        llm.clone(),
        decision_config
    )))
} else {
    None
};
```

#### 步骤 4: 使用 DecisionEngine

```rust
impl MemoryOrchestrator {
    /// 决策记忆操作
    async fn decide_memory_actions(
        &self,
        facts: Vec<ExtractedFact>,
        existing_memories: Vec<MemoryItem>,
    ) -> Result<Vec<MemoryAction>> {
        if let Some(engine) = &self.decision_engine {
            engine.decide_actions(facts, existing_memories).await
        } else {
            // 降级：直接添加所有事实
            Ok(facts.into_iter().map(|fact| {
                MemoryAction::Add {
                    content: fact.content,
                    importance: fact.importance,
                    metadata: HashMap::new(),
                }
            }).collect())
        }
    }
}
```

**验收标准**:
- ✅ DecisionEngine 成功集成
- ✅ 可以决策操作类型
- ✅ 支持 ADD/UPDATE/DELETE/MERGE

### 任务 2.3: 实现智能添加方法

**文件**: `crates/agent-mem/src/orchestrator.rs`

**新增方法**: 见下一个文件（代码太长）

**验收标准**:
- ✅ `add_memory_intelligent()` 方法实现
- ✅ 支持事实提取
- ✅ 支持智能决策
- ✅ 支持 ADD/UPDATE/DELETE/MERGE 操作
- ✅ 所有操作真实执行

---

## 📋 Phase 3: 集成混合搜索引擎 (优先级: 🟡 高)

### 任务 3.1: 使用 HybridSearchEngine

**文件**: `crates/agent-mem/src/orchestrator.rs`

**改造步骤**:

#### 步骤 1: 添加依赖

```rust
use agent_mem_core::search::{
    HybridSearchEngine,
    HybridSearchConfig,
    SearchResult
};
```

#### 步骤 2: 添加字段

```rust
pub struct MemoryOrchestrator {
    // 新增
    hybrid_search_engine: Option<Arc<HybridSearchEngine>>,
}
```

#### 步骤 3: 初始化

```rust
let hybrid_search_engine = if let Some(vector_engine) = &vector_search_engine {
    let config = HybridSearchConfig {
        vector_weight: 0.7,
        fulltext_weight: 0.3,
        rrf_k: 60,
        min_score: 0.5,
    };
    Some(Arc::new(HybridSearchEngine::new(
        vector_engine.clone(),
        fulltext_engine.clone(),
        config
    )))
} else {
    None
};
```

#### 步骤 4: 使用混合搜索

```rust
impl MemoryOrchestrator {
    pub async fn search_memories(
        &self,
        query: String,
        agent_id: String,
        user_id: Option<String>,
        limit: usize,
        memory_type: Option<MemoryType>,
    ) -> Result<Vec<MemoryItem>> {
        // 优先使用 HybridSearchEngine
        if let Some(engine) = &self.hybrid_search_engine {
            let filters = build_filters(user_id, agent_id, memory_type);
            
            let results = engine.search_hybrid(
                query.clone(),
                limit,
                Some(filters),
            ).await?;
            
            return Ok(results.into_iter()
                .map(|r| convert_search_result_to_memory_item(r))
                .collect());
        }
        
        // 降级：使用 Agent 搜索
        self.search_via_agents(query, memory_type, limit).await
    }
}
```

**验收标准**:
- ✅ HybridSearchEngine 成功集成
- ✅ 搜索使用向量+全文混合
- ✅ 结果使用 RRF 融合
- ✅ 支持相似度阈值过滤

---

## 📋 Phase 4: 添加向量存储抽象层 (优先级: 🟡 高)

### 任务 4.1: 创建 VectorStore Trait

**文件**: `crates/agent-mem-traits/src/vector_store.rs` (新建)

```rust
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

#[async_trait]
pub trait VectorStore: Send + Sync {
    /// 添加向量
    async fn add(
        &self,
        id: String,
        vector: Vec<f32>,
        metadata: HashMap<String, Value>,
    ) -> Result<()>;
    
    /// 搜索相似向量
    async fn search(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        filters: Option<HashMap<String, Value>>,
        threshold: Option<f32>,
    ) -> Result<Vec<VectorSearchResult>>;
    
    /// 删除向量
    async fn delete(&self, id: String) -> Result<()>;
    
    /// 更新向量
    async fn update(
        &self,
        id: String,
        vector: Vec<f32>,
        metadata: HashMap<String, Value>,
    ) -> Result<()>;
}

/// 向量搜索结果
#[derive(Debug, Clone)]
pub struct VectorSearchResult {
    pub id: String,
    pub score: f32,
    pub metadata: HashMap<String, Value>,
}
```

### 任务 4.2: 实现多个 VectorStore

**目录结构**:
```
crates/agent-mem-storage/src/vector/
    ├─ mod.rs
    ├─ lancedb.rs (已有，需要适配)
    ├─ qdrant.rs (新增)
    ├─ chroma.rs (新增)
    ├─ pgvector.rs (新增)
    └─ factory.rs (新增)
```

**Factory 实现**:
```rust
pub struct VectorStoreFactory;

impl VectorStoreFactory {
    pub fn create(
        provider: &str,
        config: VectorStoreConfig,
    ) -> Result<Arc<dyn VectorStore>> {
        match provider {
            "lancedb" => Ok(Arc::new(LanceDBStore::new(config)?)),
            "qdrant" => Ok(Arc::new(QdrantStore::new(config)?)),
            "chroma" => Ok(Arc::new(ChromaStore::new(config)?)),
            "pgvector" => Ok(Arc::new(PGVectorStore::new(config)?)),
            _ => Err(Error::UnsupportedProvider(provider.to_string())),
        }
    }
}
```

**验收标准**:
- ✅ VectorStore trait 定义完成
- ✅ 至少实现 4 个向量存储
- ✅ Factory 模式实现
- ✅ 可以动态切换向量库

---

## 📋 Phase 5: 添加图存储支持 (优先级: 🟢 中)

### 任务 5.1: 创建 GraphStore Trait

**文件**: `crates/agent-mem-traits/src/graph_store.rs` (新建)

```rust
#[async_trait]
pub trait GraphStore: Send + Sync {
    /// 添加实体
    async fn add_entity(&self, entity: Entity) -> Result<String>;
    
    /// 添加关系
    async fn add_relation(&self, relation: Relation) -> Result<String>;
    
    /// 查询实体
    async fn query_entities(&self, filters: HashMap<String, Value>) -> Result<Vec<Entity>>;
    
    /// 查询关系
    async fn query_relations(&self, filters: HashMap<String, Value>) -> Result<Vec<Relation>>;
    
    /// 查询路径
    async fn query_path(&self, from: String, to: String, max_depth: usize) -> Result<Vec<Path>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub entity_type: String,
    pub properties: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub id: String,
    pub from_entity: String,
    pub to_entity: String,
    pub relation_type: String,
    pub properties: HashMap<String, Value>,
}
```

### 任务 5.2: 实现 Neo4j GraphStore

**文件**: `crates/agent-mem-storage/src/graph/neo4j.rs` (新建)

**验收标准**:
- ✅ GraphStore trait 定义完成
- ✅ Neo4j 实现完成
- ✅ 支持实体和关系的 CRUD
- ✅ 支持路径查询

---

## 📋 Phase 6: 添加历史记录功能 (优先级: 🟢 中)

### 任务 6.1: 创建 HistoryStore

**文件**: `crates/agent-mem-storage/src/history.rs` (新建)

```rust
pub struct HistoryStore {
    db: Arc<LibSQLConnection>,
}

impl HistoryStore {
    /// 记录操作历史
    pub async fn record_operation(
        &self,
        memory_id: &str,
        operation: OperationType,
        old_value: Option<String>,
        new_value: Option<String>,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO memory_history (
                memory_id, operation_type, old_value, new_value, created_at
            ) VALUES (?, ?, ?, ?, ?)
            "#,
            memory_id,
            operation.to_string(),
            old_value,
            new_value,
            Utc::now()
        )
        .execute(self.db.as_ref())
        .await?;
        
        Ok(())
    }
    
    /// 获取记忆历史
    pub async fn get_history(&self, memory_id: &str) -> Result<Vec<HistoryEntry>> {
        let rows = sqlx::query_as!(
            HistoryEntry,
            r#"
            SELECT * FROM memory_history
            WHERE memory_id = ?
            ORDER BY created_at DESC
            "#,
            memory_id
        )
        .fetch_all(self.db.as_ref())
        .await?;
        
        Ok(rows)
    }
}
```

**验收标准**:
- ✅ HistoryStore 实现完成
- ✅ 可以记录所有操作
- ✅ 可以查询历史记录
- ✅ 支持时间排序

---

## 📋 Phase 7: 更新 Memory API (优先级: 🔴 最高)

### 任务 7.1: 添加 infer 参数支持

**文件**: `crates/agent-mem/src/memory.rs`

```rust
impl Memory {
    /// 添加记忆（支持 infer 参数）
    pub async fn add_with_options(
        &self,
        content: impl Into<String>,
        options: AddMemoryOptions,
    ) -> Result<AddResult> {
        let content = content.into();
        
        let orchestrator = self.orchestrator.read().await;
        
        if options.infer {
            // 智能添加
            let result = orchestrator.add_memory_intelligent(
                content,
                self.default_agent_id.clone(),
                options.user_id.or_else(|| self.default_user_id.clone()),
                options.metadata,
            ).await?;
            
            Ok(AddResult {
                results: result.operations.into_iter()
                    .map(|op| MemoryEvent::from_operation(op))
                    .collect(),
                relations: None, // TODO: 图存储
            })
        } else {
            // 直接添加
            let memory_id = orchestrator.add_memory(
                content.clone(),
                self.default_agent_id.clone(),
                options.user_id.or_else(|| self.default_user_id.clone()),
                options.memory_type,
                options.metadata,
            ).await?;
            
            Ok(AddResult {
                results: vec![MemoryEvent {
                    id: memory_id,
                    memory: content,
                    event: "ADD".to_string(),
                    actor_id: Some(self.default_agent_id.clone()),
                    role: Some("user".to_string()),
                }],
                relations: None,
            })
        }
    }
}
```

**验收标准**:
- ✅ `add_with_options()` 支持 infer 参数
- ✅ infer=true 时使用智能添加
- ✅ infer=false 时直接添加
- ✅ 返回格式 mem0 兼容

---

## 🎯 总结

### 改造优先级

1. **🔴 最高优先级** (Week 1-2):
   - Phase 1: 删除 Mock 代码
   - Phase 2: 集成智能组件
   - Phase 7: 更新 Memory API

2. **🟡 高优先级** (Week 3-4):
   - Phase 3: 集成混合搜索
   - Phase 4: 向量存储抽象

3. **🟢 中优先级** (Week 5):
   - Phase 5: 图存储支持
   - Phase 6: 历史记录功能

### 预期效果

| 指标 | 改造前 | 改造后 | 改进 |
|------|--------|--------|------|
| Mock 代码 | ~30 处 | 0 处 | -100% |
| 智能功能 | 0% | 100% | +100% |
| 向量存储支持 | 1 个 | 4+ 个 | +300% |
| 图存储支持 | 0 | 1+ 个 | +100% |
| 历史记录 | 无 | 完整 | +100% |

