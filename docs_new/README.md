# AgentMem

<div align="center">

**让 AI 拥有持久记忆**

极简易用的 AI Agent 记忆管理系统，为你的 AI 应用提供强大的记忆能力

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![Python](https://img.shields.io/badge/python-3.9+-blue.svg)](https://www.python.org)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Documentation](https://img.shields.io/badge/docs-latest-green.svg)](https://docs.agentmem.dev)

[快速开始](#快速开始) • [功能特性](#核心特性) • [文档](https://docs.agentmem.dev) • [示例](#示例) • [社区](#社区)

</div>

---

## 简介

AgentMem 是一个企业级的 AI Agent 记忆管理系统，让你的 AI 应用能够：

- 记住用户偏好和历史对话
- 语义搜索相关信息
- 自动提取和去重记忆
- 支持多模态内容（文本、图像、音频）
- 灵活的存储后端（本地文件、数据库、云端）

### 为什么选择 AgentMem?

**极简易用** - 零配置启动，一行代码集成

**智能功能** - 自动事实提取、智能去重、冲突解决

**高性能** - Rust 实现，毫秒级响应

**灵活扩展** - 插件系统、多语言支持

## 快速开始

### Rust (5 分钟)

```bash
# 1. 添加依赖
cargo add agentmem

# 2. 设置环境变量（可选）
export OPENAI_API_KEY=sk-...
```

```rust
use agentmem::Memory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 创建实例（零配置）
    let memory = Memory::new().await?;

    // 2. 添加记忆
    memory.add("用户喜欢喝咖啡").await?;

    // 3. 搜索记忆
    let results = memory.search("饮品偏好").await?;

    // 4. 打印结果
    for (i, mem) in results.iter().enumerate() {
        println!("{}. {}", i + 1, mem.content);
    }

    Ok(())
}
```

### Python (5 分钟)

```bash
# 1. 安装
pip install agentmem

# 2. 设置环境变量（可选）
export OPENAI_API_KEY=sk-...
```

```python
from agentmem import Memory

# 1. 创建实例（零配置）
memory = Memory.new()

# 2. 添加记忆
memory.add("用户喜欢喝咖啡")

# 3. 搜索记忆
results = memory.search("饮品偏好")

# 4. 打印结果
for i, mem in enumerate(results):
    print(f"{i+1}. {mem.content}")
```

## 核心特性

### 1. 零配置启动

```rust
// 自动检测环境变量，使用智能默认配置
let memory = Memory::new().await?;
```

### 2. 智能记忆管理

```rust
// 自动事实提取
memory.add("张三，30岁，住在上海").await?;

// 智能去重（自动识别重复内容）
memory.add("张三住在上海市").await?;  // 会被识别为重复

// 冲突解决（智能处理矛盾信息）
memory.add("张三今年31岁").await?;  // 自动更新年龄信息
```

### 3. 语义搜索

```rust
// 基于向量相似度的智能搜索
let results = memory.search("用户的年龄和住址").await?;

// 结果按相关性排序
for result in results {
    println!("{} (相似度: {:.2})", result.content, result.score);
}
```

### 4. 多模态支持

```rust
// 支持文本、图像、音频等多种内容类型
memory.add(
    Memory::builder()
        .content("用户上传了一张照片")
        .metadata_type("image")
        .metadata_url("https://example.com/photo.jpg")
).await?;
```

### 5. 灵活配置

```rust
// Builder 模式，按需配置
let memory = Memory::builder()
    .with_storage("libsql://./data.db")
    .with_llm("openai", "gpt-4")
    .with_embedder("openai", "text-embedding-3-small")
    .enable_intelligent_features()
    .build()
    .await?;
```

## 常见用例

### AI 聊天机器人

```rust
// 记住对话历史，提供连贯的对话体验
memory.add(
    "用户询问了关于产品的价格信息",
    Some(session_id),
    MemoryType::EPISODIC
).await?;

// 搜索相关历史记录
let context = memory.search("产品价格").await?;
```

### 个性化推荐

```rust
// 记住用户偏好
memory.add(
    "用户喜欢科幻电影，尤其是《星际穿越》",
    None,
    MemoryType::SEMANTIC
).await?;

// 基于偏好推荐
let preferences = memory.search("电影偏好").await?;
```

### 知识库管理

```rust
// 构建企业知识库
memory.add(
    "公司成立于2020年，总部位于上海",
    None,
    MemoryType::SEMANTIC
).await?;

// 语义搜索知识库
let knowledge = memory.search("公司信息").await?;
```

### 多 Agent 协作

```rust
// Agent 1: 数据收集
let agent1 = Memory::builder()
    .agent_id("data_collector")
    .build()
    .await?;

agent1.add("用户提供了新的需求文档").await?;

// Agent 2: 数据处理
let agent2 = Memory::builder()
    .agent_id("data_processor")
    .build()
    .await?;

// Agent 2 可以访问 Agent 1 的数据
let data = agent2.get_all(
    GetAllOptions::builder()
        .agent_id("data_collector")
        .build()
).await?;
```

## 文档

- [快速开始指南](quickstart.md) - 5 分钟上手 AgentMem
- [基础概念](tutorials/basic_concepts.md) - 理解 AgentMem 的核心概念
- [记忆管理](tutorials/memory_management.md) - 添加、更新、删除记忆
- [语义搜索](tutorials/semantic_search.md) - 智能搜索和过滤
- [多模态处理](tutorials/multimodal.md) - 处理图像、音频等多媒体内容
- [插件开发](tutorials/plugins.md) - 扩展 AgentMem 功能
- [生产部署](tutorials/production.md) - 部署到生产环境
- [API 参考](api_reference/) - 完整的 API 文档
- [故障排查](troubleshooting.md) - 常见问题和解决方案

## 示例

### Rust 示例

- [quickstart-zero-config](../examples/quickstart-zero-config) - 零配置快速开始
- [production-memory-demo](../examples/production-memory-demo) - 生产环境示例
- [multimodal-demo](../examples/multimodal-demo) - 多模态处理
- [intelligent-memory-demo](../examples/intelligent-memory-demo) - 智能记忆管理

### Python 示例

```bash
# 克隆仓库
git clone https://github.com/agentmem/agentmem.git
cd agentmem/sdks/python

# 运行示例
python examples/basic.py
python examples/search.py
python examples/multimodal.py
```

## 架构

```
┌─────────────────────────────────────────────────────────┐
│                      AgentMem API                       │
│                   (统一接口层)                           │
└────────────────────┬────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────┐
│              Memory Orchestrator                        │
│              (智能编排层)                                │
│  • 自动路由  • 事务管理  • 性能优化                      │
└────────────────────┬────────────────────────────────────┘
                     │
         ┌───────────┼───────────┐
         │           │           │
┌────────▼───┐ ┌────▼────┐ ┌───▼────────┐
│ CoreAgent  │ │Semantic │ │ Episodic   │
│ (核心记忆) │ │ Agent   │ │ Agent      │
│            │ │ (语义)  │ │ (情节)     │
└────────────┘ └─────────┘ └────────────┘
         │           │           │
┌────────▼───────────▼───────────▼────────────────────┐
│              Storage Layer                          │
│  • LibSQL  • PostgreSQL  • Redis  • S3             │
└─────────────────────────────────────────────────────┘
```

## 性能

| 操作 | 响应时间 | 说明 |
|------|----------|------|
| 添加记忆 | < 10ms | 包含向量嵌入 |
| 搜索记忆 | < 50ms | 语义搜索，前10条结果 |
| 批量添加 | < 100ms | 100条记忆批量写入 |
| 删除记忆 | < 5ms | 单条删除 |

*基于默认配置，Intel i7-12700K 测试结果*

## 社区

- [GitHub Issues](https://github.com/agentmem/agentmem/issues) - 报告问题和功能请求
- [Discussions](https://github.com/agentmem/agentmem/discussions) - 社区讨论
- [Discord](https://discord.gg/agentmem) - 实时聊天（即将推出）

## 贡献

我们欢迎各种形式的贡献！请参阅 [CONTRIBUTING.md](CONTRIBUTING.md) 了解详情。

### 贡献方式

1. 报告 Bug
2. 提出新功能建议
3. 提交代码改进
4. 改进文档
5. 分享使用经验

## 许可证

AgentMem 采用双重许可证：

- MIT License (LICENSE-MIT)
- Apache License 2.0 (LICENSE-APACHE)

你可以选择其中任何一个。

## 致谢

感谢所有为 AgentMem 做出贡献的开发者！

---

**官网**: https://agentmem.dev • **文档**: https://docs.agentmem.dev • **GitHub**: https://github.com/agentmem/agentmem
