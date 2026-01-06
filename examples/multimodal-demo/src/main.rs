//! 多模态内容处理演示
//!
//! 展示 AgentMem 的多模态内容处理能力和 LiteLLM 集成

use agent_mem_llm::providers::litellm::{LiteLLMMessage, LiteLLMProvider};
use agent_mem_llm::LLMFactory;
use agent_mem_traits::LLMConfig;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🎯 AgentMem 多模态内容处理演示");
    println!("{}", "=".repeat(50));

    // 演示 1: LiteLLM 多模态支持
    println!("\n📋 演示 1: LiteLLM 多模态支持");
    demo_litellm_multimodal().await?;

    // 演示 2: 内容类型识别
    println!("\n📋 演示 2: 内容类型识别");
    demo_content_type_detection();

    // 演示 3: 多模态处理流程
    println!("\n📋 演示 3: 多模态处理流程");
    demo_multimodal_processing();

    // 演示 4: LiteLLM 与多模态的结合
    println!("\n📋 演示 4: LiteLLM 与多模态的结合");
    demo_litellm_multimodal_integration().await?;

    println!("\n✅ 演示完成！");
    println!("\n💡 AgentMem 多模态特性:");
    println!("   - 统一的多模态内容处理接口");
    println!("   - LiteLLM 集成支持多种 LLM 提供商");
    println!("   - 智能内容类型检测和处理");
    println!("   - 可扩展的处理器架构");

    Ok(())
}

/// 演示 LiteLLM 多模态支持
async fn demo_litellm_multimodal() -> anyhow::Result<()> {
    println!("   🔧 创建支持多模态的 LiteLLM 提供商...");

    // 创建支持视觉的模型
    let provider = LiteLLMProvider::with_model("gpt-4-vision-preview")?;

    println!("   ✅ 提供商创建成功");
    println!("   📊 模型信息:");
    println!("      - 模型: {}", provider.get_model());
    println!("      - 支持多模态: 是");
    println!("      - 支持图像理解: 是");

    // 准备多模态消息
    let messages = vec![
        LiteLLMMessage {
            role: "system".to_string(),
            content: "你是一个专业的图像分析助手，能够理解和描述图像内容。".to_string(),
        },
        LiteLLMMessage {
            role: "user".to_string(),
            content: "请分析这张图片的内容。[图片: 一个现代办公室场景]".to_string(),
        },
    ];

    // 检查是否有 API 密钥
    if let Ok(api_key) = env::var("OPENAI_API_KEY") {
        println!("   🔑 检测到 OpenAI API 密钥，尝试多模态调用...");

        let provider_with_key = provider.with_api_key(api_key);

        match provider_with_key.generate_response(&messages).await {
            Ok(response) => {
                println!("   ✅ 多模态 LLM 响应成功:");
                println!("      {}", response.chars().take(150).collect::<String>());
                if response.len() > 150 {
                    println!("      ...(响应已截断)");
                }
            }
            Err(e) => {
                println!("   ⚠️  多模态调用失败: {e}");
                println!("      这可能是由于模型不支持或 API 配额限制");
            }
        }
    } else {
        println!("   📝 未检测到 OPENAI_API_KEY，跳过实际调用");
        println!("   💡 设置环境变量以测试实际多模态功能");
    }

    Ok(())
}

/// 演示内容类型识别
fn demo_content_type_detection() {
    println!("   🔍 内容类型自动识别:");

    let test_cases = vec![
        ("image.jpg", "image/jpeg"),
        ("document.pdf", "application/pdf"),
        ("audio.mp3", "audio/mpeg"),
        ("video.mp4", "video/mp4"),
        ("text.txt", "text/plain"),
        ("data.json", "application/json"),
    ];

    for (filename, expected_mime) in test_cases {
        let detected_type = detect_content_type_from_filename(filename);
        println!("      📄 {filename} -> {detected_type} (预期: {expected_mime})");
    }

    println!("   ✅ 内容类型识别完成");
}

