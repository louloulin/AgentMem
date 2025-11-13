# Weather Plugin 调用流程分析

**文档日期**: 2025-11-05  
**说明**: Weather Plugin 在 AgentMem 系统中的完整调用路径和触发场景

---

## 🎯 简要回答

**Weather Plugin 目前主要在以下场景被调用：**

1. **测试环境** - 在单元测试和集成测试中被调用
2. **直接 API 调用** - 通过 HTTP API 直接调用插件函数
3. **插件钩子（潜在）** - 通过记忆操作的插件钩子自动触发（需要注册为特定类型）

**⚠️ 重要发现**: Weather Plugin 作为 **DataSource** 类型插件，当前**没有自动触发点**，需要**手动调用**或通过**自定义集成**来使用。

---

## 📊 完整调用路径图

```
┌────────────────────────────────────────────────────────────────┐
│                    调用方式                                     │
├────────────────────────────────────────────────────────────────┤
│                                                                 │
│  方式1: 直接 API 调用（手动触发）                              │
│  ┌──────────────────────────────────────┐                     │
│  │ HTTP POST /api/v1/plugins/call       │                     │
│  │ {                                    │                     │
│  │   "plugin_id": "weather-plugin",     │                     │
│  │   "function": "get_weather",         │                     │
│  │   "input": {"city": "London"}        │                     │
│  │ }                                    │                     │
│  └──────────┬───────────────────────────┘                     │
│             │                                                  │
│             ▼                                                  │
│  ┌──────────────────────────────────────┐                     │
│  │ PluginManager.call_plugin()          │                     │
│  │ - 检查缓存                            │                     │
│  │ - 加载插件（如需要）                  │                     │
│  │ - 调用 WASM 函数                      │                     │
│  └──────────┬───────────────────────────┘                     │
│             │                                                  │
│             ▼                                                  │
│  ┌──────────────────────────────────────┐                     │
│  │ Weather Plugin (WASM)                │                     │
│  │ get_weather(input)                   │                     │
│  │ ├─ 解析 JSON                         │                     │
│  │ ├─ 调用宿主日志                       │                     │
│  │ ├─ 模拟/获取天气数据                  │                     │
│  │ └─ 返回 JSON 结果                     │                     │
│  └──────────────────────────────────────┘                     │
│                                                                 │
│  方式2: 测试调用（开发/验证）                                   │
│  ┌──────────────────────────────────────┐                     │
│  │ tests/                                │                     │
│  │ ├─ end_to_end_test.rs                │                     │
│  │ ├─ wasm_loading_test.rs              │                     │
│  │ └─ benchmarks/                       │                     │
│  └──────────────────────────────────────┘                     │
│                                                                 │
│  方式3: 自定义集成（需要开发）                                  │
│  ┌──────────────────────────────────────┐                     │
│  │ 在应用代码中调用:                     │                     │
│  │                                       │                     │
│  │ let weather_data =                    │                     │
│  │   plugin_manager                      │                     │
│  │     .call_plugin(                     │                     │
│  │       "weather-plugin",               │                     │
│  │       "get_weather",                  │                     │
│  │       &input                          │                     │
│  │     ).await?;                         │                     │
│  └──────────────────────────────────────┘                     │
└────────────────────────────────────────────────────────────────┘
```

---

## 🔍 详细调用场景

### 场景 1: 测试环境调用

#### 位置: `crates/agent-mem-plugins/tests/`

**示例 1: 端到端测试**
```rust
// end_to_end_test.rs

#[tokio::test]
async fn test_weather_plugin() {
    let manager = PluginManager::new(100);
    
    // 注册插件
    manager.register(weather_plugin).await?;
    
    // 调用插件
    let result = manager.call_plugin(
        "weather-plugin",
        "get_weather",
        r#"{"city": "London"}"#
    ).await?;
    
    println!("Weather: {}", result);
}
```

**调用链路**:
```
test_weather_plugin()
  → PluginManager.call_plugin()
    → get_plugin() (检查缓存/加载)
      → PluginLoader.load_plugin() (如需加载)
    → PluginLoader.call_plugin()
      → Extism::Plugin.call()
        → WASM Runtime
          → Weather Plugin::get_weather()
```

---

### 场景 2: 直接 API 调用（当前未实现专用端点）

