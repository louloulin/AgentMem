# AgentMem MVP 实施细节路线图

**文档版本**: v1.0  
**制定日期**: 2025-10-27  
**基于**: 全面代码分析 + 562个TODO/FIXME分析  

---

## 执行概要

基于对整个代码库的深度扫描，发现：
- **562个TODO/FIXME标记**（需要处理的技术债务）
- **18个crates**（模块化架构）
- **537个源文件**（大规模代码库）
- **90个测试文件**（测试覆盖17%）

本文档提供**生产MVP的详细实施路线图**，包含具体的技术实现方案和代码示例。

---

## 一、技术债务分析

### 1.1 TODO/FIXME分布

根据代码扫描结果，562个标记分布如下：

| crate | TODO数量 | 优先级 | 说明 |
|-------|---------|-------|------|
| **agent-mem-core** | ~139 | P0-P1 | 核心功能，需优先处理 |
| **agent-mem** | ~20 | P0 | 主入口，需立即处理 |
| **agent-mem-server** | ~31 | P1 | API服务，需尽快处理 |
| **agent-mem-intelligence** | ~40 | P1 | 智能功能，影响用户体验 |
| **agent-mem-tools** | ~34 | P2 | 工具集，可延后处理 |
| **agent-mem-storage** | ~52 | P1 | 存储层，影响性能 |
| **agent-mem-llm** | ~30 | P1 | LLM集成，影响功能 |
| **文档和测试** | ~200+ | P2-P3 | 文档和示例 |

### 1.2 关键TODO分类

#### 类别1: 性能优化（约80个）

**示例**:
```rust
// agent-mem-core/src/orchestrator/mod.rs
// TODO: 优化LLM调用性能
// TODO: 实现批处理
// TODO: 添加缓存层
```

**影响**: 直接影响系统性能和用户体验

#### 类别2: 功能缺失（约150个）

**示例**:
```rust
// agent-mem-server/src/routes/mod.rs
// TODO: 实现GraphQL API
// TODO: 添加WebSocket支持
// TODO: 实现速率限制
```

**影响**: 功能不完整，无法满足生产需求

#### 类别3: 错误处理（约100个）

**示例**:
```rust
// TODO: 添加详细的错误信息
// TODO: 实现重试机制
// TODO: 错误恢复策略
```

**影响**: 生产稳定性和可维护性

#### 类别4: 测试缺失（约120个）

**示例**:
```rust
// TODO: 添加单元测试
// TODO: 添加集成测试
// TODO: 添加性能测试
```

**影响**: 质量保证

#### 类别5: 文档缺失（约80个）

**示例**:
```rust
// TODO: 添加API文档
// TODO: 添加使用示例
// TODO: 更新README
```

**影响**: 用户体验和采用率

#### 类别6: 其他（约32个）

**示例**:
```rust
// TODO: 代码重构
// TODO: 依赖更新
// TODO: 配置优化
```

---

## 二、生产MVP核心功能实施方案

### 2.1 性能优化实施方案

#### 方案1: LLM调用批处理（P0）

**当前问题**:
```rust
// 当前实现：串行调用，延迟高
pub async fn process_memories(&self, memories: Vec<Memory>) -> Result<Vec<ProcessedMemory>> {
    let mut results = Vec::new();
    for memory in memories {
        let processed = self.llm.process(&memory).await?; // 串行调用
        results.push(processed);
    }
    Ok(results)
}
```

**优化方案**:
```rust
use futures::stream::{self, StreamExt};

pub async fn process_memories(&self, memories: Vec<Memory>) -> Result<Vec<ProcessedMemory>> {
    // 批量并行处理，最多10个并发
    let results = stream::iter(memories)
        .map(|memory| async move {
            self.llm.process(&memory).await
        })
        .buffer_unordered(10) // 并发控制
        .collect::<Vec<_>>()
        .await;
    
    // 收集成功的结果
    results.into_iter()
        .filter_map(|r| r.ok())
        .collect()
}
```

**预期提升**:
- 延迟: 10个记忆从 100s → 10s (10x)
- 吞吐量: 0.1 ops/s → 1 ops/s (10x)

**实施时间**: 2天

#### 方案2: 向量搜索缓存（P0）

