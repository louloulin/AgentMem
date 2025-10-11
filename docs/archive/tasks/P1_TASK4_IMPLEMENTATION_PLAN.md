# P1-4 任务实施计划 - 更新数据库 Schema 添加缺失字段

**任务编号**: P1-4  
**任务名称**: 更新数据库 schema 添加缺失字段  
**优先级**: P1  
**预计工作量**: 1-2 小时  
**依赖关系**: 无

---

## 任务概述

### 背景分析

通过深度代码分析发现：
1. **代码中已使用但数据库缺失的字段**:
   - `embedding: Option<Vec<f32>>` - 在 `hierarchy.rs:919`, `orchestrator/memory_extraction.rs:185` 等处使用
   - `expires_at: Option<i64>` - 在 `hierarchy.rs:922`, `lifecycle.rs:266` 等处使用
   - `version: i32` - 在 `hierarchy.rs:923`, `graph_memory.rs:667` 等处使用

2. **当前数据库表状态**:
   - `episodic_events` 表: 缺少 `embedding`, `expires_at`, `version`
   - `semantic_memory` 表: 缺少 `embedding`, `expires_at`, `version`
   - `procedural_memory` 表: 缺少 `embedding`, `expires_at`, `version`
   - `core_memory` 表: 缺少 `embedding`, `expires_at`, `version`
   - `working_memory` 表: 已有 `expires_at`，缺少 `embedding`, `version`

3. **影响范围**:
   - 向量搜索功能无法使用（embedding 字段缺失）
   - 记忆过期功能无法使用（expires_at 字段缺失）
   - 乐观锁功能无法使用（version 字段缺失）

### 目标

为所有记忆表添加缺失的字段，使数据库 schema 与代码中的数据结构保持一致。

---

## 功能点分解

### 1. 创建数据库迁移脚本

**详细说明**: 创建 SQL 迁移脚本，为所有记忆表添加缺失字段。

**涉及的表**:
- `episodic_events`
- `semantic_memory`
- `procedural_memory`
- `core_memory`
- `working_memory`

**新增字段**:
- `embedding`: `VECTOR(1536)` 或 `REAL[]` (PostgreSQL) / `TEXT` (LibSQL，存储 JSON)
- `expires_at`: `TIMESTAMPTZ` (可为 NULL)
- `version`: `INTEGER NOT NULL DEFAULT 1`

### 2. 更新 PostgreSQL Schema

**详细说明**: 修改 `memory_tables_migration.rs` 中的表创建语句，添加新字段。

**文件位置**: `crates/agent-mem-core/src/storage/memory_tables_migration.rs`

### 3. 更新 LibSQL Schema

**详细说明**: 修改 LibSQL 的迁移脚本，添加新字段。

**文件位置**: `crates/agent-mem-core/src/storage/libsql/migrations.rs`

### 4. 更新 Rust 结构体定义

**详细说明**: 确保所有数据库行结构体（`*Row`）包含新字段。

**涉及的文件**:
- `crates/agent-mem-storage/src/backends/postgres_episodic.rs`
- `crates/agent-mem-storage/src/backends/postgres_semantic.rs`
- `crates/agent-mem-storage/src/backends/postgres_procedural.rs`
- `crates/agent-mem-storage/src/backends/postgres_core.rs`
- `crates/agent-mem-storage/src/backends/postgres_working.rs`

### 5. 更新 CRUD 操作

**详细说明**: 更新 INSERT 和 UPDATE 语句，包含新字段。

**注意事项**:
- `embedding` 字段默认为 NULL
- `expires_at` 字段默认为 NULL
- `version` 字段默认为 1，每次更新时递增

---

## 实施步骤

### 步骤 1: 创建迁移脚本 (15 分钟)

1. 创建 `migrations/20250110_add_missing_fields.sql`
2. 编写 ALTER TABLE 语句添加字段
3. 为每个表添加索引（embedding 使用向量索引，expires_at 使用 B-tree 索引）

