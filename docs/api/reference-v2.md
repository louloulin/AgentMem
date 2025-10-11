# Agent State Database API 参考文档

## 概述

Agent State Database 是一个高性能的AI Agent状态管理系统，提供了完整的状态存储、记忆管理、向量搜索和RAG功能。

## 核心API

### AgentDB

主要的数据库接口，提供所有核心功能。

```rust
use agent_state_db::{AgentDB, AgentState, Memory, Document, StateType, MemoryType};

// 创建数据库实例
let db = AgentDB::new("./my_agent_db", 384).await?;
```

#### Agent状态管理

```rust
// 保存Agent状态
let state = AgentState::new(agent_id, session_id, StateType::WorkingMemory, data);
db.save_agent_state(&state).await?;

// 加载Agent状态
let loaded_state = db.load_agent_state(agent_id).await?;

// 批量保存状态
let states = vec![state1, state2, state3];
let results = db.batch_save_agent_states(states).await?;

// 更新Agent状态
db.update_agent_state(agent_id, new_data).await?;

// 删除Agent状态
db.delete_agent_state(agent_id).await?;

// 列出所有Agent
let agents = db.list_agents().await?;

// 获取Agent状态历史
let history = db.get_agent_state_history(agent_id, limit).await?;
```

#### 记忆管理

```rust
// 存储记忆
let memory = Memory::new(agent_id, MemoryType::Episodic, "重要事件", 0.9);
db.store_memory(&memory).await?;

// 检索Agent记忆
let memories = db.get_agent_memories(agent_id, None, 10).await?;

// 按类型检索记忆
let episodic_memories = db.get_agent_memories(agent_id, Some(MemoryType::Episodic), 10).await?;

// 搜索记忆
let search_results = db.search_memories(agent_id, "关键词", 5).await?;

// 获取重要记忆
let important_memories = db.get_important_memories(agent_id, 0.8, 10).await?;

// 批量存储记忆
let memories = vec![memory1, memory2, memory3];
let results = db.batch_store_memories(memories).await?;

// 删除记忆
db.delete_memory("memory_id").await?;

// 获取记忆统计
let stats = db.get_memory_stats(agent_id).await?;

// 清理过期记忆
let cleaned_count = db.cleanup_expired_memories().await?;
```

#### 向量操作

```rust
// 添加向量
let vector = vec![0.1, 0.2, 0.3, ...]; // 384维向量
db.add_vector("vector_id".to_string(), vector, "描述".to_string()).await?;

// 向量搜索
let query_vector = vec![0.1, 0.2, 0.3, ...];
let results = db.search_vectors(query_vector, 10).await?;

// 相似度搜索
let similarity_results = db.similarity_search(query_vector, 0.7, 5).await?;

// 批量添加向量
let vectors = vec![
    ("id1".to_string(), vector1, "desc1".to_string()),
    ("id2".to_string(), vector2, "desc2".to_string()),
];
db.batch_add_vectors(vectors).await?;

// 获取向量
let vector_data = db.get_vector("vector_id").await?;

// 删除向量
db.delete_vector("vector_id").await?;

// 获取索引统计
let index_stats = db.get_index_stats().await?;
```

#### 文档和RAG

```rust
// 添加文档
let doc = Document {
    doc_id: "doc1".to_string(),
    title: "文档标题".to_string(),
    content: "文档内容...".to_string(),
    metadata: HashMap::new(),
    created_at: chrono::Utc::now().timestamp(),
    updated_at: chrono::Utc::now().timestamp(),
};
db.add_document(doc).await?;

// 搜索文档
let search_results = db.search_documents("查询词", 10).await?;

// 混合搜索（文本+向量）
let hybrid_results = db.hybrid_search("查询词", Some(query_vector), 5).await?;

// 获取文档
let document = db.get_document("doc_id").await?;

// 列出文档
let documents = db.list_documents(20).await?;

// 更新文档
db.update_document("doc_id", new_content).await?;

// 删除文档
db.delete_document("doc_id").await?;

// 批量添加文档
let documents = vec![doc1, doc2, doc3];
let results = db.batch_add_documents(documents).await?;
```

### 高级功能

#### 批量操作

```rust
// 批量删除记忆
let memory_ids = vec!["id1".to_string(), "id2".to_string()];
let results = db.batch_delete_memories(memory_ids).await?;

// 并行搜索
let queries = vec!["查询1".to_string(), "查询2".to_string()];
let results = db.parallel_search(queries, 5).await?;

// 并行向量搜索
let query_vectors = vec![vector1, vector2];
let results = db.parallel_vector_search(query_vectors, 5).await?;
```

#### 流式处理

```rust
// 流式处理记忆
db.stream_memories(agent_id, |memory| {
    println!("处理记忆: {}", memory.content);
    Ok(())
}).await?;

// 流式处理文档
db.stream_documents(|document| {
    println!("处理文档: {}", document.title);
    Ok(())
}).await?;
```

#### 事务支持

```rust
// 执行事务
let result = db.execute_transaction(|db| {
    Box::pin(async move {
        db.save_agent_state(&state).await?;
        db.store_memory(&memory).await?;
        Ok(())
    })
}).await?;
```

#### 分析功能

```rust
// 分析Agent行为模式
let patterns = db.analyze_agent_patterns(agent_id).await?;

// 获取系统健康状态
let health = db.get_system_health().await?;
```

## 配置管理

### AgentDbConfig

