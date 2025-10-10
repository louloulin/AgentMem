# P1-5 任务实施计划 - 实现 RetrievalOrchestrator

**任务编号**: P1-5  
**任务名称**: 实现 RetrievalOrchestrator 多 Agent 协同检索  
**优先级**: P1  
**预计工作量**: 3-4 小时  
**依赖关系**: 无

---

## 任务概述

### 背景分析

通过深度代码分析发现：
1. **当前状态**:
   - `execute_retrieval()` 方法未实现（line 256-265）
   - 返回空的 Vec<RetrievedMemory>
   - TODO 注释："这里需要与各个记忆智能体进行通信"

2. **已有基础设施**:
   - ✅ 5 个 Agent 已实现 (CoreAgent, EpisodicAgent, SemanticAgent, ProceduralAgent, WorkingAgent)
   - ✅ 所有 Agent 实现了 `execute_task()` 方法
   - ✅ 所有 Agent 支持 "search" 操作
   - ✅ RoutingResult 提供了路由决策信息
   - ✅ RetrievalRequest 包含查询参数

3. **实现策略**:
   - 根据 RoutingResult 的 target_memory_types 选择对应的 Agent
   - 调用 Agent 的 execute_task() 方法执行 "search" 操作
   - 聚合多个 Agent 的搜索结果
   - 根据 selected_strategies 和 strategy_weights 计算相关性分数
   - 排序并返回 top-k 结果

### 目标

实现 `execute_retrieval()` 方法，支持多 Agent 协同检索，聚合结果并按相关性排序。

---

## 功能点分解

### 1. Agent 选择和任务分发

**详细说明**: 根据 RoutingResult 的 target_memory_types 选择对应的 Agent，并构造 TaskRequest。

**涉及的 Agent**:
- CoreAgent - MemoryType::Core
- EpisodicAgent - MemoryType::Episodic
- SemanticAgent - MemoryType::Semantic
- ProceduralAgent - MemoryType::Procedural
- WorkingAgent - MemoryType::Working

**TaskRequest 构造**:
```rust
TaskRequest {
    task_id: Uuid::new_v4().to_string(),
    operation: "search".to_string(),
    parameters: serde_json::json!({
        "query": request.query,
        "user_id": "default_user",  // 从 request.context 获取
        "limit": request.max_results
    }),
    priority: 1,
    timeout_ms: Some(5000),
}
```

### 2. 并行执行搜索

**详细说明**: 使用 tokio::join! 或 futures::join_all 并行调用多个 Agent 的 execute_task() 方法。

**优势**:
- 减少总响应时间
- 充分利用异步特性

### 3. 结果聚合和转换

**详细说明**: 将 Agent 返回的 TaskResponse 转换为 RetrievedMemory 结构。

**转换逻辑**:
```rust
// TaskResponse.result 是 JSON Value
// 需要提取 results 数组并转换为 RetrievedMemory
for result in task_response.result["results"].as_array() {
    let memory = RetrievedMemory {
        id: result["id"].as_str().unwrap_or_default().to_string(),
        memory_type: agent_memory_type,
        content: result["content"].as_str().unwrap_or_default().to_string(),
        relevance_score: calculate_relevance(result, strategy),
        source_agent: agent_id.to_string(),
        retrieval_strategy: strategy.clone(),
        metadata: result.as_object().cloned().unwrap_or_default(),
    };
}
```

### 4. 相关性评分

**详细说明**: 根据检索策略和策略权重计算每个结果的相关性分数。

**评分公式**:
```
relevance_score = base_score * strategy_weight * position_penalty
```

**因素**:
- base_score: Agent 返回的原始分数（如果有）
- strategy_weight: RoutingResult.strategy_weights 中的权重
- position_penalty: 结果位置的惩罚因子（越靠后分数越低）

### 5. 结果排序和截断

**详细说明**: 按相关性分数降序排序，返回 top-k 结果。

**实现**:
```rust
memories.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
memories.truncate(request.max_results);
```

---

## 实施步骤

### 步骤 1: 创建 Agent 注册表 (30 分钟)

