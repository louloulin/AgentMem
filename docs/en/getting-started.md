# AgentMem Getting Started Guide

## 🚀 Welcome to AgentMem

AgentMem is a high-performance AI Agent state database built on a hybrid Rust+Zig+LanceDB architecture. This guide will help you get started with AgentMem quickly.

## 📋 System Requirements

### Minimum Requirements
- **OS**: Windows 10+, Linux (Ubuntu 18.04+), macOS 10.15+
- **Memory**: 4GB RAM
- **Storage**: 1GB available space
- **Network**: Optional, for distributed features

### Recommended Configuration
- **OS**: Windows 11, Linux (Ubuntu 22.04+), macOS 12+
- **Memory**: 8GB+ RAM
- **Storage**: 10GB+ SSD
- **CPU**: 4+ cores

## 🛠️ Installation Guide

### Method 1: Build from Source

#### 1. Install Dependencies

**Rust (Required)**
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version
```

**Zig (Required)**
```bash
# Download Zig 0.14.0
# Windows: Download zig-windows-x86_64-0.14.0.zip
# Linux: Download zig-linux-x86_64-0.14.0.tar.xz
# macOS: Download zig-macos-x86_64-0.14.0.tar.xz

# Extract and add to PATH
export PATH=$PATH:/path/to/zig

# Verify installation
zig version
```

#### 2. Clone Repository

```bash
git clone https://github.com/louloulin/AgentMem.git
cd AgentMem
```

#### 3. Build Project

```bash
# Build Rust library
cargo build --release

# Generate C headers
cargo run --bin generate_bindings

# Build Zig components
zig build

# Run tests
cargo test --lib
zig build test
```

### Method 2: Pre-built Packages (Planned)

```bash
# Install using package managers (future versions)
# Rust
cargo install agent-db

# Python
pip install agent-db

# Node.js
npm install agent-db
```

## 🎯 Your First Program

### Rust Example

Create `examples/hello_AgentMem.rs`:

```rust
use agent_db::{AgentDatabase, DatabaseConfig, AgentState, StateType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Welcome to AgentMem!");
    
    // 1. Create database configuration
    let config = DatabaseConfig {
        db_path: "./hello_AgentMem".to_string(),
        ..Default::default()
    };
    
    // 2. Create database instance
    let db = AgentDatabase::new(config).await?;
    println!("✅ Database created successfully");
    
    // 3. Create Agent state
    let agent_id = 12345;
    let session_id = 67890;
    let state_data = b"Hello, AgentMem! This is my first Agent state.".to_vec();
    
    let state = AgentState::new(
        agent_id,
        session_id,
        StateType::WorkingMemory,
        state_data
    );
    
    // 4. Save state
    db.save_agent_state(&state).await?;
    println!("✅ Agent state saved successfully");
    
    // 5. Load state
    if let Some(loaded_state) = db.load_agent_state(agent_id).await? {
        let data_str = String::from_utf8_lossy(&loaded_state.data);
        println!("✅ Loaded state data: {}", data_str);
        println!("📊 State information:");
        println!("   Agent ID: {}", loaded_state.agent_id);
        println!("   Session ID: {}", loaded_state.session_id);
        println!("   State Type: {:?}", loaded_state.state_type);
        println!("   Created At: {}", loaded_state.created_at);
    } else {
        println!("❌ Agent state not found");
    }
    
    println!("🎉 AgentMem example completed!");
    Ok(())
}
```

Run the example:
```bash
cargo run --example hello_AgentMem
```

### Zig Example

Create `examples/hello_AgentMem.zig`:

```zig
const std = @import("std");
const AgentState = @import("../src/agent_state.zig").AgentState;
const StateType = @import("../src/agent_state.zig").StateType;

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    
    std.debug.print("🚀 Welcome to AgentMem (Zig API)!\n", .{});
    
    // 1. Create Agent state
    var state = try AgentState.init(
        allocator,
        12345,                    // agent_id
        67890,                    // session_id
        .working_memory,          // state_type
        "Hello from Zig! This is a Zig API example." // data
    );
    defer state.deinit(allocator);
    
    std.debug.print("✅ Agent state created successfully\n", .{});
    
    // 2. Display state information
    std.debug.print("📊 State information:\n", .{});
    state.display();
    
    // 3. Update state data
    try state.updateData(allocator, "Updated state data");
    std.debug.print("✅ State data updated successfully\n", .{});
    
    // 4. Set metadata
    try state.setMetadata(allocator, "priority", "high");
    try state.setMetadata(allocator, "category", "demo");
    std.debug.print("✅ Metadata set successfully\n", .{});
    
    // 5. Create state snapshot
    var snapshot = try state.createSnapshot(allocator, "demo_snapshot");
    defer snapshot.deinit(allocator);
    std.debug.print("✅ State snapshot created successfully\n", .{});
    
    std.debug.print("🎉 Zig API example completed!\n", .{});
}
```

Run the example:
```bash
zig run examples/hello_AgentMem.zig
```

### C Example

Create `examples/hello_AgentMem.c`:

```c
#include "../include/agent_state_db.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main() {
    printf("🚀 Welcome to AgentMem (C API)!\n");
    
    // 1. Create database instance
    CAgentStateDB* db = agent_db_new("./hello_AgentMem_c");
    if (!db) {
        printf("❌ Failed to create database\n");
        return 1;
    }
    printf("✅ Database created successfully\n");
    
    // 2. Prepare data
    const char* data = "Hello from C! This is a C API example.";
    size_t data_len = strlen(data);
    uint64_t agent_id = 12345;
    uint64_t session_id = 67890;
    
    // 3. Save Agent state
    int result = agent_db_save_state(db, agent_id, session_id, 0, 
                                    (const uint8_t*)data, data_len);
    if (result != 0) {
        printf("❌ Failed to save state\n");
        agent_db_free(db);
        return 1;
    }
    printf("✅ Agent state saved successfully\n");
    
    // 4. Load Agent state
    uint8_t* loaded_data;
    size_t loaded_len;
    result = agent_db_load_state(db, agent_id, &loaded_data, &loaded_len);
    if (result == 0) {
        printf("✅ State loaded successfully\n");
        printf("📊 State information:\n");
        printf("   Agent ID: %llu\n", agent_id);
        printf("   Session ID: %llu\n", session_id);
        printf("   Data Length: %zu bytes\n", loaded_len);
        printf("   Data Content: %.*s\n", (int)loaded_len, loaded_data);
        
        // Free data memory
        agent_db_free_data(loaded_data, loaded_len);
    } else {
        printf("❌ Failed to load state\n");
    }
    
    // 5. Cleanup resources
    agent_db_free(db);
    printf("🎉 C API example completed!\n");
    
    return 0;
}
```

Compile and run:
```bash
# Compile
gcc -o hello_AgentMem examples/hello_AgentMem.c -L./target/release -lagent_db_rust

