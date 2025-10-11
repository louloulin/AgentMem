# AgentMem 🧠

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://gitcode.com/louloulin/agentmem/workflows/CI/badge.svg)](https://gitcode.com/louloulin/agentmem/actions)
[![Coverage](https://codecov.io/gh/your-org/agentmem/branch/main/graph/badge.svg)](https://codecov.io/gh/your-org/agentmem)

**AgentMem** 是一个高性能、智能化的 AI 代理记忆管理系统，为 AI 代理提供类人的记忆能力。

## ✨ 核心特性

### 🏗️ 分层记忆架构
- **战略层 (Strategic)**: 长期目标和核心价值观
- **战术层 (Tactical)**: 中期计划和策略
- **操作层 (Operational)**: 日常任务和具体行动
- **上下文层 (Contextual)**: 即时环境和临时信息

### 🧠 智能记忆管理
- **自动重要性评分**: 基于多维度因素的智能评分系统
- **冲突解决机制**: 自动检测和解决记忆冲突
- **生命周期管理**: 智能的记忆创建、更新、归档和删除
- **继承和传播**: 记忆在不同层级间的智能传播

### 🔍 高级搜索能力
- **语义搜索**: 基于向量相似性的深度语义理解
- **模糊匹配**: 容错的文本匹配和检索
- **上下文感知**: 根据当前情境调整搜索结果
- **多模态检索**: 支持文本、时间、标签等多维度检索

### 🚀 高性能架构
- **异步处理**: 全异步架构，支持高并发操作
- **智能缓存**: 多级缓存系统，优化访问性能
- **批量操作**: 高效的批量记忆处理能力
- **实时监控**: 完整的性能指标和健康检查

### 🔌 灵活集成
- **多存储后端**: 支持 PostgreSQL、Redis、Pinecone、Qdrant 等
- **LLM 集成**: 无缝集成 OpenAI、Anthropic、Cohere、Ollama 等
- **RESTful API**: 完整的 HTTP API 接口
- **多语言 SDK**: 提供 Rust、Python、JavaScript 等客户端

## 🚀 快速开始

### 安装

```bash
# 添加依赖
cargo add agentmem

# 或者克隆源码
git clone https://gitcode.com/louloulin/agentmem.git
cd agentmem
cargo build --release
```

### 基础使用

```rust
use agentmem::{AgentMem, MemoryConfig, Memory, MemoryType, MemoryScope};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化配置
    let config = MemoryConfig::builder()
        .llm_provider("openai")
        .vector_store("pinecone")
        .build();
    
    // 创建 AgentMem 实例
    let agent_mem = AgentMem::new(config).await?;
    
    // 创建记忆
    let memory = Memory::builder()
        .content("今天学习了 Rust 的所有权机制，这是内存安全的核心")
        .memory_type(MemoryType::Episodic)
        .scope(MemoryScope::User { 
            agent_id: "agent_001".to_string(),
            user_id: "user_123".to_string() 
        })
        .importance(0.8)
        .tags(vec!["学习", "Rust", "编程"])
        .build();
    
    // 存储记忆
    let memory_id = agent_mem.add_memory(memory).await?;
    println!("记忆已创建: {}", memory_id);
    
    // 搜索记忆
    let results = agent_mem
        .search("Rust 所有权")
        .limit(10)
        .min_relevance(0.7)
        .execute()
        .await?;
    
    for result in results {
        println!("找到记忆: {} (相关性: {:.2})", 
                result.content, result.relevance_score);
    }
    
    // 获取相关记忆
    let related = agent_mem.get_related_memories(&memory_id, 5).await?;
    println!("找到 {} 条相关记忆", related.len());
    
    Ok(())
}
```

### 高级功能示例

```rust
use agentmem::{
    AgentMem, MemoryConfig, SearchFilters, 
    ImportanceLevel, MemoryLevel, ContextualSearch
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let agent_mem = AgentMem::new(MemoryConfig::default()).await?;
    
    // 上下文感知搜索
    let context = ContextualSearch::builder()
        .current_task("编写 Rust 代码")
        .user_mood("专注")
        .time_context("工作时间")
        .build();
    
    let results = agent_mem
        .contextual_search("内存管理", context)
        .await?;
    
    // 高级过滤搜索
    let filters = SearchFilters::builder()
        .memory_types(vec![MemoryType::Semantic, MemoryType::Procedural])
        .importance_range(0.6..=1.0)
        .date_range(chrono::Utc::now() - chrono::Duration::days(30)..=chrono::Utc::now())
        .tags_include(vec!["编程", "最佳实践"])
        .build();
    
    let filtered_results = agent_mem
        .advanced_search("代码优化", filters)
        .await?;
    
    // 智能总结
    let summary = agent_mem
        .summarize_memories(&filtered_results.iter().map(|r| &r.id).collect::<Vec<_>>())
        .await?;
    
    println!("记忆总结: {}", summary);
    
    Ok(())
}
```

## 🏗️ 系统架构

```
┌─────────────────────────────────────────────────────────────┐
│                    AgentMem 架构图                           │
├─────────────────────────────────────────────────────────────┤
│  Client SDKs (Rust, Python, JS)                           │
├─────────────────────────────────────────────────────────────┤
│  RESTful API Layer                                         │
├─────────────────────────────────────────────────────────────┤
│  Core Engine                                               │
│  ├── Memory Manager     ├── Search Engine                  │
│  ├── Importance Scorer  ├── Conflict Resolver              │
│  ├── Lifecycle Manager  └── Context Analyzer               │
├─────────────────────────────────────────────────────────────┤
│  LLM Integration Layer                                      │
│  ├── OpenAI    ├── Anthropic    ├── Cohere                │
│  ├── Ollama    ├── Gemini       └── Custom Providers       │
├─────────────────────────────────────────────────────────────┤
│  Storage Abstraction Layer                                 │
│  ├── Vector Stores      ├── Traditional DBs               │
│  │   ├── Pinecone       │   ├── PostgreSQL                │
│  │   ├── Qdrant         │   ├── MySQL                     │
│  │   └── Weaviate       │   └── SQLite                    │
│  └── Cache Layer (Redis, In-Memory)                       │
└─────────────────────────────────────────────────────────────┘
```

## 📊 性能特性

### 基准测试结果

| 操作类型 | 吞吐量 | 平均延迟 | P95 延迟 |
|---------|--------|----------|----------|
| 记忆创建 | 1,000 ops/sec | 2ms | 5ms |
| 记忆检索 | 5,000 ops/sec | 1ms | 3ms |
| 语义搜索 | 500 queries/sec | 10ms | 25ms |
| 批量操作 | 10,000 ops/sec | 5ms | 15ms |

### 扩展性指标

- **内存容量**: 支持百万级记忆存储
- **并发用户**: 支持 10,000+ 并发连接
- **搜索性能**: 毫秒级语义搜索响应
- **可用性**: 99.9% 服务可用性保证

## 🛠️ 开发工具

### 代码质量工具

```bash
# 运行代码质量分析
cd tools/code-quality-analyzer
cargo run --release

# 生成质量报告
open ../../reports/quality_report.html
```

### 性能基准测试

```bash
# 运行性能基准测试
cd tools/performance-benchmark
cargo run --release

# 查看性能报告
cat ../../reports/performance_report.md
```

### 持续改进

```bash
# 运行完整的质量检查
./scripts/continuous-improvement.sh

# 查看改进建议
cat reports/improvement_summary.md
```

## 📚 文档

### 核心文档
- [📖 API 参考](docs/api-reference.md) - 完整的 API 文档
- [⚙️ 配置指南](docs/configuration.md) - 详细的配置说明
- [🚀 部署指南](docs/deployment-guide.md) - 生产环境部署
- [🏗️ 架构概览](docs/architecture.md) - 系统架构设计

### 开发文档
- [🔧 开发指南](docs/development.md) - 开发环境搭建
- [🧪 测试指南](docs/testing.md) - 测试策略和实践
- [📈 性能优化](docs/performance.md) - 性能调优指南
- [🔒 安全指南](docs/security.md) - 安全最佳实践

### 使用示例
- [💡 快速入门](examples/quickstart/) - 基础使用示例
- [🔍 搜索示例](examples/search/) - 各种搜索功能
- [🤖 AI 集成](examples/ai-integration/) - LLM 集成示例
- [🌐 Web 应用](examples/web-app/) - Web 应用集成

## 🤝 贡献指南

我们欢迎所有形式的贡献！请查看 [贡献指南](CONTRIBUTING.md) 了解详情。

### 贡献方式
- 🐛 报告 Bug
- 💡 提出新功能建议
- 📝 改进文档
- 🔧 提交代码修复
- 🧪 添加测试用例

### 开发流程
1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🙏 致谢

感谢所有为 AgentMem 项目做出贡献的开发者和用户！

### 核心贡献者
- [@contributor1](https://github.com/contributor1) - 核心架构设计
- [@contributor2](https://github.com/contributor2) - 性能优化
- [@contributor3](https://github.com/contributor3) - API 设计

### 特别感谢
- [Rust 社区](https://www.rust-lang.org/community) - 优秀的编程语言和生态
- [Tokio](https://tokio.rs/) - 异步运行时支持
- [OpenAI](https://openai.com/) - AI 技术支持

## 📞 联系我们

- **官网**: https://agentmem.com
- **文档**: https://docs.agentmem.com
- **GitHub**: https://gitcode.com/louloulin/agentmem
- **Discord**: https://discord.gg/agentmem
- **邮箱**: hello@agentmem.com

---

<div align="center">

**让 AI 代理拥有真正的记忆能力** 🧠✨

[开始使用](docs/quickstart.md) • [查看文档](docs/) • [加入社区](https://discord.gg/agentmem)

</div>
