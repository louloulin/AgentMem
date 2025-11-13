# AgentMem 技术文档

## 目录

1. [系统概述](#1-系统概述)
2. [整体设计](#2-整体设计)
3. [系统架构](#3-系统架构)
4. [核心模块](#4-核心模块)
5. [API接口](#5-api接口)
6. [插件系统](#6-插件系统)
7. [使用指南](#7-使用指南)
8. [性能指标](#8-性能指标)
9. [部署指南](#9-部署指南)
10. [开发指南](#10-开发指南)

---

## 1. 系统概述

### 1.1 项目简介

AgentMem 是一个基于 Rust 开发的企业级智能记忆管理平台，专为 AI 代理和大语言模型应用设计。系统采用模块化架构，支持 WASM 插件系统、多模态处理、图记忆网络和 Mem0 兼容 API。

### 1.2 核心特性

- **🧠 智能记忆管理**: 8个专门化Agent，智能推理引擎
- **🔍 毫秒级搜索**: 5种搜索引擎，语义相似性检索
- **🔌 WASM插件**: 安全热插拔插件系统
- **🎨 多模态支持**: 图像、音频、视频处理
- **🛡️ 企业级**: RBAC权限、监控告警、高可用部署
- **🚀 高性能**: 216K ops/s，93,000x缓存加速

### 1.3 技术栈

- **核心语言**: Rust (异步优先)
- **运行时**: Tokio
- **存储**: LibSQL, PostgreSQL, LanceDB, Redis
- **嵌入**: FastEmbed, OpenAI, 本地模型
- **LLM**: 20+提供商 (DeepSeek, OpenAI, Anthropic等)
- **插件**: WebAssembly (Extism)
- **监控**: Prometheus, OpenTelemetry, Grafana

---

## 2. 整体设计

### 2.1 设计原则

1. **模块化**: 18个独立crate，职责清晰分离
2. **异步优先**: 基于Tokio的高并发架构
3. **类型安全**: Rust强类型系统保证内存安全
4. **可扩展性**: 插件系统和多后端支持
5. **企业就绪**: 完整的可观测性和安全特性

### 2.2 系统目标

- **高性能**: 毫秒级响应，百万级QPS
- **高可用**: 99.9%可用性，自动故障转移
- **易扩展**: 水平扩展，热插拔插件
- **易集成**: 简洁API，多语言SDK

### 2.3 核心创新

1. **智能推理引擎**: AI驱动的自动记忆管理
2. **分层记忆架构**: 四层记忆组织结构
3. **WASM插件系统**: 安全沙盒执行环境
4. **混合搜索引擎**: 5种引擎协同工作

---

## 3. 系统架构

### 3.1 架构全景图

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                          AgentMem 企业级记忆平台                                  │
│                                                                                 │
│  ┌────────────────────────────────────────────────────────────────────────┐   │
│  │                         应用层 (Application Layer)                       │   │
│  │                                                                          │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐   │   │
│  │  │ HTTP Server │  │  CLI Tool   │  │Python Binding│ │WASM Plugins │   │   │
│  │  │ (REST API)  │  │ agentmem-cli│  │  pyo3-based │  │ Hot-Reload  │   │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘   │   │
│  └────────────────────────────────────────────────────────────────────────┘   │
│                                      ↓                                         │
│  ┌────────────────────────────────────────────────────────────────────────┐   │
│  │                      核心服务层 (Core Services)                          │   │
│  │                                                                          │   │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────┐     │   │
│  │  │ Memory Manager  │  │ Plugin Manager  │  │ Orchestrator        │     │   │
│  │  │ - CRUD 操作     │  │ - 加载/卸载     │  │ - 智能分发          │     │   │
│  │  │ - 分层管理      │  │ - LRU 缓存      │  │ - 工作流编排        │     │   │
│  │  │ - 冲突解决      │  │ - 沙盒隔离      │  │ - Agent 协调        │     │   │
│  │  └─────────────────┘  └─────────────────┘  └─────────────────────┘     │   │
│  │                                                                          │   │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────┐     │   │
│  │  │ Search Engine   │  │ LLM Integration │  │ Multimodal Processor│     │   │
│  │  │ - Vector Search │  │ - 20+ Providers │  │ - Image/Audio/Video │     │   │
│  │  │ - BM25 Ranking  │  │ - DeepSeek      │  │ - Cross-Modal       │     │   │
│  │  │ - Hybrid Search │  │ - Smart Retry   │  │ - AI Analysis       │     │   │
│  │  └─────────────────┘  └─────────────────┘  └─────────────────────┘     │   │
│  └────────────────────────────────────────────────────────────────────────┘   │
│                                      ↓                                         │
│  ┌────────────────────────────────────────────────────────────────────────┐   │
│  │                      存储抽象层 (Storage Abstraction)                    │   │
│  │                                                                          │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌───────────┐   │   │
│  │  │ Memory Store │  │ Vector Store │  │ Graph Store  │  │ Cache     │   │   │
│  │  │ - LibSQL     │  │ - LanceDB    │  │ - Native     │  │ - LRU     │   │   │
│  │  │ - PostgreSQL │  │ - Redis      │  │ - Neo4j      │  │ - Multi-  │   │   │
│  │  │ - Pluggable  │  │ - Pinecone   │  │ - Pluggable  │  │   Level   │   │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘  └───────────┘   │   │
│  └────────────────────────────────────────────────────────────────────────┘   │
│                                      ↓                                         │
│  ┌────────────────────────────────────────────────────────────────────────┐   │
│  │                      基础设施层 (Infrastructure)                         │   │
│  │                                                                          │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐   │   │
│  │  │Observability│  │   Security  │  │Distribution │  │ Performance │   │   │
│  │  │ - Prometheus│  │ - RBAC      │  │ - K8s Ready │  │ - Async I/O │   │   │
│  │  │ - OpenTelemetry│ │ - Auth     │  │ - Scaling   │  │ - Parallel  │   │   │
│  │  │ - Grafana   │  │ - Audit Log │  │ - Failover  │  │ - Zero-Copy │   │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘   │   │
│  └────────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### 3.2 分层记忆架构

```
Global Layer    → 全局共享知识和系统配置
    ↓
Agent Layer     → 代理特定知识和行为模式
    ↓
User Layer      → 用户个人信息和偏好设置
    ↓
Session Layer   → 会话上下文和临时状态
```

### 3.3 18个模块化Crate

#### 基础层 (Foundation)
| Crate | 职责 | 代码量 | 状态 |
|-------|------|--------|------|
| **agent-mem-traits** | 核心抽象和 trait 定义 | ~2K lines | ✅ 稳定 |
| **agent-mem-utils** | 通用工具和辅助函数 | ~1K lines | ✅ 稳定 |
| **agent-mem-config** | 配置管理和环境变量 | ~1K lines | ✅ 稳定 |

#### 核心引擎层 (Core Engine)
| Crate | 职责 | 代码量 | 状态 |
|-------|------|--------|------|
| **agent-mem-core** | 记忆管理核心引擎 | ~25K lines | ✅ 生产就绪 |
| **agent-mem** | 统一 API 和编排器 | ~3K lines | ✅ 生产就绪 |
| **agent-mem-intelligence** | AI 智能推理引擎 | ~8K lines | ✅ DeepSeek 集成 |

#### 集成层 (Integration)
| Crate | 职责 | 代码量 | 状态 |
|-------|------|--------|------|
| **agent-mem-llm** | 20+ LLM 提供商集成 | ~6K lines | ✅ 全功能 |
| **agent-mem-embeddings** | 嵌入模型集成 | ~3K lines | ✅ FastEmbed |
| **agent-mem-storage** | 多后端存储抽象 | ~10K lines | ✅ 多数据库 |
| **agent-mem-tools** | MCP 工具集成 | ~5K lines | ✅ 完整 |

#### 服务层 (Services)
| Crate | 职责 | 代码量 | 状态 |
|-------|------|--------|------|
| **agent-mem-server** | HTTP REST API 服务器 | ~8K lines | ✅ 175+ 端点 |
| **agent-mem-client** | HTTP 客户端 SDK | ~2K lines | ✅ 完整 |
| **agent-mem-compat** | Mem0 兼容层 | ~3K lines | ✅ 100% 兼容 |

#### 扩展层 (Extensions)
| Crate | 职责 | 代码量 | 状态 |
|-------|------|--------|------|
| **agent-mem-plugin-sdk** | WASM 插件 SDK | ~500 lines | ✅ Extism 集成 |
| **agent-mem-plugins** | 插件管理器 | ~1.5K lines | ✅ 热插拔 |
| **agent-mem-python** | Python 绑定 (PyO3) | ~800 lines | ✅ 可用 |

#### 运维层 (Operations)
| Crate | 职责 | 代码量 | 状态 |
|-------|------|--------|------|
| **agent-mem-observability** | 可观测性集成 | ~2K lines | ✅ Full Stack |
| **agent-mem-performance** | 性能监控和优化 | ~3K lines | ✅ 完整 |
| **agent-mem-deployment** | K8s 部署工具 | ~2K lines | ✅ 生产级 |
| **agent-mem-distributed** | 分布式支持 | ~1.5K lines | ✅ 可用 |

**总计**: ~88,000+ 行生产级 Rust 代码

---

## 4. 核心模块

### 4.1 智能记忆管理

#### 8个专门化Agent

```rust
// 核心Agent类型
pub enum AgentType {
    Core,        // 核心记忆管理
    Episodic,    // 情节记忆
    Semantic,    // 语义记忆
    Procedural,  // 程序性记忆
    Working,     // 工作记忆
    Contextual,  // 上下文记忆
    Knowledge,   // 知识图谱
    Resource,    // 资源记忆
}
```

#### Agent+Manager双层架构

```rust
// Agent 接口
#[async_trait]
pub trait MemoryAgent {
    async fn process(&self, input: AgentInput) -> Result<AgentOutput>;
    async fn learn(&self, experience: &Experience) -> Result<()>;
    async fn recall(&self, query: &Query) -> Result<Vec<Memory>>;
}

// Manager 接口
#[async_trait]
pub trait AgentManager {
    async fn coordinate(&self, task: Task) -> Result<TaskResult>;
    async fn route_to_agent(&self, agent_type: AgentType, input: AgentInput) -> Result<AgentOutput>;
}
```

### 4.2 搜索引擎系统

#### 5种搜索引擎

| 引擎 | 用途 | 特点 | 实现行数 |
|------|------|------|---------|
| **VectorSearchEngine** | 语义相似性 | 基于 embedding 的向量搜索 | ~200行 |
| **BM25SearchEngine** | 关键词匹配 | 315行完整BM25实现 | ~315行 |
| **FullTextSearchEngine** | 精确文本 | PostgreSQL 原生全文搜索 | ~150行 |
| **FuzzyMatchEngine** | 模糊匹配 | Levenshtein 距离算法 | ~180行 |
| **HybridSearchEngine** | 综合排序 | RRF (Reciprocal Rank Fusion) | ~280行 |

#### 搜索架构

```rust
pub trait SearchEngine {
    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>>;
    fn get_engine_type(&self) -> EngineType;
}

pub struct SearchOrchestrator {
    engines: HashMap<EngineType, Box<dyn SearchEngine>>,
    router: QueryRouter,
    synthesizer: ResultSynthesizer,
}
```

### 4.3 智能推理引擎

#### DeepSeek集成

```rust
pub struct IntelligentMemoryProcessor {
    llm_client: Box<dyn LlmClient>,
    fact_extractor: FactExtractor,
    decision_engine: DecisionEngine,
    conflict_resolver: ConflictResolver,
}

impl IntelligentMemoryProcessor {
    pub async fn process_messages(&self, messages: &[Message], memories: &[Memory]) -> Result<ProcessingResult> {
        // 1. 事实提取
        let facts = self.fact_extractor.extract_from_messages(messages).await?;
        
        // 2. 智能决策
        let decisions = self.decision_engine.make_decisions(&facts, memories).await?;
        
        // 3. 冲突解决
        let resolved = self.conflict_resolver.resolve_conflicts(&decisions).await?;
        
        Ok(ProcessingResult {
            extracted_facts: facts,
            memory_decisions: resolved,
        })
    }
}
```

### 4.4 多模态处理

#### 支持模态

- **图像处理**: `image.rs` + `openai_vision.rs` + `real_image.rs`
- **音频处理**: `audio.rs` + `openai_whisper.rs` + `real_audio.rs`
- **视频处理**: `video.rs` + `video_analyzer.rs`
- **跨模态检索**: `cross_modal.rs` + `unified_retrieval.rs`

#### 多模态架构

```rust
pub trait MultimodalProcessor {
    async fn process_image(&self, image: &ImageInput) -> Result<ImageAnalysis>;
    async fn process_audio(&self, audio: &AudioInput) -> Result<AudioTranscription>;
    async fn process_video(&self, video: &VideoInput) -> Result<VideoAnalysis>;
    async fn cross_modal_search(&self, query: &CrossModalQuery) -> Result<Vec<MultimodalResult>>;
}
```

---

## 5. API接口

### 5.1 统一Memory API

#### 零配置使用

```rust
use agent_mem::Memory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 设置环境变量
    std::env::set_var("OPENAI_API_KEY", "sk-...");
    
    // 零配置初始化
    let mem = Memory::new().await?;
    
    // 添加记忆（默认启用智能功能）
    mem.add("I love pizza").await?;
    mem.add("I live in San Francisco").await?;
    
    // 搜索记忆
    let results = mem.search("What do you know about me?").await?;
    for result in results {
        println!("- {}", result.memory);
    }
    
    Ok(())
}
```

#### Builder模式

```rust
use agent_mem::Memory;

let mem = Memory::builder()
    .with_storage("libsql://agentmem.db")
    .with_llm("openai", "gpt-4")
    .with_embedder("openai", "text-embedding-3-small")
    .enable_intelligent_features()
    .build()
    .await?;
```

#### MemoryScope支持

```rust
use agent_mem::{Memory, MemoryScope};

// 用户级记忆
let scope = MemoryScope::User {
    user_id: "alice".to_string(),
};
mem.add_with_scope("我喜欢喝咖啡", scope).await?;

// 组织级记忆
let scope = MemoryScope::Organization {
    org_id: "acme-corp".to_string(),
};
mem.add_with_scope("公司政策：每周五远程办公", scope).await?;
```

### 5.2 REST API (175+ 端点)

#### 记忆管理API

```http
# 添加记忆
POST /api/v1/memories
Content-Type: application/json
Authorization: Bearer <token>

{
  "content": "I love pizza",
  "user_id": "user123",
  "metadata": {
    "importance": 0.8,
    "tags": ["preference", "food"]
  }
}

# 搜索记忆
GET /api/v1/memories/search?q=food&user_id=user123&limit=10

# 获取记忆详情
GET /api/v1/memories/{memory_id}

# 更新记忆
PUT /api/v1/memories/{memory_id}

# 删除记忆
DELETE /api/v1/memories/{memory_id}
```

#### 聊天API

```http
# 发送聊天消息
POST /api/v1/chat
Content-Type: application/json

{
  "message": "What do you know about my preferences?",
  "user_id": "user123",
  "stream": false
}

# 流式聊天
POST /api/v1/chat/stream
Content-Type: application/json

{
  "message": "Tell me about myself",
  "user_id": "user123"
}
```

#### 工作记忆API

```http
# 添加工作记忆
POST /api/v1/working-memory
{
  "user_id": "user123",
  "content": "Current task: write documentation",
  "expires_at": "2024-01-01T12:00:00Z"
}

# 获取工作记忆
GET /api/v1/working-memory/{user_id}

# 清理过期记忆
DELETE /api/v1/working-memory/expired
```

#### 插件管理API

```http
# 注册插件
POST /api/v1/plugins
{
  "id": "weather-plugin",
  "path": "/path/to/weather_plugin.wasm",
  "metadata": {
    "name": "Weather Plugin",
    "version": "1.0.0",
    "plugin_type": "datasource",
    "required_capabilities": ["network_access", "logging_access"]
  }
}

# 调用插件
POST /api/v1/plugins/{plugin_id}/call
{
  "function": "get_weather",
  "args": {
    "city": "San Francisco"
  }
}
```

### 5.3 Mem0兼容API

```rust
use agent_mem_compat::Mem0Client;

#[tokio::main]
async fn main() -> Result<()> {
    let client = Mem0Client::new().await?;
    
    // 使用与Mem0相同的API
    let memory_id = client.add(
        "user123", 
        "我喜欢喝咖啡，特别是拿铁", 
        None
    ).await?;
    
    let results = client.search("饮品偏好", "user123", None).await?;
    
    println!("找到 {} 条记忆", results.len());
    for memory in results {
        println!("- {}: {}", memory.id, memory.content);
    }
    
    Ok(())
}
```

### 5.4 Python SDK

```python
from agent_mem import Memory

# 零配置使用
mem = Memory()
await mem.add("I love pizza")
results = await mem.search("What do I like to eat?")

# 高级配置
mem = Memory.builder() \
    .with_storage("postgres://...") \
    .with_llm("openai", "gpt-4") \
    .build()

await mem.add("User preference", user_id="alice", scope="user")
```

---

## 6. 插件系统

### 6.1 WASM插件架构

```
┌───────────────────────────────────────────────────────────┐
│                   WASM 插件架构                            │
├───────────────────────────────────────────────────────────┤
│                                                           │
│  ┌─────────────┐      ┌─────────────┐                   │
│  │   插件 A    │      │   插件 B    │                   │
│  │  (WASM)     │      │  (WASM)     │                   │
│  └──────┬──────┘      └──────┬──────┘                   │
│         │                     │                          │
│         ▼                     ▼                          │
│  ┌──────────────────────────────────┐                   │
│  │     Extism Plugin Manager        │                   │
│  │  - LRU Cache (100 插件)          │                   │
│  │  - Sandbox Isolation             │                   │
│  │  - Capability Check              │                   │
│  └──────────────────────────────────┘                   │
│         │                                                │
│         ▼                                                │
│  ┌──────────────────────────────────┐                   │
│  │     Host Capabilities            │                   │
│  │  - Memory Access                 │                   │
│  │  - Storage Access                │                   │
│  │  - Network Access                │                   │
│  │  - LLM Access                    │                   │
│  │  - Logging Access                │                   │
│  └──────────────────────────────────┘                   │
└───────────────────────────────────────────────────────────┘
```

### 6.2 插件开发

#### 创建插件

```rust
// my_plugin/src/lib.rs
use extism_pdk::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Input {
    message: String,
}

#[derive(Serialize)]
struct Output {
    result: String,
}

#[plugin_fn]
pub fn process(input: String) -> FnResult<String> {
    let input: Input = serde_json::from_str(&input)?;
    
    // 调用宿主函数
    host::log("info", &format!("Processing: {}", input.message))?;
    
    // 处理逻辑
    let output = Output {
        result: format!("Processed: {}", input.message),
    };
    
    Ok(serde_json::to_string(&output)?)
}

#[plugin_fn]
pub fn metadata() -> FnResult<String> {
    Ok(serde_json::json!({
        "name": "my-plugin",
        "version": "1.0.0",
        "description": "My custom plugin",
        "plugin_type": "MemoryProcessor",
        "required_capabilities": ["LoggingAccess", "MemoryAccess"]
    }).to_string())
}
```

#### 编译WASM

```bash
# 安装WASM目标
rustup target add wasm32-wasip1

# 编译插件
cd my_plugin
cargo build --target wasm32-wasip1 --release

# WASM文件输出到
# target/wasm32-wasip1/release/my_plugin.wasm
```

### 6.3 插件管理

#### 注册插件

```bash
# 使用脚本注册
./register_plugins.sh

# 使用API注册
curl -X POST "http://localhost:8080/api/v1/plugins" \
  -H "Content-Type: application/json" \
  -d '{
    "id": "my-plugin",
    "path": "/path/to/my_plugin.wasm",
    "metadata": {
      "name": "my-plugin",
      "version": "1.0.0",
      "plugin_type": "memory_processor",
      "required_capabilities": ["logging_access", "memory_access"]
    }
  }'
```

#### 插件使用

```rust
use agent_mem_plugins::{PluginManager, PluginConfig};

let manager = PluginManager::new(100); // 最多100个插件

// 注册插件
let config = PluginConfig {
    id: "weather-plugin".to_string(),
    path: "/path/to/weather_plugin.wasm".to_string(),
    metadata: serde_json::json!({
        "name": "Weather Plugin",
        "version": "1.0.0"
    }),
};

manager.register_plugin(config).await?;

// 调用插件
let result = manager.call_plugin(
    "weather-plugin", 
    "get_weather", 
    r#"{"city": "San Francisco"}"#
).await?;

println!("Weather result: {}", result);
```

### 6.4 能力系统

```rust
pub enum Capability {
    MemoryAccess,      // 读写记忆数据
    StorageAccess,     // 访问存储层
    SearchAccess,      // 执行搜索操作
    LlmAccess,         // 调用 LLM API
    NetworkAccess,     // 发起网络请求
    FileSystemAccess,  // 文件系统访问
    LoggingAccess,     // 写入日志
    ConfigAccess,      // 读取配置
}

// 插件权限控制
pub struct PluginSandbox {
    allowed_capabilities: HashSet<Capability>,
    memory_limit: usize,      // 内存限制
    time_limit: Duration,     // 执行时间限制
    network_allowed: bool,    // 网络访问权限
}
```

### 6.5 性能指标

| 指标 | 数值 | 说明 |
|------|------|------|
| **首次加载** | ~31ms | WASM 模块加载和初始化 |
| **缓存命中** | ~333ns | LRU 缓存，93,000x 加速 |
| **并发吞吐** | 216K calls/s | 100 并发时的调用吞吐量 |
| **内存占用** | < 50MB | 单个插件最大内存限制 |
| **执行超时** | 30s | 可配置的执行时间限制 |

---

## 7. 使用指南

### 7.1 快速开始

#### 方式1: 零配置使用（推荐）

```rust
use agent_mem::Memory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 设置环境变量（任选其一）
    std::env::set_var("OPENAI_API_KEY", "sk-...");
    // 或 std::env::set_var("ZHIPU_API_KEY", "...");
    
    // 2. 零配置初始化
    let mem = Memory::new().await?;
    
    // 3. 添加记忆
    mem.add("I love pizza").await?;
    mem.add("I live in San Francisco").await?;
    
    // 4. 搜索记忆
    let results = mem.search("What do you know about me?").await?;
    for result in results {
        println!("- {}", result.memory);
    }
    
    Ok(())
}
```

#### 方式2: 服务器模式

```bash
# 克隆仓库
git clone https://gitcode.com/louloulin/agentmem.git
cd agentmem

# 启动服务器（推荐使用just）
just start-full-with-plugins

# 或使用cargo
cargo run --bin agent-mem-server

# 服务地址：
# - 后端API: http://localhost:8080
# - 前端UI: http://localhost:3001
# - 健康检查: http://localhost:8080/health
# - API文档: http://localhost:8080/swagger-ui/
```

### 7.2 配置选项

#### 环境变量配置

```bash
# LLM提供商（任选其一）
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
export DEEPSEEK_API_KEY="sk-..."
export ZHIPU_API_KEY="sk-..."

# 数据库配置
export DATABASE_BACKEND="libsql"  # 默认
export DATABASE_URL="agentmem.db" # 默认路径

# 或使用PostgreSQL
export DATABASE_BACKEND="postgres"
export DATABASE_URL="postgresql://user:password@localhost:5432/agentmem"
```

#### 配置文件

```toml
# config.toml
[database]
backend = "postgres"
url = "postgresql://user:password@localhost:5432/agentmem"
auto_migrate = true

[llm]
provider = "openai"
model = "gpt-4"
api_key = "${OPENAI_API_KEY}"

[embeddings]
provider = "openai"
model = "text-embedding-3-small"
dimensions = 1536

[server]
host = "0.0.0.0"
port = 8080
workers = 4

[plugins]
enabled = true
max_plugins = 100
cache_size = 50
```

### 7.3 使用场景

#### AI聊天机器人

```rust
use agent_mem::Memory;

#[tokio::main]
async fn main() -> Result<()> {
    let memory = Memory::new().await?;
    
    // 添加用户偏好
    memory.add_with_scope("我喜欢喝咖啡，不喜欢茶", 
        MemoryScope::User { user_id: "alice".to_string() }
    ).await?;
    
    // 查询相关记忆
    let results = memory.search_with_scope(
        "用户喜欢什么饮品",
        MemoryScope::User { user_id: "alice".to_string() }
    ).await?;
    
    println!("找到记忆: {:?}", results);
    Ok(())
}
```

#### 智能客服系统

```rust
use agent_mem::{Memory, AddMemoryOptions};
use agent_mem_intelligence::IntelligentMemoryProcessor;

#[tokio::main]
async fn main() -> Result<()> {
    let memory = Memory::builder()
        .llm_provider("deepseek")
        .enable_intelligent_features()
        .build()
        .await?;
    
    let processor = IntelligentMemoryProcessor::new(&api_key)?;
    
    // 处理客服对话
    let messages = vec![
        Message {
            role: "user".to_string(),
            content: "我是张三，来自北京，对您公司的产品很感兴趣".to_string(),
            timestamp: Some("2024-01-01T10:00:00Z".to_string()),
        }
    ];
    
    let result = processor.process_messages(&messages, &[]).await?;
    
    // 自动提取关键信息并存储
    for fact in result.extracted_facts {
        let options = AddMemoryOptions {
            user_id: Some(fact.user_id.clone()),
            metadata: Some(fact.metadata),
            ..Default::default()
        };
        memory.add_with_options(&fact.content, options).await?;
    }
    
    Ok(())
}
```

#### 企业知识库

```rust
use agent_mem::Memory;

#[tokio::main]
async fn main() -> Result<()> {
    let memory = Memory::builder()
        .storage_backend("postgres")
        .vector_store("pinecone")
        .embedder_provider("openai")
        .build()
        .await?;
    
    // 导入公司文档
    for doc in company_documents {
        memory.add_with_scope(&doc.content, 
            MemoryScope::Organization { 
                org_id: "acme-corp".to_string() 
            }
        ).await?;
    }
    
    // 智能搜索
    let results = memory.search_with_scope(
        "公司的休假政策是什么",
        MemoryScope::Organization { 
            org_id: "acme-corp".to_string() 
        }
    ).await?;
    
    Ok(())
}
```

### 7.4 从Mem0迁移

```python
# 原有 Mem0 代码
# from mem0 import Memory
# memory = Memory()

# AgentMem 兼容代码
from agent_mem_compat import Mem0Client

memory = Mem0Client.new()
memory_id = memory.add("user123", "I love pizza", None)
results = memory.search("food", "user123", None)
```

---

## 8. 性能指标

### 8.1 基准测试结果

#### 吞吐量基准

| 操作 | QPS | 延迟 (P50) | 延迟 (P99) |
|------|-----|-----------|-----------|
| 添加记忆 | 5,000 | 20ms | 50ms |
| 向量搜索 | 10,000 | 10ms | 30ms |
| BM25 搜索 | 15,000 | 5ms | 15ms |
| 插件调用 | 200,000+ | 1ms | 5ms |
| 批量操作 | 50,000 | 100ms | 300ms |

#### 插件性能指标

| 指标 | 数值 | 说明 |
|------|------|------|
| **首次加载** | ~31ms | WASM 模块加载和初始化 |
| **缓存命中** | ~333ns | LRU 缓存，93,000x 加速 |
| **并发吞吐** | 216K calls/s | 100 并发时的调用吞吐量 |
| **内存占用** | < 50MB | 单个插件最大内存限制 |
| **执行超时** | 30s | 可配置的执行时间限制 |

### 8.2 可扩展性

```
单节点性能:
├─ 记忆容量: 10M+ 记录
├─ 并发连接: 10,000+
├─ 内存占用: < 2GB (idle)
└─ CPU 使用: < 20% (idle)

分布式集群:
├─ 水平扩展: 支持
├─ 负载均衡: Nginx/HAProxy
├─ 数据分片: 支持
└─ 高可用: 3+ 副本
```

### 8.3 内存优化

```rust
// 多级缓存系统
pub struct MultiLevelCache {
    l1_cache: LruCache<String, CacheEntry>,  // 内存缓存
    l2_cache: RedisCache,                   // Redis缓存
    l3_cache: DiskCache,                    // 磁盘缓存
}

// 批量处理优化
pub struct BatchProcessor {
    batch_size: usize,
    buffer: Vec<MemoryItem>,
    flush_interval: Duration,
}
```

---

## 9. 部署指南

### 9.1 Docker部署

```bash
# 使用Docker Compose
git clone https://gitcode.com/louloulin/agentmem.git
cd agentmem

# 启动完整堆栈
docker-compose up -d

# 查看日志
docker-compose logs -f

# 停止服务
docker-compose down
```

#### docker-compose.yml

```yaml
version: '3.8'
services:
  agentmem-server:
    build: .
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgresql://postgres:password@db:5432/agentmem
      - OPENAI_API_KEY=${OPENAI_API_KEY}
    depends_on:
      - db
      - redis

  db:
    image: postgres:15
    environment:
      - POSTGRES_DB=agentmem
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=password
    volumes:
      - postgres_data:/var/lib/postgresql/data

  redis:
    image: redis:7
    ports:
      - "6379:6379"

  prometheus:
    image: prom/prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml

  grafana:
    image: grafana/grafana
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana_data:/var/lib/grafana

volumes:
  postgres_data:
  grafana_data:
```

### 9.2 Kubernetes部署

```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: agentmem-server
spec:
  replicas: 3
  selector:
    matchLabels:
      app: agentmem-server
  template:
    metadata:
      labels:
        app: agentmem-server
    spec:
      containers:
      - name: agentmem-server
        image: agentmem/server:latest
        ports:
        - containerPort: 8080
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: agentmem-secrets
              key: database-url
        - name: OPENAI_API_KEY
          valueFrom:
            secretKeyRef:
              name: agentmem-secrets
              key: openai-api-key
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: agentmem-service
spec:
  selector:
    app: agentmem-server
  ports:
  - protocol: TCP
    port: 80
    targetPort: 8080
  type: LoadBalancer
```

### 9.3 生产配置

#### 高可用配置

```toml
# config.production.toml
[database]
backend = "postgres"
url = "postgresql://user:password@postgres-cluster:5432/agentmem"
max_connections = 20
min_connections = 5
connection_timeout = 30

[llm]
provider = "openai"
model = "gpt-4"
api_key = "${OPENAI_API_KEY}"
timeout = 30
retry_attempts = 3

[embeddings]
provider = "openai"
model = "text-embedding-3-small"
batch_size = 100
dimensions = 1536

[server]
host = "0.0.0.0"
port = 8080
workers = 8
max_connections = 1000
timeout = 30

[cache]
memory_cache_size = 1000
redis_url = "redis://redis-cluster:6379"
cache_ttl = 3600

[plugins]
enabled = true
max_plugins = 100
execution_timeout = 30
memory_limit = 536870912  # 512MB

[observability]
prometheus_enabled = true
tracing_enabled = true
log_level = "info"
```

#### 监控配置

```yaml
# monitoring/prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "alert_rules.yml"

scrape_configs:
  - job_name: 'agentmem'
    static_configs:
      - targets: ['agentmem-server:8080']
    metrics_path: /metrics
    scrape_interval: 5s

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093
```

---

## 10. 开发指南

### 10.1 开发环境设置

```bash
# 克隆仓库
git clone https://gitcode.com/louloulin/agentmem.git
cd agentmem

# 安装Rust工具链
rustup update stable
rustup component add rustfmt clippy

# 构建项目
cargo build --workspace

# 运行测试
cargo test --workspace

# 运行示例
cargo run --example quickstart

# 代码格式化
cargo fmt --all

# 代码检查
cargo clippy -- -D warnings
```

### 10.2 项目结构

```
agentmem/
├── crates/                     # 18个核心crate
│   ├── agent-mem-traits/       # 核心抽象和接口
│   ├── agent-mem-core/         # 记忆管理引擎
│   ├── agent-mem-llm/          # LLM集成
│   ├── agent-mem-storage/      # 存储后端
│   ├── agent-mem-embeddings/   # 嵌入模型
│   ├── agent-mem-intelligence/ # 智能推理引擎
│   ├── agent-mem-server/       # HTTP服务器
│   ├── agent-mem-client/       # HTTP客户端
│   ├── agent-mem-compat/       # Mem0兼容层
│   ├── agent-mem-plugin-sdk/   # WASM插件SDK
│   ├── agent-mem-plugins/      # 插件管理器
│   └── ...                     # 其他crate
├── examples/                   # 80+示例程序
├── tests/                      # 集成测试
├── docs/                       # 技术文档
├── config/                     # 配置文件
├── scripts/                    # 构建脚本
├── docker/                     # Docker配置
└── k8s/                        # Kubernetes配置
```

### 10.3 贡献指南

#### 开发流程

1. **Fork仓库**
2. **创建功能分支**: `git checkout -b feature/new-feature`
3. **编写代码**
4. **添加测试**: `cargo test`
5. **代码格式化**: `cargo fmt`
6. **代码检查**: `cargo clippy`
7. **提交代码**: `git commit -m "feat: add new feature"`
8. **推送分支**: `git push origin feature/new-feature`
9. **创建PR**

#### 测试要求

```bash
# 运行所有测试
cargo test --workspace --all-features

# 运行特定测试
cargo test --package agent-mem-core
cargo test --test integration_test

# 运行性能测试
cargo bench --workspace

# 生成测试覆盖率报告
cargo tarpaulin --workspace --out Html
```

#### 代码质量

- 所有新代码必须有单元测试
- 集成测试覆盖核心功能
- 代码覆盖率要求 > 80%
- 遵循Rust API设计指南
- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码质量

### 10.4 插件开发

#### 创建新插件

```bash
# 创建插件目录
mkdir my_plugin
cd my_plugin

# 初始化Cargo项目
cargo init --lib

# 添加依赖
cat >> Cargo.toml << EOF
[dependencies]
extism-pdk = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[lib]
crate-type = ["cdylib"]
EOF

# 编译为WASM
cargo build --target wasm32-wasip1 --release
```

#### 插件测试

```rust
// tests/plugin_test.rs
use agent_mem_plugins::{PluginManager, PluginConfig};
use tokio_test;

#[tokio::test]
async fn test_plugin_execution() {
    let manager = PluginManager::new(10);
    
    let config = PluginConfig {
        id: "test-plugin".to_string(),
        path: "tests/fixtures/test_plugin.wasm".to_string(),
        metadata: serde_json::json!({
            "name": "Test Plugin",
            "version": "1.0.0"
        }),
    };
    
    manager.register_plugin(config).await.unwrap();
    
    let result = manager.call_plugin(
        "test-plugin",
        "test_function",
        r#"{"input": "test"}"#
    ).await.unwrap();
    
    assert!(result.contains("processed"));
}
```

---

## 总结

AgentMem 是一个功能完整、性能卓越的下一代智能记忆管理平台。通过模块化架构、WASM插件系统、智能推理引擎和企业级特性，为AI应用提供了强大的记忆能力。

### 核心优势

1. **🧠 智能推理**: AI驱动的自动记忆管理
2. **🚀 极致性能**: 216K ops/s，毫秒级响应
3. **🔌 灵活扩展**: WASM插件系统，热插拔
4. **🛡️ 企业就绪**: 完整的安全和监控体系
5. **🔄 完全兼容**: 100% Mem0 API兼容

### 技术亮点

- **88,000+行** 生产级Rust代码
- **18个** 专业化crate模块
- **175+** REST API端点
- **80+** 示例程序
- **20+** LLM提供商集成
- **5种** 搜索引擎协同工作

AgentMem 为构建下一代智能AI应用提供了坚实的技术基础。

---

*本文档涵盖AgentMem v2.0.0的完整技术规格和使用指南。*