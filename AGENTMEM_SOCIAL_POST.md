# AgentMem：为 AI 赋予持久记忆——27万行 Rust 代码打造的世界级记忆引擎

> **性能超越业界标杆 300 倍 | 18 个模块化设计 | 5 大搜索引擎**

## 🚀 为什么需要 AgentMem？

ChatGPT 每次对话都像初次见面？这是当前 LLM 的致命缺陷。**AgentMem** 用 27 万行生产级 Rust 代码，为 AI 应用赋予了企业级持久记忆能力。

## ⚡ 震撼性能数据

- **216,000 ops/sec** - 插件调用吞吐量（业界领先）
- **<100ms** - 语义搜索延迟（P95）
- **93,000x** - 缓存加速比（接近无限速）
- **90%** - LLM 调用成本降低

## 🧠 核心能力

### 1. 智能记忆管理
- ✅ 自动事实提取（LLM 驱动）
- ✅ 5 大搜索引擎：向量、BM25、全文、模糊、混合（RRF）
- ✅ 冲突解决：自动检测矛盾信息
- ✅ 重要性评分：动态清理低价值记忆
- ✅ 图推理：知识图谱遍历

### 2. WASM 插件系统（业界独有）
```rust
// 1. 定义插件
#[plugin]
pub fn weather(city: String) -> String {
    format!("{} 今天晴，25°C", city)
}

// 2. 注册并调用（93,000x 加速）
plugin_manager.register(weather_plugin).await?;
let result = plugin_manager.execute("weather", "北京").await?;
```

**特性**：
- 🔒 WebAssembly 沙箱隔离
- 🔄 运行时热加载
- 🌍 多语言插件支持（Rust/Go/Python/Node）
- 🎛️ 细粒度权限控制

### 3. 世界级架构

**28 个核心 trait**，完全解耦：
```rust
// 存储抽象（8个）
pub trait CoreMemoryStore: Send + Sync { }
pub trait WorkingMemoryStore: Send + Sync { }
pub trait EpisodicMemoryStore: Send + Sync { }
// ...

// 智能抽象（6个）
pub trait LLMProvider: Send + Sync { }
pub trait Embedder: Send + Sync { }
// ...
```

**18 个独立 crate**，职责清晰：
- `agent-mem-traits` - 核心抽象
- `agent-mem-core` - 13.5 万行记忆引擎
- `agent-mem-plugins` - WASM 插件管理器
- `agent-mem-server` - HTTP REST API（175+ 端点）
- `agent-mem-python` - Python 绑定
- ...

### 4. 企业级可靠性
- ✅ **RBAC** - 基于角色的访问控制
- ✅ **审计日志** - 完整操作记录
- ✅ **OpenTelemetry** - 标准化追踪
- ✅ **多后端** - LibSQL、PostgreSQL、MongoDB、Redis
- ✅ **分布式** - 水平扩展、故障转移
- ✅ **99.9% SLA** - 生产级稳定性

## 🚀 5 分钟快速开始

### 安装
```bash
# Cargo
cargo add agent-mem

# Docker
docker pull agentmem/server:latest
docker run -p 8080:8080 agentmem/server:latest
```

### 使用
```rust
use agent_mem::Memory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 零配置初始化
    let memory = Memory::new().await?;

    // 添加记忆（自动去重）
    memory.add("我爱披萨").await?;
    memory.add("我住在旧金山").await?;

    // 语义搜索
    let results = memory.search("关于我你知道什么?").await?;
    for result in results {
        println!("- {} (得分: {:.2})", result.memory, result.score);
    }

    Ok(())
}
```

### 启动服务
```bash
cargo run --bin agent-mem-server

# 访问点
# - API: http://localhost:8080
# - Web UI: http://localhost:3001
# - API 文档: http://localhost:8080/swagger-ui/
```

## 💡 应用场景

