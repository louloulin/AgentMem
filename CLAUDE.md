# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

AgentMem 是一个基于 Rust 开发的智能记忆管理平台，为 AI 代理提供先进的记忆处理能力。项目采用模块化架构，包含 13 个核心 crate，支持智能推理引擎、Mem0 兼容层和多数据库后端。

## 常用开发命令

### 构建和测试
```bash
# 构建所有 crates (默认使用 LibSQL)
cargo build --release

# 运行所有测试
cargo test --workspace

# 运行特定测试
cargo test --package agent-mem-core
cargo test --package agent-mem-server

# 代码格式化
cargo fmt

# 代码质量检查
cargo clippy --all-targets --all-features

# 生成文档
cargo doc --no-deps --all-features
```

### 服务器运行
```bash
# 启动服务器 (零配置，自动使用 LibSQL)
cargo run --bin agent-mem-server

# 使用 PostgreSQL 后端
DATABASE_BACKEND=postgres DATABASE_URL=postgresql://user:password@localhost:5432/agentmem cargo run --bin agent-mem-server --features postgres

# 使用配置文件启动
cargo run --bin agent-mem-server -- --config config.toml
```

### 演示程序
```bash
# 智能推理引擎演示
cargo run --bin intelligent-reasoning-demo

# Mem0 兼容性演示
cargo run --bin mem0-demo

# 客户端集成测试
cargo run --bin client-server-integration-test

# 完整功能演示
cargo run --bin complete_demo

# 性能基准测试
cargo run --bin comprehensive-test
```

### 代码质量工具
```bash
# 运行完整的持续改进脚本 (包含所有检查)
./scripts/continuous-improvement.sh

# 仅运行构建脚本
./scripts/build.sh

# 运行性能基准测试
cd tools/performance-benchmark && cargo run --release
```

## 核心架构

### 模块化 Crate 设计
项目采用 13 个专业化 crate 的模块化架构：

**核心层 Crates:**
- `agent-mem-traits` - 核心抽象和接口定义
- `agent-mem-core` - 记忆管理引擎核心
- `agent-mem-llm` - LLM 提供商集成
- `agent-mem-storage` - 存储后端抽象层
- `agent-mem-embeddings` - 嵌入模型集成
- `agent-mem-intelligence` - AI 驱动的智能记忆处理
- `agent-mem-config` - 配置管理系统
- `agent-mem-utils` - 通用工具库

**服务层 Crates:**
- `agent-mem-server` - HTTP API 服务器
- `agent-mem-client` - HTTP 客户端库
- `agent-mem-performance` - 性能监控工具
- `agent-mem-distributed` - 分布式部署支持
- `agent-mem-compat` - Mem0 兼容层

### 分层记忆架构
系统实现了四层记忆组织结构：
```
Global Layer    → 全局共享知识和系统配置
    ↓
Agent Layer     → 代理特定知识和行为模式
    ↓
User Layer      → 用户个人信息和偏好设置
    ↓
Session Layer   → 会话上下文和临时状态
```

### 智能推理引擎
基于 DeepSeek LLM 的智能推理引擎，包含：
- 事实提取器 (FactExtractor)
- 决策引擎 (DecisionEngine) 
- 智能处理器 (IntelligentProcessor)

## 数据库配置

### LibSQL (默认，开发推荐)
零配置启动，自动创建本地数据库文件：
```bash
cargo run --bin agent-mem-server
# 数据库文件自动创建在 ./data/agentmem.db
```

### PostgreSQL (生产推荐)
通过环境变量配置：
```bash
export DATABASE_BACKEND=postgres
export DATABASE_URL=postgresql://user:password@localhost:5432/agentmem
cargo run --bin agent-mem-server --features postgres
```

## API 使用模式

### Mem0 兼容 API
```rust
use agent_mem_compat::Mem0Client;

// 创建客户端
let client = Mem0Client::new().await?;

// 添加记忆
let memory_id = client.add("user123", "我喜欢喝咖啡，特别是拿铁", None).await?;

// 搜索记忆
let results = client.search("饮品偏好", "user123", None).await?;
```

