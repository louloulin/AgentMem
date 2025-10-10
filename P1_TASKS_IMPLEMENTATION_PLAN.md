# P1 任务实施计划

**日期**: 2025-01-10  
**总工作量**: 7-9 小时  
**目标**: 完成剩余 4% 的 P1 优化任务，将完成度从 96% 提升到 98%

---

## 任务概览

| 任务 | 优先级 | 工作量 | 依赖 | 状态 |
|------|--------|--------|------|------|
| 1. 为 EpisodicAgent 和 SemanticAgent 创建测试 | P1 | 1-2 小时 | 无 | 未开始 |
| 2. 修复 organization_id 硬编码 | P1 | 1 小时 | 无 | 未开始 |
| 3. 更新数据库 schema 添加缺失字段 | P1 | 1-2 小时 | 无 | 未开始 |
| 4. 实现 RetrievalOrchestrator | P1 | 3-4 小时 | 无 | 未开始 |

---

## 任务 1: 为 EpisodicAgent 和 SemanticAgent 创建测试

### 基本信息
- **优先级**: P1
- **工作量**: 1-2 小时
- **依赖**: 无
- **影响**: 提高测试覆盖率从 8/10 到 10/10

### 功能点

1. 创建 `episodic_agent_real_storage_test.rs`
2. 实现 MockEpisodicStore
3. 编写 4-5 个测试用例
4. 创建 `semantic_agent_real_storage_test.rs`
5. 实现 MockSemanticStore
6. 编写 4-5 个测试用例

### 实施步骤

#### 步骤 1: 创建 EpisodicAgent 测试文件

1. 参考 `core_agent_real_storage_test.rs` 和 `procedural_agent_real_storage_test.rs`
2. 创建 `crates/agent-mem-core/tests/episodic_agent_real_storage_test.rs`
3. 导入必要的依赖

#### 步骤 2: 实现 MockEpisodicStore

1. 查看 `agent-mem-traits/src/memory_store.rs` 中的 `EpisodicMemoryStore` trait
2. 实现所有 trait 方法：
   - `create_event()`
   - `get_event()`
   - `query_events()`
   - `update_importance()`
   - `delete_event()`
   - `get_events_by_time_range()`
   - `get_recent_events()`
   - `search_events()`
3. 使用 `Arc<Mutex<HashMap<String, EpisodicEvent>>>` 存储数据

#### 步骤 3: 编写 EpisodicAgent 测试用例

1. `test_episodic_agent_insert_with_real_store` - 验证 create_event
2. `test_episodic_agent_search_with_real_store` - 验证 query_events
3. `test_episodic_agent_update_with_real_store` - 验证 update_importance
4. `test_episodic_agent_delete_with_real_store` - 验证 delete_event
5. `test_episodic_agent_time_range_query_with_real_store` - 验证 get_events_by_time_range

#### 步骤 4: 创建 SemanticAgent 测试文件

1. 参考 EpisodicAgent 测试
2. 创建 `crates/agent-mem-core/tests/semantic_agent_real_storage_test.rs`

#### 步骤 5: 实现 MockSemanticStore

1. 查看 `SemanticMemoryStore` trait
2. 实现所有 trait 方法：
   - `create_item()`
   - `get_item()`
   - `query_items()`
   - `update_item()`
   - `delete_item()`
   - `search_by_embedding()`
3. 使用 `Arc<Mutex<HashMap<String, SemanticMemoryItem>>>` 存储数据

#### 步骤 6: 编写 SemanticAgent 测试用例

1. `test_semantic_agent_insert_with_real_store` - 验证 create_item
2. `test_semantic_agent_search_with_real_store` - 验证 query_items
3. `test_semantic_agent_update_with_real_store` - 验证 update_item
4. `test_semantic_agent_delete_with_real_store` - 验证 delete_item

#### 步骤 7: 运行测试

