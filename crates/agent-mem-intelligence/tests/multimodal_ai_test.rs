//! 多模态 AI 集成测试

use agent_mem_intelligence::multimodal::{
    AIModelConfig, AIModelProvider, AudioFormat, AudioTranscriptionRequest, DetailLevel,
    ImageAnalysisRequest, ImageAnalysisTask, VideoAnalyzerConfig,
};

#[test]
fn test_ai_model_config_creation() {
    // 测试 OpenAI 配置
    let openai_config = AIModelConfig::openai("test-key".to_string());
    assert_eq!(openai_config.provider, AIModelProvider::OpenAI);
    assert_eq!(openai_config.api_key, Some("test-key".to_string()));
    assert_eq!(
        openai_config.base_url,
        Some("https://api.openai.com/v1".to_string())
    );

    // 测试 Google 配置
    let google_config = AIModelConfig::google("test-key".to_string());
    assert_eq!(google_config.provider, AIModelProvider::Google);
    assert_eq!(google_config.api_key, Some("test-key".to_string()));

    // 测试 Azure 配置
    let azure_config = AIModelConfig::azure("test-key".to_string(), "eastus".to_string());
    assert_eq!(azure_config.provider, AIModelProvider::Azure);
    assert_eq!(azure_config.region, Some("eastus".to_string()));

    // 测试本地配置
    let local_config = AIModelConfig::local();
    assert_eq!(local_config.provider, AIModelProvider::Local);
}

#[test]
fn test_image_analysis_request_creation() {
    let request = ImageAnalysisRequest {
        image_data: "base64_data".to_string(),
        image_url: None,
        tasks: vec![
            ImageAnalysisTask::SceneDescription,
            ImageAnalysisTask::ObjectDetection,
        ],
        detail_level: DetailLevel::Medium,
        max_tokens: Some(500),
    };

    assert_eq!(request.image_data, "base64_data");
    assert_eq!(request.tasks.len(), 2);
    assert_eq!(request.detail_level, DetailLevel::Medium);
    assert_eq!(request.max_tokens, Some(500));
}

#[test]
fn test_audio_transcription_request_creation() {
    let request = AudioTranscriptionRequest {
        audio_data: "base64_audio".to_string(),
        format: AudioFormat::Mp3,
        language: Some("zh".to_string()),
        enable_timestamps: true,
        enable_speaker_diarization: false,
    };

    assert_eq!(request.audio_data, "base64_audio");
    assert_eq!(request.format, AudioFormat::Mp3);
    assert_eq!(request.language, Some("zh".to_string()));
    assert!(request.enable_timestamps);
    assert!(!request.enable_speaker_diarization);
}

#[test]
fn test_video_analyzer_config_creation() {
    let vision_config = AIModelConfig::openai("test-key".to_string());
    let audio_config = AIModelConfig::openai("test-key".to_string());

    let config = VideoAnalyzerConfig {
        vision_config: Some(vision_config),
        audio_config: Some(audio_config),
        enable_keyframe_extraction: true,
        enable_audio_extraction: true,
        enable_scene_detection: true,
        keyframe_interval_seconds: 30.0,
        scene_change_threshold: 0.3,
    };

    assert!(config.vision_config.is_some());
    assert!(config.audio_config.is_some());
    assert!(config.enable_keyframe_extraction);
    assert!(config.enable_audio_extraction);
    assert!(config.enable_scene_detection);
    assert_eq!(config.keyframe_interval_seconds, 30.0);
    assert_eq!(config.scene_change_threshold, 0.3);
}

#[test]
fn test_image_analysis_tasks() {
    let tasks = vec![
        ImageAnalysisTask::SceneDescription,
        ImageAnalysisTask::ObjectDetection,
        ImageAnalysisTask::TextRecognition,
        ImageAnalysisTask::FaceDetection,
        ImageAnalysisTask::LabelExtraction,
        ImageAnalysisTask::ColorAnalysis,
    ];

    assert_eq!(tasks.len(), 6);
    assert!(tasks.contains(&ImageAnalysisTask::SceneDescription));
    assert!(tasks.contains(&ImageAnalysisTask::ObjectDetection));
}

#[test]
fn test_audio_formats() {
    let formats = vec![
        AudioFormat::Mp3,
        AudioFormat::Mp4,
        AudioFormat::Wav,
        AudioFormat::Flac,
        AudioFormat::Ogg,
    ];

    assert_eq!(formats.len(), 5);
    assert!(formats.contains(&AudioFormat::Mp3));
    assert!(formats.contains(&AudioFormat::Wav));
}

