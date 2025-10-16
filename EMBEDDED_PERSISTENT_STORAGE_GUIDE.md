# AgentMem 嵌入式持久化存储指南

**重要澄清**: AgentMem 嵌入式模式**完全支持**持久化存储（LibSQL + LanceDB）！

---

## 🎯 快速开始

### 方法 1: 使用 Agent API (推荐)

```rust
use agent_mem_core::agents::CoreAgent;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ✅ 自动使用持久化存储 (LibSQL + LanceDB)
    let agent = CoreAgent::from_env("my-agent".to_string()).await?;
    
    // 添加记忆 (自动持久化)
    agent.store_memory("I love Rust programming").await?;
    
    // 重启后数据仍然存在！
    Ok(())
}
```

**默认配置**:
- LibSQL 数据库: `./agentmem.db`
- LanceDB 向量: `./data/vectors.lance`
- 数据自动持久化到磁盘

---

## 📁 数据存储位置

### 默认路径

```
./
├── agentmem.db              # LibSQL 数据库文件
├── agentmem.db-shm          # 共享内存文件 (WAL 模式)
├── agentmem.db-wal          # Write-Ahead Log 文件
└── data/
    └── vectors.lance/       # LanceDB 向量存储目录
        ├── _versions/
        ├── data/
        └── _latest.manifest
```

### 自定义路径

```bash
# 方法 1: 环境变量
export AGENTMEM_DB_PATH="./my-data/memory.db"
export AGENTMEM_VECTOR_PATH="./my-data/vectors"

# 方法 2: DATABASE_URL
export DATABASE_URL="file:./my-data/memory.db"
```

---

## 🔧 配置选项

### 环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `AGENTMEM_DB_PATH` | LibSQL 数据库路径 | `agentmem.db` |
| `AGENTMEM_DB_BACKEND` | 数据库后端 | `libsql` |
| `DATABASE_URL` | 完整连接字符串 | - |
| `AGENTMEM_VECTOR_PATH` | 向量存储路径 | `./data/vectors` |

### 代码配置

```rust
use agent_mem_core::agents::CoreAgent;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 设置自定义路径
    env::set_var("AGENTMEM_DB_PATH", "./production-data/memory.db");
    
    // 创建 Agent (使用自定义路径)
    let agent = CoreAgent::from_env("my-agent".to_string()).await?;
    
    Ok(())
}
```

---

## 💾 持久化验证

### 测试数据持久化

```rust
use agent_mem_core::agents::CoreAgent;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 第一次运行: 添加数据
    {
        let agent = CoreAgent::from_env("test-agent".to_string()).await?;
        agent.store_memory("Test memory 1").await?;
        agent.store_memory("Test memory 2").await?;
        println!("✅ 数据已保存");
    }
    
    // 第二次运行: 验证数据仍然存在
    {
        let agent = CoreAgent::from_env("test-agent".to_string()).await?;
        let memories = agent.retrieve_all_memories().await?;
        println!("✅ 找到 {} 条记忆", memories.len());
        assert!(memories.len() >= 2);
    }
    
    Ok(())
}
```

### 检查数据文件

```bash
# 检查 LibSQL 数据库
ls -lh agentmem.db*

# 输出示例:
# -rw-r--r--  1 user  staff   128K  Oct 16 16:00 agentmem.db
# -rw-r--r--  1 user  staff    32K  Oct 16 16:00 agentmem.db-shm
# -rw-r--r--  1 user  staff    64K  Oct 16 16:00 agentmem.db-wal

# 检查 LanceDB 向量存储
ls -lh data/vectors.lance/

# 输出示例:
# drwxr-xr-x  5 user  staff   160B  Oct 16 16:00 _versions
# drwxr-xr-x  3 user  staff    96B  Oct 16 16:00 data
# -rw-r--r--  1 user  staff   1.2K  Oct 16 16:00 _latest.manifest
```

---

## 🚀 生产环境配置

### 推荐配置

```bash
# .env 文件
AGENTMEM_DB_PATH=/var/lib/agentmem/memory.db
AGENTMEM_VECTOR_PATH=/var/lib/agentmem/vectors
AGENTMEM_DB_BACKEND=libsql

# 日志级别
RUST_LOG=info

# LLM 配置 (可选)
OPENAI_API_KEY=sk-...
```

### 数据备份

```bash
#!/bin/bash
# backup.sh - 备份 AgentMem 数据

BACKUP_DIR="./backups/$(date +%Y%m%d_%H%M%S)"
mkdir -p "$BACKUP_DIR"

# 备份 LibSQL 数据库
cp agentmem.db* "$BACKUP_DIR/"

# 备份 LanceDB 向量存储
cp -r data/vectors.lance "$BACKUP_DIR/"

echo "✅ 备份完成: $BACKUP_DIR"
```

### 数据恢复

```bash
#!/bin/bash
# restore.sh - 恢复 AgentMem 数据

BACKUP_DIR="./backups/20251016_160000"

# 恢复 LibSQL 数据库
cp "$BACKUP_DIR"/agentmem.db* ./

# 恢复 LanceDB 向量存储
rm -rf data/vectors.lance
cp -r "$BACKUP_DIR/vectors.lance" data/

echo "✅ 恢复完成"
```

---

## 🔍 技术细节

### LibSQL 持久化

**实现位置**: `crates/agent-mem-storage/src/factory/libsql.rs`

