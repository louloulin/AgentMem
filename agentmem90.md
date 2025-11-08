# AgentMem 通用记忆平台架构改造方案

**文档版本**: v3.0 (架构级改造)  
**创建日期**: 2025-11-08  
**核心理念**: 架构优先 + 抽象能力 + 泛化设计  
**参考**: agentmem80.md深度分析 + Cursor/Augment记忆机制 + Mem0架构

---

## 🎯 架构愿景

### 从功能到能力

**错误思维**: "我们要实现商品搜索、用户搜索、文档搜索..."  
**正确思维**: "我们要构建一个能够理解、存储、检索任意类型信息的通用记忆引擎"

### 核心目标

构建一个**通用记忆平台**，具备以下核心能力：

1. **理解能力** (Understanding): 理解任意查询的意图和上下文
2. **组织能力** (Organization): 以最优方式组织和索引记忆
3. **检索能力** (Retrieval): 精准检索相关记忆
4. **学习能力** (Learning): 从反馈中持续优化
5. **扩展能力** (Extension): 支持任意领域扩展

---

## 📐 核心抽象层

### 1. 记忆抽象 (Memory Abstraction)

**核心问题**: 什么是"记忆"？

**当前实现的局限**:
```rust
// ❌ 过于具体，无法泛化
struct Memory {
    content: String,
    user_id: String,
    agent_id: String,
    // ... 固定字段
}
```

**正确的抽象**:
```rust
/// 记忆 = 内容 + 属性 + 关系
pub struct Memory {
    /// 核心：内容（可以是任何形式）
    content: Content,
    
    /// 核心：属性（开放式，支持任意属性）
    attributes: AttributeSet,
    
    /// 核心：关系（与其他记忆/实体的关系）
    relations: RelationGraph,
    
    /// 元信息（系统维护）
    metadata: Metadata,
}

/// 内容抽象（多模态）
pub enum Content {
    /// 文本
    Text(String),
    
    /// 结构化数据
    Structured(Value),
    
    /// 向量（嵌入）
    Vector(Vec<f32>),
    
    /// 多模态组合
    Multimodal(Vec<Content>),
}

/// 属性集（完全开放）
pub struct AttributeSet {
    /// 属性字典（类型安全）
    attributes: HashMap<AttributeKey, AttributeValue>,
    
    /// 属性schema（可选，用于验证）
    schema: Option<AttributeSchema>,
}

/// 属性键（强类型，支持命名空间）
pub struct AttributeKey {
    /// 命名空间（避免冲突）
    namespace: String,
    
    /// 键名
    name: String,
}

/// 属性值（类型安全）
pub enum AttributeValue {
    String(String),
    Number(f64),
    Boolean(bool),
    DateTime(DateTime<Utc>),
    List(Vec<AttributeValue>),
    Map(HashMap<String, AttributeValue>),
    Custom(Box<dyn Any + Send + Sync>),
}

/// 关系图
pub struct RelationGraph {
    /// 出边（这个记忆指向其他）
    outgoing: Vec<Relation>,
    
    /// 入边（其他记忆指向这个）
    incoming: Vec<Relation>,
}

/// 关系（通用）
pub struct Relation {
    /// 关系类型（用户定义）
    relation_type: String,
    
    /// 目标ID
    target_id: String,
    
    /// 权重/强度
    strength: f32,
    
    /// 关系属性
    attributes: HashMap<String, Value>,
}
```

**关键洞察**:
- 记忆不是固定结构，是**开放的属性集合**
- 属性通过**命名空间**避免冲突（如 `ecommerce::product_id`, `user::email`）
- 关系是一等公民，支持图查询

### 2. 查询抽象 (Query Abstraction)

**核心问题**: 什么是"查询"？

**当前实现的局限**:
```rust
// ❌ 过于简单
fn search(query: String) -> Vec<Memory>
```