1. 在 ActiveRetrievalSystem 中添加 agents 字段
2. 创建 HashMap<MemoryType, Arc<RwLock<dyn MemoryAgent>>>
3. 在 new() 方法中初始化所有 Agent
4. 提供 get_agent_for_memory_type() 辅助方法

**代码示例**:
```rust
pub struct ActiveRetrievalSystem {
    // ... 现有字段
    agents: HashMap<MemoryType, Arc<RwLock<dyn MemoryAgent>>>,
}

impl ActiveRetrievalSystem {
    pub async fn new(config: ActiveRetrievalConfig) -> Result<Self> {
        // ... 现有代码
        
        let mut agents = HashMap::new();
        agents.insert(MemoryType::Core, Arc::new(RwLock::new(CoreAgent::new(...))));
        agents.insert(MemoryType::Episodic, Arc::new(RwLock::new(EpisodicAgent::new(...))));
        // ... 其他 Agent
        
        Ok(Self {
            // ... 现有字段
            agents,
        })
    }
}
```

### 步骤 2: 实现 Agent 选择逻辑 (20 分钟)

1. 从 RoutingResult.decision.target_memory_types 获取目标类型
2. 为每个类型查找对应的 Agent
3. 构造 TaskRequest

**代码示例**:
```rust
let target_agents: Vec<_> = routing_result
    .decision
    .target_memory_types
    .iter()
    .filter_map(|memory_type| self.agents.get(memory_type))
    .collect();
```

### 步骤 3: 实现并行搜索 (30 分钟)

1. 为每个 Agent 创建 TaskRequest
2. 使用 futures::join_all 并行执行
3. 处理错误和超时

**代码示例**:
```rust
let search_tasks: Vec<_> = target_agents
    .iter()
    .map(|agent| async {
        let task = TaskRequest {
            task_id: Uuid::new_v4().to_string(),
            operation: "search".to_string(),
            parameters: serde_json::json!({
                "query": request.query,
                "user_id": "default_user",
                "limit": request.max_results
            }),
            priority: 1,
            timeout_ms: Some(5000),
        };
        
        agent.write().await.execute_task(task).await
    })
    .collect();

let results = futures::join_all(search_tasks).await;
```

### 步骤 4: 实现结果转换 (40 分钟)

1. 解析 TaskResponse.result JSON
2. 提取 results 数组
3. 转换为 RetrievedMemory 结构
4. 计算相关性分数

**代码示例**:
```rust
fn convert_to_retrieved_memory(
    task_response: &TaskResponse,
    memory_type: MemoryType,
    agent_id: &str,
    strategy: &RetrievalStrategy,
    strategy_weight: f32,
) -> Vec<RetrievedMemory> {
    let results = task_response.result
        .get("results")
        .and_then(|v| v.as_array())
        .unwrap_or(&vec![]);
    
    results
        .iter()
        .enumerate()
        .map(|(index, result)| {
            let base_score = result.get("score")
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0) as f32;
            
            let position_penalty = 1.0 - (index as f32 * 0.05);
            let relevance_score = base_score * strategy_weight * position_penalty;
            
            RetrievedMemory {
                id: result.get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
                memory_type,
                content: result.get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
                relevance_score,
                source_agent: agent_id.to_string(),
                retrieval_strategy: strategy.clone(),
                metadata: result.as_object()
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|(k, v)| (k, v))
                    .collect(),
            }
        })
        .collect()
}
```

### 步骤 5: 实现结果聚合和排序 (20 分钟)

1. 合并所有 Agent 的结果
2. 按相关性分数降序排序
3. 截断到 max_results

**代码示例**:
```rust
let mut all_memories = Vec::new();

for (result, memory_type, agent_id) in results {
    if let Ok(task_response) = result {
        let strategy = routing_result.decision.selected_strategies.first()
            .cloned()
            .unwrap_or(RetrievalStrategy::StringMatch);
        
        let strategy_weight = routing_result.decision.strategy_weights
            .get(&strategy)
            .copied()
            .unwrap_or(1.0);
        
        let memories = convert_to_retrieved_memory(
            &task_response,
            memory_type,
            &agent_id,
            &strategy,
            strategy_weight,
        );
        
        all_memories.extend(memories);
    }
}

// 排序和截断
all_memories.sort_by(|a, b| {
    b.relevance_score.partial_cmp(&a.relevance_score).unwrap()
});
all_memories.truncate(request.max_results);
```

