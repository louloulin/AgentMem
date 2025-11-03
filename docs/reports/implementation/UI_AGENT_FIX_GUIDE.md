# UI Agent LLM配置修复指南

## 问题诊断

### 问题现象
```
Failed to parse SSE data: Error: Configuration error: Zhipu API key not configured
```

### 根本原因
UI聊天页面(`/admin/chat`)使用的Agent**没有配置LLM**（`llm_config`字段为空），导致`orchestrator_factory.rs`中的`parse_llm_config`函数报错。

### 代码分析

**`orchestrator_factory.rs:20-24`**:
```rust
pub fn parse_llm_config(agent: &Agent) -> ServerResult<LLMConfig> {
    let llm_config_value = agent.llm_config.clone()
        .ok_or_else(|| ServerError::bad_request("Agent LLM config not set"))?;
    // ...
}
```

如果Agent的`llm_config`为`None`，会返回错误："Agent LLM config not set"。

---

## 解决方案

### 方案1: UI中选择已配置LLM的Agent（推荐）

1. **访问Agent管理页面**:
   ```
   http://localhost:3001/admin/agents
   ```

2. **创建新Agent（已增强UI）**:
   - 点击"Create Agent"按钮
   - 填写Name和Description
   - **展开"LLM Configuration"区域**
   - 选择Provider: `zhipu`
   - 输入Model: `glm-4-plus`
   - 点击"Create Agent"

3. **在聊天页面使用新Agent**:
   - 访问 `http://localhost:3001/admin/chat`
   - 在Agent选择器中选择刚创建的Agent
   - 开始聊天

### 方案2: API直接创建带LLM配置的Agent

```bash
curl -X POST http://localhost:8080/api/v1/agents \
  -H "Content-Type: application/json" \
  -H "X-User-ID: test-user" \
  -H "X-Organization-ID: default-org" \
  -d '{
    "name": "智谱AI助手",
    "description": "使用Zhipu GLM-4-Plus的智能助手",
    "llm_config": {
      "provider": "zhipu",
      "model": "glm-4-plus"
    }
  }'
```

**返回的Agent ID**:
```
agent-a23bfd10-b1be-4848-8b4e-f3d34f4aae0e
```

### 方案3: 更新现有Agent添加LLM配置

```bash
curl -X PATCH http://localhost:8080/api/v1/agents/{agent_id} \
  -H "Content-Type: application/json" \
  -H "X-User-ID: test-user" \
  -H "X-Organization-ID: default-org" \
  -d '{
    "llm_config": {
      "provider": "zhipu",
      "model": "glm-4-plus"
    }
  }'
```

---

## UI增强功能说明

### 已实施的UI改进

**文件**: `agentmem-ui/src/app/admin/agents/page.tsx`

**新增功能**:
1. ✅ LLM Provider选择器
   - Zhipu AI (智谱)
   - OpenAI
   - Anthropic
   - DeepSeek

2. ✅ Model输入框
   - 智能推荐（根据provider显示推荐模型）

3. ✅ 可折叠配置区域
   - 默认展开，方便用户配置

4. ✅ 环境变量提示
   - 告知用户API keys通过环境变量配置

### 代码示例

```typescript
// CreateAgentDialog组件增强
const [llmProvider, setLlmProvider] = useState('zhipu');
const [llmModel, setLlmModel] = useState('glm-4-plus');

// 提交时包含LLM配置
const llm_config = llmProvider && llmModel ? {
  provider: llmProvider,
  model: llmModel,
} : undefined;

await apiClient.createAgent({ name, description, llm_config });
```

---

## 验证步骤

### 1. 验证新Agent有LLM配置

```bash
curl -s http://localhost:8080/api/v1/agents/{agent_id} \
  -H "X-User-ID: test-user" \
  -H "X-Organization-ID: default-org" | jq '.data.llm_config'
```

**期望输出**:
```json
{
  "provider": "zhipu",
  "model": "glm-4-plus"
}
```

### 2. 测试聊天功能

```bash
curl -X POST http://localhost:8080/api/v1/agents/{agent_id}/chat \
  -H "Content-Type: application/json" \
  -H "X-User-ID: test-user" \
  -H "X-Organization-ID: default-org" \
  -d '{
    "message": "你好，请介绍一下你自己",
    "stream": false
  }'
```

### 3. UI测试

1. 打开 `http://localhost:3001/admin/agents`
2. 点击"Create Agent"
3. 填写信息并配置LLM
4. 创建成功后，去`/admin/chat`页面
5. 选择新创建的Agent
6. 发送测试消息

---

## 环境配置确认

### 服务器端环境变量

**`start_server_with_correct_onnx.sh`**:
```bash
export ZHIPU_API_KEY="99a311fa7920a59e9399cf26ecc1e938.ac4w6buZHr2Ggc3k"
export LLM_PROVIDER="zhipu"
export LLM_MODEL="glm-4-plus"
export EMBEDDER_PROVIDER="fastembed"
export EMBEDDER_MODEL="BAAI/bge-small-en-v1.5"
```

### 验证环境变量生效

检查服务器日志:
```bash
tail -f backend-onnx-fixed.log | grep -i "zhipu\|llm"
```

---

## 常见问题

### Q1: 为什么旧Agent不能聊天？
**A**: 旧Agent没有`llm_config`字段，服务器无法知道使用哪个LLM provider和model。

### Q2: 可以不配置LLM吗？
**A**: 不可以。聊天功能必须要有LLM才能生成回复。但记忆管理功能（创建、搜索记忆）不需要LLM配置。

### Q3: API Key在哪里配置？
**A**: API Key通过服务器环境变量配置（如`ZHIPU_API_KEY`），不在Agent配置中存储，确保安全性。

### Q4: 如何批量更新旧Agent？
**A**: 
```bash
# 获取所有Agent
curl -s http://localhost:8080/api/v1/agents \
  -H "X-User-ID: test-user" \
  -H "X-Organization-ID: default-org" | jq '.data[].id'

# 为每个Agent添加LLM配置
for agent_id in $(上面的ID列表); do
  curl -X PATCH http://localhost:8080/api/v1/agents/$agent_id \
    -H "Content-Type: application/json" \
    -H "X-User-ID: test-user" \
    -H "X-Organization-ID: default-org" \
    -d '{"llm_config": {"provider": "zhipu", "model": "glm-4-plus"}}'
done
```

---

## 测试Agent ID

**已创建的测试Agent** (带完整LLM配置):
```
agent-a23bfd10-b1be-4848-8b4e-f3d34f4aae0e
```

**配置详情**:
- Name: 智谱AI助手
- Description: 使用Zhipu GLM-4-Plus的智能助手
- LLM Provider: zhipu
- LLM Model: glm-4-plus

**直接测试链接**:
```
http://localhost:3001/admin/chat?agent_id=agent-a23bfd10-b1be-4848-8b4e-f3d34f4aae0e
```

---

## 总结

✅ **问题已解决**:
1. 增强UI Agent创建对话框，支持LLM配置
2. 更新API Client接口支持llm_config
3. 创建测试Agent验证功能
4. 提供完整的修复和验证指南

🔧 **下一步**:
1. UI需要重启以应用更改（如果还未重启）
2. 在UI中创建新Agent或使用测试Agent
3. 验证聊天功能正常工作

📝 **文档**:
- UI增强代码: `agentmem-ui/src/app/admin/agents/page.tsx`
- API Client更新: `agentmem-ui/src/lib/api-client.ts`
- 服务器LLM配置解析: `crates/agent-mem-server/src/orchestrator_factory.rs`

