# AgentMem 记忆系统 - 最小改造方案

**重大发现**: AgentMem **已经实现了Session管理功能**！

## 🎯 现有功能分析

### ✅ 已实现的功能 (P1阶段)

#### 1. MemoryScope 枚举 (types.rs:106-223)

```rust
pub enum MemoryScope {
    Global,                                           // 全局共享
    Organization { org_id: String },                  // 组织级
    User { user_id: String },                         // 用户级
    Agent { user_id: String, agent_id: String },      // Agent级
    Run { user_id: String, run_id: String },          // 临时会话 ✅
    Session { user_id: String, session_id: String },  // 会话级 ✅✅
}
```

**优先级**: `Session > Run > Agent > Organization > User > Global`

#### 2. Session 使用方式

```rust
// 方式1: 通过 metadata 传递 session_id
let mut metadata = HashMap::new();
metadata.insert("session_id".to_string(), "window-1".to_string());

let options = AddMemoryOptions {
    user_id: Some("alice".to_string()),
    metadata,
    ..Default::default()
};

// 方式2: 直接使用 MemoryScope
let scope = MemoryScope::Session {
    user_id: "alice".to_string(),
    session_id: "window-1".to_string(),
};
mem.add_with_scope("Current conversation", scope).await?;
```

#### 3. Working Memory vs Long-term Memory

**当前实现**:
- `Run` scope = Working Memory（临时会话，可选择是否持久化）
- `Session` scope = Session Memory（会话级，持久化）
- `Agent` scope = Long-term Memory（用户+Agent，长期持久化）

**区别**:
```
Run:     临时ID，会话结束可清理
Session: 持久ID，跨会话保持
Agent:   无session隔离，所有历史记忆
```

---

## ❌ 问题诊断

### 问题1: user_id 被覆盖为 "default" ⭐⭐⭐⭐⭐

**根本原因**: Memory API 的 `default_user_id` 机制

```rust
// memory.rs:228
options.user_id.or_else(|| self.default_user_id.clone())
```

**问题场景**:
1. Memory builder 设置了 `default_user_id = Some("default")`
2. 即使 `options.user_id = Some("real_user_id")`，由于 `.or_else()` 逻辑，`Some("default")` 不会被替换
3. 数据库最终存储 `user_id = "default"`

**证据**:
- 日志: 传入 `user_id="zhipu_test_user_83533"`
- 数据库: 存储 `user_id="default"`
- 查询结果: 0 条记忆

---

### 问题2: LumosAI 未使用 Session 功能 ⭐⭐⭐

**当前实现** (memory_adapter.rs:57-63):
```rust
let options = AddMemoryOptions {
    agent_id: Some(self.agent_id.clone()),
    user_id: Some(self.user_id.clone()),
    metadata,
    infer: false,
    ..Default::default()
};
// ❌ 未设置 session_id
```

**结果**: 所有对话都混在一起，无会话隔离

---

### 问题3: 每次请求重新创建 Agent ⭐⭐

**当前流程** (chat_lumosai.rs):
```
每次 HTTP 请求:
1. 创建新的 LumosAgent 实例
2. 创建新的 AgentMemBackend
3. 执行 generate()
4. 丢弃 Agent 实例
```

**问题**: 
- 无法维护 Working Memory（内存中的临时记忆）
- 每次都从数据库检索，性能低

---

## 🔧 最小改造方案

### 方案A: 使用 Session Scope (推荐) ⭐⭐⭐⭐⭐

**改造点1**: 在 Chat API 中传递 session_id

```rust
// chat_lumosai.rs
#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub user_id: String,
    pub session_id: Option<String>,  // 🆕 新增
}

// 使用 session_id 创建 Agent
let session_id = req.session_id.unwrap_or_else(|| 
    format!("session-{}", uuid::Uuid::new_v4())
);
```

**改造点2**: AgentMemBackend 支持 session_id

```rust
// memory_adapter.rs
pub struct AgentMemBackend {
    memory_api: Arc<Memory>,
    agent_id: String,
    user_id: String,
    session_id: String,  // 🆕 新增
}

impl AgentMemBackend {
    async fn store(&self, message: &Message) -> Result<()> {
        let mut metadata = HashMap::new();
        metadata.insert("role".to_string(), role_str.to_string());
        metadata.insert("session_id".to_string(), self.session_id.clone());  // 🆕
        
        let options = AddMemoryOptions {
            agent_id: Some(self.agent_id.clone()),
            user_id: Some(self.user_id.clone()),
            metadata,
            ..Default::default()
        };
        // ✅ 自动识别为 Session scope
    }
    
    async fn retrieve(&self, config: &MemoryConfig) -> Result<Vec<Message>> {
        // 方式1: 使用 get_all 并过滤 session_id
        let mut options = GetAllOptions {
            agent_id: Some(self.agent_id.clone()),
            user_id: Some(self.user_id.clone()),
            limit: Some(config.last_messages.unwrap_or(10) * 2),  // 多获取一些
            ..Default::default()
        };
        
        let all_memories = self.memory_api.get_all(options).await?;
        
        // 过滤当前 session
        let session_memories: Vec<_> = all_memories
            .into_iter()
            .filter(|mem| {
                mem.metadata
                    .get("session_id")
                    .and_then(|v| v.as_str())
                    .map(|sid| sid == self.session_id)
                    .unwrap_or(false)
            })
            .take(config.last_messages.unwrap_or(10))
            .collect();
        
        // 转换为 LumosMessage
        Ok(convert_to_messages(session_memories))
    }
}
```

