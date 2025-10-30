//! 多模态 AI 功能演示
//!
//! 演示如何使用 OpenAI Vision 和 Whisper API 进行多模态内容处理

use agent_mem_intelligence::multimodal::{
    AIModelConfig, AudioFormat, AudioTranscriptionRequest, DetailLevel,
    ImageAnalysisRequest, ImageAnalysisTask, OpenAIVisionClient, OpenAIWhisperClient,
    VideoAnalysisRequest, VideoAnalyzer, VideoAnalyzerConfig,
};
use base64::{engine::general_purpose, Engine as _};
use tracing::Level;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化 tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    println!("=== 多模态 AI 功能演示 ===\n");

    // 演示 1: OpenAI Vision API 图像分析
    demo_vision_api().await?;

    // 演示 2: OpenAI Whisper API 音频转录
    demo_whisper_api().await?;

    // 演示 3: 视频分析器
    demo_video_analyzer().await?;

    println!("\n=== 演示完成 ===");
    Ok(())
}

/// 演示 1: OpenAI Vision API 图像分析
async fn demo_vision_api() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- 演示 1: OpenAI Vision API 图像分析 ---");

    // 检查是否有 API 密钥
    let api_key = std::env::var("OPENAI_API_KEY").ok();
    
    if api_key.is_none() {
        println!("⚠️  未设置 OPENAI_API_KEY 环境变量");
        println!("   提示: export OPENAI_API_KEY='your-api-key'");
        println!("   跳过真实 API 调用演示\n");
        demo_vision_api_mock().await?;
        return Ok(());
    }

    println!("✓ 检测到 OPENAI_API_KEY");

    // 创建配置
    let config = AIModelConfig::openai(api_key.unwrap());
    println!("✓ 创建 OpenAI Vision 配置");
    println!("  提供商: {:?}", config.provider);
    println!("  Base URL: {}", config.base_url.as_ref().unwrap());

    // 创建客户端
    let client = OpenAIVisionClient::new(config)?;
    println!("✓ 创建 OpenAI Vision 客户端");

    // 创建一个简单的测试图像（1x1 像素的红色图片）
    let test_image = create_test_image();
    println!("\n✓ 创建测试图像 (1x1 红色像素)");

    // 构建分析请求
    let request = ImageAnalysisRequest {
        image_data: test_image,
        image_url: None,
        tasks: vec![
            ImageAnalysisTask::SceneDescription,
            ImageAnalysisTask::ObjectDetection,
            ImageAnalysisTask::LabelExtraction,
        ],
        detail_level: DetailLevel::Low,
        max_tokens: Some(300),
    };
    println!("✓ 构建图像分析请求");
    println!("  分析任务: {:?}", request.tasks);
    println!("  详细程度: {:?}", request.detail_level);

    // 注意：这里会调用真实的 OpenAI API
    println!("\n⚠️  注意: 以下操作将调用真实的 OpenAI API 并产生费用");
    println!("   如果不想调用，请按 Ctrl+C 退出");
    println!("   等待 3 秒...");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    match client.analyze_image(&request).await {
        Ok(response) => {
            println!("\n✓ 图像分析成功!");
            println!("  置信度: {:.2}", response.confidence);
            
            if let Some(desc) = &response.scene_description {
                println!("\n  场景描述:");
                println!("    {}", desc);
            }
            
            if !response.objects.is_empty() {
                println!("\n  检测到的对象 ({}):", response.objects.len());
                for obj in &response.objects {
                    println!("    - {}: {:.2}%", obj.name, obj.confidence * 100.0);
                }
            }
            
            if !response.labels.is_empty() {
                println!("\n  标签 ({}):", response.labels.len());
                for label in &response.labels {
                    println!("    - {}", label.name);
                }
            }
        }
        Err(e) => {
            println!("\n✗ 图像分析失败: {}", e);
            println!("  这可能是由于 API 密钥无效或网络问题");
        }
    }

    println!();
    Ok(())
}

