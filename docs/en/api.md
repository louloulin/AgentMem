# AgentMem API Reference

## 📋 API Overview

AgentMem provides multi-layered API interfaces supporting different programming languages and use cases:

- **Rust API**: Native high-performance interface
- **Zig API**: Zero-cost abstraction layer
- **C FFI**: Cross-language interoperability interface
- **Language Bindings**: Python, JavaScript, Go, etc.

## 🦀 Rust API

### Core Database Class

#### `AgentDatabase`

The main database operation class providing complete Agent state management functionality.

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

#### Constructor Methods

```rust
// Create basic database instance
pub async fn new(config: DatabaseConfig) -> Result<Self, AgentMemError>

// Add vector search engine
pub async fn with_vector_engine(self, config: VectorIndexConfig) -> Result<Self, AgentMemError>

// Add security manager
pub fn with_security_manager(self) -> Self

// Add RAG engine
pub async fn with_rag_engine(self) -> Result<Self, AgentMemError>
```

#### Usage Example

```rust
use agent_db::{AgentDatabase, DatabaseConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create configuration
    let config = DatabaseConfig {
        db_path: "./agent_db".to_string(),
        ..Default::default()
    };
    
    // Create database instance
    let db = AgentDatabase::new(config).await?
        .with_vector_engine(Default::default()).await?
        .with_security_manager()
        .with_rag_engine().await?;
    
    Ok(())
}
```

### Agent State Operations

#### Save Agent State

```rust
pub async fn save_agent_state(&self, state: &AgentState) -> Result<(), AgentMemError>
```

**Parameters**:
- `state`: Agent state object

**Example**:
```rust
let state = AgentState::new(
    12345,                    // agent_id
    67890,                    // session_id
    StateType::WorkingMemory, // state_type
    b"agent state data".to_vec() // data
);

db.save_agent_state(&state).await?;
```

#### Load Agent State

```rust
pub async fn load_agent_state(&self, agent_id: u64) -> Result<Option<AgentState>, AgentMemError>
```

**Parameters**:
- `agent_id`: Agent unique identifier

**Return Value**:
- `Some(AgentState)`: Found state
- `None`: State not found

**Example**:
```rust
if let Some(state) = db.load_agent_state(12345).await? {
    println!("Found agent state: {:?}", state);
} else {
    println!("Agent state not found");
}
```

### Memory Management Operations

#### Store Memory

```rust
pub async fn store_memory(&self, memory: &Memory) -> Result<(), AgentMemError>
```

**Parameters**:
- `memory`: Memory object

**Example**:
```rust
let memory = Memory::new(
    12345,                           // agent_id
    MemoryType::Episodic,           // memory_type
    "Important conversation".to_string(), // content
    0.8                             // importance
);

db.store_memory(&memory).await?;
```

#### Get Memories

```rust
pub async fn get_memories(&self, agent_id: u64) -> Result<Vec<Memory>, AgentMemError>
```

**Parameters**:
- `agent_id`: Agent unique identifier

**Return Value**:
- `Vec<Memory>`: List of memories

### Vector Operations

#### Add Vector

```rust
pub async fn add_vector(
    &self, 
    id: u64, 
    vector: Vec<f32>, 
    metadata: HashMap<String, String>
) -> Result<(), AgentMemError>
```

#### Vector Search

```rust
pub async fn search_vectors(
    &self, 
    query: &[f32], 
    limit: usize
) -> Result<Vec<VectorSearchResult>, AgentMemError>
```

### RAG Operations

#### Index Document

```rust
pub async fn index_document(&self, document: &Document) -> Result<String, AgentMemError>
```

#### Search Documents

```rust
pub async fn search_documents(
    &self, 
    query: &str, 
    limit: usize
) -> Result<Vec<SearchResult>, AgentMemError>
```

#### Semantic Search

```rust
pub async fn semantic_search_documents(
    &self, 
    query_embedding: Vec<f32>, 
    limit: usize
) -> Result<Vec<SearchResult>, AgentMemError>
```

## ⚡ Zig API

### Agent State Management

#### `AgentState` Struct

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

#### Usage Example

```zig
const std = @import("std");
const AgentState = @import("agent_state.zig").AgentState;
const StateType = @import("agent_state.zig").StateType;

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    
    // Create Agent state
    var state = try AgentState.init(
        allocator,
        12345,
        67890,
        .working_memory,
        "agent state data"
    );
    defer state.deinit(allocator);
    
    // Update data
    try state.updateData(allocator, "updated data");
    
    // Set metadata
    try state.setMetadata(allocator, "priority", "high");
    
    // Create snapshot
    var snapshot = try state.createSnapshot(allocator, "backup_v1");
    defer snapshot.deinit(allocator);
}
```

### Memory Management

#### `Memory` Struct

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

### Basic Functions

#### Database Operations

```c
// Create database instance
CAgentStateDB* agent_db_new(const char* db_path);

// Free database instance
void agent_db_free(CAgentStateDB* db);

// Save Agent state
int agent_db_save_state(
    CAgentStateDB* db,
    uint64_t agent_id,
    uint64_t session_id,
    uint32_t state_type,
    const uint8_t* data,
    size_t data_len
);

// Load Agent state
int agent_db_load_state(
    CAgentStateDB* db,
    uint64_t agent_id,
    uint8_t** data,
    size_t* data_len
);

// Free data memory
void agent_db_free_data(uint8_t* data, size_t data_len);
```

#### Usage Example

```c
#include "agent_state_db.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main() {
    // Create database
    CAgentStateDB* db = agent_db_new("./test_db");
    if (!db) {
        printf("Failed to create database\n");
        return 1;
    }
    
    // Prepare data
    const char* data = "test agent state";
    size_t data_len = strlen(data);
    
    // Save state
    int result = agent_db_save_state(db, 12345, 67890, 0, 
                                    (const uint8_t*)data, data_len);
    if (result != 0) {
        printf("Failed to save state\n");
        agent_db_free(db);
        return 1;
    }
    
    // Load state
    uint8_t* loaded_data;
    size_t loaded_len;
    result = agent_db_load_state(db, 12345, &loaded_data, &loaded_len);
    if (result == 0) {
        printf("Loaded data: %.*s\n", (int)loaded_len, loaded_data);
        agent_db_free_data(loaded_data, loaded_len);
    }
    
    // Cleanup
    agent_db_free(db);
    return 0;
}
```

## 🐍 Python API (Planned)

### Basic Usage

```python
import AgentMem

# Create database
db = AgentMem.AgentDatabase("./agent_db")

# Save Agent state
state = AgentMem.AgentState(
    agent_id=12345,
    session_id=67890,
    state_type=AgentMem.StateType.WORKING_MEMORY,
    data=b"agent state data"
)
await db.save_agent_state(state)

# Load Agent state
loaded_state = await db.load_agent_state(12345)
if loaded_state:
    print(f"Found state: {loaded_state}")
```

## 📊 Error Handling

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum AgentMemError {
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}
```

### Error Handling Best Practices

```rust
match db.load_agent_state(agent_id).await {
    Ok(Some(state)) => {
        // Handle found state
        println!("State: {:?}", state);
    },
    Ok(None) => {
        // Handle not found case
        println!("Agent state not found");
    },
    Err(AgentMemError::Database(msg)) => {
        // Handle database error
        eprintln!("Database error: {}", msg);
    },
    Err(e) => {
        // Handle other errors
        eprintln!("Other error: {}", e);
    }
}
```

## 🔧 Configuration Options

### Database Configuration

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

**Document Version**: v1.0  
**Last Updated**: June 19, 2025  
**Maintainer**: AgentMem Development Team
