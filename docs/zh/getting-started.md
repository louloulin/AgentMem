# AgentMem 快速开始指南

## 🚀 欢迎使用 AgentMem

AgentMem 是一个高性能的 AI Agent 状态数据库，基于 Rust+Zig+LanceDB 混合架构构建。本指南将帮助您快速上手 AgentMem。

## 📋 系统要求

### 最低要求
- **操作系统**: Windows 10+, Linux (Ubuntu 18.04+), macOS 10.15+
- **内存**: 4GB RAM
- **存储**: 1GB 可用空间
- **网络**: 可选，用于分布式功能

### 推荐配置
- **操作系统**: Windows 11, Linux (Ubuntu 22.04+), macOS 12+
- **内存**: 8GB+ RAM
- **存储**: 10GB+ SSD
- **CPU**: 4核心以上

## 🛠️ 安装指南

### 方式一：从源码构建

#### 1. 安装依赖

**Rust (必需)**
```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 验证安装
rustc --version
cargo --version
```

**Zig (必需)**
```bash
# 下载 Zig 0.14.0
# Windows: 下载 zig-windows-x86_64-0.14.0.zip
# Linux: 下载 zig-linux-x86_64-0.14.0.tar.xz
# macOS: 下载 zig-macos-x86_64-0.14.0.tar.xz

# 解压并添加到 PATH
export PATH=$PATH:/path/to/zig

# 验证安装
zig version
```

#### 2. 克隆仓库

```bash
git clone https://github.com/louloulin/AgentMem.git
cd AgentMem
```

#### 3. 构建项目

```bash
# 构建 Rust 库
cargo build --release

# 生成 C 头文件
cargo run --bin generate_bindings

# 构建 Zig 组件
zig build

# 运行测试
cargo test --lib
zig build test
```

### 方式二：使用预编译包 (计划中)

```bash
# 使用包管理器安装 (未来版本)
# Rust
cargo install agent-db

# Python
pip install agent-db

# Node.js
npm install agent-db
```

## 🎯 第一个程序

### Rust 示例

创建 `examples/hello_AgentMem.rs`:

```rust
use agent_db::{AgentDatabase, DatabaseConfig, AgentState, StateType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 欢迎使用 AgentMem!");
    
    // 1. 创建数据库配置
    let config = DatabaseConfig {
        db_path: "./hello_AgentMem".to_string(),
        ..Default::default()
    };
    
    // 2. 创建数据库实例
    let db = AgentDatabase::new(config).await?;
    println!("✅ 数据库创建成功");
    
    // 3. 创建 Agent 状态
    let agent_id = 12345;
    let session_id = 67890;
    let state_data = b"Hello, AgentMem! 这是我的第一个 Agent 状态。".to_vec();
    
    let state = AgentState::new(
        agent_id,
        session_id,
        StateType::WorkingMemory,
        state_data
    );
    
    // 4. 保存状态
    db.save_agent_state(&state).await?;
    println!("✅ Agent 状态保存成功");
    
    // 5. 加载状态
    if let Some(loaded_state) = db.load_agent_state(agent_id).await? {
        let data_str = String::from_utf8_lossy(&loaded_state.data);
        println!("✅ 加载的状态数据: {}", data_str);
        println!("📊 状态信息:");
        println!("   Agent ID: {}", loaded_state.agent_id);
        println!("   Session ID: {}", loaded_state.session_id);
        println!("   状态类型: {:?}", loaded_state.state_type);
        println!("   创建时间: {}", loaded_state.created_at);
    } else {
        println!("❌ 未找到 Agent 状态");
    }
    
    println!("🎉 AgentMem 示例运行完成!");
    Ok(())
}
```

运行示例:
```bash
cargo run --example hello_AgentMem
```

### Zig 示例

创建 `examples/hello_AgentMem.zig`:

```zig
const std = @import("std");
const AgentState = @import("../src/agent_state.zig").AgentState;
const StateType = @import("../src/agent_state.zig").StateType;

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    
    std.debug.print("🚀 欢迎使用 AgentMem (Zig API)!\n", .{});
    
    // 1. 创建 Agent 状态
    var state = try AgentState.init(
        allocator,
        12345,                    // agent_id
        67890,                    // session_id
        .working_memory,          // state_type
        "Hello from Zig! 这是 Zig API 示例。" // data
    );
    defer state.deinit(allocator);
    
    std.debug.print("✅ Agent 状态创建成功\n", .{});
    
    // 2. 显示状态信息
    std.debug.print("📊 状态信息:\n", .{});
    state.display();
    
    // 3. 更新状态数据
    try state.updateData(allocator, "更新后的状态数据");
    std.debug.print("✅ 状态数据更新成功\n", .{});
    
    // 4. 设置元数据
    try state.setMetadata(allocator, "priority", "high");
    try state.setMetadata(allocator, "category", "demo");
    std.debug.print("✅ 元数据设置成功\n", .{});
    
    // 5. 创建状态快照
    var snapshot = try state.createSnapshot(allocator, "demo_snapshot");
    defer snapshot.deinit(allocator);
    std.debug.print("✅ 状态快照创建成功\n", .{});
    
    std.debug.print("🎉 Zig API 示例运行完成!\n", .{});
}
```

