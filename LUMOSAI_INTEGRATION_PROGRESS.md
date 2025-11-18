# LumosAI-AgentMem 集成进度报告

**日期**: 2025-11-18  
**状态**: Phase 0 已完成，Phase 1 准备就绪

---

## ✅ 已完成工作

### 1. 深入学习 AgentMem 架构

**核心发现**:

#### Memory Engine (`crates/agent-mem-core/src/engine.rs`)
```rust
pub struct MemoryEngine {
    config: MemoryEngineConfig,
    hierarchy_manager: Arc<dyn HierarchyManager>,
    importance_scorer: Arc<dyn ImportanceScorer>,
    conflict_resolver: Arc<dyn ConflictResolver>,
    memory_repository: Option<Arc<dyn MemoryRepositoryTrait>>,  // ← LibSQL持久化
}
```

**关键方法**:
- `add_memory()` - 添加记忆并自动评分
- `search_memories()` - 混合搜索（向量+全文），支持LibSQL优先
- `update_memory()` - 更新并重新评分
- `remove_memory()` - 删除记忆

**特点**:
- ✅ 使用MemoryV4 (agent_mem_traits::Memory) 作为核心数据结构
- ✅ 支持LibSQL Repository持久化
- ✅ 自动importance评分
- ✅ 混合搜索with时间衰减和用户匹配权重

#### AgentOrchestrator (`crates/agent-mem-core/src/orchestrator/mod.rs`)
```rust
pub struct AgentOrchestrator {
    config: OrchestratorConfig,
    memory_engine: Arc<MemoryEngine>,
    message_repo: Arc<dyn MessageRepositoryTrait>,
    llm_client: Arc<LLMClient>,
    tool_executor: Arc<ToolExecutor>,
    working_store: Option<Arc<dyn WorkingMemoryStore>>,
}
```

**对话循环** (`step()` 方法):
1. 验证请求参数
2. 检索相关记忆 (MemoryEngine.search_memories)
3. 构建Prompt (系统消息 + 记忆 + 用户消息)
4. 调用LLM (LLMClient.complete)
5. 提取并保存记忆 (MemoryExtractor)
6. 返回响应

#### 当前Chat API (`crates/agent-mem-server/src/routes/chat.rs`)
```rust
pub async fn send_chat_message(
    repositories: Arc<Repositories>,
    auth_user: AuthUser,
    agent_id: String,
    req: ChatMessageRequest,
) -> ServerResult<Json<ApiResponse<ChatMessageResponse>>> {
    // 1. 验证Agent
    let agent = repositories.agents.find_by_id(&agent_id).await?;
    
    // 2. 创建AgentOrchestrator
    let orchestrator = create_orchestrator(&agent, &repositories).await?;
    
    // 3. 调用orchestrator.step()
    let response = orchestrator.step(orchestrator_request).await?;
    
    // 4. 返回响应
    Ok(Json(ApiResponse::success(response)))
}
```

### 2. 环境准备完成

✅ **创建 agent-mem-lumosai crate**
```bash
crates/agent-mem-lumosai/
├── Cargo.toml  ← 已配置完整依赖
├── src/
│   ├── lib.rs  ← 待实现
│   ├── memory_adapter.rs  ← 待实现
│   ├── agent_factory.rs  ← 待实现
│   └── error.rs  ← 待实现
```

✅ **依赖已配置**:
- lumosai_core (path)
- agent-mem-core (path)
- agent-mem-traits (path)
- tokio, async-trait
- serde, serde_json
- anyhow, thiserror
- tracing, uuid, chrono

---

## 🚧 待实现工作

### Phase 1: 基础集成 (预计2-3小时)

#### Task 1.1: 实现 Memory Adapter ⭐

**文件**: `crates/agent-mem-lumosai/src/memory_adapter.rs`

**LumosAI Memory Trait** (from `lumosai_core/src/memory/mod.rs`):
```rust
#[async_trait::async_trait]
pub trait Memory: Send + Sync {
    async fn store(&self, message: &Message) -> Result<()>;
    async fn retrieve(&self, query: &str, limit: usize) -> Result<Vec<Message>>;
    async fn clear(&self) -> Result<()>;
}
```

