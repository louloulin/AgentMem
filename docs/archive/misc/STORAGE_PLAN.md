# AgentMem 存储方案完整计划

**日期**: 2025-10-08  
**目标**: 实现灵活的多存储后端支持，默认使用嵌入式数据库  
**状态**: 设计阶段

---

## 🎯 核心理念

### 设计原则

1. **零配置启动** - 默认使用嵌入式数据库，无需外部依赖
2. **渐进式增强** - 支持从嵌入式升级到分布式
3. **统一接口** - 所有存储后端使用相同的 Trait
4. **性能优先** - 针对不同场景选择最优存储

---

## 📊 存储架构

### 三层架构

```
┌─────────────────────────────────────────┐
│         Application Layer               │
│  (MemoryManager, SimpleMemory)          │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│         Storage Trait Layer             │
│  (MemoryStore, VectorStore, GraphStore) │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│      Storage Implementation Layer       │
│  (LibSQL, LanceDB, PostgreSQL, etc.)    │
└─────────────────────────────────────────┘
```

---

## 🚀 默认方案：嵌入式存储

### 方案 A: LibSQL + LanceDB (推荐) ⭐⭐⭐⭐⭐

**组合**:
- **LibSQL** - 结构化数据（元数据、关系）
- **LanceDB** - 向量数据（嵌入、搜索）

**优点**:
- ✅ 零配置 - 无需外部数据库
- ✅ 嵌入式 - 单文件部署
- ✅ 高性能 - 本地访问
- ✅ SQLite 兼容 - 成熟稳定
- ✅ 向量搜索 - 原生支持
- ✅ 跨平台 - 支持所有平台

**使用场景**:
- 开发和测试
- 单机部署
- 边缘计算
- 桌面应用
- 原型验证

**数据存储**:
```
~/.agentmem/
├── data.db          # LibSQL 数据库文件
├── vectors.lance/   # LanceDB 向量数据
└── config.toml      # 配置文件
```

---

## 📋 存储后端对比

### 结构化数据存储

| 后端 | 类型 | 部署 | 性能 | 扩展性 | 推荐度 |
|------|------|------|------|--------|--------|
| **LibSQL** | 嵌入式 | 零配置 | 高 | 单机 | ⭐⭐⭐⭐⭐ (默认) |
| **SQLite** | 嵌入式 | 零配置 | 高 | 单机 | ⭐⭐⭐⭐ |
| **PostgreSQL** | 服务器 | 需配置 | 高 | 分布式 | ⭐⭐⭐⭐⭐ (生产) |
| **MySQL** | 服务器 | 需配置 | 中 | 分布式 | ⭐⭐⭐ |
| **InMemory** | 内存 | 零配置 | 极高 | 无 | ⭐⭐⭐ (测试) |

### 向量数据存储

| 后端 | 类型 | 部署 | 性能 | 扩展性 | 推荐度 |
|------|------|------|------|--------|--------|
| **LanceDB** | 嵌入式 | 零配置 | 高 | 单机 | ⭐⭐⭐⭐⭐ (默认) |
| **Qdrant** | 服务器 | 需配置 | 极高 | 分布式 | ⭐⭐⭐⭐⭐ (生产) |
| **Milvus** | 服务器 | 需配置 | 极高 | 分布式 | ⭐⭐⭐⭐ |
| **Weaviate** | 服务器 | 需配置 | 高 | 分布式 | ⭐⭐⭐⭐ |
| **Chroma** | 嵌入式 | 零配置 | 中 | 单机 | ⭐⭐⭐ |

### 图数据存储

| 后端 | 类型 | 部署 | 性能 | 扩展性 | 推荐度 |
|------|------|------|------|--------|--------|
| **LibSQL (JSON)** | 嵌入式 | 零配置 | 中 | 单机 | ⭐⭐⭐⭐ (默认) |
| **Neo4j** | 服务器 | 需配置 | 极高 | 分布式 | ⭐⭐⭐⭐⭐ (生产) |
| **Memgraph** | 服务器 | 需配置 | 极高 | 分布式 | ⭐⭐⭐⭐ |
| **ArangoDB** | 服务器 | 需配置 | 高 | 分布式 | ⭐⭐⭐ |

---

## 🏗️ 实施计划

### Phase 1: 嵌入式存储 (本周) ⭐⭐⭐⭐⭐

**目标**: 实现 LibSQL + LanceDB 默认方案

**任务**:
1. ✅ 设计存储 Trait
2. ⏳ 实现 LibSQL 后端
3. ⏳ 实现 LanceDB 后端
4. ⏳ 集成到 MemoryManager
5. ⏳ 编写测试
6. ⏳ 更新文档

