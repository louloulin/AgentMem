# AgentMem API 参考文档

**版本:** 2.0  
**基础URL:** `https://api.agentmem.com/v2`  
**认证方式:** Bearer Token (JWT)

## 概述

AgentMem API 提供了完整的记忆管理功能，包括记忆的创建、检索、更新、删除以及智能搜索等功能。所有 API 都遵循 RESTful 设计原则。

## 认证

所有 API 请求都需要在 Header 中包含有效的 JWT 令牌：

```http
Authorization: Bearer <your-jwt-token>
```

### 获取访问令牌

```http
POST /auth/login
Content-Type: application/json

{
  "username": "your-username",
  "password": "your-password"
}
```

**响应:**
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "refresh_token": "refresh-token-here"
}
```

## 记忆管理 API

### 1. 创建记忆

创建一个新的记忆条目。

```http
POST /memories
Content-Type: application/json
Authorization: Bearer <token>

{
  "content": "今天学习了 Rust 的所有权机制",
  "memory_type": "episodic",
  "importance": 0.8,
  "tags": ["学习", "Rust", "编程"],
  "metadata": {
    "source": "学习笔记",
    "category": "技术"
  },
  "context": {
    "location": "办公室",
    "time_of_day": "morning",
    "mood": "focused"
  }
}
```

**响应:**
```json
{
  "id": "mem_1234567890abcdef",
  "content": "今天学习了 Rust 的所有权机制",
  "memory_type": "episodic",
  "importance": 0.8,
  "tags": ["学习", "Rust", "编程"],
  "metadata": {
    "source": "学习笔记",
    "category": "技术"
  },
  "context": {
    "location": "办公室",
    "time_of_day": "morning",
    "mood": "focused"
  },
  "created_at": "2025-09-04T12:00:00Z",
  "updated_at": "2025-09-04T12:00:00Z",
  "access_count": 0,
  "last_accessed": null
}
```

### 2. 获取记忆

根据 ID 获取特定的记忆。

```http
GET /memories/{memory_id}
Authorization: Bearer <token>
```

**响应:**
```json
{
  "id": "mem_1234567890abcdef",
  "content": "今天学习了 Rust 的所有权机制",
  "memory_type": "episodic",
  "importance": 0.8,
  "tags": ["学习", "Rust", "编程"],
  "metadata": {
    "source": "学习笔记",
    "category": "技术"
  },
  "context": {
    "location": "办公室",
    "time_of_day": "morning",
    "mood": "focused"
  },
  "created_at": "2025-09-04T12:00:00Z",
  "updated_at": "2025-09-04T12:00:00Z",
  "access_count": 5,
  "last_accessed": "2025-09-04T15:30:00Z"
}
```

### 3. 更新记忆

更新现有记忆的内容或元数据。

```http
PUT /memories/{memory_id}
Content-Type: application/json
Authorization: Bearer <token>

{
  "content": "今天深入学习了 Rust 的所有权机制，包括借用和生命周期",
  "importance": 0.9,
  "tags": ["学习", "Rust", "编程", "所有权"],
  "metadata": {
    "source": "学习笔记",
    "category": "技术",
    "updated_reason": "补充详细信息"
  }
}
```

### 4. 删除记忆

删除指定的记忆。

```http
DELETE /memories/{memory_id}
Authorization: Bearer <token>
```

**响应:**
```json
{
  "message": "Memory deleted successfully",
  "deleted_at": "2025-09-04T16:00:00Z"
}
```

### 5. 批量操作

批量创建、更新或删除记忆。

```http
POST /memories/batch
Content-Type: application/json
Authorization: Bearer <token>

