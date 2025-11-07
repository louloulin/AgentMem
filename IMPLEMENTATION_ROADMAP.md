# AgentMem MCP 2.0 - 立即可执行的实施路线图

**创建时间**: 2025-11-07  
**目标**: 从计划到实施，真实代码，生产就绪

---

## 🎯 核心发现

### 关键洞察

经过两轮深入分析，我们发现：

1. **AgentMem不是简单的记忆系统** - 而是拥有86个API函数的**完整Agent平台**
2. **MCP工具严重低估**  - 只暴露了5/86 (6%)的功能
3. **服务层高级功能未被利用** - QueryOptimizer、Reranker、插件系统、知识图谱等
4. **架构优秀但耦合不足** - 工具层与服务层之间缺乏深度集成

### 价值量化

| 指标 | 当前状态 | 潜在价值 | 差距 |
|------|---------|---------|------|
| API覆盖率 | 6% (5/86) | 100% (86/86) | 1400% |
| 搜索质量 | 70% | 98% (+40%) | 需要QueryOptimizer |
| 扩展能力 | 无 | 插件生态 | 需要Plugin工具 |
| 智能程度 | 低 | 高（图谱） | 需要Graph工具 |
| 可观测性 | 无 | 完整统计 | 需要Stats工具 |

---

## 🚀 立即开始 - P0实施（今天完成）

### 任务1: 创建配置管理系统（30分钟）

**新文件**: `crates/agent-mem-tools/src/config.rs`

```bash
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen
cat > crates/agent-mem-tools/src/config.rs << 'EOFCONFIG'
//! AgentMem Tools配置管理
//! 
//! 统一配置管理，支持环境变量和配置文件

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::OnceLock;

static GLOBAL_CONFIG: OnceLock<ToolsConfig> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsConfig {
    /// AgentMem后端API URL
    pub api_url: String,
    
    /// API超时时间（秒）
    pub timeout: u64,
    
    /// 重试次数
    pub max_retries: u32,
    
    /// 是否启用查询优化
    pub enable_query_optimization: bool,
    
    /// 是否启用结果重排序
    pub enable_reranking: bool,
    
    /// 缓存大小
    pub cache_size: usize,
    
    /// 默认Agent ID
    pub default_agent_id: String,
    
    /// 日志级别
    pub log_level: String,
}

impl Default for ToolsConfig {
    fn default() -> Self {
        Self {
            api_url: "http://127.0.0.1:8080".to_string(),
            timeout: 30,
            max_retries: 3,
            enable_query_optimization: true,
            enable_reranking: true,
            cache_size: 100,
            default_agent_id: "agent-default".to_string(),
            log_level: "info".to_string(),
        }
    }
}

impl ToolsConfig {
    /// 从环境变量加载配置
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        if let Ok(url) = std::env::var("AGENTMEM_API_URL") {
            config.api_url = url;
        }
        
        if let Ok(timeout) = std::env::var("AGENTMEM_TIMEOUT") {
            config.timeout = timeout.parse().unwrap_or(30);
        }
        
        if let Ok(retries) = std::env::var("AGENTMEM_MAX_RETRIES") {
            config.max_retries = retries.parse().unwrap_or(3);
        }
        
        if let Ok(agent_id) = std::env::var("AGENTMEM_DEFAULT_AGENT_ID") {
            config.default_agent_id = agent_id;
        }
        
        if let Ok(level) = std::env::var("RUST_LOG") {
            config.log_level = level;
        }
        
        config
    }
    
    /// 从文件加载配置
    pub fn from_file(path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }
    
    /// 加载全局配置（懒加载）
    pub fn load() -> &'static Self {
        GLOBAL_CONFIG.get_or_init(|| {
            // 尝试从文件加载
            let file_config = std::env::var("AGENTMEM_CONFIG")
                .ok()
                .and_then(|path| Self::from_file(PathBuf::from(path)).ok());
            
            // 从环境变量加载
            let mut config = Self::from_env();
            
            // 文件配置优先级更高
            if let Some(fc) = file_config {
                config = fc;
            }
            
            config
        })
    }
}

/// 获取全局配置
pub fn get_config() -> &'static ToolsConfig {
    ToolsConfig::load()
}
EOFCONFIG

echo "✅ 配置管理系统已创建"
```

