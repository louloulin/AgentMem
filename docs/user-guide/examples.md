# Agent State Database 使用示例

## 快速开始

### 基本设置

```rust
use agent_state_db::{AgentDB, AgentState, Memory, Document, StateType, MemoryType};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建数据库实例
    let db = AgentDB::new("./my_agent_db", 384).await?;
    
    println!("Agent State Database 已初始化!");
    Ok(())
}
```

## 示例1: 基本Agent状态管理

```rust
async fn basic_agent_example() -> Result<(), Box<dyn std::error::Error>> {
    let db = AgentDB::new("./agent_example", 384).await?;
    
    // 创建Agent状态
    let agent_id = 1001u64;
    let session_id = 1u64;
    let state_data = vec![1, 2, 3, 4, 5]; // 简单的状态数据
    
    let state = AgentState::new(agent_id, session_id, StateType::WorkingMemory, state_data);
    
    // 保存状态
    db.save_agent_state(&state).await?;
    println!("Agent {} 状态已保存", agent_id);
    
    // 加载状态
    if let Some(loaded_state) = db.load_agent_state(agent_id).await? {
        println!("加载的状态数据: {:?}", loaded_state.data);
        println!("状态类型: {:?}", loaded_state.state_type);
    }
    
    // 更新状态
    let new_data = vec![6, 7, 8, 9, 10];
    db.update_agent_state(agent_id, new_data).await?;
    println!("Agent {} 状态已更新", agent_id);
    
    Ok(())
}
```

## 示例2: 记忆管理系统

```rust
async fn memory_management_example() -> Result<(), Box<dyn std::error::Error>> {
    let db = AgentDB::new("./memory_example", 384).await?;
    let agent_id = 2001u64;
    
    // 创建不同类型的记忆
    let memories = vec![
        Memory::new(agent_id, MemoryType::Episodic, "今天遇到了一个有趣的问题".to_string(), 0.8),
        Memory::new(agent_id, MemoryType::Semantic, "Rust是一种系统编程语言".to_string(), 0.9),
        Memory::new(agent_id, MemoryType::Procedural, "如何编译Rust项目：cargo build".to_string(), 0.7),
        Memory::new(agent_id, MemoryType::Working, "当前正在处理的任务ID: 12345".to_string(), 0.6),
    ];
    
    // 批量存储记忆
    let results = db.batch_store_memories(memories).await?;
    println!("存储了 {} 条记忆", results.len());
    
    // 检索所有记忆
    let all_memories = db.get_agent_memories(agent_id, None, 10).await?;
    println!("Agent {} 总共有 {} 条记忆", agent_id, all_memories.len());
    
    // 按类型检索记忆
    let episodic_memories = db.get_agent_memories(agent_id, Some(MemoryType::Episodic), 10).await?;
    println!("情节记忆数量: {}", episodic_memories.len());
    
    // 搜索记忆
    let search_results = db.search_memories(agent_id, "Rust", 5).await?;
    println!("搜索'Rust'找到 {} 条相关记忆", search_results.len());
    
    // 获取重要记忆
    let important_memories = db.get_important_memories(agent_id, 0.8, 5).await?;
    println!("重要记忆(>0.8)数量: {}", important_memories.len());
    
    Ok(())
}
```

## 示例3: 向量搜索和相似性

```rust
async fn vector_search_example() -> Result<(), Box<dyn std::error::Error>> {
    let db = AgentDB::new("./vector_example", 384).await?;
    
    // 创建一些示例向量（实际应用中这些会是嵌入向量）
    let vectors = vec![
        ("doc1".to_string(), vec![0.1; 384], "技术文档".to_string()),
        ("doc2".to_string(), vec![0.2; 384], "用户手册".to_string()),
        ("doc3".to_string(), vec![0.3; 384], "API参考".to_string()),
        ("doc4".to_string(), vec![0.15; 384], "技术指南".to_string()),
    ];
    
    // 批量添加向量
    db.batch_add_vectors(vectors).await?;
    println!("添加了 4 个向量");
    
    // 向量搜索
    let query_vector = vec![0.12; 384]; // 查询向量
    let search_results = db.search_vectors(query_vector.clone(), 3).await?;
    
    println!("向量搜索结果:");
    for result in search_results {
        println!("  ID: {}, 距离: {:.4}, 元数据: {}", 
                result.vector_id, result.distance, result.metadata);
    }
    
    // 相似度搜索
    let similarity_results = db.similarity_search(query_vector, 0.8, 2).await?;
    println!("相似度搜索(>0.8)结果数量: {}", similarity_results.len());
    
    // 获取索引统计
    let stats = db.get_index_stats().await?;
    println!("向量索引统计: {} 个向量", stats.vector_count);
    
    Ok(())
}
```

