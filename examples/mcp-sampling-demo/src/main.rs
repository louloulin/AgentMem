//! MCP Sampling 功能演示
//!
//! 演示如何使用 MCP Sampling 功能进行 LLM 采样

use agent_mem_llm::LLMConfig;
use agent_mem_tools::mcp::{
    SamplingManager, SamplingMessage, SamplingParams,
    CreateMessageRequest, StopReason,
};
use tracing::Level;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    println!("=== MCP Sampling 功能演示 ===\n");

    // 演示 1: 禁用状态的 Sampling 管理器
    demo_disabled_sampling().await?;

    // 演示 2: 采样参数配置
    demo_sampling_params().await?;

    // 演示 3: 消息创建
    demo_message_creation().await?;

    // 演示 4: 模拟 LLM 采样（使用 Mock 配置）
    demo_mock_sampling().await?;

    println!("\n=== 演示完成 ===");
    Ok(())
}

/// 演示 1: 禁用状态的 Sampling 管理器
async fn demo_disabled_sampling() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- 演示 1: 禁用状态的 Sampling 管理器 ---");

    let manager = SamplingManager::disabled();
    println!("✓ 创建禁用的 Sampling 管理器");
    println!("  启用状态: {}", manager.is_enabled());

    // 尝试创建消息（应该失败）
    let request = CreateMessageRequest {
        messages: vec![SamplingMessage::user("Hello, world!")],
        params: SamplingParams::default(),
        model: None,
        system_prompt: None,
    };

    match manager.create_message(request).await {
        Ok(_) => println!("  ✗ 意外成功"),
        Err(e) => println!("  ✓ 预期失败: {e}"),
    }

    println!();
    Ok(())
}

/// 演示 2: 采样参数配置
async fn demo_sampling_params() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- 演示 2: 采样参数配置 ---");

    // 默认参数
    let default_params = SamplingParams::default();
    println!("✓ 默认采样参数:");
    println!("  Temperature: {:?}", default_params.temperature);
    println!("  Top-p: {:?}", default_params.top_p);
    println!("  Max tokens: {:?}", default_params.max_tokens);
    println!("  Stream: {}", default_params.stream);

    // 自定义参数
    let custom_params = SamplingParams::new()
        .with_temperature(0.5)
        .with_top_p(0.95)
        .with_max_tokens(1024)
        .with_stream(true)
        .with_stop_sequences(vec!["END".to_string(), "STOP".to_string()]);

    println!("\n✓ 自定义采样参数:");
    println!("  Temperature: {:?}", custom_params.temperature);
    println!("  Top-p: {:?}", custom_params.top_p);
    println!("  Max tokens: {:?}", custom_params.max_tokens);
    println!("  Stream: {}", custom_params.stream);
    println!("  Stop sequences: {:?}", custom_params.stop_sequences);

    // 序列化为 JSON
    let json = serde_json::to_string_pretty(&custom_params)?;
    println!("\n✓ JSON 表示:");
    println!("{json}");

    println!();
    Ok(())
}

/// 演示 3: 消息创建
async fn demo_message_creation() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- 演示 3: 消息创建 ---");

    // 创建不同角色的消息
    let system_msg = SamplingMessage::system("You are a helpful assistant.");
    println!("✓ 系统消息:");
    println!("  角色: {:?}", system_msg.role);
    println!("  内容: {}", system_msg.content);

    let user_msg = SamplingMessage::user("What is the capital of France?");
    println!("\n✓ 用户消息:");
    println!("  角色: {:?}", user_msg.role);
    println!("  内容: {}", user_msg.content);

    let assistant_msg = SamplingMessage::assistant("The capital of France is Paris.");
    println!("\n✓ 助手消息:");
    println!("  角色: {:?}", assistant_msg.role);
    println!("  内容: {}", assistant_msg.content);

    // 创建完整的对话请求
    let request = CreateMessageRequest {
        messages: vec![
            SamplingMessage::user("What is 2 + 2?"),
        ],
        params: SamplingParams::new()
            .with_temperature(0.3)
            .with_max_tokens(100),
        model: Some("gpt-4".to_string()),
        system_prompt: Some("You are a math tutor.".to_string()),
    };

    println!("\n✓ 完整的采样请求:");
    println!("  消息数量: {}", request.messages.len());
    println!("  模型: {:?}", request.model);
    println!("  系统提示词: {:?}", request.system_prompt);
    println!("  Temperature: {:?}", request.params.temperature);

    println!();
    Ok(())
}

/// 演示 4: 模拟 LLM 采样
async fn demo_mock_sampling() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- 演示 4: 模拟 LLM 采样 ---");

    // 注意：这里使用 Mock 配置，因为我们没有真实的 LLM API 密钥
    // 在实际使用中，需要配置真实的 LLM 提供商
    
    println!("✓ 创建 Mock LLM 配置");
    let llm_config = LLMConfig {
        provider: "mock".to_string(),
        model: "mock-model".to_string(),
        api_key: Some("mock-key".to_string()),
        base_url: None,
        temperature: Some(0.7),
        max_tokens: Some(2048),
        top_p: Some(0.9),
        frequency_penalty: None,
        presence_penalty: None,
        response_format: None,
    };

    // 创建 Sampling 管理器
    match SamplingManager::new(Some(&llm_config)) {
        Ok(manager) => {
            println!("✓ Sampling 管理器创建成功");
            println!("  启用状态: {}", manager.is_enabled());
            println!("  默认参数: {:?}", manager.default_params());
            
            // 注意：由于使用 Mock 配置，实际调用会失败
            // 这里只是演示 API 的使用方式
            println!("\n✓ 采样请求示例:");
            let request = CreateMessageRequest {
                messages: vec![
                    SamplingMessage::user("Explain quantum computing in simple terms."),
                ],
                params: SamplingParams::new()
                    .with_temperature(0.7)
                    .with_max_tokens(500),
                model: Some("mock-model".to_string()),
                system_prompt: Some("You are a science educator.".to_string()),
            };
            
            println!("  消息: {:?}", request.messages[0].content);
            println!("  参数: temperature={:?}, max_tokens={:?}", 
                request.params.temperature, request.params.max_tokens);
            
            // 实际调用会失败，因为 Mock 提供商不存在
            match manager.create_message(request).await {
                Ok(response) => {
                    println!("\n✓ 采样响应:");
                    println!("  消息: {}", response.message.content);
                    println!("  停止原因: {:?}", response.stop_reason);
                    println!("  令牌使用: input={}, output={}, total={}",
                        response.usage.input_tokens,
                        response.usage.output_tokens,
                        response.usage.total_tokens);
                    println!("  模型: {}", response.model);
                    println!("  创建时间: {}", response.created_at);
                }
                Err(e) => {
                    println!("\n  ⚠ 采样失败（预期，因为使用 Mock 配置）: {e}");
                }
            }
        }
        Err(e) => {
            println!("  ⚠ Sampling 管理器创建失败（预期，因为 Mock 提供商不存在）: {e}");
        }
    }

    println!("\n✓ 停止原因类型:");
    println!("  MaxTokens: {:?}", StopReason::MaxTokens);
    println!("  StopSequence: {:?}", StopReason::StopSequence);
    println!("  EndTurn: {:?}", StopReason::EndTurn);
    println!("  Other: {:?}", StopReason::Other);

    println!();
    Ok(())
}