#### ⚠️ 当前状态: 需要实现

虽然插件系统已就绪，但**目前没有专用的 API 端点来调用插件函数**。

**需要添加的端点** (建议实现):

```rust
// crates/agent-mem-server/src/routes/plugins.rs

/// Call a plugin function
#[utoipa::path(
    post,
    path = "/api/v1/plugins/{plugin_id}/call",
    tag = "plugins",
)]
pub async fn call_plugin_function(
    State(memory_manager): State<Arc<MemoryManager>>,
    Path(plugin_id): Path<String>,
    Json(request): Json<CallPluginRequest>,
) -> ServerResult<Json<serde_json::Value>> {
    info!("Calling plugin {} function {}", plugin_id, request.function);
    
    let result = memory_manager.memory
        .plugin_manager()
        .call_plugin(&plugin_id, &request.function, &request.input)
        .await
        .map_err(|e| ServerError::internal(e.to_string()))?;
    
    let value: serde_json::Value = serde_json::from_str(&result)
        .map_err(|e| ServerError::internal(e.to_string()))?;
    
    Ok(Json(value))
}

#[derive(Deserialize)]
struct CallPluginRequest {
    function: String,
    input: String,
}
```

**使用方式**:
```bash
# 调用 Weather Plugin 获取天气
curl -X POST "http://localhost:8080/api/v1/plugins/weather-plugin/call" \
  -H "Content-Type: application/json" \
  -H "X-User-ID: default" \
  -d '{
    "function": "get_weather",
    "input": "{\"city\": \"London\"}"
  }'
```

---

### 场景 3: 通过插件钩子自动触发

#### 位置: `crates/agent-mem/src/plugin_integration.rs`

**当前支持的插件类型自动触发**:

1. **MemoryProcessor** - 在添加记忆时自动调用
2. **SearchAlgorithm** - 在搜索时自动调用

**Weather Plugin 的问题**: 

Weather Plugin 被定义为 **DataSource** 类型：
```rust
// weather_plugin/src/lib.rs
pub fn metadata() -> FnResult<String> {
    let metadata = serde_json::json!({
        "plugin_type": "DataSource",  // ← 不会自动触发
        // ...
    });
}
```

**DataSource 类型插件目前没有自动触发点！**

#### 如果要自动触发 Weather Plugin，需要：

**选项 A: 修改为 MemoryProcessor**

```rust
// 修改 weather_plugin 元数据
pub fn metadata() -> FnResult<String> {
    let metadata = serde_json::json!({
        "plugin_type": "MemoryProcessor",  // ← 改为 MemoryProcessor
        "required_capabilities": ["NetworkAccess", "LoggingAccess"]
    });
    Ok(metadata.to_string())
}

// 实现 process_memory 函数
#[plugin_fn]
pub fn process_memory(input: String) -> FnResult<String> {
    // 从记忆内容中提取位置信息
    let memory: serde_json::Value = serde_json::from_str(&input)?;
    
    // 检查是否包含位置关键词
    if let Some(content) = memory.get("content").and_then(|v| v.as_str()) {
        if content.contains("伦敦") || content.contains("London") {
            // 获取天气并添加到 metadata
            let weather = simulate_weather_fetch("London");
            // 修改 memory.metadata 添加天气信息
        }
    }
    
    Ok(serde_json::to_string(&memory)?)
}
```

**自动触发流程**:
```
用户添加记忆
  → Memory.add()
    → PluginHooks.before_add_memory() ◄─ 自动触发点
      → 遍历所有 MemoryProcessor 插件
        → Weather Plugin::process_memory()
          → 提取位置 → 获取天气 → 添加到元数据
```

**选项 B: 添加专门的 DataSource 钩子**

```rust
// 在 plugin_integration.rs 中添加新钩子

#[async_trait::async_trait]
pub trait PluginHooks {
    // ... 现有钩子 ...
    
    /// 调用数据源插件获取外部数据
    async fn fetch_external_data(
        &self, 
        data_type: &str,
        params: &serde_json::Value
    ) -> Result<serde_json::Value> {
        // 查找 DataSource 插件并调用
    }
}
```

**使用方式**:
```rust
// 在添加记忆时自动获取天气
let weather = plugin_hooks
    .fetch_external_data("weather", &json!({"city": "London"}))
    .await?;

memory.metadata.insert("weather".to_string(), weather);
```

