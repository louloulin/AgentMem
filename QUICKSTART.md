# AgentMem 快速开始指南

> **5 分钟快速上手 AgentMem**
>
> 最后更新: 2025-10-21

---

## 📦 安装

### 方式 1: 添加依赖

```toml
# Cargo.toml
[dependencies]
agent-mem = { path = "crates/agent-mem" }
tokio = { version = "1.0", features = ["full"] }
```

### 方式 2: 克隆仓库

```bash
git clone https://gitcode.com/louloulin/agentmem.git
cd agentmem
cargo build --release
```

---

## 🚀 基础使用

### 零配置模式（推荐开始）

```rust
use agent_mem::Memory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 创建 Memory 实例（零配置）
    let mem = Memory::new().await?;
    
    // 2. 添加记忆
    let result = mem.add("我喜欢喝咖啡，特别是拿铁").await?;
    println!("添加成功: {:?}", result);
    
    // 3. 搜索记忆
    let results = mem.search("饮品偏好", None).await?;
    println!("找到 {} 条记忆", results.len());
    
    Ok(())
}
```

**特点**:
- ✅ 零配置：无需任何设置
- ✅ 嵌入式：LibSQL + LanceDB 自动创建
- ✅ 快速：<100ms 启动，31,456 ops/s 写入
- ✅ 完整：支持所有核心功能

---

## 🧠 智能模式（mem0 兼容）

### 启用智能推理

```rust
use agent_mem::{Memory, types::AddMemoryOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mem = Memory::new().await?;
    
    // 智能模式：自动事实提取、去重、冲突检测
    let mut options = AddMemoryOptions::default();
    options.infer = true;  // 启用智能推理
    
    let result = mem.add_with_options(
        "我现在在上海工作，之前在北京",
        options
    ).await?;
    
    // 输出决策结果（ADD/UPDATE/DELETE）
    for event in result.results {
        println!("{}: {}", event.event, event.memory);
    }
    
    Ok(())
}
```

**智能功能**:
- ✅ 事实提取：自动提取关键信息
- ✅ 自动去重：避免重复记忆
- ✅ 冲突检测：识别矛盾信息
- ✅ 智能决策：ADD/UPDATE/DELETE/MERGE
- ✅ 关系提取：构建知识图谱

---

## 🔍 高级搜索

### 混合搜索 + 相似度阈值

```rust
use agent_mem::{Memory, types::SearchOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mem = Memory::new().await?;
    
    // 添加多条记忆
    mem.add("我喜欢披萨").await?;
    mem.add("我爱意大利面").await?;
    mem.add("我享受意大利美食").await?;
    
    // 混合搜索：向量 + 全文 + BM25 + 模糊匹配
    let mut options = SearchOptions::default();
    options.limit = Some(10);
    options.threshold = Some(0.7);  // 只返回相似度 > 0.7 的结果
    
    let results = mem.search("意大利美食", Some(options)).await?;
    
    for item in results {
        println!("- {}: {} (重要性: {:.2})", 
            item.id, item.content, item.importance);
    }
    
    Ok(())
}
```

---

## 🖼️ 多模态支持

### 图像记忆

```rust
use agent_mem::{Memory, types::AddMemoryOptions};
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mem = Memory::new().await?;
    
    // 读取图像文件
    let image_data = fs::read("photo.jpg")?;
    
    // 添加图像记忆
    let mut options = AddMemoryOptions::default();
    options.metadata.insert("filename".to_string(), "photo.jpg".to_string());
    options.metadata.insert("taken_at".to_string(), "2024-10-21".to_string());
    
    let result = mem.add_image(image_data, Some(options)).await?;
    println!("图像记忆已添加: {:?}", result);
    
    Ok(())
}
```

### 音频记忆

```rust
// 读取音频文件
let audio_data = fs::read("recording.mp3")?;

// 添加音频记忆（自动转录）
let mut options = AddMemoryOptions::default();
options.metadata.insert("language".to_string(), "zh".to_string());

let result = mem.add_audio(audio_data, Some(options)).await?;
```

---

## ⚡ 性能优化

### 批量添加（并行处理）

```rust
use agent_mem::{Memory, types::AddMemoryOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mem = Memory::new().await?;
    
    // 批量添加（并行处理）
    let contents = vec![
        "记忆 1".to_string(),
        "记忆 2".to_string(),
        "记忆 3".to_string(),
        // ... 1000 条记忆
    ];
    
    let options = AddMemoryOptions::default();
    let results = mem.add_batch(contents, options).await?;
    
    println!("批量添加完成: {} 成功", results.len());
    // 性能: 100,000+ ops/s (批量) vs 31,456 ops/s (单个)
    
    Ok(())
}
```