### 智能推理引擎
```rust
use agent_mem_intelligence::{IntelligentMemoryProcessor, Message};

// 创建智能处理器
let processor = IntelligentMemoryProcessor::new(api_key)?;

// 处理消息并提取事实
let result = processor.process_messages(&messages, &[]).await?;
```

## 测试策略

### 测试覆盖范围
- 100+ 单元测试用例覆盖核心功能
- 集成测试验证端到端工作流
- Mem0 兼容性测试 14 个全部通过
- 智能推理和事实提取引擎验证

### 运行特定测试
```bash
# 核心记忆功能测试
cargo test --package agent-mem-core

# 服务器集成测试
cargo test --package agent-mem-server

# 智能推理测试
cargo test --package agent-mem-intelligence

# Mem0 兼容性测试
cargo test --package agent-mem-compat
```

## 环境变量配置

### 常用环境变量
```bash
# 数据库配置
DATABASE_BACKEND=libsql          # 或 postgres
DATABASE_URL=postgresql://...    # PostgreSQL 连接字符串

# LLM API 密钥
DEEPSEEK_API_KEY=your-deepseek-api-key
OPENAI_API_KEY=your-openai-api-key
ANTHROPIC_API_KEY=your-anthropic-api-key

# 服务器配置
RUST_LOG=info                    # 日志级别
SERVER_PORT=8080                 # 服务器端口
```

## 项目结构特点

### 多语言实现
- **Rust**: 核心实现，13 个 crates
- **Cangjie (CJ)**: 函数式实现，位于 `cj/` 目录
- **Python**: Python SDK，位于 `sdks/python/`
- **JavaScript**: Node.js SDK，位于 `sdks/javascript/`
- **Go**: Go SDK，位于 `sdks/go/`

### 示例程序
项目包含 50+ 个演示程序，展示不同功能：
- 基础记忆操作 (`simple-memory-demo`, `basic-demo`)
- 智能推理 (`intelligent-reasoning-demo`, `intelligence-demo`)
- 服务器集成 (`server-demo`, `client-demo`)
- Mem0 兼容性 (`mem0-compat-demo`)
- 性能优化 (`performance-demo`, `performance-optimization-demo`)

### 开发工具
- **代码质量分析**: `tools/code-quality-analyzer/`
- **性能基准测试**: `tools/performance-benchmark/`
- **CLI 工具**: `tools/agentmem-cli/`

## 生产部署考虑

### Docker 部署
```bash
# 构建镜像
docker build -t agentmem .

# 运行容器
docker run -p 8080:8080 -e DATABASE_URL=postgresql://... agentmem
```

### 性能优化
- 使用 `--release` 构建获得最佳性能
- 考虑启用 PostgreSQL 用于生产环境
- 配置适当的缓存策略
- 监控内存使用和查询性能

### 安全注意事项
- 所有 API 密钥通过环境变量配置
- 数据库连接使用 SSL
- 定期运行安全审计：`cargo audit`

## 故障排除

### 常见问题
1. **数据库连接失败**: 检查 `DATABASE_URL` 环境变量
2. **LLM API 错误**: 验证 API 密钥配置
3. **编译错误**: 确保使用 Rust 1.75+ 版本
4. **测试失败**: 检查依赖项是否正确安装

### 调试命令
```bash
# 检查编译错误
cargo check

# 查看详细错误信息
RUST_LOG=debug cargo run --bin agent-mem-server

# 运行特定失败的测试
cargo test -- --nocapture test_name
```

## 贡献指南

### 开发流程
1. 创建功能分支
2. 实现功能并添加测试
3. 运行 `./scripts/continuous-improvement.sh` 确保代码质量
4. 提交并创建 PR

### 代码质量标准
- 所有 `cargo clippy` 警告必须解决
- 代码格式必须符合 `cargo fmt` 标准
- 新功能需要包含相应的测试用例
- 公共 API 必须有完整的文档注释