**SQL 示例**:
```sql
-- 为 episodic_events 表添加字段
ALTER TABLE episodic_events 
ADD COLUMN IF NOT EXISTS embedding VECTOR(1536),
ADD COLUMN IF NOT EXISTS expires_at TIMESTAMPTZ,
ADD COLUMN IF NOT EXISTS version INTEGER NOT NULL DEFAULT 1;

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_episodic_expires ON episodic_events(expires_at) WHERE expires_at IS NOT NULL;
```

### 步骤 2: 更新 PostgreSQL 迁移代码 (20 分钟)

1. 打开 `crates/agent-mem-core/src/storage/memory_tables_migration.rs`
2. 修改 `create_episodic_events_table()` 函数
3. 修改 `create_semantic_memory_table()` 函数
4. 修改 `create_procedural_memory_table()` 函数
5. 修改 `create_core_memory_table()` 函数
6. 修改 `create_working_memory_table()` 函数
7. 更新 `create_memory_indexes()` 函数，添加新索引

### 步骤 3: 更新 LibSQL 迁移代码 (15 分钟)

1. 打开 `crates/agent-mem-core/src/storage/libsql/migrations.rs`
2. 添加新的迁移版本（version 2）
3. 实现 `migrate_v2()` 函数，添加新字段
4. 更新 `run_migrations()` 函数，调用新迁移

**注意**: LibSQL 不支持 VECTOR 类型，使用 TEXT 存储 JSON 格式的向量

### 步骤 4: 更新数据库行结构体 (20 分钟)

1. 更新 `EpisodicEventRow` 结构体
2. 更新 `SemanticMemoryItemRow` 结构体
3. 更新 `ProceduralMemoryItemRow` 结构体
4. 更新 `CoreMemoryRow` 结构体
5. 更新 `WorkingMemoryItemRow` 结构体

**示例**:
```rust
#[derive(Debug, sqlx::FromRow)]
struct EpisodicEventRow {
    id: String,
    // ... 现有字段
    embedding: Option<Vec<f32>>,  // 新增
    expires_at: Option<DateTime<Utc>>,  // 新增
    version: i32,  // 新增
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
```

### 步骤 5: 更新 INSERT 语句 (15 分钟)

1. 修改 `create_episode()` 方法的 INSERT 语句
2. 修改 `create_item()` 方法的 INSERT 语句（semantic, procedural, core, working）
3. 添加 `embedding`, `expires_at`, `version` 字段

**示例**:
```rust
sqlx::query(
    r#"
    INSERT INTO episodic_events (
        id, organization_id, user_id, agent_id,
        occurred_at, event_type, actor, summary, details,
        importance_score, metadata,
        embedding, expires_at, version,  -- 新增字段
        created_at, updated_at
    ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
    "#,
)
.bind(&episode.id)
// ... 其他绑定
.bind(&episode.embedding)  // 新增
.bind(&episode.expires_at)  // 新增
.bind(&episode.version)  // 新增
.bind(&episode.created_at)
.bind(&episode.updated_at)
```

### 步骤 6: 更新 UPDATE 语句 (15 分钟)

1. 修改 `update_episode()` 方法的 UPDATE 语句
2. 修改 `update_item()` 方法的 UPDATE 语句
3. 实现乐观锁：WHERE version = $old_version，SET version = version + 1

**示例**:
```rust
sqlx::query(
    r#"
    UPDATE episodic_events 
    SET summary = $1, details = $2, importance_score = $3,
        embedding = $4, expires_at = $5,  -- 新增字段
        version = version + 1,  -- 乐观锁
        updated_at = $6
    WHERE id = $7 AND user_id = $8 AND version = $9  -- 乐观锁检查
    "#,
)
```

### 步骤 7: 运行迁移测试 (10 分钟)