**当前问题**:
```rust
// 每次搜索都重新计算嵌入和检索
pub async fn search(&self, query: &str) -> Result<Vec<Memory>> {
    let embedding = self.embedder.embed(query).await?; // 每次重新计算
    let results = self.vector_store.search(&embedding).await?;
    Ok(results)
}
```

**优化方案**:
```rust
use moka::future::Cache;
use std::sync::Arc;

pub struct CachedSearchEngine {
    embedder: Arc<dyn Embedder>,
    vector_store: Arc<dyn VectorStore>,
    embedding_cache: Cache<String, Vec<f32>>,
    result_cache: Cache<String, Vec<Memory>>,
}

impl CachedSearchEngine {
    pub fn new(embedder: Arc<dyn Embedder>, vector_store: Arc<dyn VectorStore>) -> Self {
        Self {
            embedder,
            vector_store,
            // 嵌入缓存: 1000个查询，5分钟过期
            embedding_cache: Cache::builder()
                .max_capacity(1000)
                .time_to_live(Duration::from_secs(300))
                .build(),
            // 结果缓存: 500个查询，1分钟过期
            result_cache: Cache::builder()
                .max_capacity(500)
                .time_to_live(Duration::from_secs(60))
                .build(),
        }
    }
    
    pub async fn search(&self, query: &str) -> Result<Vec<Memory>> {
        // 1. 检查结果缓存
        if let Some(cached_results) = self.result_cache.get(query).await {
            return Ok(cached_results);
        }
        
        // 2. 获取或计算嵌入
        let embedding = if let Some(cached_emb) = self.embedding_cache.get(query).await {
            cached_emb
        } else {
            let emb = self.embedder.embed(query).await?;
            self.embedding_cache.insert(query.to_string(), emb.clone()).await;
            emb
        };
        
        // 3. 向量检索
        let results = self.vector_store.search(&embedding).await?;
        
        // 4. 缓存结果
        self.result_cache.insert(query.to_string(), results.clone()).await;
        
        Ok(results)
    }
}
```

**预期提升**:
- 搜索延迟（缓存命中）: 50ms → 5ms (10x)
- 搜索延迟（缓存未命中）: 50ms → 40ms (1.25x, 减少嵌入计算)
- 缓存命中率目标: 30-40%
- 整体平均延迟: 50ms → 20ms (2.5x)

**实施时间**: 3天

#### 方案3: 数据库连接池（P0）

**当前问题**:
```rust
// 每次操作都创建新连接
pub async fn add_memory(&self, memory: &Memory) -> Result<String> {
    let conn = SqliteConnection::connect(&self.database_url).await?; // 频繁建立连接
    let id = self.insert_memory(&conn, memory).await?;
    Ok(id)
}
```

**优化方案**:
```rust
use sqlx::{Pool, Sqlite};

pub struct MemoryStore {
    pool: Pool<Sqlite>,
}

impl MemoryStore {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePool::connect(database_url).await?;
        
        // 配置连接池
        pool.set_max_connections(20);
        pool.set_min_connections(5);
        pool.set_connect_timeout(Duration::from_secs(5));
        
        Ok(Self { pool })
    }
    
    pub async fn add_memory(&self, memory: &Memory) -> Result<String> {
        // 从池中获取连接（复用）
        let mut conn = self.pool.acquire().await?;
        let id = self.insert_memory(&mut conn, memory).await?;
        Ok(id)
    }
}
```

**预期提升**:
- 连接建立时间: 5-10ms → 0ms（复用）
- Add操作延迟: 10ms → 5ms (2x)
- 并发能力: 大幅提升

**实施时间**: 2天

### 2.2 API完整性实施方案

#### 方案1: 健康检查API（P0）