/// 模拟演示（不调用真实 API）
async fn demo_vision_api_mock() -> Result<(), Box<dyn std::error::Error>> {
    println!("✓ 使用模拟模式演示功能");
    println!("\n模拟图像分析结果:");
    println!("  场景描述: 一个简单的红色方块图像");
    println!("  检测到的对象:");
    println!("    - 方块: 95.00%");
    println!("    - 红色物体: 90.00%");
    println!("  标签:");
    println!("    - geometric");
    println!("    - red");
    println!("    - simple");
    println!("  置信度: 0.92");
    Ok(())
}

/// 演示 2: OpenAI Whisper API 音频转录
async fn demo_whisper_api() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- 演示 2: OpenAI Whisper API 音频转录 ---");

    // 检查是否有 API 密钥
    let api_key = std::env::var("OPENAI_API_KEY").ok();
    
    if api_key.is_none() {
        println!("⚠️  未设置 OPENAI_API_KEY 环境变量");
        println!("   跳过真实 API 调用演示\n");
        demo_whisper_api_mock().await?;
        return Ok(());
    }

    println!("✓ 检测到 OPENAI_API_KEY");

    // 创建配置
    let config = AIModelConfig::openai(api_key.unwrap());
    println!("✓ 创建 OpenAI Whisper 配置");

    // 创建客户端
    let client = OpenAIWhisperClient::new(config)?;
    println!("✓ 创建 OpenAI Whisper 客户端");

    // 创建一个简单的测试音频（空的 WAV 文件）
    let test_audio = create_test_audio();
    println!("\n✓ 创建测试音频 (空 WAV 文件)");

    // 构建转录请求
    let request = AudioTranscriptionRequest {
        audio_data: test_audio,
        format: AudioFormat::Wav,
        language: Some("zh".to_string()),
        enable_timestamps: true,
        enable_speaker_diarization: false,
    };
    println!("✓ 构建音频转录请求");
    println!("  音频格式: {:?}", request.format);
    println!("  语言: {:?}", request.language);
    println!("  启用时间戳: {}", request.enable_timestamps);

    println!("\n⚠️  注意: 由于测试音频为空，API 可能返回错误");
    println!("   这是正常的，仅用于演示 API 调用流程");

    match client.transcribe_audio(&request).await {
        Ok(response) => {
            println!("\n✓ 音频转录成功!");
            println!("  转录文本: {}", response.text);
            println!("  检测语言: {:?}", response.detected_language);
            println!("  段落数: {}", response.segments.len());
            println!("  置信度: {:.2}", response.confidence);
        }
        Err(e) => {
            println!("\n✗ 音频转录失败: {}", e);
            println!("  这是预期的，因为测试音频为空");
        }
    }

    println!();
    Ok(())
}

/// 模拟演示（不调用真实 API）
async fn demo_whisper_api_mock() -> Result<(), Box<dyn std::error::Error>> {
    println!("✓ 使用模拟模式演示功能");
    println!("\n模拟音频转录结果:");
    println!("  转录文本: 这是一段测试音频");
    println!("  检测语言: zh");
    println!("  段落数: 1");
    println!("  段落 1:");
    println!("    时间: 0.00s - 2.50s");
    println!("    文本: 这是一段测试音频");
    println!("    置信度: 0.95");
    Ok(())
}

