//! LiteLLM 集成演示
//!
//! 展示 AgentMem 的 LiteLLM 统一接口功能，支持多种 LLM 提供商

use agent_mem_llm::providers::litellm::{LiteLLMMessage, LiteLLMProvider, SupportedModel};
use agent_mem_llm::LLMFactory;
use agent_mem_traits::LLMConfig;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🚀 AgentMem LiteLLM 集成演示");
    println!("{}", "=".repeat(50));

    // 演示 1: 直接使用 LiteLLM 提供商
    println!("\n📋 演示 1: 直接使用 LiteLLM 提供商");
    demo_direct_litellm().await?;

    // 演示 2: 通过工厂模式使用 LiteLLM
    println!("\n📋 演示 2: 通过工厂模式使用 LiteLLM");
    demo_factory_litellm().await?;

    // 演示 3: 支持的模型展示
    println!("\n📋 演示 3: 支持的模型展示");
    demo_supported_models();

    // 演示 4: 配置选项展示
    println!("\n📋 演示 4: 配置选项展示");
    demo_configuration_options();

    println!("\n✅ 演示完成！");
    println!("\n💡 提示:");
    println!("   - 设置相应的 API 密钥环境变量以测试实际 LLM 调用");
    println!("   - 支持的环境变量: OPENAI_API_KEY, ANTHROPIC_API_KEY, DEEPSEEK_API_KEY 等");
    println!("   - LiteLLM 提供统一接口，简化多 LLM 提供商集成");

    Ok(())
}

/// 演示直接使用 LiteLLM 提供商
async fn demo_direct_litellm() -> anyhow::Result<()> {
    println!("   🔧 创建 LiteLLM 提供商...");

    // 创建 LiteLLM 提供商
    let provider = LiteLLMProvider::with_model("gpt-3.5-turbo")?;

    println!("   ✅ 提供商创建成功");
    println!("   📊 模型信息:");
    println!("      - 模型: {}", provider.get_model());
    println!("      - 最大 Token: {:?}", provider.get_max_tokens());

    // 准备测试消息
    let messages = vec![
        LiteLLMMessage {
            role: "system".to_string(),
            content: "你是一个有用的AI助手。".to_string(),
        },
        LiteLLMMessage {
            role: "user".to_string(),
            content: "请简单介绍一下人工智能。".to_string(),
        },
    ];

    // 检查是否有 API 密钥
    if let Ok(api_key) = env::var("OPENAI_API_KEY") {
        println!("   🔑 检测到 OpenAI API 密钥，尝试实际调用...");

        let provider_with_key = provider.with_api_key(api_key);

        match provider_with_key.generate_response(&messages).await {
            Ok(response) => {
                println!("   ✅ LLM 响应成功:");
                println!("      {}", response.chars().take(100).collect::<String>());
                if response.len() > 100 {
                    println!("      ...(响应已截断)");
                }
            }
            Err(e) => {
                println!("   ⚠️  LLM 调用失败: {e}");
                println!("      这可能是由于网络问题或 API 配额限制");
            }
        }
    } else {
        println!("   📝 未检测到 OPENAI_API_KEY，跳过实际 LLM 调用");
        println!("   💡 设置环境变量 OPENAI_API_KEY 以测试实际调用");
    }

    Ok(())
}

