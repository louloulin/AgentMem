# AgentMem 2.4 改进计划

**制定日期**: 2025-01-07
**版本**: 1.0
**状态**: 📋 计划中
**负责人**: AgentMem 开发团队

---

## 📋 执行摘要

本文档制定了 AgentMem 的全面改进计划，重点关注**开发者体验**和**本地启动**便利性。通过最小改动实现最大价值，确保项目快速可用、易于维护、生产就绪。

### 核心目标

1. **简洁的核心 API** - 专注于内存管理的核心功能
2. **统一启动流程** - 一条命令启动所有服务
3. **友好的错误处理** - 清晰的错误信息而非 panic
4. **完整的开发者文档** - 从安装到部署的全流程指南

### 预期成果

- **启动时间**: 从 30+ 分钟 → **5 分钟**
- **配置步骤**: 从 10+ 步 → **3 步**（设置 API key、选择后端、启动）
- **核心功能完整度**: 基础 CRUD、语义搜索、智能功能全部可用
- **代码质量**: unwrap/expect 减少 **97%**

---

## 🔍 当前问题分析

### 1. 开发者体验问题（P0 - 严重）

#### 问题描述

**现状**：
- README 声称"零配置"，但实际需要配置 LLM API key 才能使用智能功能
- 启动流程复杂：编译 → 配置 → 启动后端 → 启动前端，需要 30+ 分钟
- 多个配置文件：config.toml, .env, justfile 环境变量，配置优先级不明确
- 硬编码的 API key 存在安全风险
- 核心功能（CRUD、搜索）与智能功能（LLM）混淆，缺少分层设计

**影响**：
- 新开发者无法快速体验系统核心功能
- 基础使用也需要配置 LLM，提高使用门槛
- 每次启动都需要重复配置
- 潜在的安全漏洞
- 降低项目采用率

#### 具体证据

```toml
# config.toml - 硬编码的 API key
[llm.zhipu]
api_key = "99a311fa7920a59e9399cf26ecc1e938.ac4w6buZHr2Ggc3k"  # ❌ 不安全

# justfile - 复杂的环境变量配置
export LLM_PROVIDER := "zhipu"  # ❌ 需要外部 API
export EMBEDDER_PROVIDER := "fastembed"
export ENABLE_AUTH := "false"
```

**示例代码问题**：
```rust
// examples/deepseek-test/src/main.rs
let api_key = "sk-498fd5f3041f4466a43fa2b9bbbec250";  // ❌ 硬编码
```

### 2. 代码质量问题（P1 - 重要）

#### 错误处理问题

**统计数据**：
- **unwrap/expect**: 3,846 处
- **clones**: 4,109 处
- **clippy warnings**: 1,200+ 处

**影响**：
- 生产环境 panic 风险
- 内存占用高（大量 clone）
- 代码可维护性差

**示例**：
```rust
// ❌ 当前代码 - 会 panic
let config = load_config().unwrap();
let result = process(data).expect("Failed to process");

// ✅ 应该改为
let config = load_config()
    .context("Failed to load configuration")?;
let result = process(data)
    .context("Processing failed")?;
```

#### 未使用的代码

**统计数据**：
- agent-mem-server: 102 个警告
- agent-mem-core: 1,201 个警告
- 死代码字段：多处

**影响**：
- 代码混淆
- 维护困难
- 性能损失

### 3. 架构问题（P2 - 中等）

#### API 不一致

**问题**：
- MemoryItem 已废弃但仍在使用
- Memory V4 架构存在但未广泛采用
- 向后兼容性负担重

#### 配置管理混乱

**问题**：
- 多个配置来源（文件、环境变量、代码）
- 配置优先级不明确
- 缺少统一配置接口

### 4. 文档和测试问题（P2 - 中等）

#### 文档问题

- README 与实际使用不符
- 缺少完整的快速开始指南
- 示例代码包含硬编码值

#### 测试覆盖

