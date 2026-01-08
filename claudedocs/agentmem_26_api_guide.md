# AgentMem 2.6 API 使用指南

## 📋 目录

1. [快速开始](#快速开始)
2. [核心 API](#核心-api)
3. [P0-P3 功能 API](#p0-p3-功能-api)
4. [插件开发](#插件开发)
5. [常见场景](#常见场景)
6. [故障排除](#故障排除)

---

## 快速开始

### 安装

```toml
[dependencies]
agent-mem = "0.2.6"
agent-mem-core = "0.2.6"
agent-mem-plugins = "0.2.6"
```

### 5 分钟入门

```rust
use agent_mem_core::{Memory, MemoryEngine, MemoryEngineConfig};
use agent_mem_traits::{AttributeKey, AttributeValue};

#[tokio::main]
async fn main() -> Result<()> {
    // 1. 创建引擎
    let config = MemoryEngineConfig::default();
    let engine = MemoryEngine::new(config).await?;

    // 2. 添加记忆
    let memory = Memory::builder()
        .content("今天学习了 AgentMem 2.6")
        .attribute("importance", 0.9)
        .build();

    engine.add(memory).await?;

    // 3. 搜索记忆
    let results = engine.search("AgentMem", None, Some(10)).await?;
    for memory in results {
        println!("找到: {}", memory.content);
    }

    Ok(())
}
```

---

## 核心 API

### Memory API

#### 创建记忆

```rust
use agent_mem_core::Memory;
use agent_mem_traits::{AttributeKey, AttributeValue};

// 方式 1: 使用 Builder
let memory = Memory::builder()
    .content("记忆内容")
    .attribute("importance", 0.9)
    .attribute("category", "工作")
    .build();

// 方式 2: 使用 AttributeSet
let mut attributes = AttributeSet::new();
attributes.insert(
    AttributeKey::from("importance"),
    AttributeValue::Number(0.9)
);

let memory = Memory::new(
    MemoryContent::Text("记忆内容".to_string()),
    MemoryMetadata::new(),
    attributes
);

// 方式 3: 多模态内容
let memory = Memory::builder()
    .content(MemoryContent::Structured(json!({
        "title": "项目报告",
        "status": "进行中"
    })))
    .build();
```

#### 访问记忆属性

```rust
// 获取内容
let content = memory.content.as_str()?;

// 获取属性
let importance = memory.attributes
    .get(&AttributeKey::from("importance"))
    .and_then(|v| v.as_number());

// 检查属性存在
if memory.attributes.contains_key(&AttributeKey::from("category")) {
    println!("有分类属性");
}

// 遍历所有属性
for (key, value) in &memory.attributes {
    println!("{}: {:?}", key, value);
}
```

### MemoryEngine API

#### 创建引擎

```rust
use agent_mem_core::{MemoryEngine, MemoryEngineConfig, MemoryScheduler};
use agent_mem_core::scheduler::DefaultMemoryScheduler;

// 基础引擎
let config = MemoryEngineConfig::default();
let engine = MemoryEngine::new(config).await?;

// 带调度器的引擎
let config = MemoryEngineConfig::default();
let scheduler = DefaultMemoryScheduler::new(ScheduleConfig::default());
let engine = MemoryEngine::new(config)
    .await?
    .with_scheduler(scheduler);
```

#### 添加记忆

```rust
// 单个添加
engine.add(memory).await?;

// 批量添加
let memories = vec![memory1, memory2, memory3];
engine.add_batch(memories).await?;
```

#### 搜索记忆

```rust
// 简单搜索
let results = engine.search("查询内容", None, Some(10)).await?;

// 带作用域搜索
use agent_mem_core::MemoryScope;
let scope = MemoryScope::User {
    agent_id: "agent_123".to_string(),
    user_id: "user_456".to_string(),
};

let results = engine.search("查询内容", Some(scope), Some(10)).await?;

// 带调度器搜索
let results = engine.search_with_scheduler(
    "查询内容",
    Some(scope),
    Some(10)
).await?;
```

#### 更新和删除

```rust
// 更新记忆
engine.update(memory_id, updated_memory).await?;

// 删除记忆
engine.delete(memory_id).await?;

// 批量删除
let ids = vec![id1, id2, id3];
engine.delete_batch(ids).await?;
```

---

## P0-P3 功能 API

### P0: MemoryScheduler API

```rust
use agent_mem_core::scheduler::{
    DefaultMemoryScheduler, ScheduleConfig, ExponentialDecayModel
};

// 创建调度器
let decay_model = ExponentialDecayModel::new(0.01); // λ = 0.01
let config = ScheduleConfig::builder()
    .decay_model(decay_model)
    .importance_weight(0.3)
    .recency_weight(0.2)
    .relevance_weight(0.5)
    .build();

let scheduler = DefaultMemoryScheduler::new(config);

// 手动调度
let mut memories = vec![memory1, memory2, memory3];
let scheduled = scheduler.schedule(memories);

// 计算单个记忆得分
let score = scheduler.calculate_score(&memory);
println!("记忆得分: {:.2}", score);
```

### P1: 高级能力 API

#### 1. 主动检索 (ActiveRetrieval)

```rust
use agent_mem_core::retrieval::ActiveRetrievalSystem;
use agent_mem_core::AgentOrchestrator;

let orchestrator = AgentOrchestrator::new(config).await?;

// 使用 orchestrator 的方法
let memories = orchestrator
    .search_enhanced("项目进展", agent_id, user_id, 10)
    .await?;

// 直接使用系统
let system = ActiveRetrievalSystem::new(system_config);
let result = system
    .search_with_topic_extraction("我最近在做什么?", &scope)
    .await?;
```

#### 2. 时序推理 (TemporalReasoning)

```rust
use agent_mem_core::temporal_reasoning::TemporalReasoningEngine;
use chrono::{Utc, DateTime};

let orchestrator = AgentOrchestrator::new(config).await?;

// 时序查询
let timeline = orchestrator
    .temporal_query("上周一到周五的工作记录")
    .await?;

// 时间范围查询
let start: DateTime<Utc> = "2025-01-01T00:00:00Z".parse()?;
let end: DateTime<Utc> = "2025-01-07T23:59:59Z".parse()?;

let memories = orchestrator
    .temporal_range_query(start, end, &scope)
    .await?;

// 直接使用引擎
let engine = TemporalReasoningEngine::new(engine_config);
let results = engine
    .query_by_range("最近一周", &scope)
    .await?;
```

#### 3. 因果推理 (CausalReasoning)

```rust
use agent_mem_core::causal_reasoning::CausalReasoningEngine;

let orchestrator = AgentOrchestrator::new(config).await?;

// 解释因果关系
let causality = orchestrator
    .explain_causality("为什么项目延期了?")
    .await?;

println!("原因: {:?}", causality.causes);
println!("结果: {:?}", causality.effects);

// 反事实推理
let counterfactual = orchestrator
    .counterfactual_reasoning(
        "如果当时用了更好的算法会怎样?",
        &memory_id
    )
    .await?;

// 直接使用引擎
let engine = CausalReasoningEngine::new(engine_config);
let analysis = engine
    .analyze_causality("事件A", "事件B")
    .await?;
```

#### 4. 图记忆 (GraphMemory)

```rust
use agent_mem_core::graph_memory::GraphMemoryEngine;

let orchestrator = AgentOrchestrator::new(config).await?;

// 图遍历
let graph = orchestrator
    .graph_traverse(start_memory_id, max_depth=3)
    .await?;

println!("找到 {} 个相关记忆", graph.len());

// 社区发现
let communities = orchestrator
    .discover_communities(min_size=3)
    .await?;

// 关系推理
let relations = orchestrator
    .infer_relations(memory_id)
    .await?;

// 直接使用引擎
let engine = GraphMemoryEngine::new(engine_config);
let path = engine
    .find_shortest_path(from_id, to_id)
    .await?;
```

#### 5. 自适应策略 (AdaptiveStrategy)

```rust
use agent_mem_core::adaptive_strategy::AdaptiveStrategyManager;

let orchestrator = AgentOrchestrator::new(config).await?;

// 自动选择策略
let strategy = orchestrator
    .select_strategy("复杂查询任务")
    .await?;

println!("推荐策略: {:?}", strategy);

// 性能分析
let metrics = orchestrator
    .analyze_performance()
    .await?;

println!("当前性能指标: {:?}", metrics);
```

#### 6. LLM 优化器 (LlmOptimizer)

```rust
use agent_mem_core::llm_optimizer::{
    LlmOptimizer, LlmOptimizationConfig,
    OptimizationStrategy, PromptTemplateType
};

// 创建优化器
let config = LlmOptimizationConfig::default();
let mut optimizer = LlmOptimizer::new(config);

// 优化请求
let mut variables = HashMap::new();
variables.insert("text".to_string(), "记忆内容".to_string());

let response = optimizer
    .optimize_request(
        PromptTemplateType::MemoryExtraction,
        variables,
        &llm_provider
    )
    .await?;

println!("优化后的提示: {}", response.content);

// 查看缓存统计
let (cache_size, hits, misses) = optimizer.get_cache_stats();
println!("缓存大小: {}, 命中: {}, 未命中: {}", cache_size, hits, misses);
```

#### 7. 性能优化器 (PerformanceOptimizer)

```rust
use agent_mem_core::performance::optimizer::PerformanceOptimizer;

let optimizer = PerformanceOptimizer::new(config);

// 批量优化
let tasks = vec![task1, task2, task3];
let results = optimizer
    .execute_batch_optimized(tasks)
    .await?;

// 并发优化
let results = optimizer
    .execute_parallel_optimized(queries)
    .await?;
```

#### 8. 多模态处理 (MultimodalProcessor)

```rust
#[cfg(feature = "multimodal")]
use agent_mem_intelligence::multimodal::MultimodalProcessor;

#[cfg(feature = "multimodal")]
let processor = MultimodalProcessor::new(config)?;

// 图像处理
#[cfg(feature = "multimodal")]
let image_memory = processor
    .process_image("path/to/image.jpg")
    .await?;

// 音频处理
#[cfg(feature = "multimodal")]
let audio_memory = processor
    .process_audio("path/to/audio.wav")
    .await?;
```

### P2: 性能优化 API

#### 1. ContextCompressor

```rust
use agent_mem_core::llm_optimizer::{
    ContextCompressor, ContextCompressorConfig
};

// 创建压缩器
let config = ContextCompressorConfig {
    max_context_tokens: 3000,
    target_compression_ratio: 0.7,
    preserve_important_memories: true,
    importance_threshold: 0.7,
    enable_deduplication: true,
    dedup_threshold: 0.85,
};

let compressor = ContextCompressor::new(config);

// 压缩上下文
let result = compressor.compress_context(query, &memories)?;

println!("压缩统计:");
println!("  原始 Token: {}", result.original_tokens);
println!("  压缩 Token: {}", result.compressed_tokens);
println!("  压缩比: {:.1}%", result.compression_ratio * 100.0);
println!("  移除记忆: {}", result.memories_removed);
println!("  保留记忆: {}", result.memories_preserved);
println!("  去重节省: {}", result.duplication_savings);

// 使用压缩后的上下文
let compressed_context = result.compressed_context;
```

#### 2. MultiLevelCache

```rust
use agent_mem_core::llm_optimizer::{
    MultiLevelCache, MultiLevelCacheConfig, CacheLevelConfig
};

// 创建多级缓存
let config = MultiLevelCacheConfig {
    l1: CacheLevelConfig {
        size: 100,
        ttl_seconds: 300,  // 5 分钟
        enabled: true,
    },
    l2: CacheLevelConfig {
        size: 1000,
        ttl_seconds: 1800,  // 30 分钟
        enabled: true,
    },
    l3: CacheLevelConfig {
        size: 10000,
        ttl_seconds: 7200,  // 2 小时
        enabled: true,
    },
};

let cache = MultiLevelCache::new(config);

// 写入缓存
cache.set("key1".to_string(), "value1".to_string()).await;

// 读取缓存（自动 L1 → L2 → L3）
if let Some(value) = cache.get("key1").await {
    println!("缓存命中: {}", value);
}

// 失效缓存
cache.invalidate("key1").await;

// 清空缓存
cache.clear().await;

// 获取统计信息
let stats = cache.stats().await;
println!("统计: {:?}", stats);
```

#### 3. 集成到 LlmOptimizer

```rust
use agent_mem_core::llm_optimizer::{
    LlmOptimizer, LlmOptimizationConfig, ContextCompressorConfig
};

// 创建带压缩的优化器
let config = LlmOptimizationConfig::default();
let optimizer = LlmOptimizer::new(config)
    .with_context_compressor(ContextCompressorConfig::default());

// 压缩上下文
let result = optimizer.compress_context(query, &memories)?;
```

### P3: 插件 API

#### 插件管理器

```rust
use agent_mem_plugins::{PluginManager, PluginRegistry, RegisteredPlugin};

// 创建插件管理器
let manager = PluginManager::new(10);  // LRU 缓存大小

// 注册插件
let plugin = WeatherPlugin::new(api_key);
let registered = manager.register(plugin).await?;

// 列出插件
let plugins = manager.list_plugins().await;
for plugin_info in plugins {
    println!("插件: {} ({})", plugin_info.name, plugin_info.id);
}

// 调用插件
let input = r#"{"content": "今天是晴天"}"#;
let output = manager
    .call_plugin(&plugin_id, "process_memory", input)
    .await?;

// 卸载插件
manager.unregister(&plugin_id).await?;

// 获取插件状态
let status = manager.get_plugin_status(&plugin_id).await?;
println!("状态: {:?}", status);
```

#### 插件开发

```rust
use agent_mem_plugins::sdk::*;
use agent_mem_traits::{Memory, Result};
use async_trait::async_trait;

/// 定义插件元数据
#[plugin]
pub struct MyPlugin {
    name: String,
    version: String,
}

impl MyPlugin {
    pub fn new() -> Self {
        Self {
            name: "MyPlugin".to_string(),
            version: "1.0.0".to_string(),
        }
    }
}

/// 实现 MemoryProcessorPlugin trait
#[async_trait]
impl MemoryProcessorPlugin for MyPlugin {
    async fn process_memory(&self, memory: &mut Memory) -> Result<()> {
        // 处理记忆内容
        let content = memory.content.to_string();

        // 添加自定义属性
        memory.attributes.insert(
            AttributeKey::from("processed_by"),
            AttributeValue::String(self.name.clone())
        );

        Ok(())
    }
}

/// 实现 Plugin trait
impl Plugin for MyPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: self.name.clone(),
            version: self.version.clone(),
            description: "我的自定义插件".to_string(),
            author: "Your Name".to_string(),
            plugin_type: PluginType::MemoryProcessor,
            capabilities: vec![Capability::MemoryProcess],
        }
    }

    async fn initialize(&mut self) -> Result<()> {
        // 初始化逻辑
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        // 清理逻辑
        Ok(())
    }
}
```

---

## 常见场景

### 场景 1: 构建聊天机器人记忆系统

```rust
use agent_mem_core::{
    AgentOrchestrator, OrchestratorConfig,
    Memory, MemoryEngine
};
use agent_mem_traits::{AttributeKey, AttributeValue};

#[tokio::main]
async fn main() -> Result<()> {
    // 1. 创建编排器
    let config = OrchestratorConfig::default();
    let orchestrator = AgentOrchestrator::new(config).await?;

    // 2. 用户发送消息
    let user_message = "我上周学习了 Rust 语言";

    // 3. 创建记忆并添加
    let memory = Memory::builder()
        .content(user_message)
        .attribute("importance", 0.8)
        .attribute("category", "学习")
        .attribute("timestamp", Utc::now())
        .build();

    orchestrator.add_memory(memory, user_id).await?;

    // 4. 获取上下文
    let context = orchestrator
        .get_context_for_chat(user_id, agent_id, 5)
        .await?;

    // 5. 生成回复
    let response = orchestrator
        .chat(&format!("上下文: {}\n用户: {}", context, user_message))
        .await?;

    println!("AI: {}", response.message);

    Ok(())
}
```

### 场景 2: 项目管理系统

```rust
use agent_mem_core::{
    Memory, MemoryEngine, MemoryEngineConfig,
    temporal_reasoning::TemporalReasoningEngine
};
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<()> {
    let config = MemoryEngineConfig::default();
    let engine = MemoryEngine::new(config).await?;

    // 1. 记录项目事件
    let events = vec![
        ("项目启动", "项目", "开始"),
        ("完成设计", "项目", "设计"),
        ("开始开发", "项目", "开发"),
    ];

    for (description, category, status) in events {
        let memory = Memory::builder()
            .content(description)
            .attribute("category", category)
            .attribute("status", status)
            .attribute("timestamp", Utc::now())
            .build();

        engine.add(memory).await?;
    }

    // 2. 时序查询
    let temporal_engine = TemporalReasoningEngine::new(config)?;
    let timeline = temporal_engine
        .query_by_range("最近一周", &scope)
        .await?;

    println!("项目时间线:");
    for event in timeline {
        println!("  - {}", event.content);
    }

    // 3. 进度分析
    let completed = engine
        .search("项目 状态:完成", None, Some(100))
        .await?;

    println!("已完成事件: {}", completed.len());

    Ok(())
}
```

### 场景 3: 知识图谱构建

```rust
use agent_mem_core::{
    Memory, MemoryEngine,
    graph_memory::GraphMemoryEngine
};

#[tokio::main]
async fn main() -> Result<()> {
    let config = MemoryEngineConfig::default();
    let engine = MemoryEngine::new(config).await?;
    let graph_engine = GraphMemoryEngine::new(config)?;

    // 1. 添加实体和关系
    let rust = Memory::builder()
        .content("Rust")
        .attribute("type", "programming_language")
        .build();

    let memory = Memory::builder()
        .content("AgentMem")
        .attribute("type", "project")
        .attribute("implemented_in", "Rust")
        .build();

    engine.add(rust).await?;
    engine.add(memory).await?;

    // 2. 构建关系图
    graph_engine.build_relation_graph(&scope).await?;

    // 3. 关系推理
    let relations = graph_engine
        .infer_relations(memory.id.clone())
        .await?;

    println!("AgentMem 的关系:");
    for relation in relations {
        println!("  - {:?}", relation);
    }

    // 4. 图遍历
    let graph = graph_engine
        .graph_traverse(memory.id, 2)
        .await?;

    println!("相关概念:");
    for node in graph {
        println!("  - {}", node.content);
    }

    Ok(())
}
```

### 场景 4: 性能优化

```rust
use agent_mem_core::llm_optimizer::{
    LlmOptimizer, LlmOptimizationConfig,
    ContextCompressorConfig, MultiLevelCacheConfig
};

#[tokio::main]
async fn main() -> Result<()> {
    // 1. 创建优化配置
    let config = LlmOptimizationConfig {
        enable_caching: true,
        enable_prompt_optimization: true,
        enable_cost_tracking: true,
        ..Default::default()
    };

    // 2. 创建优化器（带压缩和缓存）
    let optimizer = LlmOptimizer::new(config)
        .with_context_compressor(ContextCompressorConfig::default());

    // 3. 模拟大量记忆
    let memories: Vec<Memory> = (0..1000)
        .map(|i| Memory::builder().content(format!("记忆 {}", i)).build())
        .collect();

    // 4. 压缩上下文
    let query = "最近重要的工作是什么?";
    let result = optimizer.compress_context(query, &memories)?;

    println!("优化效果:");
    println!("  Token 减少: {:.1}%",
        (1.0 - result.compression_ratio) * 100.0);
    println!("  记忆过滤: {} -> {}",
        memories.len(), result.memories_preserved);

    // 5. 使用优化后的提示
    let optimized_prompt = optimizer.optimize_prompt(
        PromptTemplateType::MemoryContext,
        &result.compressed_context
    )?;

    // 6. 调用 LLM（会自动缓存）
    let response = llm_provider.generate(&optimized_prompt).await?;

    // 7. 查看性能统计
    let metrics = optimizer.get_performance_metrics();
    println!("性能统计:");
    println!("  缓存命中率: {:.1}%",
        metrics.cache_hits as f64 / (metrics.cache_hits + metrics.cache_misses) as f64 * 100.0);
    println!("  平均响应时间: {:?}", metrics.average_response_time);
    println!("  总成本: ${:.2}", metrics.total_cost);

    Ok(())
}
```

---

## 故障排除

### 常见错误

#### 1. 内存不足

```rust
// ❌ 一次性加载太多记忆
let all_memories = engine.get_all(&session).await?;  // 可能很大

// ✅ 使用分页和过滤
let memories = engine
    .search("查询", Some(scope), Some(100))
    .await?;
```

#### 2. 搜索太慢

```rust
// ❌ 没有使用索引
let results = engine.search_slow(query).await?;

// ✅ 使用调度器和缓存
let scheduler = DefaultMemoryScheduler::new(config);
let engine = MemoryEngine::new(config)
    .await?
    .with_scheduler(scheduler);

let results = engine
    .search_with_scheduler(query, Some(scope), Some(10))
    .await?;
```

#### 3. Token 超限

```rust
// ❌ 直接传递大量记忆给 LLM
let context = memories.iter()
    .map(|m| m.content.to_string())
    .collect::<Vec<_>>()
    .join("\n");

// ✅ 使用上下文压缩
let optimizer = LlmOptimizer::new(config)
    .with_context_compressor(ContextCompressorConfig::default());

let result = optimizer.compress_context(query, &memories)?;
let compressed_context = result.compressed_context;
```

#### 4. 插件加载失败

```rust
// ❌ 没有错误处理
manager.load_plugin("path/to/plugin.so").await?;

// ✅ 适当的错误处理
match manager.load_plugin("path/to/plugin.so").await {
    Ok(_) => println!("插件加载成功"),
    Err(e) => {
        eprintln!("插件加载失败: {:?}", e);
        // 使用默认行为继续
    }
}
```

### 调试技巧

#### 1. 启用日志

```rust
use tracing::{info, debug, error};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // 使用日志
    debug!("开始搜索: {}", query);
    let results = engine.search(query, None, Some(10)).await?;
    info!("找到 {} 条结果", results.len());

    Ok(())
}
```

#### 2. 性能分析

```rust
use std::time::Instant;

let start = Instant::now();
let results = engine.search(query, None, Some(10)).await?;
let duration = start.elapsed();

debug!("搜索耗时: {:?}", duration);

if duration.as_millis() > 100 {
    warn!("搜索耗时超过 100ms");
}
```

#### 3. 内存监控

```rust
// 检查缓存大小
let stats = optimizer.get_cache_stats();
info!("缓存大小: {}", stats.0);

// 检查记忆数量
let count = engine.count(&scope).await?;
info!("记忆数量: {}", count);

// 清理不必要的缓存
if stats.0 > 1000 {
    optimizer.clear_cache();
}
```

### 性能优化建议

1. **使用批处理**
   - 批量添加记忆
   - 批量删除
   - 批量更新

2. **启用缓存**
   - LLM 响应缓存
   - 搜索结果缓存
   - Embedding 缓存

3. **限制返回数量**
   - 搜索时使用合理的 limit
   - 分页获取大量数据

4. **使用调度器**
   - 自动过滤低价值记忆
   - 提高搜索相关性

5. **压缩上下文**
   - 使用 ContextCompressor
   - 减少 Token 使用

---

## 总结

### API 快速参考

| 功能 | API | 代码示例 |
|------|-----|----------|
| 创建记忆 | `Memory::builder()` | `Memory::builder().content("...").build()` |
| 创建引擎 | `MemoryEngine::new()` | `MemoryEngine::new(config).await?` |
| 添加记忆 | `engine.add()` | `engine.add(memory).await?` |
| 搜索记忆 | `engine.search()` | `engine.search("query", None, Some(10)).await?` |
| 时序查询 | `orchestrator.temporal_query()` | `temporal_query("上周", ...).await?` |
| 因果推理 | `orchestrator.explain_causality()` | `explain_causality("为什么...").await?` |
| 图遍历 | `orchestrator.graph_traverse()` | `graph_traverse(id, 3).await?` |
| 压缩上下文 | `optimizer.compress_context()` | `compress_context(query, &memories)?` |
| 插件调用 | `manager.call_plugin()` | `call_plugin(id, method, input).await?` |

### 最佳实践

1. ✅ 使用 Builder 模式创建对象
2. ✅ 使用 `?` 传播错误
3. ✅ 使用 `.await` 等待异步操作
4. ✅ 限制搜索结果数量
5. ✅ 使用上下文压缩减少 Token
6. ✅ 启用缓存提高性能
7. ✅ 记录日志便于调试
8. ✅ 批量操作提高效率

### 获取帮助

- 📖 架构文档: `claudedocs/agentmem_26_architecture.md`
- 💻 示例代码: `examples/` 目录
- 🧪 测试代码: `tests/` 目录
- 📝 Rustdoc: `cargo doc --open`

**Happy Coding! 🚀**
