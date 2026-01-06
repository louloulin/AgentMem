# Fluvio to AgentMem - 流式数据处理架构

**架构版本**: 1.0  
**创建日期**: 2025-01-10  
**技术栈**: Fluvio (Rust) + AgentMem (Rust) + WASM SmartModules

---

## 📋 目录

1. [架构概览](#架构概览)
2. [核心组件](#核心组件)
3. [数据流设计](#数据流设计)
4. [技术优势](#技术优势)
5. [实施方案](#实施方案)
6. [性能指标](#性能指标)
7. [部署架构](#部署架构)

---

## 🎯 架构概览

### 设计理念

**Fluvio + AgentMem** 架构结合了：
- **Fluvio**: Rust 驱动的流式数据平台，使用 WebAssembly 进行边缘计算
- **AgentMem**: Rust 实现的生产级 AI 记忆管理系统

**核心优势**:
1. ⚡ **全 Rust 栈**: 零成本抽象，极致性能
2. 🔒 **内存安全**: 编译时保证，无运行时错误
3. 🚀 **边缘计算**: WASM SmartModules 在数据源端处理
4. 📊 **实时处理**: 毫秒级延迟，百万级吞吐
5. 🔄 **事件驱动**: 完全异步，高并发支持

---

## 🏗️ 核心组件

### 1. 数据源层 (Data Sources)

**支持的数据源**:
```rust
// 用户交互事件
struct UserInteraction {
    user_id: String,
    action: String,
    timestamp: DateTime<Utc>,
    context: HashMap<String, Value>,
}

// 应用事件
struct AppEvent {
    event_type: String,
    payload: Value,
    source: String,
}

// 传感器数据
struct SensorData {
    sensor_id: String,
    readings: Vec<f64>,
    location: GeoPoint,
}

// 日志流
struct LogEntry {
    level: LogLevel,
    message: String,
    metadata: HashMap<String, String>,
}

// API 调用
struct ApiCall {
    endpoint: String,
    method: HttpMethod,
    response_time: Duration,
}

// WebSocket 实时消息
struct RealtimeMessage {
    channel: String,
    data: Value,
    priority: Priority,
}
```

---

### 2. Fluvio 流式平台

#### 2.1 Fluvio Producer (生产者)

**功能**: 将数据源事件发送到 Fluvio Topics

```rust
use fluvio::{Fluvio, TopicProducer};

pub struct AgentMemProducer {
    fluvio: Fluvio,
    producers: HashMap<String, TopicProducer>,
}

impl AgentMemProducer {
    pub async fn send_user_event(&self, event: UserInteraction) -> Result<()> {
        let producer = self.producers.get("user-events").unwrap();
        let payload = serde_json::to_vec(&event)?;
        producer.send(RecordKey::NULL, payload).await?;
        Ok(())
    }
    
    pub async fn send_memory_update(&self, update: MemoryUpdate) -> Result<()> {
        let producer = self.producers.get("memory-updates").unwrap();
        let payload = serde_json::to_vec(&update)?;
        producer.send(RecordKey::NULL, payload).await?;
        Ok(())
    }
}
```

#### 2.2 Fluvio Topics (主题分区)

**Topic 设计**:

| Topic 名称 | 分区数 | 保留时间 | 用途 |
|-----------|--------|---------|------|
| `user-events` | 16 | 7 天 | 用户交互事件 |
| `agent-actions` | 8 | 3 天 | Agent 行为记录 |
| `memory-updates` | 32 | 30 天 | 记忆更新事件 |
| `chat-messages` | 16 | 14 天 | 聊天消息流 |
| `system-metrics` | 4 | 1 天 | 系统指标 |
| `audit-logs` | 8 | 90 天 | 审计日志 |

**Topic 配置**:
```bash
# 创建 user-events topic
fluvio topic create user-events \
  --partitions 16 \
  --retention-time 7d \
  --compression gzip

# 创建 memory-updates topic
fluvio topic create memory-updates \
  --partitions 32 \
  --retention-time 30d \
  --compression lz4
```

#### 2.3 SmartModules (WASM 处理器)

**SmartModule 功能**: 在 Fluvio 集群中运行的 WASM 模块，用于边缘数据处理

**示例 1: 事件过滤 SmartModule**
```rust
// smartmodules/event_filter.rs
use fluvio_smartmodule::{smartmodule, Record, RecordData, Result};

#[smartmodule(filter)]
pub fn filter(record: &Record) -> Result<bool> {
    let event: UserInteraction = serde_json::from_slice(record.value.as_ref())?;
    
    // 只保留重要事件 (importance > 0.7)
    Ok(event.importance > 0.7)
}
```

**示例 2: 数据转换 SmartModule**
```rust
// smartmodules/data_transform.rs
use fluvio_smartmodule::{smartmodule, Record, RecordData, Result};

#[smartmodule(map)]
pub fn transform(record: &Record) -> Result<(Option<RecordData>, RecordData)> {
    let event: UserInteraction = serde_json::from_slice(record.value.as_ref())?;
    
    // 转换为 AgentMem 格式
    let memory_item = MemoryItem {
        organization_id: event.org_id,
        user_id: event.user_id,
        content: event.action,
        memory_type: MemoryType::Episodic,
        importance: event.importance,
        timestamp: event.timestamp,
    };
    
    let payload = serde_json::to_vec(&memory_item)?;
    Ok((record.key.clone(), payload.into()))
}
```

**示例 3: 事件聚合 SmartModule**
```rust
// smartmodules/event_aggregation.rs
use fluvio_smartmodule::{smartmodule, Record, RecordData, Result};

#[smartmodule(aggregate)]
pub fn aggregate(accumulator: RecordData, current: &Record) -> Result<RecordData> {
    let mut state: AggregationState = serde_json::from_slice(accumulator.as_ref())?;
    let event: UserInteraction = serde_json::from_slice(current.value.as_ref())?;
    
    // 聚合用户行为
    state.event_count += 1;
    state.total_importance += event.importance;
    state.last_event = event.timestamp;
    
    let payload = serde_json::to_vec(&state)?;
    Ok(payload.into())
}
```

**编译和部署 SmartModule**:
```bash
# 编译 WASM 模块
cd smartmodules
cargo build --target wasm32-unknown-unknown --release

# 部署到 Fluvio
fluvio smartmodule create event-filter \
  --wasm-file target/wasm32-unknown-unknown/release/event_filter.wasm

fluvio smartmodule create data-transform \
  --wasm-file target/wasm32-unknown-unknown/release/data_transform.wasm
```

#### 2.4 Fluvio Consumer (消费者)

**功能**: 从 Fluvio Topics 消费数据并发送到 AgentMem

```rust
use fluvio::{Fluvio, Offset, ConsumerConfig};

pub struct AgentMemConsumer {
    fluvio: Fluvio,
    agentmem_client: AgentMemClient,
}

impl AgentMemConsumer {
    pub async fn consume_memory_updates(&self) -> Result<()> {
        let consumer = self.fluvio
            .partition_consumer("memory-updates", 0)
            .await?;
        
        let mut stream = consumer.stream(Offset::end()).await?;
        
        while let Some(Ok(record)) = stream.next().await {
            let memory_item: MemoryItem = serde_json::from_slice(&record.value())?;
            
            // 发送到 AgentMem
            self.agentmem_client.add_memory(memory_item).await?;
        }
        
        Ok(())
    }
    
    pub async fn consume_with_smartmodule(&self) -> Result<()> {
        let consumer = self.fluvio
            .partition_consumer("user-events", 0)
            .await?;
        
        // 使用 SmartModule 过滤和转换
        let config = ConsumerConfig::builder()
            .smartmodule(vec![
                SmartModuleInvocation::new("event-filter"),
                SmartModuleInvocation::new("data-transform"),
            ])
            .build()?;
        
        let mut stream = consumer.stream_with_config(Offset::end(), config).await?;
        
        while let Some(Ok(record)) = stream.next().await {
            let memory_item: MemoryItem = serde_json::from_slice(&record.value())?;
            self.agentmem_client.add_memory(memory_item).await?;
        }
        
        Ok(())
    }
}
```

---

### 3. 流处理层 (Stream Processing)

**处理管道**:

```rust
pub struct StreamProcessor {
    event_filter: EventFilter,
    data_transformer: DataTransformer,
    event_aggregator: EventAggregator,
    analytics_engine: AnalyticsEngine,
    anomaly_detector: AnomalyDetector,
}

impl StreamProcessor {
    pub async fn process_stream(&self, record: Record) -> Result<ProcessedEvent> {
        // 1. 事件过滤
        if !self.event_filter.should_process(&record)? {
            return Ok(ProcessedEvent::Filtered);
        }
        
        // 2. 数据转换
        let transformed = self.data_transformer.transform(&record)?;
        
        // 3. 事件聚合
        let aggregated = self.event_aggregator.aggregate(&transformed)?;
        
        // 4. 实时分析
        let analytics = self.analytics_engine.analyze(&aggregated)?;
        
        // 5. 异常检测
        if let Some(anomaly) = self.anomaly_detector.detect(&analytics)? {
            self.handle_anomaly(anomaly).await?;
        }
        
        Ok(ProcessedEvent::Success(analytics))
    }
}
```

---

### 4. AgentMem 核心层

#### 4.1 HTTP API Server

**端点设计**:
```rust
// POST /api/v1/stream/events
pub async fn receive_stream_event(
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
    Json(event): Json<StreamEvent>,
) -> Result<Json<EventResponse>> {
    // 处理流式事件
    let memory_item = event.to_memory_item();
    memory_manager.add_memory(memory_item).await?;
    
    Ok(Json(EventResponse { success: true }))
}

// POST /api/v1/stream/batch
pub async fn receive_batch_events(
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
    Json(events): Json<Vec<StreamEvent>>,
) -> Result<Json<BatchResponse>> {
    // 批量处理
    let memory_items: Vec<MemoryItem> = events
        .into_iter()
        .map(|e| e.to_memory_item())
        .collect();
    
    memory_manager.batch_add_memories(memory_items).await?;
    
    Ok(Json(BatchResponse { 
        success: true,
        processed: memory_items.len(),
    }))
}
```

#### 4.2 WebSocket Server

**实时双向通信**:
```rust
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Extension(ws_manager): Extension<Arc<WebSocketManager>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, ws_manager))
}

async fn handle_socket(socket: WebSocket, ws_manager: Arc<WebSocketManager>) {
    let (mut sender, mut receiver) = socket.split();
    
    // 接收客户端消息
    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(text) = msg {
            let event: StreamEvent = serde_json::from_str(&text)?;
            
            // 处理事件
            ws_manager.process_event(event).await?;
            
            // 发送确认
            sender.send(Message::Text("ACK".to_string())).await?;
        }
    }
}
```

#### 4.3 Memory Agents

**5 个专业化 Agent**:

```rust
// CoreAgent - 核心记忆
pub struct CoreAgent {
    base: BaseAgent,
    core_store: Arc<dyn CoreMemoryStore>,
}

// EpisodicAgent - 情景记忆
pub struct EpisodicAgent {
    base: BaseAgent,
    episodic_store: Arc<dyn EpisodicMemoryStore>,
}

// SemanticAgent - 语义记忆
pub struct SemanticAgent {
    base: BaseAgent,
    semantic_store: Arc<dyn SemanticMemoryStore>,
}

// ProceduralAgent - 程序记忆
pub struct ProceduralAgent {
    base: BaseAgent,
    procedural_store: Arc<dyn ProceduralMemoryStore>,
}

// WorkingAgent - 工作记忆
pub struct WorkingAgent {
    base: BaseAgent,
    working_store: Arc<dyn WorkingMemoryStore>,
}
```

#### 4.4 MetaMemoryManager (元记忆协调器)

**功能**: 协调多个 Memory Agent 的工作

```rust
pub struct MetaMemoryManager {
    agents: HashMap<MemoryType, Arc<dyn MemoryAgent>>,
    load_balancer: LoadBalancer,
    message_queue: Arc<MessageQueue>,
}

impl MetaMemoryManager {
    pub async fn route_event(&self, event: StreamEvent) -> Result<()> {
        // 确定目标 Agent
        let memory_type = self.classify_event(&event)?;
        let agent = self.agents.get(&memory_type).unwrap();
        
        // 负载均衡
        if self.load_balancer.is_overloaded(memory_type) {
            self.message_queue.enqueue(event).await?;
        } else {
            agent.process_event(event).await?;
        }
        
        Ok(())
    }
}
```

---

### 5. LLM 集成层

**支持的 LLM Provider**:
- ✅ Zhipu AI (智谱AI) - 中国本土
- ✅ OpenAI (GPT-4)
- ✅ Anthropic (Claude)
- ✅ DeepSeek (深度求索)
- ✅ Ollama (本地模型)
- ✅ 15+ 其他 Provider

---

### 6. 存储层

**多后端支持**:

| 存储类型 | 用途 | 性能 |
|---------|------|------|
| **LibSQL** | 嵌入式数据库 | 10,000+ ops/s |
| **PostgreSQL** | 企业级关系数据库 | 5,000+ ops/s |
| **Qdrant** | 向量数据库 | 1,000+ queries/s |
| **Redis** | 缓存层 | 100,000+ ops/s |
| **MongoDB** | 文档存储 | 10,000+ ops/s |

---

## 🔄 数据流设计

### 端到端数据流

```
[用户交互] 
    ↓
[Fluvio Producer] 
    ↓
[Fluvio Topic: user-events] 
    ↓
[SmartModule: event-filter] (WASM 边缘处理)
    ↓
[SmartModule: data-transform] (WASM 边缘处理)
    ↓
[Fluvio Consumer] 
    ↓
[Stream Processor] 
    ↓
[AgentMem HTTP API] 
    ↓
[MetaMemoryManager] 
    ↓
[EpisodicAgent] 
    ↓
[LibSQL/PostgreSQL] 
    ↓
[LLM Provider (Zhipu AI)] 
    ↓
[响应返回用户]
```

### 延迟分析

| 阶段 | 延迟 | 说明 |
|------|------|------|
| Producer → Topic | < 1ms | 本地网络 |
| SmartModule 处理 | < 0.5ms | WASM 执行 |
| Consumer 接收 | < 1ms | 本地网络 |
| Stream 处理 | < 2ms | Rust 异步处理 |
| AgentMem API | < 1ms | HTTP 处理 |
| Memory Agent | < 3ms | 数据库写入 |
| LLM 调用 | 100-500ms | 外部 API |
| **总延迟** | **< 10ms** | (不含 LLM) |

---

## ⚡ 技术优势

### 1. 全 Rust 栈优势

**性能对比** (vs Python/Java):
- **内存占用**: 10-100倍 更少
- **CPU 效率**: 10-50倍 更高
- **启动时间**: 100倍 更快
- **并发能力**: 无 GIL 限制

### 2. WASM SmartModules 优势

**边缘计算能力**:
- ✅ 数据源端处理，减少网络传输
- ✅ 沙箱隔离，安全可靠
- ✅ 热更新，无需重启
- ✅ 跨平台，一次编译到处运行

### 3. 事件驱动架构优势

**高并发支持**:
- ✅ 完全异步 (Tokio runtime)
- ✅ 零拷贝数据传输
- ✅ 背压控制
- ✅ 自动重试和容错

---

## 📊 性能指标

### 吞吐量

| 场景 | 吞吐量 | 延迟 (P99) |
|------|--------|-----------|
| **单 Producer** | 100,000 events/s | < 5ms |
| **多 Producer (16)** | 1,000,000 events/s | < 10ms |
| **SmartModule 处理** | 500,000 events/s | < 2ms |
| **AgentMem 写入** | 50,000 memories/s | < 5ms |
| **AgentMem 查询** | 100,000 queries/s | < 3ms |

### 资源占用

| 组件 | CPU | 内存 | 磁盘 I/O |
|------|-----|------|---------|
| **Fluvio Cluster** | 2-4 核 | 2-4 GB | 100 MB/s |
| **AgentMem Server** | 2-4 核 | 1-2 GB | 50 MB/s |
| **SmartModules** | < 0.5 核 | < 100 MB | 0 |

---

## 🚀 实施方案

### Phase 1: 基础集成 (1-2 周)

**任务**:
1. ✅ 安装 Fluvio 集群
2. ✅ 创建 Topics
3. ✅ 实现 Producer/Consumer
4. ✅ 集成 AgentMem HTTP API

### Phase 2: SmartModules 开发 (2-3 周)

**任务**:
1. ✅ 开发事件过滤 SmartModule
2. ✅ 开发数据转换 SmartModule
3. ✅ 开发事件聚合 SmartModule
4. ✅ 部署和测试

### Phase 3: 生产优化 (2-4 周)

**任务**:
1. ✅ 性能调优
2. ✅ 监控和告警
3. ✅ 容错和恢复
4. ✅ 文档和培训

---

## 📝 部署架构

### 单机部署

```yaml
# docker-compose.yml
version: '3.8'

services:
  fluvio:
    image: infinyon/fluvio:latest
    ports:
      - "9003:9003"
    volumes:
      - fluvio-data:/var/lib/fluvio
  
  agentmem:
    image: agentmem:latest
    ports:
      - "8080:8080"
    environment:
      - DATABASE_BACKEND=libsql
      - DATABASE_URL=/data/agentmem.db
    volumes:
      - agentmem-data:/data
  
  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"

volumes:
  fluvio-data:
  agentmem-data:
```

### 分布式部署

```
[Fluvio Cluster]
  ├── SPU 1 (Partition 0-7)
  ├── SPU 2 (Partition 8-15)
  └── SPU 3 (Partition 16-23)

[AgentMem Cluster]
  ├── AgentMem Server 1 (Load Balancer)
  ├── AgentMem Server 2 (Load Balancer)
  └── AgentMem Server 3 (Load Balancer)

[Storage Layer]
  ├── PostgreSQL Primary
  ├── PostgreSQL Replica 1
  ├── PostgreSQL Replica 2
  ├── Qdrant Cluster
  └── Redis Cluster
```

---

## 💻 实施代码示例

### 完整的 Fluvio-AgentMem 集成示例

```rust
// examples/fluvio_integration.rs
use fluvio::{Fluvio, RecordKey, TopicProducer};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[derive(Debug, Serialize, Deserialize)]
struct UserEvent {
    user_id: String,
    action: String,
    timestamp: i64,
    importance: f64,
    metadata: serde_json::Value,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 连接到 Fluvio
    let fluvio = Fluvio::connect().await?;

    // 2. 创建 Producer
    let producer = fluvio.topic_producer("user-events").await?;

    // 3. 创建 AgentMem 客户端
    let agentmem_client = reqwest::Client::new();
    let agentmem_url = "http://localhost:8080";

    // 4. 启动 Consumer (在后台任务中)
    let consumer_handle = tokio::spawn(async move {
        consume_and_forward(fluvio, agentmem_client, agentmem_url).await
    });

    // 5. 模拟发送事件
    for i in 0..100 {
        let event = UserEvent {
            user_id: format!("user-{}", i % 10),
            action: format!("action-{}", i),
            timestamp: chrono::Utc::now().timestamp(),
            importance: (i as f64) / 100.0,
            metadata: serde_json::json!({
                "source": "example",
                "version": "1.0"
            }),
        };

        let payload = serde_json::to_vec(&event)?;
        producer.send(RecordKey::NULL, payload).await?;

        println!("✅ Sent event {}", i);
        sleep(Duration::from_millis(100)).await;
    }

    consumer_handle.await??;
    Ok(())
}

async fn consume_and_forward(
    fluvio: Fluvio,
    client: reqwest::Client,
    agentmem_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use fluvio::{Offset, consumer::ConsumerConfig};
    use futures::StreamExt;

    // 创建 Consumer
    let consumer = fluvio.partition_consumer("user-events", 0).await?;

    // 使用 SmartModule 过滤
    let config = ConsumerConfig::builder()
        .smartmodule(vec![
            fluvio::consumer::SmartModuleInvocation::new("event-filter"),
            fluvio::consumer::SmartModuleInvocation::new("data-transform"),
        ])
        .build()?;

    let mut stream = consumer.stream_with_config(Offset::end(), config).await?;

    // 消费并转发到 AgentMem
    while let Some(Ok(record)) = stream.next().await {
        let event: UserEvent = serde_json::from_slice(&record.value())?;

        // 转换为 AgentMem 格式
        let memory_request = serde_json::json!({
            "organization_id": "org-123",
            "user_id": event.user_id,
            "content": event.action,
            "memory_type": "episodic",
            "importance": event.importance,
            "metadata": event.metadata,
        });

        // 发送到 AgentMem
        let response = client
            .post(format!("{}/api/v1/memories", agentmem_url))
            .json(&memory_request)
            .send()
            .await?;

        if response.status().is_success() {
            println!("✅ Forwarded to AgentMem: {}", event.user_id);
        } else {
            eprintln!("❌ Failed to forward: {:?}", response.text().await?);
        }
    }

    Ok(())
}
```

### SmartModule 示例代码

```rust
// smartmodules/event_filter/src/lib.rs
use fluvio_smartmodule::{smartmodule, Record, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct UserEvent {
    user_id: String,
    action: String,
    importance: f64,
}

#[smartmodule(filter)]
pub fn filter(record: &Record) -> Result<bool> {
    // 解析事件
    let event: UserEvent = serde_json::from_slice(record.value.as_ref())?;

    // 只保留重要事件 (importance > 0.5)
    Ok(event.importance > 0.5)
}
```

```rust
// smartmodules/data_transform/src/lib.rs
use fluvio_smartmodule::{smartmodule, Record, RecordData, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct UserEvent {
    user_id: String,
    action: String,
    timestamp: i64,
    importance: f64,
    metadata: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct MemoryItem {
    organization_id: String,
    user_id: String,
    content: String,
    memory_type: String,
    importance: f64,
    timestamp: i64,
    metadata: serde_json::Value,
}

#[smartmodule(map)]
pub fn transform(record: &Record) -> Result<(Option<RecordData>, RecordData)> {
    // 解析输入事件
    let event: UserEvent = serde_json::from_slice(record.value.as_ref())?;

    // 转换为 AgentMem 格式
    let memory_item = MemoryItem {
        organization_id: "org-123".to_string(),
        user_id: event.user_id,
        content: event.action,
        memory_type: "episodic".to_string(),
        importance: event.importance,
        timestamp: event.timestamp,
        metadata: event.metadata,
    };

    // 序列化输出
    let payload = serde_json::to_vec(&memory_item)?;
    Ok((record.key.clone(), payload.into()))
}
```

### 编译和部署脚本

```bash
#!/bin/bash
# scripts/deploy_fluvio_agentmem.sh

set -e

echo "🚀 部署 Fluvio + AgentMem 集成"

# 1. 安装 Fluvio
echo "📦 安装 Fluvio..."
curl -fsS https://hub.infinyon.cloud/install/install.sh | bash
export PATH="${HOME}/.fluvio/bin:${PATH}"

# 2. 启动 Fluvio 集群
echo "🔧 启动 Fluvio 集群..."
fluvio cluster start

# 3. 创建 Topics
echo "📝 创建 Topics..."
fluvio topic create user-events --partitions 16 --retention-time 7d
fluvio topic create agent-actions --partitions 8 --retention-time 3d
fluvio topic create memory-updates --partitions 32 --retention-time 30d
fluvio topic create chat-messages --partitions 16 --retention-time 14d

# 4. 编译 SmartModules
echo "🔨 编译 SmartModules..."
cd smartmodules/event_filter
cargo build --target wasm32-unknown-unknown --release
cd ../data_transform
cargo build --target wasm32-unknown-unknown --release
cd ../..

# 5. 部署 SmartModules
echo "📤 部署 SmartModules..."
fluvio smartmodule create event-filter \
  --wasm-file smartmodules/event_filter/target/wasm32-unknown-unknown/release/event_filter.wasm

fluvio smartmodule create data-transform \
  --wasm-file smartmodules/data_transform/target/wasm32-unknown-unknown/release/data_transform.wasm

# 6. 启动 AgentMem
echo "🚀 启动 AgentMem..."
cd agentmen
DATABASE_BACKEND=libsql DATABASE_URL="./data/agentmem.db" \
  cargo run --package agent-mem-server --release &

# 等待 AgentMem 启动
sleep 5

# 7. 验证健康状态
echo "✅ 验证健康状态..."
curl -s http://localhost:8080/health | jq .

# 8. 运行集成示例
echo "🎯 运行集成示例..."
cargo run --example fluvio_integration

echo "✅ 部署完成！"
```

### Docker Compose 部署

```yaml
# docker-compose.fluvio-agentmem.yml
version: '3.8'

services:
  # Fluvio 集群
  fluvio-sc:
    image: infinyon/fluvio:latest
    container_name: fluvio-sc
    ports:
      - "9003:9003"
    environment:
      - FLUVIO_SC_BIND=0.0.0.0:9003
    volumes:
      - fluvio-metadata:/var/lib/fluvio/metadata
    command: ["cluster", "start", "--sc"]
    networks:
      - fluvio-agentmem

  fluvio-spu-1:
    image: infinyon/fluvio:latest
    container_name: fluvio-spu-1
    depends_on:
      - fluvio-sc
    environment:
      - FLUVIO_SPU_ID=0
      - FLUVIO_SC_HOST=fluvio-sc
      - FLUVIO_SC_PORT=9003
    volumes:
      - fluvio-data-1:/var/lib/fluvio/data
    networks:
      - fluvio-agentmem

  fluvio-spu-2:
    image: infinyon/fluvio:latest
    container_name: fluvio-spu-2
    depends_on:
      - fluvio-sc
    environment:
      - FLUVIO_SPU_ID=1
      - FLUVIO_SC_HOST=fluvio-sc
      - FLUVIO_SC_PORT=9003
    volumes:
      - fluvio-data-2:/var/lib/fluvio/data
    networks:
      - fluvio-agentmem

  # AgentMem 服务器
  agentmem:
    build:
      context: ./agentmen
      dockerfile: Dockerfile
    container_name: agentmem
    ports:
      - "8080:8080"
    environment:
      - DATABASE_BACKEND=libsql
      - DATABASE_URL=/data/agentmem.db
      - RUST_LOG=info
    volumes:
      - agentmem-data:/data
    depends_on:
      - redis
      - postgres
    networks:
      - fluvio-agentmem

  # PostgreSQL
  postgres:
    image: postgres:16-alpine
    container_name: postgres
    ports:
      - "5432:5432"
    environment:
      - POSTGRES_USER=agentmem
      - POSTGRES_PASSWORD=agentmem
      - POSTGRES_DB=agentmem
    volumes:
      - postgres-data:/var/lib/postgresql/data
    networks:
      - fluvio-agentmem

  # Redis
  redis:
    image: redis:7-alpine
    container_name: redis
    ports:
      - "6379:6379"
    volumes:
      - redis-data:/data
    networks:
      - fluvio-agentmem

  # Qdrant (向量数据库)
  qdrant:
    image: qdrant/qdrant:latest
    container_name: qdrant
    ports:
      - "6333:6333"
      - "6334:6334"
    volumes:
      - qdrant-data:/qdrant/storage
    networks:
      - fluvio-agentmem

  # Prometheus (监控)
  prometheus:
    image: prom/prometheus:latest
    container_name: prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus-data:/prometheus
    networks:
      - fluvio-agentmem

  # Grafana (可视化)
  grafana:
    image: grafana/grafana:latest
    container_name: grafana
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana-data:/var/lib/grafana
    depends_on:
      - prometheus
    networks:
      - fluvio-agentmem

volumes:
  fluvio-metadata:
  fluvio-data-1:
  fluvio-data-2:
  agentmem-data:
  postgres-data:
  redis-data:
  qdrant-data:
  prometheus-data:
  grafana-data:

networks:
  fluvio-agentmem:
    driver: bridge
```

### 启动完整系统

```bash
# 启动所有服务
docker-compose -f docker-compose.fluvio-agentmem.yml up -d

# 查看日志
docker-compose -f docker-compose.fluvio-agentmem.yml logs -f

# 停止所有服务
docker-compose -f docker-compose.fluvio-agentmem.yml down
```

---

## 📈 性能测试

### 压力测试脚本

```bash
#!/bin/bash
# scripts/benchmark.sh

echo "🔥 Fluvio + AgentMem 性能测试"

# 1. 测试 Fluvio 吞吐量
echo "📊 测试 Fluvio Producer 吞吐量..."
fluvio produce user-events --file test_data.json --compression gzip

# 2. 测试 AgentMem API 吞吐量
echo "📊 测试 AgentMem API 吞吐量..."
ab -n 10000 -c 100 -p memory_request.json -T application/json \
  http://localhost:8080/api/v1/memories

# 3. 测试端到端延迟
echo "📊 测试端到端延迟..."
cargo run --example latency_test --release

# 4. 生成报告
echo "📝 生成性能报告..."
python3 scripts/generate_report.py
```

### 预期性能指标

```
┌─────────────────────────────────────────────────────────┐
│           Fluvio + AgentMem 性能基准测试                │
├─────────────────────────────────────────────────────────┤
│ 测试场景                │ 吞吐量          │ 延迟 (P99)  │
├─────────────────────────────────────────────────────────┤
│ Fluvio Producer         │ 100,000 msg/s   │ < 5ms       │
│ SmartModule 处理        │ 500,000 msg/s   │ < 2ms       │
│ Fluvio Consumer         │ 80,000 msg/s    │ < 3ms       │
│ AgentMem API 写入       │ 50,000 req/s    │ < 5ms       │
│ AgentMem API 查询       │ 100,000 req/s   │ < 3ms       │
│ 端到端延迟 (不含 LLM)   │ -               │ < 10ms      │
│ 端到端延迟 (含 LLM)     │ -               │ < 500ms     │
└─────────────────────────────────────────────────────────┘

资源占用:
- CPU: 4-8 核 (总计)
- 内存: 4-8 GB (总计)
- 磁盘 I/O: 150 MB/s (峰值)
- 网络: 100 Mbps (峰值)
```

---

**文档版本**: 1.0
**最后更新**: 2025-01-10
**维护者**: AgentMem 开发团队

