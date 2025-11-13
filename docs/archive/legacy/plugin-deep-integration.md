# AgentMem 插件系统深度集成方案

**版本**: v3.0  
**日期**: 2025-11-04  
**状态**: 📋 设计完成，待实现

---

## 🎯 集成目标

将插件系统深度集成到 AgentMem 的核心操作中，使插件能够：

1. **拦截和增强记忆操作** - 在添加、更新、删除记忆时调用插件
2. **自定义搜索算法** - 使用插件实现的搜索算法
3. **代码分析增强** - 使用插件分析不同语言的代码
4. **数据源集成** - 通过插件集成外部数据源
5. **LLM 增强** - 使用插件提供额外的 LLM 功能

---

## 🏗️ 集成架构

### 1. Memory 结构扩展

```rust
pub struct Memory {
    /// 内部编排器
    orchestrator: Arc<RwLock<MemoryOrchestrator>>,
    
    /// 插件增强层（可选）
    #[cfg(feature = "plugins")]
    plugin_layer: Arc<RwLock<PluginEnhancedMemory>>,
    
    /// 其他字段...
}
```

### 2. 操作流程

```
用户操作
    ↓
Memory API
    ↓
插件钩子（before_*）
    ↓
核心操作（Orchestrator）
    ↓
插件钩子（after_*）
    ↓
返回结果
```

---

## 📋 插件钩子设计

### 记忆操作钩子

```rust
pub trait PluginHooks {
    // 添加记忆前
    async fn before_add_memory(&self, memory: &mut MemoryItem) -> Result<()>;
    
    // 添加记忆后
    async fn after_add_memory(&self, memory: &MemoryItem) -> Result<()>;
    
    // 搜索前（可修改查询）
    async fn before_search(&self, query: &mut String) -> Result<()>;
    
    // 搜索后（可修改结果）
    async fn after_search(&self, results: &mut Vec<MemoryItem>) -> Result<()>;
    
    // 更新记忆前
    async fn before_update_memory(&self, memory: &mut MemoryItem) -> Result<()>;
    
    // 删除记忆前
    async fn before_delete_memory(&self, id: &str) -> Result<bool>; // false = 取消删除
}
```

### 插件类型与钩子映射

| 插件类型 | 触发钩子 | 用途 |
|---------|---------|------|
| MemoryProcessor | before_add_memory | 清洗、格式化、提取元数据 |
| SearchAlgorithm | before_search, after_search | 自定义搜索、重排序 |
| CodeAnalyzer | before_add_memory | 代码理解和分析 |
| DataSource | （主动调用） | 外部数据导入 |

---

## 🔧 集成实现方案

### Phase 1: 基础集成 ✅ 已完成

1. **依赖配置**
```toml
[dependencies]
agent-mem-plugins = { path = "../agent-mem-plugins", optional = true }

[features]
plugins = ["agent-mem-plugins"]
```

2. **模块导出**
```rust
#[cfg(feature = "plugins")]
pub use agent_mem_plugins as plugins;
```

3. **创建集成模块**
- `src/plugin_integration.rs` - 插件集成逻辑
- `PluginEnhancedMemory` - 插件增强包装器

### Phase 2: Memory 集成 📋 待实现

#### 2.1 扩展 Memory 结构

```rust
// src/memory.rs
pub struct Memory {
    orchestrator: Arc<RwLock<MemoryOrchestrator>>,
    
    #[cfg(feature = "plugins")]
    plugin_layer: Arc<RwLock<PluginEnhancedMemory>>,
    
    default_user_id: Option<String>,
    default_agent_id: String,
}
```

#### 2.2 添加插件相关方法

```rust
impl Memory {
    /// 注册插件
    #[cfg(feature = "plugins")]
    pub fn register_plugin(&self, plugin: RegisteredPlugin) -> Result<()> {
        let mut plugin_layer = self.plugin_layer.write().await;
        plugin_layer.register_plugin(plugin)
    }
    
    /// 列出已注册插件
    #[cfg(feature = "plugins")]
    pub fn list_plugins(&self) -> Vec<PluginMetadata> {
        let plugin_layer = self.plugin_layer.read().await;
        plugin_layer.registry().list()
            .iter()
            .map(|p| p.metadata.clone())
            .collect()
    }
    
    /// 启用/禁用插件
    #[cfg(feature = "plugins")]
    pub fn enable_plugin(&self, plugin_id: &str, enabled: bool) -> Result<()> {
        // TODO: 实现
    }
}
```

#### 2.3 集成到 add() 操作

```rust
pub async fn add(&self, content: impl Into<String>) -> Result<String> {
    let content = content.into();
    let mut memory = self.create_memory_item(content);
    
    // 插件前钩子
    #[cfg(feature = "plugins")]
    {
        let plugin_layer = self.plugin_layer.read().await;
        plugin_layer.before_add_memory(&mut memory).await?;
    }
    
    // 核心操作
    let result = self.add_with_options(memory, None).await?;
    
    // 插件后钩子
    #[cfg(feature = "plugins")]
    {
        let plugin_layer = self.plugin_layer.read().await;
        plugin_layer.after_add_memory(&result.memory).await?;
    }
    
    Ok(result.id)
}
```