### 缓存搜索（智能缓存）

```rust
// 第一次查询（命中数据库，~15ms）
let results1 = mem.search_cached("pizza", None).await?;

// 第二次查询（命中缓存，<1ms）
let results2 = mem.search_cached("pizza", None).await?;

// 性能提升: 15x+
```

---

## 🔧 配置选项

### Builder 模式

```rust
use agent_mem::Memory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mem = Memory::builder()
        .with_storage("postgresql://user:pass@localhost/agentmem")
        .with_llm("openai", "gpt-4-turbo-preview")
        .with_embedder("openai", "text-embedding-3-small")
        .with_vector_store("qdrant", "http://localhost:6333")
        .enable_intelligent_features()
        .build()
        .await?;
    
    Ok(())
}
```

### 环境变量配置

```bash
# LLM 配置
export ZHIPU_API_KEY="your-zhipu-api-key"      # 优先级 1
export OPENAI_API_KEY="your-openai-api-key"   # 优先级 2
export ANTHROPIC_API_KEY="your-key"            # 优先级 3

# 向量存储配置
export VECTOR_STORE="qdrant"                   # 默认: lancedb
export QDRANT_URL="http://localhost:6333"

# 数据库配置
export DATABASE_URL="postgresql://localhost/agentmem"  # 默认: LibSQL

# 启动
cargo run --bin agentmem-server
```

---

## 📊 性能监控

### 获取性能统计

```rust
let mem = Memory::new().await?;

// 添加一些记忆...
mem.add("test").await?;

// 获取性能统计
let stats = mem.get_performance_stats().await?;
println!("总记忆数: {}", stats.total_memories);
println!("缓存命中率: {:.2}%", stats.cache_hit_rate * 100.0);
println!("平均添加延迟: {:.2}ms", stats.avg_add_latency_ms);
println!("平均搜索延迟: {:.2}ms", stats.avg_search_latency_ms);
println!("QPS: {:.0}", stats.queries_per_second);
```

---

## 🐳 Docker 部署

### 使用 Docker Compose

```bash
# 克隆仓库
git clone https://gitcode.com/louloulin/agentmem.git
cd agentmem

# 启动完整服务栈
docker-compose up -d

# 包含服务:
# - AgentMem Server
# - PostgreSQL
# - Redis
# - Qdrant
# - Prometheus
# - Grafana
```

### HTTP API 使用

```bash
# 添加记忆
curl -X POST http://localhost:8080/api/v1/memories \
  -H "Content-Type: application/json" \
  -d '{
    "content": "I love pizza",
    "user_id": "alice",
    "infer": true
  }'

# 搜索记忆
curl -X GET "http://localhost:8080/api/v1/memories/search?query=food&user_id=alice&limit=10"

# 获取统计
curl -X GET http://localhost:8080/api/v1/stats
```

---

## ☸️ Kubernetes 部署

### 使用 Helm

```bash
# 添加 Helm 仓库
helm repo add agentmem https://agentmem.github.io/charts

# 安装
helm install agentmem agentmem/agentmem \
  --set postgresql.enabled=true \
  --set redis.enabled=true \
  --set qdrant.enabled=true

# 检查状态
kubectl get pods -l app=agentmem
```

### 使用 kubectl

```bash
# 部署到 Kubernetes
kubectl apply -f k8s/deployment.yaml

# 检查部署
kubectl get all -l app=agentmem

# 查看日志
kubectl logs -f deployment/agentmem-server
```

---

## 📚 更多示例

### 示例项目

```bash
# 智能推理演示
cargo run --bin intelligent-reasoning-demo

# Mem0 兼容演示
cargo run --bin mem0-compat-demo

# 多模态演示
cargo run --bin multimodal-demo

# 性能测试
cargo run --bin performance-demo

# 完整功能演示
cargo run --bin complete_demo
```

### 示例列表

查看 `examples/` 目录，包含 **86 个示例项目**：
- 基础使用示例
- 智能功能示例
- 多模态处理示例
- 性能优化示例
- 集成测试示例
- 生产部署示例

---

## 🔗 相关资源

### 文档

- **战略分析**: `agentmem100.md` (3,492 行完整分析)
- **技术计划**: `agentmem30.md` (2,407 行实施计划)
- **API 文档**: `docs/api-reference.md`
- **架构设计**: `BEST_ARCHITECTURE_DESIGN.md`

### 社区