1. 启动测试数据库
2. 运行迁移脚本
3. 验证表结构正确
4. 验证索引创建成功

### 步骤 8: 运行集成测试 (10 分钟)

1. 运行所有 Agent 的真实存储测试
2. 验证 INSERT 和 UPDATE 操作正常
3. 验证新字段可以正确读写

---

## 验收标准

- [ ] 所有 5 个记忆表都添加了 `embedding`, `expires_at`, `version` 字段
- [ ] PostgreSQL 迁移脚本创建成功
- [ ] LibSQL 迁移脚本创建成功
- [ ] 所有数据库行结构体包含新字段
- [ ] 所有 INSERT 语句包含新字段
- [ ] 所有 UPDATE 语句包含新字段并实现乐观锁
- [ ] 为 `expires_at` 创建了索引
- [ ] 为 `embedding` 创建了向量索引（PostgreSQL）
- [ ] 所有现有测试通过（21/21）
- [ ] 代码编译无错误
- [ ] 迁移脚本可以在空数据库和现有数据库上运行

---

## 技术细节

### 1. 向量字段处理

**PostgreSQL**:
```sql
-- 需要 pgvector 扩展
CREATE EXTENSION IF NOT EXISTS vector;

-- 使用 VECTOR 类型
embedding VECTOR(1536)

-- 创建向量索引
CREATE INDEX idx_episodic_embedding ON episodic_events 
USING ivfflat (embedding vector_cosine_ops) 
WITH (lists = 100);
```

**LibSQL**:
```sql
-- 使用 TEXT 存储 JSON
embedding TEXT

-- 在应用层序列化/反序列化
-- Rust: serde_json::to_string(&vec) / serde_json::from_str(&text)
```

### 2. 乐观锁实现

```rust
// UPDATE 时检查 version
let result = sqlx::query(
    "UPDATE table SET ..., version = version + 1 WHERE id = $1 AND version = $2"
)
.bind(id)
.bind(current_version)
.execute(pool)
.await?;

// 检查是否更新成功
if result.rows_affected() == 0 {
    return Err(Error::ConcurrentModification);
}
```

### 3. 过期时间处理

```rust
// 设置过期时间
let expires_at = Some(Utc::now() + Duration::hours(24));

// 查询时过滤过期记忆
WHERE (expires_at IS NULL OR expires_at > NOW())
```

---

## 风险和注意事项

### 风险 1: 向量扩展未安装

**问题**: PostgreSQL 可能没有安装 pgvector 扩展

**解决方案**: 
- 在迁移脚本中添加 `CREATE EXTENSION IF NOT EXISTS vector`
- 如果扩展不可用，使用 `REAL[]` 类型作为降级方案

### 风险 2: 现有数据迁移

**问题**: 现有数据没有 embedding 和 version 字段

**解决方案**:
- 使用 `ALTER TABLE ADD COLUMN IF NOT EXISTS`
- 设置合理的默认值（embedding = NULL, version = 1）
- 不影响现有数据

### 风险 3: LibSQL 兼容性

**问题**: LibSQL 不支持 VECTOR 类型

**解决方案**:
- 使用 TEXT 类型存储 JSON 格式的向量
- 在应用层处理序列化/反序列化

---

## 预期结果

完成后，数据库 schema 将与代码中的数据结构完全一致，支持：
- ✅ 向量搜索（embedding 字段）
- ✅ 记忆过期（expires_at 字段）
- ✅ 乐观锁（version 字段）
- ✅ 向后兼容（现有数据不受影响）

---

## 参考资料

- 现有迁移脚本: `migrations/20251007_create_*.sql`
- PostgreSQL 迁移代码: `crates/agent-mem-core/src/storage/memory_tables_migration.rs`
- LibSQL 迁移代码: `crates/agent-mem-core/src/storage/libsql/migrations.rs`
- 数据结构定义: `crates/agent-mem-core/src/hierarchy.rs`