**依赖**:
```toml
[dependencies]
libsql = "0.3"
lancedb = "0.4"
```

**预计时间**: 3 天

---

### Phase 2: PostgreSQL 支持 (下周)

**目标**: 支持 PostgreSQL 作为生产后端

**任务**:
1. ⏳ 修复 SQLx 问题
2. ⏳ 实现 PostgreSQL 后端
3. ⏳ 迁移工具
4. ⏳ 性能优化
5. ⏳ 文档

**预计时间**: 2 天

---

### Phase 3: 向量数据库支持 (2 周后)

**目标**: 支持 Qdrant 等专业向量数据库

**任务**:
1. ⏳ Qdrant 集成
2. ⏳ Milvus 集成
3. ⏳ 性能对比
4. ⏳ 迁移工具

**预计时间**: 3 天

---

### Phase 4: 图数据库支持 (3 周后)

**目标**: 支持 Neo4j 等图数据库

**任务**:
1. ⏳ Neo4j 集成
2. ⏳ Memgraph 集成
3. ⏳ 图查询优化

**预计时间**: 2 天

---

## 💻 技术实现

### 1. 存储 Trait 定义

```rust
// crates/agent-mem-traits/src/storage.rs

/// 结构化数据存储
#[async_trait]
pub trait MemoryStore: Send + Sync {
    /// 创建记忆
    async fn create(&self, memory: Memory) -> Result<String>;
    
    /// 获取记忆
    async fn get(&self, id: &str) -> Result<Option<Memory>>;
    
    /// 更新记忆
    async fn update(&self, memory: Memory) -> Result<()>;
    
    /// 删除记忆
    async fn delete(&self, id: &str) -> Result<bool>;
    
    /// 搜索记忆
    async fn search(&self, query: MemoryQuery) -> Result<Vec<Memory>>;
    
    /// 批量操作
    async fn batch_create(&self, memories: Vec<Memory>) -> Result<Vec<String>>;
}

/// 向量数据存储
#[async_trait]
pub trait VectorStore: Send + Sync {
    /// 添加向量
    async fn add_vector(&self, id: &str, vector: Vec<f32>, metadata: HashMap<String, String>) -> Result<()>;
    
    /// 向量搜索
    async fn search_vectors(&self, query_vector: Vec<f32>, limit: usize) -> Result<Vec<VectorSearchResult>>;
    
    /// 删除向量
    async fn delete_vector(&self, id: &str) -> Result<bool>;
}

/// 图数据存储
#[async_trait]
pub trait GraphStore: Send + Sync {
    /// 添加节点
    async fn add_node(&self, node: GraphNode) -> Result<String>;
    
    /// 添加边
    async fn add_edge(&self, edge: GraphEdge) -> Result<String>;
    
    /// 图查询
    async fn query_graph(&self, query: GraphQuery) -> Result<Vec<GraphNode>>;
}
```

### 2. LibSQL 实现

```rust
// crates/agent-mem-storage/src/libsql.rs

use libsql::{Database, Connection};

pub struct LibSQLStore {
    db: Database,
    conn: Connection,
}

impl LibSQLStore {
    pub async fn new(path: &str) -> Result<Self> {
        let db = Database::open(path).await?;
        let conn = db.connect()?;
        
        // 创建表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS memories (
                id TEXT PRIMARY KEY,
                agent_id TEXT NOT NULL,
                user_id TEXT,
                content TEXT NOT NULL,
                memory_type TEXT NOT NULL,
                importance REAL NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                metadata TEXT
            )",
            (),
        ).await?;
        
        Ok(Self { db, conn })
    }
}

#[async_trait]
impl MemoryStore for LibSQLStore {
    async fn create(&self, memory: Memory) -> Result<String> {
        self.conn.execute(
            "INSERT INTO memories (id, agent_id, user_id, content, memory_type, importance, created_at, updated_at, metadata)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            (
                &memory.id,
                &memory.agent_id,
                &memory.user_id,
                &memory.content,
                &memory.memory_type.to_string(),
                memory.importance,
                memory.created_at.timestamp(),
                memory.updated_at.timestamp(),
                serde_json::to_string(&memory.metadata)?,
            ),
        ).await?;
        
        Ok(memory.id)
    }
    
    async fn search(&self, query: MemoryQuery) -> Result<Vec<Memory>> {
        let mut sql = "SELECT * FROM memories WHERE 1=1".to_string();
        let mut params = Vec::new();
        
        if let Some(agent_id) = &query.agent_id {
            sql.push_str(" AND agent_id = ?");
            params.push(agent_id.clone());
        }
        
        if let Some(user_id) = &query.user_id {
            sql.push_str(" AND user_id = ?");
            params.push(user_id.clone());
        }
        
        sql.push_str(" ORDER BY created_at DESC LIMIT ?");
        params.push(query.limit.unwrap_or(10).to_string());
        
        let rows = self.conn.query(&sql, params).await?;
        
        // 转换为 Memory 对象
        let memories = rows.into_iter()
            .map(|row| Memory::from_row(row))
            .collect::<Result<Vec<_>>>()?;
        
        Ok(memories)
    }
}
```

