//! 多模态内容处理模块
//!
//! 支持图像、音频、视频等多媒体内容的智能处理

pub mod audio;
pub mod image;
pub mod text;
pub mod video;

// 真实处理器模块
pub mod real_audio;
pub mod real_image;

// AI 模型集成模块（需要 multimodal feature）
#[cfg(feature = "multimodal")]
pub mod ai_models;
#[cfg(feature = "multimodal")]
pub mod openai_vision;
#[cfg(feature = "multimodal")]
pub mod openai_whisper;
#[cfg(feature = "multimodal")]
pub mod video_analyzer;

// 跨模态关联模块
pub mod cross_modal;
pub mod unified_retrieval;

// 性能优化模块
pub mod optimization;

// 重新导出 AI 模型相关类型（需要 multimodal feature）
#[cfg(feature = "multimodal")]
pub use ai_models::*;
#[cfg(feature = "multimodal")]
pub use openai_vision::OpenAIVisionClient;
#[cfg(feature = "multimodal")]
pub use openai_whisper::OpenAIWhisperClient;
#[cfg(feature = "multimodal")]
pub use video_analyzer::{VideoAnalyzer, VideoAnalyzerConfig, VideoAnalysisRequest, VideoAnalysisResponse};

use agent_mem_traits::{AgentMemError, Result};
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 多模态内容类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ContentType {
    Text,
    Image,
    Audio,
    Video,
    Document,
    Unknown,
}

impl ContentType {
    /// 从 MIME 类型推断内容类型
    pub fn from_mime_type(mime_type: &str) -> Self {
        match mime_type {
            mime if mime.starts_with("text/") => ContentType::Text,
            mime if mime.starts_with("image/") => ContentType::Image,
            mime if mime.starts_with("audio/") => ContentType::Audio,
            mime if mime.starts_with("video/") => ContentType::Video,
            "application/pdf"
            | "application/msword"
            | "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => {
                ContentType::Document
            }
            _ => ContentType::Unknown,
        }
    }

    /// 获取支持的文件扩展名
    pub fn supported_extensions(&self) -> Vec<&'static str> {
        match self {
            ContentType::Text => vec!["txt", "md", "json", "xml", "html"],
            ContentType::Image => vec!["jpg", "jpeg", "png", "gif", "bmp", "webp", "svg"],
            ContentType::Audio => vec!["mp3", "wav", "flac", "aac", "ogg", "m4a"],
            ContentType::Video => vec!["mp4", "avi", "mov", "wmv", "flv", "webm", "mkv"],
            ContentType::Document => vec!["pdf", "doc", "docx", "ppt", "pptx", "xls", "xlsx"],
            ContentType::Unknown => vec![],
        }
    }
}

/// 多模态内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalContent {
    /// 内容 ID
    pub id: String,
    /// 内容类型
    pub content_type: ContentType,
    /// 原始数据（Base64 编码）
    pub data: Option<String>,
    /// 文件路径（如果是文件）
    pub file_path: Option<String>,
    /// URL（如果是网络资源）
    pub url: Option<String>,
    /// MIME 类型
    pub mime_type: Option<String>,
    /// 文件大小（字节）
    pub size: Option<u64>,
    /// 元数据
    pub metadata: HashMap<String, serde_json::Value>,
    /// 提取的文本内容
    pub extracted_text: Option<String>,
    /// 处理状态
    pub processing_status: ProcessingStatus,
}

/// 处理状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessingStatus {
    Pending,
    Processing,
    Completed,
    Failed(String),
}

impl MultimodalContent {
    /// 创建新的多模态内容
    pub fn new(id: String, content_type: ContentType) -> Self {
        Self {
            id,
            content_type,
            data: None,
            file_path: None,
            url: None,
            mime_type: None,
            size: None,
            metadata: HashMap::new(),
            extracted_text: None,
            processing_status: ProcessingStatus::Pending,
        }
    }

    /// 从文件创建
    pub fn from_file(id: String, file_path: String, mime_type: Option<String>) -> Self {
        let content_type = mime_type
            .as_ref()
            .map(|m| ContentType::from_mime_type(m))
            .unwrap_or(ContentType::Unknown);

        Self {
            id,
            content_type,
            data: None,
            file_path: Some(file_path),
            url: None,
            mime_type,
            size: None,
            metadata: HashMap::new(),
            extracted_text: None,
            processing_status: ProcessingStatus::Pending,
        }
    }

