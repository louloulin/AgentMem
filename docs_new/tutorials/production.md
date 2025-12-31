# 生产部署指南

学习如何将 AgentMem 部署到生产环境，并进行性能优化和监控。

## 目录

- [部署前准备](#部署前准备)
- [数据库配置](#数据库配置)
- [性能优化](#性能优化)
- [监控和日志](#监控和日志)
- [备份和恢复](#备份和恢复)
- [安全最佳实践](#安全最佳实践)
- [扩展性](#扩展性)
- [故障排查](#故障排查)

## 部署前准备

### 1. 环境检查

```bash
# 检查 Rust 版本
rustc --version  # 需要 >= 1.70

# 检查可用资源
free -h
df -h

# 检查网络连接
ping -c 3 google.com
```

### 2. 配置文件

创建 `agentmem.toml`:

```toml
# AgentMem 生产配置

[server]
host = "0.0.0.0"
port = 8080
workers = 4

[storage]
# PostgreSQL 配置
type = "postgresql"
url = "postgresql://user:password@localhost:5432/agentmem"
max_connections = 100
connection_timeout = 30

# 向量数据库配置
[storage.vector]
type = "pgvector"  # 或 "qdrant", "pinecone"
url = "postgresql://user:password@localhost:5432/agentmem"
dimension = 1536

# Redis 缓存配置
[storage.cache]
type = "redis"
url = "redis://localhost:6379"
max_memory = "2gb"
eviction_policy = "allkeys-lru"

[llm]
provider = "openai"
model = "gpt-4"
api_key = "${OPENAI_API_KEY}"
max_tokens = 2000
timeout = 30

[embeddings]
provider = "openai"
model = "text-embedding-3-small"
dimensions = 1536
batch_size = 100

[intelligent]
enabled = true
fact_extraction = true
deduplication = true
conflict_resolution = true

[performance]
cache_enabled = true
cache_size = 10000
search_cache_ttl = 300
batch_operations = true
max_concurrent_requests = 100

[logging]
level = "info"
format = "json"
outputs = ["stdout", "file"]
file_path = "/var/log/agentmem/app.log"
rotation = "daily"
retention = "30d"

[monitoring]
enabled = true
metrics_port = 9090
health_check_interval = 30
```

### 3. 环境变量

```bash
# .env
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-...

DATABASE_URL=postgresql://user:password@localhost:5432/agentmem
REDIS_URL=redis://localhost:6379

# JWT 密钥
JWT_SECRET=your-secret-key-here

# 加密密钥
ENCRYPTION_KEY=your-encryption-key-here

# 日志级别
LOG_LEVEL=info

# 环境
ENVIRONMENT=production
```

## 数据库配置

### PostgreSQL

```sql
-- 创建数据库和用户
CREATE DATABASE agentmem;
CREATE USER agentmem_user WITH PASSWORD 'your_password';
GRANT ALL PRIVILEGES ON DATABASE agentmem TO agentmem_user;

-- 连接到数据库
\c agentmem

-- 创建扩展
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "vector";

-- 创建表
CREATE TABLE memories (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    content TEXT NOT NULL,
    memory_type VARCHAR(50) NOT NULL,
    agent_id VARCHAR(255) NOT NULL,
    session_id UUID,
    metadata JSONB,
    embedding VECTOR(1536),
    importance FLOAT DEFAULT 0.5,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- 创建索引
CREATE INDEX idx_memories_agent_id ON memories(agent_id);
CREATE INDEX idx_memories_session_id ON memories(session_id);
CREATE INDEX idx_memories_type ON memories(memory_type);
CREATE INDEX idx_memories_created_at ON memories(created_at DESC);
CREATE INDEX idx_memories_embedding ON memories USING ivfflat (embedding vector_cosine_ops) WITH (lists = 100);

-- 性能优化
ALTER TABLE memories SET (autovacuum_vacuum_scale_factor = 0.1);
ALTER TABLE memories SET (autovacuum_analyze_scale_factor = 0.05);
```

### Redis 配置

```conf
# redis.conf
maxmemory 2gb
maxmemory-policy allkeys-lru
save 900 1
save 300 10
save 60 10000
appendonly yes
```

### 连接池配置

```rust
use agentmem::Memory;

let memory = Memory::builder()
    .with_storage("postgresql://user:password@localhost:5432/agentmem")
    .connection_pool(
        ConnectionPoolOptions {
            max_size: 100,
            min_idle: 10,
            connection_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(1800),
        }
    )
    .build()
    .await?;
```

## 性能优化

### 1. 批量操作

```rust
// ❌ 慢：逐条添加
for item in items {
    memory.add(item).await?;
}

// ✅ 快：批量添加
memory.add_batch(items, None, MemoryType::SEMANTIC).await?;
```

### 2. 异步处理

```rust
use tokio::task::JoinSet;

let mut tasks = JoinSet::new();

for item in items {
    let memory_clone = memory.clone();
    tasks.spawn(async move {
        memory_clone.add(item).await
    });
}

while let Some(result) = tasks.join_next().await {
    result??;
}
```

### 3. 缓存策略

```rust
let memory = Memory::builder()
    // 向量缓存
    .embedding_cache(
        CacheOptions {
            max_size: 10000,
            ttl: Duration::from_secs(3600),
        }
    )
    // 搜索缓存
    .search_cache(
        CacheOptions {
            max_size: 5000,
            ttl: Duration::from_secs(300),
        }
    )
    // LLM 响应缓存
    .llm_cache(
        CacheOptions {
            max_size: 1000,
            ttl: Duration::from_secs(86400),
        }
    )
    .build()
    .await?;
```

### 4. 索引优化

```sql
-- 创建复合索引
CREATE INDEX idx_memories_agent_type
ON memories(agent_id, memory_type);

-- 创建部分索引
CREATE INDEX idx_memories_important
ON memories(agent_id, created_at DESC)
WHERE importance > 0.8;

-- 创建表达式索引
CREATE INDEX idx_memories_content_length
ON memories(LENGTH(content));
```

### 5. 查询优化

```rust
// ✅ 使用特定查询
let results = memory.get_all(
    GetAllOptions::builder()
        .agent_id(Some(agent_id))
        .memory_type(Some(MemoryType::SEMANTIC))
        .limit(100)
        .build()
).await?;

// ❌ 避免全表扫描
let results = memory.get_all(GetAllOptions::default()).await?;
```

## 监控和日志

### 1. Prometheus 指标

```rust
use agentmem::observability::MetricsExporter;

let exporter = MetricsExporter::new()
    .with_prometheus("0.0.0.0:9090")
    .start()
    .await?;

// 自动收集指标
// - 内存添加速率
// - 搜索延迟
// - 缓存命中率
// - 数据库连接数
// - 错误率
```

### 2. 健康检查

```rust
use agentmem::health::HealthChecker;

let health_checker = HealthChecker::new(memory.clone());

// 启动健康检查端点
axum::Router::new()
    .route("/health", axum::get(health_check_handler))
    .with_state(memory.clone());

async fn health_check_handler(
    State(memory): State<Memory>
) -> Json<HealthStatus> {
    let status = memory.health_check().await;

    Json(HealthStatus {
        status: if status.healthy { "ok" } else { "error" },
        checks: status.checks,
        timestamp: Utc::now(),
    })
}
```

### 3. 结构化日志

```rust
use tracing::{info, warn, error};
use tracing_subscriber;

// 初始化日志
tracing_subscriber::fmt()
    .json()
    .with_max_level(tracing::Level::INFO)
    .init();

// 记录日志
info!(
    agent_id = %agent_id,
    memory_count = memories.len(),
    "Memories retrieved successfully"
);

warn!(
    agent_id = %agent_id,
    reason = "high_latency",
    latency_ms = 150,
    "Query took longer than expected"
);

error!(
    error = %error,
    agent_id = %agent_id,
    "Failed to add memory"
);
```

### 4. 分布式追踪

```rust
use opentelemetry::trace::TraceContextExt;

let tracer = opentelemetry::global::tracer("agentmem");

#[tracing::instrument(skip(memory))]
async fn add_memory_with_tracing(
    memory: &Memory,
    content: &str
) -> Result<()> {
    let span = tracer.start("add_memory");
    let cx = opentelemetry::Context::current_with_span(span);

    // ... 操作 ...

    Ok(())
}
```

## 备份和恢复

### 1. 数据库备份

```bash
#!/bin/bash
# backup.sh

DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/backups/agentmem"
DATABASE_URL="postgresql://user:password@localhost:5432/agentmem"

# 创建备份目录
mkdir -p $BACKUP_DIR

# 备份数据库
pg_dump $DATABASE_URL | gzip > $BACKUP_DIR/backup_$DATE.sql.gz

# 保留最近 30 天的备份
find $BACKUP_DIR -name "backup_*.sql.gz" -mtime +30 -delete

echo "Backup completed: backup_$DATE.sql.gz"
```

### 2. 增量备份

```rust
use agentmem::backup::BackupManager;

let backup_manager = BackupManager::new(memory.clone())
    .backup_interval(Duration::from_secs(3600))  // 每小时
    .retention(Duration::from_secs(86400 * 7)  // 保留 7 天
    .backup_path("/backups/agentmem")
    .start()
    .await?;
```

### 3. 恢复数据

```bash
#!/bin/bash
# restore.sh

BACKUP_FILE=$1

if [ -z "$BACKUP_FILE" ]; then
    echo "Usage: ./restore.sh <backup_file>"
    exit 1
fi

# 解压并恢复
gunzip < $BACKUP_FILE | psql postgresql://user:password@localhost:5432/agentmem

echo "Restore completed from $BACKUP_FILE"
```

### 4. 导出和导入

```rust
// 导出数据
let exporter = DataExporter::new(memory.clone());

exporter.export_to_file(
    "/backups/export.json",
    ExportOptions {
        include_embeddings: true,
        include_metadata: true,
        compression: true,
    }
).await?;

// 导入数据
let importer = DataImporter::new(memory.clone());

importer.import_from_file(
    "/backups/export.json",
    ImportOptions {
        skip_duplicates: true,
        validate_data: true,
    }
).await?;
```

## 安全最佳实践

### 1. 认证和授权

```rust
use agentmem::security::{AuthManager, JwtConfig};

let auth_manager = AuthManager::new(JwtConfig {
    secret: std::env::var("JWT_SECRET")?,
    expiration: Duration::from_secs(3600),
});

// 保护 API 端点
let app = Router::new()
    .route("/memories", post(create_memory))
    .route_layer(middleware::from_fn(
        auth_middleware
    ))
    .with_state(memory);

async fn auth_middleware(
    State(auth): State<AuthManager>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    match token {
        Some(token) => {
            auth.validate(token).await?;
            Ok(next.run(req).await)
        }
        None => Err(StatusCode::UNAUTHORIZED),
    }
}
```

### 2. 数据加密

```rust
use agentmem::security::Encryption;

let memory = Memory::builder()
    .encryption(
        Encryption::new(
            std::env::var("ENCRYPTION_KEY")?
        )
    )
    .build()
    .await?;

// 敏感数据自动加密
memory.add("用户的密码是 secret123").await?;  // 自动加密存储
```

### 3. 输入验证

```rust
use agentmem::validation::{Validator, ValidationError};

let validator = Validator::new()
    .max_content_length(10000)
    .allowed_content_types(vec!["text/plain", "application/json"])
    .block_malicious_patterns(true);

// 验证输入
validator.validate_content(content)?;
validator.validate_metadata(&metadata)?;
```

### 4. 速率限制

```rust
use agentmem::security::RateLimiter;

let rate_limiter = RateLimiter::new()
    .max_requests(100)
    .per_duration(Duration::from_secs(60))
    .key_extractor(|req| {
        req.headers()
            .get("X-API-Key")
            .map(|h| h.to_str().unwrap().to_string())
    });

// 应用到路由
Router::new()
    .route("/search", get(search))
    .layer(middleware::from_fn(
        rate_limit_middleware
    ))
    .with_state(memory);
```

## 扩展性

### 1. 水平扩展

```rust
// 使用 Redis 作为共享存储
let memory = Memory::builder()
    .with_storage("postgresql://primary-db")
    .with_cache("redis://shared-redis")
    .build()
    .await?;

// 多个实例可以共享同一个数据库和缓存
```

### 2. 读写分离

```rust
let memory = Memory::builder()
    .read_replicas(vec![
        "postgresql://replica-1",
        "postgresql://replica-2",
        "postgresql://replica-3",
    ])
    .write_source("postgresql://primary")
    .build()
    .await?;
```

### 3. 分片策略

```rust
let memory = Memory::builder()
    .sharding(ShardingConfig {
        strategy: ShardingStrategy::Hash,  // 或 Range, ConsistentHash
        shard_count: 10,
        shard_key: "agent_id",
    })
    .build()
    .await?;
```

### 4. 负载均衡

```nginx
# nginx.conf
upstream agentmem_backends {
    least_conn;
    server agentmem-1:8080 weight=3;
    server agentmem-2:8080 weight=2;
    server agentmem-3:8080 weight=1;

    keepalive 64;
}

server {
    listen 80;

    location / {
        proxy_pass http://agentmem_backends;
        proxy_http_version 1.1;
        proxy_set_header Connection "";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

## 故障排查

### 常见问题

#### 1. 连接超时

```rust
// 增加连接超时时间
let memory = Memory::builder()
    .connection_timeout(Duration::from_secs(60))
    .request_timeout(Duration::from_secs(120))
    .build()
    .await?;
```

#### 2. 内存不足

```bash
# 监控内存使用
watch -n 1 'ps aux | grep agentmem'

# 调整缓存大小
let memory = Memory::builder()
    .cache_size(1000)  # 减小缓存
    .max_concurrent_requests(50)  # 限制并发
    .build()
    .await?;
```

#### 3. 慢查询

```sql
-- 查找慢查询
SELECT query, mean_exec_time, calls
FROM pg_stat_statements
ORDER BY mean_exec_time DESC
LIMIT 10;

-- 分析查询计划
EXPLAIN ANALYZE SELECT * FROM memories WHERE agent_id = 'xxx';
```

#### 4. 高 CPU 使用

```rust
// 限制并发操作
let memory = Memory::builder()
    .max_concurrent_requests(100)
    .worker_threads(4)
    .build()
    .await?;

// 使用批处理减少请求次数
memory.add_batch(items, None, MemoryType::SEMANTIC).await?;
```

## 生产检查清单

- [ ] 使用环境变量存储敏感信息
- [ ] 启用数据库连接池
- [ ] 配置 Redis 缓存
- [ ] 设置日志记录和监控
- [ ] 配置自动备份
- [ ] 启用认证和授权
- [ ] 配置速率限制
- [ ] 设置健康检查端点
- [ ] 配置负载均衡
- [ ] 压力测试和性能基准
- [ ] 准备回滚计划
- [ ] 配置告警和通知

## 下一步

- [故障排查](../troubleshooting.md) - 详细的问题诊断指南
- [API 参考](../api_reference/) - 完整的 API 文档
