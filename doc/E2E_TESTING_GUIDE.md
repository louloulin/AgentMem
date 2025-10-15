# E2E 自动化测试指南

**实现日期**: 2025-10-15  
**状态**: ✅ 完成  
**版本**: 1.0

---

## 📋 概述

AgentMem 的端到端（E2E）测试套件提供了完整的 API 集成测试，确保系统在真实环境中的稳定性和功能完整性。

### 测试覆盖

| 测试类别 | 测试数量 | 状态 |
|---------|---------|------|
| Health Check | 1 | ✅ 完成 |
| Agent CRUD | 1 | ✅ 完成 |
| Memory CRUD | 1 | ✅ 完成 |
| Chat Workflow | 1 | ✅ 完成 |
| Streaming Chat | 1 | ✅ 完成 |
| Authentication | 1 | ✅ 完成 |
| Error Handling | 1 | ✅ 完成 |
| **总计** | **7** | ✅ **完成** |

---

## 🚀 快速开始

### 前置条件

1. **Rust 工具链**: 1.70+
2. **PostgreSQL**: 14+ (或 LibSQL)
3. **环境变量**: 配置必要的环境变量

### 运行 E2E 测试

#### 方法 1: 使用测试脚本（推荐）

```bash
# 1. 启动服务器
./scripts/start-test-server.sh

# 2. 运行 E2E 测试
./scripts/run-e2e-tests.sh

# 3. 停止服务器
./scripts/stop-test-server.sh
```

#### 方法 2: 手动运行

```bash
# 1. 启动服务器
cd agentmen
cargo run --bin agent-mem-server &
SERVER_PID=$!

# 2. 等待服务器启动
sleep 5

# 3. 运行 E2E 测试
cargo test --test e2e_api_test -- --ignored --test-threads=1

# 4. 停止服务器
kill $SERVER_PID
```

---

## 📝 测试用例详解

### 1. Health Check 测试

**测试函数**: `test_e2e_health_check`

**测试内容**:
- 验证服务器健康检查端点
- 确认服务器正常运行

**请求**:
```bash
GET /health
```

**预期响应**:
```json
{
  "status": "healthy"
}
```

**验证点**:
- ✅ 状态码: 200 OK
- ✅ 响应包含 "status": "healthy"

---

### 2. Agent CRUD 测试

**测试函数**: `test_e2e_complete_agent_workflow`

**测试流程**:

#### Step 1: 创建 Agent
```bash
POST /api/v1/agents
Authorization: Bearer test-token-e2e

{
  "name": "E2E Test Agent",
  "description": "Agent for end-to-end testing",
  "organization_id": "test-org-e2e",
  "state": "active",
  "config": {
    "llm_provider": "openai",
    "llm_model": "gpt-4"
  }
}
```

**预期响应**: 201 Created

#### Step 2: 获取 Agent
```bash
GET /api/v1/agents/{agent_id}
Authorization: Bearer test-token-e2e
```

**预期响应**: 200 OK

#### Step 3: 更新 Agent
```bash
PUT /api/v1/agents/{agent_id}
Authorization: Bearer test-token-e2e

{
  "name": "E2E Test Agent (Updated)",
  "description": "Updated description"
}
```

**预期响应**: 200 OK

#### Step 4: 列出 Agents
```bash
GET /api/v1/agents
Authorization: Bearer test-token-e2e
```

**预期响应**: 200 OK

#### Step 5: 删除 Agent
```bash
DELETE /api/v1/agents/{agent_id}
Authorization: Bearer test-token-e2e
```

**预期响应**: 204 No Content

**验证点**:
- ✅ 所有 CRUD 操作成功
- ✅ 数据一致性
- ✅ 正确的状态码

---

### 3. Memory CRUD 测试

**测试函数**: `test_e2e_complete_memory_workflow`

**测试流程**:

#### Step 1: 创建 Memory
```bash
POST /api/v1/memories
Authorization: Bearer test-token-e2e

{
  "agent_id": "test-agent-memory-workflow",
  "user_id": "test-user-e2e",
  "memory_type": "episodic",
  "content": "User prefers morning meetings at 9 AM",
  "importance": 0.8,
  "metadata": {
    "category": "preferences",
    "tags": ["meetings", "schedule", "morning"]
  }
}
```

**预期响应**: 201 Created

#### Step 2: 获取 Memory
```bash
GET /api/v1/memories/{memory_id}
Authorization: Bearer test-token-e2e
```

**预期响应**: 200 OK

#### Step 3: 搜索 Memories
```bash
POST /api/v1/memories/search
Authorization: Bearer test-token-e2e

{
  "query": "morning meetings",
  "agent_id": "test-agent-memory-workflow",
  "limit": 10
}
```

**预期响应**: 200 OK

#### Step 4: 更新 Memory
```bash
PUT /api/v1/memories/{memory_id}
Authorization: Bearer test-token-e2e

{
  "importance": 0.9,
  "content": "User strongly prefers morning meetings at 9 AM"
}
```

**预期响应**: 200 OK

#### Step 5: 删除 Memory
```bash
DELETE /api/v1/memories/{memory_id}
Authorization: Bearer test-token-e2e
```

**预期响应**: 204 No Content

**验证点**:
- ✅ 所有 CRUD 操作成功
- ✅ 搜索功能正常
- ✅ 数据一致性

---

### 4. Chat Workflow 测试

**测试函数**: `test_e2e_chat_workflow`

**测试流程**:

