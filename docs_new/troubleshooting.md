# 故障排查指南

常见问题的诊断和解决方案。

## 目录

- [安装问题](#安装问题)
- [连接问题](#连接问题)
- [性能问题](#性能问题)
- [搜索问题](#搜索问题)
- [内存问题](#内存问题)
- [错误代码](#错误代码)

## 安装问题

### Rust 编译错误

**问题**: `error: linking with cc failed`

**解决方案**:
```bash
# 确保安装了必要的系统依赖
# Ubuntu/Debian
sudo apt-get install build-essential pkg-config libssl-dev

# macOS
xcode-select --install

# 更新 Rust
rustup update
```

### 依赖冲突

**问题**: `error: duplicate crate`

**解决方案**:
```bash
# 清理构建缓存
cargo clean

# 更新依赖
cargo update

# 重新构建
cargo build
```

## 连接问题

### 数据库连接超时

**问题**: `Connection timeout after 30s`

**解决方案**:
```rust
// 增加超时时间
let memory = Memory::builder()
    .connection_timeout(Duration::from_secs(60))
    .build()
    .await?;

// 或检查连接字符串
let memory = Memory::builder()
    .with_storage("postgresql://user:password@localhost:5432/agentmem?connect_timeout=60")
    .build()
    .await?;
```

### Redis 连接失败

**问题**: `Connection refused: redis://localhost:6379`

**解决方案**:
```bash
# 检查 Redis 是否运行
redis-cli ping

# 启动 Redis
redis-server

# 或禁用 Redis 缓存
let memory = Memory::builder()
    .cache_enabled(false)
    .build()
    .await?;
```

## 性能问题

### 搜索速度慢

**问题**: 搜索耗时超过 1 秒

**诊断**:
```rust
// 启用性能分析
let options = SearchOptions::builder()
    .query("查询")
    .enable_profiling(true)
    .build();

let results = memory.searchWithOptions(options).await?;

if let Some(profile) = results.profile {
    println!("向量搜索: {:.2}ms", profile.vector_search_duration);
    println!("数据库查询: {:.2}ms", profile.db_query_duration);
}
```

**解决方案**:
1. 创建向量索引
2. 减少搜索结果数量
3. 启用缓存
4. 使用批量操作

### 添加记忆慢

**问题**: 每条记忆添加耗时 > 100ms

**解决方案**:
```rust
// 使用批量添加
let items = vec![/* ... */];
memory.add_batch(items, None, MemoryType::SEMANTIC).await?;

// 或禁用智能功能（如果不需要）
let memory = Memory::builder()
    .infer(false)
    .build()
    .await?;
```

## 搜索问题

### 返回空结果

**问题**: 搜索返回空结果，但记忆确实存在

**诊断**:
```rust
// 检查记忆是否真的存在
let all = memory.get_all(GetAllOptions::default()).await?;
println!("总记忆数: {}", all.len());

// 降低相似度阈值
let results = memory.search(
    SearchOptions::builder()
        .query("查询")
        .threshold(0.3)  // 降低阈值
        .build()
).await?;
```

**解决方案**:
1. 检查搜索查询是否正确
2. 降低相似度阈值
3. 尝试关键词搜索
4. 检查记忆类型过滤

### 结果不相关

**问题**: 搜索结果与查询不相关

**解决方案**:
```rust
// 使用混合搜索
let options = SearchOptions::builder()
    .query("查询")
    .hybrid_search(HybridSearchConfig {
        semantic_weight: 0.5,
        keyword_weight: 0.5,
        fusion_method: FusionMethod::RRF { k: 60.0 },
    })
    .build();

let results = memory.searchWithOptions(options).await?;
```

## 内存问题

### 内存使用过高

**问题**: 进程内存占用 > 2GB

**诊断**:
```bash
# 检查内存使用
ps aux | grep agentmem

# 或使用内置监控
let stats = memory.get_memory_stats().await?;
println!("缓存大小: {}", stats.cache_size);
```

**解决方案**:
```rust
// 限制缓存大小
let memory = Memory::builder()
    .cache_size(1000)  // 减小缓存
    .max_concurrent_requests(50)
    .build()
    .await?;

// 定期清理
memory.cleanup(
    CleanupOptions::builder()
        .before(Utc::now() - Duration::days(30))
        .build()
).await?;
```

### 内存泄漏

**问题**: 内存持续增长

**诊断**:
```bash
# 使用 valgrind 检查
valgrind --leak-check=full ./agentmem

# 或使用内存分析工具
heaptrack ./agentmem
```

**解决方案**:
1. 确保正确关闭连接
2. 避免循环引用
3. 定期清理不需要的记忆
4. 更新到最新版本

## 错误代码

### AgentMemError

所有 AgentMem 错误的基类。

```rust
pub enum AgentMemError {
    // 连接错误
    ConnectionError(String),

    // 存储错误
    StorageError(String),

    // LLM 错误
    LLMError(String),

    // 嵌入错误
    EmbeddingError(String),

    // 验证错误
    ValidationError(String),

    // 未找到
    NotFound(String),

    // 超时
    Timeout(String),
}
```

### 处理错误

```rust
use agentmem::{AgentMemError, Memory};

match memory.add("内容").await {
    Ok(result) => println!("成功: {}", result.id),
    Err(AgentMemError::ConnectionError(msg)) => {
        eprintln!("连接错误: {}", msg);
        // 重试逻辑
    }
    Err(AgentMemError::ValidationError(msg)) => {
        eprintln!("验证错误: {}", msg);
        // 修正输入
    }
    Err(e) => {
        eprintln!("其他错误: {:?}", e);
    }
}
```

## 日志调试

### 启用详细日志

```rust
use tracing_subscriber;

tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();

let memory = Memory::new().await?;
```

### 查看日志

```bash
# 查看应用日志
tail -f /var/log/agentmem/app.log

# 过滤错误
grep "ERROR" /var/log/agentmem/app.log

# 查看特定时间段
grep "2024-01-15 10:" /var/log/agentmem/app.log
```

## 获取帮助

如果问题仍未解决：

1. 查看 [GitHub Issues](https://github.com/agentmem/agentmem/issues)
2. 在 [Discussions](https://github.com/agentmem/agentmem/discussions) 提问
3. 联系支持: support@agentmem.dev

## 报告问题

报告问题时，请包含：

1. AgentMem 版本
2. 操作系统和版本
3. 最小可复现代码
4. 完整错误信息
5. 日志输出（如果相关）

```rust
// 获取版本信息
println!("AgentMem version: {}", env!("CARGO_PKG_VERSION"));
```
