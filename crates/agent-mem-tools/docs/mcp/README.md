# MCP (Model Context Protocol) 文档

欢迎使用 AgentMem MCP 文档！本文档集提供了使用 AgentMem 实现 MCP 的完整指南。

## 📚 文档导航

### 入门指南

- **[快速开始](QUICKSTART.md)** - 5 分钟快速上手 MCP
  - 安装和配置
  - 第一个 MCP 服务端
  - 第一个 MCP 客户端
  - 常见问题

### 开发指南

- **[服务端开发指南](SERVER_GUIDE.md)** - 如何开发 MCP 服务端
  - 核心概念
  - 实现工具
  - 实现资源
  - 实现提示词
  - 传输层
  - 安全和认证
  - 最佳实践

- **[客户端开发指南](CLIENT_GUIDE.md)** - 如何开发 MCP 客户端
  - 核心概念
  - 调用工具
  - 访问资源
  - 使用提示词
  - 传输层
  - 错误处理
  - 最佳实践

### 进阶主题

- **[最佳实践](BEST_PRACTICES.md)** - MCP 开发的最佳实践
  - 架构设计
  - 工具设计
  - 资源管理
  - 性能优化
  - 安全性
  - 错误处理
  - 测试
  - 监控和日志
  - 部署

### API 参考

- **[API 参考](API_REFERENCE.md)** - 完整的 API 文档
  - 服务端 API
  - 客户端 API
  - 数据类型
  - 错误类型
  - 协议规范

## 🚀 快速链接

### 我想...

- **开始使用 MCP** → [快速开始](QUICKSTART.md)
- **创建一个 MCP 服务端** → [服务端开发指南](SERVER_GUIDE.md)
- **连接到 MCP 服务端** → [客户端开发指南](CLIENT_GUIDE.md)
- **了解最佳实践** → [最佳实践](BEST_PRACTICES.md)
- **查找 API 文档** → [API 参考](API_REFERENCE.md)

### 常见任务

#### 创建一个简单的工具

```rust
use agent_mem_tools::mcp::{McpTool, ToolContext};
use serde_json::Value;

fn create_calculator_tool() -> McpTool {
    McpTool {
        name: "calculator".to_string(),
        description: "执行数学计算".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "数学表达式"
                }
            },
            "required": ["expression"]
        }),
        handler: Box::new(|args: Value, _ctx: &ToolContext| {
            Box::pin(async move {
                let expr = args["expression"].as_str()
                    .ok_or("Missing expression")?;
                let result = evaluate(expr)?;
                Ok(serde_json::json!({"result": result}))
            })
        }),
    }
}
```

