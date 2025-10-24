//! AgentMem 6.0 LLM 提供商真实化演示程序
//!
//! 这个演示程序展示了 AgentMem 6.0 中 LLM 提供商的真实实现，
//! 包括本地测试提供商和真实 API 提供商的功能验证。

use agent_mem_llm::providers::{LocalTestProvider, OllamaProvider};
use agent_mem_traits::{LLMConfig, LLMProvider, Message, MessageRole};
use chrono::Utc;
use std::sync::Arc;
use std::time::Instant;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter("llm_provider_demo=info,agent_mem_llm=debug")
        .init();

    info!("🚀 AgentMem 6.0 LLM 提供商真实化演示");
    info!("📋 Phase 1.3: LLM 提供商 Mock 清理和真实实现验证");

    // 演示 1: 本地测试 LLM 提供商
    demo_local_test_provider().await?;

    // 演示 2: Ollama 本地 LLM 提供商（如果可用）
    demo_ollama_provider().await?;

    // 演示 3: LLM 提供商性能测试
    demo_performance_testing().await?;

    // 演示 4: 多轮对话测试
    demo_multi_turn_conversation().await?;

    // 演示 5: 错误处理和恢复
    demo_error_handling().await?;

    info!("✅ AgentMem 6.0 LLM 提供商真实化演示完成");
    Ok(())
}

/// 演示本地测试 LLM 提供商
async fn demo_local_test_provider() -> Result<(), Box<dyn std::error::Error>> {
    info!("📝 演示 1: 本地测试 LLM 提供商功能验证");

    let config = LLMConfig::default();
    let provider = LocalTestProvider::new(config)?;

    // 测试模型信息获取
    let model_info = provider.get_model_info();
    info!(
        "   模型信息: {} ({})",
        model_info.model, model_info.provider
    );
    info!("   最大 tokens: {}", model_info.max_tokens);
    info!("   支持函数调用: {}", model_info.supports_functions);

    // 测试健康检查
    let is_healthy = provider.health_check().await?;
    info!(
        "   健康状态: {}",
        if is_healthy {
            "✅ 正常"
        } else {
            "❌ 异常"
        }
    );

    // 测试基本对话
    let messages = vec![
        Message {
            role: MessageRole::System,
            content: "你是一个有用的AI助手".to_string(),
            timestamp: Some(Utc::now()),
        },
        Message {
            role: MessageRole::User,
            content: "你好，请介绍一下你自己".to_string(),
            timestamp: Some(Utc::now()),
        },
    ];

    let start_time = Instant::now();
    let response = provider.generate(&messages).await?;
    let duration = start_time.elapsed();

    info!("   响应时间: {:?}", duration);
    info!("   响应内容: {}", response);

    // 测试带元数据的生成
    let (response_with_meta, metadata) = provider.generate_with_metadata(&messages).await?;
    info!("   元数据响应长度: {}", response_with_meta.len());

    if let Some(usage) = metadata.get("usage") {
        info!(
            "   Token 使用情况: {}",
            serde_json::to_string_pretty(usage)?
        );
    }

    info!("✅ 本地测试 LLM 提供商测试完成");
    Ok(())
}

/// 演示 Ollama 本地 LLM 提供商
async fn demo_ollama_provider() -> Result<(), Box<dyn std::error::Error>> {
    info!("🦙 演示 2: Ollama 本地 LLM 提供商连接测试");

    let mut config = LLMConfig::default();
    config.base_url = Some("http://localhost:11434".to_string());
    config.model = "llama2".to_string();

    match OllamaProvider::new(config) {
        Ok(provider) => {
            info!("   Ollama 提供商创建成功");

            // 测试基本功能（Ollama 没有 health_check 方法）
            info!("   Ollama 提供商创建成功，尝试基本对话...");

            // 尝试简单对话
            let messages = vec![Message {
                role: MessageRole::User,
                content: "Hello, can you respond briefly?".to_string(),
                timestamp: Some(Utc::now()),
            }];

            match provider.generate(&messages).await {
                Ok(response) => {
                    info!("   Ollama 响应: {}", response);
                }
                Err(e) => {
                    warn!("   Ollama 生成响应失败: {}", e);
                }
            }
        }
        Err(e) => {
            warn!("⚠️ Ollama 连接失败（预期行为）: {}", e);
            info!("💡 这是正常的，因为演示环境可能没有运行 Ollama");
        }
    }

    info!("✅ Ollama 提供商测试完成");
    Ok(())
}