#### 2.4 集成到 search() 操作

```rust
pub async fn search(&self, query: impl Into<String>) -> Result<Vec<MemoryItem>> {
    let mut query = query.into();
    
    // 插件前钩子（可修改查询）
    #[cfg(feature = "plugins")]
    {
        let plugin_layer = self.plugin_layer.read().await;
        plugin_layer.before_search(&mut query).await?;
    }
    
    // 核心搜索
    let mut results = self.search_with_options(&query, None).await?;
    
    // 插件后钩子（可重排序结果）
    #[cfg(feature = "plugins")]
    {
        let plugin_layer = self.plugin_layer.read().await;
        plugin_layer.after_search(&mut results).await?;
    }
    
    Ok(results)
}
```

### Phase 3: Builder 集成 📋 待实现

#### 3.1 添加插件配置选项

```rust
// src/builder.rs
pub struct MemoryBuilder {
    config: OrchestratorConfig,
    
    #[cfg(feature = "plugins")]
    plugin_paths: Vec<String>,
    
    #[cfg(feature = "plugins")]
    enable_default_plugins: bool,
    
    // 其他字段...
}

impl MemoryBuilder {
    /// 加载插件
    #[cfg(feature = "plugins")]
    pub fn with_plugin(mut self, path: impl Into<String>) -> Self {
        self.plugin_paths.push(path.into());
        self
    }
    
    /// 启用默认插件
    #[cfg(feature = "plugins")]
    pub fn enable_default_plugins(mut self) -> Self {
        self.enable_default_plugins = true;
        self
    }
    
    /// 从目录加载所有插件
    #[cfg(feature = "plugins")]
    pub fn load_plugins_from_dir(mut self, dir: impl Into<String>) -> Self {
        // TODO: 扫描目录并加载所有 .wasm 文件
        self
    }
}
```

#### 3.2 在 build() 中初始化插件

```rust
pub async fn build(self) -> Result<Memory> {
    let orchestrator = MemoryOrchestrator::new(self.config).await?;
    
    #[cfg(feature = "plugins")]
    let plugin_layer = {
        let mut layer = PluginEnhancedMemory::new();
        
        // 加载配置的插件
        for path in self.plugin_paths {
            layer.load_plugin_from_path(&path).await?;
        }
        
        // 加载默认插件
        if self.enable_default_plugins {
            layer.load_default_plugins().await?;
        }
        
        Arc::new(RwLock::new(layer))
    };
    
    Ok(Memory {
        orchestrator: Arc::new(RwLock::new(orchestrator)),
        #[cfg(feature = "plugins")]
        plugin_layer,
        default_user_id: self.default_user_id,
        default_agent_id: self.default_agent_id,
    })
}
```

### Phase 4: 高级插件功能 📋 待实现

#### 4.1 插件事件系统

```rust
pub enum PluginEvent {
    MemoryAdded { id: String, content: String },
    MemoryUpdated { id: String },
    MemoryDeleted { id: String },
    SearchPerformed { query: String, result_count: usize },
}

impl Memory {
    /// 订阅插件事件
    #[cfg(feature = "plugins")]
    pub fn subscribe_to_events(
        &self,
        plugin_id: &str,
        callback: impl Fn(PluginEvent) + Send + Sync + 'static,
    ) -> Result<()> {
        // TODO: 实现事件订阅
    }
}
```

#### 4.2 插件配置管理

```rust
impl Memory {
    /// 更新插件配置
    #[cfg(feature = "plugins")]
    pub fn update_plugin_config(
        &self,
        plugin_id: &str,
        config: PluginConfig,
    ) -> Result<()> {
        // TODO: 实现配置更新
    }
    
    /// 获取插件配置
    #[cfg(feature = "plugins")]
    pub fn get_plugin_config(&self, plugin_id: &str) -> Result<PluginConfig> {
        // TODO: 实现配置获取
    }
}
```

#### 4.3 插件性能监控

```rust
impl Memory {
    /// 获取插件性能指标
    #[cfg(feature = "plugins")]
    pub fn plugin_metrics(&self, plugin_id: &str) -> Result<ExecutionMetrics> {
        let plugin_layer = self.plugin_layer.read().await;
        plugin_layer.get_plugin_metrics(plugin_id)
    }
    
    /// 获取所有插件性能指标
    #[cfg(feature = "plugins")]
    pub fn all_plugin_metrics(&self) -> HashMap<String, ExecutionMetrics> {
        let plugin_layer = self.plugin_layer.read().await;
        plugin_layer.get_all_metrics()
    }
}
```

---

## 🧪 测试计划

### 集成测试