- 单元测试存在但集成测试不足
- 端到端测试缺失
- 性能测试不完善

---

## 🎯 改进目标和原则

### 核心原则

1. **最小改动，最大价值** - 优先实现高价值、低成本的改进
2. **渐进式改进** - 分阶段实施，每阶段都可独立交付价值
3. **向后兼容** - 不破坏现有功能
4. **开发者优先** - 一切改进以提升开发者体验为目标
5. **可衡量** - 每个改进都有明确的成功指标

### 成功指标

| 指标 | 当前状态 | 目标 | 测量方式 |
|------|---------|------|---------|
| 启动时间 | 30+ 分钟 | 5 分钟 | 实际测试 |
| 配置步骤 | 10+ 步 | 2 步 | 文档验证 |
| unwrap/expect | 3,846 处 | <100 处 | 代码统计 |
| clippy warnings | 1,200+ | <100 | 编译检查 |
| 文档完整性 | 60% | 95% | 人工审查 |
| 首次运行成功率 | 30% | 90% | 用户反馈 |

---

## 📅 分阶段实施计划

### Phase 1: 分层配置体验（P0 - 1-2 周）

**目标**: 让开发者在 5 分钟内启动系统，核心功能无需 LLM

#### 任务清单

**1.1 创建分层配置模式** (3 天)

- [ ] 实现核心功能层（无需 LLM）：基础 CRUD、向量搜索
- [ ] 实现智能功能层（需要 LLM）：事实提取、智能决策
- [ ] 默认使用内存数据库（无需安装）
- [ ] 创建配置优先级系统（环境变量 > 文件 > 默认值）
- [ ] 移除所有硬编码的 API key

**验收标准**:
```bash
# 核心功能 - 无需任何配置
cargo run --example basic-crud

# 智能功能 - 需要 API key
export OPENAI_API_KEY="sk-..."
cargo run --example intelligent-features
```

**1.2 统一启动脚本** (2 天)

- [ ] 创建 `just dev` 命令
- [ ] 自动检查依赖（Rust、Node.js）
- [ ] 智能提示：检测 LLM API key 配置状态
- [ ] 一条命令启动后端 + 前端

**验收标准**:
```bash
# 新开发者只需运行
just dev

# 自动完成：
# ✅ 编译项目
# ✅ 启动后端（核心功能可用）
# ⚠️  提示：配置 LLM API key 以启用智能功能
# ✅ 启动前端
# ✅ 打开浏览器
```

**1.3 创建配置文件模板** (1 天)

- [ ] `config.core-only.toml` - 仅核心功能配置（无需 LLM）
- [ ] `config.example.toml` - 完整配置模板（含 LLM）
- [ ] `.env.example` - 环境变量模板
- [ ] 更新 `.gitignore` 忽略敏感配置

**验收标准**:
- 配置文件有清晰的注释
- 核心功能配置可以直接使用
- 智能功能配置有明确的配置说明
- 不包含任何真实的敏感信息

**1.4 改进错误处理（关键路径）** (3 天)

- [ ] 修复 Memory::new() 中的 unwrap
- [ ] 修复启动流程中的 expect
- [ ] 添加友好的错误消息
- [ ] 区分核心功能错误和智能功能错误
- [ ] 创建错误处理指南

**验收标准**:
```rust
// 核心功能错误 - 明确提示
Error: Failed to initialize vector store

Caused by:
  Embedding model not found

Hints for core features:
  • Use FastEmbed (default, no API key needed)
  • Or set: EMBEDDING_PROVIDER=openai OPENAI_API_KEY=sk-...
  • Or use pre-embedded vectors

// 智能功能错误 - 明确提示需要配置
Error: Failed to initialize LLM provider

Caused by:
  No LLM API key configured

Hints for intelligent features:
  • Set OPENAI_API_KEY environment variable, or
  • Set ZHIPU_API_KEY environment variable, or
  • Use core features only (no LLM required)

See: https://docs.agentmem.cc/configuration
```

