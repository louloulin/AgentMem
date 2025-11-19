# AgentMem 华为 MaaS Chat 功能使用说明

## 📌 功能概述

AgentMem 已完整集成华为 MaaS (Model as a Service) 服务，支持通过 Chat API 使用华为云的 AI 模型（如 DeepSeek V3.2）进行智能对话。

### ✅ 核心特性

- **完全集成**：基于 LumosAI Agent Builder 实现，无需额外开发
- **自动记忆管理**：对话历史自动存储和检索，支持上下文感知
- **多模型支持**：支持华为 MaaS 平台的所有兼容模型
- **环境变量配置**：支持安全的 API Key 管理
- **多租户隔离**：每个用户的对话独立存储

## 🏗️ 架构说明

### 实现架构

```
HTTP POST /api/v1/agents/{agent_id}/chat/lumosai
         ↓
chat_lumosai.rs (Chat API Handler)
         ↓
LumosAgentFactory::create_chat_agent()
         ↓
    ┌────────┴────────┐
    ↓                 ↓
providers::huawei_maas()  AgentMemBackend
(LumosAI Provider)    (Memory Integration)
    ↓                 ↓
HuaweiMaasProvider   AgentMem API
(lumosai_core)       (自动存储/检索)
```

### 关键组件

1. **HuaweiMaasProvider** (`lumosai/lumosai_core/src/llm/huawei_maas.rs`)
   - 完整的华为 MaaS API 客户端实现
   - 支持同步/流式生成
   - 支持函数调用 (Function Calling)

2. **LumosAgentFactory** (`crates/agent-mem-lumosai/src/agent_factory.rs`)
   - Agent 创建工厂
   - 第 120 行：添加了 "maas" Provider 支持

3. **Chat API** (`crates/agent-mem-server/src/routes/chat_lumosai.rs`)
   - RESTful Chat 接口
   - 自动处理认证和权限
   - 集成记忆管理

## 🚀 快速开始

### 1. 环境配置

设置华为 MaaS API Key：

```bash
# 方式 1: 使用 MAAS_API_KEY
export MAAS_API_KEY="your_huawei_maas_api_key"

# 方式 2: 使用 HUAWEI_MAAS_API_KEY
export HUAWEI_MAAS_API_KEY="your_huawei_maas_api_key"

# 可选: 设置默认模型
export MAAS_MODEL="deepseek-v3.2-exp"
```

### 2. 启动服务

```bash
# 使用 lumosai 特性编译和运行
cargo run --bin agent-mem-server --features lumosai --release
```

或使用启动脚本：

```bash
./start_backend.sh
```

### 3. 创建 MaaS Agent

使用 HTTP API 创建 Agent：

```bash
curl -X POST http://localhost:8000/api/v1/agents \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer test-token" \
  -d '{
    "name": "我的 MaaS 助手",
    "description": "基于华为 MaaS 的智能助手",
    "system": "你是一个由华为 MaaS 驱动的AI助手，请用中文回答问题。",
    "llm_config": {
      "provider": "maas",
      "model": "deepseek-v3.2-exp",
      "api_key": null
    }
  }'
```

**响应示例**：

```json
{
  "success": true,
  "data": {
    "id": "agent-abc123",
    "name": "我的 MaaS 助手",
    "llm_config": {
      "provider": "maas",
      "model": "deepseek-v3.2-exp"
    }
  }
}
```

### 4. 发送聊天消息

```bash
curl -X POST http://localhost:8000/api/v1/agents/agent-abc123/chat/lumosai \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer test-token" \
  -d '{
    "message": "你好，请介绍一下你自己",
    "user_id": "user-001"
  }'
```

**响应示例**：

```json
{
  "success": true,
  "data": {
    "message_id": "msg-xyz789",
    "content": "你好！我是一个基于华为 MaaS 平台的 AI 助手...",
    "memories_updated": true,
    "memories_count": 5,
    "processing_time_ms": 1234
  }
}
```

## 📚 详细配置

### Agent LLM 配置说明

#### 配置字段

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `provider` | string | ✅ | 必须设置为 `"maas"` |
| `model` | string | ✅ | 华为 MaaS 支持的模型名称 |
| `api_key` | string/null | ❌ | API Key，为 null 时从环境变量读取 |

#### 支持的模型

华为 MaaS 平台支持的模型（示例）：

- `deepseek-v3.2-exp` - DeepSeek V3.2 实验版（推荐）
- `deepseek-chat` - DeepSeek Chat 版本
- `qwen-max` - 通义千问 Max
- `glm-4` - 智谱 GLM-4
- 其他华为 MaaS 平台提供的模型

### API Key 配置方式

#### 方式 1: 环境变量（推荐）

**优点**：更安全，不会将 Key 存储在数据库