**代码量**: ~30 行修改

**优点**:
- ✅ 复用现有 Session 功能
- ✅ 最小改造
- ✅ 会话隔离
- ✅ 支持多窗口对话

**缺点**:
- ⚠️  需要前端传递 session_id
- ⚠️  检索时需要额外过滤

---

### 方案B: 使用 Run Scope (临时会话)

**适用场景**: 不需要持久化会话历史

```rust
let run_id = format!("run-{}", uuid::Uuid::new_v4());

let options = AddMemoryOptions {
    user_id: Some(self.user_id.clone()),
    agent_id: Some(self.agent_id.clone()),
    run_id: Some(run_id),  // 🆕 使用 run_id
    ..Default::default()
};
```

**优点**: 
- ✅ 更简单
- ✅ 自动隔离

**缺点**:
- ❌ 每次请求不同 run_id，无法跨请求保持会话

---

### 方案C: 修复 default_user_id (必须) ⭐⭐⭐⭐⭐

**问题**: Memory 初始化设置了 default_user_id

**解决方案1**: 不设置默认值

```rust
// agent-mem-server/src/routes/memory.rs
let memory = Memory::builder()
    .with_storage(&db_path)
    .with_embedder(provider, model)
    // ❌ 不设置 default_user_id 和 default_agent_id
    .build()
    .await?;
```

**解决方案2**: 修改 Memory API 逻辑

```rust
// agent-mem/src/memory.rs:228
// 修改前:
options.user_id.or_else(|| self.default_user_id.clone())

// 修改后:
if options.user_id.is_none() && self.default_user_id.is_some() {
    options.user_id = self.default_user_id.clone();
    warn!("Using default_user_id because options.user_id was None");
}
options.user_id  // 返回原始值或默认值
```

**推荐**: 方案1（更简单，更安全）

---

## 🚀 实施步骤

### Phase 0: 紧急修复 (30分钟)

**目标**: 让记忆功能可用

- [x] 分析现有 Session 功能
- [x] 制定最小改造方案
- [ ] **Step 1**: 修复 default_user_id 问题
  ```bash
  # 修改 agent-mem-server/src/routes/memory.rs
  # 不设置 default_user_id 和 default_agent_id
  
  # 重新编译
  cargo build --release --package agent-mem-server --features lumosai
  
  # 重启服务器
  pkill agent-mem-server && ./start_server_no_auth.sh
  ```

- [ ] **Step 2**: 验证 user_id 正确存储
  ```bash
  # 测试对话
  export ZHIPU_API_KEY='...'
  ./test_zhipu_memory.sh
  
  # 检查数据库
  sqlite3 ./data/agentmem.db \
    "SELECT user_id, COUNT(*) FROM memories 
     WHERE created_at > datetime('now', '-5 minutes') 
     GROUP BY user_id;"
  # 应该显示实际的 user_id，而非 "default"
  ```

**验收**: 
- ✅ 记忆存储时 user_id 正确
- ✅ 检索能返回记忆（即使还未完美隔离）

---

### Phase 1: Session 支持 (2小时)

**目标**: 实现会话隔离

- [ ] **Task 1.1**: 修改 ChatRequest 添加 session_id
  ```rust
  // agent-mem-server/src/routes/chat_lumosai.rs
  #[derive(Debug, Deserialize)]
  pub struct ChatRequest {
      pub message: String,
      pub user_id: String,
      pub session_id: Option<String>,  // 🆕
  }
  ```

- [ ] **Task 1.2**: AgentMemBackend 支持 session_id
  ```rust
  // agent-mem-lumosai/src/memory_adapter.rs
  pub struct AgentMemBackend {
      session_id: String,  // 🆕
      // ...
  }
  
  // store() 中添加 session_id 到 metadata
  metadata.insert("session_id".to_string(), self.session_id.clone());
  
  // retrieve() 中过滤 session_id
  ```

- [ ] **Task 1.3**: Agent Factory 传递 session_id
  ```rust
  // agent-mem-lumosai/src/agent_factory.rs
  pub async fn create_chat_agent(
      &self,
      agent: &Agent,
      user_id: &str,
      session_id: &str,  // 🆕
  ) -> anyhow::Result<Arc<BasicAgent>> {
      let memory_backend = Arc::new(AgentMemBackend::new(
          self.memory_api.clone(),
          agent.id.clone(),
          user_id.to_string(),
          session_id.to_string(),  // 🆕
      ));
      // ...
  }
  ```