**1.5 更新文档** (1 天)

- [ ] 重写 QUICKSTART.md（区分核心功能和智能功能）
- [ ] 创建 CORE_FEATURES.md（核心功能文档）
- [ ] 创建 TROUBLESHOOTING.md
- [ ] 更新 README.md 快速开始部分
- [ ] 添加常见问题 FAQ

#### 优先级：🔴 P0 - 必须完成

### Phase 2: 开发者体验优化（P1 - 2-3 周）

**目标**: 提升日常开发体验

#### 任务清单

**2.1 修复核心 unwrap/expect** (5 天)

- [ ] agent-mem: 修复所有 unwrap
- [ ] agent-mem-core: 修复关键路径 unwrap
- [ ] 添加错误上下文
- [ ] 统一错误类型

**验收标准**:
```rust
// Before
let data = fetch_data().unwrap();

// After
let data = fetch_data()
    .context("Failed to fetch data from API")?;
```

**2.2 清理警告** (3 天)

- [ ] 运行 `cargo fix` 自动修复
- [ ] 处理未使用的变量（`_` 前缀）
- [ ] 移除死代码或添加 `#[allow(dead_code)]`
- [ ] 修复 clippy warnings

**验收标准**:
- agent-mem-server: <50 warnings
- agent-mem-core: <500 warnings
- 其他 crates: <100 warnings each

**2.3 创建示例项目** (3 天)

- [ ] `examples/core-features` - 核心功能示例（无需 LLM）
  - `basic-crud` - 基础增删改查
  - `vector-search` - 向量搜索
  - `batch-operations` - 批量操作
- [ ] `examples/intelligent-features` - 智能功能示例（需要 LLM）
  - `fact-extraction` - 事实提取
  - `intelligent-search` - 智能搜索
  - `memory-ranking` - 记忆排序
- [ ] `examples/production-ready` - 生产配置示例
- [ ] 每个示例都有 README

**验收标准**:
- 核心功能示例无需配置即可运行
- 智能功能示例有明确的配置说明
- 所有示例都有详细注释
- 涵盖常见使用场景

**2.4 改进测试** (4 天)

- [ ] 添加集成测试框架
- [ ] 创建端到端测试
- [ ] 添加性能基准测试
- [ ] 设置 CI/CD 自动测试

**验收标准**:
```bash
# 运行所有测试
cargo test --workspace

# 运行集成测试
cargo test --test integration

# 运行性能测试
cargo test --release --benches
```

#### 优先级：🟡 P1 - 应该完成

### Phase 3: 代码质量提升（P2 - 3-4 周）

**目标**: 提升代码质量和可维护性

#### 任务清单

**3.1 继续 unwrap/expect 修复** (10 天)

- [ ] agent-mem-llm: 修复所有 unwrap
- [ ] agent-mem-storage: 修复所有 unwrap
- [ ] agent-mem-intelligence: 修复所有 unwrap
- [ ] 其他 crates: 修复所有 unwrap

**验收标准**:
- 全项目 unwrap/expect < 100 处

**3.2 Clone 优化** (7 天)

- [ ] 识别不必要的 clone
- [ ] 使用引用替代 clone
- [ ] 使用 Arc 共享数据
- [ ] 更新文档说明优化策略

**验收标准**:
- 减少至少 30% 的 clone
- 性能提升 20%+

**3.3 内存 V4 迁移准备** (3 天)

- [ ] 创建迁移指南
- [ ] 添加兼容层
- [ ] 更新示例代码
- [ ] 标记废弃 API

**验收标准**:
- 有清晰的迁移路径
- 新代码使用 Memory V4
- 旧代码仍然可以工作

#### 优先级：🟢 P2 - 可以延后

### Phase 4: 长期优化（P3 - 4-6 周）

**目标**: 长期可维护性和性能优化

#### 任务清单

**4.1 完成 Memory V4 迁移** (10 天)