- **GitHub**: https://github.com/agentmem/agentmem
- **文档站**: https://docs.agentmem.dev
- **Discord**: https://discord.gg/agentmem
- **技术博客**: https://blog.agentmem.dev

### 支持

- **问题反馈**: GitHub Issues
- **技术讨论**: Discord
- **企业咨询**: enterprise@agentmem.dev

---

## 🎯 核心特性速查

| 特性 | 命令 | 说明 |
|------|------|------|
| **添加记忆** | `mem.add("content")` | 简单模式 |
| **智能添加** | `mem.add_with_options(content, {infer: true})` | 事实提取 + 去重 |
| **搜索记忆** | `mem.search("query", options)` | 混合搜索 |
| **批量添加** | `mem.add_batch(contents, options)` | 并行处理 |
| **缓存搜索** | `mem.search_cached("query", options)` | 智能缓存 |
| **图像记忆** | `mem.add_image(image_data, options)` | 多模态 |
| **音频记忆** | `mem.add_audio(audio_data, options)` | 多模态 |
| **视频记忆** | `mem.add_video(video_data, options)` | 多模态 |
| **性能统计** | `mem.get_performance_stats()` | 监控 |

---

## ⚙️ 性能调优建议

### 开发环境

```rust
// 使用嵌入式模式（零配置）
let mem = Memory::new().await?;

// 性能指标:
// - 启动时间: <100ms
// - 添加: 31,456 ops/s
// - 搜索: ~23ms
// - 存储: 本地文件
```

### 生产环境

```rust
// 使用服务器模式（PostgreSQL + Qdrant）
let mem = Memory::builder()
    .with_storage("postgresql://prod-db:5432/agentmem")
    .with_vector_store("qdrant", "http://qdrant:6333")
    .with_cache("redis://redis:6379")
    .with_llm("openai", "gpt-4-turbo")
    .build()
    .await?;

// 性能指标:
// - 并发: 10,000+ QPS
// - 可用性: 99.95%
// - 延迟: P95 < 50ms
// - 扩展: 无限水平扩展
```

---

## 🎓 学习路径

### 初学者

1. 阅读本快速指南（5分钟）
2. 运行基础示例 `examples/simple-demo`（10分钟）
3. 尝试智能模式 `examples/intelligent-reasoning-demo`（20分钟）
4. 阅读 API 文档（30分钟）

### 进阶开发者

1. 学习架构设计 `BEST_ARCHITECTURE_DESIGN.md`
2. 研究智能组件 `agent-mem-intelligence/`
3. 探索混合搜索 `agent-mem-core/src/search/`
4. 部署生产环境 `docker-compose.yml` + `k8s/`

### 企业用户

1. 阅读商业分析 `agentmem100.md`（应用场景、ROI）
2. 评估部署方案（私有化 vs SaaS）
3. 进行 POC 测试
4. 联系企业支持（enterprise@agentmem.dev）

---

## 🆘 常见问题

### Q1: 如何启用智能功能？

A: 设置环境变量 `ZHIPU_API_KEY` 或 `OPENAI_API_KEY`，AgentMem 会自动启用智能功能。

### Q2: 性能不够怎么办？

A: 
1. 使用批量操作 `add_batch()`
2. 启用缓存 `search_cached()`
3. 使用服务器模式（PostgreSQL + Qdrant）
4. 增加硬件资源（CPU、内存）

### Q3: 如何切换向量库？

A:
```rust
let mem = Memory::builder()
    .with_vector_store("qdrant", "http://localhost:6333")  // Qdrant
    // .with_vector_store("pinecone", "api-key")          // Pinecone
    // .with_vector_store("chroma", "http://localhost:8000") // Chroma
    .build().await?;
```

### Q4: 支持哪些 LLM？

A: 16 个 LLM 提供商：
- OpenAI (GPT-4, GPT-3.5)
- Anthropic (Claude 3.5)
- 智谱 AI (ChatGLM)
- DeepSeek
- Ollama (本地)
- 等等...

### Q5: 数据存储在哪里？

A:
- **嵌入式模式**: `./data/agentmem.db` (LibSQL) + `./data/memory_vectors.lance` (LanceDB)
- **服务器模式**: PostgreSQL + Qdrant（可配置）

---

## ✅ 下一步

- 📖 阅读完整文档: `agentmem100.md`
- 🧪 运行示例程序: `examples/`
- 🚀 部署到生产: `docker-compose.yml` 或 `k8s/`
- 💬 加入社区: Discord
- 📧 企业咨询: enterprise@agentmem.dev

---

**AgentMem** - 为 AI 应用提供智能记忆能力 🧠✨

