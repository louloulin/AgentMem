# AgentMem

<div align="center">

**企业级 AI 记忆平台 - 生产就绪**

[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/louloulin/agentmem/actions)
[![Coverage](https://img.shields.io/badge/coverage-95%25-green.svg)](https://github.com/louloulin/agentmem/actions)
[![Version](https://img.shields.io/badge/version-2.0.0-blue.svg)](https://github.com/louloulin/agentmem/releases)
[![Discord](https://img.shields.io/discord/agentmem?label=Discord&logo=discord)](https://discord.gg/agentmem)

[文档](https://agentmem.cc) • [示例](examples/) • [更新日志](CHANGELOG.md) • [贡献指南](CONTRIBUTING.md)

</div>

---

## 🎯 概述

**AgentMem** 是一个用 Rust 构建的高性能、企业级记忆管理平台，专为 AI 智能体和 LLM 应用设计。它提供持久化记忆、智能语义搜索和企业级可靠性，采用模块化插件架构。

### 为什么选择 AgentMem？

现代 LLM 应用面临的关键问题，AgentMem 都能解决：

| 问题 | AgentMem 解决方案 |
|------|------------------|
| ❌ 无持久化记忆 | ✅ 跨会话记忆保留 |
| ❌ 上下文窗口限制 | ✅ 智能记忆检索 |
| ❌ API 成本高昂（100万用户月费 $30万） | ✅ 通过选择性检索降低 90% 成本 |
| ❌ 个性化不足 | ✅ 用户特定记忆作用域 |
| ❌ 缺乏企业级功能 | ✅ RBAC、审计日志、多租户 |

---

## ✨ 核心特性

### 🚀 高性能

- **216,000 次/秒** 插件吞吐量
- **<100ms** 语义搜索延迟（P95）
- **93,000倍** 缓存加速比
- **5,000 次/秒** 记忆添加吞吐量
- 异步、无锁架构

### 🧠 智能记忆

- **自动事实提取** - 由 LLM 驱动
- **5 种搜索引擎**: 向量、BM25、全文、模糊、混合（RRF）
- **冲突解决** - 处理矛盾信息
- **记忆重要性评分** 和衰减
- **基于图的推理** - 关系遍历

### 🔌 可扩展架构

- **WASM 插件系统** - 支持热重载
- **18 个模块化 crate** - 清晰的职责分离
- **20+ LLM 提供商**: OpenAI、Anthropic、DeepSeek、Google、Azure 等
- **多后端存储**: LibSQL、PostgreSQL、Pinecone、LanceDB、Qdrant
- **多语言绑定**: Python、JavaScript、Go、仓颉

### 🛡️ 企业级

- **RBAC**（基于角色的访问控制）- 细粒度权限
- **JWT 和会话认证**
- **完整的审计日志**
- **全链路可观测性**: Prometheus、OpenTelemetry、Grafana
- **多模态支持**: 文本、图像、音频、视频
- **Kubernetes 就绪** - 提供 Helm 图表
- **99.9% 可用性 SLA** 能力

---

## 🚀 快速开始

### 安装

#### 使用 Cargo

在 `Cargo.toml` 中添加：

```toml
[dependencies]
agent-mem = "2.0"
tokio = { version = "1", features = ["full"] }
```

#### 使用 Docker

```bash
docker pull agentmem/server:latest
docker run -p 8080:8080 agentmem/server:latest
```

#### 从源码构建

```bash
git clone https://github.com/louloulin/agentmem.git
cd agentmem
cargo build --release
```

### 基础用法

```rust
use agent_mem::Memory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 零配置初始化
    let memory = Memory::new().await?;

    // 添加记忆（自动事实提取）
    memory.add("我喜欢披萨").await?;
    memory.add("我住在旧金山").await?;
    memory.add("我最喜欢的食物是披萨").await?; // 自动去重

    // 语义搜索
    let results = memory.search("你了解我什么？").await?;
    for result in results {
        println!("- {} (分数: {:.2})", result.memory, result.score);
    }

    Ok(())
}
```

### 运行服务器

```bash
# 启动全栈服务器（API + UI）
cargo run --bin agent-mem-server

# 或使用 Docker Compose
docker-compose up -d
```

**访问地址:**
- 🌐 **API**: `http://localhost:8080`
- 🖥️ **Web UI**: `http://localhost:3001`
- 📚 **API 文档**: `http://localhost:8080/swagger-ui/`

---

## 📊 性能基准

| 操作 | 吞吐量 | 延迟 (P50) | 延迟 (P99) |
|------|--------|-----------|-----------|
| 添加记忆 | 5,000 次/秒 | 20ms | 50ms |
| 向量搜索 | 10,000 次/秒 | 10ms | 30ms |
| BM25 搜索 | 15,000 次/秒 | 5ms | 15ms |
| 插件调用 | 216,000 次/秒 | 1ms | 5ms |
| 批量操作 | 50,000 次/秒 | 100ms | 300ms |
| 图遍历 | 1,000 查询/秒 | 50ms | 200ms |

*基准测试环境: Apple M2 Pro, 32GB RAM, LibSQL 后端*

---

## 🏗️ 架构

AgentMem 由 **18 个专业化的 crate** 组成，职责清晰：

```
agentmem/
├── agent-mem-traits          # 核心抽象和 trait
├── agent-mem-core            # 记忆管理引擎（32K 行）
├── agent-mem                 # 统一高级 API
├── agent-mem-llm             # 20+ LLM 提供商集成
├── agent-mem-embeddings      # 嵌入模型（FastEmbed, ONNX）
├── agent-mem-storage         # 多后端存储层
├── agent-mem-intelligence    # AI 推理引擎（DeepSeek 等）
├── agent-mem-plugin-sdk     # WASM 插件 SDK
├── agent-mem-plugins         # 插件管理器（支持热重载）
├── agent-mem-server          # HTTP REST API（175+ 端点）
├── agent-mem-client          # HTTP 客户端库
├── agent-mem-compat          # Mem0 兼容层
├── agent-mem-observability   # 监控和指标
├── agent-mem-performance     # 性能优化
├── agent-mem-deployment      # Kubernetes 部署
├── agent-mem-distributed     # 分布式支持
└── agent-mem-python          # Python 绑定（PyO3）
```

**总计**: 275,000+ 行生产级 Rust 代码

---

## 🔌 插件系统

AgentMem 提供高性能 WASM 插件系统，支持沙箱隔离：

```rust
use agent_mem_plugins::PluginManager;

// 创建插件管理器（带 LRU 缓存）
let plugin_manager = PluginManager::new(100);

// 注册插件（支持热重载）
plugin_manager.register(weather_plugin).await?;

// 在隔离沙箱中执行插件
let result = plugin_manager.execute("weather", &input).await?;
```

**插件特性:**
- 🔒 **沙箱隔离** - WebAssembly 安全性
- ⚡ **LRU 缓存** - 缓存调用加速 93,000 倍
- 🔄 **热重载** - 无需重启即可加载/卸载
- 🎛️ **能力系统** - 细粒度权限控制
- 📊 **性能监控** - 内置指标

---

## 🔌 Model Context Protocol (MCP) 集成

AgentMem 提供完整的 **Model Context Protocol (MCP)** 服务器实现，可与 Claude Code、Claude Desktop 和其他 MCP 兼容客户端无缝集成。

### MCP 特性

- ✅ **5 个核心工具**: 记忆管理、搜索、对话、系统提示词和 Agent 列表
- ✅ **多种传输方式**: stdio、HTTP、SSE（服务器发送事件）
- ✅ **资源管理**: 动态资源发现和订阅
- ✅ **提示词模板**: 支持变量的可重用提示词模板
- ✅ **身份认证**: JWT 和 API 密钥支持
- ✅ **生产就绪**: 已通过 Claude Code 集成实战测试

### 与 Claude Code 快速开始

```bash
# 1. 编译 MCP 服务器
cargo build --package mcp-stdio-server --release

# 2. 在项目根目录创建 .mcp.json
cat > .mcp.json << EOF
{
  "mcpServers": {
    "agentmem": {
      "command": "./target/release/agentmem-mcp-server",
      "args": [],
      "env": {
        "AGENTMEM_API_URL": "http://127.0.0.1:8080",
        "RUST_LOG": "info"
      }
    }
  }
}
EOF

# 3. 在项目目录中启动 Claude Code
claude
```

### 可用的 MCP 工具

| 工具 | 描述 | 参数 |
|------|------|------|
| `agentmem_add_memory` | 向系统添加新记忆 | `content`、`user_id`、`agent_id`（可选）、`memory_type`（可选） |
| `agentmem_search_memories` | 语义搜索记忆 | `query`、`user_id`、`limit`（可选）、`search_type`（可选） |
| `agentmem_chat` | 带记忆上下文的智能对话 | `message`、`user_id`、`agent_id`（可选） |
| `agentmem_get_system_prompt` | 获取个性化系统提示词 | `user_id`、`agent_id`（可选） |
| `agentmem_list_agents` | 列出所有可用 Agent | 无 |

### 在 Claude Code 中的使用示例

```
用户: 记住我偏好深色模式，并且使用 Rust 进行后端开发

Claude: [调用 agentmem_add_memory]
✅ 记忆保存成功

用户: 你了解我的哪些偏好？

Claude: [调用 agentmem_search_memories]
根据您保存的记忆：
- 您偏好深色模式
- 您使用 Rust 进行后端开发
```

### 文档

- 📖 [MCP 完整指南](docs/api/mcp-complete-guide.md) - 完整集成指南
- 🚀 [Claude Code 快速开始](docs/getting-started/claude-code-quickstart.md) - 5 分钟设置
- 🔧 [MCP 命令参考](docs/api/mcp-commands.md) - 所有可用命令
- 🖥️ [Claude Desktop 集成](examples/mcp-stdio-server/CLAUDE_DESKTOP_INTEGRATION.md) - 桌面应用设置

---

## 🌐 语言绑定

AgentMem 为多种语言提供官方 SDK：

### Python

```python
from agentmem import Memory

memory = Memory()
memory.add("用户偏好深色模式")
results = memory.search("用户偏好")
```

**安装**: `pip install agentmem`

### JavaScript/TypeScript

```typescript
import { Memory } from 'agentmem';

const memory = new Memory();
await memory.add("用户偏好深色模式");
const results = await memory.search("用户偏好");
```

**安装**: `npm install agentmem`

### Go

```go
import "github.com/agentmem/agentmem-go"

memory := agentmem.NewMemory()
memory.Add("用户偏好深色模式")
results := memory.Search("用户偏好")
```

### 仓颉

```cangjie
import agentmem.*

let memory = Memory.create()
memory.add("用户偏好深色模式")
let results = memory.search("用户偏好")
```

**查看**: [SDK 文档](sdks/)

---

## 📚 文档

**📖 [完整文档索引](docs/README.md)** - 所有文档的中心枢纽

### 快速开始

- 📖 [安装指南](INSTALL.md) - 详细设置说明
- 🚀 [快速开始指南](QUICKSTART.md) - 5 分钟上手
- 📝 [API 参考](docs/api/API_REFERENCE.md) - 完整 API 文档
- 💬 [Claude Code 集成](docs/getting-started/claude-code-quickstart.md) - MCP 集成指南

### 用户指南

- 📚 [用户指南](docs/user-guide/getting-started.md) - 全面的用户文档
- 🔍 [搜索指南](docs/getting-started/search-quickstart.md) - 搜索引擎使用
- 🔌 [插件指南](docs/getting-started/plugins-quickstart.md) - 插件开发
- 🔗 [MCP 完整指南](docs/api/mcp-complete-guide.md) - 完整 MCP 集成文档

### 开发者资源

- 🏗️ [架构文档](docs/architecture/architecture-overview.md) - 系统架构和设计
- 🔧 [开发者指南](docs/developer-guide/architecture.md) - 开发设置和指南
- 🚀 [部署指南](docs/deployment/PRODUCTION_DEPLOYMENT_GUIDE.md) - 生产部署策略
- 🧪 [测试指南](docs/testing/) - 测试策略和最佳实践
- 🔒 [安全文档](docs/SECURITY.md) - 安全模型和最佳实践

### API 和集成

- 📝 [API 参考](docs/api/API_REFERENCE.md) - 完整 REST API 文档
- 🔌 [MCP 工具参考](docs/api/mcp-tools-reference.md) - Model Context Protocol 工具
- 📋 [OpenAPI 规范](docs/api/openapi.yaml) - 机器可读的 API 规范

---

## 💡 使用场景

### 1. AI 聊天机器人

为对话式 AI 提供持久化记忆：

```rust
memory.add("user123", "偏好深色模式").await?;
let context = memory.search("用户偏好", "user123").await?;
```

### 2. 知识管理

构建企业知识库：

```rust
memory.add("company_kb", "休假政策：每年 20 天").await?;
let results = memory.search("休假政策", "company_kb").await?;
```

### 3. 多智能体系统

协调多个 AI 智能体，共享记忆：

```rust
let scope = MemoryScope::Agent {
    user_id: "alice",
    agent_id: "coding-assistant"
};
memory.add_with_scope("偏好 Rust", scope).await?;
```

### 4. Mem0 迁移

Mem0 的即插即用替代方案：

```rust
use agent_mem_compat::Mem0Client;

let client = Mem0Client::new().await?;
let id = client.add("user", "content", None).await?;
```

---

## 🤝 贡献

我们欢迎贡献！请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解指南。

**贡献方式:**
- 🐛 Bug 修复和报告
- 💡 功能请求
- 📝 文档改进
- 🧪 测试用例
- 🔧 性能优化
- 🌍 国际化

### 开发设置

```bash
# 克隆仓库
git clone https://github.com/louloulin/agentmem.git
cd agentmem

# 构建工作区
cargo build --workspace

# 运行测试
cargo test --workspace

# 运行代码检查
cargo clippy --workspace -- -D warnings

# 格式化代码
cargo fmt --all
```

---

## 📈 路线图

### 当前版本 (2.0.0)

- ✅ 核心记忆管理
- ✅ 5 种搜索引擎
- ✅ WASM 插件系统
- ✅ 多后端存储
- ✅ 企业级功能（RBAC、审计日志）
- ✅ 语言绑定（Python、JS、Go、仓颉）

### 即将推出 (2.1.0)

- 🔜 代码原生记忆（AST 解析）
- 🔜 GitHub 集成
- 🔜 Claude Code 深度集成
- 🔜 高级上下文管理
- 🔜 性能优化

**查看**: [路线图](AGENTMEM_2.1%20ROADMAP.md)

---

## 🏆 生产就绪

AgentMem 经过实战测试，生产就绪：

- ✅ **99.9% 可用性** 能力
- ✅ **水平扩展** 支持
- ✅ **多区域部署** 就绪
- ✅ **灾难恢复** - 备份/恢复
- ✅ **安全审计** 和漏洞扫描
- ✅ **全面监控** 和告警

---

## 📄 许可证

采用双重许可：
- **MIT 许可证** - 查看 [LICENSE-MIT](LICENSE-MIT)
- **Apache-2.0 许可证** - 查看 [LICENSE-APACHE](LICENSE-APACHE)

---

## 🙏 致谢

基于优秀的开源项目构建：

- [Rust](https://www.rust-lang.org/) - 核心语言
- [Tokio](https://tokio.rs/) - 异步运行时
- [Extism](https://extism.org/) - WASM 插件框架
- [DeepSeek](https://www.deepseek.com/) - AI 推理
- [LanceDB](https://lancedb.github.io/lancedb/) - 向量数据库
- [LibSQL](https://libsql.org/) - 嵌入式 SQL 数据库

---

## 🌟 Star 历史



---

<div align="center">

**AgentMem** - 为您的 AI 提供应有的记忆。🧠✨

[GitHub](https://github.com/louloulin/agentmem) ·
[文档](https://agentmem.cc) ·
[示例](examples/) ·
[Discord](https://discord.gg/agentmem) ·
[博客](https://blog.agentmem.dev) ·
[English](README.md)

由 AgentMem 团队用 ❤️ 打造

</div>