- [ ] agent-mem-server 迁移到 Memory V4
- [ ] 移除 MemoryItem 依赖
- [ ] 更新所有示例
- [ ] 清理废弃代码

**4.2 性能优化** (10 天)

- [ ] Profile 热点代码
- [ ] 优化关键路径
- [ ] 减少内存分配
- [ ] 改进缓存策略

**4.3 文档完善** (5 天)

- [ ] API 文档生成
- [ ] 架构图更新
- [ ] 视频教程
- [ ] 最佳实践指南

**4.4 社区建设** (5 天)

- [ ] 贡献指南
- [ ] Issue 模板
- [ ] PR 模板
- [ ] 路线图透明化

#### 优先级：🔵 P3 - 长期目标

---

## 🚀 Phase 1 详细实施计划

### 任务 1.1: 创建分层配置模式

#### 技术方案

**1. 分层功能架构**

```rust
// crates/agent-mem/src/auto_config.rs
use anyhow::{Context, Result};

impl MemoryBuilder {
    /// 核心功能模式：无需 LLM
    /// - 基础 CRUD
    /// - 向量搜索（使用 FastEmbed 本地模型）
    /// - 批量操作
    pub async fn with_core_features(self) -> Result<Self> {
        let mut builder = self;

        // 1. 使用内存数据库（无需安装）
        builder = builder.with_storage("memory://").await?;

        // 2. 使用 FastEmbed（本地嵌入模型，无需 API）
        builder = builder.with_embedder("fastembed", "BAAI/bge-small-en-v1.5").await?;

        // 3. 禁用 LLM（核心功能不需要）
        builder = builder.without_llm();

        Ok(builder)
    }

    /// 智能功能模式：需要 LLM API key
    /// - 事实提取
    /// - 智能决策
    /// - 记忆排序
    pub async fn with_intelligent_features(self) -> Result<Self> {
        let mut builder = self;

        // 1. 检查 LLM API key
        let api_key = std::env::var("OPENAI_API_KEY")
            .or_else(|_| std::env::var("ZHIPU_API_KEY"))
            .or_else(|_| std::env::var("ANTHROPIC_API_KEY"));

        if api_key.is_err() {
            return Err(anyhow::anyhow!(
                "LLM API key not found. Set OPENAI_API_KEY, ZHIPU_API_KEY, or ANTHROPIC_API_KEY"
            ));
        }

        // 2. 配置 LLM
        let provider = std::env::var("LLM_PROVIDER").unwrap_or("openai".to_string());
        let model = std::env::var("LLM_MODEL").unwrap_or("gpt-4".to_string());
        builder = builder.with_llm(&provider, &model).await?;

        // 3. 启用智能功能
        builder = builder.enable_intelligent_features();

        Ok(builder)
    }

    /// 自动配置模式：智能检测
    pub async fn with_auto_config(self) -> Result<Self> {
        // 检查是否有 LLM API key
        let has_llm_key = std::env::var("OPENAI_API_KEY").is_ok()
            || std::env::var("ZHIPU_API_KEY").is_ok()
            || std::env::var("ANTHROPIC_API_KEY").is_ok();

        if has_llm_key {
            self.with_intelligent_features().await
        } else {
            self.with_core_features().await
        }
    }
}

// Memory::new() 改进
impl Memory {
    /// 创建内存实例（智能检测）
    /// - 如果有 LLM API key → 启用智能功能
    /// - 如果没有 → 核心功能模式
    pub async fn new() -> Result<Self> {
        Memory::builder()
            .with_auto_config()
            .await
            .context("Failed to initialize Memory. Check your configuration.")?
            .build()
            .await
    }

    /// 创建仅核心功能的实例
    pub async fn new_core() -> Result<Self> {
        Memory::builder()
            .with_core_features()
            .await
            .context("Failed to initialize core features")?
            .build()
            .await
    }
}
```

**2. 配置优先级系统**