#[test]
fn test_detail_levels() {
    assert_eq!(DetailLevel::Low, DetailLevel::Low);
    assert_ne!(DetailLevel::Low, DetailLevel::High);

    let levels = vec![
        DetailLevel::Low,
        DetailLevel::Medium,
        DetailLevel::High,
        DetailLevel::Auto,
    ];

    assert_eq!(levels.len(), 4);
}

#[test]
fn test_ai_model_provider_equality() {
    assert_eq!(AIModelProvider::OpenAI, AIModelProvider::OpenAI);
    assert_ne!(AIModelProvider::OpenAI, AIModelProvider::Google);
    assert_ne!(AIModelProvider::Google, AIModelProvider::Azure);
}

#[test]
fn test_default_config() {
    let config = AIModelConfig::default();
    assert_eq!(config.provider, AIModelProvider::OpenAI);
    assert_eq!(config.timeout_seconds, 60);
    assert_eq!(config.max_retries, 3);
}

#[cfg(feature = "multimodal")]
#[tokio::test]
async fn test_openai_vision_client_creation() {
    use agent_mem_intelligence::multimodal::OpenAIVisionClient;

    let config = AIModelConfig::openai("test-key".to_string());
    let result = OpenAIVisionClient::new(config);
    assert!(result.is_ok());
}

#[cfg(feature = "multimodal")]
#[tokio::test]
async fn test_openai_whisper_client_creation() {
    use agent_mem_intelligence::multimodal::OpenAIWhisperClient;

    let config = AIModelConfig::openai("test-key".to_string());
    let result = OpenAIWhisperClient::new(config);
    assert!(result.is_ok());
}

#[cfg(feature = "multimodal")]
#[tokio::test]
async fn test_video_analyzer_creation() {
    use agent_mem_intelligence::multimodal::VideoAnalyzer;

    let vision_config = AIModelConfig::openai("test-key".to_string());
    let audio_config = AIModelConfig::openai("test-key".to_string());

    let config = VideoAnalyzerConfig {
        vision_config: Some(vision_config),
        audio_config: Some(audio_config),
        enable_keyframe_extraction: true,
        enable_audio_extraction: true,
        enable_scene_detection: true,
        keyframe_interval_seconds: 30.0,
        scene_change_threshold: 0.3,
    };

    let result = VideoAnalyzer::new(config);
    assert!(result.is_ok());
}

#[cfg(feature = "multimodal")]
#[tokio::test]
async fn test_video_analyzer_without_clients() {
    use agent_mem_intelligence::multimodal::VideoAnalyzer;

    let config = VideoAnalyzerConfig {
        vision_config: None,
        audio_config: None,
        enable_keyframe_extraction: false,
        enable_audio_extraction: false,
        enable_scene_detection: true,
        keyframe_interval_seconds: 30.0,
        scene_change_threshold: 0.3,
    };

    let result = VideoAnalyzer::new(config);
    assert!(result.is_ok());
}

#[test]
fn test_image_analysis_request_with_url() {
    let request = ImageAnalysisRequest {
        image_data: String::new(),
        image_url: Some("https://example.com/image.jpg".to_string()),
        tasks: vec![ImageAnalysisTask::SceneDescription],
        detail_level: DetailLevel::Auto,
        max_tokens: None,
    };

    assert!(request.image_url.is_some());
    assert_eq!(request.image_url.unwrap(), "https://example.com/image.jpg");
}

#[test]
fn test_audio_transcription_with_speaker_diarization() {
    let request = AudioTranscriptionRequest {
        audio_data: "base64_audio".to_string(),
        format: AudioFormat::Wav,
        language: None,
        enable_timestamps: true,
        enable_speaker_diarization: true,
    };

    assert!(request.enable_speaker_diarization);
    assert!(request.enable_timestamps);
    assert!(request.language.is_none());
}

#[test]
fn test_video_analyzer_config_defaults() {
    let config = VideoAnalyzerConfig::default();

    assert!(config.vision_config.is_none());
    assert!(config.audio_config.is_none());
    assert!(config.enable_keyframe_extraction);
    assert!(config.enable_audio_extraction);
    assert!(config.enable_scene_detection);
    assert_eq!(config.keyframe_interval_seconds, 30.0);
    assert_eq!(config.scene_change_threshold, 0.3);
}
