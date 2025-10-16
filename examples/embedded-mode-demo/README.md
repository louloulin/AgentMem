# AgentMem 嵌入式模式使用示例

本目录包含 AgentMem 嵌入式模式（LibSQL + LanceDB）的完整使用示例。

## ✅ 功能状态

嵌入式模式已 **100% 完成**，包括：

- ✅ **LibSQL**: 结构化数据存储（用户、组织、Agent、消息等）
- ✅ **LanceDB**: 向量数据存储（语义搜索、记忆检索）
- ✅ **9 个 Repositories**: User, Organization, Agent, Message, Tool, ApiKey, Memory, Block, Association
- ✅ **向量操作**: 插入、搜索、删除、更新、获取
- ✅ **性能优异**: 31,456 ops/s 插入，22.98ms 搜索

## 📦 示例列表

### 1. 基础使用 (`basic_usage.rs`)

演示如何使用嵌入式模式进行基本的 CRUD 操作：

- 创建组织、用户、Agent
- 查询数据
- 更新数据
- 软删除

**运行**:
```bash
cd agentmen
cargo run --bin basic-usage --features "libsql,lancedb"
```

### 2. 向量搜索 (`vector_search.rs`)

演示如何使用 LanceDB 进行向量存储和语义搜索：

- 插入向量数据
- 执行语义搜索
- 获取单个向量
- 更新向量
- 删除向量
- 查看统计信息

**运行**:
```bash
cd agentmen
cargo run --bin vector-search --features "libsql,lancedb"
```

### 3. 完整工作流 (`full_workflow.rs`)

演示一个完整的 AI Agent 工作流：

- 创建用户和 Agent
- 存储对话消息
- 生成消息嵌入向量
- 执行语义搜索查找相关历史
- 更新 Agent 配置

**运行**:
```bash
cd agentmen
cargo run --bin full-workflow --features "libsql,lancedb"
```

## 🚀 快速开始

### 1. 安装依赖

确保已安装 Rust 1.70+：

```bash
rustc --version
```

### 2. 运行示例

```bash
# 进入 agentmen 目录
cd agentmen

# 运行基础使用示例
cargo run --example basic-usage --features "libsql,lancedb"

# 运行向量搜索示例
cargo run --example vector-search --features "libsql,lancedb"

# 运行完整工作流示例
cargo run --example full-workflow --features "libsql,lancedb"
```

### 3. 查看数据

示例运行后，数据会保存在 `./data/` 目录：

```bash
ls -lh data/
# agentmem.db        - LibSQL 数据库文件
# vectors.lance/     - LanceDB 向量数据目录
```

## 📖 代码示例

### 创建嵌入式模式配置

```rust
use agent_mem_config::{AgentMemConfig, DeploymentMode, EmbeddedModeConfig};
use std::path::PathBuf;

let config = AgentMemConfig {
    deployment_mode: DeploymentMode::Embedded(EmbeddedModeConfig {
        database_path: PathBuf::from("./data/agentmem.db"),
        vector_path: PathBuf::from("./data/vectors.lance"),
        dimension: 1536,
        enable_wal: true,
        cache_size_kb: 10240,
    }),
};
```

### 创建 Storage Factory

```rust
use agent_mem_core::storage::factory::StorageFactory;

let repositories = StorageFactory::create(config).await?;
```

### 使用 Repositories

```rust
// 创建用户
let user = User {
    id: Uuid::new_v4().to_string(),
    organization_id: org_id.clone(),
    name: "张三".to_string(),
    status: "active".to_string(),
    timezone: "Asia/Shanghai".to_string(),
    created_at: Utc::now(),
    updated_at: Utc::now(),
    is_deleted: false,
    created_by_id: None,
    last_updated_by_id: None,
};

let created_user = repositories.users.create(&user).await?;

// 查询用户
let found_user = repositories.users.find_by_id(&user_id).await?;

// 更新用户
let mut updated_user = found_user.unwrap();
updated_user.name = "李四".to_string();
repositories.users.update(&updated_user).await?;

// 删除用户（软删除）
repositories.users.delete(&user_id).await?;
```

### 使用向量存储