详细信息请参考 [服务端开发指南 - 实现工具](SERVER_GUIDE.md#实现工具)

#### 调用一个工具

```rust
use agent_mem_tools::mcp::{McpClient, McpClientConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = McpClientConfig {
        server_command: "my-mcp-server".to_string(),
        ..Default::default()
    };
    
    let mut client = McpClient::new(config);
    client.connect().await?;
    
    let result = client.call_tool(
        "calculator",
        serde_json::json!({"expression": "2 + 2"})
    ).await?;
    
    println!("Result: {:?}", result);
    
    Ok(())
}
```

详细信息请参考 [客户端开发指南 - 调用工具](CLIENT_GUIDE.md#调用工具)

#### 提供资源

```rust
use agent_mem_tools::mcp::{McpResource, ResourceProvider};

struct MyResourceProvider;

#[async_trait::async_trait]
impl ResourceProvider for MyResourceProvider {
    async fn list_resources(&self) -> Result<Vec<McpResource>, Box<dyn std::error::Error>> {
        Ok(vec![
            McpResource {
                uri: "file:///path/to/file.txt".to_string(),
                name: "示例文件".to_string(),
                description: Some("一个示例文件".to_string()),
                mime_type: Some("text/plain".to_string()),
            },
        ])
    }
    
    async fn read_resource(&self, uri: &str) -> Result<String, Box<dyn std::error::Error>> {
        // 读取资源内容
        Ok("资源内容".to_string())
    }
}
```

详细信息请参考 [服务端开发指南 - 实现资源](SERVER_GUIDE.md#实现资源)

## 📖 概念说明

### 什么是 MCP？

MCP (Model Context Protocol) 是一个标准化的协议，用于 AI 模型与外部工具、资源和服务的交互。它提供了：

- **工具 (Tools)**: AI 模型可以调用的函数
- **资源 (Resources)**: AI 模型可以访问的数据源
- **提示词 (Prompts)**: 预定义的提示词模板

### MCP 架构

```
┌─────────────────────────────────────────────────────────┐
│                      AI 模型                             │
│                   (GPT-4, Claude, etc.)                  │
└─────────────────────┬───────────────────────────────────┘
                      │
                      │ MCP 协议
                      │
┌─────────────────────▼───────────────────────────────────┐
│                   MCP 客户端                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │   工具调用    │  │   资源访问    │  │  提示词使用   │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
└─────────────────────┬───────────────────────────────────┘
                      │
                      │ 传输层 (Stdio/SSE/HTTP)
                      │
┌─────────────────────▼───────────────────────────────────┐
│                   MCP 服务端                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │   工具管理    │  │   资源管理    │  │  提示词管理   │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
└─────────────────────────────────────────────────────────┘
```

### 核心组件

#### 1. 工具 (Tools)

工具是 AI 模型可以调用的函数。每个工具包含：
- 名称和描述
- 输入参数的 JSON Schema
- 执行逻辑

示例：计算器、文件读写、数据库查询、API 调用等。

#### 2. 资源 (Resources)

资源是 AI 模型可以访问的数据源。每个资源包含：
- URI（唯一标识符）
- 名称和描述
- MIME 类型
- 内容

示例：文件、配置、数据库记录、API 响应等。

#### 3. 提示词 (Prompts)

提示词是预定义的模板，可以包含变量。每个提示词包含：
- 名称和描述
- 参数定义
- 消息模板

示例：代码审查、文档生成、问题回答等。

## 🎯 使用场景

### 1. AI 助手

使用 MCP 为 AI 助手提供工具和资源访问能力：

```rust
// 注册文件操作工具
server.register_tool(create_read_file_tool()).await?;
server.register_tool(create_write_file_tool()).await?;

// 注册代码执行工具
server.register_tool(create_execute_code_tool()).await?;

// 注册搜索工具
server.register_tool(create_web_search_tool()).await?;
```

### 2. 代码生成

使用 MCP 为代码生成提供上下文：

```rust
// 提供项目文件作为资源
server.register_resource_provider(
    Box::new(ProjectFilesProvider::new("./src"))
).await?;

// 提供代码模板作为提示词
server.register_prompt_provider(
    Box::new(CodeTemplateProvider::new())
).await?;
```

### 3. 数据分析

使用 MCP 为数据分析提供数据访问：

```rust
// 注册数据库查询工具
server.register_tool(create_sql_query_tool()).await?;

// 注册数据可视化工具
server.register_tool(create_plot_tool()).await?;

// 提供数据集作为资源
server.register_resource_provider(
    Box::new(DatasetProvider::new())
).await?;
```

## 🔧 示例项目

AgentMem 提供了多个示例项目，展示 MCP 的各种用法：

- `examples/mcp-resources-demo/` - 资源管理示例
- `examples/mcp-prompts-demo/` - 提示词使用示例
- `examples/mcp-transport-demo/` - 传输层示例
- `examples/mcp-tool-discovery-demo/` - 工具发现示例
- `examples/mcp-auth-demo/` - 认证示例

## 🤝 贡献

欢迎贡献！如果您发现文档中的错误或有改进建议，请：

1. 提交 Issue
2. 创建 Pull Request
3. 联系维护者

## 📝 许可证

本文档遵循 MIT 许可证。

## 🔗 相关链接

- [AgentMem 主页](../../README.md)
- [MCP 规范](https://modelcontextprotocol.io/)
- [示例项目](../../../examples/)

## 📮 获取帮助

如果您在使用 MCP 时遇到问题：

1. 查看 [快速开始](QUICKSTART.md) 中的常见问题
2. 阅读 [最佳实践](BEST_PRACTICES.md)
3. 查看 [API 参考](API_REFERENCE.md)
4. 提交 Issue

---

**祝您使用愉快！** 🎉

