# Agent State Database 故障排除指南

## 常见问题和解决方案

### 1. 数据库连接问题

#### 问题：数据库初始化失败
```
Error: Failed to create database connection
```

**可能原因：**
- 数据库路径不存在或无权限
- 磁盘空间不足
- 文件系统权限问题

**解决方案：**
```rust
// 检查路径是否存在
use std::fs;
let db_path = "./my_agent_db";
if !std::path::Path::new(db_path).exists() {
    fs::create_dir_all(db_path)?;
}

// 使用绝对路径
let absolute_path = std::env::current_dir()?.join("agent_db");
let db = AgentDB::new(absolute_path.to_str().unwrap(), 384).await?;
```

#### 问题：连接池耗尽
```
Error: Failed to acquire connection: semaphore closed
```

**解决方案：**
```rust
// 增加连接池大小
let mut config = AgentDbConfig::default();
config.performance.parallel_workers = 50; // 增加到50个连接
config.database.max_connections = 50;

// 设置连接超时
config.database.connection_timeout_ms = 10000; // 10秒超时
```

### 2. 性能问题

#### 问题：向量搜索速度慢
```
Vector search taking too long (>5 seconds)
```

**解决方案：**
```rust
// 1. 使用HNSW索引
let mut config = AgentDbConfig::default();
config.vector.index_type = VectorIndexType::HNSW;

// 2. 调整HNSW参数
config.vector.hnsw_config.max_connections = 32;
config.vector.hnsw_config.ef_construction = 400;

// 3. 启用缓存
config.performance.cache_size_mb = 2048; // 2GB缓存

// 4. 使用批量操作
let vectors = vec![/* 多个向量 */];
db.batch_add_vectors(vectors).await?;
```

#### 问题：内存使用过高
```
System running out of memory
```

**解决方案：**
```rust
// 1. 设置内存限制
let mut config = AgentDbConfig::default();
config.performance.memory_limit_mb = 4096; // 4GB限制

// 2. 启用记忆压缩
config.memory.compression_enabled = true;
config.memory.max_memories_per_agent = 10000;

// 3. 使用流式处理
db.stream_memories(agent_id, |memory| {
    // 处理单个记忆，避免一次性加载所有数据
    process_memory(memory)?;
    Ok(())
}).await?;

// 4. 定期清理
db.cleanup_expired_memories().await?;
```

### 3. 数据一致性问题

#### 问题：数据丢失或损坏
```
Error: Data corruption detected
```

**解决方案：**
```rust
// 1. 启用备份
let mut config = AgentDbConfig::default();
config.database.backup_enabled = true;
config.database.backup_interval_hours = 6; // 每6小时备份

// 2. 使用事务
let result = db.execute_transaction(|db| {
    Box::pin(async move {
        db.save_agent_state(&state).await?;
        db.store_memory(&memory).await?;
        // 所有操作成功才提交
        Ok(())
    })
}).await?;

// 3. 验证数据完整性
let loaded_state = db.load_agent_state(agent_id).await?;
if loaded_state.is_none() {
    return Err("Data integrity check failed".into());
}
```

### 4. 并发问题

#### 问题：并发操作冲突
```
Error: Resource temporarily unavailable
```

**解决方案：**
```rust
// 1. 使用信号量控制并发
use tokio::sync::Semaphore;
use std::sync::Arc;

let semaphore = Arc::new(Semaphore::new(10)); // 最多10个并发操作

let permit = semaphore.acquire().await?;
// 执行数据库操作
db.save_agent_state(&state).await?;
drop(permit); // 释放许可

// 2. 实现重试机制
async fn retry_operation<F, T>(operation: F, max_retries: usize) -> Result<T, Box<dyn std::error::Error>>
where
    F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, Box<dyn std::error::Error>>>>>,
{
    for attempt in 0..max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt < max_retries - 1 => {
                tokio::time::sleep(tokio::time::Duration::from_millis(100 * (attempt + 1) as u64)).await;
                continue;
            }
            Err(e) => return Err(e),
        }
    }
    unreachable!()
}
```

### 5. 配置问题

#### 问题：配置验证失败
```
Error: Invalid configuration: Vector dimension must be greater than 0
```

**解决方案：**
```rust
// 1. 验证配置
let mut config = AgentDbConfig::default();
config.vector.dimension = 384; // 确保大于0
config.vector.similarity_threshold = 0.7; // 0.0-1.0之间

// 验证配置
config.validate()?;

// 2. 使用环境变量覆盖
std::env::set_var("AGENT_DB_VECTOR_DIMENSION", "768");
let config = AgentDbConfig::from_env();

// 3. 从文件加载配置
let config = AgentDbConfig::from_file("config.json")
    .unwrap_or_else(|_| {
        println!("使用默认配置");
        AgentDbConfig::default()
    });
```

### 6. 监控和调试

