//! AI 模型集成模块
//!
//! 集成真实的 AI 模型用于多模态内容处理

use serde::{Deserialize, Serialize};

/// AI 模型提供商
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AIModelProvider {
    /// OpenAI (GPT-4 Vision, Whisper)
    OpenAI,
    /// Google (Vision AI, Speech-to-Text)
    Google,
    /// Azure (Computer Vision, Speech)
    Azure,
    /// Anthropic (Claude with vision)
    Anthropic,
    /// 本地模型
    Local,
}

/// AI 模型配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIModelConfig {
    /// 提供商
    pub provider: AIModelProvider,
    /// API 密钥
    pub api_key: Option<String>,
    /// API 基础 URL
    pub base_url: Option<String>,
    /// 模型名称
    pub model_name: Option<String>,
    /// 区域（Azure 使用）
    pub region: Option<String>,
    /// 超时时间（秒）
    pub timeout_seconds: u64,
    /// 最大重试次数
    pub max_retries: u32,
}

impl Default for AIModelConfig {
    fn default() -> Self {
        Self {
            provider: AIModelProvider::OpenAI,
            api_key: None,
            base_url: None,
            model_name: None,
            region: None,
            timeout_seconds: 60,
            max_retries: 3,
        }
    }
}

impl AIModelConfig {
    /// 创建 OpenAI 配置
    pub fn openai(api_key: String) -> Self {
        Self {
            provider: AIModelProvider::OpenAI,
            api_key: Some(api_key),
            base_url: Some("https://api.openai.com/v1".to_string()),
            ..Default::default()
        }
    }

    /// 创建 Google 配置
    pub fn google(api_key: String) -> Self {
        Self {
            provider: AIModelProvider::Google,
            api_key: Some(api_key),
            base_url: Some("https://vision.googleapis.com/v1".to_string()),
            ..Default::default()
        }
    }

    /// 创建 Azure 配置
    pub fn azure(api_key: String, region: String) -> Self {
        Self {
            provider: AIModelProvider::Azure,
            api_key: Some(api_key),
            region: Some(region),
            ..Default::default()
        }
    }

    /// 创建本地模型配置
    pub fn local() -> Self {
        Self {
            provider: AIModelProvider::Local,
            ..Default::default()
        }
    }
}

/// 图像分析请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageAnalysisRequest {
    /// 图像数据（base64 编码）
    pub image_data: String,
    /// 图像 URL（可选）
    pub image_url: Option<String>,
    /// 分析任务
    pub tasks: Vec<ImageAnalysisTask>,
    /// 详细程度
    pub detail_level: DetailLevel,
    /// 最大 tokens
    pub max_tokens: Option<u32>,
}

/// 图像分析任务
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ImageAnalysisTask {
    /// 场景描述
    SceneDescription,
    /// 对象检测
    ObjectDetection,
    /// OCR 文本识别
    TextRecognition,
    /// 人脸检测
    FaceDetection,
    /// 标签提取
    LabelExtraction,
    /// 颜色分析
    ColorAnalysis,
}

/// 详细程度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DetailLevel {
    /// 低详细度
    Low,
    /// 中等详细度
    Medium,
    /// 高详细度
    High,
    /// 自动选择
    Auto,
}

/// 图像分析响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageAnalysisResponse {
    /// 场景描述
    pub scene_description: Option<String>,
    /// 检测到的对象
    pub objects: Vec<DetectedObject>,
    /// 识别的文本
    pub text: Option<String>,
    /// 检测到的人脸
    pub faces: Vec<DetectedFace>,
    /// 标签
    pub labels: Vec<Label>,
    /// 主要颜色
    pub dominant_colors: Vec<Color>,
    /// 置信度
    pub confidence: f32,
}

/// 检测到的对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedObject {
    /// 对象名称
    pub name: String,
    /// 置信度
    pub confidence: f32,
    /// 边界框
    pub bounding_box: Option<BoundingBox>,
}

/// 边界框
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// 检测到的人脸
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedFace {
    /// 置信度
    pub confidence: f32,
    /// 边界框
    pub bounding_box: BoundingBox,
    /// 年龄估计
    pub age_estimate: Option<u32>,
    /// 性别估计
    pub gender_estimate: Option<String>,
    /// 情绪
    pub emotion: Option<String>,
}

/// 标签
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    /// 标签名称
    pub name: String,
    /// 置信度
    pub confidence: f32,
    /// 类别
    pub category: Option<String>,
}

/// 颜色
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Color {
    /// 红色分量 (0-255)
    pub r: u8,
    /// 绿色分量 (0-255)
    pub g: u8,
    /// 蓝色分量 (0-255)
    pub b: u8,
    /// 颜色名称
    pub name: Option<String>,
    /// 占比
    pub percentage: f32,
}

/// 音频转录请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioTranscriptionRequest {
    /// 音频数据（base64 编码）
    pub audio_data: String,
    /// 音频格式
    pub format: AudioFormat,
    /// 语言（可选，ISO 639-1 代码）
    pub language: Option<String>,
    /// 是否启用时间戳
    pub enable_timestamps: bool,
    /// 是否启用说话人识别
    pub enable_speaker_diarization: bool,
}

/// 音频格式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AudioFormat {
    Mp3,
    Mp4,
    Mpeg,
    Mpga,
    M4a,
    Wav,
    Webm,
    Flac,
    Ogg,
}

/// 音频转录响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioTranscriptionResponse {
    /// 转录文本
    pub text: String,
    /// 检测到的语言
    pub detected_language: Option<String>,
    /// 时间戳段落
    pub segments: Vec<TranscriptionSegment>,
    /// 说话人信息
    pub speakers: Vec<Speaker>,
    /// 置信度
    pub confidence: f32,
}

/// 转录段落
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionSegment {
    /// 开始时间（秒）
    pub start_time: f64,
    /// 结束时间（秒）
    pub end_time: f64,
    /// 文本
    pub text: String,
    /// 说话人 ID
    pub speaker_id: Option<u32>,
    /// 置信度
    pub confidence: f32,
}

/// 说话人
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Speaker {
    /// 说话人 ID
    pub id: u32,
    /// 说话人标签
    pub label: String,
    /// 总发言时长（秒）
    pub total_duration: f64,
}