## 示例4: 文档RAG系统

```rust
async fn rag_system_example() -> Result<(), Box<dyn std::error::Error>> {
    let db = AgentDB::new("./rag_example", 384).await?;
    
    // 添加文档
    let documents = vec![
        Document {
            doc_id: "rust_guide".to_string(),
            title: "Rust编程指南".to_string(),
            content: "Rust是一种系统编程语言，注重安全性、速度和并发性。它通过所有权系统管理内存，避免了垃圾回收的开销。".to_string(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("category".to_string(), "programming".to_string());
                meta.insert("language".to_string(), "rust".to_string());
                meta
            },
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
        },
        Document {
            doc_id: "async_programming".to_string(),
            title: "异步编程概念".to_string(),
            content: "异步编程允许程序在等待I/O操作时执行其他任务。Rust的async/await语法使异步编程变得简单和安全。".to_string(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("category".to_string(), "programming".to_string());
                meta.insert("topic".to_string(), "async".to_string());
                meta
            },
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
        },
    ];
    
    // 批量添加文档
    let results = db.batch_add_documents(documents).await?;
    println!("添加了 {} 个文档", results.len());
    
    // 文本搜索
    let search_results = db.search_documents("Rust编程", 5).await?;
    println!("文本搜索结果:");
    for result in search_results {
        println!("  文档: {}, 分数: {:.4}", result.chunk_id, result.score);
        println!("  内容: {}", result.content.chars().take(100).collect::<String>());
    }
    
    // 混合搜索（文本+向量）
    let query_vector = vec![0.1; 384]; // 查询的嵌入向量
    let hybrid_results = db.hybrid_search("异步编程", Some(query_vector), 3).await?;
    println!("混合搜索结果数量: {}", hybrid_results.len());
    
    // 列出所有文档
    let all_docs = db.list_documents(10).await?;
    println!("数据库中共有 {} 个文档", all_docs.len());
    
    Ok(())
}
```

## 示例5: 性能优化和缓存

```rust
use agent_state_db::{CacheManager, AgentDbConfig};

async fn performance_optimization_example() -> Result<(), Box<dyn std::error::Error>> {
    let config = AgentDbConfig::default();
    
    // 创建缓存管理器
    let cache_manager = CacheManager::new(config.performance);
    
    // 模拟查询操作
    let query_hash = 12345u64;
    let expensive_query_result = vec![1, 2, 3, 4, 5]; // 模拟查询结果
    
    // 检查缓存
    if let Some(cached_result) = cache_manager.get(query_hash) {
        println!("缓存命中! 结果: {:?}", cached_result);
    } else {
        println!("缓存未命中，执行查询...");
        
        // 模拟耗时查询
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // 将结果存入缓存
        cache_manager.set(query_hash, expensive_query_result.clone(), 1);
        println!("查询完成，结果已缓存: {:?}", expensive_query_result);
    }
    
    // 获取缓存统计
    let stats = cache_manager.get_statistics();
    println!("缓存统计:");
    println!("  总条目: {}", stats.total_entries);
    println!("  总命中: {}", stats.total_hits);
    println!("  命中率: {:.2}%", stats.hit_rate * 100.0);
    println!("  内存使用: {} 字节", stats.memory_usage);
    
    Ok(())
}
```

## 示例6: 监控和日志