#### 启用详细日志
```rust
// 1. 设置日志级别
let mut config = AgentDbConfig::default();
config.logging.level = "debug".to_string();
config.logging.console_enabled = true;
config.logging.file_enabled = true;

// 2. 使用监控系统
let monitor = MonitoringManager::new(config.logging);

// 记录关键操作
monitor.log(LogLevel::Debug, "database", "开始保存状态", None);
let result = db.save_agent_state(&state).await;
match result {
    Ok(_) => monitor.log(LogLevel::Info, "database", "状态保存成功", None),
    Err(e) => monitor.record_error("save_state", &e.to_string(), None),
}

// 3. 性能监控
let start_time = std::time::Instant::now();
let result = db.search_vectors(query, 10).await?;
let duration = start_time.elapsed();
monitor.record_metric("vector_search_time", duration.as_secs_f64(), "seconds", None);
```

#### 健康检查
```rust
// 定期健康检查
async fn health_check_loop(monitor: &MonitoringManager) {
    loop {
        let db_health = monitor.health_check("database").await;
        let memory_health = monitor.health_check("memory").await;
        let cache_health = monitor.health_check("cache").await;
        
        if db_health.status != HealthStatus::Healthy {
            println!("数据库健康检查失败: {}", db_health.message);
        }
        
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}
```

### 7. 性能调优

#### 向量搜索优化
```rust
// 1. 选择合适的索引类型
let mut config = AgentDbConfig::default();

// 小数据集使用Flat索引
if vector_count < 1000 {
    config.vector.index_type = VectorIndexType::Flat;
}
// 大数据集使用HNSW索引
else {
    config.vector.index_type = VectorIndexType::HNSW;
    config.vector.hnsw_config.max_connections = 16;
    config.vector.hnsw_config.ef_construction = 200;
}

// 2. 批量操作优化
config.performance.batch_size = 1000; // 增加批量大小
config.performance.parallel_workers = num_cpus::get(); // 使用所有CPU核心
```

#### 缓存优化
```rust
// 1. 调整缓存大小
let mut config = AgentDbConfig::default();
config.performance.cache_size_mb = 1024; // 1GB缓存

// 2. 缓存预热
let cache_manager = CacheManager::new(config.performance);

// 预加载常用查询
let common_queries = vec![/* 常用查询哈希 */];
for query_hash in common_queries {
    if let Some(data) = load_query_result(query_hash) {
        cache_manager.set(query_hash, data, 1);
    }
}
```

### 8. 错误恢复

#### 自动恢复机制
```rust
use agent_state_db::{AgentDB, AgentDbError};

struct ResilientAgentDB {
    primary_db: AgentDB,
    backup_db: Option<AgentDB>,
}

impl ResilientAgentDB {
    async fn save_agent_state_with_retry(&self, state: &AgentState) -> Result<(), AgentDbError> {
        // 尝试主数据库
        match self.primary_db.save_agent_state(state).await {
            Ok(()) => Ok(()),
            Err(e) => {
                println!("主数据库失败: {}, 尝试备份数据库", e);
                
                // 尝试备份数据库
                if let Some(ref backup) = self.backup_db {
                    backup.save_agent_state(state).await
                } else {
                    Err(e)
                }
            }
        }
    }
}
```

### 9. 部署问题

#### Docker部署
```dockerfile
# Dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/my_agent /usr/local/bin/
COPY --from=builder /app/config.json /etc/agent_db/

# 创建数据目录
RUN mkdir -p /var/lib/agent_db
VOLUME ["/var/lib/agent_db"]

ENV AGENT_DB_PATH=/var/lib/agent_db
EXPOSE 8080

CMD ["my_agent"]
```

#### 环境变量配置
```bash
# 生产环境配置
export AGENT_DB_PATH="/var/lib/agent_db"
export AGENT_DB_VECTOR_DIMENSION="768"
export AGENT_DB_LOG_LEVEL="info"
export AGENT_DB_CACHE_SIZE_MB="2048"
export AGENT_DB_MAX_CONNECTIONS="100"
```

### 10. 监控和告警

#### 设置监控指标
```rust
// 关键指标监控
async fn setup_monitoring(monitor: &MonitoringManager) {
    // 响应时间监控
    let response_time_threshold = 1.0; // 1秒
    
    // 内存使用监控
    let memory_threshold = 0.8; // 80%
    
    // 错误率监控
    let error_rate_threshold = 0.05; // 5%
    
    // 定期检查指标
    tokio::spawn(async move {
        loop {
            let metrics = monitor.get_metrics(None, Some(100));
            
            // 检查响应时间
            for metric in metrics.iter().filter(|m| m.metric_name == "response_time") {
                if metric.value > response_time_threshold {
                    monitor.record_error("performance", 
                        &format!("响应时间过长: {:.2}s", metric.value), None);
                }
            }
            
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    });
}
```

## 获取帮助

如果您遇到本指南未涵盖的问题：

1. **检查日志**：启用详细日志记录以获取更多信息
2. **查看监控数据**：使用监控系统分析性能指标
3. **社区支持**：在GitHub Issues中报告问题
4. **文档**：查阅API参考文档和示例

## 预防措施

1. **定期备份**：启用自动备份功能
2. **监控告警**：设置关键指标的告警阈值
3. **容量规划**：根据数据增长预测调整配置
4. **测试**：在生产环境部署前进行充分测试
5. **版本控制**：保持配置文件的版本控制