# Run (Windows)
set PATH=%PATH%;./target/release
hello_AgentMem.exe

# Run (Linux/macOS)
export LD_LIBRARY_PATH=./target/release:$LD_LIBRARY_PATH
./hello_AgentMem
```

## 🧪 Running Tests

### Basic Functionality Tests

```bash
# Rust tests
cargo test --lib

# Zig tests
zig build test

# Performance benchmarks
cargo test benchmark --lib

# Stress tests
cargo test stress_test --lib
```

### Distributed Functionality Tests

```bash
# Distributed network tests
zig test verify_distributed.zig

# Real-time streaming tests
zig build test-realtime
```

## 📊 Performance Verification

Run performance benchmarks to verify system performance:

```bash
# Run all benchmark tests
cargo test benchmark --lib -- --nocapture

# View detailed performance report
cat PERFORMANCE_REPORT.md
```

Expected performance metrics:
- **Vector Search**: < 25ms
- **Document Search**: < 30ms  
- **Semantic Search**: < 20ms
- **Memory Retrieval**: < 200ms
- **Integrated Workflow**: < 300ms

## 🔧 Configuration Options

### Basic Configuration

Create `config.toml`:

```toml
[database]
path = "./AgentMem"
max_connections = 10
connection_timeout = 30
query_timeout = 60
enable_wal = true
cache_size = 104857600  # 100MB

[vector]
dimension = 384
similarity_algorithm = "cosine"
index_type = "hnsw"

[memory]
max_memories_per_agent = 10000
importance_threshold = 0.1
decay_factor = 0.01

[security]
enable_auth = false
enable_encryption = false
jwt_secret = "your-secret-key"

[performance]
enable_cache = true
batch_size = 1000
worker_threads = 4
```

### Environment Variables

```bash
# Set database path
export AgentMem_PATH="./my_AgentMem"

# Set log level
export RUST_LOG="info"

# Set performance mode
export AgentMem_PERFORMANCE_MODE="high"
```

## 🚨 Common Issues

### Q: LanceDB-related errors during compilation?
**A**: Ensure network connection is stable, LanceDB dependencies need to be downloaded from the network. Try:
```bash
cargo clean
cargo build --release
```

### Q: Zig tests failing?
**A**: Ensure Rust library is built first:
```bash
cargo build --release
cargo run --bin generate_bindings
zig build test
```

### Q: C FFI linking errors?
**A**: Ensure library path is correct:
```bash
# Windows
set PATH=%PATH%;./target/release

# Linux/macOS  
export LD_LIBRARY_PATH=./target/release:$LD_LIBRARY_PATH
```

### Q: Performance not as expected?
**A**: Check configuration and system resources:
- Ensure using `--release` mode for builds
- Increase cache size configuration
- Check disk I/O performance
- Adjust worker thread count

## 📚 Next Steps

Congratulations! You've successfully run your first AgentMem program. Next, you can:

1. **Deep Learning**: Read the [API Reference](api.md)
2. **Architecture Understanding**: Check the [Architecture Design](architecture.md)  
3. **Advanced Features**: Explore distributed and RAG functionality
4. **Performance Optimization**: Learn performance tuning techniques
5. **Community Participation**: Join the developer community

## 🤝 Getting Help

- **Documentation**: [Complete Documentation](../README.md)
- **Examples**: [examples/](../../examples/) directory
- **Issue Reports**: [GitHub Issues](https://github.com/louloulin/AgentMem/issues)
- **Community Discussion**: [GitHub Discussions](https://github.com/louloulin/AgentMem/discussions)

---

**Document Version**: v1.0  
**Last Updated**: June 19, 2025  
**Maintainer**: AgentMem Development Team
