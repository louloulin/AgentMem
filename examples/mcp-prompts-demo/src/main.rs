//! MCP Prompts 功能演示
//!
//! 本示例演示了 MCP (Model Context Protocol) Prompts 功能的使用：
//! 1. 注册提示词模板
//! 2. 列出所有提示词
//! 3. 获取提示词（带参数渲染）
//! 4. 多内容类型提示词
//! 5. MCP 服务器集成

use agent_mem_tools::mcp::{
    PromptManager, McpPrompt, PromptArgument, PromptContent,
    McpServer, McpGetPromptRequest,
};
use agent_mem_tools::mcp::server::McpServerConfig;
use agent_mem_tools::executor::ToolExecutor;
use std::sync::Arc;
use std::collections::HashMap;
use tracing::{info, Level};
use tracing_subscriber;
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("🚀 MCP Prompts 功能演示");
    info!("{}", "=".repeat(60));

    // 演示 1: 提示词管理器功能
    demo_prompt_manager().await?;

    info!("");
    info!("{}", "=".repeat(60));

    // 演示 2: MCP 服务器集成
    demo_mcp_server().await?;

    info!("");
    info!("🎉 所有演示完成！");

    Ok(())
}

/// 演示提示词管理器功能
async fn demo_prompt_manager() -> anyhow::Result<()> {
    info!("📋 演示 1: 提示词管理器功能");
    info!("{}", "-".repeat(60));

    // 创建提示词管理器
    let manager = PromptManager::new();

    // 1. 注册简单提示词
    info!("1️⃣ 注册简单提示词:");
    let greeting_prompt = McpPrompt::new(
        "greeting",
        vec![PromptContent::Text {
            text: "Hello, {{name}}! Welcome to AgentMem.".to_string(),
        }],
    )
    .with_description("A simple greeting prompt")
    .with_argument(
        PromptArgument::new("name")
            .required()
            .with_type("string")
            .with_description("User name"),
    )
    .with_tag("greeting")
    .with_version("1.0.0");

    manager.register_prompt(greeting_prompt).await?;
    info!("  ✅ 注册成功: greeting");

    // 2. 注册复杂提示词
    info!("");
    info!("2️⃣ 注册复杂提示词:");
    let analysis_prompt = McpPrompt::new(
        "code_analysis",
        vec![
            PromptContent::Text {
                text: "Analyze the following {{language}} code:".to_string(),
            },
            PromptContent::Text {
                text: "```{{language}}\n{{code}}\n```".to_string(),
            },
            PromptContent::Text {
                text: "Focus on: {{focus_areas}}".to_string(),
            },
        ],
    )
    .with_description("Code analysis prompt")
    .with_argument(
        PromptArgument::new("language")
            .required()
            .with_type("string")
            .with_description("Programming language"),
    )
    .with_argument(
        PromptArgument::new("code")
            .required()
            .with_type("string")
            .with_description("Code to analyze"),
    )
    .with_argument(
        PromptArgument::new("focus_areas")
            .with_type("string")
            .with_description("Areas to focus on"),
    )
    .with_tag("code")
    .with_tag("analysis")
    .with_version("2.0.0");

    manager.register_prompt(analysis_prompt).await?;
    info!("  ✅ 注册成功: code_analysis");

    // 3. 注册带资源引用的提示词
    info!("");
    info!("3️⃣ 注册带资源引用的提示词:");
    let memory_prompt = McpPrompt::new(
        "memory_query",
        vec![
            PromptContent::Text {
                text: "Query: {{query}}".to_string(),
            },
            PromptContent::Resource {
                uri: "agentmem://memory/core".to_string(),
                mime_type: Some("application/json".to_string()),
            },
            PromptContent::Text {
                text: "Return relevant memories.".to_string(),
            },
        ],
    )
    .with_description("Memory query prompt with resource reference")
    .with_argument(
        PromptArgument::new("query")
            .required()
            .with_type("string")
            .with_description("Search query"),
    )
    .with_tag("memory")
    .with_version("1.0.0");

    manager.register_prompt(memory_prompt).await?;
    info!("  ✅ 注册成功: memory_query");

    // 4. 列出所有提示词
    info!("");
    info!("4️⃣ 列出所有提示词:");
    let prompts = manager.list_prompts().await?;
    info!("  找到 {} 个提示词:", prompts.len());
    for prompt in &prompts {
        info!("    - {} (v{})", prompt.name, prompt.version.as_ref().unwrap_or(&"N/A".to_string()));
        if let Some(desc) = &prompt.description {
            info!("      描述: {}", desc);
        }
        info!("      参数: {} 个", prompt.arguments.len());
        info!("      内容: {} 个部分", prompt.content.len());
        info!("      标签: {:?}", prompt.tags);
    }

    // 5. 获取并渲染提示词
    info!("");
    info!("5️⃣ 获取并渲染提示词:");
    
    // 渲染 greeting
    let mut args = HashMap::new();
    args.insert("name".to_string(), json!("Alice"));
    let response = manager.get_prompt("greeting", args).await?;
    info!("  greeting 渲染结果:");
    for content in &response.content {
        if let PromptContent::Text { text } = content {
            info!("    {}", text);
        }
    }

    // 渲染 code_analysis
    let mut args = HashMap::new();
    args.insert("language".to_string(), json!("Rust"));
    args.insert("code".to_string(), json!("fn main() { println!(\"Hello\"); }"));
    args.insert("focus_areas".to_string(), json!("performance, safety"));
    let response = manager.get_prompt("code_analysis", args).await?;
    info!("");
    info!("  code_analysis 渲染结果:");
    for (idx, content) in response.content.iter().enumerate() {
        if let PromptContent::Text { text } = content {
            info!("    Part {}: {}", idx + 1, text);
        }
    }

    // 渲染 memory_query
    let mut args = HashMap::new();
    args.insert("query".to_string(), json!("user preferences"));
    let response = manager.get_prompt("memory_query", args).await?;
    info!("");
    info!("  memory_query 渲染结果:");
    for (idx, content) in response.content.iter().enumerate() {
        match content {
            PromptContent::Text { text } => {
                info!("    Part {}: {}", idx + 1, text);
            }
            PromptContent::Resource { uri, .. } => {
                info!("    Part {}: Resource({})", idx + 1, uri);
            }
            _ => {}
        }
    }

    // 6. 测试必需参数验证
    info!("");
    info!("6️⃣ 测试必需参数验证:");
    let args = HashMap::new(); // 缺少必需参数
    let result = manager.get_prompt("greeting", args).await;
    if result.is_err() {
        info!("  ✅ 正确检测到缺少必需参数");
    } else {
        info!("  ❌ 应该检测到缺少必需参数");
    }

    Ok(())
}