- [ ] **Task 1.4**: 端到端测试
  ```bash
  # 测试脚本添加 session_id
  curl -X POST http://localhost:8080/api/v1/agents/$AGENT_ID/chat/lumosai \
    -H "Content-Type: application/json" \
    -d '{"message":"你好","user_id":"user1","session_id":"session-abc"}'
  
  # 验证 session 隔离
  # Session A: user1 + session-abc
  # Session B: user1 + session-xyz
  # 两个 session 的记忆应该互不干扰
  ```

**验收**:
- ✅ 不同 session_id 的对话互不干扰
- ✅ 相同 session_id 的对话能记住历史
- ✅ AI 能正确使用会话内的记忆

---

### Phase 2: 混合检索优化 (1天)

**目标**: 提升检索准确率

- [ ] **Task 2.1**: 实现混合检索
  ```rust
  async fn retrieve(&self, config: &MemoryConfig) -> Result<Vec<Message>> {
      // 1. 当前 session 的最近消息（保证连贯性）
      let recent = self.get_session_recent(5).await?;
      
      // 2. 当前 session 的语义相关消息
      let semantic = if let Some(query) = &config.query {
          self.search_session(query, 5).await?
      } else {
          vec![]
      };
      
      // 3. 跨 session 的重要记忆（可选）
      let important = self.get_important_memories(user_id, agent_id, 3).await?;
      
      // 4. 合并去重
      merge_and_deduplicate(recent, semantic, important, limit)
  }
  ```

- [ ] **Task 2.2**: 添加去重逻辑
  ```rust
  fn deduplicate(memories: Vec<MemoryItem>) -> Vec<MemoryItem> {
      let mut seen = HashSet::new();
      memories.into_iter()
          .filter(|mem| seen.insert(mem.id.clone()))
          .collect()
  }
  ```

**验收**:
- ✅ 检索召回率 > 80%
- ✅ 检索准确率 > 70%
- ✅ 响应时间 < 100ms

---

## 📊 改造对比

| 方案 | 代码改动 | 功能完整性 | 复杂度 | 推荐 |
|------|---------|-----------|--------|------|
| Phase 0 | 5行 | 60% | ⭐ | ✅ 立即执行 |
| Phase 0+1 | 30行 | 90% | ⭐⭐ | ✅ 本周完成 |
| Phase 0+1+2 | 100行 | 100% | ⭐⭐⭐ | ⏳ 下周优化 |

---

## 🎯 成功指标

### Phase 0 (紧急修复)
- ✅ 记忆存储 user_id 正确率 = 100%
- ✅ 记忆检索成功率 > 80%
- ✅ 编译零错误

### Phase 1 (Session 支持)
- ✅ Session 隔离准确率 = 100%
- ✅ 跨 session 无串话
- ✅ AI 能记住同 session 历史

### Phase 2 (混合检索)
- ✅ 检索召回率 > 80%
- ✅ 检索准确率 > 70%
- ✅ 检索延迟 < 100ms (P95)

---

## 💡 关键洞察

### 1. AgentMem 功能已很完善

**现有功能**:
- ✅ Session 管理（MemoryScope）
- ✅ 多层隔离（Global/Org/User/Agent/Run/Session）
- ✅ 语义搜索
- ✅ 批量操作
- ✅ 向量存储

**缺失功能**:
- ❌ Working Memory（内存缓存）
- ❌ 自动去重
- ❌ 重要性评分
- ❌ 时间衰减

### 2. default_user_id 是最大问题

**影响**: 
- 🔴 致命 - 完全无法使用记忆隔离
- 🔴 数据污染 - 4752条记忆都是"default"

**修复难度**: ⭐ (极简单)

**修复价值**: ⭐⭐⭐⭐⭐ (立即可用)

### 3. Session 功能已实现但未使用

**原因**: 
- LumosAI Adapter 未传递 session_id
- HTTP API 未接受 session_id 参数

**修复难度**: ⭐⭐ (简单)

**修复价值**: ⭐⭐⭐⭐ (会话隔离)

---

## 🔄 下一步行动

### 立即执行 (今天)

1. ✅ 全面分析现有代码
2. ✅ 发现 Session 功能已存在
3. ✅ 制定最小改造方案
4. ⏳ **执行 Phase 0**: 修复 default_user_id
5. ⏳ 验证记忆存储正确

### 本周完成

1. ⏳ 执行 Phase 1: Session 支持
2. ⏳ 端到端测试
3. ⏳ 更新文档

### 下周优化

1. ⏳ 执行 Phase 2: 混合检索
2. ⏳ 性能优化
3. ⏳ Working Memory（可选）

---

**文档版本**: v2.0 (最小改造方案)
**创建时间**: 2025-11-18 17:35
**状态**: ⏳ 待执行 Phase 0