**正确的抽象**:
```rust
/// 查询 = 意图 + 约束 + 偏好
pub struct Query {
    /// 核心：查询意图（多种表达）
    intent: QueryIntent,
    
    /// 核心：约束条件（必须满足）
    constraints: Vec<Constraint>,
    
    /// 核心：偏好（软性要求）
    preferences: Vec<Preference>,
    
    /// 上下文（影响理解和检索）
    context: QueryContext,
}

/// 查询意图（多种表达方式）
pub enum QueryIntent {
    /// 自然语言
    NaturalLanguage {
        text: String,
        language: Language,
    },
    
    /// 结构化查询
    Structured {
        predicates: Vec<Predicate>,
    },
    
    /// 向量相似度
    Vector {
        embedding: Vec<f32>,
    },
    
    /// 混合（组合多种）
    Hybrid {
        intents: Vec<QueryIntent>,
        fusion: FusionStrategy,
    },
}

/// 约束（硬性条件）
pub enum Constraint {
    /// 属性约束
    Attribute {
        key: AttributeKey,
        operator: ComparisonOperator,
        value: AttributeValue,
    },
    
    /// 关系约束
    Relation {
        relation_type: String,
        target: String,
    },
    
    /// 时间约束
    Temporal {
        time_range: TimeRange,
    },
    
    /// 空间约束（Scope）
    Spatial {
        scope: ScopeConstraint,
    },
    
    /// 逻辑组合
    Logical {
        operator: LogicalOperator,
        constraints: Vec<Constraint>,
    },
}

/// Scope约束（抽象，不限于User/Agent/Global）
pub enum ScopeConstraint {
    /// 属性匹配
    AttributeMatch {
        key: AttributeKey,
        value: AttributeValue,
    },
    
    /// 关系匹配
    RelationMatch {
        relation_type: String,
        target: String,
    },
    
    /// 任意（无约束）
    Any,
}

/// 偏好（软性要求）
pub struct Preference {
    /// 偏好类型
    preference_type: PreferenceType,
    
    /// 权重（可调整）
    weight: f32,
}

pub enum PreferenceType {
    /// 时间偏好（新鲜度）
    Temporal(TemporalPreference),
    
    /// 相关性偏好
    Relevance(RelevancePreference),
    
    /// 多样性偏好
    Diversity(DiversityPreference),
    
    /// 自定义偏好
    Custom(Box<dyn CustomPreference>),
}
```

**关键洞察**:
- 查询是**意图+约束+偏好**的组合
- 约束是硬性的（必须满足）
- 偏好是软性的（影响排序）
- Scope不是固定的User/Agent，而是**任意属性或关系约束**

### 3. 检索抽象 (Retrieval Abstraction)

**核心问题**: 如何检索？

**正确的抽象**:
```rust
/// 检索引擎（可组合）
pub trait RetrievalEngine: Send + Sync {
    /// 检索
    async fn retrieve(
        &self,
        query: &Query,
        context: &RetrievalContext,
    ) -> Result<RetrievalResult>;
    
    /// 引擎名称
    fn name(&self) -> &str;
    
    /// 支持的查询类型
    fn supported_intents(&self) -> Vec<QueryIntentType>;
}

/// 检索结果
pub struct RetrievalResult {
    /// 记忆列表
    memories: Vec<ScoredMemory>,
    
    /// 解释（可选，用于调试）
    explanation: Option<RetrievalExplanation>,
    
    /// 性能指标
    metrics: RetrievalMetrics,
}

/// 评分的记忆
pub struct ScoredMemory {
    /// 记忆
    memory: Memory,
    
    /// 总分
    score: f32,
    
    /// 分数分解（可解释性）
    score_breakdown: HashMap<String, f32>,
}

/// 检索解释
pub struct RetrievalExplanation {
    /// 为什么选择这些记忆
    reasoning: Vec<ReasoningStep>,
    
    /// 使用的引擎
    engines_used: Vec<String>,
    
    /// 融合策略
    fusion_strategy: String,
}

/// 组合检索引擎（核心）
pub struct CompositeRetrievalEngine {
    /// 子引擎
    engines: Vec<Box<dyn RetrievalEngine>>,
    
    /// 路由策略（根据查询选择引擎）
    router: Box<dyn EngineRouter>,
    
    /// 融合策略（合并结果）
    fusion: Box<dyn ResultFusion>,
}

impl RetrievalEngine for CompositeRetrievalEngine {
    async fn retrieve(&self, query: &Query, context: &RetrievalContext) -> Result<RetrievalResult> {
        // 1. 路由：选择合适的引擎
        let selected_engines = self.router.route(query, &self.engines)?;
        
        // 2. 并行检索
        let results = futures::future::join_all(
            selected_engines.iter().map(|engine| {
                engine.retrieve(query, context)
            })
        ).await;
        
        // 3. 融合结果
        self.fusion.fuse(results)
    }
}
```