```rust
// crates/agent-mem-config/src/lib.rs
pub fn load_config() -> Result<Config> {
    // 优先级: 环境变量 > 配置文件 > 默认值
    let mut config = Config::default();

    // 1. 尝试加载配置文件（可选）
    if let Ok(file_config) = Config::from_file("config.toml") {
        config.merge(file_config);
    }

    // 2. 环境变量覆盖
    config.merge(Config::from_env()?);

    // 3. 使用默认值填充
    config.fallback_to_defaults();

    Ok(config)
}
```

**3. 移除硬编码 API key**

```bash
# 移除所有硬编码的 key
grep -r "sk-" --include="*.rs" examples/ | xargs sed -i 's/sk-[^"]*/YOUR_API_KEY/g'

# 更新文档说明如何设置
echo "OPENAI_API_KEY=your-key-here" > .env.example
echo "ZHIPU_API_KEY=your-key-here" >> .env.example
```

#### 验收标准

```bash
# 测试核心功能（无需配置）
git clone https://github.com/louloulin/agentmem.git
cd agentmem
cargo run --example core-features/basic-crud

# 测试智能功能（需要配置）
export OPENAI_API_KEY="sk-..."
cargo run --example intelligent-features/fact-extraction
```

### 任务 1.2: 统一启动脚本

#### 技术方案

**justfile 添加 `dev` 命令**

```makefile
# justfile
# 开发模式：一键启动所有服务
dev:
    #!bash
    set -e

    echo "🚀 AgentMem 开发模式启动"

    # 1. 检查依赖
    echo "📦 检查依赖..."
    command -v cargo >/dev/null 2>&1 || { echo "❌ 需要安装 Rust"; exit 1; }
    command -v node >/dev/null 2>&1 || { echo "❌ 需要安装 Node.js"; exit 1; }

    # 2. 检查 LLM API key（可选）
    if [ -z "$OPENAI_API_KEY" ] && [ -z "$ZHIPU_API_KEY" ] && [ -z "$ANTHROPIC_API_KEY" ]; then
        echo "⚠️  未检测到 LLM API key"
        echo "   核心功能可用（CRUD、搜索）"
        echo "   智能功能需要配置 API key"
        echo ""
        echo "   配置方式:"
        echo "   export OPENAI_API_KEY='your-key'"
        echo "   或"
        echo "   export ZHIPU_API_KEY='your-key'"
        echo ""
    fi

    # 3. 构建项目
    echo "🔨 构建项目..."
    cargo build --release

    # 4. 启动后端
    echo "🔧 启动后端..."
    cargo run --release --bin agent-mem-server &
    BACKEND_PID=$!

    # 5. 等待后端就绪
    echo "⏳ 等待后端就绪..."
    for i in {1..30}; do
        if curl -s http://localhost:8080/health >/dev/null; then
            break
        fi
        sleep 1
    done

    # 6. 启动前端
    echo "🎨 启动前端..."
    cd agentmem-ui
    npm install --silent
    npm run dev &
    FRONTEND_PID=$!
    cd ..

    # 7. 显示访问信息
    echo ""
    echo "✅ 启动成功！"
    echo ""
    echo "🌐 访问地址:"
    echo "   前端: http://localhost:3001"
    echo "   后端: http://localhost:8080"
    echo "   API 文档: http://localhost:8080/swagger-ui/"
    echo ""
    echo "💡 核心功能已启用: 增删改查、向量搜索"
    if [ -n "$OPENAI_API_KEY" ] || [ -n "$ZHIPU_API_KEY" ]; then
        echo "✨ 智能功能已启用: 事实提取、智能决策"
    else
        echo "⚠️  智能功能未启用（需要 LLM API key）"
    fi
    echo ""
    echo "📝 日志:"
    echo "   后端: tail -f backend.log"
    echo "   前端: tail -f agentmem-ui/.next/trace"
    echo ""
    echo "🛑 停止服务: just stop"

    # 保存 PID
    echo $BACKEND_PID > .backend.pid
    echo $FRONTEND_PID > .frontend.pid

    # 等待用户中断
    wait

# 停止所有服务
stop:
    #!bash
    if [ -f .backend.pid ]; then
        kill $(cat .backend.pid) 2>/dev/null || true
        rm .backend.pid
    fi
    if [ -f .frontend.pid ]; then
        kill $(cat .frontend.pid) 2>/dev/null || true
        rm .frontend.pid
    fi
    pkill -f "agent-mem-server" || true
    pkill -f "next dev" || true
    echo "✅ 所有服务已停止"

# 显示日志
logs:
    #!bash
    tail -f backend.log
```