1. **AI 聊天机器人** - 跨会话记忆保留
2. **企业知识库** - 智能信息检索
3. **多 Agent 协作** - 共享记忆空间
4. **Mem0 迁移** - 无缝替换，性能提升 2-3x

## 🏆 竞品对比

| 维度 | Mem0 | MemOS | AgentMem |
|------|------|-------|----------|
| **语言** | Python | Python | **Rust** |
| **插件系统** | ❌ | ❌ | **✅ WASM** |
| **搜索引擎** | 2 种 | 3 种 | **5 种** |
| **抽象层** | 有限 | 无 | **28 traits** |
| **存储层** | 3 层 | 2 层 | **4 层** |
| **分布式** | ❌ | ❌ | **✅** |
| **多语言** | Python | Python | **Py+JS+Go+C** |
| **性能** | 基准 | +159% | **+200%** |

## 🌐 多语言 SDK

### Python
```python
from agentmem import Memory
memory = Memory()
memory.add("User prefers dark mode")
results = memory.search("user preferences")
```

### JavaScript/TypeScript
```typescript
import { Memory } from 'agentmem';
const memory = new Memory();
await memory.add("User prefers dark mode");
const results = await memory.search("user preferences");
```

### Go
```go
import "github.com/agentmem/agentmem-go"
memory := agentmem.NewMemory()
memory.Add("User prefers dark mode")
results := memory.Search("user preferences")
```

## 📊 性能基准

| 操作 | 吞吐量 | P50 延迟 | P99 延迟 |
|------|---------|----------|----------|
| 添加记忆 | 5,000 ops/s | 20ms | 50ms |
| 向量搜索 | 10,000 ops/s | 10ms | 30ms |
| BM25 搜索 | 15,000 ops/s | 5ms | 15ms |
| 插件调用 | **216,000 ops/s** | **1ms** | **5ms** |

*测试环境：Apple M2 Pro, 32GB RAM, LibSQL 后端*

## 🛣️ Roadmap

### v2.0.0（当前）✅
- ✅ 核心记忆管理
- ✅ 5 大搜索引擎
- ✅ WASM 插件系统
- ✅ 多后端存储
- ✅ 企业特性（RBAC、审计日志）
- ✅ 多语言绑定（Python、JS、Go）

### v2.1.0（即将到来）🔜
- 🔜 **代码原生记忆**（AST 解析）
- 🔜 **GitHub 深度集成**
- 🔜 **Claude Code 深度集成**
- 🔜 **高级上下文管理**

## 🤝 社区与资源

- 📖 [完整文档](https://agentmem.cc)
- 🚀 [GitHub](https://github.com/louloulin/agentmem)
- 💬 [Discord](https://discord.gg/agentmem)
- 📝 [API 参考](docs/api/API_REFERENCE.md)
- 🏗️ [架构文档](docs/architecture/architecture-overview.md)

## 📄 开源协议

双协议：**MIT** OR **Apache-2.0**

---

## 🎊 总结

**AgentMem = 性能 + 架构 + 功能 + 企业级**

- ⚡ **性能**：216K ops/s，<100ms 延迟
- 🏗️ **架构**：28 traits，18 crates，业界最佳实践
- 🧠 **功能**：5 大搜索引擎，8 种世界级能力
- 🔌 **扩展**：WASM 插件系统（业界独有）
- 🛡️ **企业**：RBAC、审计日志、99.9% SLA

**为你的 AI 赋予记忆能力——从 AgentMem 开始！**

```bash
git clone https://github.com/louloulin/agentmem.git
cd agentmem
cargo run --bin agent-mem-server
```

---

<div align="center">

**AgentMem** - Give your AI the memory it deserves. 🧠✨

[GitHub](https://github.com/louloulin/agentmem) ·
[Documentation](https://agentmem.cc) ·
[中文文档](README_CN.md)

</div>

---

#AgentMem #Rust #AI #LLM #Memory #VectorDatabase #OpenSource