/// 演示 MCP 服务器集成
async fn demo_mcp_server() -> anyhow::Result<()> {
    info!("📋 演示 2: MCP 服务器集成");
    info!("{}", "-".repeat(60));

    // 创建 MCP 服务器
    let config = McpServerConfig::default();
    let tool_executor = Arc::new(ToolExecutor::new());
    let server = McpServer::new(config, tool_executor);

    // 注册一些提示词
    let prompt1 = McpPrompt::new(
        "summarize",
        vec![PromptContent::Text {
            text: "Summarize the following text:\n\n{{text}}".to_string(),
        }],
    )
    .with_description("Text summarization prompt")
    .with_argument(
        PromptArgument::new("text")
            .required()
            .with_type("string"),
    );

    let prompt2 = McpPrompt::new(
        "translate",
        vec![PromptContent::Text {
            text: "Translate from {{source_lang}} to {{target_lang}}:\n\n{{text}}".to_string(),
        }],
    )
    .with_description("Translation prompt")
    .with_argument(PromptArgument::new("source_lang").required())
    .with_argument(PromptArgument::new("target_lang").required())
    .with_argument(PromptArgument::new("text").required());

    server.prompt_manager().register_prompt(prompt1).await?;
    server.prompt_manager().register_prompt(prompt2).await?;

    // 初始化服务器
    info!("1️⃣ 初始化 MCP 服务器:");
    server.initialize().await?;
    info!("  ✅ 服务器已初始化");

    // 获取服务器信息
    info!("");
    info!("2️⃣ 服务器信息:");
    let info_data = server.get_server_info();
    info!("  名称: {}", info_data.name);
    info!("  版本: {}", info_data.version);
    info!("  能力:");
    info!("    - Tools: {}", info_data.capabilities.tools);
    info!("    - Resources: {}", info_data.capabilities.resources);
    info!("    - Prompts: {}", info_data.capabilities.prompts);

    // 列出提示词
    info!("");
    info!("3️⃣ 列出 MCP 提示词:");
    let response = server.list_prompts().await?;
    info!("  找到 {} 个提示词:", response.prompts.len());
    for prompt in &response.prompts {
        info!("    - {}", prompt.name);
        if let Some(desc) = &prompt.description {
            info!("      {}", desc);
        }
    }

    // 获取提示词
    info!("");
    info!("4️⃣ 获取 MCP 提示词:");
    let mut args = HashMap::new();
    args.insert("text".to_string(), json!("This is a long text that needs to be summarized..."));
    
    let request = McpGetPromptRequest {
        name: "summarize".to_string(),
        arguments: args,
    };
    
    let response = server.get_prompt(request).await?;
    info!("  ✅ 成功获取提示词");
    info!("  内容:");
    for content in &response.content {
        if let PromptContent::Text { text } = content {
            info!("    {}", text);
        }
    }

    Ok(())
}