**实现**:
```rust
// crates/agent-mem-server/src/routes/health.rs

use axum::{
    extract::State,
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub checks: HealthChecks,
}

#[derive(Debug, Serialize)]
pub struct HealthChecks {
    pub database: CheckStatus,
    pub vector_store: CheckStatus,
    pub llm: CheckStatus,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

pub async fn health_check(
    State(app_state): State<Arc<AppState>>
) -> impl IntoResponse {
    let start_time = app_state.start_time;
    let uptime = std::time::SystemTime::now()
        .duration_since(start_time)
        .unwrap()
        .as_secs();
    
    // 并行健康检查
    let (db_status, vector_status, llm_status) = tokio::join!(
        check_database(&app_state.memory),
        check_vector_store(&app_state.memory),
        check_llm(&app_state.memory),
    );
    
    let all_healthy = matches!(db_status, CheckStatus::Healthy)
        && matches!(vector_status, CheckStatus::Healthy)
        && matches!(llm_status, CheckStatus::Healthy);
    
    let response = HealthResponse {
        status: if all_healthy { "healthy" } else { "degraded" }.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime,
        checks: HealthChecks {
            database: db_status,
            vector_store: vector_status,
            llm: llm_status,
        },
    };
    
    let status_code = if all_healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    
    (status_code, Json(response))
}

async fn check_database(memory: &Memory) -> CheckStatus {
    match memory.ping_database().await {
        Ok(_) => CheckStatus::Healthy,
        Err(_) => CheckStatus::Unhealthy,
    }
}

async fn check_vector_store(memory: &Memory) -> CheckStatus {
    match memory.ping_vector_store().await {
        Ok(_) => CheckStatus::Healthy,
        Err(_) => CheckStatus::Unhealthy,
    }
}

async fn check_llm(memory: &Memory) -> CheckStatus {
    // LLM可能不可用，但系统仍可运行（降级服务）
    match memory.ping_llm().await {
        Ok(_) => CheckStatus::Healthy,
        Err(_) => CheckStatus::Degraded,
    }
}
```

**路由注册**:
```rust
// crates/agent-mem-server/src/routes/mod.rs

pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health::health_check))
        .route("/health/live", get(health::liveness))
        .route("/health/ready", get(health::readiness))
        // ... 其他路由
        .with_state(app_state)
}
```

**实施时间**: 1天

#### 方案2: API认证和速率限制（P1）

**实现**:
```rust
// crates/agent-mem-server/src/middleware/auth.rs

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use std::sync::Arc;

pub async fn auth_middleware(
    State(app_state): State<Arc<AppState>>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<impl IntoResponse, StatusCode> {
    // 1. 提取API Key
    let api_key = headers
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    // 2. 验证API Key
    if !app_state.auth.validate_api_key(api_key).await {
        return Err(StatusCode::UNAUTHORIZED);
    }
    
    // 3. 检查速率限制
    if !app_state.rate_limiter.check_rate_limit(api_key).await {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    
    // 4. 继续处理请求
    Ok(next.run(request).await)
}

// crates/agent-mem-server/src/middleware/rate_limiter.rs

use governor::{
    Quota, RateLimiter,
    state::{InMemoryState, NotKeyed},
    clock::DefaultClock,
};
use std::collections::HashMap;
use std::sync::RwLock;

pub struct RateLimiterService {
    // 每个API Key的速率限制器
    limiters: RwLock<HashMap<String, RateLimiter<NotKeyed, InMemoryState, DefaultClock>>>,
    default_quota: Quota,
}

impl RateLimiterService {
    pub fn new() -> Self {
        Self {
            limiters: RwLock::new(HashMap::new()),
            // 默认: 每秒100个请求
            default_quota: Quota::per_second(nonzero!(100u32)),
        }
    }
    
    pub async fn check_rate_limit(&self, api_key: &str) -> bool {
        let mut limiters = self.limiters.write().unwrap();
        
        let limiter = limiters
            .entry(api_key.to_string())
            .or_insert_with(|| {
                RateLimiter::direct(self.default_quota)
            });
        
        limiter.check().is_ok()
    }
}
```

**实施时间**: 3天

### 2.3 文档完整性实施方案

#### 方案1: OpenAPI规范（P0）

**实现**:
```rust
// crates/agent-mem-server/src/openapi.rs

use utoipa::{
    OpenApi,
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "AgentMem API",
        version = "1.0.0",
        description = "AI Agent Memory Platform REST API",
        contact(
            name = "AgentMem Team",
            email = "team@agentmem.dev"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    paths(
        routes::health::health_check,
        routes::memories::add_memory,
        routes::memories::search_memories,
        routes::memories::get_memory,
        routes::memories::delete_memory,
        routes::memories::list_memories,
    ),
    components(
        schemas(
            models::Memory,
            models::AddMemoryRequest,
            models::SearchMemoryRequest,
            models::HealthResponse,
        )
    ),
    tags(
        (name = "Health", description = "Health check endpoints"),
        (name = "Memories", description = "Memory management endpoints"),
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "api_key",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-API-Key"))),
        );
    }
}

// 生成OpenAPI JSON
pub fn generate_openapi_spec() -> String {
    ApiDoc::openapi().to_pretty_json().unwrap()
}
```