/// 演示多模态处理流程
fn demo_multimodal_processing() {
    println!("   ⚙️  多模态处理流程:");

    let processing_steps = vec![
        ("内容接收", "接收各种格式的输入内容"),
        ("类型检测", "自动识别内容类型和格式"),
        ("预处理", "标准化和清理内容数据"),
        ("特征提取", "提取关键特征和元数据"),
        ("内容分析", "执行专门的分析算法"),
        ("结果整合", "整合分析结果和元数据"),
        ("输出生成", "生成统一的处理结果"),
    ];

    for (i, (step, description)) in processing_steps.iter().enumerate() {
        println!("      {}. {}: {}", i + 1, step, description);
    }

    println!("   ✅ 处理流程展示完成");
}

/// 演示 LiteLLM 与多模态的结合
async fn demo_litellm_multimodal_integration() -> anyhow::Result<()> {
    println!("   🔗 LiteLLM 与多模态集成:");

    // 创建配置
    let config = LLMConfig {
        provider: "litellm".to_string(),
        model: "claude-3-sonnet-20240229".to_string(),
        api_key: env::var("ANTHROPIC_API_KEY").ok(),
        temperature: Some(0.7),
        max_tokens: Some(1000),
        ..Default::default()
    };

    // 通过工厂创建提供商
    match LLMFactory::create_provider(&config) {
        Ok(provider) => {
            println!("   ✅ 集成提供商创建成功");

            let model_info = provider.get_model_info();
            println!("   📊 集成模型信息:");
            println!("      - 提供商: {}", model_info.provider);
            println!("      - 模型: {}", model_info.model);
            println!("      - 最大 Token: {}", model_info.max_tokens);

            // 准备多模态分析消息
            let messages = vec![
                agent_mem_traits::Message {
                    role: agent_mem_traits::MessageRole::System,
                    content: "你是一个多模态内容分析专家。".to_string(),
                    timestamp: None,
                },
                agent_mem_traits::Message {
                    role: agent_mem_traits::MessageRole::User,
                    content: "请分析以下多模态内容的处理策略：图像识别、文本提取、音频转录。"
                        .to_string(),
                    timestamp: None,
                },
            ];

            if config.api_key.is_some() {
                println!("   🔑 尝试集成多模态分析...");

                match provider.generate(&messages).await {
                    Ok(response) => {
                        println!("   ✅ 集成分析成功:");
                        println!("      {}", response.chars().take(200).collect::<String>());
                        if response.len() > 200 {
                            println!("      ...(响应已截断)");
                        }
                    }
                    Err(e) => {
                        println!("   ⚠️  集成分析失败: {e}");
                    }
                }
            } else {
                println!("   📝 未设置 API 密钥，跳过实际调用");
            }
        }
        Err(e) => {
            println!("   ❌ 集成提供商创建失败: {e}");
        }
    }

    // 展示支持的多模态模型
    println!("   🎯 支持的多模态模型:");
    let multimodal_models = vec![
        ("GPT-4 Vision", "gpt-4-vision-preview", "OpenAI"),
        ("Claude 3 Opus", "claude-3-opus-20240229", "Anthropic"),
        ("Claude 3 Sonnet", "claude-3-sonnet-20240229", "Anthropic"),
        ("Gemini Pro Vision", "gemini-pro-vision", "Google"),
    ];

    for (name, model_id, provider_name) in multimodal_models {
        println!("      - {name}: {model_id} ({provider_name})");
    }

    Ok(())
}

/// 简化的内容类型检测
fn detect_content_type_from_filename(filename: &str) -> &'static str {
    let extension = filename.split('.').next_back().unwrap_or("");
    match extension.to_lowercase().as_str() {
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" => "image/*",
        "mp3" | "wav" | "flac" | "aac" | "ogg" => "audio/*",
        "mp4" | "avi" | "mov" | "wmv" | "mkv" => "video/*",
        "pdf" => "application/pdf",
        "doc" | "docx" => "application/msword",
        "txt" | "md" => "text/plain",
        "json" => "application/json",
        "xml" => "application/xml",
        "html" | "htm" => "text/html",
        _ => "application/octet-stream",
    }
}
