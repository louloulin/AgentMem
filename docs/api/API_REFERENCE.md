# AgentMem API 参考文档

**版本**: v2.0.0  
**更新日期**: 2025-10-27  
**API类型**: REST API  
**基础URL**: `http://localhost:8080`  

---

## 📋 目录

1. [快速开始](#快速开始)
2. [认证](#认证)
3. [Memory管理](#memory管理)
4. [Agent管理](#agent管理)
5. [Chat对话](#chat对话)
6. [用户管理](#用户管理)
7. [组织管理](#组织管理)
8. [健康检查](#健康检查)
9. [错误代码](#错误代码)

---

## 🚀 快速开始

### 访问Swagger UI

```
http://localhost:8080/swagger-ui
```

### 基础请求示例

```bash
# 健康检查
curl http://localhost:8080/health

# 添加记忆
curl -X POST http://localhost:8080/api/v1/memories \
  -H "Content-Type: application/json" \
  -d '{
    "agent_id": "agent-123",
    "content": "用户喜欢披萨",
    "importance": 0.8
  }'

# 搜索记忆
curl -X POST http://localhost:8080/api/v1/memories/search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "用户喜欢什么",
    "agent_id": "agent-123",
    "limit": 10
  }'
```

---

## 🔐 认证

### Bearer Token认证

```http
Authorization: Bearer <your-jwt-token>
```

### 获取Token

```bash
curl -X POST http://localhost:8080/api/v1/users/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "your-password"
  }'
```

**响应**:
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": {
    "id": "user-123",
    "email": "user@example.com"
  }
}
```

---

## 💾 Memory管理

### 添加记忆

`POST /api/v1/memories`

**请求体**:
```json
{
  "agent_id": "agent-123",
  "user_id": "user-456",
  "content": "用户在2023年访问过巴黎",
  "memory_type": "episodic",
  "importance": 0.8,
  "metadata": {
    "location": "Paris",
    "year": "2023"
  }
}
```

**响应** (200 OK):
```json
{
  "id": "mem-789",
  "message": "Memory added successfully"
}
```

### 获取记忆

`GET /api/v1/memories/{id}`

**响应** (200 OK):
```json
{
  "id": "mem-789",
  "agent_id": "agent-123",
  "content": "用户在2023年访问过巴黎",
  "memory_type": "episodic",
  "importance": 0.8,
  "metadata": {
    "location": "Paris",
    "year": "2023"
  },
  "created_at": "2023-10-27T12:00:00Z",
  "updated_at": "2023-10-27T12:00:00Z"
}
```

### 更新记忆

`PUT /api/v1/memories/{id}`

**请求体**:
```json
{
  "content": "用户在2023年夏天访问过巴黎",
  "importance": 0.9
}
```

### 删除记忆

`DELETE /api/v1/memories/{id}`

**响应** (200 OK):
```json
{
  "id": "mem-789",
  "message": "Memory deleted successfully"
}
```

### 搜索记忆

`POST /api/v1/memories/search`

**请求体**:
```json
{
  "query": "用户去过哪里旅行",
  "agent_id": "agent-123",
  "memory_type": "episodic",
  "limit": 10,
  "threshold": 0.7
}
```

**响应** (200 OK):
```json
{
  "results": [
    {
      "id": "mem-789",
      "content": "用户在2023年访问过巴黎",
      "similarity": 0.95,
      "importance": 0.8
    }
  ],
  "total": 1
}
```

### 批量添加记忆

`POST /api/v1/memories/batch`

**请求体**:
```json
{
  "memories": [
    {
      "agent_id": "agent-123",
      "content": "记忆1",
      "importance": 0.7
    },
    {
      "agent_id": "agent-123",
      "content": "记忆2",
      "importance": 0.8
    }
  ]
}
```

**响应** (200 OK):
```json
{
  "successful": 2,
  "failed": 0,
  "results": [
    {"id": "mem-001", "message": "Success"},
    {"id": "mem-002", "message": "Success"}
  ],
  "errors": []
}
```

### 获取Agent的所有记忆

`GET /api/v1/agents/{agent_id}/memories`

**响应** (200 OK):
```json
{
  "memories": [
    {
      "id": "mem-001",
      "content": "记忆内容",
      "importance": 0.8
    }
  ],
  "total": 1
}
```

---

## 🤖 Agent管理

### 创建Agent

`POST /api/v1/agents`

**请求体**:
```json
{
  "organization_id": "org-123",
  "name": "Customer Support Bot",
  "description": "处理客户咨询的智能助手"
}
```

**响应** (200 OK):
```json
{
  "id": "agent-456",
  "organization_id": "org-123",
  "name": "Customer Support Bot",
  "state": "active",
  "created_at": "2023-10-27T12:00:00Z"
}
```

### 获取Agent

`GET /api/v1/agents/{id}`

### 更新Agent

`PUT /api/v1/agents/{id}`

**请求体**:
```json
{
  "name": "Updated Bot Name",
  "description": "Updated description"
}
```

### 删除Agent

`DELETE /api/v1/agents/{id}`

### 列出所有Agents

`GET /api/v1/agents`

**查询参数**:
- `limit`: 返回数量（默认50）
- `offset`: 偏移量（默认0）

**响应** (200 OK):
```json
{
  "agents": [
    {
      "id": "agent-001",
      "name": "Bot 1",
      "state": "active"
    }
  ],
  "total": 1
}
```

### 获取Agent状态

`GET /api/v1/agents/{agent_id}/state`

**响应** (200 OK):
```json
{
  "agent_id": "agent-123",
  "state": "active",
  "last_active_at": "2023-10-27T12:00:00Z",
  "memory_count": 150,
  "error_message": null
}
```

### 更新Agent状态

`PUT /api/v1/agents/{agent_id}/state`

**请求体**:
```json
{
  "state": "idle",
  "error_message": null
}
```

---

## 💬 Chat对话

### 发送消息

`POST /api/v1/agents/{agent_id}/chat`

**请求体**:
```json
{
  "message": "你好，我需要帮助",
  "context": {
    "user_id": "user-123",
    "session_id": "session-456"
  }
}
```

**响应** (200 OK):
```json
{
  "response": "你好！我是你的智能助手，很高兴为你服务。请问有什么可以帮到你的？",
  "agent_id": "agent-123",
  "timestamp": "2023-10-27T12:00:00Z",
  "tool_calls": []
}
```

### 流式对话

`POST /api/v1/agents/{agent_id}/chat/stream`

**响应**: Server-Sent Events (SSE)

```javascript
// JavaScript示例
const eventSource = new EventSource('/api/v1/agents/agent-123/chat/stream');

eventSource.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log(data.chunk); // 流式内容
};
```

### 获取对话历史

`GET /api/v1/agents/{agent_id}/chat/history`

**查询参数**:
- `limit`: 返回数量（默认50）
- `before`: 时间戳，获取之前的消息

**响应** (200 OK):
```json
{
  "messages": [
    {
      "role": "user",
      "content": "你好",
      "timestamp": "2023-10-27T12:00:00Z"
    },
    {
      "role": "assistant",
      "content": "你好！有什么可以帮你？",
      "timestamp": "2023-10-27T12:00:01Z"
    }
  ],
  "total": 2
}
```

---

## 👤 用户管理

### 注册用户

`POST /api/v1/users/register`

**请求体**:
```json
{
  "email": "user@example.com",
  "password": "SecurePassword123!",
  "full_name": "张三"
}
```

**响应** (200 OK):
```json
{
  "id": "user-123",
  "email": "user@example.com",
  "full_name": "张三",
  "created_at": "2023-10-27T12:00:00Z"
}
```

### 登录

`POST /api/v1/users/login`

**请求体**:
```json
{
  "email": "user@example.com",
  "password": "SecurePassword123!"
}
```

**响应** (200 OK):
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": {
    "id": "user-123",
    "email": "user@example.com"
  },
  "expires_at": "2023-10-28T12:00:00Z"
}
```

### 获取当前用户

`GET /api/v1/users/me`

**Headers**: `Authorization: Bearer <token>`

### 更新当前用户

`PUT /api/v1/users/me`

**请求体**:
```json
{
  "full_name": "李四",
  "avatar_url": "https://example.com/avatar.jpg"
}
```

### 修改密码

`POST /api/v1/users/me/password`

**请求体**:
```json
{
  "old_password": "OldPassword123!",
  "new_password": "NewPassword123!"
}
```

---

## 🏢 组织管理

### 创建组织

`POST /api/v1/organizations`

**请求体**:
```json
{
  "name": "Acme Corp",
  "description": "我们的智能助手平台"
}
```

### 获取组织

`GET /api/v1/organizations/{org_id}`

### 更新组织

`PUT /api/v1/organizations/{org_id}`

### 删除组织

`DELETE /api/v1/organizations/{org_id}`

### 获取组织成员

`GET /api/v1/organizations/{org_id}/members`

---

## 🏥 健康检查

### 基础健康检查

`GET /health`

**响应** (200 OK):
```json
{
  "status": "healthy",
  "timestamp": "2023-10-27T12:00:00Z",
  "version": "2.0.0",
  "checks": {
    "database": {
      "status": "healthy",
      "message": "Database connection successful",
      "last_check": "2023-10-27T12:00:00Z"
    },
    "memory_system": {
      "status": "healthy",
      "message": "Memory system operational",
      "last_check": "2023-10-27T12:00:00Z"
    }
  }
}
```

### Liveness探针

`GET /health/live`

**响应** (200 OK):
```json
{
  "status": "alive",
  "timestamp": "2023-10-27T12:00:00Z",
  "version": "2.0.0"
}
```

### Readiness探针

`GET /health/ready`

**响应** (200 OK / 503 Service Unavailable):
```json
{
  "status": "ready",
  "timestamp": "2023-10-27T12:00:00Z",
  "checks": {
    "database": true,
    "memory_system": true
  }
}
```

---

## ❌ 错误代码

### 标准错误响应

```json
{
  "code": "RESOURCE_NOT_FOUND",
  "message": "Memory with id 'mem-123' not found",
  "details": {
    "resource_type": "memory",
    "resource_id": "mem-123"
  },
  "timestamp": "2023-10-27T12:00:00Z"
}
```

### HTTP状态码

| 状态码 | 说明 | 示例 |
|--------|------|------|
| **200** | 成功 | 请求成功处理 |
| **201** | 已创建 | 资源创建成功 |
| **400** | 请求错误 | 参数验证失败 |
| **401** | 未授权 | Token无效或过期 |
| **403** | 禁止访问 | 权限不足 |
| **404** | 未找到 | 资源不存在 |
| **409** | 冲突 | 资源已存在 |
| **422** | 无法处理 | 语义错误 |
| **429** | 请求过多 | 速率限制 |
| **500** | 服务器错误 | 内部错误 |
| **503** | 服务不可用 | 维护中 |

### 错误代码列表

| 错误代码 | HTTP状态 | 说明 |
|---------|----------|------|
| `INVALID_REQUEST` | 400 | 请求参数无效 |
| `UNAUTHORIZED` | 401 | 未授权访问 |
| `FORBIDDEN` | 403 | 权限不足 |
| `RESOURCE_NOT_FOUND` | 404 | 资源未找到 |
| `RESOURCE_CONFLICT` | 409 | 资源冲突 |
| `VALIDATION_ERROR` | 422 | 验证失败 |
| `RATE_LIMIT_EXCEEDED` | 429 | 超过速率限制 |
| `INTERNAL_ERROR` | 500 | 服务器内部错误 |
| `SERVICE_UNAVAILABLE` | 503 | 服务不可用 |

---

## 📚 SDK示例

### Python

```python
import requests

# 基础配置
BASE_URL = "http://localhost:8080"
TOKEN = "your-jwt-token"

headers = {
    "Authorization": f"Bearer {TOKEN}",
    "Content-Type": "application/json"
}

# 添加记忆
response = requests.post(
    f"{BASE_URL}/api/v1/memories",
    headers=headers,
    json={
        "agent_id": "agent-123",
        "content": "用户喜欢披萨",
        "importance": 0.8
    }
)
print(response.json())

# 搜索记忆
response = requests.post(
    f"{BASE_URL}/api/v1/memories/search",
    headers=headers,
    json={
        "query": "用户喜欢什么",
        "agent_id": "agent-123",
        "limit": 10
    }
)
print(response.json())
```

### JavaScript/TypeScript

```typescript
const BASE_URL = 'http://localhost:8080';
const TOKEN = 'your-jwt-token';

const headers = {
  'Authorization': `Bearer ${TOKEN}`,
  'Content-Type': 'application/json'
};

// 添加记忆
const addMemory = async () => {
  const response = await fetch(`${BASE_URL}/api/v1/memories`, {
    method: 'POST',
    headers,
    body: JSON.stringify({
      agent_id: 'agent-123',
      content: '用户喜欢披萨',
      importance: 0.8
    })
  });
  return await response.json();
};

// 搜索记忆
const searchMemories = async () => {
  const response = await fetch(`${BASE_URL}/api/v1/memories/search`, {
    method: 'POST',
    headers,
    body: JSON.stringify({
      query: '用户喜欢什么',
      agent_id: 'agent-123',
      limit: 10
    })
  });
  return await response.json();
};
```

### cURL

```bash
# 设置变量
export BASE_URL="http://localhost:8080"
export TOKEN="your-jwt-token"

# 添加记忆
curl -X POST "$BASE_URL/api/v1/memories" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "agent_id": "agent-123",
    "content": "用户喜欢披萨",
    "importance": 0.8
  }'

# 搜索记忆
curl -X POST "$BASE_URL/api/v1/memories/search" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "用户喜欢什么",
    "agent_id": "agent-123",
    "limit": 10
  }'
```

---

## 🔧 速率限制

### 默认限制

| 端点类型 | 限制 | 时间窗口 |
|---------|------|---------|
| **读取操作** | 1000请求 | 1分钟 |
| **写入操作** | 100请求 | 1分钟 |
| **搜索操作** | 500请求 | 1分钟 |

### 响应头

```http
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 999
X-RateLimit-Reset: 1635350400
```

### 超限响应

```json
{
  "code": "RATE_LIMIT_EXCEEDED",
  "message": "Rate limit exceeded. Try again in 60 seconds.",
  "retry_after": 60
}
```

---

## 🔗 相关资源

- **Swagger UI**: http://localhost:8080/swagger-ui
- **OpenAPI JSON**: http://localhost:8080/api-docs/openapi.json
- **健康检查**: http://localhost:8080/health
- **Metrics**: http://localhost:8080/metrics/prometheus

---

**文档版本**: v2.0.0  
**最后更新**: 2025-10-27  
**维护团队**: AgentMem API Team

