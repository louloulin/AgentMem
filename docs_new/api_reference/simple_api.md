# Level 1: 简单 API

AgentMem 的最简单 API，适合快速开始和小型项目。

## 核心类型

### Memory

记忆管理的核心接口。

```rust
use agentmem::Memory;

// 创建实例
let memory = Memory::new().await?;
```

## 基础方法

### add()

添加一条记忆。

```rust
let result = memory.add("用户喜欢咖啡").await?;
```

**参数**:
- `content: &str` - 记忆内容

**返回**:
- `Result<AddResult>`

**示例**:
```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let memory = Memory::new().await?;

    // 添加简单记忆
    let result = memory.add("用户喜欢咖啡").await?;

    println!("记忆 ID: {}", result.id);
    println!("重要性: {:.2}", result.importance);

    Ok(())
}
```

### search()

搜索记忆。

```rust
let results = memory.search("饮品").await?;
```

**参数**:
- `query: &str` - 搜索查询

**返回**:
- `Result<Vec<SearchResult>>`

**示例**:
```rust
let results = memory.search("用户的爱好？").await?;

for (i, result) in results.iter().enumerate() {
    println!("{}. {} (相似度: {:.2})",
        i + 1,
        result.content,
        result.score.unwrap_or(0.0)
    );
}
```

### get_all()

获取所有记忆。

```rust
let memories = memory.get_all(GetAllOptions::default()).await?;
```

**参数**:
- `options: GetAllOptions` - 查询选项

**返回**:
- `Result<Vec<Memory>>`

**示例**:
```rust
use agentmem::GetAllOptions;

// 获取所有记忆
let all = memory.get_all(GetAllOptions::default()).await?;

// 限制数量
let limited = memory.get_all(
    GetAllOptions::builder()
        .limit(10)
        .build()
).await?;
```

### delete()

删除一条记忆。

```rust
memory.delete("memory_id").await?;
```

**参数**:
- `id: &str` - 记忆 ID

**返回**:
- `Result<()>`

**示例**:
```rust
let result = memory.add("临时记忆").await?;
memory.delete(&result.id).await?;
```

### delete_all()

删除所有记忆。

```rust
memory.delete_all(DeleteAllOptions::default()).await?;
```

**参数**:
- `options: DeleteAllOptions` - 删除选项

**返回**:
- `Result<()>`

**示例**:
```rust
use agentmem::{DeleteAllOptions, MemoryType};

// 删除特定类型
memory.delete_all(
    DeleteAllOptions::builder()
        .memory_type(MemoryType::CORE)
        .build()
).await?;
```

## Builder 模式

### Memory::builder()

使用 Builder 模式创建配置化的 Memory 实例。

```rust
let memory = Memory::builder()
    .infer(true)  // 启用智能功能
    .build()
    .await?;
```

### 配置选项

#### infer()

启用或禁用智能推断。

```rust
// 启用智能功能（需要 API Key）
let memory = Memory::builder()
    .infer(true)
    .build()
    .await?;

// 禁用智能功能（无需 API Key）
let memory = Memory::builder()
    .infer(false)
    .build()
    .await?;
```

#### with_storage()

设置存储后端。

```rust
// LibSQL（默认，零配置）
let memory = Memory::builder()
    .with_storage("libsql://./agentmem.db")
    .build()
    .await?;

// PostgreSQL
let memory = Memory::builder()
    .with_storage("postgresql://user:password@localhost:5432/agentmem")
    .build()
    .await?;
```

#### with_llm()

设置 LLM 提供商。

```rust
// OpenAI
let memory = Memory::builder()
    .with_llm("openai", "gpt-4")
    .build()
    .await?;

// Anthropic
let memory = Memory::builder()
    .with_llm("anthropic", "claude-3-opus-20240229")
    .build()
    .await?;
```

#### with_embedder()

设置嵌入模型。

```rust
// OpenAI
let memory = Memory::builder()
    .with_embedder("openai", "text-embedding-3-small")
    .build()
    .await?;

// FastEmbed（本地）
let memory = Memory::builder()
    .with_embedder("fastembed", "BAAI/bge-small-en-v1.5")
    .build()
    .await?;
```

## 类型定义

### AddResult

添加操作的结果。

```rust
pub struct AddResult {
    pub id: String,          // 记忆 ID
    pub score: f32,          // 相似度分数（如果去重）
    pub importance: f32,     // 重要性评分
}
```

### SearchResult

搜索结果。

```rust
pub struct SearchResult {
    pub id: String,          // 记忆 ID
    pub content: String,     // 记忆内容
    pub score: Option<f32>,  // 相似度分数
}
```

### MemoryType

记忆类型。

```rust
pub enum MemoryType {
    CORE,        // 核心记忆
    WORKING,     // 工作记忆
    EPISODIC,    // 情节记忆
    SEMANTIC,    // 语义记忆
    PROCEDURAL,  // 程序记忆
    RESOURCE,    // 资源记忆
}
```

## 完整示例

```rust
use agentmem::{Memory, MemoryType, GetAllOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 创建实例
    let memory = Memory::new().await?;

    // 2. 添加记忆
    let result1 = memory.add("用户喜欢咖啡").await?;
    println!("添加记忆: {} (重要性: {:.2})",
        result1.id, result1.importance);

    let result2 = memory.add("用户住在上海").await?;
    println!("添加记忆: {}", result2.id);

    // 3. 搜索记忆
    let results = memory.search("饮品偏好").await?;
    println!("\n搜索结果:");
    for (i, result) in results.iter().enumerate() {
        println!("  {}. {} (相似度: {:.2})",
            i + 1,
            result.content,
            result.score.unwrap_or(0.0)
        );
    }

    // 4. 获取所有记忆
    let all = memory.get_all(GetAllOptions::default()).await?;
    println!("\n所有记忆 ({} 条):", all.len());
    for (i, mem) in all.iter().enumerate() {
        println!("  {}. {}", i + 1, mem.content);
    }

    // 5. 清理
    memory.delete_all(Default::default()).await?;
    println!("\n已清理所有记忆");

    Ok(())
}
```

## Python SDK

### 基础使用

```python
from agentmem import Memory

# 创建实例
memory = await Memory.new()

# 添加记忆
result = await memory.add("用户喜欢咖啡")

# 搜索记忆
results = await memory.search("饮品偏好")

for i, result in enumerate(results):
    print(f"{i+1}. {result.content} (相似度: {result.score:.2f})")
```

### 配置选项

```python
from agentmem import Memory

# 使用 Builder 模式
memory = await Memory.builder() \
    .infer(True) \
    .with_storage("postgresql://user:password@localhost/agentmem") \
    .with_llm("openai", "gpt-4") \
    .with_embedder("openai", "text-embedding-3-small") \
    .build()
```

## 下一步

- [标准 API](standard_api.md) - 更多功能和选项
- [高级 API](advanced_api.md) - 高级特性和自定义