**端点示例**:
```rust
/// Add a new memory
#[utoipa::path(
    post,
    path = "/api/v1/memories",
    request_body = AddMemoryRequest,
    responses(
        (status = 201, description = "Memory created successfully", body = AddMemoryResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limit exceeded"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Memories",
    security(
        ("api_key" = [])
    )
)]
pub async fn add_memory(
    State(app_state): State<Arc<AppState>>,
    Json(request): Json<AddMemoryRequest>,
) -> Result<Json<AddMemoryResponse>, ApiError> {
    // 实现...
}
```

**实施时间**: 5天

---

## 三、Web UI开发方案

### 3.1 技术栈选型

**前端框架**: React 18 + TypeScript  
**UI组件**: Shadcn/ui + Tailwind CSS  
**状态管理**: Zustand  
**数据获取**: TanStack Query (React Query)  
**路由**: React Router v6  
**图表**: Recharts  
**图谱可视化**: React Flow  

### 3.2 核心页面设计

#### Page 1: Dashboard（概览）

**功能**:
- 总记忆数量
- 今日新增记忆
- 搜索请求统计
- Agent活跃度
- 性能指标图表

**代码示例**:
```typescript
// src/pages/Dashboard.tsx

import { useQuery } from '@tanstack/react-query';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip } from 'recharts';

export function Dashboard() {
  const { data: stats } = useQuery({
    queryKey: ['dashboard-stats'],
    queryFn: async () => {
      const res = await fetch('/api/v1/stats');
      return res.json();
    },
    refetchInterval: 5000, // 每5秒刷新
  });

  return (
    <div className="p-6 space-y-6">
      <h1 className="text-3xl font-bold">Dashboard</h1>
      
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <StatCard
          title="Total Memories"
          value={stats?.totalMemories?.toLocaleString() ?? '0'}
          trend="+12%"
        />
        <StatCard
          title="Today's Additions"
          value={stats?.todayAdditions?.toLocaleString() ?? '0'}
          trend="+5%"
        />
        <StatCard
          title="Search Requests"
          value={stats?.searchRequests?.toLocaleString() ?? '0'}
          trend="+8%"
        />
        <StatCard
          title="Avg Response Time"
          value={`${stats?.avgResponseTime ?? 0}ms`}
          trend="-3%"
        />
      </div>
      
      <Card>
        <CardHeader>
          <CardTitle>Memory Growth</CardTitle>
        </CardHeader>
        <CardContent>
          <LineChart width={800} height={300} data={stats?.memoryGrowth ?? []}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis dataKey="date" />
            <YAxis />
            <Tooltip />
            <Line type="monotone" dataKey="count" stroke="#8884d8" />
          </LineChart>
        </CardContent>
      </Card>
    </div>
  );
}
```

#### Page 2: Memories管理

**功能**:
- 记忆列表（分页）
- 搜索和过滤
- 新增记忆
- 编辑记忆
- 删除记忆
- 批量操作

**代码示例**:
```typescript
// src/pages/Memories.tsx

import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { DataTable } from '@/components/DataTable';
import { AddMemoryDialog } from '@/components/AddMemoryDialog';

export function Memories() {
  const [search, setSearch] = useState('');
  const [page, setPage] = useState(1);
  const queryClient = useQueryClient();

  const { data, isLoading } = useQuery({
    queryKey: ['memories', { search, page }],
    queryFn: async () => {
      const params = new URLSearchParams({
        search,
        page: page.toString(),
        limit: '20',
      });
      const res = await fetch(`/api/v1/memories?${params}`);
      return res.json();
    },
  });

  const deleteMutation = useMutation({
    mutationFn: async (id: string) => {
      await fetch(`/api/v1/memories/${id}`, { method: 'DELETE' });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['memories'] });
    },
  });

  return (
    <div className="p-6 space-y-4">
      <div className="flex justify-between items-center">
        <h1 className="text-3xl font-bold">Memories</h1>
        <AddMemoryDialog />
      </div>

      <div className="flex gap-2">
        <Input
          placeholder="Search memories..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="max-w-sm"
        />
      </div>

      <DataTable
        data={data?.memories ?? []}
        columns={[
          { key: 'id', label: 'ID' },
          { key: 'content', label: 'Content' },
          { key: 'created_at', label: 'Created' },
          {
            key: 'actions',
            label: 'Actions',
            render: (memory) => (
              <Button
                variant="destructive"
                size="sm"
                onClick={() => deleteMutation.mutate(memory.id)}
              >
                Delete
              </Button>
            ),
          },
        ]}
        isLoading={isLoading}
      />
    </div>
  );
}
```