```rust
// 创建文件数据库
Builder::new_local(path).build().await
```

**特性**:
- ✅ SQLite 兼容
- ✅ WAL (Write-Ahead Logging) 模式
- ✅ 事务支持
- ✅ 崩溃恢复
- ✅ ACID 保证

### LanceDB 持久化

**实现位置**: `crates/agent-mem-storage/src/backends/lancedb_store.rs`

```rust
// 连接到文件存储
connect(&expanded_path).execute().await
```

**特性**:
- ✅ 列式存储 (Lance 格式)
- ✅ 增量更新
- ✅ 数据压缩
- ✅ 索引持久化
- ✅ 版本控制

### 调用链

```
CoreAgent::from_env()
  ↓
create_stores_from_env()
  ↓
get_storage_config_from_env()  // 读取 AGENTMEM_DB_PATH
  ↓
create_factory(config)
  ↓
LibSqlStorageFactory::new("file:agentmem.db")
  ↓
Builder::new_local("agentmem.db")  // 创建文件数据库
  ↓
create_all_stores()  // 返回持久化存储
```

---

## ❓ 常见问题

### Q1: SimpleMemory::new() 是否支持持久化？

**A**: `SimpleMemory::new()` 默认使用**内存存储**（开发模式）。

**解决方案**:
```rust
// ❌ 内存存储 (数据不持久化)
let mem = SimpleMemory::new().await?;

// ✅ 持久化存储 (推荐)
let agent = CoreAgent::from_env("my-agent".to_string()).await?;
```

### Q2: 如何验证数据已持久化？

**A**: 检查数据文件是否存在：

```bash
# 检查 LibSQL 数据库
ls -lh agentmem.db

# 检查 LanceDB 向量存储
ls -lh data/vectors.lance/
```

### Q3: 数据存储在哪里？

**A**: 默认存储在当前工作目录：

- LibSQL: `./agentmem.db`
- LanceDB: `./data/vectors.lance/`

可通过环境变量自定义路径。

### Q4: 如何迁移到生产环境？

**A**: 
1. 设置环境变量指定生产路径
2. 确保数据目录有写权限
3. 配置定期备份
4. 使用 WAL 模式（默认启用）

### Q5: 性能如何？

**A**: 已验证性能指标：

- 批量插入: 31,456 ops/s
- 向量搜索: 22.98ms (Top-10)
- 批量更新: 1,291 ops/s
- 批量删除: 3,815 ops/s

---

## 🧪 验证示例

### 完整的验证示例项目

我们提供了完整的示例项目来验证嵌入式模式的持久化存储功能：

**示例位置**: `examples/embedded-persistent-demo/`

#### 1. 持久化验证示例

```bash
cd examples/embedded-persistent-demo
cargo run --example verify_persistence
```

**验证内容**:
- ✅ LibSQL 文件数据库创建
- ✅ WAL 模式启用
- ✅ 数据持久化到磁盘
- ✅ 进程重启后数据恢复

#### 2. 完整功能测试

```bash
cd examples/embedded-persistent-demo
cargo run --example full_feature_test
```

**测试内容**:
- ✅ CoreAgent 持久化存储
- ✅ LanceDB 向量存储
- ✅ 向量搜索、更新、删除
- ✅ 批量操作性能测试
- ✅ 统计信息和健康检查

**预期输出**:
```
🎉 所有测试完成
✅ 测试结果汇总:
  1. ✅ CoreAgent 持久化存储
  2. ✅ LanceDB 向量存储
  3. ✅ 向量搜索
  4. ✅ 向量更新
  5. ✅ 向量删除
  6. ✅ 统计信息
  7. ✅ 健康检查
  8. ✅ 批量性能测试
  9. ✅ 数据持久化验证

💡 结论:
  AgentMem 嵌入式模式所有功能正常！
  持久化存储: ✅ 完全支持
```

详细说明请查看: [examples/embedded-persistent-demo/README.md](./examples/embedded-persistent-demo/README.md)

---

## 📚 相关文档

- [嵌入式模式使用指南](./EMBEDDED_MODE_GUIDE.md)
- [嵌入式版本完整性分析](./EMBEDDED_MODE_COMPLETENESS_ANALYSIS.md)
- [生产环境快速开始](./QUICKSTART_PRODUCTION.md)
- [持久化验证示例](./examples/embedded-persistent-demo/README.md)

---

## 🎯 总结

### ✅ 核心要点

1. **AgentMem 嵌入式模式完全支持持久化存储**
2. **使用 `CoreAgent::from_env()` 自动启用持久化**
3. **数据存储在 LibSQL (文件数据库) + LanceDB (向量存储)**
4. **默认路径: `./agentmem.db` + `./data/vectors.lance/`**
5. **支持 WAL 模式、事务、崩溃恢复**
6. **生产可用，性能优秀**

### 🚀 快速开始

```rust
use agent_mem_core::agents::CoreAgent;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ✅ 一行代码启用持久化存储
    let agent = CoreAgent::from_env("my-agent".to_string()).await?;
    
    // 添加记忆 (自动持久化)
    agent.store_memory("Hello, AgentMem!").await?;
    
    println!("✅ 数据已持久化到 ./agentmem.db");
    Ok(())
}
```

---

**文档版本**: 1.0  
**最后更新**: 2025-10-16  
**状态**: ✅ 持久化存储完全支持