```bash
export MAAS_API_KEY="your_api_key"
```

Agent 配置：

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

**优点**：灵活，可为不同 Agent 配置不同 Key

```json
{
  "llm_config": {
    "provider": "maas",
    "model": "deepseek-v3.2-exp",
    "api_key": "sk-xxx..."  ← 直接指定
  }
}
```

## 🧪 测试和验证

### 自动化测试脚本

项目提供了完整的测试脚本：

```bash
# 1. 设置环境变量
export MAAS_API_KEY="your_api_key"

# 2. 确保服务运行
./start_backend.sh

# 3. 运行测试脚本
./test_maas_chat.sh
```

测试脚本会执行以下步骤：

1. ✅ 检查依赖 (curl, jq)
2. ✅ 验证环境变量
3. ✅ 创建 MaaS Agent
4. ✅ 发送聊天消息
5. ✅ 验证 AI 响应
6. ✅ 检查 Memory 存储

### 手动测试步骤

#### 1. 测试 Agent 创建

```bash
RESPONSE=$(curl -s -X POST http://localhost:8000/api/v1/agents \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer test-token" \
  -d '{
    "name": "测试 Agent",
    "system": "你是一个有帮助的AI助手",
    "llm_config": {
      "provider": "maas",
      "model": "deepseek-v3.2-exp",
      "api_key": null
    }
  }')

echo $RESPONSE | jq .
```

#### 2. 测试单轮对话

```bash
AGENT_ID="agent-abc123"  # 替换为实际 Agent ID

curl -X POST http://localhost:8000/api/v1/agents/$AGENT_ID/chat/lumosai \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer test-token" \
  -d '{
    "message": "1+1等于几？",
    "user_id": "test-user"
  }' | jq .
```

#### 3. 测试多轮对话（验证记忆功能）

```bash
# 第一轮
curl -X POST http://localhost:8000/api/v1/agents/$AGENT_ID/chat/lumosai \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer test-token" \
  -d '{
    "message": "我的名字叫小明",
    "user_id": "test-user"
  }' | jq .

# 第二轮（测试是否记住名字）
curl -X POST http://localhost:8000/api/v1/agents/$AGENT_ID/chat/lumosai \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer test-token" \
  -d '{
    "message": "我叫什么名字？",
    "user_id": "test-user"
  }' | jq .
```

#### 4. 查看存储的记忆

```bash
curl -X GET http://localhost:8000/api/v1/agents/$AGENT_ID/memories \
  -H "Authorization: Bearer test-token" | jq .
```

## 🔍 技术细节

### 实现原理

#### 1. Provider 创建流程

```rust
// agent_factory.rs 第 120 行
"maas" => Arc::new(providers::huawei_maas(api_key, Some(model)))
```

调用 LumosAI 的便利函数创建 Provider。

#### 2. Memory 自动集成

```rust
// agent_factory.rs 第 42-62 行
let memory_backend = self.create_memory_backend(agent, user_id).await?;

let mut lumos_agent = AgentBuilder::new()
    .name(agent_name)
    .instructions(&agent.system)
    .model(llm_provider)
    .build()?;

// 关键：设置 Memory Backend
lumos_agent = lumos_agent.with_memory(memory_backend);
```

#### 3. Chat 调用流程

```rust
// chat_lumosai.rs 第 108-116 行
let response = lumos_agent.generate(
    &all_messages,
    &AgentGenerateOptions::default()
).await?;

// Memory 的 retrieve() 和 store() 由 LumosAI 自动调用
```

### API 端点详情

#### POST /api/v1/agents/:agent_id/chat/lumosai

**请求体**：

```typescript
{
  message: string,        // 用户消息内容
  user_id?: string,      // 用户ID（可选，默认使用认证用户）
  session_id?: string,   // 会话ID（可选）
  metadata?: object      // 元数据（可选）
}
```

**响应体**：

```typescript
{
  success: boolean,
  data: {
    message_id: string,          // 消息ID
    content: string,             // AI回复内容
    memories_updated: boolean,   // 记忆是否更新
    memories_count: number,      // 使用的历史记忆数量
    processing_time_ms: number   // 处理耗时（毫秒）
  }
}
```

## 🛠️ 常见问题

### Q1: 如何检查华为 MaaS 是否正确配置？

**A**: 检查以下几点：

1. 环境变量是否设置：
   ```bash
   echo $MAAS_API_KEY
   ```

2. 编译时是否启用 lumosai 特性：
   ```bash
   cargo build --features lumosai
   ```

3. 查看服务日志中是否有相关错误

### Q2: 如何切换不同的模型？

**A**: 修改 Agent 的 `llm_config.model` 字段：

