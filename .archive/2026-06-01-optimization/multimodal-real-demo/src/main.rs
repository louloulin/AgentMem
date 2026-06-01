//! 真实多模态处理演示程序
//!
//! 演示真实的图像和音频处理功能

use agent_mem_intelligence::multimodal::{
    real_audio::{RealAudioProcessor, RealAudioProcessorConfig},
    real_image::{RealImageProcessor, RealImageProcessorConfig},
    ContentType, MultimodalContent, MultimodalProcessor,
};
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter("multimodal_real_demo=info,agent_mem_intelligence=info")
        .init();

    info!("🚀 开始真实多模态处理演示");

    // 测试真实图像处理
    test_real_image_processing().await?;

    // 测试真实音频处理
    test_real_audio_processing().await?;

    info!("✅ 真实多模态处理演示完成");
    Ok(())
}

/// 测试真实图像处理
async fn test_real_image_processing() -> Result<(), Box<dyn std::error::Error>> {
    info!("🖼️  开始测试真实图像处理");

    // 创建真实图像处理器配置
    let config = RealImageProcessorConfig {
        enable_ocr: true,
        enable_object_detection: true,
        enable_scene_analysis: true,
        openai_api_key: std::env::var("OPENAI_API_KEY").ok(),
        google_vision_api_key: std::env::var("GOOGLE_VISION_API_KEY").ok(),
        tesseract_path: None,
        use_local_models: true,
    };

    let processor = RealImageProcessor::new(config);

    // 测试不同类型的图像
    let test_images = vec![
        ("screenshot_ui.png", "screenshot", "image/png"),
        ("document_scan.jpg", "document", "image/jpeg"),
        ("chart_data.png", "chart", "image/png"),
        ("photo_nature.jpg", "photo", "image/jpeg"),
    ];

    for (filename, content_type, mime_type) in test_images {
        info!("处理图像: {}", filename);

        // 创建模拟图像内容
        let mut content = create_mock_image_content(filename, content_type, mime_type);

        // 处理图像
        match processor.process(&mut content).await {
            Ok(()) => {
                info!("✅ 图像处理成功: {}", filename);

                if let Some(extracted_text) = &content.extracted_text {
                    info!("📝 提取的文本: {}", extracted_text);
                }

                // 生成摘要
                if let Ok(Some(summary)) = processor.generate_summary(&content).await {
                    info!("📋 图像摘要: {}", summary);
                }
            }
            Err(e) => {
                error!("❌ 图像处理失败: {} - {}", filename, e);
            }
        }

        println!(); // 空行分隔
    }

    Ok(())
}

/// 测试真实音频处理
async fn test_real_audio_processing() -> Result<(), Box<dyn std::error::Error>> {
    info!("🎵 开始测试真实音频处理");

    // 创建真实音频处理器配置
    let config = RealAudioProcessorConfig {
        enable_speech_to_text: true,
        enable_audio_analysis: true,
        openai_api_key: std::env::var("OPENAI_API_KEY").ok(),
        google_speech_api_key: std::env::var("GOOGLE_SPEECH_API_KEY").ok(),
        azure_speech_api_key: std::env::var("AZURE_SPEECH_API_KEY").ok(),
        azure_speech_region: std::env::var("AZURE_SPEECH_REGION").ok(),
        use_local_whisper: true,
        whisper_model_size: "base".to_string(),
    };

    let processor = RealAudioProcessor::new(config);

    // 测试不同类型的音频
    let test_audios = vec![
        ("speech_recording.wav", "speech", "audio/wav"),
        ("meeting_call.mp3", "meeting", "audio/mp3"),
        ("interview_session.wav", "interview", "audio/wav"),
        ("music_song.mp3", "music", "audio/mp3"),
    ];

    for (filename, content_type, mime_type) in test_audios {
        info!("处理音频: {}", filename);

        // 创建模拟音频内容
        let mut content = create_mock_audio_content(filename, content_type, mime_type);

        // 处理音频
        match processor.process(&mut content).await {
            Ok(()) => {
                info!("✅ 音频处理成功: {}", filename);

                if let Some(extracted_text) = &content.extracted_text {
                    info!("🗣️  转录文本: {}", extracted_text);
                }

                // 生成摘要
                if let Ok(Some(summary)) = processor.generate_summary(&content).await {
                    info!("📋 音频摘要: {}", summary);
                }
            }
            Err(e) => {
                error!("❌ 音频处理失败: {} - {}", filename, e);
            }
        }

        println!(); // 空行分隔
    }

    Ok(())
}

/// 创建模拟图像内容
fn create_mock_image_content(
    filename: &str,
    content_type: &str,
    mime_type: &str,
) -> MultimodalContent {
    let mut content = MultimodalContent::new(uuid::Uuid::new_v4().to_string(), ContentType::Image);

    // 设置基本信息
    content.mime_type = Some(mime_type.to_string());
    content.size = Some(generate_mock_file_size(content_type));

    // 设置元数据
    content.set_metadata(
        "filename".to_string(),
        serde_json::Value::String(filename.to_string()),
    );
    content.set_metadata(
        "content_type".to_string(),
        serde_json::Value::String(content_type.to_string()),
    );

    // 模拟 Base64 数据（实际应用中这里是真实的图像数据）
    let mock_data = format!("mock_image_data_for_{}", filename);
    content.data = Some(base64::encode(mock_data.as_bytes()));

    content
}

/// 创建模拟音频内容
fn create_mock_audio_content(
    filename: &str,
    content_type: &str,
    mime_type: &str,
) -> MultimodalContent {
    let mut content = MultimodalContent::new(uuid::Uuid::new_v4().to_string(), ContentType::Audio);

    // 设置基本信息
    content.mime_type = Some(mime_type.to_string());
    content.size = Some(generate_mock_file_size(content_type));

    // 设置元数据
    content.set_metadata(
        "filename".to_string(),
        serde_json::Value::String(filename.to_string()),
    );
    content.set_metadata(
        "content_type".to_string(),
        serde_json::Value::String(content_type.to_string()),
    );

    // 模拟 Base64 数据（实际应用中这里是真实的音频数据）
    let mock_data = format!("mock_audio_data_for_{}", filename);
    content.data = Some(base64::encode(mock_data.as_bytes()));

    content
}

/// 生成模拟文件大小
fn generate_mock_file_size(content_type: &str) -> u64 {
    match content_type {
        "screenshot" => 150_000,  // 150KB
        "document" => 500_000,    // 500KB
        "chart" => 80_000,        // 80KB
        "photo" => 2_000_000,     // 2MB
        "speech" => 1_200_000,    // 1.2MB
        "meeting" => 5_000_000,   // 5MB
        "interview" => 3_000_000, // 3MB
        "music" => 8_000_000,     // 8MB
        _ => 1_000_000,           // 1MB 默认
    }
}

/// Base64 编码函数（简化版）
mod base64 {
    pub fn encode(data: &[u8]) -> String {
        // 简化的 Base64 编码实现
        format!("base64_encoded_{}_bytes", data.len())
    }
}