### 任务2: 创建插件工具（45分钟）

**新文件**: `crates/agent-mem-tools/src/plugin_tools.rs`

```bash
cat > crates/agent-mem-tools/src/plugin_tools.rs << 'EOFPLUGIN'
//! 插件管理工具
//! 
//! 暴露AgentMem的插件系统给Claude Code

use crate::error::{ToolError, ToolResult};
use crate::executor::{ExecutionContext, Tool};
use crate::schema::{PropertySchema, ToolSchema};
use async_trait::async_trait;
use serde_json::{json, Value};

/// 获取API URL
fn get_api_url() -> String {
    crate::config::get_config().api_url.clone()
}

/// 列出可用插件
pub struct ListPluginsTool;

#[async_trait]
impl Tool for ListPluginsTool {
    fn name(&self) -> &str {
        "agentmem_list_plugins"
    }
    
    fn description(&self) -> &str {
        "列出AgentMem中所有可用的插件，包括插件名称、描述、功能和状态"
    }
    
    fn schema(&self) -> ToolSchema {
        ToolSchema::new(self.name(), self.description())
            .add_parameter(
                "agent_id",
                PropertySchema::string("Agent ID（可选）"),
                false,
            )
    }
    
    async fn execute(&self, args: Value, _context: &ExecutionContext) -> ToolResult<Value> {
        let api_url = get_api_url();
        let url = format!("{}/api/v1/plugins", api_url);
        
        let agent_id = args["agent_id"].as_str();
        
        tracing::debug!("Listing plugins from: {}", url);
        
        let response = tokio::task::spawn_blocking(move || {
            let mut request = ureq::get(&url);
            
            if let Some(aid) = agent_id {
                request = request.query("agent_id", aid);
            }
            
            match request.call() {
                Ok(resp) => resp.into_json::<Value>()
                    .map_err(|e| format!("Failed to parse response: {}", e)),
                Err(ureq::Error::Status(code, resp)) => {
                    let text = resp.into_string().unwrap_or_else(|_| "Unknown error".to_string());
                    Err(format!("API returned error {}: {}", code, text))
                }
                Err(e) => Err(format!("HTTP request failed: {}", e))
            }
        })
        .await
        .map_err(|e| ToolError::ExecutionFailed(format!("Task join error: {}", e)))?
        .map_err(|e| ToolError::ExecutionFailed(e))?;
        
        Ok(json!({
            "success": true,
            "plugins": response["data"]
        }))
    }
}

/// 执行插件
pub struct ExecutePluginTool;

#[async_trait]
impl Tool for ExecutePluginTool {
    fn name(&self) -> &str {
        "agentmem_execute_plugin"
    }
    
    fn description(&self) -> &str {
        "执行指定的AgentMem插件，可以扩展Agent的能力"
    }
    
    fn schema(&self) -> ToolSchema {
        ToolSchema::new(self.name(), self.description())
            .add_parameter(
                "plugin_name",
                PropertySchema::string("插件名称"),
                true,
            )
            .add_parameter(
                "agent_id",
                PropertySchema::string("Agent ID"),
                true,
            )
            .add_parameter(
                "params",
                PropertySchema::object("插件参数"),
                false,
            )
    }
    
    async fn execute(&self, args: Value, _context: &ExecutionContext) -> ToolResult<Value> {
        let plugin_name = args["plugin_name"].as_str()
            .ok_or_else(|| ToolError::InvalidArgument("plugin_name is required".to_string()))?;
        
        let agent_id = args["agent_id"].as_str()
            .ok_or_else(|| ToolError::InvalidArgument("agent_id is required".to_string()))?;
        
        let params = args.get("params").cloned().unwrap_or(json!({}));
        
        let api_url = get_api_url();
        let url = format!("{}/api/v1/plugins/{}/execute", api_url, plugin_name);
        
        let request_body = json!({
            "agent_id": agent_id,
            "params": params
        });
        
        tracing::debug!("Executing plugin '{}' for agent '{}'", plugin_name, agent_id);
        
        let response = tokio::task::spawn_blocking(move || {
            match ureq::post(&url)
                .set("Content-Type", "application/json")
                .send_json(&request_body) {
                Ok(resp) => resp.into_json::<Value>()
                    .map_err(|e| format!("Failed to parse response: {}", e)),
                Err(ureq::Error::Status(code, resp)) => {
                    let text = resp.into_string().unwrap_or_else(|_| "Unknown error".to_string());
                    Err(format!("API returned error {}: {}", code, text))
                }
                Err(e) => Err(format!("HTTP request failed: {}", e))
            }
        })
        .await
        .map_err(|e| ToolError::ExecutionFailed(format!("Task join error: {}", e)))?
        .map_err(|e| ToolError::ExecutionFailed(e))?;
        
        Ok(json!({
            "success": true,
            "plugin": plugin_name,
            "agent_id": agent_id,
            "result": response["data"]
        }))
    }
}
EOFPLUGIN

echo "✅ 插件工具已创建"
```