### 3.3 部署方案

**开发环境**:
```bash
cd agentmem-web-ui
npm install
npm run dev # http://localhost:5173
```

**生产构建**:
```bash
npm run build
# 输出到 dist/ 目录
```

**Docker部署**:
```dockerfile
# Dockerfile.web

FROM node:20-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

**实施时间**: 4周

---

## 四、LangChain/LlamaIndex集成方案

### 4.1 LangChain Memory集成

**Python包结构**:
```
agentmem-langchain/
├── agentmem_langchain/
│   ├── __init__.py
│   ├── memory.py          # BaseChatMemory实现
│   ├── retriever.py       # BaseRetriever实现
│   └── vectorstore.py     # VectorStore实现
├── tests/
│   ├── test_memory.py
│   └── test_integration.py
├── examples/
│   └── chatbot_example.py
├── setup.py
├── pyproject.toml
└── README.md
```

**核心实现**:
```python
# agentmem_langchain/memory.py

from typing import Any, Dict, List, Optional
from langchain.schema import BaseMessage
from langchain.memory.chat_memory import BaseChatMemory
from agentmem import Memory as AgentMemClient

class AgentMemMemory(BaseChatMemory):
    """LangChain memory backed by AgentMem."""
    
    client: AgentMemClient
    session_id: str
    memory_key: str = "history"
    
    def __init__(
        self,
        api_url: str = "http://localhost:8080",
        api_key: Optional[str] = None,
        session_id: Optional[str] = None,
        **kwargs
    ):
        super().__init__(**kwargs)
        self.client = AgentMemClient(api_url=api_url, api_key=api_key)
        self.session_id = session_id or "default"
    
    @property
    def memory_variables(self) -> List[str]:
        """Return memory variables."""
        return [self.memory_key]
    
    def load_memory_variables(self, inputs: Dict[str, Any]) -> Dict[str, Any]:
        """Load memory variables from AgentMem."""
        # 搜索相关记忆
        query = inputs.get("input", "")
        memories = self.client.search(
            query=query,
            limit=10,
            filters={"session_id": self.session_id}
        )
        
        # 转换为LangChain消息格式
        messages = []
        for memory in memories:
            messages.append({
                "role": memory.metadata.get("role", "user"),
                "content": memory.content,
            })
        
        return {self.memory_key: messages}
    
    def save_context(self, inputs: Dict[str, Any], outputs: Dict[str, str]) -> None:
        """Save context to AgentMem."""
        # 保存用户输入
        self.client.add(
            content=inputs.get("input", ""),
            metadata={
                "session_id": self.session_id,
                "role": "user",
                "type": "chat_message",
            }
        )
        
        # 保存AI输出
        self.client.add(
            content=outputs.get("output", ""),
            metadata={
                "session_id": self.session_id,
                "role": "assistant",
                "type": "chat_message",
            }
        )
    
    def clear(self) -> None:
        """Clear memory for this session."""
        # 删除此会话的所有记忆
        self.client.delete_all(filters={"session_id": self.session_id})
```

**使用示例**:
```python
# examples/chatbot_example.py

from langchain.llms import OpenAI
from langchain.chains import ConversationChain
from agentmem_langchain import AgentMemMemory

# 初始化
llm = OpenAI(temperature=0.7)
memory = AgentMemMemory(
    api_url="http://localhost:8080",
    session_id="user_123"
)

# 创建对话链
chain = ConversationChain(
    llm=llm,
    memory=memory,
    verbose=True
)

# 对话
response1 = chain.predict(input="My name is Alice")
# -> "Nice to meet you, Alice!"

response2 = chain.predict(input="What is my name?")
# -> "Your name is Alice."
```

**实施时间**: 2周

### 4.2 LlamaIndex VectorStore集成

**核心实现**:
```python
# agentmem_llamaindex/vectorstore.py