**关键洞察**:
- 检索引擎是**可组合的**（类似Unix管道）
- 路由策略决定使用哪些引擎
- 融合策略决定如何合并结果
- 支持可解释性

---

## 🏗️ 架构模式

### 1. 分层架构 (Layered Architecture)

```
┌─────────────────────────────────────────────────────────┐
│           应用层 (Application Layer)                     │
│  - REST API                                             │
│  - GraphQL API                                          │
│  - SDK                                                  │
└────────────────────┬────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────┐
│           服务层 (Service Layer)                         │
│  - MemoryService: 记忆增删改查                          │
│  - QueryService: 查询理解与执行                         │
│  - LearningService: 学习与优化                          │
└────────────────────┬────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────┐
│         能力层 (Capability Layer)                        │
│  - Understanding: 查询/记忆理解                         │
│  - Organization: 索引与组织                             │
│  - Retrieval: 检索                                      │
│  - Scoring: 评分                                        │
│  - Learning: 学习                                       │
└────────────────────┬────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────┐
│         引擎层 (Engine Layer)                            │
│  - VectorEngine: 向量检索                               │
│  - FulltextEngine: 全文检索                             │
│  - GraphEngine: 图查询                                  │
│  - StructuredEngine: 结构化查询                         │
│  - HybridEngine: 混合检索                               │
└────────────────────┬────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────┐
│         存储层 (Storage Layer)                           │
│  - VectorStore: 向量数据库                              │
│  - DocumentStore: 文档数据库                            │
│  - GraphStore: 图数据库                                 │
│  - RelationalStore: 关系数据库                          │
└─────────────────────────────────────────────────────────┘
```

**关键原则**:
1. **单向依赖**: 上层依赖下层，下层不依赖上层
2. **接口隔离**: 每层通过接口通信
3. **可替换性**: 每层的实现可替换

### 2. 管道与过滤器 (Pipeline & Filter)

```rust
/// 处理管道（通用模式）
pub struct Pipeline<T, R> {
    /// 过滤器链
    filters: Vec<Box<dyn Filter<T, R>>>,
}

/// 过滤器（可组合）
pub trait Filter<T, R>: Send + Sync {
    async fn process(&self, input: T) -> Result<R>;
}

/// 示例：查询处理管道
pub struct QueryPipeline {
    pipeline: Pipeline<Query, RetrievalResult>,
}

impl QueryPipeline {
    pub fn new() -> Self {
        Self {
            pipeline: Pipeline::new()
                // 1. 查询理解
                .add_filter(QueryUnderstandingFilter::new())
                // 2. 查询优化
                .add_filter(QueryOptimizationFilter::new())
                // 3. 查询路由
                .add_filter(QueryRoutingFilter::new())
                // 4. 查询执行
                .add_filter(QueryExecutionFilter::new())
                // 5. 结果后处理
                .add_filter(ResultPostProcessingFilter::new()),
        }
    }
}
```

**关键原则**:
1. **可组合**: 过滤器可以任意组合
2. **可配置**: 管道结构可配置
3. **可观测**: 每个阶段可独立监控

### 3. 策略模式 (Strategy Pattern)

```rust
/// 策略注册表（支持动态策略）
pub struct StrategyRegistry<T: ?Sized> {
    strategies: HashMap<String, Box<T>>,
}

impl<T: ?Sized> StrategyRegistry<T> {
    /// 注册策略
    pub fn register(&mut self, name: String, strategy: Box<T>) {
        self.strategies.insert(name, strategy);
    }
    
    /// 获取策略
    pub fn get(&self, name: &str) -> Option<&T> {
        self.strategies.get(name).map(|s| &**s)
    }
}

/// 示例：评分策略
pub trait ScoringStrategy: Send + Sync {
    fn score(&self, query: &Query, memory: &Memory, context: &Context) -> f32;
}

pub struct ScoringStrategyRegistry {
    registry: StrategyRegistry<dyn ScoringStrategy>,
}
```