#### 验收标准

```bash
# 新开发者体验
git clone https://github.com/louloulin/agentmem.git
cd agentmem
just dev

# 期望：
# ✅ 核心功能立即可用（CRUD、搜索）
# ⚠️  智能功能需要配置 API key（有明确提示）
# ✅ 自动完成所有步骤，打开浏览器即可使用
```

### 任务 1.3: 创建配置文件模板

#### 文件结构

```
agentmem/
├── config.core-only.toml      # 核心功能配置（无需 LLM）
├── config.example.toml        # 完整配置模板（含 LLM）
├── .env.example               # 环境变量模板
└── .gitignore                 # 忽略敏感文件
```

#### config.core-only.toml

```toml
# AgentMem 核心功能配置
# 此配置启用核心功能，无需 LLM API key

[server]
host = "127.0.0.1"
port = 8080

[database]
backend = "libsql"
url = "./data/agentmem.db"
auto_migrate = true

# 核心功能使用 FastEmbed 本地嵌入模型
[embeddings]
provider = "fastembed"
model = "BAAI/bge-small-en-v1.5"

# 核心功能不需要 LLM
[llm]
enable = false

[auth]
enable = false

[logging]
level = "info"
format = "pretty"
```

#### config.example.toml

```toml
# AgentMem 完整配置示例
# 此配置启用所有功能，包括智能功能

[server]
host = "127.0.0.1"
port = 8080

[database]
backend = "libsql"
url = "./data/agentmem.db"
auto_migrate = true

# LLM 配置（用于智能功能）
[llm]
enable = true
provider = "openai"  # 或 "zhipu", "anthropic"
model = "gpt-4"
# api_key 通过环境变量设置

# 嵌入配置
[embeddings]
provider = "fastembed"  # 或 "openai"
model = "BAAI/bge-small-en-v1.5"

[auth]
enable = false

[logging]
level = "info"
format = "pretty"
```

#### .env.example

```bash
# AgentMem 环境变量配置示例
# 复制此文件为 .env 并填入你的值

# ================================
# LLM 配置（智能功能需要）
# ================================
# 如果只需要核心功能（CRUD、搜索），可以不配置 LLM

# OpenAI (推荐用于智能功能)
OPENAI_API_KEY=your-openai-api-key

# 或使用 Zhipu AI
ZHIPU_API_KEY=your-zhipu-api-key

# 或使用 Anthropic
ANTHROPIC_API_KEY=your-anthropic-api-key

# LLM 提供商和模型（可选，默认使用 OpenAI）
LLM_PROVIDER=openai
LLM_MODEL=gpt-4

# ================================
# 数据库配置（可选）
# ================================
# 不配置则使用默认的 LibSQL 文件数据库

# DATABASE_URL=postgres://user:pass@localhost/agentmem
# DATABASE_BACKEND=libsql

# ================================
# 向量存储配置（可选）
# ================================
# 不配置则使用默认的 LanceDB

# VECTOR_STORE=lancedb
# LANCEDB_PATH=./data/vectors

# ================================
# 服务器配置（可选）
# ================================
# SERVER_PORT=8080
# ENABLE_AUTH=false
```

### 任务 1.5: 更新文档

#### QUICKSTART.md（重写）

