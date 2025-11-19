# 华为 MaaS 集成指南

## 概述

AgentMem 现已完全支持华为 ModelArts MaaS (Model as a Service) 平台，基于 LumosAI 架构实现。您可以通过简单的配置即可使用华为云的 AI 模型（如 DeepSeek V3.2）。

## 特性

✅ **完整的 LLM Provider 实现**
- 文本生成 (generate)
- 多轮对话 (generate_with_messages)
- 流式响应 (generate_stream)
- Function Calling 支持
- OpenAI 兼容的 API 格式

✅ **灵活的配置方式**
- 环境变量配置
- 代码配置
- 自动 Provider 选择

✅ **与 AgentBuilder 无缝集成**
- 支持所有 AgentBuilder 功能
- 多租户支持
- 工具调用支持

## 快速开始

### 1. 环境变量配置

设置华为 MaaS API Key：

```bash
# 必需：API Key
export MAAS_API_KEY="your_maas_api_key"

# 可选：模型名称（默认：deepseek-v3.2-exp）
export MAAS_MODEL="deepseek-v3.2-exp"

# 或使用带前缀的变量名
export HUAWEI_MAAS_API_KEY="your_maas_api_key"
export HUAWEI_MAAS_MODEL="deepseek-v3.2-exp"
```

### 2. 代码示例

#### 方法 1: 使用 AgentBuilder（推荐）

```rust
use lumosai_core::agent::AgentBuilder;
use lumosai_core::llm::providers;
use lumosai_core::Result;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // 从环境变量创建 MaaS provider
    let maas_provider = providers::huawei_maas_from_env()?;
    
    // 创建 Agent
    let agent = AgentBuilder::new()
        .name("my_assistant")
        .instructions("你是一个智能助手")
        .model(Arc::new(maas_provider))
        .temperature(0.7)
        .max_tokens(1000)
        .build()?;
    
    // 使用 Agent
    let response = agent.generate("你好", &Default::default()).await?;
    println!("响应: {}", response);
    
    Ok(())
}
```

#### 方法 2: 使用 auto_provider

```rust
use lumosai_core::agent::AgentBuilder;
use lumosai_core::llm::providers;
use std::sync::Arc;

// auto_provider 会自动检测可用的 LLM provider
// 包括华为 MaaS（如果设置了 MAAS_API_KEY）
let llm = providers::auto_provider()?;

let agent = AgentBuilder::new()
    .name("auto_assistant")
    .instructions("你是一个智能助手")
    .model(Arc::new(llm))
    .build()?;
```

#### 方法 3: 直接使用 LLM Provider

```rust
use lumosai_core::llm::{HuaweiMaasProvider, LlmProvider, LlmOptions};

// 从环境变量创建
let provider = HuaweiMaasProvider::from_env()?;

// 或手动创建
let provider = HuaweiMaasProvider::new(
    "your-api-key".to_string(),
    Some("deepseek-v3.2-exp".to_string())
);

// 生成文本
let options = LlmOptions::default()
    .with_temperature(0.7)
    .with_max_tokens(1000);

let response = provider.generate("你好", &options).await?;
println!("{}", response);
```

### 3. 流式响应

```rust
use futures::StreamExt;

let mut stream = agent.generate_stream("讲个故事", &LlmOptions::default()).await?;

while let Some(chunk) = stream.next().await {
    match chunk {
        Ok(text) => print!("{}", text),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### 4. 多轮对话

```rust
use lumosai_core::llm::{Message, Role};

let messages = vec![
    Message {
        role: Role::System,
        content: "你是一个专业顾问".to_string(),
        metadata: None,
        name: None,
    },
    Message {
        role: Role::User,
        content: "云计算的优势是什么？".to_string(),
        metadata: None,
        name: None,
    },
];

let response = agent.generate_with_messages(&messages, &LlmOptions::default()).await?;
```

## API 端点

华为 MaaS 使用以下 API 端点：

- **基础 URL**: `https://api.modelarts-maas.com`
- **聊天补全**: `POST /v2/chat/completions`
- **API 格式**: OpenAI 兼容

## 支持的模型

当前支持的模型：

- `deepseek-v3.2-exp` (默认)
- 其他华为 MaaS 平台支持的模型

## 完整示例

运行完整示例：

```bash
cd lumosai/lumosai_core
cargo run --example huawei_maas_agent
```

示例代码位于：`lumosai/lumosai_core/examples/huawei_maas_agent.rs`

## 与 AgentMem Memory 集成

AgentBuilder 自动支持记忆功能：

```rust
use lumosai_core::memory::MemoryConfig;

let agent = AgentBuilder::new()
    .name("memory_assistant")
    .instructions("你是一个有记忆的助手")
    .model(Arc::new(maas_provider))
    .memory_config(Some(MemoryConfig::default()))
    .build()?;

// Agent 会自动记住对话历史
```

## Function Calling 支持

华为 MaaS 支持 OpenAI 格式的 Function Calling：