**关键原则**:
1. **可扩展**: 用户可注册自定义策略
2. **可配置**: 通过配置选择策略
3. **类型安全**: 使用Trait保证类型安全

### 4. 发布-订阅 (Pub-Sub)

```rust
/// 事件总线（解耦通信）
pub struct EventBus {
    subscribers: HashMap<String, Vec<Box<dyn EventHandler>>>,
}

pub trait EventHandler: Send + Sync {
    async fn handle(&self, event: &Event) -> Result<()>;
}

/// 示例：学习事件
pub enum LearningEvent {
    /// 用户反馈
    UserFeedback {
        query: Query,
        result: RetrievalResult,
        feedback: Feedback,
    },
    
    /// 查询执行
    QueryExecuted {
        query: Query,
        result: RetrievalResult,
        metrics: Metrics,
    },
}

// 学习器订阅事件
impl EventHandler for LearningService {
    async fn handle(&self, event: &Event) -> Result<()> {
        match event {
            Event::Learning(LearningEvent::UserFeedback { query, result, feedback }) => {
                self.learn_from_feedback(query, result, feedback).await?;
            }
            _ => {}
        }
        Ok(())
    }
}
```

**关键原则**:
1. **解耦**: 组件间通过事件通信
2. **异步**: 事件处理异步进行
3. **可观测**: 所有事件可追踪

---

## 💡 核心能力设计

### 能力1: 查询理解 (Query Understanding)

**目标**: 将任意形式的查询转换为标准化的Query对象

```rust
/// 查询理解器（可扩展）
pub struct QueryUnderstanding {
    /// 特征提取器（可插拔）
    extractors: Vec<Box<dyn FeatureExtractor>>,
    
    /// 意图分类器（可学习）
    classifier: Box<dyn IntentClassifier>,
    
    /// 约束推断器（基于规则或学习）
    constraint_inferrer: Box<dyn ConstraintInferrer>,
}

pub trait FeatureExtractor: Send + Sync {
    /// 提取特征
    async fn extract(&self, input: &str) -> Result<Features>;
}

pub trait IntentClassifier: Send + Sync {
    /// 分类意图
    async fn classify(&self, features: &Features) -> Result<QueryIntent>;
}

pub trait ConstraintInferrer: Send + Sync {
    /// 推断约束
    async fn infer(&self, features: &Features, context: &Context) -> Result<Vec<Constraint>>;
}
```

**实现策略**:
1. **特征提取**: 从查询中提取各种特征（ID、实体、语义等）
2. **意图分类**: 判断查询类型（精确查询、模糊查询、探索查询）
3. **约束推断**: 推断隐式约束（如Scope）

### 能力2: 智能组织 (Intelligent Organization)

**目标**: 以最优方式组织和索引记忆

```rust
/// 组织策略
pub trait OrganizationStrategy: Send + Sync {
    /// 决定记忆的存储位置和索引方式
    async fn organize(&self, memory: &Memory) -> Result<OrganizationPlan>;
}

/// 组织计划
pub struct OrganizationPlan {
    /// 存储位置
    storage_targets: Vec<StorageTarget>,
    
    /// 索引策略
    indexing_strategies: Vec<IndexingStrategy>,
    
    /// 关系建立
    relation_building: Vec<RelationPlan>,
}

/// 存储目标
pub struct StorageTarget {
    /// 存储类型
    storage_type: StorageType,
    
    /// 存储配置
    config: HashMap<String, Value>,
}

pub enum StorageType {
    /// 向量存储（用于语义检索）
    Vector,
    
    /// 全文索引（用于关键词检索）
    Fulltext,
    
    /// 图存储（用于关系检索）
    Graph,
    
    /// 结构化存储（用于属性查询）
    Structured,
}
```