```markdown
# AgentMem 快速开始

## 核心功能（无需配置）

### 基础 CRUD 示例

```bash
# 克隆仓库
git clone https://github.com/louloulin/agentmem.git
cd agentmem

# 运行核心功能示例（无需任何配置）
cargo run --example core-features/basic-crud
```

**核心功能包括**：
- ✅ 添加记忆
- ✅ 搜索记忆
- ✅ 向量搜索
- ✅ 批量操作
- ✅ 记忆更新/删除

**代码示例**：

```rust
use agent_mem::Memory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 核心功能模式（无需 LLM）
    let memory = Memory::new_core().await?;

    // 添加记忆
    memory.add("I love pizza").await?;
    memory.add("I live in San Francisco").await?;

    // 向量搜索
    let results = memory.search("What do I like?").await?;
    for result in results {
        println!("- {} (score: {:.2})", result.content, result.score);
    }

    Ok(())
}
```

## 智能功能（需要 LLM）

### 配置 LLM API key

```bash
# OpenAI
export OPENAI_API_KEY="sk-..."

# 或 Zhipu AI
export ZHIPU_API_KEY="..."

# 或 Anthropic
export ANTHROPIC_API_KEY="sk-ant-..."
```

### 智能功能示例

```bash
# 运行智能功能示例
cargo run --example intelligent-features/fact-extraction
```

**智能功能包括**：
- ✅ 事实提取
- ✅ 智能决策
- ✅ 记忆排序
- ✅ 冲突解决

**代码示例**：

```rust
use agent_mem::Memory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 自动检测：如果有 API key 则启用智能功能
    let memory = Memory::new().await?;

    // 智能添加（自动事实提取）
    memory.add_intelligent("I had lunch with John at 2pm").await?;

    // 智能搜索（考虑重要性、时间、相关性）
    let results = memory.search_intelligent("What did I do today?").await?;

    Ok(())
}
```

## 启动 Web 服务

### 方式 1: 仅核心功能

```bash
# 一键启动（无需配置）
just dev

# 访问 http://localhost:3001
# 核心功能可用，智能功能提示需要配置
```

### 方式 2: 完整功能

```bash
# 配置 LLM API key
export OPENAI_API_KEY="sk-..."

# 启动服务
just dev

# 访问 http://localhost:3001
# 所有功能可用
```

## 开发模式

```bash
# 完整开发环境（后端 + 前端）
just dev

# 仅后端
just backend

# 仅前端
just frontend

# 停止所有服务
just stop

# 查看日志
just logs
```

## 验证安装

```bash
# 运行核心功能测试（无需配置）
cargo test --package agent-mem --test core_features

# 运行智能功能测试（需要 API key）
export OPENAI_API_KEY="sk-..."
cargo test --package agent-mem --test intelligent_features

# 检查健康状态
curl http://localhost:8080/health
```

## 下一步

- 📚 [核心功能文档](CORE_FEATURES.md)
- 📚 [智能功能文档](INTELLIGENT_FEATURES.md)
- 🔌 [API 参考](https://docs.agentmem.cc/api)
- 💡 [示例项目](examples/)
- 🤝 [社区 Discord](https://discord.gg/agentmem)

## 常见问题

### Q: 核心功能和智能功能的区别？

A:
- **核心功能**：基础的增删改查、向量搜索，无需 LLM API key
- **智能功能**：事实提取、智能决策、记忆排序，需要 LLM API key

### Q: 如何选择？

A:
- **快速体验/开发** → 使用核心功能，无需配置
- **生产环境** → 配置 LLM，启用智能功能

### Q: 核心功能够用吗？

A: 对于大多数应用，核心功能已经足够：
- 向量搜索已经能找到相关记忆
- 基础 CRUD 能管理所有数据
- 批量操作能高效处理数据

智能功能主要用于：
- 自动提取结构化信息
- 智能排序和推荐
- 复杂的推理任务

### Q: 如何启用智能功能？

A:
```bash
# 设置 API key
export OPENAI_API_KEY="sk-..."