```rust
#[cfg(all(test, feature = "plugins"))]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_memory_with_plugins() {
        let mem = Memory::builder()
            .with_plugin("target/wasm32-wasip1/release/memory_processor.wasm")
            .build()
            .await
            .unwrap();
        
        // 添加记忆应该触发插件
        let id = mem.add("Test content").await.unwrap();
        
        // 验证插件已处理
        let plugins = mem.list_plugins();
        assert!(!plugins.is_empty());
    }
    
    #[tokio::test]
    async fn test_search_with_plugin() {
        let mem = Memory::builder()
            .with_plugin("target/wasm32-wasip1/release/search_plugin.wasm")
            .build()
            .await
            .unwrap();
        
        mem.add("Rust programming").await.unwrap();
        
        // 搜索应该使用插件算法
        let results = mem.search("Rust").await.unwrap();
        assert!(!results.is_empty());
    }
}
```

---

## 📊 性能影响评估

### 插件开销

| 操作 | 无插件 | 有插件（未激活） | 有插件（激活） |
|------|--------|----------------|---------------|
| add() | 1x | 1.001x | 1.1x - 1.5x |
| search() | 1x | 1.001x | 1.2x - 2.0x |
| 内存开销 | 基准 | +1MB | +10-50MB |

### 优化策略

1. **延迟加载**: 只在需要时加载插件
2. **LRU 缓存**: 缓存最近使用的插件实例
3. **异步执行**: 非关键插件异步执行
4. **批量处理**: 批量调用插件减少开销

---

## 🔐 安全考虑

### 1. 权限控制

- 插件只能访问授权的能力
- 资源限制严格执行
- 沙盒隔离防止恶意代码

### 2. 数据保护

- 敏感数据加密后传给插件
- 插件无法访问底层存储
- 审计日志记录所有插件操作

### 3. 插件验证

- 数字签名验证
- 来源白名单
- 定期安全扫描

---

## 📚 使用示例

### 基础集成

```rust
use agent_mem::Memory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 启用插件功能
    let mem = Memory::builder()
        .with_storage("libsql://agentmem.db")
        .with_plugin("plugins/memory_processor.wasm")
        .with_plugin("plugins/search_algorithm.wasm")
        .build()
        .await?;
    
    // 使用带插件增强的记忆系统
    mem.add("I love Rust programming").await?;
    
    let results = mem.search("programming").await?;
    println!("Found {} memories", results.len());
    
    // 查看插件状态
    #[cfg(feature = "plugins")]
    {
        let plugins = mem.list_plugins();
        for plugin in plugins {
            println!("Plugin: {} v{}", plugin.name, plugin.version);
        }
    }
    
    Ok(())
}
```

### 高级配置

```rust
let mem = Memory::builder()
    .with_storage("libsql://agentmem.db")
    .load_plugins_from_dir("./plugins")
    .enable_default_plugins()
    .build()
    .await?;

// 动态注册插件
#[cfg(feature = "plugins")]
{
    use agent_mem::plugins::*;
    
    let metadata = PluginMetadata {
        name: "custom-plugin".to_string(),
        version: "1.0.0".to_string(),
        description: "My custom plugin".to_string(),
        author: "Me".to_string(),
        plugin_type: PluginType::MemoryProcessor,
        required_capabilities: vec![Capability::MemoryAccess],
        config_schema: None,
    };
    
    let plugin = RegisteredPlugin {
        id: "custom-plugin".to_string(),
        metadata,
        path: "custom.wasm".to_string(),
        status: PluginStatus::Registered,
        config: PluginConfig::default(),
        registered_at: chrono::Utc::now(),
        last_loaded_at: None,
    };
    
    mem.register_plugin(plugin)?;
}
```

---

## 🎯 实施计划

### Phase 1: 基础框架 ✅ 已完成
- [x] 创建 plugin_integration.rs
- [x] 实现 PluginEnhancedMemory
- [x] 定义 PluginHooks trait
- [x] 基础测试

### Phase 2: Memory 集成 📋 下一步
- [ ] 扩展 Memory 结构
- [ ] 集成到 add() 操作
- [ ] 集成到 search() 操作
- [ ] 添加插件管理方法
- [ ] 编写集成测试

### Phase 3: Builder 集成
- [ ] 添加插件配置选项
- [ ] 实现插件自动加载
- [ ] 支持默认插件
- [ ] 文档和示例

### Phase 4: 高级功能
- [ ] 插件事件系统
- [ ] 插件配置管理
- [ ] 性能监控集成
- [ ] 安全审计

---

## 📖 相关文档

- [plugin.md](plugin.md) - 插件系统完整设计
- [PLUGIN_INTEGRATION_GUIDE.md](PLUGIN_INTEGRATION_GUIDE.md) - 集成指南
- [PLUGIN_FINAL_VERIFICATION.md](PLUGIN_FINAL_VERIFICATION.md) - 验证报告

---

**文档版本**: v3.0  
**最后更新**: 2025-11-04  
**状态**: 📋 Phase 1 完成，Phase 2-4 待实现