```rust
use agent_state_db::{AgentDbConfig, ConfigManager};

// 使用默认配置
let config = AgentDbConfig::default();

// 从文件加载配置
let config = AgentDbConfig::from_file("config.json")?;

// 从环境变量加载配置
let config = AgentDbConfig::from_env();

// 配置管理器
let mut manager = ConfigManager::new();
manager.update_config(config)?;
```

### 配置选项

```rust
// 数据库配置
config.database.path = "./my_db".to_string();
config.database.max_connections = 20;

// 向量配置
config.vector.dimension = 768;
config.vector.index_type = VectorIndexType::HNSW;

// 性能配置
config.performance.cache_size_mb = 1024;
config.performance.batch_size = 200;

// 日志配置
config.logging.level = "debug".to_string();
config.logging.file_enabled = true;
```

## 性能优化

### 缓存管理

```rust
use agent_state_db::CacheManager;

let cache_manager = CacheManager::new(config.performance);

// 设置缓存
cache_manager.set(query_hash, data, result_count);

// 获取缓存
if let Some(cached_data) = cache_manager.get(query_hash) {
    // 使用缓存数据
}

// 获取缓存统计
let stats = cache_manager.get_statistics();
```

### 连接池

```rust
use agent_state_db::ConnectionPool;

let pool = ConnectionPool::new("./db", config.performance).await?;

// 获取连接
let connection = pool.get_connection().await?;

// 归还连接
pool.return_connection(connection);

// 获取连接池统计
let stats = pool.get_statistics();
```

## 监控和日志

### 监控管理器

```rust
use agent_state_db::{MonitoringManager, LogLevel};

let monitor = MonitoringManager::new(config.logging);

// 记录日志
monitor.log(LogLevel::Info, "module", "消息", None);

// 记录性能指标
monitor.record_metric("response_time", 0.123, "seconds", None);

// 记录错误
monitor.record_error("error_type", "错误消息", None);

// 健康检查
let health_result = monitor.health_check("database").await;

// 获取监控数据
let logs = monitor.get_logs(Some(LogLevel::Error), Some(100));
let metrics = monitor.get_metrics(Some("response_time"), Some(50));
let errors = monitor.get_error_summary();

// 导出监控数据
let exported_data = monitor.export_monitoring_data()?;
```

## 工具函数

### 文本处理

```rust
use agent_state_db::utils::text;

// 文本清理
let cleaned = text::clean_text("原始文本...");

// 文本分词
let tokens = text::tokenize("要分词的文本");

// 计算相似性
let similarity = text::jaccard_similarity("文本1", "文本2");

// 提取关键词
let keywords = text::extract_keywords("文本内容", 10);
```

### 向量计算

```rust
use agent_state_db::utils::vector;

// 向量归一化
let mut vector = vec![1.0, 2.0, 3.0];
vector::normalize(&mut vector);

// 向量点积
let dot_product = vector::dot_product(&vector1, &vector2)?;

// 向量加法
let sum = vector::add(&vector1, &vector2)?;

// 计算范数
let l2_norm = vector::l2_norm(&vector);
```

### 时间处理

```rust
use agent_state_db::utils::time;

// 获取当前时间戳
let timestamp = time::current_timestamp();

// 时间戳转字符串
let time_str = time::timestamp_to_string(timestamp);

// 检查是否过期
let is_expired = time::is_expired(timestamp, 3600); // 1小时TTL
```

## 错误处理

所有API都返回`Result<T, AgentDbError>`类型，主要错误类型包括：

```rust
use agent_state_db::AgentDbError;

match result {
    Ok(data) => {
        // 处理成功结果
    }
    Err(AgentDbError::NotFound) => {
        // 处理未找到错误
    }
    Err(AgentDbError::InvalidArgument(msg)) => {
        // 处理参数错误
    }
    Err(AgentDbError::Internal(msg)) => {
        // 处理内部错误
    }
    Err(AgentDbError::Serde(err)) => {
        // 处理序列化错误
    }
    Err(AgentDbError::LanceDb(err)) => {
        // 处理数据库错误
    }
}
```

## C FFI接口

对于C/C++集成，提供了完整的FFI接口：

```c
#include "agent_state_db.h"

// 创建数据库
CAgentStateDB* db = agent_db_new("./db_path");

// 保存状态
int result = agent_db_save_state(db, agent_id, session_id, state_type, data, data_len);

// 加载状态
uint8_t* data;
size_t data_len;
int result = agent_db_load_state(db, agent_id, &data, &data_len);

// 释放数据
agent_db_free_data(data, data_len);

// 释放数据库
agent_db_free(db);
```

## 最佳实践

1. **配置优化**：根据使用场景调整缓存大小和批量操作参数
2. **错误处理**：始终检查API返回的错误状态
3. **资源管理**：及时释放不需要的资源
4. **监控**：使用监控系统跟踪性能和错误
5. **批量操作**：对于大量数据操作，使用批量API提高性能
6. **流式处理**：对于大数据集，使用流式处理避免内存溢出

## 故障排除

### 常见问题

**Q: 数据库连接失败**
A: 检查数据库路径权限，确保目录存在且可写

**Q: 向量搜索性能差**
A: 考虑使用HNSW索引，调整缓存大小，使用批量操作

**Q: 内存使用过高**
A: 启用记忆压缩，调整缓存大小，使用流式处理

**Q: 并发操作失败**
A: 检查连接池配置，增加最大连接数

详细故障排除指南请参考 [故障排除文档](troubleshooting.md)。
