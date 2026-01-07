# AgentMem 2.5 快速开始指南

> **AgentMem** - 企业级 AI Agent 记忆管理系统
> 支持 CRUD、向量搜索、智能提取等功能

## 🎯 两种使用模式

AgentMem 2.5 提供两种使用模式，满足不同需求:

### 模式一：核心功能模式（无需配置）⚡

**适合场景**: 大多数应用，只需要 CRUD 和向量搜索

**特点**:
- ✅ **零配置启动** - 无需 API Key
- ✅ **本地运行** - 数据完全在本地
- ✅ **向量搜索** - 语义相似度搜索
- ✅ **快速部署** - 5 分钟内启动

**可用功能**:
- 添加记忆
- 向量搜索
- 批量操作
- 记忆管理
- 导出/导入

### 模式二：智能功能模式（需 LLM）🧠

**适合场景**: 需要自动提取结构化信息、智能排序

**特点**:
- ✅ **事实提取** - 自动从文本提取关键信息
- ✅ **智能排序** - 基于重要性、时间排序
- ✅ **自动分类** - 智能识别记忆类型
- ✅ **上下文理解** - 更精准的搜索结果

**需要配置**: LLM API Key (OpenAI/智谱/Anthropic)

---

## 🚀 快速开始

### 1️⃣ 核心功能模式（推荐新手）

#### 第一步：克隆项目

```bash
git clone https://github.com/louloulin/agentmem.git
cd agentmem
```

#### 第二步：使用核心配置

```bash
# 使用核心功能配置
cp config.core-only.toml config.toml
```

#### 第三步：启动服务

```bash
# 一键启动（使用 justfile）
just dev

# 或手动启动
cargo build --release
./target/release/agent-mem-server
```

#### 第四步：验证服务

```bash
# 健康检查
curl http://localhost:8080/health

# 查看API文档
open http://localhost:8080/swagger-ui/
```

✅ **就这么简单！** 核心功能已就绪

#### 使用示例

**添加记忆**:

```bash
curl -X POST http://localhost:8080/api/v1/memories \
  -H "Content-Type: application/json" \
  -H "X-User-ID: default" \
  -d '{
    "content": "I love Rust programming language",
    "metadata": {"category": "programming"}
  }'
```

**向量搜索**:

```bash
curl -X POST http://localhost:8080/api/v1/memories/search \
  -H "Content-Type: application/json" \
  -H "X-User-ID: default" \
  -d '{
    "query": "programming languages",
    "limit": 10
  }'
```

**代码示例**:

```rust
use agent_mem::Memory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 核心功能模式（无需 LLM）
    let memory = Memory::new_core().await?;

    // 添加记忆
    memory.add("I love Rust programming").await?;

    // 向量搜索
    let results = memory.search("programming").await?;
    for result in results {
        println!("{} (score: {:.2})", result.content, result.score);
    }

    Ok(())
}
```

---

### 2️⃣ 智能功能模式（高级功能）

#### 第一步：配置 LLM API Key

**方式 A: 使用环境变量**

```bash
# OpenAI
export OPENAI_API_KEY="sk-your-openai-api-key"

# 或智谱 AI（国产）
export ZHIPU_API_KEY="your-zhipu-api-key"

# 或 Anthropic Claude
export ANTHROPIC_API_KEY="sk-ant-your-key"
```

**方式 B: 使用配置文件**

```bash
# 复制示例配置
cp config.core-only.toml config.toml

# 编辑 config.toml，启用 LLM
vim config.toml
```

修改以下配置:

```toml
[llm]
enable = true
provider = "openai"  # 或 "zhipu", "anthropic"
api_key = "your-api-key-here"
model = "gpt-4"
```

#### 第二步：启动服务

```bash
just dev
```

#### 第三步：使用智能功能

```rust
use agent_mem::Memory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 智能功能模式（需要 LLM API Key）
    let memory = Memory::new().await?;

    // 智能添加（自动提取事实）
    let memory_id = memory.add_intelligent(
        "I had lunch with John at 2pm at the Italian restaurant"
    ).await?;

    // 智能搜索（考虑重要性、时间、相关性）
    let results = memory.search_intelligent(
        "What did I do today?"
    ).await?;

    for result in results {
        println!(
            "{}\nImportance: {:.2}\nRelevance: {:.2}",
            result.content,
            result.importance,
            result.relevance
        );
    }

    Ok(())
}
```

---

## 📚 更多示例

### 核心功能示例

```bash
# examples/core-features/basic-crud
cargo run --example basic-crud

# examples/core-features/vector-search
cargo run --example vector-search

# examples/core-features/batch-operations
cargo run --example batch-operations
```

### 智能功能示例

```bash
# examples/intelligent-features/fact-extraction
cargo run --example fact-extraction

# examples/intelligent-features/intelligent-search
cargo run --example intelligent-search

# examples/intelligent-features/auto-categorization
cargo run --example auto-categorization
```