    /// 从 URL 创建
    pub fn from_url(id: String, url: String, mime_type: Option<String>) -> Self {
        let content_type = mime_type
            .as_ref()
            .map(|m| ContentType::from_mime_type(m))
            .unwrap_or(ContentType::Unknown);

        Self {
            id,
            content_type,
            data: None,
            file_path: None,
            url: Some(url),
            mime_type,
            size: None,
            metadata: HashMap::new(),
            extracted_text: None,
            processing_status: ProcessingStatus::Pending,
        }
    }

    /// 从数据创建
    pub fn from_data(id: String, data: Vec<u8>, mime_type: String) -> Self {
        let content_type = ContentType::from_mime_type(&mime_type);
        let base64_data = general_purpose::STANDARD.encode(&data);

        Self {
            id,
            content_type,
            data: Some(base64_data),
            file_path: None,
            url: None,
            mime_type: Some(mime_type),
            size: Some(data.len() as u64),
            metadata: HashMap::new(),
            extracted_text: None,
            processing_status: ProcessingStatus::Pending,
        }
    }

    /// 设置元数据
    pub fn set_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
    }

    /// 获取元数据
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }

    /// 设置提取的文本
    pub fn set_extracted_text(&mut self, text: String) {
        self.extracted_text = Some(text);
    }

    /// 设置处理状态
    pub fn set_processing_status(&mut self, status: ProcessingStatus) {
        self.processing_status = status;
    }

    /// 是否已处理完成
    pub fn is_processed(&self) -> bool {
        matches!(self.processing_status, ProcessingStatus::Completed)
    }

    /// 是否处理失败
    pub fn is_failed(&self) -> bool {
        matches!(self.processing_status, ProcessingStatus::Failed(_))
    }
}

/// 多模态处理器特征
#[async_trait]
pub trait MultimodalProcessor: Send + Sync {
    /// 处理多模态内容
    async fn process(&self, content: &mut MultimodalContent) -> Result<()>;

    /// 支持的内容类型
    fn supported_types(&self) -> Vec<ContentType>;

    /// 提取文本内容
    async fn extract_text(&self, content: &MultimodalContent) -> Result<Option<String>>;

    /// 生成内容摘要
    async fn generate_summary(&self, content: &MultimodalContent) -> Result<Option<String>>;
}

/// 多模态处理器枚举
#[derive(Debug)]
pub enum ProcessorType {
    Text(text::TextProcessor),
    Image(image::ImageProcessor),
    Audio(audio::AudioProcessor),
    Video(video::VideoProcessor),
}

impl ProcessorType {
    /// 处理内容
    pub async fn process(&self, content: &mut MultimodalContent) -> Result<()> {
        match self {
            ProcessorType::Text(processor) => processor.process(content).await,
            ProcessorType::Image(processor) => processor.process(content).await,
            ProcessorType::Audio(processor) => processor.process(content).await,
            ProcessorType::Video(processor) => processor.process(content).await,
        }
    }

    /// 支持的内容类型
    pub fn supported_types(&self) -> Vec<ContentType> {
        match self {
            ProcessorType::Text(processor) => processor.supported_types(),
            ProcessorType::Image(processor) => processor.supported_types(),
            ProcessorType::Audio(processor) => processor.supported_types(),
            ProcessorType::Video(processor) => processor.supported_types(),
        }
    }

    /// 提取文本
    pub async fn extract_text(&self, content: &MultimodalContent) -> Result<Option<String>> {
        match self {
            ProcessorType::Text(processor) => processor.extract_text(content).await,
            ProcessorType::Image(processor) => processor.extract_text(content).await,
            ProcessorType::Audio(processor) => processor.extract_text(content).await,
            ProcessorType::Video(processor) => processor.extract_text(content).await,
        }
    }