### 3. LanceDB 实现

```rust
// crates/agent-mem-storage/src/lancedb.rs

use lancedb::{Connection, Table};

pub struct LanceDBStore {
    conn: Connection,
    table: Table,
}

impl LanceDBStore {
    pub async fn new(path: &str) -> Result<Self> {
        let conn = lancedb::connect(path).await?;
        
        // 创建表
        let table = conn.create_table(
            "vectors",
            vec![
                ("id", DataType::Utf8),
                ("vector", DataType::FixedSizeList(Box::new(DataType::Float32), 1536)),
                ("metadata", DataType::Utf8),
            ],
        ).await?;
        
        Ok(Self { conn, table })
    }
}

#[async_trait]
impl VectorStore for LanceDBStore {
    async fn add_vector(&self, id: &str, vector: Vec<f32>, metadata: HashMap<String, String>) -> Result<()> {
        self.table.add(vec![
            (id, vector, serde_json::to_string(&metadata)?),
        ]).await?;
        
        Ok(())
    }
    
    async fn search_vectors(&self, query_vector: Vec<f32>, limit: usize) -> Result<Vec<VectorSearchResult>> {
        let results = self.table
            .search(&query_vector)
            .limit(limit)
            .execute()
            .await?;
        
        Ok(results.into_iter()
            .map(|r| VectorSearchResult {
                id: r.id,
                score: r.distance,
                metadata: serde_json::from_str(&r.metadata).unwrap_or_default(),
            })
            .collect())
    }
}
```

---

## 📦 配置示例

### 默认配置（嵌入式）

```toml
# config.toml

[storage]
# 默认使用嵌入式存储
backend = "embedded"

[storage.embedded]
# LibSQL 数据文件
data_path = "~/.agentmem/data.db"

# LanceDB 向量数据
vector_path = "~/.agentmem/vectors.lance"

# 自动创建目录
auto_create = true
```

### 生产配置（PostgreSQL + Qdrant）

```toml
[storage]
backend = "distributed"

[storage.distributed]
# PostgreSQL
database_url = "postgresql://user:pass@localhost/agentmem"

# Qdrant
vector_url = "http://localhost:6333"

# Neo4j
graph_url = "bolt://localhost:7687"
graph_user = "neo4j"
graph_password = "password"
```

---

## 🧪 测试计划

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_libsql_create() {
        let store = LibSQLStore::new(":memory:").await.unwrap();
        let memory = Memory::new(...);
        let id = store.create(memory).await.unwrap();
        assert!(!id.is_empty());
    }
    
    #[tokio::test]
    async fn test_lancedb_search() {
        let store = LanceDBStore::new("test.lance").await.unwrap();
        let vector = vec![0.1; 1536];
        store.add_vector("test", vector.clone(), HashMap::new()).await.unwrap();
        
        let results = store.search_vectors(vector, 10).await.unwrap();
        assert_eq!(results.len(), 1);
    }
}
```

### 集成测试

```rust
#[tokio::test]
async fn test_embedded_storage_integration() {
    let manager = MemoryManager::with_embedded_storage().await.unwrap();
    
    // 添加记忆
    let id = manager.add_memory(...).await.unwrap();
    
    // 搜索记忆
    let results = manager.search(...).await.unwrap();
    assert!(!results.is_empty());
}
```

---

## 📊 性能目标

| 操作 | 嵌入式 | PostgreSQL | 目标 |
|------|--------|------------|------|
| **写入** | < 1ms | < 5ms | < 10ms |
| **读取** | < 0.5ms | < 2ms | < 5ms |
| **向量搜索** | < 10ms | < 20ms | < 50ms |
| **批量写入** | < 10ms/100条 | < 50ms/100条 | < 100ms/100条 |

---

## 🎯 成功指标

- ✅ 零配置启动
- ✅ 单文件部署
- ✅ 跨平台支持
- ✅ 性能达标
- ✅ 100% 测试覆盖
- ✅ 完整文档

---

**下一步**: 开始实现 LibSQL + LanceDB 嵌入式存储方案！