---

## 🔧 配置说明

### 核心功能配置文件

**文件**: `config.core-only.toml`

**关键配置**:

```toml
[database]
backend = "libsql"
url = "file:./data/agentmem.db"

[embeddings]
provider = "fastembed"
model = "BAAI/bge-small-en-v1.5"

[llm]
enable = false  # 核心功能不需要 LLM
```

### 环境变量配置

**文件**: `.env`

**必需配置**（智能功能）:

```bash
# LLM API Key（选择一个）
OPENAI_API_KEY=sk-your-key
# ZHIPU_API_KEY=your-key
# ANTHROPIC_API_KEY=sk-ant-your-key
```

**可选配置**:

```bash
# 服务器
SERVER_PORT=8080

# 数据库
DATABASE_URL=file:./data/agentmem.db

# 日志
LOG_LEVEL=info
```

---

## 🎯 功能对比

| 功能 | 核心模式 | 智能模式 |
|------|---------|---------|
| **添加记忆** | ✅ | ✅ |
| **向量搜索** | ✅ | ✅ |
| **CRUD 操作** | ✅ | ✅ |
| **批量操作** | ✅ | ✅ |
| **事实提取** | ❌ | ✅ |
| **智能排序** | ❌ | ✅ |
| **自动分类** | ❌ | ✅ |
| **API Key** | 不需要 | 需要 |
| **配置难度** | 极简 | 中等 |
| **使用成本** | 免费 | 付费 |
| **启动时间** | 1 分钟 | 3 分钟 |

---

## 🌟 API 文档

启动服务后访问:

- **Swagger UI**: http://localhost:8080/swagger-ui/
- **Redoc UI**: http://localhost:8080/redoc/
- **健康检查**: http://localhost:8080/health
- **指标监控**: http://localhost:8080/metrics

---

## ❓ 常见问题

### Q1: 核心功能够用吗？

**A**: 对大多数应用，是的。向量搜索已经能找到相关记忆，无需复杂的智能功能。

### Q2: 何时需要智能功能？

**A**: 需要以下功能时:
- 自动提取结构化信息（人名、时间、地点）
- 智能排序（按重要性、时间衰减）
- 自动分类（工作、个人、学习等）
- 上下文理解（更精准的搜索）

### Q3: 数据库需要安装吗？

**A**: 不需要。默认使用 LibSQL 文件数据库（`./data/agentmem.db`），零依赖。

### Q4: 可以从核心模式升级到智能模式吗？

**A**: 可以！只需配置 LLM API Key 并重启服务，无需迁移数据。

### Q5: 性能如何？

**A**:
- 核心模式：本地处理，延迟 < 50ms
- 智能模式：LLM 调用，延迟 200-500ms

### Q6: 数据安全吗？

**A**:
- 核心模式：数据完全在本地，100% 安全
- 智能模式：需要发送到 LLM API，遵循提供商隐私政策

---

## 🛠️ 故障排除

### 问题：服务启动失败

```bash
# 检查配置文件语法
cat config.toml

# 检查端口占用
lsof -i :8080

# 查看日志
just logs backend
```

### 问题：向量搜索无结果

```bash
# 确认已添加记忆
curl http://localhost:8080/api/v1/memories \
  -H "X-User-ID: default"

# 检查嵌入模型
curl http://localhost:8080/api/v1/health | jq .
```

### 问题：LLM 调用失败

```bash
# 检查 API Key
echo $OPENAI_API_KEY

# 测试 API 连接
curl https://api.openai.com/v1/models \
  -H "Authorization: Bearer $OPENAI_API_KEY"
```

---

## 📖 下一步

### 学习资源

- 📖 [完整文档](./docs/README.md)
- 🎓 [示例项目](./examples/)
- 💡 [最佳实践](./docs/BEST_PRACTICES.md)
- 🔧 [API 参考](./docs/API_REFERENCE.md)

### 进阶功能

- 🔌 [插件开发](./docs/PLUGIN_DEVELOPMENT.md)
- 🚀 [部署指南](./docs/DEPLOYMENT.md)
- 📊 [性能优化](./docs/PERFORMANCE.md)
- 🔒 [安全配置](./docs/SECURITY.md)

### 社区支持

- 💬 [Discussions](https://github.com/louloulin/agentmem/discussions)
- 🐛 [Bug 报告](https://github.com/louloulin/agentmem/issues)
- ✨ [功能请求](https://github.com/louloulin/agentmem/issues)

---

## 🎉 开始使用 AgentMem

选择适合你的模式:

**新手/快速原型** → 核心功能模式
**生产应用/智能需求** → 智能功能模式

```bash
# 核心功能（5分钟启动）
cp config.core-only.toml config.toml
just dev

# 智能功能（需要 API Key）
export OPENAI_API_KEY="sk-..."
just dev
```

祝使用愉快！🚀
