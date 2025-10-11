# Phase 1.1: 智能功能集成实施计划

**开始日期**: 2025-10-08  
**预计完成**: 2025-10-15 (7 天)  
**负责人**: AgentMem Team

---

## 📋 任务概览

### 目标
将已实现的智能功能（FactExtractor, DecisionEngine, MemoryDeduplicator）集成到 `MemoryManager::add_memory()` 主流程中。

### 成功指标
- ✅ 智能提取准确率 > 90%
- ✅ 决策准确率 > 80%
- ✅ 去重准确率 > 85%
- ✅ 性能 P95 < 500ms
- ✅ 测试覆盖率 > 80%

---

## 📅 Day 1-2: 集成 FactExtractor (2025-10-08 ~ 2025-10-09)

### 任务清单

#### 1. 更新 IntelligenceConfig
- [ ] 添加 `enable_intelligent_extraction` 字段
- [ ] 添加 `enable_decision_engine` 字段
- [ ] 添加 `enable_deduplication` 字段
- [ ] 添加 `fact_extraction_config` 子配置
- [ ] 更新默认值

**文件**: `crates/agent-mem-config/src/memory.rs`

#### 2. 更新 MemoryManager 结构
- [ ] 添加 `fact_extractor: Option<Arc<FactExtractor>>` 字段
- [ ] 添加 `decision_engine: Option<Arc<MemoryDecisionEngine>>` 字段
- [ ] 添加 `deduplicator: Option<Arc<MemoryDeduplicator>>` 字段
- [ ] 更新构造函数初始化逻辑

**文件**: `crates/agent-mem-core/src/manager.rs`

#### 3. 实现智能 add_memory 流程
- [ ] 步骤 1: 检查是否启用智能提取
- [ ] 步骤 2: 调用 FactExtractor 提取事实
- [ ] 步骤 3: 对每个事实调用 DecisionEngine
- [ ] 步骤 4: 根据决策执行操作 (ADD/UPDATE/DELETE/MERGE)
- [ ] 步骤 5: 可选去重检查
- [ ] 步骤 6: 记录历史和生命周期

**文件**: `crates/agent-mem-core/src/manager.rs`

#### 4. 添加辅助方法
- [ ] `extract_facts_from_content()` - 提取事实
- [ ] `make_decision_for_fact()` - 决策
- [ ] `execute_memory_action()` - 执行操作
- [ ] `find_similar_memories()` - 查找相似记忆

**文件**: `crates/agent-mem-core/src/manager.rs`

---

## 📅 Day 3-4: 集成 DecisionEngine (2025-10-10 ~ 2025-10-11)

### 任务清单

#### 1. 实现决策逻辑
- [ ] ADD 决策: 直接添加新记忆
- [ ] UPDATE 决策: 更新现有记忆
- [ ] DELETE 决策: 删除过时记忆
- [ ] MERGE 决策: 合并重复记忆
- [ ] NoAction 决策: 跳过操作

#### 2. 实现合并策略
- [ ] Replace: 完全替换
- [ ] Append: 追加信息
- [ ] Merge: 智能合并
- [ ] Prioritize: 优先保留重要信息

#### 3. 错误处理
- [ ] LLM 调用失败降级
- [ ] 部分失败继续处理
- [ ] 详细错误日志

---

## 📅 Day 5: 配置和优化 (2025-10-12)

### 任务清单

#### 1. 配置默认值
- [ ] 设置 `enable_intelligent_extraction = true`
- [ ] 设置 `enable_decision_engine = true`
- [ ] 设置 `enable_deduplication = false` (可选)
- [ ] 配置相似度阈值

#### 2. 性能优化
- [ ] 添加缓存机制
- [ ] 批量处理优化
- [ ] 并发控制

#### 3. 可观测性
- [ ] 添加 Prometheus 指标
- [ ] 添加结构化日志
- [ ] 添加性能追踪

---

## 📅 Day 6-7: 测试和文档 (2025-10-13 ~ 2025-10-14)

### 任务清单

#### 1. 单元测试
- [ ] test_intelligent_extraction_enabled
- [ ] test_intelligent_extraction_disabled
- [ ] test_add_decision
- [ ] test_update_decision
- [ ] test_delete_decision
- [ ] test_merge_decision
- [ ] test_no_action_decision
- [ ] test_llm_failure_fallback
- [ ] test_performance_benchmark