```rust
use agent_mem_storage::backends::lancedb_store::LanceDBStore;
use agent_mem_traits::{VectorStore, VectorData};

// 创建向量存储
let vector_store = LanceDBStore::new("./data/vectors.lance", "embeddings").await?;

// 插入向量
let vectors = vec![
    VectorData {
        id: "doc1".to_string(),
        vector: vec![0.1, 0.2, 0.3, ...], // 1536 维向量
        metadata: HashMap::from([
            ("text".to_string(), serde_json::json!("示例文本")),
        ]),
    },
];
vector_store.add_vectors(vectors).await?;

// 搜索向量
let query = vec![0.1, 0.2, 0.3, ...]; // 查询向量
let results = vector_store.search_vectors(query, 10, Some(0.7)).await?;

// 获取向量
let vector = vector_store.get_vector("doc1").await?;

// 更新向量
let updated = VectorData { id: "doc1".to_string(), ... };
vector_store.update_vectors(vec![updated]).await?;

// 删除向量
vector_store.delete_vectors(vec!["doc1".to_string()]).await?;
```

## 🎯 使用场景

### 1. 单机部署

适合：
- 开发环境
- 小型应用
- 边缘设备
- 离线应用

优势：
- ✅ 零配置，开箱即用
- ✅ 单个二进制文件
- ✅ 无需外部依赖
- ✅ 性能优异

### 2. AI Agent 应用

适合：
- 对话机器人
- 智能助手
- 知识库问答
- 文档检索

功能：
- ✅ 存储对话历史
- ✅ 语义搜索
- ✅ 上下文检索
- ✅ 记忆管理

### 3. 原型开发

适合：
- 快速原型
- MVP 开发
- 概念验证
- 功能测试

优势：
- ✅ 快速启动
- ✅ 简单部署
- ✅ 易于调试
- ✅ 低成本

## 📊 性能指标

| 指标 | 实际值 | 目标值 | 达成率 |
|------|--------|--------|--------|
| 插入性能 | 31,456 ops/s | > 1,000 ops/s | **3,045%** ✅ |
| 搜索性能 (1K 向量) | 22.98 ms | < 50 ms | **217%** ✅ |
| 向量相似度 (Cosine) | 2.35 µs | < 10 µs | **425%** ✅ |
| 批量操作 (1000 向量) | 5.02 ms | < 10 ms | **199%** ✅ |

## 🔧 配置选项

### EmbeddedModeConfig

```rust
pub struct EmbeddedModeConfig {
    /// LibSQL 数据库文件路径
    pub database_path: PathBuf,
    
    /// LanceDB 向量数据目录路径
    pub vector_path: PathBuf,
    
    /// 向量维度（默认 1536，OpenAI embeddings）
    pub dimension: usize,
    
    /// 启用 Write-Ahead Logging（推荐）
    pub enable_wal: bool,
    
    /// 缓存大小（KB）
    pub cache_size_kb: usize,
}
```

### 推荐配置

**开发环境**:
```rust
EmbeddedModeConfig {
    database_path: PathBuf::from("./dev-data/agentmem.db"),
    vector_path: PathBuf::from("./dev-data/vectors.lance"),
    dimension: 1536,
    enable_wal: true,
    cache_size_kb: 10240, // 10 MB
}
```

**生产环境**:
```rust
EmbeddedModeConfig {
    database_path: PathBuf::from("/var/lib/agentmem/agentmem.db"),
    vector_path: PathBuf::from("/var/lib/agentmem/vectors.lance"),
    dimension: 1536,
    enable_wal: true,
    cache_size_kb: 102400, // 100 MB
}
```

## 🐛 故障排除

### 问题 1: 数据库文件权限错误

**错误**: `Failed to create LibSQL connection: Permission denied`

**解决**:
```bash
# 确保数据目录存在且有写权限
mkdir -p ./data
chmod 755 ./data
```

### 问题 2: 向量维度不匹配

**错误**: `Vector dimension mismatch: expected 1536, got 768`

**解决**:
```rust
// 确保配置的维度与实际向量维度一致
let config = EmbeddedModeConfig {
    dimension: 768, // 修改为实际维度
    ...
};
```

### 问题 3: 编译错误

**错误**: `feature 'lancedb' not enabled`

**解决**:
```bash
# 确保启用了正确的 features
cargo build --features "libsql,lancedb"
```

## 📚 相关文档

- [AgentMem 架构设计](../../../doc/technical-design/memory-systems/mem21.md)
- [API 文档](https://docs.rs/agent-mem-core)
- [性能测试报告](../../../doc/technical-design/memory-systems/mem21.md#性能优化与测试)

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

MIT License

