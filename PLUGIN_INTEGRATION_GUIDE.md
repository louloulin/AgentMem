# AgentMem 插件系统集成指南

**版本**: v2.1  
**日期**: 2025-11-04  
**状态**: ✅ 已集成

---

## 📋 集成概览

插件系统已成功集成到 AgentMem 主系统中，通过可选的 `plugins` feature 提供。

### 集成方式

1. **依赖配置** (`Cargo.toml`):
   ```toml
   [dependencies]
   agent-mem-plugins = { path = "../agent-mem-plugins", optional = true }
   
   [features]
   plugins = ["agent-mem-plugins"]
   ```

2. **模块导出** (`lib.rs`):
   ```rust
   #[cfg(feature = "plugins")]
   pub use agent_mem_plugins as plugins;
   ```

---

## 🚀 快速开始

### 1. 启用插件功能

在项目的 `Cargo.toml` 中启用 `plugins` feature：

```toml
[dependencies]
agent-mem = { path = "path/to/agent-mem", features = ["plugins"] }
```

### 2. 使用插件系统

```rust
use agent_mem::plugins::{
    PluginManager, PluginRegistry, PluginMetadata,
    PluginType, Capability,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建插件注册表
    let mut registry = PluginRegistry::new();
    
    // 注册插件
    let metadata = PluginMetadata {
        name: "my-plugin".to_string(),
        version: "0.1.0".to_string(),
        description: "My custom plugin".to_string(),
        author: "Me".to_string(),
        plugin_type: PluginType::MemoryProcessor,
        required_capabilities: vec![Capability::MemoryAccess],
        config_schema: None,
    };
    
    // 创建插件管理器
    let manager = PluginManager::new(10); // LRU cache size
    
    Ok(())
}
```

---

## 📦 可用功能

### 核心组件

1. **插件管理**:
   - `PluginRegistry` - 插件注册表
   - `PluginLoader` - 插件加载器
   - `PluginManager` - 插件管理器（带 LRU 缓存）

2. **宿主能力**:
   - `MemoryCapability` - 内存访问
   - `StorageCapability` - 键值存储
   - `SearchCapability` - 搜索功能
   - `LlmCapability` - LLM 调用
   - `NetworkCapability` - HTTP 请求
   - `LoggingCapability` - 日志记录

3. **安全机制**:
   - `SandboxConfig` - 沙盒配置
   - `PermissionChecker` - 权限检查
   - `ResourceLimits` - 资源限制
   - `ResourceMonitor` - 资源监控

4. **监控系统**:
   - `PluginMonitor` - 执行监控
   - `ExecutionMetrics` - 性能指标
   - `ExecutionTracker` - 执行追踪

---

## 🔧 高级用法

### 资源限制

```rust
use agent_mem::plugins::{
    ResourceLimits, ResourceMonitor, MemoryLimits, CpuLimits, IoLimits,
};

// 自定义资源限制
let limits = ResourceLimits {
    memory: MemoryLimits {
        max_heap_bytes: 50 * 1024 * 1024, // 50 MB
        max_stack_bytes: 1 * 1024 * 1024,  // 1 MB
        max_total_allocations: 5000,
    },
    cpu: CpuLimits {
        max_execution_time_ms: 3000,  // 3 seconds
        max_instructions: 500_000_000,
        max_cpu_time_ms: 2500,
    },
    io: IoLimits {
        max_network_requests: 50,
        max_file_operations: 500,
        max_read_bytes: 5 * 1024 * 1024,  // 5 MB
        max_write_bytes: 5 * 1024 * 1024, // 5 MB
        max_concurrent_connections: 5,
    },
};

let monitor = ResourceMonitor::new(limits);
let usage = monitor.usage();

// 追踪资源使用
usage.start_timing();
usage.record_allocation(1024);
usage.record_network_request();

// 检查限制
if let Err(e) = monitor.check_limits() {
    eprintln!("Resource limit exceeded: {}", e);
}
```

### 插件监控

```rust
use agent_mem::plugins::PluginMonitor;

let monitor = PluginMonitor::new();

// 开始追踪
let tracker = monitor.start_execution("my-plugin");

// ... 执行插件操作 ...

// 完成（成功）
tracker.complete();

// 或者失败
// tracker.fail("Error message".to_string());

// 获取指标
if let Some(metrics) = monitor.get_metrics("my-plugin") {
    println!("Total calls: {}", metrics.total_calls);
    println!("Success rate: {:.1}%", metrics.success_rate() * 100.0);
    println!("Avg time: {:?}", metrics.avg_execution_time);
}
```

### 网络能力

```rust
use agent_mem::plugins::capabilities::{
    NetworkCapability, HttpRequest, HttpMethod,
};
use std::collections::HashMap;

let network = NetworkCapability::new(false, 100); // 非 mock 模式

let request = HttpRequest {
    url: "https://api.example.com/data".to_string(),
    method: HttpMethod::POST,
    headers: {
        let mut h = HashMap::new();
        h.insert("Content-Type".to_string(), "application/json".to_string());
        h
    },
    body: Some(r#"{"query": "test"}"#.to_string()),
    timeout_ms: Some(5000),
};

match network.http_request(request) {
    Ok(response) => {
        println!("Status: {}", response.status);
        println!("Body: {}", response.body);
    }
    Err(e) => {
        eprintln!("Request failed: {}", e);
    }
}
```