```rust
use agent_state_db::{MonitoringManager, LogLevel, AgentDbConfig};

async fn monitoring_example() -> Result<(), Box<dyn std::error::Error>> {
    let config = AgentDbConfig::default();
    let monitor = MonitoringManager::new(config.logging);
    
    // 记录不同级别的日志
    monitor.log(LogLevel::Info, "app", "应用程序启动", None);
    monitor.log(LogLevel::Debug, "database", "数据库连接建立", None);
    monitor.log(LogLevel::Warn, "cache", "缓存使用率较高", None);
    monitor.log(LogLevel::Error, "network", "网络连接失败", None);
    
    // 记录性能指标
    monitor.record_metric("response_time", 0.123, "seconds", None);
    monitor.record_metric("memory_usage", 512.0, "MB", None);
    monitor.record_metric("active_connections", 25.0, "count", None);
    
    // 记录错误
    monitor.record_error("connection_error", "数据库连接超时", None);
    monitor.record_error("validation_error", "输入参数无效", None);
    
    // 执行健康检查
    let db_health = monitor.health_check("database").await;
    println!("数据库健康状态: {:?}", db_health.status);
    
    let memory_health = monitor.health_check("memory").await;
    println!("内存健康状态: {:?}", memory_health.status);
    
    // 获取监控数据
    let recent_logs = monitor.get_logs(Some(LogLevel::Error), Some(10));
    println!("最近的错误日志数量: {}", recent_logs.len());
    
    let metrics = monitor.get_metrics(Some("response_time"), Some(5));
    println!("响应时间指标数量: {}", metrics.len());
    
    let error_summary = monitor.get_error_summary();
    println!("错误类型统计:");
    for error in error_summary {
        println!("  {}: {} 次", error.error_type, error.count);
    }
    
    // 导出监控数据
    let exported_data = monitor.export_monitoring_data()?;
    println!("监控数据已导出，大小: {} 字节", exported_data.len());
    
    Ok(())
}
```

## 示例7: 流式处理大数据

```rust
async fn stream_processing_example() -> Result<(), Box<dyn std::error::Error>> {
    let db = AgentDB::new("./stream_example", 384).await?;
    let agent_id = 7001u64;
    
    // 首先创建大量测试数据
    println!("创建测试数据...");
    for i in 0..1000 {
        let memory = Memory::new(
            agent_id, 
            MemoryType::Episodic, 
            format!("流式处理测试记忆 {}", i), 
            0.5 + (i as f32 / 1000.0) * 0.5
        );
        db.store_memory(&memory).await?;
    }
    
    // 流式处理记忆
    println!("开始流式处理记忆...");
    let mut processed_count = 0;
    let mut total_importance = 0.0;
    
    db.stream_memories(agent_id, |memory| {
        processed_count += 1;
        total_importance += memory.importance;
        
        // 每处理100条记忆输出一次进度
        if processed_count % 100 == 0 {
            println!("已处理 {} 条记忆", processed_count);
        }
        
        Ok(())
    }).await?;
    
    let average_importance = total_importance / processed_count as f32;
    println!("流式处理完成:");
    println!("  总处理数量: {}", processed_count);
    println!("  平均重要性: {:.3}", average_importance);
    
    Ok(())
}
```

## 示例8: 配置管理

```rust
use agent_state_db::{AgentDbConfig, ConfigManager, VectorIndexType};

fn configuration_example() -> Result<(), Box<dyn std::error::Error>> {
    // 创建自定义配置
    let mut config = AgentDbConfig::default();
    
    // 数据库配置
    config.database.path = "./custom_db".to_string();
    config.database.max_connections = 20;
    config.database.backup_enabled = true;
    
    // 向量配置
    config.vector.dimension = 768;
    config.vector.index_type = VectorIndexType::HNSW;
    config.vector.similarity_threshold = 0.8;
    
    // 性能配置
    config.performance.cache_size_mb = 1024;
    config.performance.batch_size = 200;
    config.performance.parallel_workers = 8;
    
    // 日志配置
    config.logging.level = "debug".to_string();
    config.logging.file_enabled = true;
    config.logging.file_path = Some("./logs/agent_db.log".to_string());
    
    // 验证配置
    config.validate()?;
    println!("配置验证通过");
    
    // 保存配置到文件
    config.save_to_file("./config.json")?;
    println!("配置已保存到文件");
    
    // 使用配置管理器
    let mut manager = ConfigManager::new();
    manager.update_config(config)?;
    
    let current_config = manager.get_config();
    println!("当前向量维度: {}", current_config.vector.dimension);
    println!("当前缓存大小: {} MB", current_config.performance.cache_size_mb);
    
    Ok(())
}
```

## 示例9: 错误处理最佳实践