```bash
cargo test --package agent-mem-core --test episodic_agent_real_storage_test -- --nocapture
cargo test --package agent-mem-core --test semantic_agent_real_storage_test -- --nocapture
```

### 验收标准

- [ ] `episodic_agent_real_storage_test.rs` 创建完成
- [ ] MockEpisodicStore 实现所有 trait 方法
- [ ] 至少 4 个 EpisodicAgent 测试通过
- [ ] `semantic_agent_real_storage_test.rs` 创建完成
- [ ] MockSemanticStore 实现所有 trait 方法
- [ ] 至少 4 个 SemanticAgent 测试通过
- [ ] 所有测试验证数据真正存储（不只是 API 响应）

---

## 任务 2: 修复 organization_id 硬编码

### 基本信息
- **优先级**: P1
- **工作量**: 1 小时
- **依赖**: 无
- **影响**: 支持多租户场景

### 功能点

1. 在 ChatRequest 中添加 `organization_id` 字段
2. 在 create_user_message() 中从 request 获取 organization_id
3. 在 create_assistant_message() 中从 request 获取 organization_id
4. 更新相关测试

### 实施步骤

#### 步骤 1: 修改 ChatRequest 结构

**文件**: `crates/agent-mem-core/src/types.rs`

```rust
pub struct ChatRequest {
    pub user_id: String,
    pub agent_id: String,
    pub message: String,
    pub organization_id: Option<String>, // 新增字段
    // ... 其他字段
}
```

#### 步骤 2: 修改 create_user_message()

**文件**: `crates/agent-mem-core/src/orchestrator/mod.rs:353-370`

```rust
let message = DbMessage {
    id: Uuid::new_v4().to_string(),
    organization_id: request.organization_id.clone().unwrap_or_else(|| "default".to_string()),
    user_id: request.user_id.clone(),
    // ...
};
```

#### 步骤 3: 修改 create_assistant_message()

**文件**: `crates/agent-mem-core/src/orchestrator/mod.rs:396-410`

```rust
let message = DbMessage {
    id: Uuid::new_v4().to_string(),
    organization_id: request.organization_id.clone().unwrap_or_else(|| "default".to_string()),
    user_id: request.user_id.clone(),
    // ...
};
```

#### 步骤 4: 更新测试

更新所有创建 ChatRequest 的测试，添加 `organization_id: Some("test-org".to_string())`

#### 步骤 5: 编译和测试

```bash
cargo build --package agent-mem-core
cargo test --package agent-mem-core
```

### 验收标准

- [ ] ChatRequest 添加 organization_id 字段
- [ ] create_user_message() 使用 request.organization_id
- [ ] create_assistant_message() 使用 request.organization_id
- [ ] 所有测试通过
- [ ] 编译无错误

---

## 任务 3: 更新数据库 schema 添加缺失字段

### 基本信息
- **优先级**: P1
- **工作量**: 1-2 小时
- **依赖**: 无
- **影响**: 支持向量搜索、记忆过期、乐观锁

### 功能点

1. 检查当前数据库 schema
2. 创建新的迁移脚本添加缺失字段
3. 更新 PostgreSQL 存储实现
4. 更新 LibSQL 存储实现
5. 更新测试数据

### 缺失字段

| 字段 | 类型 | 说明 |
|------|------|------|
| agent_id | VARCHAR(255) | Agent 标识符 |
| user_id | VARCHAR(255) | 用户标识符 |
| embedding | VECTOR(1536) | 向量嵌入 |
| expires_at | TIMESTAMP | 过期时间 |
| version | INTEGER | 版本号（乐观锁） |

### 实施步骤

#### 步骤 1: 检查当前 schema

查看 `crates/agent-mem-storage/migrations/` 目录

#### 步骤 2: 创建迁移脚本

**文件**: `crates/agent-mem-storage/migrations/postgres/006_add_missing_fields.sql`