**实现策略**:
1. **多索引**: 同一记忆建立多种索引
2. **智能路由**: 根据记忆特征选择存储
3. **自动关联**: 自动发现和建立记忆间关系

### 能力3: 自适应检索 (Adaptive Retrieval)

**目标**: 根据查询特征动态调整检索策略

```rust
/// 自适应检索引擎
pub struct AdaptiveRetrievalEngine {
    /// 可用引擎池
    engine_pool: Vec<Box<dyn RetrievalEngine>>,
    
    /// 路由器（动态选择引擎）
    router: Box<dyn AdaptiveRouter>,
    
    /// 融合器（动态调整融合策略）
    fusion: Box<dyn AdaptiveFusion>,
    
    /// 性能监控
    monitor: PerformanceMonitor,
}

pub trait AdaptiveRouter: Send + Sync {
    /// 根据查询和历史性能选择引擎
    async fn route(
        &self,
        query: &Query,
        engines: &[Box<dyn RetrievalEngine>],
        history: &PerformanceHistory,
    ) -> Result<Vec<usize>>;
}

pub trait AdaptiveFusion: Send + Sync {
    /// 动态调整融合权重
    async fn fuse(
        &self,
        results: Vec<RetrievalResult>,
        query: &Query,
        history: &PerformanceHistory,
    ) -> Result<RetrievalResult>;
}
```

**实现策略**:
1. **性能监控**: 记录每种策略的性能
2. **在线学习**: 根据反馈调整策略
3. **多臂老虎机**: 平衡探索与利用

### 能力4: 持续学习 (Continuous Learning)

**目标**: 从反馈中持续优化系统

```rust
/// 学习框架
pub struct LearningFramework {
    /// 数据收集器
    collector: FeedbackCollector,
    
    /// 学习器池
    learners: Vec<Box<dyn Learner>>,
    
    /// 模型仓库
    model_repo: ModelRepository,
}

pub trait Learner: Send + Sync {
    /// 学习类型
    fn learning_type(&self) -> LearningType;
    
    /// 从数据学习
    async fn learn(&mut self, data: &LearningData) -> Result<Model>;
    
    /// 增量更新
    async fn update(&mut self, feedback: &Feedback) -> Result<()>;
}

pub enum LearningType {
    /// 监督学习
    Supervised,
    
    /// 无监督学习
    Unsupervised,
    
    /// 强化学习
    Reinforcement,
    
    /// 在线学习
    Online,
}

/// 学习数据
pub struct LearningData {
    /// 特征
    features: Vec<Features>,
    
    /// 标签（监督学习）
    labels: Option<Vec<Label>>,
    
    /// 奖励信号（强化学习）
    rewards: Option<Vec<f32>>,
}
```

**实现策略**:
1. **多级学习**: 特征学习、策略学习、端到端学习
2. **增量更新**: 支持在线学习
3. **A/B测试**: 验证学习效果

### 能力5: 可扩展性 (Extensibility)

**目标**: 支持用户自定义扩展

```rust
/// 插件系统
pub struct PluginSystem {
    /// 插件注册表
    registry: PluginRegistry,
    
    /// 插件加载器
    loader: PluginLoader,
}

/// 插件接口
pub trait Plugin: Send + Sync {
    /// 插件元信息
    fn metadata(&self) -> PluginMetadata;
    
    /// 初始化
    async fn initialize(&mut self, config: &Config) -> Result<()>;
    
    /// 插件类型
    fn plugin_type(&self) -> PluginType;
}

pub enum PluginType {
    /// 特征提取器插件
    FeatureExtractor(Box<dyn FeatureExtractor>),
    
    /// 检索引擎插件
    RetrievalEngine(Box<dyn RetrievalEngine>),
    
    /// 评分器插件
    Scorer(Box<dyn Scorer>),
    
    /// 学习器插件
    Learner(Box<dyn Learner>),
}
```

**实现策略**:
1. **插件系统**: 支持动态加载插件
2. **配置驱动**: 通过配置启用/禁用插件
3. **版本管理**: 插件版本控制

---

## 🔄 实施路线图

### Phase 0: 核心抽象重构（4周）