```bash
curl -X PUT http://localhost:8000/api/v1/agents/$AGENT_ID \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer test-token" \
  -d '{
    "llm_config": {
      "provider": "maas",
      "model": "qwen-max",  ← 切换到其他模型
      "api_key": null
    }
  }'
```

### Q3: 如何调试 MaaS API 调用？

**A**: 启用详细日志：

```bash
RUST_LOG=debug cargo run --bin agent-mem-server --features lumosai
```

查看日志中的以下信息：

- `💬 Chat request (LumosAI)` - Chat 请求开始
- `✅ Created LumosAI agent` - Agent 创建成功
- `Calling LumosAI Agent.generate()` - 调用生成
- `✅ Chat response generated` - 响应生成成功

### Q4: 记忆功能如何工作？

**A**: 记忆功能由 `AgentMemBackend` 自动管理：

1. **存储时机**：每次对话后自动存储
2. **检索时机**：生成响应前自动检索相关历史
3. **存储内容**：用户消息 + AI 回复
4. **隔离方式**：按 `(agent_id, user_id)` 隔离

### Q5: 如何查看某个用户的对话历史？

**A**: 使用 Memory API：

```bash
curl -X POST http://localhost:8000/api/v1/memories/search \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer test-token" \
  -d '{
    "agent_id": "agent-abc123",
    "user_id": "user-001",
    "query": "对话",
    "limit": 20
  }' | jq .
```

## 📊 性能优化建议

### 1. API Key 管理

- ✅ **推荐**：使用环境变量，避免在数据库存储
- ❌ **不推荐**：在代码中硬编码 API Key

### 2. 模型选择

| 模型 | 速度 | 质量 | 适用场景 |
|------|------|------|----------|
| deepseek-v3.2-exp | ⭐⭐⭐ 快 | ⭐⭐⭐⭐⭐ 优秀 | 生产环境、复杂任务 |
| deepseek-chat | ⭐⭐⭐⭐ 较快 | ⭐⭐⭐⭐ 好 | 一般对话 |
| qwen-max | ⭐⭐⭐ 快 | ⭐⭐⭐⭐ 好 | 中文对话 |

### 3. Memory 配置

- 合理设置记忆检索数量（默认 10 条）
- 定期清理无用记忆
- 为高频用户启用记忆缓存

## 📦 部署建议

### Docker 部署

```dockerfile
FROM rust:1.75 as builder

# 复制源代码
COPY . /app
WORKDIR /app

# 编译（启用 lumosai 特性）
RUN cargo build --release --features lumosai

FROM debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && apt-get install -y ca-certificates

# 复制可执行文件
COPY --from=builder /app/target/release/agent-mem-server /usr/local/bin/

# 设置环境变量
ENV MAAS_API_KEY=""
ENV RUST_LOG=info

# 暴露端口
EXPOSE 8000

CMD ["agent-mem-server"]
```

### Kubernetes 部署

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: maas-secret
type: Opaque
stringData:
  MAAS_API_KEY: "your_api_key"

---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: agentmem-server
spec:
  replicas: 3
  selector:
    matchLabels:
      app: agentmem
  template:
    metadata:
      labels:
        app: agentmem
    spec:
      containers:
      - name: server
        image: agentmem:latest
        env:
        - name: MAAS_API_KEY
          valueFrom:
            secretKeyRef:
              name: maas-secret
              key: MAAS_API_KEY
        ports:
        - containerPort: 8000
```

## 🔐 安全建议

1. **API Key 管理**
   - 使用 Kubernetes Secrets 或 AWS Secrets Manager
   - 定期轮换 API Key
   - 不要在日志中打印 API Key

2. **认证和授权**
   - 启用 JWT 认证
   - 实施细粒度权限控制
   - 使用 HTTPS 传输

3. **速率限制**
   - 对 Chat API 实施速率限制
   - 防止滥用和 DDoS 攻击

## 📖 相关文档

- [华为 MaaS 集成详细报告](./HUAWEI_MAAS_CHAT_INTEGRATION.md)
- [LumosAI 集成说明](./LUMOSAI_INTEGRATION_SUMMARY.md)
- [AgentMem API 文档](./README.md)

## 🎉 总结

华为 MaaS Chat 功能已**完整实现**：

- ✅ 基于 LumosAI Agent Builder，代码复用率高
- ✅ 仅修改 2 行代码添加 Provider 支持
- ✅ 自动集成 Memory 管理，无需额外开发
- ✅ 完整的测试脚本和文档
- ✅ 支持环境变量和直接配置两种方式

**立即开始使用**：

```bash
# 1. 设置 API Key
export MAAS_API_KEY="your_key"

# 2. 启动服务
cargo run --features lumosai --release

# 3. 运行测试
./test_maas_chat.sh
```

---

**最后更新**: 2025-11-19  
**文档版本**: v1.0  
**作者**: AgentMem Team