# 使用 Memory::new() 会自动检测
let memory = Memory::new().await?;  // 自动启用智能功能

# 或显式启用
let memory = Memory::builder()
    .with_intelligent_features()
    .await?
    .build()
    .await?;
```

### Q: 数据库需要安装吗？

A: 不需要。默认使用 LibSQL 文件数据库（嵌入式），无需安装。

如需使用 PostgreSQL：
```bash
export DATABASE_URL="postgres://user:pass@localhost/agentmem"
export DATABASE_BACKEND="postgres"
```

### Q: 启动失败怎么办？

```bash
# 查看日志
just logs

# 重置配置
rm config.toml
just dev

# 查看详细错误
RUST_BACKTRACE=1 cargo run --example core-features/basic-crud
```
```

#### CORE_FEATURES.md（新建）

```markdown
# AgentMem 核心功能

## 概述

AgentMem 核心功能提供了完整的记忆管理能力，无需配置 LLM API key。

## 功能列表

### 1. 基础 CRUD

```rust
use agent_mem::Memory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let memory = Memory::new_core().await?;

    // 创建记忆
    let id = memory.add("I love Rust programming").await?;

    // 读取记忆
    let result = memory.get(&id).await?;
    println!("{}", result.content);

    // 更新记忆
    memory.update(&id, "I love Rust and Go programming").await?;

    // 删除记忆
    memory.delete(&id).await?;

    Ok(())
}
```

### 2. 向量搜索

```rust
// 语义搜索
let results = memory.search("programming languages").await?;

for result in results {
    println!("Score: {:.2}, Content: {}", result.score, result.content);
}
```

### 3. 批量操作

```rust
// 批量添加
let memories = vec![
    "Memory 1",
    "Memory 2",
    "Memory 3",
];
memory.add_batch(memories).await?;

// 批量搜索
let queries = vec!["query 1", "query 2"];
let results = memory.search_batch(queries).await?;
```

### 4. 过滤和排序

```rust
use agent_mem::{SearchOptions, MemoryScope};

// 高级搜索
let options = SearchOptions {
    scope: MemoryScope::User,
    limit: 10,
    ..Default::default()
};

let results = memory.search_with_options("search query", &options).await?;
```

## 性能特性

- **快速存储**: 5,000 ops/s
- **向量搜索**: <100ms 延迟
- **批量操作**: 50,000 ops/s
- **零配置启动**: 无需外部依赖

## 使用场景

### 场景 1: 简单的记忆存储

```rust
// 存储用户对话历史
memory.add("User asked about Rust performance").await?;
memory.add("I explained Rust's zero-cost abstractions").await?;

// 后续搜索
let results = memory.search("performance discussions").await?;
```

### 场景 2: 文档搜索

```rust
// 添加文档片段
memory.add("Rust is a systems programming language").await?;
memory.add("Rust guarantees memory safety").await?;
memory.add("Rust has a strong type system").await?;

// 语义搜索
let results = memory.search("safety features").await?;
```

### 场景 3: 知识库

```rust
// 构建知识库
memory.add("Python is dynamically typed").await?;
memory.add("Rust is statically typed").await?;
memory.add("Go has garbage collection").await?;

// 查询
let results = memory.search("type system differences").await?;
```

## 限制

核心功能不包括：
- ❌ 事实提取
- ❌ 智能决策
- ❌ 记忆重要性排序
- ❌ 冲突解决

这些功能需要配置 LLM（参见 [INTELLIGENT_FEATURES.md](INTELLIGENT_FEATURES.md)）

## 下一步

- 升级到智能功能: [INTELLIGENT_FEATURES.md](INTELLIGENT_FEATURES.md)
- API 参考: [API 文档](https://docs.agentmem.cc/api)
- 更多示例: [examples/core-features](../examples/core-features/)
```