    /// 生成摘要
    pub async fn generate_summary(&self, content: &MultimodalContent) -> Result<Option<String>> {
        match self {
            ProcessorType::Text(processor) => processor.generate_summary(content).await,
            ProcessorType::Image(processor) => processor.generate_summary(content).await,
            ProcessorType::Audio(processor) => processor.generate_summary(content).await,
            ProcessorType::Video(processor) => processor.generate_summary(content).await,
        }
    }
}

/// 多模态处理器管理器
pub struct MultimodalProcessorManager {
    processors: HashMap<ContentType, ProcessorType>,
}

impl MultimodalProcessorManager {
    /// 创建新的管理器
    pub fn new() -> Self {
        Self {
            processors: HashMap::new(),
        }
    }

    /// 注册处理器
    pub fn register_processor(&mut self, content_type: ContentType, processor: ProcessorType) {
        self.processors.insert(content_type, processor);
    }

    /// 处理内容
    pub async fn process_content(&self, content: &mut MultimodalContent) -> Result<()> {
        if let Some(processor) = self.processors.get(&content.content_type) {
            content.set_processing_status(ProcessingStatus::Processing);

            match processor.process(content).await {
                Ok(()) => {
                    content.set_processing_status(ProcessingStatus::Completed);
                    Ok(())
                }
                Err(e) => {
                    content.set_processing_status(ProcessingStatus::Failed(e.to_string()));
                    Err(e)
                }
            }
        } else {
            let error = format!(
                "No processor found for content type: {:?}",
                content.content_type
            );
            content.set_processing_status(ProcessingStatus::Failed(error.clone()));
            Err(AgentMemError::ParsingError(error))
        }
    }

    /// 批量处理内容
    pub async fn process_batch(
        &self,
        contents: &mut [MultimodalContent],
    ) -> Result<Vec<Result<()>>> {
        let mut results = Vec::new();

        for content in contents.iter_mut() {
            let result = self.process_content(content).await;
            results.push(result);
        }

        Ok(results)
    }

    /// 获取支持的内容类型
    pub fn supported_types(&self) -> Vec<ContentType> {
        self.processors.keys().cloned().collect()
    }
}

impl Default for MultimodalProcessorManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_from_mime_type() {
        assert_eq!(ContentType::from_mime_type("text/plain"), ContentType::Text);
        assert_eq!(
            ContentType::from_mime_type("image/jpeg"),
            ContentType::Image
        );
        assert_eq!(ContentType::from_mime_type("audio/mp3"), ContentType::Audio);
        assert_eq!(ContentType::from_mime_type("video/mp4"), ContentType::Video);
        assert_eq!(
            ContentType::from_mime_type("application/pdf"),
            ContentType::Document
        );
        assert_eq!(
            ContentType::from_mime_type("unknown/type"),
            ContentType::Unknown
        );
    }

    #[test]
    fn test_multimodal_content_creation() {
        let content = MultimodalContent::new("test-id".to_string(), ContentType::Image);
        assert_eq!(content.id, "test-id");
        assert_eq!(content.content_type, ContentType::Image);
        assert_eq!(content.processing_status, ProcessingStatus::Pending);
    }

    #[test]
    fn test_multimodal_content_from_file() {
        let content = MultimodalContent::from_file(
            "test-id".to_string(),
            "/path/to/image.jpg".to_string(),
            Some("image/jpeg".to_string()),
        );
        assert_eq!(content.content_type, ContentType::Image);
        assert_eq!(content.file_path, Some("/path/to/image.jpg".to_string()));
        assert_eq!(content.mime_type, Some("image/jpeg".to_string()));
    }

    #[test]
    fn test_multimodal_content_from_data() {
        let data = vec![1, 2, 3, 4, 5];
        let content = MultimodalContent::from_data(
            "test-id".to_string(),
            data.clone(),
            "image/png".to_string(),
        );
        assert_eq!(content.content_type, ContentType::Image);
        assert_eq!(content.size, Some(5));
        assert!(content.data.is_some());
    }

    #[test]
    fn test_content_type_supported_extensions() {
        let image_extensions = ContentType::Image.supported_extensions();
        assert!(image_extensions.contains(&"jpg"));
        assert!(image_extensions.contains(&"png"));

        let audio_extensions = ContentType::Audio.supported_extensions();
        assert!(audio_extensions.contains(&"mp3"));
        assert!(audio_extensions.contains(&"wav"));
    }
}