#### Step 1: 发送聊天消息
```bash
POST /api/v1/agents/{agent_id}/chat
Authorization: Bearer test-token-e2e

{
  "message": "Hello! I prefer morning meetings at 9 AM.",
  "user_id": "test-user-e2e",
  "stream": false,
  "max_memories": 10
}
```

**预期响应**: 200 OK

#### Step 2: 发送后续消息
```bash
POST /api/v1/agents/{agent_id}/chat
Authorization: Bearer test-token-e2e

{
  "message": "When should we schedule our next meeting?",
  "user_id": "test-user-e2e",
  "stream": false
}
```

**预期响应**: 200 OK

**验证点**:
- ✅ 聊天响应正常
- ✅ 记忆提取和检索
- ✅ 上下文连贯性

---

### 5. Streaming Chat 测试

**测试函数**: `test_e2e_streaming_chat`

**测试内容**:
- 验证流式聊天端点
- 确认 SSE 响应格式

**请求**:
```bash
POST /api/v1/agents/{agent_id}/chat/stream
Authorization: Bearer test-token-e2e

{
  "message": "Tell me a short story about AI",
  "user_id": "test-user-e2e",
  "stream": true
}
```

**预期响应**:
- 状态码: 200 OK
- Content-Type: text/event-stream

**验证点**:
- ✅ SSE 流式响应
- ✅ 正确的 Content-Type

---

### 6. Authentication 测试

**测试函数**: `test_e2e_authentication`

**测试场景**:

#### Scenario 1: 无认证令牌
```bash
GET /api/v1/agents
```

**预期响应**: 401 Unauthorized

#### Scenario 2: 无效的认证令牌
```bash
GET /api/v1/agents
Authorization: Bearer invalid-token
```

**预期响应**: 401 Unauthorized

#### Scenario 3: 有效的认证令牌
```bash
GET /api/v1/agents
Authorization: Bearer test-token-e2e
```

**预期响应**: 200 OK

**验证点**:
- ✅ 未认证请求被拒绝
- ✅ 无效令牌被拒绝
- ✅ 有效令牌被接受

---

### 7. Error Handling 测试

**测试函数**: `test_e2e_error_handling`

**测试场景**:

#### Scenario 1: 创建无效的 Agent（空名称）
```bash
POST /api/v1/agents
Authorization: Bearer test-token-e2e

{
  "name": "",
  "description": "Test",
  "organization_id": "test-org-e2e"
}
```

**预期响应**: 400 Bad Request

#### Scenario 2: 获取不存在的 Agent
```bash
GET /api/v1/agents/non-existent-agent-id
Authorization: Bearer test-token-e2e
```

**预期响应**: 404 Not Found

#### Scenario 3: 创建无效的 Memory（importance > 1.0）
```bash
POST /api/v1/memories
Authorization: Bearer test-token-e2e

{
  "agent_id": "test-agent",
  "user_id": "test-user-e2e",
  "memory_type": "episodic",
  "content": "Test",
  "importance": 1.5
}
```

**预期响应**: 400 Bad Request

**验证点**:
- ✅ 输入验证正常
- ✅ 正确的错误状态码
- ✅ 资源不存在返回 404

---

## 🔧 配置

### 环境变量

```bash
# 服务器配置
export AGENTMEM_HOST=127.0.0.1
export AGENTMEM_PORT=3000

# 数据库配置
export DATABASE_URL=postgresql://user:pass@localhost/agentmem_test

# LLM 配置（可选，用于真实 LLM 测试）
export OPENAI_API_KEY=your-api-key

# 认证配置
export JWT_SECRET=test-secret-key
```

### 测试配置

在 `e2e_api_test.rs` 中修改：

```rust
const BASE_URL: &str = "http://localhost:3000";
const API_VERSION: &str = "v1";
const TEST_ORG_ID: &str = "test-org-e2e";
const TEST_USER_ID: &str = "test-user-e2e";
```

---

## 📊 测试报告

### 运行测试

```bash
cargo test --test e2e_api_test -- --ignored --test-threads=1 --nocapture
```

### 预期输出

```
running 7 tests
test test_e2e_health_check ... ok
test test_e2e_complete_agent_workflow ... ok
test test_e2e_complete_memory_workflow ... ok
test test_e2e_chat_workflow ... ok
test test_e2e_streaming_chat ... ok
test test_e2e_authentication ... ok
test test_e2e_error_handling ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

---

## 🚀 下一步工作

### 短期（1-2 周）

1. ✅ **添加性能测试**
   - 并发请求测试
   - 负载测试
   - 压力测试

2. ✅ **添加更多场景**
   - 多用户并发
   - 长时间运行测试
   - 边界条件测试

3. ✅ **CI/CD 集成**
   - GitHub Actions 配置
   - 自动化测试运行
   - 测试报告生成

### 中期（2-4 周）

4. ✅ **测试数据管理**
   - 测试数据生成器
   - 测试数据清理
   - 测试隔离

5. ✅ **测试覆盖率**
   - 代码覆盖率报告
   - 提高覆盖率到 90%+

---

## 📚 相关文档

- [API 文档](../crates/agent-mem-server/README.md)
- [集成测试文档](../crates/agent-mem-server/tests/README.md)
- [性能测试文档](./PERFORMANCE_TESTING_GUIDE.md)

---

## 🎯 总结

E2E 自动化测试已完全实现并可运行。AgentMem 现在具有：

- ✅ 7 个完整的 E2E 测试用例
- ✅ 覆盖所有核心 API 功能
- ✅ 认证和错误处理测试
- ✅ 完整的测试文档
- ✅ 易于运行和维护

这确保了 AgentMem 在生产环境中的稳定性和可靠性！🚀