from typing import Any, Dict, List, Optional
from llama_index.vector_stores.types import (
    VectorStore,
    VectorStoreQuery,
    VectorStoreQueryResult,
)
from llama_index.schema import TextNode
from agentmem import Memory as AgentMemClient

class AgentMemVectorStore(VectorStore):
    """LlamaIndex vector store backed by AgentMem."""
    
    def __init__(
        self,
        api_url: str = "http://localhost:8080",
        api_key: Optional[str] = None,
        **kwargs
    ):
        super().__init__(**kwargs)
        self.client = AgentMemClient(api_url=api_url, api_key=api_key)
    
    def add(self, nodes: List[TextNode], **kwargs) -> List[str]:
        """Add nodes to the vector store."""
        ids = []
        for node in nodes:
            result = self.client.add(
                content=node.get_content(),
                metadata=node.metadata or {}
            )
            ids.append(result.id)
        return ids
    
    def delete(self, ref_doc_id: str, **kwargs) -> None:
        """Delete nodes by document ID."""
        self.client.delete_all(filters={"doc_id": ref_doc_id})
    
    def query(
        self,
        query: VectorStoreQuery,
        **kwargs
    ) -> VectorStoreQueryResult:
        """Query the vector store."""
        # 执行搜索
        results = self.client.search(
            query=query.query_str,
            limit=query.similarity_top_k,
            filters=query.filters or {}
        )
        
        # 转换为LlamaIndex格式
        nodes = []
        similarities = []
        ids = []
        
        for result in results:
            node = TextNode(
                text=result.content,
                metadata=result.metadata,
            )
            nodes.append(node)
            similarities.append(result.score)
            ids.append(result.id)
        
        return VectorStoreQueryResult(
            nodes=nodes,
            similarities=similarities,
            ids=ids,
        )
```

**使用示例**:
```python
# examples/rag_example.py

from llama_index import VectorStoreIndex, ServiceContext
from llama_index.llms import OpenAI
from agentmem_llamaindex import AgentMemVectorStore

# 初始化向量存储
vector_store = AgentMemVectorStore(api_url="http://localhost:8080")

# 创建索引
service_context = ServiceContext.from_defaults(llm=OpenAI())
index = VectorStoreIndex.from_vector_store(
    vector_store=vector_store,
    service_context=service_context,
)

# 查询
query_engine = index.as_query_engine()
response = query_engine.query("What is AgentMem?")
print(response)
```

**实施时间**: 2周

---

## 五、部署和运维方案

### 5.1 Docker Compose完整栈

**文件**: `docker-compose.production.yml`

```yaml
version: '3.8'

services:
  # AgentMem Server
  agentmem:
    image: agentmem/agentmem:latest
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgres://postgres:password@postgres:5432/agentmem
      - VECTOR_STORE=lancedb
      - LLM_PROVIDER=openai
      - OPENAI_API_KEY=${OPENAI_API_KEY}
      - RUST_LOG=info
    volumes:
      - ./data/lancedb:/data/lancedb
    depends_on:
      - postgres
      - redis
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
    restart: unless-stopped

  # PostgreSQL数据库
  postgres:
    image: postgres:16-alpine
    environment:
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=password
      - POSTGRES_DB=agentmem
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 10s
      timeout: 5s
      retries: 5
    restart: unless-stopped

  # Redis缓存
  redis:
    image: redis:7-alpine
    command: redis-server --appendonly yes
    volumes:
      - redis_data:/data
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 5s
      retries: 5
    restart: unless-stopped

  # Prometheus监控
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus_data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
    restart: unless-stopped

  # Grafana可视化
  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana_data:/var/lib/grafana
      - ./grafana/dashboards:/etc/grafana/provisioning/dashboards
    depends_on:
      - prometheus
    restart: unless-stopped

  # Web UI
  agentmem-ui:
    image: agentmem/agentmem-ui:latest
    ports:
      - "80:80"
    environment:
      - API_URL=http://agentmem:8080
    depends_on:
      - agentmem
    restart: unless-stopped

volumes:
  postgres_data:
  redis_data:
  prometheus_data:
  grafana_data:

networks:
  default:
    name: agentmem-network