---

### 场景 4: 自定义应用集成

#### 示例: 记忆增强服务

```rust
// 在自定义服务中使用 Weather Plugin

pub struct MemoryEnricher {
    plugin_manager: Arc<PluginManager>,
}

impl MemoryEnricher {
    /// 自动为包含位置的记忆添加天气信息
    pub async fn enrich_with_weather(
        &self,
        memory: &mut MemoryItem,
    ) -> Result<()> {
        // 1. 从记忆内容中提取位置
        let location = self.extract_location(&memory.content)?;
        
        // 2. 调用 Weather Plugin
        let weather_request = serde_json::json!({
            "city": location.city,
            "country": location.country
        });
        
        let weather_json = self.plugin_manager
            .call_plugin(
                "weather-plugin",
                "get_weather",
                &weather_request.to_string()
            )
            .await?;
        
        // 3. 解析并添加到元数据
        let weather: serde_json::Value = serde_json::from_str(&weather_json)?;
        memory.metadata.insert("weather".to_string(), weather);
        
        Ok(())
    }
    
    /// 从文本中提取位置信息
    fn extract_location(&self, content: &str) -> Result<Location> {
        // 使用 NLP 或正则表达式提取位置
        // 简化示例:
        if content.contains("伦敦") || content.contains("London") {
            Ok(Location {
                city: "London".to_string(),
                country: Some("UK".to_string()),
            })
        } else {
            Err(anyhow::anyhow!("No location found"))
        }
    }
}

struct Location {
    city: String,
    country: Option<String>,
}
```

**使用示例**:
```rust
// 在添加记忆的 API 处理中
pub async fn add_memory_handler(
    memory_input: MemoryInput,
) -> Result<MemoryItem> {
    let mut memory = create_memory_from_input(memory_input)?;
    
    // 自动增强：添加天气信息
    if let Some(enricher) = &self.memory_enricher {
        let _ = enricher.enrich_with_weather(&mut memory).await;
    }
    
    // 保存记忆
    self.memory.add(memory).await
}
```

---

## 🔧 实际使用示例

### 1. 通过代码直接调用

```rust
use agent_mem_plugins::PluginManager;

#[tokio::main]
async fn main() -> Result<()> {
    let manager = PluginManager::new(100);
    
    // 注册 Weather Plugin
    manager.register(weather_plugin_info).await?;
    
    // 查询单个城市
    let london_weather = manager
        .call_plugin(
            "weather-plugin",
            "get_weather",
            r#"{"city": "London"}"#
        )
        .await?;
    
    println!("London: {}", london_weather);
    
    // 批量查询
    let batch_weather = manager
        .call_plugin(
            "weather-plugin",
            "get_batch_weather",
            r#"{"cities": ["London", "Paris", "Tokyo"]}"#
        )
        .await?;
    
    println!("Batch: {}", batch_weather);
    
    Ok(())
}
```

### 2. 定时任务调用

```rust
use tokio::time::{interval, Duration};

async fn weather_cache_updater(manager: Arc<PluginManager>) {
    let mut ticker = interval(Duration::from_secs(3600)); // 每小时
    
    loop {
        ticker.tick().await;
        
        // 更新常用城市天气
        let cities = vec!["London", "Paris", "Tokyo", "New York"];
        let request = serde_json::json!({ "cities": cities });
        
        match manager
            .call_plugin(
                "weather-plugin",
                "get_batch_weather",
                &request.to_string()
            )
            .await
        {
            Ok(weather_data) => {
                // 更新缓存
                update_weather_cache(weather_data).await;
            }
            Err(e) => {
                eprintln!("Failed to update weather: {}", e);
            }
        }
    }
}
```

### 3. 事件驱动调用

```rust
use tokio::sync::mpsc;

struct WeatherEvent {
    city: String,
    callback: oneshot::Sender<String>,
}

async fn weather_service(
    manager: Arc<PluginManager>,
    mut rx: mpsc::Receiver<WeatherEvent>,
) {
    while let Some(event) = rx.recv().await {
        let request = serde_json::json!({ "city": event.city });
        
        let result = manager
            .call_plugin(
                "weather-plugin",
                "get_weather",
                &request.to_string()
            )
            .await
            .unwrap_or_else(|e| format!(r#"{{"error": "{}"}}"#, e));
        
        let _ = event.callback.send(result);
    }
}
```

