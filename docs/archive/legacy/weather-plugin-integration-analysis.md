# Weather Plugin 与 AgentMem 集成分析

**分析日期**: 2025-11-05  
**插件路径**: `crates/agent-mem-plugin-sdk/examples/weather_plugin/`  
**目标**: 详细分析 Weather Plugin 如何与 AgentMem 核心系统集成

---

## 📋 目录

1. [架构概览](#架构概览)
2. [技术栈](#技术栈)
3. [集成流程](#集成流程)
4. [关键组件](#关键组件)
5. [数据流](#数据流)
6. [代码分析](#代码分析)
7. [实际应用](#实际应用)

---

## 🏗️ 架构概览

```
┌─────────────────────────────────────────────────────────────┐
│                     AgentMem 核心系统                        │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌────────────────┐        ┌──────────────────┐            │
│  │  HTTP Server   │◄──────►│  Plugin Manager  │            │
│  │  (Axum)        │        │  (插件管理器)     │            │
│  └────────────────┘        └────────┬─────────┘            │
│                                     │                       │
│                            ┌────────▼─────────┐             │
│                            │  Plugin Loader   │             │
│                            │  (Extism)        │             │
│                            └────────┬─────────┘             │
│                                     │                       │
│                            ┌────────▼─────────┐             │
│                            │   WASM Runtime   │             │
│                            │   (沙盒隔离)      │             │
│                            └────────┬─────────┘             │
└─────────────────────────────────────┼─────────────────────┘
                                      │
                    ┌─────────────────▼──────────────────┐
                    │     Weather Plugin (WASM)          │
                    ├────────────────────────────────────┤
                    │  • get_weather()                   │
                    │  • get_batch_weather()             │
                    │  • metadata()                      │
                    │                                    │
                    │  依赖宿主能力:                      │
                    │  ✓ NetworkAccess (网络请求)        │
                    │  ✓ LoggingAccess (日志记录)        │
                    └────────────────────────────────────┘
```

---

## 🔧 技术栈

### Weather Plugin 端 (WASM)

```toml
# Cargo.toml
[package]
name = "weather_plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]  # ← 编译为动态链接库 (WASM)

[dependencies]
extism-pdk = "1.2"       # ← Extism Plugin Development Kit
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

**关键技术**:
- **Extism PDK**: WASM 插件开发框架
- **cdylib**: 编译目标为 C 动态库（WASM 格式）
- **Serde**: JSON 序列化/反序列化

### AgentMem 端 (Rust)

```rust
// 核心组件
- PluginManager: 插件生命周期管理
- PluginLoader: 基于 Extism 的 WASM 加载器
- PluginRegistry: 插件注册表
- Capabilities: 宿主能力系统
```

---

## 🔄 集成流程

### 1. 编译阶段

```bash
# 编译为 WASM
cd crates/agent-mem-plugin-sdk/examples/weather_plugin
cargo build --target wasm32-wasip1 --release

# 输出文件
target/wasm32-wasip1/release/weather_plugin.wasm
```

**编译关键点**:
- `wasm32-wasip1`: WASI (WebAssembly System Interface) 目标
- `cdylib`: 生成 WASM 二进制文件
- 大小优化: Release 模式

### 2. 注册阶段

```bash
# 通过 API 注册插件
curl -X POST http://localhost:8080/api/v1/plugins \
  -H "Content-Type: application/json" \
  -H "X-User-ID: default" \
  -d '{
    "id": "weather-plugin",
    "metadata": {
      "name": "Weather Plugin",
      "version": "0.1.0",
      "description": "Fetches weather data from external APIs",
      "author": "AgentMem Team",
      "plugin_type": "data_source",
      "required_capabilities": ["network_access", "logging_access"]
    },
    "path": "target/wasm32-wasip1/release/weather_plugin.wasm",
    "config": {}
  }'
```

**注册流程**:
```rust
// 在 PluginManager 中
pub async fn register_plugin(&mut self, request: RegisterPluginRequest) -> Result<RegisteredPlugin> {
    // 1. 验证插件元数据
    self.validate_metadata(&request.metadata)?;
    
    // 2. 检查权限
    self.permission_checker.check_capabilities(&request.metadata.required_capabilities)?;
    
    // 3. 保存到注册表
    let plugin = RegisteredPlugin {
        id: request.id,
        metadata: request.metadata,
        path: request.path,
        status: PluginStatus::Registered,
        config: request.config,
        registered_at: Utc::now(),
    };
    
    self.registry.register(plugin.clone())?;
    Ok(plugin)
}
```

### 3. 加载阶段

```rust
// PluginLoader::load_plugin()
pub fn load_plugin(&self, plugin_info: &RegisteredPlugin) -> Result<LoadedPlugin> {
    // 1. 读取 WASM 文件
    let wasm_bytes = fs::read(&plugin_info.path)?;
    
    // 2. 创建 Extism 插件实例
    let manifest = Manifest::new([wasm_bytes]);
    let plugin = Plugin::new(&manifest, [], true)?;
    
    // 3. 返回加载的插件
    Ok(LoadedPlugin {
        id: plugin_info.id.clone(),
        metadata: plugin_info.metadata.clone(),
        plugin,  // ← Extism Plugin 实例
    })
}
```

**关键点**:
- **LRU 缓存**: 已加载的插件会被缓存，避免重复加载
- **沙盒隔离**: Extism 提供 WASM 沙盒环境
- **按需加载**: 插件在首次调用时才加载

### 4. 执行阶段

```rust
// 调用插件函数
pub fn call_plugin(plugin: &mut Plugin, function_name: &str, input: &str) -> Result<String> {
    plugin.call(function_name, input)
        .map_err(|e| anyhow!("Failed to call plugin function {}: {}", function_name, e))
}
```

**执行流程**:
```
1. API 请求 → HTTP Handler
2. HTTP Handler → PluginManager
3. PluginManager → 检查缓存
4. 如果未缓存 → PluginLoader.load_plugin()
5. 调用 plugin.call("get_weather", input_json)
6. WASM 执行 → 返回结果
7. 结果返回给客户端
```

---

## 🧩 关键组件

### Weather Plugin 内部结构

```rust
// lib.rs - 插件实现

// 1️⃣ 数据结构
#[derive(Deserialize)]
struct WeatherRequest {
    city: String,
    country: Option<String>,
}

#[derive(Serialize)]
struct WeatherData {
    city: String,
    temperature: f32,
    description: String,
    humidity: u32,
}

// 2️⃣ 主要函数
#[plugin_fn]  // ← Extism 宏，导出为 WASM 函数
pub fn get_weather(input: String) -> FnResult<String> {
    // a. 解析输入
    let request: WeatherRequest = serde_json::from_str(&input)?;
    
    // b. 调用宿主日志功能
    log(LogLevel::Info, &format!("Fetching weather for: {}", request.city))?;
    
    // c. 模拟天气数据获取 (实际应调用外部 API)
    let weather = simulate_weather_fetch(&request.city);
    
    // d. 返回 JSON 响应
    Ok(serde_json::to_string(&WeatherResponse {
        success: true,
        data: Some(weather),
        error: None,
    })?)
}

// 3️⃣ 批量查询
#[plugin_fn]
pub fn get_batch_weather(input: String) -> FnResult<String> {
    // 支持批量查询多个城市
    // ...
}

// 4️⃣ 元数据
#[plugin_fn]
pub fn metadata() -> FnResult<String> {
    // 返回插件元数据
    Ok(serde_json::json!({
        "name": "weather-plugin",
        "version": "0.1.0",
        "required_capabilities": ["NetworkAccess", "LoggingAccess"]
    }).to_string())
}
```

### AgentMem 宿主能力系统

```rust
// crates/agent-mem-plugins/src/capabilities/

// 1️⃣ 能力定义
pub enum Capability {
    MemoryAccess,      // 访问记忆
    StorageAccess,     // 键值存储
    SearchAccess,      // 搜索功能
    LlmAccess,         // LLM 调用
    NetworkAccess,     // ← Weather Plugin 需要
    FileSystemAccess,  // 文件系统
    LoggingAccess,     // ← Weather Plugin 需要
    ConfigAccess,      // 配置访问
}

// 2️⃣ 网络能力实现
pub struct NetworkCapability {
    client: reqwest::Client,
    rate_limiter: RateLimiter,
}

impl NetworkCapability {
    pub async fn http_request(&self, request: HttpRequest) -> Result<HttpResponse> {
        // 检查速率限制
        self.rate_limiter.check()?;
        
        // 发起 HTTP 请求
        let response = match request.method {
            HttpMethod::GET => self.client.get(&request.url),
            HttpMethod::POST => self.client.post(&request.url).body(request.body?),
            // ...
        }
        .send()
        .await?;
        
        Ok(HttpResponse {
            status: response.status().as_u16(),
            headers: /* ... */,
            body: response.text().await?,
        })
    }
}

// 3️⃣ 日志能力实现
pub struct LoggingCapability;

impl LoggingCapability {
    pub fn log(&self, level: LogLevel, message: &str) {
        match level {
            LogLevel::Error => tracing::error!("[Plugin] {}", message),
            LogLevel::Warn => tracing::warn!("[Plugin] {}", message),
            LogLevel::Info => tracing::info!("[Plugin] {}", message),
            LogLevel::Debug => tracing::debug!("[Plugin] {}", message),
        }
    }
}
```

---

## 📊 数据流

### 完整调用链路

```
┌──────────────┐
│ 客户端请求    │
│ POST /api/   │
│ weather?     │
│ city=London  │
└──────┬───────┘
       │
       ▼
┌──────────────────────────────────────┐
│ AgentMem HTTP Server (Axum)          │
│ ┌────────────────────────────────┐   │
│ │ Plugin API Handler             │   │
│ │ - 解析请求                      │   │
│ │ - 提取参数: city=London        │   │
│ └─────────────┬──────────────────┘   │
└───────────────┼──────────────────────┘
                │
                ▼
┌──────────────────────────────────────┐
│ Plugin Manager                       │
│ - get_plugin("weather-plugin")       │
│ - 检查 LRU 缓存                      │
│ - 如果未缓存，调用 PluginLoader      │
└─────────────┬────────────────────────┘
              │
              ▼
┌──────────────────────────────────────┐
│ Plugin Loader (Extism)               │
│ - load_plugin()                      │
│ - 读取 WASM 文件                     │
│ - 创建沙盒环境                       │
│ - 返回 LoadedPlugin                  │
└─────────────┬────────────────────────┘
              │
              ▼
┌──────────────────────────────────────┐
│ WASM Runtime (沙盒隔离)              │
│ ┌────────────────────────────────┐   │
│ │ Weather Plugin                 │   │
│ │                                │   │
│ │ 1. get_weather() 被调用        │   │
│ │ 2. 解析 JSON 输入              │   │
│ │ 3. 调用宿主函数:               │   │
│ │    log("Fetching...")          │───┼─► LoggingCapability
│ │ 4. 模拟获取天气数据            │   │
│ │ 5. 返回 JSON 结果              │   │
│ └────────────────────────────────┘   │
└─────────────┬────────────────────────┘
              │
              │ WeatherResponse JSON
              │ {
              │   "success": true,
              │   "data": {
              │     "city": "London",
              │     "temperature": 15.5,
              │     "description": "Cloudy",
              │     "humidity": 75
              │   }
              │ }
              ▼
┌──────────────────────────────────────┐
│ 返回给客户端                          │
└──────────────────────────────────────┘
```

---

## 💻 代码分析

### 1. `#[plugin_fn]` 宏的作用

```rust
// 源代码
#[plugin_fn]
pub fn get_weather(input: String) -> FnResult<String> {
    // ...
}

// 宏展开后 (简化版)
#[no_mangle]
pub extern "C" fn get_weather() {
    // 1. 从 WASM 内存读取输入
    let input = extism_pdk::input::get_input();
    
    // 2. 调用实际函数
    let result = actual_get_weather(input);
    
    // 3. 将结果写入 WASM 内存
    extism_pdk::output::set_output(result);
}
```

**关键点**:
- `#[no_mangle]`: 保持函数名不变，便于宿主调用
- `extern "C"`: 使用 C ABI，保证二进制兼容
- 内存管理: Extism PDK 处理 WASM 线性内存

### 2. 宿主函数调用机制

```rust
// 在插件中调用宿主函数

use extism_pdk::*;

// 定义宿主函数导入
extern "C" {
    fn host_http_request(ptr: u64) -> u64;
    fn host_log(level: u32, ptr: u64);
}

// 封装的便捷函数
pub fn log(level: LogLevel, message: &str) -> FnResult<()> {
    unsafe {
        let message_ptr = /* 分配内存并写入 message */;
        host_log(level as u32, message_ptr);
    }
    Ok(())
}
```

**实际实现** (在 AgentMem 端):

```rust
// crates/agent-mem-plugins/src/host_functions.rs

impl PluginManager {
    fn setup_host_functions(&mut self, plugin: &mut Plugin) {
        // 注册日志函数
        plugin.register_host_fn("host_log", |level: u32, message: &str| {
            let logging = LoggingCapability::new();
            logging.log(LogLevel::from(level), message);
        });
        
        // 注册 HTTP 函数
        plugin.register_host_fn("host_http_request", |request_json: &str| {
            let network = NetworkCapability::new();
            let request: HttpRequest = serde_json::from_str(request_json)?;
            let response = network.http_request(request).await?;
            serde_json::to_string(&response)
        });
    }
}
```

### 3. 权限检查

```rust
// 注册时检查
pub fn register_plugin(&mut self, request: RegisterPluginRequest) -> Result<()> {
    // 检查请求的能力是否被允许
    for capability in &request.metadata.required_capabilities {
        if !self.allowed_capabilities.contains(capability) {
            return Err(anyhow!("Capability {:?} not allowed", capability));
        }
    }
    
    // 运行时每次调用宿主函数时也会检查
    // ...
}
```

---

## 🎯 实际应用场景

### 场景 1: 集成到记忆系统

```rust
// 在 AgentMem 中使用 Weather Plugin

pub async fn enrich_memory_with_weather(
    memory: &Memory,
    plugin_manager: &PluginManager,
) -> Result<Memory> {
    // 从记忆内容中提取位置信息
    let location = extract_location(&memory.content)?;
    
    // 调用 Weather Plugin
    let weather_request = serde_json::json!({
        "city": location.city,
        "country": location.country
    });
    
    let weather_data = plugin_manager
        .call_plugin("weather-plugin", "get_weather", &weather_request.to_string())
        .await?;
    
    // 将天气信息添加到记忆元数据
    let mut enriched = memory.clone();
    enriched.metadata.insert(
        "weather".to_string(),
        weather_data.into()
    );
    
    Ok(enriched)
}
```

**使用示例**:
```bash
# 添加记忆时自动获取天气
POST /api/v1/memories
{
  "content": "今天在伦敦见了客户",
  "enrich_with_plugins": ["weather-plugin"]
}

# 响应 (自动添加天气数据)
{
  "id": "mem-123",
  "content": "今天在伦敦见了客户",
  "metadata": {
    "weather": {
      "city": "London",
      "temperature": 15.5,
      "description": "Cloudy"
    }
  }
}
```

### 场景 2: 批量天气查询

```rust
// 查询多个城市的天气
let cities = vec!["London", "Paris", "Tokyo"];
let batch_request = serde_json::json!({
    "cities": cities
});

let results = plugin_manager
    .call_plugin("weather-plugin", "get_batch_weather", &batch_request.to_string())
    .await?;

// 并行处理多个城市
```

### 场景 3: 定时更新

```rust
// 定时任务：更新天气缓存
async fn weather_cache_updater(plugin_manager: Arc<PluginManager>) {
    let mut interval = tokio::time::interval(Duration::from_secs(3600));
    
    loop {
        interval.tick().await;
        
        // 获取常用城市列表
        let cities = get_popular_cities();
        
        // 批量更新天气数据
        let batch_request = serde_json::json!({ "cities": cities });
        let _ = plugin_manager
            .call_plugin("weather-plugin", "get_batch_weather", &batch_request.to_string())
            .await;
    }
}
```

---

## 🔐 安全机制

### 1. WASM 沙盒隔离

```
┌────────────────────────────────────┐
│ WASM 沙盒                          │
│ ┌────────────────────────────────┐ │
│ │ Weather Plugin                 │ │
│ │ - 无法访问文件系统             │ │
│ │ - 无法创建网络连接             │ │
│ │ - 无法执行系统调用             │ │
│ │ - 只能通过宿主函数交互         │ │
│ └────────────────────────────────┘ │
└────────────────────────────────────┘
```

### 2. 权限系统

```rust
// 权限定义
required_capabilities: ["network_access", "logging_access"]

// 权限检查 (注册时)
if plugin.requires("network_access") && !is_allowed("network_access") {
    return Err("Network access not allowed");
}

// 权限检查 (运行时)
fn host_http_request(...) {
    if !has_capability(current_plugin, "network_access") {
        panic!("Permission denied: network_access");
    }
    // 执行请求
}
```

### 3. 资源限制

```rust
// 限制配置
ResourceLimits {
    max_memory: 50 * 1024 * 1024,  // 50 MB
    max_cpu_time: Duration::from_secs(30),
    max_concurrent_requests: 10,
    rate_limit: RateLimit {
        requests_per_minute: 60,
    }
}
```

---

## 📈 性能考虑

### 1. LRU 缓存

```rust
// PluginManager 内部使用 LRU 缓存
use lru::LruCache;

pub struct PluginManager {
    cache: LruCache<String, LoadedPlugin>,  // ← LRU 缓存
    // ...
}

// 性能对比
- 首次加载: ~31ms
- 缓存命中: ~333ns (快 93,000+ 倍!)
```

### 2. 并发控制

```rust
// 使用信号量限制并发
let semaphore = Arc::new(Semaphore::new(max_concurrent));

async fn call_plugin_with_limit(...) {
    let _permit = semaphore.acquire().await?;
    // 调用插件
}
```

### 3. 超时控制

```rust
use tokio::time::timeout;

let result = timeout(
    Duration::from_secs(30),
    plugin_manager.call_plugin(...)
).await??;
```

---

## 🚀 开发新插件

### 步骤 1: 创建项目

```bash
cd crates/agent-mem-plugin-sdk/examples
cargo new my_plugin --lib
cd my_plugin
```

### 步骤 2: 配置 Cargo.toml

```toml
[package]
name = "my_plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
extism-pdk = "1.2"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### 步骤 3: 实现插件

```rust
use extism_pdk::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct MyInput {
    data: String,
}

#[derive(Serialize)]
struct MyOutput {
    result: String,
}

#[plugin_fn]
pub fn process(input: String) -> FnResult<String> {
    // 1. 解析输入
    let input: MyInput = serde_json::from_str(&input)?;
    
    // 2. 处理逻辑
    let result = format!("Processed: {}", input.data);
    
    // 3. 返回结果
    let output = MyOutput { result };
    Ok(serde_json::to_string(&output)?)
}

#[plugin_fn]
pub fn metadata() -> FnResult<String> {
    Ok(serde_json::json!({
        "name": "my-plugin",
        "version": "0.1.0",
        "required_capabilities": []
    }).to_string())
}
```

### 步骤 4: 编译

```bash
cargo build --target wasm32-wasip1 --release
```

### 步骤 5: 注册到 AgentMem

```bash
curl -X POST http://localhost:8080/api/v1/plugins \
  -H "Content-Type: application/json" \
  -d '{
    "id": "my-plugin",
    "metadata": {
      "name": "My Plugin",
      "version": "0.1.0",
      "description": "My custom plugin",
      "author": "Me",
      "plugin_type": "custom",
      "required_capabilities": []
    },
    "path": "target/wasm32-wasip1/release/my_plugin.wasm",
    "config": {}
  }'
```

---

## 🧪 测试

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_weather_parsing() {
        let input = r#"{"city": "London"}"#;
        let result = get_weather(input.to_string()).unwrap();
        assert!(result.contains("London"));
    }
}
```

### 集成测试

```rust
// crates/agent-mem-plugins/tests/weather_plugin_test.rs

#[tokio::test]
async fn test_load_weather_plugin() {
    let manager = PluginManager::new();
    let plugin = manager.load_plugin("weather-plugin").await.unwrap();
    
    let result = manager.call_plugin(
        &plugin,
        "get_weather",
        r#"{"city": "London"}"#
    ).await.unwrap();
    
    assert!(result.contains("temperature"));
}
```

---

## 📊 总结

### 集成优势

| 特性 | 说明 |
|------|------|
| **安全隔离** | WASM 沙盒完全隔离，无法直接访问系统资源 |
| **跨平台** | WASM 可在任何平台运行 |
| **动态加载** | 无需重启即可加载/卸载插件 |
| **权限控制** | 细粒度的能力权限系统 |
| **高性能** | LRU 缓存 + 并发控制 |
| **易开发** | Extism PDK 提供简单的 API |

### 关键要点

1. **编译为 WASM**: 使用 `wasm32-wasip1` 目标编译
2. **Extism 框架**: 宿主和插件都使用 Extism
3. **能力系统**: 插件声明所需能力，宿主检查并提供
4. **JSON 通信**: 插件和宿主之间通过 JSON 交换数据
5. **LRU 缓存**: 已加载的插件被缓存以提高性能

### 最佳实践

✅ **DO**:
- 明确声明所需的能力
- 使用结构化的输入/输出 (JSON)
- 实现 `metadata()` 函数
- 添加错误处理
- 编写单元测试

❌ **DON'T**:
- 请求不必要的权限
- 在插件中执行长时间运行的操作
- 假设宿主环境
- 忽略错误
- 硬编码配置

---

## 📚 参考资料

- [Extism 文档](https://extism.org/)
- [WASI 规范](https://wasi.dev/)
- [AgentMem 插件系统设计](../plugin.md)
- [Weather Plugin 源码](./src/lib.rs)

---

**🎉 现在你已经理解了 Weather Plugin 如何与 AgentMem 集成！**