/// 演示 3: 视频分析器
async fn demo_video_analyzer() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- 演示 3: 视频分析器 ---");

    // 检查是否有 API 密钥
    let api_key = std::env::var("OPENAI_API_KEY").ok();
    
    if api_key.is_none() {
        println!("⚠️  未设置 OPENAI_API_KEY 环境变量");
        println!("   跳过视频分析演示\n");
        demo_video_analyzer_mock().await?;
        return Ok(());
    }

    println!("✓ 检测到 OPENAI_API_KEY");

    // 创建配置
    let vision_config = AIModelConfig::openai(api_key.clone().unwrap());
    let audio_config = AIModelConfig::openai(api_key.unwrap());

    let config = VideoAnalyzerConfig {
        vision_config: Some(vision_config),
        audio_config: Some(audio_config),
        enable_keyframe_extraction: true,
        enable_audio_extraction: true,
        enable_scene_detection: true,
        keyframe_interval_seconds: 30.0,
        scene_change_threshold: 0.3,
    };
    println!("✓ 创建视频分析器配置");

    // 创建分析器
    let analyzer = VideoAnalyzer::new(config)?;
    println!("✓ 创建视频分析器");

    // 创建测试视频分析请求
    let test_image = create_test_image();
    let request = VideoAnalysisRequest {
        video_id: "test-video-001".to_string(),
        duration_seconds: 60.0,
        width: 1920,
        height: 1080,
        fps: 30.0,
        keyframes: vec![test_image.clone(), test_image],
        audio_track: None,
    };
    println!("\n✓ 创建视频分析请求");
    println!("  视频 ID: {}", request.video_id);
    println!("  时长: {:.1}s", request.duration_seconds);
    println!("  分辨率: {}x{}", request.width, request.height);
    println!("  帧率: {:.1} fps", request.fps);
    println!("  关键帧数: {}", request.keyframes.len());

    println!("\n⚠️  注意: 视频分析将调用多次 OpenAI API");
    println!("   跳过真实 API 调用，使用模拟结果");
    
    demo_video_analyzer_mock().await?;

    println!();
    Ok(())
}

/// 模拟演示（不调用真实 API）
async fn demo_video_analyzer_mock() -> Result<(), Box<dyn std::error::Error>> {
    println!("✓ 使用模拟模式演示功能");
    println!("\n模拟视频分析结果:");
    println!("  关键帧分析:");
    println!("    关键帧 1 (0.0s): 开场场景，包含标题文字");
    println!("    关键帧 2 (30.0s): 主要内容场景");
    println!("  场景检测:");
    println!("    场景 1 (0.0s - 25.0s): 室内场景");
    println!("    场景 2 (25.0s - 60.0s): 室外场景");
    println!("  视频摘要:");
    println!("    视频包含 2 个场景。场景 1: 包含: indoor, room (0.0s - 25.0s)。");
    println!("    场景 2: 包含: outdoor, street (25.0s - 60.0s)");
    println!("  标签: indoor, outdoor, room, street");
    println!("  置信度: 0.88");
    Ok(())
}

/// 创建测试图像（1x1 红色像素的 PNG）
fn create_test_image() -> String {
    // 最小的 PNG 图像（1x1 红色像素）
    let png_data = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG 签名
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1
        0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, 0xDE,
        0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, // IDAT chunk
        0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00, 0x00,
        0x03, 0x01, 0x01, 0x00, 0x18, 0xDD, 0x8D, 0xB4,
        0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, // IEND chunk
        0xAE, 0x42, 0x60, 0x82,
    ];
    
    general_purpose::STANDARD.encode(&png_data)
}

/// 创建测试音频（空的 WAV 文件）
fn create_test_audio() -> String {
    // 最小的 WAV 文件头
    let wav_data = vec![
        0x52, 0x49, 0x46, 0x46, // "RIFF"
        0x24, 0x00, 0x00, 0x00, // 文件大小
        0x57, 0x41, 0x56, 0x45, // "WAVE"
        0x66, 0x6D, 0x74, 0x20, // "fmt "
        0x10, 0x00, 0x00, 0x00, // fmt chunk 大小
        0x01, 0x00, 0x01, 0x00, // PCM, 单声道
        0x44, 0xAC, 0x00, 0x00, // 44100 Hz
        0x88, 0x58, 0x01, 0x00, // 字节率
        0x02, 0x00, 0x10, 0x00, // 块对齐, 16 位
        0x64, 0x61, 0x74, 0x61, // "data"
        0x00, 0x00, 0x00, 0x00, // 数据大小（0）
    ];
    
    general_purpose::STANDARD.encode(&wav_data)
}

