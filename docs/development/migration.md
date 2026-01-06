# AgentMem 数据库迁移指南

本指南帮助你从旧版本的 AgentMem (仅支持 PostgreSQL) 迁移到新版本 (支持 LibSQL 和 PostgreSQL)。

---

## 📋 目录

1. [迁移概述](#迁移概述)
2. [迁移前准备](#迁移前准备)
3. [迁移步骤](#迁移步骤)
4. [配置更新](#配置更新)
5. [代码更新](#代码更新)
6. [测试验证](#测试验证)
7. [常见问题](#常见问题)
8. [回滚方案](#回滚方案)

---

## 迁移概述

### 主要变更

**架构变更**:
- ✅ 引入 Repository Trait 抽象层
- ✅ 支持多数据库后端 (LibSQL 默认, PostgreSQL 可选)
- ✅ 通过 Factory 模式创建 repositories
- ✅ 所有路由使用 trait 而非具体实现

**向后兼容性**:
- ✅ PostgreSQL 功能完全保留
- ✅ 现有 API 接口不变
- ✅ 数据模型不变
- ✅ 通过 feature flags 控制编译

### 迁移收益

- 🚀 **零配置启动**: 默认使用 LibSQL，无需外部数据库
- 🔄 **灵活切换**: 通过配置文件或环境变量轻松切换数据库
- 📦 **更小部署**: LibSQL 嵌入式，减少依赖
- ⚡ **更快启动**: LibSQL 启动时间 < 100ms
- 🧪 **更易测试**: 内存数据库支持，测试更快

---

## 迁移前准备

### 1. 备份数据

**PostgreSQL 用户**:
```bash
# 备份整个数据库
pg_dump -U postgres agentmem > agentmem_backup_$(date +%Y%m%d).sql

# 或者只备份数据 (不包括 schema)
pg_dump -U postgres --data-only agentmem > agentmem_data_backup_$(date +%Y%m%d).sql
```

### 2. 检查依赖版本

确保你的项目使用以下最低版本:
```toml
[dependencies]
agent-mem-core = "0.1.0"
agent-mem-server = "0.1.0"
agent-mem-config = "0.1.0"
```

### 3. 了解当前配置

记录当前的数据库配置:
```bash
# 查看当前 PostgreSQL 连接信息
echo $DATABASE_URL

# 查看当前配置文件
cat config.toml
```

---

## 迁移步骤

### 选项 1: 继续使用 PostgreSQL (推荐用于生产环境)

如果你想继续使用 PostgreSQL，迁移非常简单：

#### 步骤 1: 更新依赖

```toml
# Cargo.toml
[dependencies]
agent-mem-server = { version = "0.1.0", features = ["postgres"] }
```

#### 步骤 2: 更新配置

创建或更新 `config.toml`:
```toml
[database]
backend = "postgres"
url = "postgresql://user:password@localhost:5432/agentmem"
auto_migrate = true
```

或使用环境变量:
```bash
export DATABASE_BACKEND=postgres
export DATABASE_URL=postgresql://user:password@localhost:5432/agentmem
```

#### 步骤 3: 运行迁移

```bash
# 编译项目
cargo build --release --features postgres

# 运行服务器 (会自动运行 migrations)
cargo run --release --features postgres
```

**完成！** 你的应用现在使用新架构但仍然连接到 PostgreSQL。

---

### 选项 2: 迁移到 LibSQL (推荐用于开发/测试)

如果你想切换到 LibSQL:

#### 步骤 1: 导出 PostgreSQL 数据

```bash
# 导出为 SQL 格式
pg_dump -U postgres --data-only agentmem > data_export.sql
```

#### 步骤 2: 更新配置

```toml
# config.toml
[database]
backend = "libsql"
url = "./data/agentmem.db"
auto_migrate = true
```

#### 步骤 3: 启动 LibSQL

```bash
# 编译项目 (默认使用 libsql feature)
cargo build --release

# 运行服务器 (会自动创建数据库和表)
cargo run --release
```

#### 步骤 4: 导入数据 (可选)

如果需要导入 PostgreSQL 数据:

```bash
# 使用 LibSQL CLI 导入
libsql ./data/agentmem.db < data_export.sql

# 或者使用 AgentMem 提供的迁移工具 (如果可用)
cargo run --bin migrate-data -- \
  --from postgresql://localhost/agentmem \
  --to ./data/agentmem.db
```

---

### 选项 3: 混合使用 (开发用 LibSQL，生产用 PostgreSQL)

#### 开发环境配置

```toml
# config.dev.toml
[database]
backend = "libsql"
url = "./data/dev.db"
auto_migrate = true
```

```bash
# 启动开发服务器
cargo run -- --config config.dev.toml
```

#### 生产环境配置

```toml
# config.prod.toml
[database]
backend = "postgres"
url = "postgresql://prod-db.example.com:5432/agentmem"
auto_migrate = false

[database.pool]
max_connections = 50
min_connections = 10
```

```bash
# 启动生产服务器
cargo run --release --features postgres -- --config config.prod.toml
```

---

## 配置更新

### 环境变量映射

| 旧环境变量 | 新环境变量 | 说明 |
|-----------|-----------|------|
| `DATABASE_URL` | `DATABASE_URL` | 保持不变 |
| - | `DATABASE_BACKEND` | 新增: "libsql" 或 "postgres" |
| - | `DATABASE_AUTO_MIGRATE` | 新增: "true" 或 "false" |

### 配置文件示例

**旧配置** (仅 PostgreSQL):
```toml
[database]
url = "postgresql://localhost/agentmem"
```

**新配置** (支持多后端):
```toml
[database]
backend = "postgres"  # 或 "libsql"
url = "postgresql://localhost/agentmem"
auto_migrate = true
```

---

## 代码更新

### 如果你直接使用了 Repository

**旧代码**:
```rust
use agent_mem_core::storage::UserRepository;

let user_repo = UserRepository::new(pool.clone());
let user = user_repo.create(&new_user).await?;
```

**新代码**:
```rust
use agent_mem_core::storage::factory::{RepositoryFactory, DatabaseConfig};

let config = DatabaseConfig::default(); // 或从配置文件加载
let repos = RepositoryFactory::create_repositories(&config).await?;
let user = repos.user.create(&new_user).await?;
```

### 如果你使用了 Server

**旧代码**:
```rust
use agent_mem_server::MemoryServer;

let server = MemoryServer::new(pool).await?;
server.run().await?;
```

**新代码** (无需更改):
```rust
use agent_mem_server::MemoryServer;

// Server 内部已自动使用 RepositoryFactory
let server = MemoryServer::new().await?;
server.run().await?;
```

---

## 测试验证

### 1. 编译测试

```bash
# 测试 LibSQL 编译
cargo build --features libsql

# 测试 PostgreSQL 编译
cargo build --features postgres

# 测试默认编译 (应该使用 libsql)
cargo build
```

### 2. 单元测试

```bash
# 运行所有测试
cargo test

# 运行 LibSQL 集成测试
cargo test --test integration_libsql

# 运行 PostgreSQL 测试 (如果有)
cargo test --features postgres
```

### 3. 功能测试

```bash
# 启动服务器
cargo run

# 测试健康检查
curl http://localhost:8080/health

# 测试创建用户
curl -X POST http://localhost:8080/api/v1/users \
  -H "Content-Type: application/json" \
  -d '{"name": "Test User", "organization_id": "org_123"}'

# 测试查询用户
curl http://localhost:8080/api/v1/users/{user_id}
```

### 4. 性能测试

```bash
# 使用 wrk 进行压力测试
wrk -t4 -c100 -d30s http://localhost:8080/api/v1/users

# 检查响应时间
# LibSQL 应该 < 10ms
# PostgreSQL 应该 < 20ms
```

---

## 常见问题

### Q1: 迁移后性能有变化吗？

**A**: 
- LibSQL: 单机性能更好 (2-5ms 延迟)，适合开发和中小规模部署
- PostgreSQL: 企业级功能更强，适合大规模生产环境

### Q2: 可以在运行时切换数据库吗？

**A**: 不可以。数据库后端在启动时确定，需要重启服务器才能切换。

### Q3: LibSQL 支持哪些 PostgreSQL 特性？

**A**: 
- ✅ 支持: CRUD 操作、事务、索引、外键
- ✅ 支持: JSON 存储 (作为 TEXT)
- ❌ 不支持: 复杂的 PostgreSQL 扩展 (如 pgvector)
- ❌ 不支持: 分布式特性

### Q4: 如何在测试中使用内存数据库？

**A**:
```rust
use agent_mem_core::storage::factory::{DatabaseConfig, DatabaseBackend};

let config = DatabaseConfig {
    backend: DatabaseBackend::LibSql,
    url: ":memory:".to_string(),
    auto_migrate: true,
};
let repos = RepositoryFactory::create_repositories(&config).await?;
```

### Q5: 迁移失败如何回滚？

**A**: 参见下一节 "回滚方案"。

---

## 回滚方案

### 如果迁移出现问题

#### 步骤 1: 停止新版本服务

```bash
# 停止服务器
pkill agent-mem-server
```

#### 步骤 2: 恢复旧版本代码

```bash
# 回退到旧版本
git checkout <previous-version-tag>

# 重新编译
cargo build --release
```

#### 步骤 3: 恢复数据库 (如果需要)

```bash
# 恢复 PostgreSQL 备份
psql -U postgres agentmem < agentmem_backup_YYYYMMDD.sql
```

#### 步骤 4: 启动旧版本服务

```bash
cargo run --release
```

### 保留双版本运行

在迁移期间，可以同时运行新旧两个版本:

```bash
# 旧版本 (端口 8080)
OLD_VERSION/cargo run --release

# 新版本 (端口 8081)
NEW_VERSION/cargo run --release -- --port 8081
```

通过负载均衡器逐步切换流量。

---

## 获取帮助

如果遇到问题:

1. 查看日志: `tail -f logs/agentmem.log`
2. 检查 GitHub Issues: https://github.com/louloulin/agentmem/issues
3. 加入 Discord 社区: https://discord.gg/agentmem
4. 发送邮件: support@agentmem.io

---

**文档版本**: 1.0  
**最后更新**: 2025-01-09  
**适用版本**: AgentMem 0.1.0+