/// 演示性能测试
async fn demo_performance_testing() -> Result<(), Box<dyn std::error::Error>> {
    info!("⚡ 演示 3: LLM 提供商性能测试");

    let config = LLMConfig::default();
    let provider = Arc::new(LocalTestProvider::new(config)?);

    // 并发请求测试
    let concurrent_requests = 10;
    let mut handles = Vec::new();

    let start_time = Instant::now();

    for i in 0..concurrent_requests {
        let provider_clone = Arc::clone(&provider);
        let handle = tokio::spawn(async move {
            let messages = vec![Message {
                role: MessageRole::User,
                content: format!("这是第 {} 个并发请求", i + 1),
                timestamp: Some(Utc::now()),
            }];

            let start = Instant::now();
            let result = provider_clone.generate(&messages).await;
            let duration = start.elapsed();

            (i, result, duration)
        });
        handles.push(handle);
    }

    let mut total_duration = std::time::Duration::ZERO;
    let mut successful_requests = 0;

    for handle in handles {
        let (request_id, result, duration) = handle.await?;
        total_duration += duration;

        match result {
            Ok(_) => {
                successful_requests += 1;
                info!("   请求 {} 完成，耗时: {:?}", request_id + 1, duration);
            }
            Err(e) => {
                warn!("   请求 {} 失败: {}", request_id + 1, e);
            }
        }
    }

    let total_time = start_time.elapsed();
    let avg_duration = total_duration / concurrent_requests;
    let requests_per_second = concurrent_requests as f64 / total_time.as_secs_f64();

    info!("   并发请求数: {}", concurrent_requests);
    info!("   成功请求数: {}", successful_requests);
    info!("   总耗时: {:?}", total_time);
    info!("   平均单请求耗时: {:?}", avg_duration);
    info!("   请求速率: {:.2} 请求/秒", requests_per_second);

    info!("✅ 性能测试完成");
    Ok(())
}

/// 演示多轮对话
async fn demo_multi_turn_conversation() -> Result<(), Box<dyn std::error::Error>> {
    info!("💬 演示 4: 多轮对话测试");

    let config = LLMConfig::default();
    let provider = LocalTestProvider::new(config)?;

    let mut conversation = Vec::new();

    // 系统消息
    conversation.push(Message {
        role: MessageRole::System,
        content: "你是一个专业的AI助手，擅长回答技术问题".to_string(),
        timestamp: Some(Utc::now()),
    });

    // 第一轮对话
    conversation.push(Message {
        role: MessageRole::User,
        content: "什么是 AgentMem？".to_string(),
        timestamp: Some(Utc::now()),
    });

    let response1 = provider.generate(&conversation).await?;
    info!("   用户: 什么是 AgentMem？");
    info!("   助手: {}", response1);

    conversation.push(Message {
        role: MessageRole::Assistant,
        content: response1,
        timestamp: Some(Utc::now()),
    });

    // 第二轮对话
    conversation.push(Message {
        role: MessageRole::User,
        content: "它有什么特点？".to_string(),
        timestamp: Some(Utc::now()),
    });

    let response2 = provider.generate(&conversation).await?;
    info!("   用户: 它有什么特点？");
    info!("   助手: {}", response2);

    conversation.push(Message {
        role: MessageRole::Assistant,
        content: response2,
        timestamp: Some(Utc::now()),
    });

    // 第三轮对话
    conversation.push(Message {
        role: MessageRole::User,
        content: "请总结一下我们的对话".to_string(),
        timestamp: Some(Utc::now()),
    });

    let (response3, metadata) = provider.generate_with_metadata(&conversation).await?;
    info!("   用户: 请总结一下我们的对话");
    info!("   助手: {}", response3);

    if let Some(usage) = metadata.get("usage") {
        info!(
            "   对话 Token 统计: {}",
            serde_json::to_string_pretty(usage)?
        );
    }

    info!("✅ 多轮对话测试完成");
    Ok(())
}

/// 演示错误处理
async fn demo_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    info!("🛠️ 演示 5: 错误处理和恢复机制");

    let config = LLMConfig::default();
    let provider = LocalTestProvider::new(config)?;

    // 测试空消息列表
    info!("   测试空消息列表处理...");
    match provider.generate(&[]).await {
        Ok(_) => warn!("   预期应该失败，但成功了"),
        Err(e) => info!("   ✅ 正确处理空消息错误: {}", e),
    }

    // 测试超长消息
    info!("   测试超长消息处理...");
    let long_message = "很长的消息内容 ".repeat(1000);
    let messages = vec![Message {
        role: MessageRole::User,
        content: long_message,
        timestamp: Some(Utc::now()),
    }];

    match provider.generate(&messages).await {
        Ok(response) => {
            info!("   ✅ 超长消息处理成功，响应长度: {}", response.len());
        }
        Err(e) => {
            warn!("   超长消息处理失败: {}", e);
        }
    }

    // 测试重试机制（模拟）
    info!("   测试重试机制...");
    let mut retry_count = 0;
    let max_retries = 3;

    loop {
        match provider.health_check().await {
            Ok(true) => {
                info!("   ✅ 健康检查成功");
                break;
            }
            Ok(false) => {
                retry_count += 1;
                if retry_count >= max_retries {
                    warn!("   ❌ 重试次数超限");
                    break;
                }
                info!("   重试 {}/{}", retry_count, max_retries);
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
            Err(e) => {
                warn!("   健康检查错误: {}", e);
                break;
            }
        }
    }

    info!("✅ 错误处理测试完成");
    Ok(())
}