**文件**: `crates/agent-mem-core/tests/intelligent_integration_test.rs`

#### 2. 集成测试
- [ ] test_end_to_end_intelligent_flow
- [ ] test_multiple_facts_extraction
- [ ] test_conflict_resolution
- [ ] test_deduplication_integration

**文件**: `crates/agent-mem-core/tests/integration_test.rs`

#### 3. 文档
- [ ] 更新 README.md
- [ ] 编写集成指南
- [ ] 编写配置文档
- [ ] 编写 API 文档
- [ ] 添加示例代码

**文件**: `docs/intelligent_integration.md`

---

## 🔧 技术实施细节

### 1. IntelligenceConfig 扩展

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceConfig {
    // 现有字段
    pub similarity_threshold: f32,
    pub clustering_threshold: f32,
    pub enable_conflict_detection: bool,
    pub enable_memory_summarization: bool,
    pub importance_scoring: bool,
    
    // 新增字段
    pub enable_intelligent_extraction: bool,
    pub enable_decision_engine: bool,
    pub enable_deduplication: bool,
    pub fact_extraction_config: FactExtractionConfig,
    pub decision_engine_config: DecisionEngineConfig,
}
```

### 2. MemoryManager 扩展

```rust
pub struct MemoryManager {
    operations: Arc<RwLock<Box<dyn MemoryOperations + Send + Sync>>>,
    lifecycle: Arc<RwLock<MemoryLifecycle>>,
    history: Arc<RwLock<MemoryHistory>>,
    config: MemoryConfig,
    
    // 新增智能组件
    fact_extractor: Option<Arc<FactExtractor>>,
    decision_engine: Option<Arc<MemoryDecisionEngine>>,
    deduplicator: Option<Arc<MemoryDeduplicator>>,
    llm_provider: Option<Arc<dyn LLMProvider>>,
}
```

### 3. 智能 add_memory 流程

```rust
pub async fn add_memory(
    &self,
    agent_id: String,
    user_id: Option<String>,
    content: String,
    memory_type: Option<MemoryType>,
    importance: Option<f32>,
    metadata: Option<HashMap<String, String>>,
) -> Result<String> {
    // 1. 检查是否启用智能提取
    if self.config.intelligence.enable_intelligent_extraction {
        return self.add_memory_intelligent(
            agent_id, user_id, content, memory_type, importance, metadata
        ).await;
    }
    
    // 2. 降级到原始流程
    self.add_memory_simple(
        agent_id, user_id, content, memory_type, importance, metadata
    ).await
}

async fn add_memory_intelligent(
    &self,
    agent_id: String,
    user_id: Option<String>,
    content: String,
    memory_type: Option<MemoryType>,
    importance: Option<f32>,
    metadata: Option<HashMap<String, String>>,
) -> Result<String> {
    // 1. 提取事实
    let facts = self.extract_facts(&content).await?;
    
    // 2. 对每个事实做决策
    let mut memory_ids = Vec::new();
    for fact in facts {
        // 查找相似记忆
        let similar_memories = self.find_similar_memories(&fact, &agent_id).await?;
        
        // 决策
        let decision = self.make_decision(&fact, &similar_memories).await?;
        
        // 执行
        let memory_id = self.execute_decision(decision, &agent_id, &user_id).await?;
        if let Some(id) = memory_id {
            memory_ids.push(id);
        }
    }
    
    // 3. 返回主记忆ID
    Ok(memory_ids.first().cloned().unwrap_or_default())
}
```

---

## 📊 进度追踪

| 任务 | 状态 | 完成日期 | 备注 |
|------|------|---------|------|
| Day 1-2: FactExtractor 集成 | ⏳ 进行中 | - | - |
| Day 3-4: DecisionEngine 集成 | ⏳ 待开始 | - | - |
| Day 5: 配置和优化 | ⏳ 待开始 | - | - |
| Day 6-7: 测试和文档 | ⏳ 待开始 | - | - |

---

## 🎯 下一步行动

1. **立即开始**: 更新 IntelligenceConfig
2. **然后**: 更新 MemoryManager 结构
3. **最后**: 实现智能 add_memory 流程

**预计今天完成**: Day 1 的所有任务