---

## 📈 调用性能

### 缓存机制

```
首次调用:
  加载 WASM → ~31ms
  执行函数 → ~1ms
  总计 → ~32ms

后续调用（缓存命中）:
  检查缓存 → ~333ns
  执行函数 → ~1ms
  总计 → ~1ms

性能提升: 32,000x (缓存 vs 首次加载)
```

### 并发调用

```rust
// 并发调用多个插件函数
let mut handles = vec![];

for city in cities {
    let manager = manager.clone();
    let handle = tokio::spawn(async move {
        manager.call_plugin(
            "weather-plugin",
            "get_weather",
            &serde_json::json!({"city": city}).to_string()
        ).await
    });
    handles.push(handle);
}

// 等待所有结果
let results = futures::future::join_all(handles).await;
```

---

## 🚀 推荐的集成方式

### 方式 1: API 端点（推荐）

**优点**:
- ✅ 简单直接
- ✅ 易于测试
- ✅ 支持外部调用
- ✅ 可缓存结果

**实现**:
```rust
// 添加到 routes/plugins.rs

#[utoipa::path(post, path = "/api/v1/weather")]
pub async fn get_weather(
    State(plugin_manager): State<Arc<PluginManager>>,
    Query(city): Query<CityQuery>,
) -> ServerResult<Json<WeatherData>> {
    let request = serde_json::json!({ "city": city.name });
    
    let result = plugin_manager
        .call_plugin("weather-plugin", "get_weather", &request.to_string())
        .await
        .map_err(|e| ServerError::internal(e.to_string()))?;
    
    let weather: WeatherData = serde_json::from_str(&result)
        .map_err(|e| ServerError::internal(e.to_string()))?;
    
    Ok(Json(weather))
}

#[derive(Deserialize)]
struct CityQuery {
    name: String,
}
```

**使用**:
```bash
curl http://localhost:8080/api/v1/weather?name=London | jq
```

### 方式 2: 记忆增强中间件

**优点**:
- ✅ 自动化
- ✅ 对用户透明
- ✅ 统一增强逻辑

**实现**:
```rust
// 在 MemoryManager 中添加

pub async fn add_memory_with_enrichment(
    &self,
    mut memory: MemoryItem,
    enrich_options: EnrichOptions,
) -> Result<MemoryItem> {
    // 如果启用天气增强
    if enrich_options.weather {
        if let Some(location) = extract_location(&memory.content) {
            let weather = self
                .call_weather_plugin(&location)
                .await
                .ok();
            
            if let Some(w) = weather {
                memory.metadata.insert("weather", w);
            }
        }
    }
    
    // 保存记忆
    self.memory.add(memory).await
}
```

### 方式 3: 定时后台任务

**优点**:
- ✅ 不阻塞主流程
- ✅ 可批量处理
- ✅ 支持缓存更新

**实现**: 见上文"定时任务调用"示例

---

## 📊 总结

### 当前状态

| 场景 | 状态 | 说明 |
|------|------|------|
| **测试调用** | ✅ 已实现 | 在测试中可以调用 |
| **直接 API** | ⚠️ 需实现 | 没有专用端点 |
| **自动触发** | ❌ 不支持 | DataSource 无钩子 |
| **自定义集成** | ✅ 可用 | 需手动编码 |

### 关键要点

1. **Weather Plugin 不会自动触发** - 需要主动调用
2. **没有专用 API 端点** - 需要添加路由
3. **可通过代码集成** - PluginManager.call_plugin()
4. **性能优秀** - LRU 缓存，首次 31ms，后续 ~1ms

### 建议行动

1. ✅ **添加 API 端点** - 方便直接调用插件
2. ✅ **实现记忆增强** - 自动为记忆添加天气数据
3. ✅ **添加定时任务** - 定期更新天气缓存
4. ⚠️ **考虑添加 DataSource 钩子** - 统一外部数据源接口

---

**🎯 简单回答**: Weather Plugin 目前主要通过 `PluginManager.call_plugin()` 方法**手动调用**，没有自动触发机制。可以在测试、自定义代码、或（建议添加的）API 端点中使用。

