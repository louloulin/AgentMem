# AgentMem API 参考文档

## 📋 API 概述

AgentMem 提供了多层次的 API 接口，支持不同编程语言和使用场景：

- **Rust API**: 原生高性能接口
- **Zig API**: 零成本抽象层
- **C FFI**: 跨语言互操作接口
- **多语言绑定**: Python、JavaScript、Go 等

## 🦀 Rust API

### 核心数据库类

#### `AgentDatabase`

主要的数据库操作类，提供完整的 Agent 状态管理功能。

```rust
pub struct AgentDatabase {
    pub agent_state_db: AgentStateDB,
    pub memory_manager: MemoryManager,
    pub vector_engine: Option<AdvancedVectorEngine>,
    pub security_manager: Option<SecurityManager>,
    pub rag_engine: Option<RAGEngine>,
    pub config: DatabaseConfig,
}
```

#### 构造方法

```rust
// 创建基础数据库实例
pub async fn new(config: DatabaseConfig) -> Result<Self, AgentMemError>

// 添加向量搜索引擎
pub async fn with_vector_engine(self, config: VectorIndexConfig) -> Result<Self, AgentMemError>

// 添加安全管理器
pub fn with_security_manager(self) -> Self

// 添加RAG引擎
pub async fn with_rag_engine(self) -> Result<Self, AgentMemError>
```

#### 使用示例

```rust
use agent_db::{AgentDatabase, DatabaseConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建配置
    let config = DatabaseConfig {
        db_path: "./agent_db".to_string(),
        ..Default::default()
    };
    
    // 创建数据库实例
    let db = AgentDatabase::new(config).await?
        .with_vector_engine(Default::default()).await?
        .with_security_manager()
        .with_rag_engine().await?;
    
    Ok(())
}
```

### Agent状态操作

#### 保存Agent状态

```rust
pub async fn save_agent_state(&self, state: &AgentState) -> Result<(), AgentMemError>
```

**参数**:
- `state`: Agent状态对象

**示例**:
```rust
let state = AgentState::new(
    12345,                    // agent_id
    67890,                    // session_id
    StateType::WorkingMemory, // state_type
    b"agent state data".to_vec() // data
);

db.save_agent_state(&state).await?;
```

#### 加载Agent状态

```rust
pub async fn load_agent_state(&self, agent_id: u64) -> Result<Option<AgentState>, AgentMemError>
```

**参数**:
- `agent_id`: Agent唯一标识符

**返回值**:
- `Some(AgentState)`: 找到的状态
- `None`: 未找到状态

**示例**:
```rust
if let Some(state) = db.load_agent_state(12345).await? {
    println!("找到Agent状态: {:?}", state);
} else {
    println!("未找到Agent状态");
}
```

### 记忆管理操作

#### 存储记忆

```rust
pub async fn store_memory(&self, memory: &Memory) -> Result<(), AgentMemError>
```

**参数**:
- `memory`: 记忆对象

**示例**:
```rust
let memory = Memory::new(
    12345,                           // agent_id
    MemoryType::Episodic,           // memory_type
    "重要的对话内容".to_string(),      // content
    0.8                             // importance
);

db.store_memory(&memory).await?;
```

#### 获取记忆

```rust
pub async fn get_memories(&self, agent_id: u64) -> Result<Vec<Memory>, AgentMemError>
```

**参数**:
- `agent_id`: Agent唯一标识符

**返回值**:
- `Vec<Memory>`: 记忆列表

### 向量操作

#### 添加向量

```rust
pub async fn add_vector(
    &self, 
    id: u64, 
    vector: Vec<f32>, 
    metadata: HashMap<String, String>
) -> Result<(), AgentMemError>
```

#### 向量搜索

```rust
pub async fn search_vectors(
    &self, 
    query: &[f32], 
    limit: usize
) -> Result<Vec<VectorSearchResult>, AgentMemError>
```

### RAG操作

#### 索引文档

```rust
pub async fn index_document(&self, document: &Document) -> Result<String, AgentMemError>
```

#### 搜索文档

```rust
pub async fn search_documents(
    &self, 
    query: &str, 
    limit: usize
) -> Result<Vec<SearchResult>, AgentMemError>
```

#### 语义搜索

```rust
pub async fn semantic_search_documents(
    &self, 
    query_embedding: Vec<f32>, 
    limit: usize
) -> Result<Vec<SearchResult>, AgentMemError>
```

## ⚡ Zig API

### Agent状态管理

#### `AgentState` 结构体

```zig
pub const AgentState = struct {
    agent_id: u64,
    session_id: u64,
    state_type: StateType,
    data: []const u8,
    checksum: []const u8,
    created_at: i64,
    updated_at: i64,
    
    pub fn init(
        allocator: std.mem.Allocator,
        agent_id: u64,
        session_id: u64,
        state_type: StateType,
        data: []const u8,
    ) !AgentState
    
    pub fn deinit(self: *AgentState, allocator: std.mem.Allocator) void
    
    pub fn updateData(self: *AgentState, allocator: std.mem.Allocator, new_data: []const u8) !void
    
    pub fn setMetadata(self: *AgentState, allocator: std.mem.Allocator, key: []const u8, value: []const u8) !void
    
    pub fn createSnapshot(self: AgentState, allocator: std.mem.Allocator, name: []const u8) !AgentStateSnapshot
};
```

