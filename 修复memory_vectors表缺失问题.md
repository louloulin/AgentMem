# 修复 memory_vectors 表缺失问题

## 🔴 问题描述

**错误信息**:
```
ERROR Failed to search memories: Storage error: Table 'memory_vectors' does not exist. Use add_vectors to create it.
```

**发生场景**: 向量搜索时（嵌入式模式）

**根本原因**: LibSQL 迁移脚本中缺少 `memory_vectors` 表的创建语句

---

## 🔍 问题分析

### 1. 当前迁移状态

查看 `crates/agent-mem-core/src/storage/libsql/migrations.rs`，发现：

**已有的表**:
- ✅ organizations
- ✅ users
- ✅ agents
- ✅ messages
- ✅ blocks
- ✅ tools
- ✅ **memories** (主表，存储记忆内容)
- ✅ api_keys
- ✅ junction tables
- ✅ memory_associations
- ✅ learning_feedback

**缺失的表**:
- ❌ **memory_vectors** (向量索引表，用于向量搜索)

### 2. 向量搜索架构

```
记忆存储架构
├── memories 表（主表）
│   └── content, metadata, embedding (JSON存储)
│
└── memory_vectors 表（向量索引）❌ 缺失
    └── id, memory_id, vector, dimension
    └── 用于高效向量相似度搜索
```

### 3. 为什么需要 memory_vectors 表？

**memories 表的 embedding 字段**:
- 存储为 JSON/TEXT 格式
- 不适合高效的向量相似度计算

**memory_vectors 表**:
- 专门的向量存储结构
- 支持向量索引（如 IVFFlat, HNSW）
- 提供高效的 K-NN 搜索

---

## 🛠️ 解决方案

### 方案 A: 添加迁移脚本（推荐）

在 `migrations.rs` 中添加新的迁移：

```rust
// 在 run_migrations() 函数中添加
run_migration(
    &conn_guard,
    14,  // 新的迁移版本号
    "create_memory_vectors",
    create_memory_vectors_table(&conn_guard),
)
.await?;

// 添加创建表的函数
async fn create_memory_vectors_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE memory_vectors (
            id TEXT PRIMARY KEY,
            memory_id TEXT NOT NULL,
            vector BLOB NOT NULL,
            dimension INTEGER NOT NULL,
            created_at INTEGER NOT NULL,
            FOREIGN KEY (memory_id) REFERENCES memories(id) ON DELETE CASCADE
        )",
        (),
    )
    .await
    .map_err(|e| {
        AgentMemError::StorageError(format!("Failed to create memory_vectors table: {e}"))
    })?;

    // 创建索引以加速查询
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_memory_vectors_memory_id ON memory_vectors(memory_id)",
        (),
    )
    .await
    .map_err(|e| AgentMemError::StorageError(format!("Failed to create index: {e}")))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_memory_vectors_dimension ON memory_vectors(dimension)",
        (),
    )
    .await
    .map_err(|e| AgentMemError::StorageError(format!("Failed to create index: {e}")))?;

    Ok(())
}
```

### 方案 B: 手动创建表（临时）

如果无法重启服务，可以手动执行 SQL：

```bash
# 连接到数据库
sqlite3 /path/to/agentmem.db

# 创建表
CREATE TABLE memory_vectors (
    id TEXT PRIMARY KEY,
    memory_id TEXT NOT NULL,
    vector BLOB NOT NULL,
    dimension INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (memory_id) REFERENCES memories(id) ON DELETE CASCADE
);

# 创建索引
CREATE INDEX idx_memory_vectors_memory_id ON memory_vectors(memory_id);
CREATE INDEX idx_memory_vectors_dimension ON memory_vectors(dimension);
```

### 方案 C: 使用 add_vectors 自动创建（不推荐）

错误信息提示 "Use add_vectors to create it"，但这种方式：
- ❌ 不规范（绕过迁移系统）
- ❌ 可能导致表结构不一致
- ❌ 难以追踪和维护

---

## 📝 实施步骤

### 步骤 1: 修改迁移脚本

编辑文件：`crates/agent-mem-core/src/storage/libsql/migrations.rs`

1. 在 `run_migrations()` 中添加第 14 号迁移
2. 实现 `create_memory_vectors_table()` 函数
3. 更新测试中的迁移计数（从 13 改为 14）

### 步骤 2: 重新编译

```bash
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen
cargo build --release --bin agent-mem-server --features lumosai
```

### 步骤 3: 停止并重启服务

```bash
# 停止旧服务
pkill -f agent-mem-server

# 启动新服务（会自动运行迁移）
./start_server_no_auth.sh
```