/// 演示通过工厂模式使用 LiteLLM
async fn demo_factory_litellm() -> anyhow::Result<()> {
    println!("   🏭 通过工厂模式创建 LiteLLM 提供商...");

    // 创建 LLM 配置
    let config = LLMConfig {
        provider: "litellm".to_string(),
        model: "gpt-4".to_string(),
        api_key: env::var("OPENAI_API_KEY").ok(),
        temperature: Some(0.7),
        max_tokens: Some(1000),
        ..Default::default()
    };

    // 通过工厂创建提供商
    match LLMFactory::create_provider(&config) {
        Ok(provider) => {
            println!("   ✅ 工厂创建成功");

            let model_info = provider.get_model_info();
            println!("   📊 模型信息:");
            println!("      - 提供商: {}", model_info.provider);
            println!("      - 模型: {}", model_info.model);
            println!("      - 最大 Token: {}", model_info.max_tokens);
            println!("      - 支持流式: {}", model_info.supports_streaming);
            println!("      - 支持函数: {}", model_info.supports_functions);

            // 准备测试消息
            let messages = vec![
                agent_mem_traits::Message {
                    role: agent_mem_traits::MessageRole::System,
                    content: "你是一个专业的技术顾问。".to_string(),
                    timestamp: None,
                },
                agent_mem_traits::Message {
                    role: agent_mem_traits::MessageRole::User,
                    content: "请解释什么是 LiteLLM？".to_string(),
                    timestamp: None,
                },
            ];

            if config.api_key.is_some() {
                println!("   🔑 尝试通过工厂接口调用 LLM...");

                match provider.generate(&messages).await {
                    Ok(response) => {
                        println!("   ✅ 工厂接口调用成功:");
                        println!("      {}", response.chars().take(100).collect::<String>());
                        if response.len() > 100 {
                            println!("      ...(响应已截断)");
                        }
                    }
                    Err(e) => {
                        println!("   ⚠️  工厂接口调用失败: {e}");
                    }
                }
            } else {
                println!("   📝 未设置 API 密钥，跳过实际调用");
            }
        }
        Err(e) => {
            println!("   ❌ 工厂创建失败: {e}");
        }
    }

    Ok(())
}

/// 演示支持的模型
fn demo_supported_models() {
    println!("   📚 LiteLLM 支持的模型:");

    let models = vec![
        (
            "OpenAI",
            vec![
                SupportedModel::GPT4,
                SupportedModel::GPT4Turbo,
                SupportedModel::GPT35Turbo,
            ],
        ),
        (
            "Anthropic",
            vec![
                SupportedModel::Claude3Opus,
                SupportedModel::Claude3Sonnet,
                SupportedModel::Claude3Haiku,
            ],
        ),
        (
            "AWS Bedrock",
            vec![SupportedModel::BedrockClaude, SupportedModel::BedrockTitan],
        ),
        (
            "Azure OpenAI",
            vec![SupportedModel::AzureGPT4, SupportedModel::AzureGPT35],
        ),
        (
            "Google",
            vec![SupportedModel::Gemini15Pro, SupportedModel::Gemini15Flash],
        ),
        (
            "其他",
            vec![
                SupportedModel::Groq,
                SupportedModel::Together,
                SupportedModel::Ollama,
            ],
        ),
    ];

    for (provider, provider_models) in models {
        println!("      🏢 {provider}:");
        for model in provider_models {
            println!("         - {}", model.as_str());
        }
    }

    println!("   💡 使用方法:");
    println!("      let provider = LiteLLMProvider::with_model(\"gpt-4\")?;");
    println!("      let provider = LiteLLMProvider::with_model(\"claude-3-sonnet-20240229\")?;");
}

/// 演示配置选项
fn demo_configuration_options() {
    println!("   ⚙️  LiteLLM 配置选项:");

    println!("      🔧 基础配置:");
    println!("         - model: 模型名称");
    println!("         - api_key: API 密钥");
    println!("         - api_base: 自定义 API 基础 URL");
    println!("         - temperature: 温度参数 (0.0-2.0)");
    println!("         - max_tokens: 最大 token 数");

    println!("      🔄 重试配置:");
    println!("         - max_retries: 最大重试次数");
    println!("         - backoff_factor: 退避因子");
    println!("         - max_backoff: 最大退避时间");

    println!("      🚦 速率限制:");
    println!("         - requests_per_minute: 每分钟请求数");
    println!("         - tokens_per_minute: 每分钟 token 数");
    println!("         - concurrent_requests: 并发请求数");

    println!("   💡 配置示例:");
    println!("      ```rust");
    println!("      let config = LiteLLMConfig {{");
    println!("          model: \"gpt-4\".to_string(),");
    println!("          api_key: Some(\"your-api-key\".to_string()),");
    println!("          temperature: Some(0.7),");
    println!("          max_tokens: Some(2000),");
    println!("          ..Default::default()");
    println!("      }};");
    println!("      ```");
}