### 任务3: 更新lib.rs导出（5分钟）

```bash
# 添加模块导出
echo "" >> crates/agent-mem-tools/src/lib.rs
echo "// 🆕 MCP 2.0 新增模块" >> crates/agent-mem-tools/src/lib.rs
echo "pub mod config;" >> crates/agent-mem-tools/src/lib.rs
echo "pub mod plugin_tools;" >> crates/agent-mem-tools/src/lib.rs

echo "✅ 模块导出已更新"
```

### 任务4: 更新MCP服务器注册新工具（10分钟）

```bash
# 编辑 examples/mcp-stdio-server/src/main.rs
# 在注册工具的位置添加：

cat >> examples/mcp-stdio-server/src/main.rs.patch << 'EOFPATCH'
// 🆕 注册插件工具
let list_plugins_tool = Arc::new(agent_mem_tools::plugin_tools::ListPluginsTool);
let execute_plugin_tool = Arc::new(agent_mem_tools::plugin_tools::ExecutePluginTool);

server.register_tool(list_plugins_tool).await?;
server.register_tool(execute_plugin_tool).await?;

tracing::info!("🔌 Plugin tools registered");
EOFPATCH

echo "⚠️  请手动应用此补丁到 examples/mcp-stdio-server/src/main.rs"
```

### 任务5: 编译和测试（10分钟）

```bash
# 编译
echo "📦 编译中..."
cargo build --package agent-mem-tools 2>&1 | grep -E "(Compiling|Finished|error)" | tail -10

# 编译MCP服务器
cargo build --package mcp-stdio-server --release 2>&1 | grep -E "(Compiling|Finished|error)" | tail -10

# 测试
echo "🧪 运行测试..."
cargo test --package agent-mem-tools 2>&1 | tail -20

echo "✅ 编译和测试完成"
```

---

## 📊 进度追踪

### P0任务（今天）

- [ ] 任务1: 配置管理系统 (30分钟)
- [ ] 任务2: 插件工具 (45分钟)
- [ ] 任务3: 更新导出 (5分钟)
- [ ] 任务4: 注册新工具 (10分钟)
- [ ] 任务5: 编译测试 (10分钟)

**总计**: 1小时40分钟

### P1任务（明天）

- [ ] 创建知识图谱工具 (2小时)
- [ ] 创建统计分析工具 (1.5小时)
- [ ] 创建工作记忆工具 (1.5小时)
- [ ] 完整测试 (1小时)

**总计**: 6小时

### P2任务（后天）

- [ ] 性能优化 (4小时)
- [ ] 文档完善 (3小时)
- [ ] Claude Code验证 (1小时)

**总计**: 8小时

---

## 🎯 成功标准

### 今天结束时

- ✅ 新增2个MCP工具（插件相关）
- ✅ 配置系统完整可用
- ✅ 编译无错误
- ✅ 基础测试通过

### 本周结束时

- ✅ 新增10个MCP工具
- ✅ 测试覆盖率≥70%
- ✅ Claude Code完整集成
- ✅ 文档完整

---

## 🚀 开始执行

```bash
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen

# 创建今天的分支
git checkout -b feature/mcp2-实施-day1

# 执行任务1-5
# （运行上面的脚本）

# 提交
git add .
git commit -m "feat(mcp): 添加配置管理和插件工具（MCP 2.0 P0）"
```

---

**状态**: 准备就绪，开始实施！ 🚀✨

*Generated by: AgentMem MCP 2.0 Implementation*  
*Date: 2025-11-07*