**实现策略**:
```rust
use async_trait::async_trait;
use lumosai_core::memory::Memory as LumosMemory;
use lumosai_core::llm::Message as LumosMessage;
use lumosai_core::Result;
use agent_mem_core::engine::MemoryEngine;
use agent_mem_traits::{MemoryV4 as Memory, Content, AttributeKey, AttributeValue};
use std::sync::Arc;

pub struct AgentMemBackend {
    engine: Arc<MemoryEngine>,
    agent_id: String,
    user_id: String,
}

#[async_trait]
impl LumosMemory for AgentMemBackend {
    async fn store(&self, message: &LumosMessage) -> Result<()> {
        // 1. 转换LumosMessage为AgentMem Memory
        let content = Content::text(format!("[{}]: {}", message.role, message.content));
        
        // 2. 构建Memory
        let mut memory = Memory::new(
            uuid::Uuid::new_v4().to_string(),
            content,
        );
        
        // 3. 设置属性
        memory.attributes.set(
            AttributeKey::core("agent_id"),
            AttributeValue::String(self.agent_id.clone())
        );
        memory.attributes.set(
            AttributeKey::core("user_id"),
            AttributeValue::String(self.user_id.clone())
        );
        
        // 4. 调用MemoryEngine.add_memory
        self.engine.add_memory(memory).await
            .map_err(|e| lumosai_core::Error::Memory(e.to_string()))?;
        
        Ok(())
    }
    
    async fn retrieve(&self, query: &str, limit: usize) -> Result<Vec<LumosMessage>> {
        // 1. 调用MemoryEngine.search_memories
        use agent_mem_core::hierarchy::MemoryScope;
        let scope = Some(MemoryScope::User {
            agent_id: self.agent_id.clone(),
            user_id: self.user_id.clone(),
        });
        
        let memories = self.engine.search_memories(query, scope, Some(limit))
            .await
            .map_err(|e| lumosai_core::Error::Memory(e.to_string()))?;
        
        // 2. 转换Memory为LumosMessage
        let messages = memories.into_iter()
            .map(|mem| {
                let content_text = match &mem.content {
                    Content::Text(t) => t.clone(),
                    _ => String::new(),
                };
                
                LumosMessage {
                    role: lumosai_core::llm::Role::User,
                    content: content_text,
                    metadata: None,
                    name: None,
                }
            })
            .collect();
        
        Ok(messages)
    }
    
    async fn clear(&self) -> Result<()> {
        // AgentMem没有直接的clear方法，这里留空或记录警告
        tracing::warn!("Memory clear requested but not implemented");
        Ok(())
    }
}
```

#### Task 1.2: 实现 Agent Factory ⭐

**文件**: `crates/agent-mem-lumosai/src/agent_factory.rs`

**实现策略**:
```rust
use lumosai_core::agent::{AgentBuilder, Agent as LumosAgent};
use lumosai_core::llm::providers;
use agent_mem_core::storage::models::Agent;
use agent_mem_core::storage::factory::Repositories;
use agent_mem_core::engine::{MemoryEngine, MemoryEngineConfig};
use crate::memory_adapter::AgentMemBackend;
use std::sync::Arc;

pub struct LumosAgentFactory {
    repositories: Arc<Repositories>,
}

impl LumosAgentFactory {
    pub fn new(repositories: Arc<Repositories>) -> Self {
        Self { repositories }
    }
    
    pub async fn create_chat_agent(
        &self,
        agent: &Agent,
        user_id: &str,
    ) -> anyhow::Result<Arc<dyn LumosAgent>> {
        // 1. 解析LLM配置
        let llm_config = self.parse_llm_config(agent)?;
        
        // 2. 创建LLM Provider
        let llm_provider = self.create_llm_provider(&llm_config)?;
        
        // 3. 创建Memory Backend
        let memory_backend = self.create_memory_backend(agent, user_id).await?;
        
        // 4. 使用AgentBuilder构建LumosAI Agent
        let lumos_agent = AgentBuilder::new()
            .name(&agent.name)
            .instructions(&agent.system.clone().unwrap_or_default())
            .model(llm_provider)
            .memory(memory_backend)
            .temperature(0.7)
            .max_tokens(2000)
            .build()?;
        
        Ok(Arc::new(lumos_agent))
    }
    
    fn parse_llm_config(&self, agent: &Agent) -> anyhow::Result<agent_mem_traits::LLMConfig> {
        let llm_config_value = agent.llm_config.clone()
            .ok_or_else(|| anyhow::anyhow!("Agent LLM config not set"))?;
        
        let mut llm_config: agent_mem_traits::LLMConfig = 
            serde_json::from_value(llm_config_value)?;
        
        // 从环境变量读取API key
        if llm_config.api_key.is_none() {
            let env_var_name = format!("{}_API_KEY", llm_config.provider.to_uppercase());
            if let Ok(api_key) = std::env::var(&env_var_name) {
                llm_config.api_key = Some(api_key);
            }
        }
        
        Ok(llm_config)
    }
    
    fn create_llm_provider(
        &self,
        config: &agent_mem_traits::LLMConfig,
    ) -> anyhow::Result<Arc<dyn lumosai_core::llm::LlmProvider>> {
        let api_key = config.api_key.clone()
            .ok_or_else(|| anyhow::anyhow!("API key not configured"))?;
        
        let provider: Arc<dyn lumosai_core::llm::LlmProvider> = match config.provider.as_str() {
            "zhipu" => Arc::new(providers::zhipu(api_key, Some(config.model.clone()))),
            "openai" => Arc::new(providers::openai(api_key, Some(config.model.clone()))),
            "anthropic" => Arc::new(providers::anthropic(api_key, Some(config.model.clone()))),
            "deepseek" => Arc::new(providers::deepseek(api_key, Some(config.model.clone()))),
            "qwen" => Arc::new(providers::qwen(api_key, Some(config.model.clone()))),
            _ => return Err(anyhow::anyhow!("Unsupported provider: {}", config.provider)),
        };
        
        Ok(provider)
    }
    
    async fn create_memory_backend(
        &self,
        agent: &Agent,
        user_id: &str,
    ) -> anyhow::Result<Arc<dyn lumosai_core::memory::Memory>> {
        // 创建MemoryEngine with LibSQL Repository
        let memory_engine = Arc::new(MemoryEngine::with_repository(
            MemoryEngineConfig::default(),
            self.repositories.memories.clone(),
        ));
        
        // 包装为AgentMemBackend
        let backend = Arc::new(AgentMemBackend::new(
            memory_engine,
            agent.id.clone(),
            user_id.to_string(),
        ));
        
        Ok(backend as Arc<dyn lumosai_core::memory::Memory>)
    }
}
```