```sql
-- 添加缺失字段到 memories 表
ALTER TABLE memories ADD COLUMN IF NOT EXISTS agent_id VARCHAR(255);
ALTER TABLE memories ADD COLUMN IF NOT EXISTS user_id VARCHAR(255);
ALTER TABLE memories ADD COLUMN IF NOT EXISTS embedding VECTOR(1536);
ALTER TABLE memories ADD COLUMN IF NOT EXISTS expires_at TIMESTAMP;
ALTER TABLE memories ADD COLUMN IF NOT EXISTS version INTEGER DEFAULT 1;

-- 添加索引
CREATE INDEX IF NOT EXISTS idx_memories_agent_id ON memories(agent_id);
CREATE INDEX IF NOT EXISTS idx_memories_user_id ON memories(user_id);
CREATE INDEX IF NOT EXISTS idx_memories_expires_at ON memories(expires_at);
```

#### 步骤 3: 更新 PostgreSQL 存储实现

**文件**: `crates/agent-mem-core/src/storage/postgres.rs:100-126`

```rust
let memory = crate::types::Memory {
    id: row.try_get("id")?,
    agent_id: row.try_get("agent_id").unwrap_or_else(|_| "default".to_string()),
    user_id: row.try_get("user_id").ok(),
    memory_type,
    content: row.try_get("content")?,
    importance: row.try_get("importance")?,
    embedding: row.try_get("embedding").ok(),
    created_at: created_at.timestamp(),
    last_accessed_at: last_accessed.map(|dt| dt.timestamp()).unwrap_or_else(|| chrono::Utc::now().timestamp()),
    access_count: row.try_get::<i64, _>("access_count").map(|v| v as u32).unwrap_or(0),
    expires_at: row.try_get::<Option<DateTime<Utc>>, _>("expires_at").ok().flatten().map(|dt| dt.timestamp()),
    metadata: metadata_map,
    version: row.try_get("version").unwrap_or(1),
};
```

#### 步骤 4: 更新 LibSQL 存储实现

类似地更新 LibSQL 存储实现

#### 步骤 5: 运行迁移和测试

```bash
# 运行迁移
psql -U postgres -d agentmem -f crates/agent-mem-storage/migrations/postgres/006_add_missing_fields.sql

# 运行测试
cargo test --package agent-mem-core
```

### 验收标准

- [ ] 迁移脚本创建完成
- [ ] PostgreSQL 存储实现更新
- [ ] LibSQL 存储实现更新
- [ ] 所有测试通过
- [ ] 编译无错误

---

## 任务 4: 实现 RetrievalOrchestrator

### 基本信息
- **优先级**: P1
- **工作量**: 3-4 小时
- **依赖**: 无
- **影响**: 支持多 Agent 协同检索

### 功能点

1. 设计 Agent 间通信机制
2. 实现 execute_retrieval() 方法
3. 调用各个 Agent 的 search 方法
4. 合并和排序检索结果
5. 实现结果去重和排名
6. 编写测试验证

### 实施步骤

#### 步骤 1: 设计 Agent 间通信机制

**方案**: 使用 TaskRequest/TaskResponse 模式

```rust
// 在 ActiveRetrievalSystem 中添加 Agent 引用
pub struct ActiveRetrievalSystem {
    // ... 现有字段
    core_agent: Option<Arc<CoreAgent>>,
    episodic_agent: Option<Arc<EpisodicAgent>>,
    semantic_agent: Option<Arc<SemanticAgent>>,
    procedural_agent: Option<Arc<ProceduralAgent>>,
    working_agent: Option<Arc<WorkingAgent>>,
}
```

#### 步骤 2: 实现 execute_retrieval()

**文件**: `crates/agent-mem-core/src/retrieval/mod.rs:256-265`