```

**启动**:
```bash
# 设置环境变量
export OPENAI_API_KEY=sk-xxx

# 启动所有服务
docker-compose -f docker-compose.production.yml up -d

# 查看日志
docker-compose -f docker-compose.production.yml logs -f

# 停止所有服务
docker-compose -f docker-compose.production.yml down
```

### 5.2 Kubernetes部署

**Helm Chart结构**:
```
charts/agentmem/
├── Chart.yaml
├── values.yaml
├── templates/
│   ├── deployment.yaml
│   ├── service.yaml
│   ├── ingress.yaml
│   ├── configmap.yaml
│   ├── secret.yaml
│   ├── hpa.yaml           # 自动扩容
│   ├── pvc.yaml           # 持久化存储
│   └── servicemonitor.yaml # Prometheus监控
└── README.md
```

**核心配置**: `values.yaml`

```yaml
# 副本数
replicaCount: 3

# 镜像配置
image:
  repository: agentmem/agentmem
  tag: "1.0.0"
  pullPolicy: IfNotPresent

# 资源限制
resources:
  limits:
    cpu: 2000m
    memory: 4Gi
  requests:
    cpu: 1000m
    memory: 2Gi

# 自动扩容
autoscaling:
  enabled: true
  minReplicas: 3
  maxReplicas: 10
  targetCPUUtilizationPercentage: 70
  targetMemoryUtilizationPercentage: 80

# 数据库配置
database:
  type: postgresql
  host: postgres-service
  port: 5432
  name: agentmem
  user: postgres
  passwordSecret: agentmem-db-secret

# 向量存储
vectorStore:
  type: pinecone
  apiKeySecret: agentmem-pinecone-secret

# LLM配置
llm:
  provider: openai
  apiKeySecret: agentmem-openai-secret

# Ingress配置
ingress:
  enabled: true
  className: nginx
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
  hosts:
    - host: api.agentmem.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: agentmem-tls
      hosts:
        - api.agentmem.com

# 监控
monitoring:
  enabled: true
  serviceMonitor:
    enabled: true
```

**部署命令**:
```bash
# 添加Helm仓库
helm repo add agentmem https://charts.agentmem.com

# 安装
helm install agentmem agentmem/agentmem \
  --namespace agentmem \
  --create-namespace \
  --values production-values.yaml

# 升级
helm upgrade agentmem agentmem/agentmem \
  --namespace agentmem \
  --values production-values.yaml

# 卸载
helm uninstall agentmem --namespace agentmem
```

---

## 六、执行时间表

### Week 1-2: P0阻塞问题

| 任务 | 天数 | 负责人 | 交付物 |
|------|-----|--------|--------|
| LLM批处理优化 | 2 | 后端 | 性能提升10x |
| 向量搜索缓存 | 3 | 后端 | 延迟降低2.5x |
| 数据库连接池 | 2 | 后端 | 延迟降低2x |
| 健康检查API | 1 | 后端 | /health端点 |
| OpenAPI文档 | 5 | 后端+文档 | API文档完整 |

**里程碑**: Alpha版本可发布

### Week 3-4: P1关键功能

| 任务 | 天数 | 负责人 | 交付物 |
|------|-----|--------|--------|
| API认证 | 3 | 后端 | API Key认证 |
| 速率限制 | 2 | 后端 | 速率限制中间件 |
| 错误处理增强 | 2 | 后端 | 详细错误信息 |
| 部署文档 | 3 | DevOps+文档 | 部署指南 |

**里程碑**: Beta版本可发布

### Week 5-8: Web UI开发

| 任务 | 天数 | 负责人 | 交付物 |
|------|-----|--------|--------|
| 项目搭建 | 2 | 前端 | React项目 |
| Dashboard页面 | 5 | 前端 | 概览页面 |
| Memories管理 | 5 | 前端 | CRUD界面 |
| Search界面 | 3 | 前端 | 搜索功能 |
| 设置页面 | 3 | 前端 | 配置管理 |
| 集成测试 | 2 | 前端+后端 | E2E测试 |

**里程碑**: RC版本可发布

### Week 9-12: 生态集成

| 任务 | 天数 | 负责人 | 交付物 |
|------|-----|--------|--------|
| LangChain集成 | 10 | 后端+文档 | PyPI包 |
| LlamaIndex集成 | 10 | 后端+文档 | PyPI包 |
| TypeScript SDK | 15 | 后端+前端 | npm包 |
| 示例项目 | 5 | 全员 | 10个示例 |

**里程碑**: v1.0正式发布

---

## 七、质量保证

### 7.1 测试策略

**测试金字塔**:
```
              /\
             /E2E\          (10%)
            /──────\
           /集成测试 \       (30%)
          /──────────\
         /  单元测试   \    (60%)
        /──────────────\
