# 数据库 Schema 缺失字段演示

此示例演示 AgentMem 数据库 Schema 中新添加的字段功能。

## 新增字段

### 1. `embedding` - 向量嵌入

**用途**: 存储记忆的向量嵌入，用于语义搜索

**类型**:
- PostgreSQL: `JSONB`
- LibSQL: `TEXT` (JSON 格式)

**示例**:
```rust
let embedding_vector = Vector {
    values: vec![0.1, 0.2, 0.3, 0.4, 0.5],
};
memory.embedding = Some(embedding_vector);
```

**应用场景**:
- 语义相似度搜索
- 向量数据库集成
- RAG (Retrieval-Augmented Generation)

### 2. `expires_at` - 过期时间

**用途**: 设置记忆的过期时间，用于自动清理临时记忆

**类型**:
- PostgreSQL: `TIMESTAMPTZ`
- LibSQL: `INTEGER` (Unix 时间戳)

**示例**:
```rust
// 设置1小时后过期
let expires_at = Utc::now().timestamp() + 3600;
memory.set_expiration(expires_at);

// 检查是否过期
if memory.is_expired() {
    println!("记忆已过期");
}
```

**应用场景**:
- 工作记忆自动清理
- 临时会话数据管理
- 缓存失效策略

### 3. `version` - 版本号

**用途**: 实现乐观锁定，防止并发更新冲突

**类型**:
- PostgreSQL: `INTEGER NOT NULL DEFAULT 1`
- LibSQL: `INTEGER NOT NULL DEFAULT 1`

**示例**:
```rust
// 每次更新内容时，版本号自动递增
memory.update_content("新内容".to_string());
println!("当前版本: {}", memory.version); // 版本号 +1
```

**应用场景**:
- 并发控制
- 冲突检测
- 审计追踪

## 运行示例

```bash
cargo run --package database-schema-demo
```

## 输出示例

```
╔══════════════════════════════════════════════════════════════════════╗
║          AgentMem 数据库 Schema 新字段演示                           ║
╚══════════════════════════════════════════════════════════════════════╝

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
1️⃣  演示 embedding 字段（向量嵌入）
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ 创建带有 embedding 的记忆:
   - ID: xxx-xxx-xxx
   - 内容: 这是一段包含向量嵌入的语义记忆
   - Embedding 维度: 5
   - Embedding 前5个值: [0.1, 0.2, 0.3, 0.4, 0.5]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
2️⃣  演示 expires_at 字段（过期时间）
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ 创建带有 expires_at 的工作记忆:
   - ID: xxx-xxx-xxx
   - 内容: 这是一段临时工作记忆，将在1小时后过期
   - 创建时间: 2025-10-12 10:00:00 UTC
   - 过期时间: 2025-10-12 11:00:00 UTC
   - 是否已过期: false

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
3️⃣  演示 version 字段（乐观锁定）
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ 创建带有 version 的记忆:
   - ID: xxx-xxx-xxx
   - 内容: 这是一段支持版本控制的核心记忆
   - 初始版本: 1

📝 执行第一次更新...
   - 新内容: 更新后的核心记忆内容 - 版本 2
   - 新版本: 2

📝 执行第二次更新...
   - 新内容: 再次更新的核心记忆内容 - 版本 3
   - 新版本: 3
```

## 数据库迁移

### 自动迁移

新字段会在运行 `run_migrations()` 时自动添加：

```rust
use agent_mem_core::storage::migrations::run_migrations;

// 运行所有迁移（包括新字段）
run_migrations(&pool).await?;
```

### 手动迁移

如果需要手动应用迁移：

```rust
use agent_mem_core::storage::migrations::migrate_add_missing_fields;

// 仅添加缺失字段
migrate_add_missing_fields(&pool).await?;
```

### 迁移特性

- ✅ **幂等性**: 可以安全地多次运行
- ✅ **向后兼容**: 不影响现有数据
- ✅ **自动索引**: 自动创建性能优化索引

## 技术细节

### PostgreSQL Schema

```sql
ALTER TABLE memories ADD COLUMN embedding JSONB;
ALTER TABLE memories ADD COLUMN expires_at TIMESTAMPTZ;
ALTER TABLE memories ADD COLUMN version INTEGER NOT NULL DEFAULT 1;

-- 性能优化索引
CREATE INDEX idx_memories_expires_at ON memories(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX idx_memories_version ON memories(version);
```

### LibSQL Schema

```sql
ALTER TABLE memories ADD COLUMN embedding TEXT;
ALTER TABLE memories ADD COLUMN expires_at INTEGER;
ALTER TABLE memories ADD COLUMN version INTEGER NOT NULL DEFAULT 1;
```

## 相关文件

- **Schema 定义**: `crates/agent-mem-core/src/storage/migrations.rs`
- **PostgreSQL 实现**: `crates/agent-mem-core/src/storage/postgres.rs`
- **LibSQL 实现**: `crates/agent-mem-core/src/storage/libsql/migrations.rs`
- **Memory 类型**: `crates/agent-mem-core/src/types.rs`

## 下一步

- 集成向量数据库（Qdrant, Milvus, Weaviate）
- 实现基于 embedding 的语义搜索
- 添加自动过期清理任务
- 实现版本冲突解决策略