#### Task 1.3: 更新 Chat API ⭐

**文件**: `crates/agent-mem-server/src/routes/chat.rs`

**修改策略**:
1. 添加`agent-mem-lumosai`依赖到`agent-mem-server/Cargo.toml`
2. 在`send_chat_message`中使用LumosAI Agent

```rust
// 新增导入
use agent_mem_lumosai::agent_factory::LumosAgentFactory;

pub async fn send_chat_message(
    Extension(repositories): Extension<Arc<Repositories>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(agent_id): Path<String>,
    Json(req): Json<ChatMessageRequest>,
) -> ServerResult<Json<ApiResponse<ChatMessageResponse>>> {
    let start_time = std::time::Instant::now();
    
    // 1. 验证Agent
    let agent = repositories.agents
        .find_by_id(&agent_id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to read agent: {e}")))?
        .ok_or_else(|| ServerError::not_found("Agent not found"))?;
    
    // 2. 权限检查
    if agent.organization_id != auth_user.org_id {
        return Err(ServerError::forbidden("Access denied"));
    }
    
    // 3. 创建LumosAI Agent (使用AgentMem作为记忆后端)
    let factory = LumosAgentFactory::new(repositories.clone());
    let user_id = req.user_id.as_ref().unwrap_or(&auth_user.user_id);
    let lumos_agent = factory.create_chat_agent(&agent, user_id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to create agent: {e}")))?;
    
    // 4. 构建消息
    let messages = vec![
        lumosai_core::llm::Message {
            role: lumosai_core::llm::Role::User,
            content: req.message.clone(),
            metadata: req.metadata.clone(),
            name: None,
        },
    ];
    
    // 5. 调用LumosAI Agent生成响应
    let response = lumos_agent.generate(
        &messages,
        &lumosai_core::agent::AgentGenerateOptions::default(),
    ).await
        .map_err(|e| ServerError::internal_error(format!("Agent failed: {e}")))?;
    
    // 6. 返回响应
    let processing_time_ms = start_time.elapsed().as_millis() as u64;
    
    Ok(Json(ApiResponse::success(ChatMessageResponse {
        message_id: uuid::Uuid::new_v4().to_string(),
        content: response,
        memories_updated: true,
        memories_count: 1,
        tool_calls: None,
        processing_time_ms,
    })))
}
```

---

## 📋 实施步骤

### Step 1: 实现核心文件