{
  "operation": "create",
  "memories": [
    {
      "content": "记忆内容1",
      "memory_type": "episodic",
      "importance": 0.7
    },
    {
      "content": "记忆内容2",
      "memory_type": "semantic",
      "importance": 0.8
    }
  ]
}
```

## 搜索 API

### 1. 基础搜索

根据关键词搜索记忆。

```http
GET /search?q=Rust&limit=10&offset=0
Authorization: Bearer <token>
```

**查询参数:**
- `q`: 搜索关键词
- `limit`: 返回结果数量限制 (默认: 20, 最大: 100)
- `offset`: 分页偏移量 (默认: 0)
- `memory_type`: 记忆类型过滤 (episodic, semantic, procedural)
- `min_importance`: 最小重要性分数 (0.0-1.0)
- `tags`: 标签过滤 (逗号分隔)

**响应:**
```json
{
  "results": [
    {
      "id": "mem_1234567890abcdef",
      "content": "今天学习了 Rust 的所有权机制",
      "memory_type": "episodic",
      "importance": 0.8,
      "relevance_score": 0.95,
      "tags": ["学习", "Rust", "编程"],
      "created_at": "2025-09-04T12:00:00Z",
      "snippet": "今天学习了 <mark>Rust</mark> 的所有权机制"
    }
  ],
  "total": 1,
  "limit": 10,
  "offset": 0,
  "query_time_ms": 45
}
```

### 2. 高级搜索

使用复杂查询条件进行搜索。

```http
POST /search/advanced
Content-Type: application/json
Authorization: Bearer <token>

{
  "query": {
    "text": "Rust 编程",
    "memory_types": ["episodic", "semantic"],
    "importance_range": [0.5, 1.0],
    "date_range": {
      "start": "2025-09-01T00:00:00Z",
      "end": "2025-09-30T23:59:59Z"
    },
    "tags": {
      "include": ["编程", "学习"],
      "exclude": ["废弃"]
    },
    "metadata_filters": {
      "category": "技术",
      "source": "学习笔记"
    }
  },
  "options": {
    "search_type": "hybrid",  // exact, fuzzy, semantic, hybrid
    "include_similar": true,
    "similarity_threshold": 0.7,
    "max_results": 50,
    "sort_by": "relevance",  // relevance, importance, date, access_count
    "sort_order": "desc"
  }
}
```

### 3. 语义搜索

基于语义相似性搜索记忆。

```http
POST /search/semantic
Content-Type: application/json
Authorization: Bearer <token>

{
  "query": "如何管理内存安全",
  "similarity_threshold": 0.7,
  "limit": 10,
  "include_context": true
}
```

### 4. 相关记忆推荐

根据当前记忆推荐相关记忆。

```http
GET /memories/{memory_id}/related?limit=5
Authorization: Bearer <token>
```

## 智能功能 API

### 1. 记忆总结

生成记忆内容的智能总结。

```http
POST /memories/{memory_id}/summarize
Content-Type: application/json
Authorization: Bearer <token>