### 步骤 4: 验证

```bash
# 检查表是否创建
sqlite3 data/agentmem.db "SELECT name FROM sqlite_master WHERE type='table' AND name='memory_vectors';"

# 应该输出: memory_vectors
```

---

## 🎯 预期结果

修复后：
- ✅ `memory_vectors` 表存在
- ✅ 向量搜索正常工作
- ✅ 记忆检索功能恢复
- ✅ Chat 对话可以访问历史记忆

---

## 🔧 完整的迁移代码

将以下代码添加到 `migrations.rs`:

```rust
// 在 run_migrations() 函数的第 108 行后添加:
    run_migration(
        &conn_guard,
        14,
        "create_memory_vectors",
        create_memory_vectors_table(&conn_guard),
    )
    .await?;

// 在文件末尾的 init_default_data() 函数前添加:

/// Create memory_vectors table for vector search
async fn create_memory_vectors_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE memory_vectors (
            id TEXT PRIMARY KEY,
            memory_id TEXT NOT NULL,
            vector BLOB NOT NULL,
            dimension INTEGER NOT NULL,
            created_at INTEGER NOT NULL,
            FOREIGN KEY (memory_id) REFERENCES memories(id) ON DELETE CASCADE
        )",
        (),
    )
    .await
    .map_err(|e| {
        AgentMemError::StorageError(format!("Failed to create memory_vectors table: {e}"))
    })?;

    // Create indexes for performance
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_memory_vectors_memory_id ON memory_vectors(memory_id)",
        (),
    )
    .await
    .map_err(|e| AgentMemError::StorageError(format!("Failed to create index: {e}")))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_memory_vectors_dimension ON memory_vectors(dimension)",
        (),
    )
    .await
    .map_err(|e| AgentMemError::StorageError(format!("Failed to create index: {e}")))?;

    Ok(())
}

// 更新测试中的迁移计数:
// Line 696 和 719: 将 13 改为 14
assert_eq!(count, 14); // 14 migrations (including memory_vectors)
```

---

## ⚠️ 注意事项

1. **迁移是幂等的**: 可以安全地多次运行，不会重复创建表
2. **外键约束**: memory_vectors 通过 FOREIGN KEY 关联到 memories 表
3. **级联删除**: 删除 memory 时会自动删除关联的 vectors
4. **向量维度**: dimension 字段记录向量维度，确保一致性

---

## 🚀 快速修复（紧急）

如果需要立即修复而不重新编译，可以直接在数据库中执行：

```bash
# 1. 定位数据库文件
DB_PATH="/Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen/data/agentmem.db"

# 2. 执行 SQL
sqlite3 "$DB_PATH" <<EOF
CREATE TABLE IF NOT EXISTS memory_vectors (
    id TEXT PRIMARY KEY,
    memory_id TEXT NOT NULL,
    vector BLOB NOT NULL,
    dimension INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (memory_id) REFERENCES memories(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_memory_vectors_memory_id ON memory_vectors(memory_id);
CREATE INDEX IF NOT EXISTS idx_memory_vectors_dimension ON memory_vectors(dimension);

-- 记录迁移
INSERT INTO _migrations (id, name, applied_at) 
VALUES (14, 'create_memory_vectors', strftime('%s', 'now'))
ON CONFLICT DO NOTHING;
EOF

# 3. 重启服务
pkill -f agent-mem-server
./start_server_no_auth.sh
```

---

## 📊 验证清单

- [ ] memory_vectors 表已创建
- [ ] 相关索引已创建
- [ ] 迁移记录已添加（_migrations 表）
- [ ] 服务重启成功
- [ ] 向量搜索不再报错
- [ ] Chat 功能正常（能检索记忆）

---

## 📞 故障排查

### 如果修复后仍然报错

1. **检查表是否创建**:
   ```bash
   sqlite3 data/agentmem.db ".schema memory_vectors"
   ```

2. **检查迁移记录**:
   ```bash
   sqlite3 data/agentmem.db "SELECT * FROM _migrations WHERE id=14;"
   ```

3. **查看服务日志**:
   ```bash
   tail -f backend-no-auth.log | grep -i vector
   ```

4. **完全重建数据库**（⚠️ 会丢失数据）:
   ```bash
   rm data/agentmem.db
   ./start_server_no_auth.sh  # 会重新创建并运行所有迁移
   ```

---

**状态**: 🔴 待修复  
**优先级**: 🔥 高（影响核心记忆功能）  
**预计修复时间**: 10-15 分钟