#### 使用示例

```zig
const std = @import("std");
const AgentState = @import("agent_state.zig").AgentState;
const StateType = @import("agent_state.zig").StateType;

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    
    // 创建Agent状态
    var state = try AgentState.init(
        allocator,
        12345,
        67890,
        .working_memory,
        "agent state data"
    );
    defer state.deinit(allocator);
    
    // 更新数据
    try state.updateData(allocator, "updated data");
    
    // 设置元数据
    try state.setMetadata(allocator, "priority", "high");
    
    // 创建快照
    var snapshot = try state.createSnapshot(allocator, "backup_v1");
    defer snapshot.deinit(allocator);
}
```

### 记忆管理

#### `Memory` 结构体

```zig
pub const Memory = struct {
    agent_id: u64,
    memory_type: MemoryType,
    content: []const u8,
    importance: f32,
    timestamp: i64,
    metadata: std.StringHashMap([]const u8),
    
    pub fn init(
        allocator: std.mem.Allocator,
        agent_id: u64,
        memory_type: MemoryType,
        content: []const u8,
        importance: f32,
    ) !Memory
    
    pub fn deinit(self: *Memory, allocator: std.mem.Allocator) void
    
    pub fn updateImportance(self: *Memory, new_importance: f32) void
    
    pub fn addMetadata(self: *Memory, allocator: std.mem.Allocator, key: []const u8, value: []const u8) !void
};
```

## 🔗 C FFI API

### 基础函数

#### 数据库操作

```c
// 创建数据库实例
CAgentStateDB* agent_db_new(const char* db_path);

// 释放数据库实例
void agent_db_free(CAgentStateDB* db);

// 保存Agent状态
int agent_db_save_state(
    CAgentStateDB* db,
    uint64_t agent_id,
    uint64_t session_id,
    uint32_t state_type,
    const uint8_t* data,
    size_t data_len
);

// 加载Agent状态
int agent_db_load_state(
    CAgentStateDB* db,
    uint64_t agent_id,
    uint8_t** data,
    size_t* data_len
);

// 释放数据内存
void agent_db_free_data(uint8_t* data, size_t data_len);
```

#### 使用示例

```c
#include "agent_state_db.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main() {
    // 创建数据库
    CAgentStateDB* db = agent_db_new("./test_db");
    if (!db) {
        printf("创建数据库失败\n");
        return 1;
    }
    
    // 准备数据
    const char* data = "test agent state";
    size_t data_len = strlen(data);
    
    // 保存状态
    int result = agent_db_save_state(db, 12345, 67890, 0, 
                                    (const uint8_t*)data, data_len);
    if (result != 0) {
        printf("保存状态失败\n");
        agent_db_free(db);
        return 1;
    }
    
    // 加载状态
    uint8_t* loaded_data;
    size_t loaded_len;
    result = agent_db_load_state(db, 12345, &loaded_data, &loaded_len);
    if (result == 0) {
        printf("加载的数据: %.*s\n", (int)loaded_len, loaded_data);
        agent_db_free_data(loaded_data, loaded_len);
    }
    
    // 清理
    agent_db_free(db);
    return 0;
}
```

## 🐍 Python API (计划中)

### 基础用法

```python
import AgentMem

# 创建数据库
db = AgentMem.AgentDatabase("./agent_db")

# 保存Agent状态
state = AgentMem.AgentState(
    agent_id=12345,
    session_id=67890,
    state_type=AgentMem.StateType.WORKING_MEMORY,
    data=b"agent state data"
)
await db.save_agent_state(state)

# 加载Agent状态
loaded_state = await db.load_agent_state(12345)
if loaded_state:
    print(f"找到状态: {loaded_state}")
```

## 📊 错误处理

### 错误类型

```rust
#[derive(Debug, thiserror::Error)]
pub enum AgentMemError {
    #[error("数据库错误: {0}")]
    Database(String),
    
    #[error("序列化错误: {0}")]
    Serialization(String),
    
    #[error("验证错误: {0}")]
    Validation(String),
    
    #[error("未找到: {0}")]
    NotFound(String),
    
    #[error("内部错误: {0}")]
    Internal(String),
}
```

### 错误处理最佳实践

```rust
match db.load_agent_state(agent_id).await {
    Ok(Some(state)) => {
        // 处理找到的状态
        println!("状态: {:?}", state);
    },
    Ok(None) => {
        // 处理未找到的情况
        println!("未找到Agent状态");
    },
    Err(AgentMemError::Database(msg)) => {
        // 处理数据库错误
        eprintln!("数据库错误: {}", msg);
    },
    Err(e) => {
        // 处理其他错误
        eprintln!("其他错误: {}", e);
    }
}
```

## 🔧 配置选项

### 数据库配置

```rust
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub db_path: String,
    pub max_connections: usize,
    pub connection_timeout: Duration,
    pub query_timeout: Duration,
    pub enable_wal: bool,
    pub cache_size: usize,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            db_path: "./agent_db".to_string(),
            max_connections: 10,
            connection_timeout: Duration::from_secs(30),
            query_timeout: Duration::from_secs(60),
            enable_wal: true,
            cache_size: 1024 * 1024 * 100, // 100MB
        }
    }
}
```

---

**文档版本**: v1.0  
**最后更新**: 2025年6月19日  
**维护者**: AgentMem开发团队