运行示例:
```bash
zig run examples/hello_AgentMem.zig
```

### C 示例

创建 `examples/hello_AgentMem.c`:

```c
#include "../include/agent_state_db.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main() {
    printf("🚀 欢迎使用 AgentMem (C API)!\n");
    
    // 1. 创建数据库实例
    CAgentStateDB* db = agent_db_new("./hello_AgentMem_c");
    if (!db) {
        printf("❌ 数据库创建失败\n");
        return 1;
    }
    printf("✅ 数据库创建成功\n");
    
    // 2. 准备数据
    const char* data = "Hello from C! 这是 C API 示例。";
    size_t data_len = strlen(data);
    uint64_t agent_id = 12345;
    uint64_t session_id = 67890;
    
    // 3. 保存 Agent 状态
    int result = agent_db_save_state(db, agent_id, session_id, 0, 
                                    (const uint8_t*)data, data_len);
    if (result != 0) {
        printf("❌ 保存状态失败\n");
        agent_db_free(db);
        return 1;
    }
    printf("✅ Agent 状态保存成功\n");
    
    // 4. 加载 Agent 状态
    uint8_t* loaded_data;
    size_t loaded_len;
    result = agent_db_load_state(db, agent_id, &loaded_data, &loaded_len);
    if (result == 0) {
        printf("✅ 状态加载成功\n");
        printf("📊 状态信息:\n");
        printf("   Agent ID: %llu\n", agent_id);
        printf("   Session ID: %llu\n", session_id);
        printf("   数据长度: %zu 字节\n", loaded_len);
        printf("   数据内容: %.*s\n", (int)loaded_len, loaded_data);
        
        // 释放数据内存
        agent_db_free_data(loaded_data, loaded_len);
    } else {
        printf("❌ 状态加载失败\n");
    }
    
    // 5. 清理资源
    agent_db_free(db);
    printf("🎉 C API 示例运行完成!\n");
    
    return 0;
}
```

编译和运行:
```bash
# 编译
gcc -o hello_AgentMem examples/hello_AgentMem.c -L./target/release -lagent_db_rust

# 运行 (Windows)
set PATH=%PATH%;./target/release
hello_AgentMem.exe

# 运行 (Linux/macOS)
export LD_LIBRARY_PATH=./target/release:$LD_LIBRARY_PATH
./hello_AgentMem
```

## 🧪 运行测试

### 基础功能测试

```bash
# Rust 测试
cargo test --lib

# Zig 测试
zig build test

# 性能基准测试
cargo test benchmark --lib

# 压力测试
cargo test stress_test --lib
```

### 分布式功能测试

```bash
# 分布式网络测试
zig test verify_distributed.zig

# 实时流处理测试
zig build test-realtime
```

## 📊 性能验证

运行性能基准测试来验证系统性能:

```bash
# 运行所有基准测试
cargo test benchmark --lib -- --nocapture

# 查看详细性能报告
cat PERFORMANCE_REPORT.md
```

预期性能指标:
- **向量搜索**: < 25ms
- **文档搜索**: < 30ms  
- **语义搜索**: < 20ms
- **记忆检索**: < 200ms
- **集成工作流**: < 300ms

## 🔧 配置选项

### 基础配置

创建 `config.toml`:

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

### 环境变量

```bash
# 设置数据库路径
export AgentMem_PATH="./my_AgentMem"

# 设置日志级别
export RUST_LOG="info"

# 设置性能模式
export AgentMem_PERFORMANCE_MODE="high"
```

## 🚨 常见问题

### Q: 编译时出现 LanceDB 相关错误？
**A**: 确保网络连接正常，LanceDB 依赖需要从网络下载。可以尝试:
```bash
cargo clean
cargo build --release
```

### Q: Zig 测试失败？
**A**: 确保 Rust 库已经构建完成:
```bash
cargo build --release
cargo run --bin generate_bindings
zig build test
```

### Q: C FFI 链接错误？
**A**: 确保库文件路径正确:
```bash
# Windows
set PATH=%PATH%;./target/release

# Linux/macOS  
export LD_LIBRARY_PATH=./target/release:$LD_LIBRARY_PATH
```

### Q: 性能不如预期？
**A**: 检查配置和系统资源:
- 确保使用 `--release` 模式构建
- 增加缓存大小配置
- 检查磁盘 I/O 性能
- 调整工作线程数量

## 📚 下一步

恭喜！您已经成功运行了第一个 AgentMem 程序。接下来可以：

1. **深入学习**: 阅读 [API 参考文档](api.md)
2. **架构理解**: 查看 [架构设计文档](architecture.md)  
3. **高级功能**: 探索分布式和 RAG 功能
4. **性能优化**: 学习性能调优技巧
5. **社区参与**: 加入开发者社区

## 🤝 获取帮助

- **文档**: [完整文档](../README.md)
- **示例**: [examples/](../../examples/) 目录
- **问题反馈**: [GitHub Issues](https://github.com/louloulin/AgentMem/issues)
- **社区讨论**: [GitHub Discussions](https://github.com/louloulin/AgentMem/discussions)

---

**文档版本**: v1.0  
**最后更新**: 2025年6月19日  
**维护者**: AgentMem开发团队
