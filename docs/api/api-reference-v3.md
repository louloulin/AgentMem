# AgentMem API 参考文档 v3.0

**版本**: v3.0.0  
**更新日期**: 2025-12-10  
**状态**: ✅ Phase 3.1 API简化已完成

---

## 📋 目录

1. [快速开始](#快速开始)
2. [零配置启动](#零配置启动)
3. [核心API](#核心api)
4. [链式调用API](#链式调用api)
5. [智能默认值](#智能默认值)
6. [错误处理](#错误处理)
7. [高级配置](#高级配置)

---

## 🚀 快速开始

### 最简单的使用方式（零配置）

```rust
use agent_mem::Memory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 零配置启动 - 自动检测环境变量
    let mem = Memory::new_smart().await?;
    
    // 添加记忆
    mem.add("我喜欢Rust编程语言").await?;
    
    // 搜索记忆
    let results = mem.search("编程").await?;
    for result in results {
        println!("- {}", result.content);
    }
    
    Ok(())
}
```

---

## 🎯 零配置启动

### `Memory::new_smart()`

自动检测环境变量并应用智能默认值。

**环境变量检测**:
- `OPENAI_API_KEY` → 使用OpenAI LLM
- `DEEPSEEK_API_KEY` → 使用DeepSeek LLM
- `DATABASE_URL` → 使用指定数据库
- `REDIS_URL` → 启用Redis缓存

**智能默认值**:
- 存储: LibSQL嵌入式数据库（如果未指定）
- 嵌入模型: FastEmbed本地模型（如果未指定LLM）
- 缓存: 内存缓存（如果未指定Redis）

**示例**:
```rust
// 自动检测环境变量
let mem = Memory::new_smart().await?;

// 等价于（如果检测到OPENAI_API_KEY和DATABASE_URL）:
let mem = Memory::builder()
    .with_llm_provider("openai")
    .with_storage("libsql://agentmem.db")
    .build()
    .await?;
```

---

## 📚 核心API

### 添加记忆

```rust
// 简单添加
mem.add("用户喜欢披萨").await?;

// 带选项添加
mem.add_with_options("用户喜欢披萨", AddOptions {
    user_id: Some("user-123".to_string()),
    agent_id: Some("agent-1".to_string()),
    memory_type: Some("episodic".to_string()),
    importance: Some(0.8),
    metadata: Some(serde_json::json!({
        "source": "conversation",
        "timestamp": "2025-12-10"
    })),
}).await?;
```

### 搜索记忆

```rust
// 简单搜索
let results = mem.search("用户喜欢什么").await?;

// 高级搜索
let results = mem.search_with_options(SearchOptions {
    query: "用户喜欢什么".to_string(),
    agent_id: Some("agent-1".to_string()),
    user_id: Some("user-123".to_string()),
    limit: Some(10),
    min_importance: Some(0.5),
    memory_type: Some("episodic".to_string()),
}).await?;
```

### 获取记忆

```rust
// 根据ID获取
let memory = mem.get("mem-123").await?;

// 列出所有记忆
let memories = mem.list(ListOptions {
    agent_id: Some("agent-1".to_string()),
    limit: Some(20),
}).await?;
```

### 更新记忆

```rust
mem.update("mem-123", UpdateOptions {
    content: Some("用户非常喜欢披萨".to_string()),
    importance: Some(0.9),
}).await?;
```

### 删除记忆

```rust
mem.delete("mem-123").await?;
```

---

## 🔗 链式调用API

### `FluentMemory` - 链式调用支持

```rust
use agent_mem::FluentMemory;

let mem = Memory::new_smart().await?;
let fluent = FluentMemory::new(mem);

// 链式调用
let results = fluent
    .add("我喜欢Rust")
    .await?
    .add("我喜欢Python")
    .await?
    .search("编程语言")
    .await?;

for result in results {
    println!("- {}", result.content);
}
```

**优势**:
- ✅ 代码更简洁
- ✅ 减少中间变量
- ✅ 更符合函数式编程风格

---

## 🎨 智能默认值

### `SmartDefaults` - 自动配置检测

```rust
use agent_mem::api_simplification::SmartDefaults;

let defaults = SmartDefaults::detect().await?;

println!("检测到的配置:");
println!("- LLM Provider: {:?}", defaults.llm_provider);
println!("- Storage: {:?}", defaults.storage);
println!("- Embedder: {:?}", defaults.embedder);
println!("- Cache: {:?}", defaults.cache);

// 使用检测到的默认值创建Memory
let mem = Memory::with_smart_defaults(defaults).await?;
```

**检测逻辑**:
1. 检查环境变量（`OPENAI_API_KEY`, `DEEPSEEK_API_KEY`等）
2. 检查配置文件（`~/.agentmem/config.toml`）
3. 检查系统默认值
4. 提供建议配置

---

## ⚠️ 错误处理

### `EnhancedError` - 友好的错误信息

```rust
use agent_mem::api_simplification::{EnhancedError, ErrorEnhancer};

match mem.add("test").await {
    Ok(_) => println!("成功"),
    Err(e) => {
        let enhanced = ErrorEnhancer::enhance(e);
        println!("错误: {}", enhanced.message());
        if let Some(suggestion) = enhanced.suggestion() {
            println!("建议: {}", suggestion);
        }
    }
}
```

**错误类型**:
- `MemoryError` - 记忆操作错误
- `StorageError` - 存储错误
- `LLMError` - LLM服务错误
- `NetworkError` - 网络错误
- `ConfigError` - 配置错误

**错误恢复建议**:
- 自动提供恢复建议
- 包含相关文档链接
- 提供代码示例

---

## ⚙️ 高级配置

### Builder模式（完整配置）

```rust
let mem = Memory::builder()
    .with_storage("postgresql://localhost/agentmem")
    .with_llm_provider("openai")
    .with_llm_model("gpt-4")
    .with_embedder("openai")
    .with_embedder_model("text-embedding-3-small")
    .with_redis_cache("redis://localhost:6379")
    .enable_intelligent_features()
    .with_batch_size(100)
    .build()
    .await?;
```

### 配置选项

| 选项 | 说明 | 默认值 |
|------|------|--------|
| `storage` | 存储后端 | LibSQL嵌入式 |
| `llm_provider` | LLM提供商 | 自动检测 |
| `llm_model` | LLM模型 | gpt-3.5-turbo |
| `embedder` | 嵌入模型 | FastEmbed |
| `cache` | 缓存后端 | 内存缓存 |
| `batch_size` | 批量操作大小 | 50 |
| `enable_intelligent_features` | 启用智能功能 | false |

---

## 📖 完整示例

### 示例1: 零配置快速开始

```rust
use agent_mem::Memory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mem = Memory::new_smart().await?;
    
    // 添加记忆
    mem.add("用户Alice喜欢咖啡").await?;
    mem.add("用户Bob喜欢茶").await?;
    
    // 搜索
    let results = mem.search("Alice喜欢什么").await?;
    println!("找到 {} 条相关记忆", results.len());
    
    Ok(())
}
```

### 示例2: 链式调用

```rust
use agent_mem::{Memory, FluentMemory};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mem = Memory::new_smart().await?;
    let fluent = FluentMemory::new(mem);
    
    let results = fluent
        .add("今天学习了Rust")
        .await?
        .add("明天计划学习Python")
        .await?
        .search("学习计划")
        .await?;
    
    for result in results {
        println!("- {}", result.content);
    }
    
    Ok(())
}
```

### 示例3: 错误处理

```rust
use agent_mem::{Memory, EnhancedError};

#[tokio::main]
async fn main() {
    let mem = Memory::new_smart().await.unwrap();
    
    match mem.add("test").await {
        Ok(_) => println!("✅ 成功"),
        Err(e) => {
            let enhanced = ErrorEnhancer::enhance(e);
            eprintln!("❌ 错误: {}", enhanced.message());
            if let Some(suggestion) = enhanced.suggestion() {
                eprintln!("💡 建议: {}", suggestion);
            }
        }
    }
}
```

---

## 🔗 相关文档

- [架构文档](../architecture/architecture-overview.md)
- [最佳实践指南](../guides/best-practices.md)
- [示例集合](../../examples/README.md)
- [CLI工具文档](../../tools/agentmem-cli/README.md)

---

## 📝 更新日志

### v3.0.0 (2025-12-10)
- ✅ 新增 `Memory::new_smart()` 零配置启动
- ✅ 新增 `FluentMemory` 链式调用支持
- ✅ 新增 `SmartDefaults` 智能默认值检测
- ✅ 新增 `EnhancedError` 友好错误处理
- ✅ 完善API文档和示例

---

**文档维护**: AgentMem Team  
**反馈**: https://github.com/agentmem/agentmem/issues