{
  "summary_type": "brief",  // brief, detailed, key_points
  "max_length": 200
}
```

**响应:**
```json
{
  "summary": "学习了 Rust 编程语言的核心概念：所有权机制，这是 Rust 内存安全的基础。",
  "key_points": [
    "所有权机制是 Rust 的核心特性",
    "确保内存安全",
    "避免数据竞争"
  ],
  "generated_at": "2025-09-04T16:00:00Z"
}
```

### 2. 记忆分析

分析记忆的情感、主题等特征。

```http
POST /memories/{memory_id}/analyze
Authorization: Bearer <token>
```

**响应:**
```json
{
  "sentiment": {
    "polarity": 0.8,
    "subjectivity": 0.6,
    "label": "positive"
  },
  "topics": [
    {
      "topic": "编程学习",
      "confidence": 0.9
    },
    {
      "topic": "技术概念",
      "confidence": 0.8
    }
  ],
  "entities": [
    {
      "text": "Rust",
      "type": "TECHNOLOGY",
      "confidence": 0.95
    }
  ],
  "complexity_score": 0.7
}
```

### 3. 智能标签建议

为记忆内容建议合适的标签。

```http
POST /memories/{memory_id}/suggest-tags
Authorization: Bearer <token>
```

**响应:**
```json
{
  "suggested_tags": [
    {
      "tag": "内存管理",
      "confidence": 0.9,
      "reason": "内容涉及内存安全概念"
    },
    {
      "tag": "系统编程",
      "confidence": 0.8,
      "reason": "Rust 是系统编程语言"
    }
  ]
}
```

## 统计和分析 API

### 1. 记忆统计

获取记忆的统计信息。

```http
GET /stats/memories
Authorization: Bearer <token>
```

**响应:**
```json
{
  "total_memories": 1250,
  "by_type": {
    "episodic": 800,
    "semantic": 350,
    "procedural": 100
  },
  "by_importance": {
    "high": 125,
    "medium": 750,
    "low": 375
  },
  "recent_activity": {
    "created_today": 15,
    "accessed_today": 45,
    "updated_today": 8
  },
  "top_tags": [
    {"tag": "学习", "count": 200},
    {"tag": "工作", "count": 150},
    {"tag": "编程", "count": 120}
  ]
}
```

### 2. 使用分析

分析记忆的使用模式。

```http
GET /stats/usage?period=30d
Authorization: Bearer <token>
```

**响应:**
```json
{
  "period": "30d",
  "total_accesses": 2500,
  "unique_memories_accessed": 800,
  "average_accesses_per_memory": 3.1,
  "most_accessed": [
    {
      "memory_id": "mem_abc123",
      "access_count": 25,
      "title": "Rust 所有权机制"
    }
  ],
  "access_patterns": {
    "by_hour": [/* 24小时访问分布 */],
    "by_day": [/* 7天访问分布 */],
    "by_memory_type": {
      "episodic": 1500,
      "semantic": 800,
      "procedural": 200
    }
  }
}
```

## 错误处理

API 使用标准的 HTTP 状态码和结构化的错误响应。

### 错误响应格式

```json
{
  "error": {
    "code": "MEMORY_NOT_FOUND",
    "message": "The requested memory was not found",
    "details": {
      "memory_id": "mem_invalid123",
      "timestamp": "2025-09-04T16:00:00Z"
    }
  }
}
```

### 常见错误码

| HTTP状态码 | 错误码 | 描述 |
|-----------|--------|------|
| 400 | INVALID_REQUEST | 请求参数无效 |
| 401 | UNAUTHORIZED | 未授权访问 |
| 403 | FORBIDDEN | 权限不足 |
| 404 | MEMORY_NOT_FOUND | 记忆不存在 |
| 409 | MEMORY_CONFLICT | 记忆冲突 |
| 422 | VALIDATION_ERROR | 数据验证失败 |
| 429 | RATE_LIMIT_EXCEEDED | 请求频率超限 |
| 500 | INTERNAL_ERROR | 服务器内部错误 |

## 速率限制

API 实施速率限制以确保服务稳定性：

- **认证用户:** 1000 请求/小时
- **搜索 API:** 100 请求/分钟
- **批量操作:** 10 请求/分钟

超出限制时，API 将返回 429 状态码和以下响应头：

```http
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1693843200
Retry-After: 3600
```

## SDK 和客户端库

我们提供多种语言的 SDK：

- **Rust:** `agentmem-client`
- **Python:** `agentmem-python`
- **JavaScript/TypeScript:** `@agentmem/client`
- **Go:** `github.com/agentmem/go-client`

### Rust SDK 示例

```rust
use agentmem_client::{AgentMemClient, Memory, MemoryType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AgentMemClient::new("https://api.agentmem.com/v2")
        .with_token("your-jwt-token");
    
    // 创建记忆
    let memory = Memory::builder()
        .content("学习 Rust 异步编程")
        .memory_type(MemoryType::Episodic)
        .importance(0.8)
        .tags(vec!["学习", "Rust", "异步"])
        .build();
    
    let created = client.create_memory(memory).await?;
    println!("Created memory: {}", created.id);
    
    // 搜索记忆
    let results = client.search("Rust").limit(10).execute().await?;
    for result in results.memories {
        println!("Found: {}", result.content);
    }
    
    Ok(())
}
```

## Webhook 支持

AgentMem 支持 Webhook 来实时通知记忆相关事件。

### 配置 Webhook

```http
POST /webhooks
Content-Type: application/json
Authorization: Bearer <token>

{
  "url": "https://your-app.com/webhooks/agentmem",
  "events": ["memory.created", "memory.updated", "memory.deleted"],
  "secret": "your-webhook-secret"
}
```

### Webhook 事件格式

```json
{
  "event": "memory.created",
  "timestamp": "2025-09-04T16:00:00Z",
  "data": {
    "memory": {
      "id": "mem_1234567890abcdef",
      "content": "新创建的记忆内容",
      "memory_type": "episodic",
      "importance": 0.8
    }
  }
}
```

---

**文档版本:** 2.0  
**最后更新:** 2025年9月4日  
**支持联系:** api-support@agentmem.com