---

## 📚 示例插件

### 1. Memory Processor Plugin

处理和增强记忆内容：

```rust
use extism_pdk::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Memory {
    id: String,
    content: String,
    // ... 其他字段
}

#[plugin_fn]
pub fn process_memory(input: String) -> FnResult<String> {
    let memory: Memory = serde_json::from_str(&input)?;
    
    // 处理记忆内容
    let processed = clean_and_enhance(&memory.content);
    
    // 返回结果
    Ok(serde_json::to_string(&processed)?)
}
```

### 2. Search Algorithm Plugin

自定义搜索算法：

```rust
#[plugin_fn]
pub fn search(input: String) -> FnResult<String> {
    let request: SearchRequest = serde_json::from_str(&input)?;
    
    // 执行搜索
    let results = perform_search(&request.query, &request.memories);
    
    Ok(serde_json::to_string(&results)?)
}
```

### 3. Data Source Plugin

集成外部数据源：

```rust
#[plugin_fn]
pub fn fetch_data(input: String) -> FnResult<String> {
    let config: DataSourceConfig = serde_json::from_str(&input)?;
    
    // 从外部数据源获取数据
    let data = fetch_from_source(&config)?;
    
    Ok(serde_json::to_string(&data)?)
}
```

---

## 🛠️ 开发插件

### 1. 创建插件项目

```bash
cargo new --lib my-plugin
cd my-plugin
```

### 2. 配置 Cargo.toml

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
extism-pdk = "1.2"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### 3. 编写插件代码

```rust
use extism_pdk::*;

#[plugin_fn]
pub fn hello(input: String) -> FnResult<String> {
    Ok(format!("Hello, {}!", input))
}

#[plugin_fn]
pub fn metadata() -> FnResult<String> {
    let metadata = serde_json::json!({
        "name": "my-plugin",
        "version": "0.1.0",
        "description": "My custom plugin",
        "author": "Me",
        "plugin_type": "Custom"
    });
    Ok(metadata.to_string())
}
```

### 4. 编译为 WASM

```bash
cargo build --target wasm32-wasi --release
```

输出文件: `target/wasm32-wasi/release/my_plugin.wasm`

---

## 🧪 测试

### 运行插件测试

```bash
# 所有测试
cargo test -p agent-mem-plugins

# 特定模块测试
cargo test -p agent-mem-plugins monitor
cargo test -p agent-mem-plugins network
cargo test -p agent-mem-plugins security::limits
```

### 运行集成示例

```bash
cargo run --example plugin_integration --features plugins
```

---

## 📊 测试统计

| 测试类型 | 数量 | 状态 |
|---------|------|------|
| 单元测试 | 52 | ✅ 100% |
| 集成测试 | 4 | ✅ 100% |
| 网络测试 | 7 | ✅ 100% |
| 搜索测试 | 8 | ✅ 100% |
| 资源限制测试 | 15 | ✅ 100% |
| 监控测试 | 1 | ✅ 100% |
| **总计** | **88** | **✅ 100%** |

---

## ⚠️ 注意事项

### 1. Feature Flag

插件功能是可选的，必须通过 feature flag 启用：

```toml
agent-mem = { version = "*", features = ["plugins"] }
```

### 2. WASM 编译

插件必须编译为 `wasm32-wasi` 目标：

```bash
rustup target add wasm32-wasi
cargo build --target wasm32-wasi --release
```

### 3. 安全性

- 插件运行在 WASM 沙盒中
- 必须显式授予能力权限
- 资源使用受限制监控
- 建议启用所有安全机制

### 4. 性能

- 使用 LRU 缓存提高性能
- 监控插件执行时间
- 设置合理的资源限制
- 定期检查资源使用情况

---

## 📖 相关文档

- [plugin.md](plugin.md) - 插件系统完整设计文档
- [PLUGIN_IMPLEMENTATION_REPORT_V2.md](PLUGIN_IMPLEMENTATION_REPORT_V2.md) - 实现报告
- [PLUGIN_VERIFICATION_REPORT.md](PLUGIN_VERIFICATION_REPORT.md) - 验证报告

---

## 🤝 贡献

欢迎贡献新的插件示例和功能改进！

1. Fork 项目
2. 创建 feature 分支
3. 提交更改
4. 创建 Pull Request

---

## 📝 更新日志

### v2.1 (2025-11-04)
- ✅ 集成到 AgentMem 主系统
- ✅ 添加 `plugins` feature flag
- ✅ 创建集成示例
- ✅ 完善文档

### v2.0 (2025-11-04)
- ✅ 实现资源限制系统
- ✅ 添加监控功能
- ✅ 网络能力支持
- ✅ 搜索算法插件

### v1.0 (2025-11-04)
- ✅ 基础插件系统
- ✅ 示例插件
- ✅ 安全机制

---

**文档版本**: v2.1  
**最后更新**: 2025-11-04  
**维护者**: AgentMem Team

