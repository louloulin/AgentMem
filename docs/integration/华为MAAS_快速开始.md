# 华为 MaaS Chat 功能 - 快速开始指南

## ✅ 实现状态

AgentMem 已**完整支持**华为 MaaS！基于 LumosAI 实现，仅修改 2 行代码。

## 🚀 5 分钟快速开始

### 步骤 1: 设置环境变量

```bash
# 设置华为 MaaS API Key
export MAAS_API_KEY="your_huawei_maas_api_key"

# 可选：设置默认模型
export MAAS_MODEL="deepseek-v3.2-exp"
```

### 步骤 2: 启动服务

```bash
# 方式 1: 使用 cargo
cargo run --bin agent-mem-server --features lumosai --release

# 方式 2: 使用启动脚本
./start_backend.sh
```

### 步骤 3: 创建 MaaS Agent

```bash
curl -X POST http://localhost:8000/api/v1/agents \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer test-token" \
  -d '{
    "name": "我的 MaaS 助手",
    "description": "基于华为 MaaS 的智能助手",
    "system": "你是一个由华为 MaaS 驱动的AI助手。",
    "llm_config": {
      "provider": "maas",
      "model": "deepseek-v3.2-exp",
      "api_key": null
    }
  }'
```

**响应示例**:
```json
{
  "success": true,
  "data": {
    "id": "agent-abc123",
    "name": "我的 MaaS 助手"
  }
}
```

### 步骤 4: 开始聊天

```bash
# 替换 agent-abc123 为你的 Agent ID
curl -X POST http://localhost:8000/api/v1/agents/agent-abc123/chat/lumosai \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer test-token" \
  -d '{
    "message": "你好，请介绍一下你自己",
    "user_id": "user-001"
  }'
```

**响应示例**:
```json
{
  "success": true,
  "data": {
    "message_id": "msg-xyz789",
    "content": "你好！我是一个基于华为 MaaS 平台的 AI 助手...",
    "memories_updated": true,
    "memories_count": 0,
    "processing_time_ms": 1234
  }
}
```

### 步骤 5: 运行自动化测试

```bash
# 确保已设置 MAAS_API_KEY
./test_maas_chat.sh
```

## 📋 核心特性

### ✅ 已实现功能

| 功能 | 状态 | 说明 |
|------|------|------|
| 文本生成 | ✅ | 支持同步和流式 |
| 多轮对话 | ✅ | 自动记忆管理 |
| 函数调用 | ✅ | Tool Calling 支持 |
| 用户隔离 | ✅ | 多租户安全 |
| 环境变量 | ✅ | 安全配置 |

### 🔑 关键实现

1. **最小改造**: 仅修改 2 行代码（agent_factory.rs 第 120 行）
2. **完全复用**: 100% 复用 LumosAI 的 HuaweiMaasProvider
3. **自动记忆**: 对话自动存储和检索
4. **零维护**: 无额外维护负担

## 📚 支持的模型

| 模型 | 提供商 | 推荐场景 |
|------|--------|----------|
| deepseek-v3.2-exp | DeepSeek | 生产环境（推荐） |
| deepseek-chat | DeepSeek | 一般对话 |
| qwen-max | 阿里 | 中文场景 |
| glm-4 | 智谱 | 复杂任务 |

## 💡 使用示例

### 示例 1: 简单问答

```bash
curl -X POST http://localhost:8000/api/v1/agents/agent-id/chat/lumosai \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer test-token" \
  -d '{
    "message": "1+1等于几？",
    "user_id": "user-001"
  }'
```

### 示例 2: 多轮对话（验证记忆）

```bash
# 第一轮：告诉信息
curl -X POST ... -d '{
  "message": "我的名字叫小明，我喜欢编程",
  "user_id": "user-001"
}'

# 第二轮：测试记忆
curl -X POST ... -d '{
  "message": "我叫什么名字？我喜欢什么？",
  "user_id": "user-001"
}'

# AI 应该回答：你叫小明，你喜欢编程
```

### 示例 3: 不同用户隔离

```bash
# User A
curl -X POST ... -d '{"message": "我叫Alice", "user_id": "user-a"}'

# User B
curl -X POST ... -d '{"message": "我叫什么名字？", "user_id": "user-b"}'

# User B 不知道 User A 的信息（记忆隔离）
```

## 🔍 代码分析

### 关键代码位置

1. **LumosAI Provider 实现**
   - 文件: `lumosai/lumosai_core/src/llm/huawei_maas.rs`
   - 行数: 654 行
   - 功能: 完整的华为 MaaS API 客户端

2. **Agent Factory 集成**
   - 文件: `crates/agent-mem-lumosai/src/agent_factory.rs`
   - 修改: 第 120 行添加 `"maas"` 分支
   - 代码:
     ```rust
     "maas" => Arc::new(providers::huawei_maas(api_key, Some(model))),
     ```

3. **Chat API**
   - 文件: `crates/agent-mem-server/src/routes/chat_lumosai.rs`
   - 功能: 完整的 Chat 接口
   - 无需修改: 完全兼容

4. **Memory 集成**
   - 文件: `crates/agent-mem-lumosai/src/memory_adapter.rs`
   - 功能: 自动记忆存储和检索
   - 自动化: 无需手动调用

### 实现原理