**目标**: 建立新的抽象层，不破坏现有功能

#### Week 1-2: 抽象层定义

**任务**:
1. 定义核心抽象接口（Memory, Query, Retrieval）
2. 实现抽象层到现有实现的适配器
3. 单元测试

**交付物**:
- `agent-mem-abstractions` crate
- 适配器实现
- 测试套件

#### Week 3-4: 管道架构

**任务**:
1. 实现Pipeline框架
2. 重构现有流程为管道模式
3. 集成测试

**交付物**:
- Pipeline框架
- 重构的检索流程
- 性能基准测试

### Phase 1: 能力层构建（6周）

**目标**: 基于新抽象构建5大核心能力

#### Week 5-6: 查询理解能力

**任务**:
1. 实现特征提取框架
2. 实现意图分类器
3. 实现约束推断器

#### Week 7-8: 组织与检索能力

**任务**:
1. 实现智能组织策略
2. 实现自适应检索引擎
3. 实现融合策略

#### Week 9-10: 学习能力

**任务**:
1. 实现学习框架
2. 实现反馈收集
3. 实现在线学习

### Phase 2: 生产化（2周）

**目标**: 性能优化、监控、部署

#### Week 11-12: 优化与部署

**任务**:
1. 性能优化
2. 监控与告警
3. 文档与部署

---

## 📊 成功标准

### 抽象能力评估

| 能力维度 | 评估指标 | 目标 |
|---------|---------|------|
| **泛化能力** | 支持领域数量 | 10+ |
| **扩展能力** | 插件开发工作量 | <1天 |
| **学习能力** | 准确率提升速度 | 每周+5% |
| **性能** | 检索延迟 | <100ms |
| **可维护性** | 代码复杂度 | <10 (圈复杂度) |

### 架构质量评估

1. **依赖清晰度**: 无循环依赖
2. **接口稳定性**: 向后兼容
3. **可测试性**: 单元测试覆盖率>90%
4. **可观测性**: 所有关键路径可追踪
5. **文档完整性**: API文档100%覆盖

---

## 📚 参考架构

### Cursor/Augment 记忆机制

**核心洞察**:
1. **索引优先**: 强大的代码索引能力（AST、符号表、依赖图）
2. **上下文合成**: 智能选择和组合上下文
3. **增量更新**: 高效的增量索引
4. **多层缓存**: 查询缓存、嵌入缓存、结果缓存

**AgentMem如何借鉴**:
- 不复制代码索引功能（非核心）
- 学习其**抽象能力**（如何表示和查询任意结构化信息）
- 学习其**性能优化**（缓存、增量、并行）
- 学习其**用户体验**（快速响应、准确结果）

### Mem0 架构

**核心洞察**:
1. **图记忆**: 基于图的记忆组织
2. **分类系统**: 自动分类记忆
3. **多LLM**: 支持多种LLM
4. **向量存储**: 统一的向量存储接口

**AgentMem如何借鉴**:
- 图记忆已实现，需增强
- 分类系统可借鉴，但要更通用
- 多LLM已支持，继续扩展
- 向量存储已支持14+，继续扩展到30+

---

## 🎯 核心原则总结

### 1. 抽象优于具体

**错误**: 为每种场景写特定代码  
**正确**: 定义通用抽象，场景通过配置或插件实现

### 2. 组合优于继承

**错误**: 复杂的继承层次  
**正确**: 小而美的组件通过组合实现复杂功能

### 3. 配置优于硬编码

**错误**: 在代码中硬编码规则和参数  
**正确**: 所有规则和参数可配置

### 4. 学习优于调优

**错误**: 手动调整参数  
**正确**: 系统自动从反馈中学习最优参数

### 5. 开放优于封闭

**错误**: 系统功能固定  
**正确**: 通过插件系统支持无限扩展

---

**文档版本**: v3.0 (架构级改造)  
**状态**: ✅ 架构设计完成  
**下一步**: 开始Phase 0实施

**核心要点**:
- ✅ 从架构和抽象能力出发
- ✅ 不纠结于具体实现细节
- ✅ 建立通用的能力模型
- ✅ 支持无限扩展和持续学习