```rust
async fn execute_retrieval(
    &self,
    request: &RetrievalRequest,
    routing_result: &RoutingResult,
) -> Result<Vec<RetrievedMemory>> {
    let mut all_memories = Vec::new();
    
    // 根据路由决策调用相应的 Agent
    for strategy in &routing_result.decision.selected_strategies {
        match strategy {
            RetrievalStrategy::Episodic => {
                if let Some(agent) = &self.episodic_agent {
                    let memories = self.retrieve_from_episodic(agent, request).await?;
                    all_memories.extend(memories);
                }
            }
            RetrievalStrategy::Semantic => {
                if let Some(agent) = &self.semantic_agent {
                    let memories = self.retrieve_from_semantic(agent, request).await?;
                    all_memories.extend(memories);
                }
            }
            // ... 其他策略
        }
    }
    
    // 去重和排序
    self.deduplicate_and_rank(all_memories, request.max_results)
}
```

#### 步骤 3: 实现各个 Agent 的检索方法

```rust
async fn retrieve_from_episodic(
    &self,
    agent: &Arc<EpisodicAgent>,
    request: &RetrievalRequest,
) -> Result<Vec<RetrievedMemory>> {
    let task = TaskRequest {
        task_id: Uuid::new_v4().to_string(),
        memory_type: MemoryType::Episodic,
        operation: "search".to_string(),
        parameters: json!({
            "query": request.query,
            "limit": request.max_results,
        }),
        priority: 1,
        timeout: None,
        retry_count: 0,
    };
    
    let response = agent.execute_task(task).await?;
    
    // 转换为 RetrievedMemory
    self.convert_to_retrieved_memories(response, MemoryType::Episodic, RetrievalStrategy::Episodic)
}
```

#### 步骤 4: 实现去重和排名

```rust
fn deduplicate_and_rank(
    &self,
    mut memories: Vec<RetrievedMemory>,
    max_results: usize,
) -> Result<Vec<RetrievedMemory>> {
    // 按 ID 去重
    let mut seen = std::collections::HashSet::new();
    memories.retain(|m| seen.insert(m.id.clone()));
    
    // 按相关性分数排序
    memories.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
    
    // 限制结果数量
    memories.truncate(max_results);
    
    Ok(memories)
}
```

#### 步骤 5: 编写测试

创建 `crates/agent-mem-core/tests/retrieval_orchestrator_test.rs`

#### 步骤 6: 运行测试

```bash
cargo test --package agent-mem-core --test retrieval_orchestrator_test -- --nocapture
```

### 验收标准

- [ ] execute_retrieval() 实现完成
- [ ] 支持调用所有 5 个 Agent
- [ ] 实现结果去重和排名
- [ ] 至少 3 个测试通过
- [ ] 编译无错误

---

## 完成后的预期状态

### 完成度更新

**当前**: 96%  
**完成后**: **98%**

### 测试覆盖更新

**当前**: 15/15 tests (8/10 评分)  
**完成后**: 25+/25+ tests (10/10 评分)

### 功能完整性

- ✅ 所有 Agent 有测试覆盖
- ✅ 支持多租户
- ✅ 数据库 schema 完整
- ✅ 高级检索功能可用

---

## 实施顺序建议

**推荐顺序**:
1. 任务 1 (1-2 小时) - 提高测试覆盖率
2. 任务 2 (1 小时) - 修复 organization_id
3. 任务 3 (1-2 小时) - 更新数据库 schema
4. 任务 4 (3-4 小时) - 实现 RetrievalOrchestrator

**总工作量**: 7-9 小时

---

## 文档更新

完成后需要更新的文档：

1. `mem14.1.md` - 标记 P1 任务完成
2. `PRODUCTION_ROADMAP_FINAL.md` - 更新进度到 98%
3. 创建 `P1_TASKS_COMPLETION_REPORT.md` - 详细完成报告

---

**结论**: 完成这 4 个 P1 任务后，AgentMem 将达到 98% 完成度，测试覆盖率 10/10，功能更加完整，更适合生产环境使用。