```rust
use lumosai_core::llm::function_calling::{FunctionDefinition, ToolChoice};
use serde_json::json;

let functions = vec![
    FunctionDefinition {
        name: "get_weather".to_string(),
        description: Some("获取天气信息".to_string()),
        parameters: json!({
            "type": "object",
            "properties": {
                "city": {"type": "string", "description": "城市名称"}
            },
            "required": ["city"]
        }),
    },
];

let response = provider.generate_with_functions(
    &messages,
    &functions,
    &ToolChoice::Auto,
    &LlmOptions::default(),
).await?;

// 检查是否有函数调用
if !response.function_calls.is_empty() {
    for call in response.function_calls {
        println!("Function: {}", call.name);
        println!("Arguments: {}", call.arguments);
    }
}
```

## 配置选项

### LlmOptions

```rust
use lumosai_core::llm::LlmOptions;

let options = LlmOptions {
    model: Some("deepseek-v3.2-exp".to_string()),
    temperature: Some(0.7),      // 0.0-2.0
    max_tokens: Some(1000),      // 最大生成长度
    top_p: Some(1.0),            // Nucleus sampling
    frequency_penalty: Some(0.0), // 频率惩罚
    presence_penalty: Some(0.0),  // 存在惩罚
    stop: None,                   // 停止序列
    user: None,                   // 用户 ID
    seed: None,                   // 随机种子
};
```

### AgentBuilder 配置

```rust
let agent = AgentBuilder::new()
    .name("advanced_assistant")
    .instructions("你是一个高级助手")
    .model(Arc::new(maas_provider))
    .temperature(0.8)                    // 温度
    .max_tokens(2000)                    // 最大 tokens
    .max_tool_calls(5)                   // 最大工具调用次数
    .tool_timeout(60)                    // 工具超时（秒）
    .enable_smart_defaults()             // 启用智能默认值
    .build()?;
```

## 错误处理

```rust
match providers::huawei_maas_from_env() {
    Ok(provider) => {
        // 使用 provider
    }
    Err(e) => {
        match e {
            lumosai_core::Error::Configuration(msg) => {
                eprintln!("配置错误: {}", msg);
                eprintln!("请设置 MAAS_API_KEY 环境变量");
            }
            lumosai_core::Error::Llm(msg) => {
                eprintln!("LLM 错误: {}", msg);
            }
            _ => {
                eprintln!("其他错误: {}", e);
            }
        }
    }
}
```

## 性能优化

### 1. 使用流式响应

对于长文本生成，使用流式响应可以获得更好的用户体验：

```rust
let stream = agent.generate_stream(prompt, &options).await?;
```

### 2. 调整 max_tokens

根据实际需求设置合适的 max_tokens 以节省 API 费用：

```rust
let options = LlmOptions::default().with_max_tokens(500);
```

### 3. 批量请求

对于多个独立请求，可以使用并发处理：

```rust
use futures::future::join_all;

let tasks: Vec<_> = prompts.iter().map(|prompt| {
    agent.generate(prompt, &options)
}).collect();

let results = join_all(tasks).await;
```

## 限制和注意事项

1. **Embedding 支持**: 华为 MaaS 当前不支持 embedding API，如需 embedding 功能请使用 OpenAI 或其他 provider

2. **API 速率限制**: 请遵守华为 MaaS 的 API 调用限制

3. **区域限制**: 确保您的网络可以访问华为云 MaaS API 端点

4. **模型可用性**: 不同区域可能支持不同的模型，请查看华为云文档

## 故障排查

### 问题 1: "MAAS_API_KEY environment variable not set"

**解决方案**:
```bash
export MAAS_API_KEY="your_api_key"
```

### 问题 2: API 请求失败

检查：
1. API Key 是否正确
2. 网络连接是否正常
3. 模型名称是否正确
4. API 端点是否可访问

### 问题 3: 流式响应中断

可能原因：
- 网络不稳定
- API 限流
- 超时设置过短

## 架构说明

华为 MaaS 集成基于 LumosAI 的架构：

```
AgentMem (应用层)
    ↓
LumosAI AgentBuilder (Agent 构建)
    ↓
LumosAI LLM Provider (统一接口)
    ↓
HuaweiMaasProvider (华为 MaaS 实现)
    ↓
华为 MaaS API
```

**优势**：
- ✅ 最小改造：复用现有 LumosAI 架构
- ✅ 统一接口：与其他 LLM provider 一致
- ✅ 易于维护：代码独立在单个文件中
- ✅ 可扩展：支持所有 AgentBuilder 特性

## 相关文档

- [LumosAI LLM 文档](../lumosai/lumosai_core/src/llm/README.md)
- [AgentBuilder 文档](../lumosai/lumosai_core/src/agent/README.md)
- [华为 MaaS API 文档](https://support.huaweicloud.com/modelarts/)

## 更新日志

### v1.0.0 (2025-11-19)
- ✅ 初始版本
- ✅ 基础文本生成
- ✅ 流式响应
- ✅ Function Calling
- ✅ 多轮对话
- ✅ AgentBuilder 集成
- ✅ auto_provider 支持

## 联系方式

如有问题或建议，请提交 Issue 或 PR。