```rust
use agent_state_db::{AgentDB, AgentDbError};

async fn error_handling_example() -> Result<(), Box<dyn std::error::Error>> {
    let db = AgentDB::new("./error_example", 384).await?;
    
    // 处理不同类型的错误
    match db.load_agent_state(99999).await {
        Ok(Some(state)) => {
            println!("找到状态: {:?}", state.state_type);
        }
        Ok(None) => {
            println!("Agent状态不存在");
        }
        Err(AgentDbError::NotFound) => {
            println!("数据未找到");
        }
        Err(AgentDbError::InvalidArgument(msg)) => {
            println!("参数错误: {}", msg);
        }
        Err(AgentDbError::Internal(msg)) => {
            println!("内部错误: {}", msg);
        }
        Err(e) => {
            println!("其他错误: {:?}", e);
        }
    }
    
    // 使用Result的便捷方法
    let memory_result = db.get_agent_memories(1001, None, 10).await;
    match memory_result {
        Ok(memories) => {
            println!("获取到 {} 条记忆", memories.len());
        }
        Err(e) => {
            eprintln!("获取记忆失败: {}", e);
            // 可以选择重试或使用默认值
        }
    }
    
    Ok(())
}
```

## 示例10: 完整的AI Agent应用

```rust
use agent_state_db::*;
use std::collections::HashMap;

struct AIAgent {
    id: u64,
    db: AgentDB,
    config: AgentDbConfig,
    monitor: MonitoringManager,
}

impl AIAgent {
    async fn new(agent_id: u64, db_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config = AgentDbConfig::default();
        let db = AgentDB::new(db_path, config.vector.dimension).await?;
        let monitor = MonitoringManager::new(config.logging.clone());
        
        Ok(Self {
            id: agent_id,
            db,
            config,
            monitor,
        })
    }
    
    async fn process_input(&self, input: &str) -> Result<String, Box<dyn std::error::Error>> {
        // 记录输入日志
        self.monitor.log(LogLevel::Info, "agent", &format!("处理输入: {}", input), None);
        
        // 搜索相关记忆
        let memories = self.db.search_memories(self.id, input, 5).await?;
        self.monitor.record_metric("memory_search_count", memories.len() as f64, "count", None);
        
        // 搜索相关文档
        let documents = self.db.search_documents(input, 3).await?;
        
        // 生成响应（简化）
        let response = format!("基于 {} 条记忆和 {} 个文档生成的响应", memories.len(), documents.len());
        
        // 存储新记忆
        let new_memory = Memory::new(
            self.id,
            MemoryType::Episodic,
            format!("用户输入: {} | 响应: {}", input, response),
            0.7
        );
        self.db.store_memory(&new_memory).await?;
        
        // 更新Agent状态
        let state_data = format!("最后处理: {}", input).into_bytes();
        let state = AgentState::new(self.id, 1, StateType::WorkingMemory, state_data);
        self.db.save_agent_state(&state).await?;
        
        Ok(response)
    }
    
    async fn get_status(&self) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
        let mut status = HashMap::new();
        
        // 获取记忆统计
        let memory_stats = self.db.get_memory_stats(self.id).await?;
        status.insert("total_memories".to_string(), memory_stats.values().sum::<u64>().to_string());
        
        // 获取系统健康状态
        let health = self.db.get_system_health().await?;
        status.extend(health);
        
        // 获取运行时间
        let uptime = self.monitor.get_uptime();
        status.insert("uptime_seconds".to_string(), uptime.as_secs().to_string());
        
        Ok(status)
    }
}

async fn complete_ai_agent_example() -> Result<(), Box<dyn std::error::Error>> {
    // 创建AI Agent
    let agent = AIAgent::new(9001, "./ai_agent_db").await?;
    
    // 处理一些输入
    let inputs = vec![
        "什么是Rust编程语言？",
        "如何使用异步编程？",
        "向量数据库的优势是什么？",
    ];
    
    for input in inputs {
        let response = agent.process_input(input).await?;
        println!("输入: {}", input);
        println!("响应: {}", response);
        println!("---");
    }
    
    // 获取Agent状态
    let status = agent.get_status().await?;
    println!("Agent状态:");
    for (key, value) in status {
        println!("  {}: {}", key, value);
    }
    
    Ok(())
}
```

## 运行示例

要运行这些示例，请在您的`Cargo.toml`中添加依赖：

```toml
[dependencies]
agent-state-db = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
chrono = "0.4"
```

然后在您的`main.rs`中调用相应的示例函数：

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 运行基本示例
    basic_agent_example().await?;
    
    // 运行记忆管理示例
    memory_management_example().await?;
    
    // 运行其他示例...
    
    Ok(())
}
```

这些示例展示了Agent State Database的主要功能和使用模式，可以作为您开发AI Agent应用的起点。