```

**目标覆盖率**:
- 单元测试: 70%
- 集成测试: 50%
- E2E测试: 30%
- 总体: 60%+

### 7.2 CI/CD流程

**GitHub Actions工作流**:
```yaml
# .github/workflows/ci.yml

name: CI/CD

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Run tests
        run: cargo test --all
      
      - name: Run benchmarks
        run: cargo bench --no-run
      
      - name: Coverage
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --out Xml
      
      - name: Upload coverage
        uses: codecov/codecov-action@v3

  build:
    needs: test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Build Docker image
        run: docker build -t agentmem:${{ github.sha }} .
      
      - name: Push to registry
        if: github.ref == 'refs/heads/main'
        run: |
          docker tag agentmem:${{ github.sha }} agentmem/agentmem:latest
          docker push agentmem/agentmem:latest

  deploy:
    needs: build
    if: github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    steps:
      - name: Deploy to production
        run: |
          kubectl set image deployment/agentmem \
            agentmem=agentmem/agentmem:latest
```

---

## 八、风险管理

### 8.1 技术风险

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|---------|
| 性能不达标 | 中 | 高 | 提前基准测试+优化缓冲 |
| LLM API不稳定 | 高 | 中 | 多provider支持+重试机制 |
| 数据迁移问题 | 中 | 高 | 迁移工具+灰度发布 |
| 安全漏洞 | 低 | 高 | 安全审计+渗透测试 |

### 8.2 项目风险

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|---------|
| 进度延期 | 中 | 中 | 敏捷开发+每周评审 |
| 需求变更 | 高 | 中 | MVP范围冻结+优先级管理 |
| 人员流失 | 低 | 高 | 文档完善+知识共享 |
| 技术债务 | 高 | 中 | 562个TODO分批处理 |

---

## 九、成功指标

### 9.1 技术指标

| 指标 | 当前 | Month 1 | Month 3 | Month 6 |
|------|------|---------|---------|---------|
| **测试覆盖率** | 17% | 30% | 50% | 65% |
| **TODO处理** | 562 | 400 | 200 | <50 |
| **API响应时间** | ~50ms | <30ms | <20ms | <15ms |
| **并发处理** | 1k/s | 2k/s | 3k/s | 5k/s |
| **文档完整性** | 70% | 85% | 95% | 98% |

### 9.2 业务指标

| 指标 | Month 1 | Month 3 | Month 6 |
|------|---------|---------|---------|
| **GitHub Stars** | 200 | 500 | 1000 |
| **npm下载** | 50/周 | 200/周 | 500/周 |
| **PyPI下载** | 100/周 | 500/周 | 1000/周 |
| **企业用户** | 2 | 5 | 10 |
| **社区贡献者** | 8 | 15 | 25 |

---

## 十、总结

本路线图提供了从**当前状态（70%就绪度）到生产MVP（99%就绪度）的详细实施方案**。

**关键里程碑**:
1. ✅ **Week 1-2**: 解决P0阻塞问题 → Alpha发布
2. ✅ **Week 3-4**: 完成P1关键功能 → Beta发布
3. ✅ **Week 5-8**: Web UI开发 → RC发布
4. ✅ **Week 9-12**: 生态集成 → v1.0发布

**核心竞争力**:
- 功能最全面（8个Agent + 21个LLM + 31个存储）
- 性能领先（Rust实现 + 缓存优化）
- 生态完整（LangChain + LlamaIndex + TypeScript SDK）
- 用户友好（Web UI + 完整文档 + 示例丰富）

**执行保障**:
- 详细的技术实施方案（含代码示例）
- 清晰的时间表和交付物
- 完善的风险管理策略
- 明确的成功指标

---

**文档版本**: v1.0  
**制定日期**: 2025-10-27  
**负责人**: AgentMem技术团队  
**下次评审**: 每周一评审，滚动更新