### 步骤 6: 错误处理和日志 (20 分钟)

1. 处理 Agent 执行失败的情况
2. 添加日志记录
3. 提供降级方案

**代码示例**:
```rust
for (result, memory_type, agent_id) in results {
    match result {
        Ok(task_response) => {
            log::info!("Agent {} returned {} results", agent_id, task_response.result["total_count"]);
            // ... 处理结果
        }
        Err(e) => {
            log::warn!("Agent {} failed: {}", agent_id, e);
            // 继续处理其他 Agent 的结果
        }
    }
}
```

### 步骤 7: 测试验证 (60 分钟)

1. 创建测试文件 `tests/retrieval_orchestrator_test.rs`
2. 测试单个 Agent 检索
3. 测试多个 Agent 并行检索
4. 测试结果聚合和排序
5. 测试错误处理

---

## 验收标准

- [ ] execute_retrieval() 方法实现完成
- [ ] 支持根据 RoutingResult 选择 Agent
- [ ] 支持并行调用多个 Agent
- [ ] 结果正确转换为 RetrievedMemory 结构
- [ ] 相关性评分逻辑正确
- [ ] 结果按相关性降序排序
- [ ] 支持 max_results 截断
- [ ] 错误处理完善（Agent 失败不影响其他 Agent）
- [ ] 添加日志记录
- [ ] 所有测试通过
- [ ] 代码编译无错误

---

## 技术细节

### 1. Agent 接口

所有 Agent 实现了 `MemoryAgent` trait:
```rust
#[async_trait]
pub trait MemoryAgent: Send + Sync {
    async fn execute_task(&mut self, task: TaskRequest) -> CoordinationResult<TaskResponse>;
}
```

### 2. TaskRequest 结构

```rust
pub struct TaskRequest {
    pub task_id: String,
    pub operation: String,  // "search"
    pub parameters: Value,  // JSON 参数
    pub priority: u8,
    pub timeout_ms: Option<u64>,
}
```

### 3. TaskResponse 结构

```rust
pub struct TaskResponse {
    pub task_id: String,
    pub success: bool,
    pub result: Value,  // JSON 结果
    pub error: Option<String>,
    pub execution_time_ms: u64,
}
```

### 4. 搜索参数格式

不同 Agent 的搜索参数略有不同：

**CoreAgent**:
```json
{
    "query": "search text",
    "user_id": "user-123",
    "block_type": "optional"
}
```

**EpisodicAgent**:
```json
{
    "query": "search text",
    "user_id": "user-123",
    "limit": 10
}
```

**SemanticAgent**:
```json
{
    "user_id": "user-123",
    "name_query": "search text",
    "limit": 10
}
```

---

## 风险和注意事项

### 风险 1: Agent 未初始化

**问题**: Agent 可能未初始化，调用 execute_task() 会失败

**解决方案**: 
- 在 ActiveRetrievalSystem::new() 中初始化所有 Agent
- 调用 agent.initialize().await

### 风险 2: 并发访问冲突

**问题**: 多个检索请求同时访问同一个 Agent

**解决方案**:
- 使用 Arc<RwLock<dyn MemoryAgent>> 保护 Agent
- 读锁允许并发读取

### 风险 3: 结果格式不一致

**问题**: 不同 Agent 返回的 JSON 格式可能不同

**解决方案**:
- 使用 .get().and_then() 安全提取字段
- 提供默认值
- 添加日志记录异常情况

---

## 预期结果

完成后，RetrievalOrchestrator 将支持：
- ✅ 多 Agent 协同检索
- ✅ 并行执行提高性能
- ✅ 智能结果聚合和排序
- ✅ 灵活的策略权重配置
- ✅ 完善的错误处理

---

## 参考资料

- Agent 实现: `crates/agent-mem-core/src/agents/*.rs`
- MemoryAgent trait: `crates/agent-mem-core/src/agents/mod.rs`
- RoutingResult: `crates/agent-mem-core/src/retrieval/router.rs`
- RetrievalRequest/Response: `crates/agent-mem-core/src/retrieval/mod.rs`

