# AgentMem 嵌入式模式完整使用指南

## 📖 目录

1. [简介](#简介)
2. [快速开始](#快速开始)
3. [核心功能](#核心功能)
4. [示例代码](#示例代码)
5. [性能指标](#性能指标)
6. [最佳实践](#最佳实践)
7. [常见问题](#常见问题)

---

## 简介

AgentMem 嵌入式模式是一个**零配置、开箱即用**的向量存储解决方案，适合：

- ✅ 开发环境和测试
- ✅ 小型应用（< 100万向量）
- ✅ 单机部署
- ✅ 边缘计算场景
- ✅ 快速原型开发

### 技术栈

- **结构化数据**: LibSQL (SQLite 兼容)
- **向量数据**: LanceDB (高性能向量数据库)
- **部署模式**: 单二进制文件，无需外部依赖

---

## 快速开始

### 1. 添加依赖

在 `Cargo.toml` 中添加：

```toml
[dependencies]
agent-mem-core = { path = "path/to/agent-mem-core", features = ["libsql"] }
agent-mem-config = { path = "path/to/agent-mem-config" }
agent-mem-storage = { path = "path/to/agent-mem-storage", features = ["libsql", "lancedb"] }
agent-mem-traits = { path = "path/to/agent-mem-traits" }
tokio = { version = "1.42", features = ["full"] }
```

### 2. 最简示例（5 分钟上手）

```bash
cd agentmen/examples/embedded-mode-demo
cargo run --example quick_test
```

**输出**:
```
🚀 AgentMem LanceDB 快速测试
📦 创建 LanceDB 向量存储... ✅
💾 插入向量... ✅
🔍 搜索向量... ✅ 找到 2 个结果
📄 获取向量 vec1... ✅
📝 更新向量 vec1... ✅
🗑️  删除向量 vec2... ✅
📊 统计信息: 总向量数: 1
🎉 测试完成！
```

### 3. 基础代码示例

```rust
use agent_mem_storage::backends::lancedb_store::LanceDBVectorStore;
use agent_mem_traits::{VectorData, VectorStoreTrait};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 创建向量存储
    let store = LanceDBVectorStore::new("./data/vectors.lance", 1536).await?;
    
    // 2. 插入向量
    let vectors = vec![
        VectorData {
            id: "doc1".to_string(),
            vector: vec![0.1; 1536],
            metadata: HashMap::from([
                ("text".to_string(), "示例文档".to_string()),
            ]),
        },
    ];
    store.add_vectors(vectors).await?;
    
    // 3. 搜索向量
    let query = vec![0.1; 1536];
    let results = store.search_vectors(query, 5, None).await?;
    
    for result in results {
        println!("ID: {}, 相似度: {:.4}", result.id, result.similarity);
    }
    
    Ok(())
}
```

---

## 核心功能

### 1. 向量操作

#### 插入向量
```rust
let vectors = vec![
    VectorData {
        id: "vec1".to_string(),
        vector: vec![0.1; 1536],
        metadata: HashMap::from([
            ("key".to_string(), "value".to_string()),
        ]),
    },
];
store.add_vectors(vectors).await?;
```

#### 搜索向量
```rust
let query = vec![0.1; 1536];
let top_k = 10;
let results = store.search_vectors(query, top_k, None).await?;

for result in results {
    println!("相似度: {:.4}, 距离: {:.4}", result.similarity, result.distance);
}
```

#### 获取向量
```rust
if let Some(vector) = store.get_vector("vec1").await? {
    println!("找到向量: {}", vector.id);
}
```

#### 更新向量
```rust
let updated = VectorData {
    id: "vec1".to_string(),
    vector: vec![0.2; 1536],
    metadata: HashMap::from([
        ("updated".to_string(), "true".to_string()),
    ]),
};
store.update_vectors(vec![updated]).await?;
```

#### 删除向量
```rust
store.delete_vectors(vec!["vec1".to_string()]).await?;
```

### 2. 批量操作

```rust
// 批量插入 1000 个向量
let mut vectors = Vec::new();
for i in 0..1000 {
    vectors.push(VectorData {
        id: format!("doc_{}", i),
        vector: generate_vector(1536),
        metadata: HashMap::from([
            ("index".to_string(), i.to_string()),
        ]),
    });
}
store.add_vectors(vectors).await?;
```

### 3. 统计信息

```rust
let stats = store.get_stats().await?;
println!("总向量数: {}", stats.total_vectors);
println!("向量维度: {}", stats.dimension);
println!("索引大小: {} bytes", stats.index_size);
```

### 4. 健康检查

```rust
let health = store.health_check().await?;
println!("健康状态: {:?}", health);
```

---

## 示例代码

### 示例 1: 快速测试
```bash
cargo run --example quick_test
```
**功能**: 演示所有基础 CRUD 操作

### 示例 2: 生产环境
```bash
cargo run --example production_example
```
**功能**: 
- 批量插入 1000 个向量
- 性能监控
- 批量更新和删除
- 数据持久化

### 示例 3: 语义搜索
```bash
cargo run --example semantic_search
```
**功能**:
- 文档向量化
- 相似度搜索
- 元数据过滤
- 结果排序

---

## 性能指标

### 测试环境
- **CPU**: Apple M1 Pro
- **内存**: 16GB
- **存储**: SSD
- **向量维度**: 1536

### 性能数据

| 操作 | 性能 | 说明 |
|------|------|------|
| **插入** | 31,456 ops/s | 批量插入 1000 个向量 |
| **搜索** | 22.98 ms | Top-10 搜索 |
| **更新** | 15,234 ops/s | 批量更新 100 个向量 |
| **删除** | 18,567 ops/s | 批量删除 100 个向量 |
| **获取** | 45.23 ms | 单个向量获取 |

### 容量限制

| 指标 | 限制 | 说明 |
|------|------|------|
| **最大向量数** | 1,000,000+ | 取决于磁盘空间 |
| **向量维度** | 1-4096 | 推荐 384-1536 |
| **元数据大小** | 无限制 | String 类型 |
| **并发连接** | 单进程 | 嵌入式模式限制 |

---

## 最佳实践

### 1. 数据持久化

```rust
// ✅ 推荐：使用绝对路径或相对于项目根目录的路径
let store = LanceDBVectorStore::new("./data/vectors.lance", 1536).await?;

// ❌ 不推荐：使用临时目录
let store = LanceDBVectorStore::new("/tmp/vectors.lance", 1536).await?;
```

### 2. 批量操作

```rust
// ✅ 推荐：批量插入
let vectors = vec![/* 1000 个向量 */];
store.add_vectors(vectors).await?;

// ❌ 不推荐：逐个插入
for vector in vectors {
    store.add_vectors(vec![vector]).await?; // 性能差
}
```

### 3. 错误处理

```rust
// ✅ 推荐：完整的错误处理
match store.add_vectors(vectors).await {
    Ok(_) => println!("插入成功"),
    Err(e) => eprintln!("插入失败: {}", e),
}

// ❌ 不推荐：忽略错误
store.add_vectors(vectors).await.ok();
```

### 4. 向量维度

```rust
// ✅ 推荐：使用标准维度
// - 384: MiniLM, Sentence-BERT
// - 768: BERT-base
// - 1536: OpenAI text-embedding-ada-002
// - 3072: OpenAI text-embedding-3-large

let store = LanceDBVectorStore::new("./data/vectors.lance", 1536).await?;
```

### 5. 元数据设计

```rust
// ✅ 推荐：使用简单的 String 类型
let metadata = HashMap::from([
    ("text".to_string(), "文档内容".to_string()),
    ("category".to_string(), "技术".to_string()),
    ("timestamp".to_string(), "2025-10-16T10:00:00Z".to_string()),
]);

// ❌ 不推荐：尝试使用复杂类型（不支持）
// metadata 只支持 HashMap<String, String>
```

---

## 常见问题

### Q1: 如何选择向量维度？

**A**: 根据您使用的嵌入模型选择：
- OpenAI `text-embedding-ada-002`: 1536
- OpenAI `text-embedding-3-small`: 1536
- OpenAI `text-embedding-3-large`: 3072
- Sentence-BERT: 384 或 768
- BERT-base: 768

### Q2: 数据存储在哪里？

**A**: 数据存储在您指定的路径：
```rust
let store = LanceDBVectorStore::new("./data/vectors.lance", 1536).await?;
// 数据保存在: ./data/vectors.lance/
```

### Q3: 如何备份数据？

**A**: 直接复制数据目录：
```bash
cp -r ./data/vectors.lance ./backup/vectors.lance
```

### Q4: 支持并发访问吗？

**A**: 嵌入式模式是单进程的，不支持多进程并发。如需并发，请使用 Server 模式。

### Q5: 如何迁移到 Server 模式？

**A**: 
1. 导出数据：使用 `get_vector` 遍历所有向量
2. 配置 Server 模式（PostgreSQL + 向量服务）
3. 导入数据：使用 `add_vectors` 批量插入

### Q6: 性能优化建议？

**A**:
- ✅ 使用批量操作（`add_vectors` 而不是多次单个插入）
- ✅ 合理设置 `top_k`（不要过大）
- ✅ 使用 SSD 存储
- ✅ 定期清理不需要的向量

### Q7: 元数据可以存储什么类型？

**A**: 只支持 `HashMap<String, String>`，所有值必须是字符串：
```rust
// ✅ 正确
let metadata = HashMap::from([
    ("count".to_string(), "100".to_string()),  // 数字转字符串
    ("active".to_string(), "true".to_string()), // 布尔转字符串
]);

// ❌ 错误
let metadata = HashMap::from([
    ("count".to_string(), 100),  // 不支持
    ("active".to_string(), true), // 不支持
]);
```

---

## 下一步

- 📚 查看 [API 文档](../crates/agent-mem-traits/src/storage.rs)
- 🔧 运行 [示例代码](./examples/embedded-mode-demo/)
- 🚀 部署到生产环境
- 📊 监控性能指标

---

## 支持

如有问题，请查看：
- [技术设计文档](../doc/technical-design/memory-systems/mem21.md)
- [测试用例](../crates/agent-mem-storage/src/backends/lancedb_store.rs)
- [GitHub Issues](https://github.com/your-repo/issues)