```bash
# 1. 实现lib.rs
cat > crates/agent-mem-lumosai/src/lib.rs << 'EOF'
pub mod memory_adapter;
pub mod agent_factory;
pub mod error;

pub use memory_adapter::AgentMemBackend;
pub use agent_factory::LumosAgentFactory;
EOF

# 2. 实现memory_adapter.rs (复制上面的代码)

# 3. 实现agent_factory.rs (复制上面的代码)

# 4. 实现error.rs
cat > crates/agent-mem-lumosai/src/error.rs << 'EOF'
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LumosIntegrationError {
    #[error("Memory error: {0}")]
    Memory(String),
    
    #[error("Agent error: {0}")]
    Agent(String),
    
    #[error("LLM error: {0}")]
    Llm(String),
}
EOF
```

### Step 2: 更新依赖

```toml
# crates/agent-mem-server/Cargo.toml
[dependencies]
agent-mem-lumosai = { path = "../agent-mem-lumosai" }
```

### Step 3: 编译测试

```bash
# 编译lumosai集成crate
cargo build --package agent-mem-lumosai

# 编译server
cargo build --package agent-mem-server --release
```

### Step 4: 集成测试

创建测试脚本 `/tmp/test_lumosai_integration.sh`:

```bash
#!/bin/bash
API="http://localhost:8080/api/v1"

echo "=== LumosAI-AgentMem 集成测试 ==="

# 1. 创建Agent
AGENT_ID=$(curl -s -X POST "$API/agents" \
  -H "Content-Type: application/json" \
  -d '{
    "name":"LumosAI测试Agent",
    "llm_config":{
      "provider":"zhipu",
      "model":"glm-4"
    }
  }' | jq -r '.data.id')

echo "创建Agent: $AGENT_ID"

# 2. 测试Chat (应使用LumosAI Agent)
RESPONSE=$(curl -s -X POST "$API/agents/$AGENT_ID/chat" \
  -H "Content-Type: application/json" \
  -d '{"message":"你好，请介绍一下你自己"}')

echo "Chat响应:"
echo "$RESPONSE" | jq '.'

# 3. 验证记忆保存
MEMORIES=$(curl -s "$API/memories?user_id=test_user&limit=5")
echo "记忆数量:" $(echo "$MEMORIES" | jq '.data | length')

echo "✅ 集成测试完成"
```

---

## 验证标准

- [ ] **编译通过**: `cargo build --release --bin agent-mem-server`
- [ ] **LumosAI Agent创建**: Factory能成功创建Agent
- [ ] **Memory Backend**:
  - [ ] `store()` 正确保存到AgentMem
  - [ ] `retrieve()` 正确检索记忆
  - [ ] 记忆包含score字段
- [ ] **Chat API**:
  - [ ] POST `/api/v1/agents/{id}/chat` 返回正确响应
  - [ ] 响应包含`message_id`, `content`, `memories_count`
  - [ ] 记忆正确保存到LibSQL数据库
- [ ] **性能**:
  - [ ] 响应延迟 < 500ms (不含LLM调用)
  - [ ] 记忆存储延迟 < 50ms
- [ ] **集成测试**: 所有Chat API测试通过

---

## 🎯 下一步行动 (按优先级)

### 立即执行 (今天)
1. ✅ 创建`agent-mem-lumosai` crate
2. ✅ 配置依赖
3. ⏳ 实现`memory_adapter.rs`
4. ⏳ 实现`agent_factory.rs`
5. ⏳ 实现`lib.rs`和`error.rs`

### 本周目标
- 完成Task 1.1-1.3
- 编译测试通过
- 基础集成可工作

### 预期效果

集成完成后，AgentMem将获得：

| 功能 | 集成前 | 集成后 |
|------|--------|--------|
| **LLM Providers** | 4个 | 14+ 个 |
| **Function Calling** | 基础 | OpenAI标准 |
| **工具系统** | 基础 | 25+ 内置工具 |
| **多Agent协作** | 无 | 5+ 协作模式 |
| **工作流** | 无 | DAG + Pipeline |
| **流式响应** | 自定义 | 标准API |
| **记忆管理** | 专业级 | 专业级 (保持) |

---

## 📚 参考文档

1. `lumosai1.txt` - 详细集成方案
2. `lumosai/lumosai_core/src/agent/builder.rs` - AgentBuilder API
3. `lumosai/lumosai_core/src/memory/mod.rs` - Memory trait
4. `crates/agent-mem-core/src/engine.rs` - MemoryEngine实现
5. `crates/agent-mem-server/src/routes/chat.rs` - 当前Chat API

---

**状态**: 环境准备完成，等待代码实现  
**下一步**: 实现memory_adapter.rs和agent_factory.rs  
**预计完成时间**: 2-3小时 (Phase 1)