```
用户请求
  ↓
Chat API (chat_lumosai.rs)
  ↓
LumosAgentFactory::create_chat_agent()
  ├─ parse_llm_config() - 解析配置（环境变量）
  ├─ create_llm_provider() - 创建 MaaS Provider ← 新增！
  └─ create_memory_backend() - 创建记忆后端
  ↓
Agent.generate()
  ├─ memory.retrieve() - 自动检索历史
  ├─ llm.generate() - 调用 MaaS API
  └─ memory.store() - 自动存储对话
  ↓
返回响应
```

## 🛠️ 配置选项

### API Key 配置

#### 方式 1: 环境变量（推荐）

```bash
export MAAS_API_KEY="your_key"
```

```json
{
  "llm_config": {
    "provider": "maas",
    "model": "deepseek-v3.2-exp",
    "api_key": null  ← 从环境变量读取
  }
}
```

#### 方式 2: 直接配置

```json
{
  "llm_config": {
    "provider": "maas",
    "model": "deepseek-v3.2-exp",
    "api_key": "sk-xxx..."  ← 直接指定
  }
}
```

### 模型参数（可选）

Agent 的 system prompt 可以包含指令：

```json
{
  "system": "你是一个AI助手。请用中文回答，保持简洁。",
  "llm_config": {
    "provider": "maas",
    "model": "deepseek-v3.2-exp"
  }
}
```

## ❓ 常见问题

### Q1: 如何检查是否配置正确？

```bash
# 1. 检查环境变量
echo $MAAS_API_KEY

# 2. 检查服务日志
RUST_LOG=debug cargo run --features lumosai

# 3. 查找这些日志
# - "💬 Chat request (LumosAI)"
# - "✅ Created LumosAI agent"
```

### Q2: 如何切换模型？

修改 Agent 的 `llm_config.model` 字段：

```bash
curl -X PUT http://localhost:8000/api/v1/agents/agent-id \
  -H "Content-Type: application/json" \
  -d '{
    "llm_config": {
      "provider": "maas",
      "model": "qwen-max"  ← 切换模型
    }
  }'
```

### Q3: 如何查看对话历史？

```bash
curl -X GET http://localhost:8000/api/v1/agents/agent-id/memories \
  -H "Authorization: Bearer test-token" | jq .
```

### Q4: 支持哪些华为 MaaS 模型？

所有华为 MaaS 平台支持的模型都可以使用。常用模型：
- `deepseek-v3.2-exp` - DeepSeek 最新版
- `deepseek-chat` - DeepSeek 稳定版
- `qwen-max` - 通义千问 Max
- `glm-4` - 智谱 GLM-4

### Q5: 是否支持流式响应？

支持！LumosAI Provider 已实现流式响应（SSE）。
（需要前端实现 Server-Sent Events 接收）

## 📖 详细文档

- **使用说明**: `华为MAAS_CHAT功能使用说明.md`
- **验证报告**: `华为MAAS集成验证报告.md`
- **英文文档**: `HUAWEI_MAAS_CHAT_INTEGRATION.md`
- **测试脚本**: `test_maas_chat.sh`

## 🎯 核心代码变更

**修改的文件**: `crates/agent-mem-lumosai/src/agent_factory.rs`

**修改内容**:

```diff
fn create_llm_provider(&self, config: &Value) -> anyhow::Result<Arc<dyn LlmProvider>> {
    // ... 省略前面的代码 ...
    
    let provider: Arc<dyn LlmProvider> = match provider_name {
        "zhipu" => Arc::new(providers::zhipu(api_key, Some(model))),
        "openai" => Arc::new(providers::openai(api_key, Some(model))),
        "anthropic" => Arc::new(providers::anthropic(api_key, Some(model))),
        "deepseek" => Arc::new(providers::deepseek(api_key, Some(model))),
        "qwen" => Arc::new(providers::qwen(api_key, Some(model))),
        "gemini" => Arc::new(providers::gemini(api_key, model)),
        "cohere" => Arc::new(providers::cohere(api_key, model)),
+       "maas" => Arc::new(providers::huawei_maas(api_key, Some(model))),  // ← 新增
        _ => return Err(anyhow::anyhow!(
-           "Unsupported LLM provider: {}. Supported: zhipu, openai, anthropic, deepseek, qwen, gemini, cohere",
+           "Unsupported LLM provider: {}. Supported: zhipu, openai, anthropic, deepseek, qwen, gemini, cohere, maas",
            provider_name
        )),
    };
    
    Ok(provider)
}
```

**总计**: 仅修改 2 行代码！

## 🎉 总结

### ✅ 实现完成

- ✅ 代码实现完整（2 行修改）
- ✅ 功能完整（文本生成、记忆、函数调用）
- ✅ 测试脚本完整
- ✅ 文档完整（中英文）

### 🏆 实现优势

1. **最小改造**: 仅修改 2 行代码
2. **完全复用**: 复用 LumosAI 654 行实现
3. **自动化**: Memory 自动管理
4. **安全**: 环境变量配置
5. **稳定**: 与其他 Provider 一致的体验

### 🚀 立即使用

```bash
# 1. 设置 API Key
export MAAS_API_KEY="your_key"

# 2. 启动服务
cargo run --features lumosai --release

# 3. 创建 Agent（provider: "maas"）
curl -X POST http://localhost:8000/api/v1/agents ...

# 4. 开始聊天
curl -X POST http://localhost:8000/api/v1/agents/{id}/chat/lumosai ...
```

---

**最后更新**: 2025-11-19  
**版本**: v1.0  
**作者**: AgentMem Team
