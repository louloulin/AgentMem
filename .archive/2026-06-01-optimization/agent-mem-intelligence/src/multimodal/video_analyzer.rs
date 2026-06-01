//! 视频分析器
//!
//! 使用 AI 模型进行视频内容分析

use super::ai_models::*;
use super::openai_vision::OpenAIVisionClient;
use super::openai_whisper::OpenAIWhisperClient;
use agent_mem_traits::{AgentMemError, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// 视频分析器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoAnalyzerConfig {
    /// 图像分析配置
    pub vision_config: Option<AIModelConfig>,
    /// 音频分析配置
    pub audio_config: Option<AIModelConfig>,
    /// 是否启用关键帧提取
    pub enable_keyframe_extraction: bool,
    /// 是否启用音频提取
    pub enable_audio_extraction: bool,
    /// 是否启用场景检测
    pub enable_scene_detection: bool,
    /// 关键帧间隔（秒）
    pub keyframe_interval_seconds: f64,
    /// 场景变化阈值
    pub scene_change_threshold: f32,
}

impl Default for VideoAnalyzerConfig {
    fn default() -> Self {
        Self {
            vision_config: None,
            audio_config: None,
            enable_keyframe_extraction: true,
            enable_audio_extraction: true,
            enable_scene_detection: true,
            keyframe_interval_seconds: 30.0,
            scene_change_threshold: 0.3,
        }
    }
}

/// 视频分析器
pub struct VideoAnalyzer {
    /// 配置
    config: VideoAnalyzerConfig,
    /// 图像分析客户端
    vision_client: Option<OpenAIVisionClient>,
    /// 音频分析客户端
    audio_client: Option<OpenAIWhisperClient>,
}

impl VideoAnalyzer {
    /// 创建新的视频分析器
    pub fn new(config: VideoAnalyzerConfig) -> Result<Self> {
        let vision_client = if let Some(vision_config) = &config.vision_config {
            Some(OpenAIVisionClient::new(vision_config.clone())?)
        } else {
            None
        };

        let audio_client = if let Some(audio_config) = &config.audio_config {
            Some(OpenAIWhisperClient::new(audio_config.clone())?)
        } else {
            None
        };

        Ok(Self {
            config,
            vision_client,
            audio_client,
        })
    }

    /// 分析视频
    pub async fn analyze_video(
        &self,
        request: &VideoAnalysisRequest,
    ) -> Result<VideoAnalysisResponse> {
        info!("开始视频分析: {}", request.video_id);

        let mut response = VideoAnalysisResponse {
            video_id: request.video_id.clone(),
            duration_seconds: request.duration_seconds,
            width: request.width,
            height: request.height,
            fps: request.fps,
            keyframes: Vec::new(),
            scenes: Vec::new(),
            audio_transcription: None,
            summary: None,
            tags: Vec::new(),
            confidence: 0.0,
        };

        // 1. 提取和分析关键帧
        if self.config.enable_keyframe_extraction && self.vision_client.is_some() {
            response.keyframes = self.extract_and_analyze_keyframes(request).await?;
            info!("提取了 {} 个关键帧", response.keyframes.len());
        }

        // 2. 检测场景
        if self.config.enable_scene_detection {
            response.scenes = self.detect_scenes(request, &response.keyframes).await?;
            info!("检测到 {} 个场景", response.scenes.len());
        }

        // 3. 提取和转录音频
        if self.config.enable_audio_extraction && self.audio_client.is_some() {
            if let Some(audio_data) = &request.audio_track {
                response.audio_transcription = Some(self.transcribe_audio(audio_data).await?);
                info!("音频转录完成");
            }
        }

        // 4. 生成视频摘要
        response.summary = Some(self.generate_summary(&response).await?);

        // 5. 提取标签
        response.tags = self.extract_tags(&response).await?;

        // 6. 计算总体置信度
        response.confidence = self.calculate_confidence(&response);

        info!("视频分析完成，置信度: {:.2}", response.confidence);

        Ok(response)
    }

    /// 提取和分析关键帧
    async fn extract_and_analyze_keyframes(
        &self,
        request: &VideoAnalysisRequest,
    ) -> Result<Vec<KeyframeAnalysis>> {
        let vision_client = self
            .vision_client
            .as_ref()
            .ok_or_else(|| AgentMemError::internal_error("图像分析客户端未初始化".to_string()))?;

        let mut keyframes = Vec::new();
        let interval = self.config.keyframe_interval_seconds;
        let num_keyframes = (request.duration_seconds / interval).ceil() as usize;

        for i in 0..num_keyframes {
            let timestamp = i as f64 * interval;
            if timestamp >= request.duration_seconds {
                break;
            }

            // 获取关键帧图像数据
            if let Some(frame_data) = request.keyframes.get(i) {
                // 分析关键帧
                let analysis_request = ImageAnalysisRequest {
                    image_data: frame_data.clone(),
                    image_url: None,
                    tasks: vec![
                        ImageAnalysisTask::SceneDescription,
                        ImageAnalysisTask::ObjectDetection,
                        ImageAnalysisTask::LabelExtraction,
                    ],
                    detail_level: DetailLevel::Medium,
                    max_tokens: Some(500),
                };

                let analysis = vision_client.analyze_image(&analysis_request).await?;

                keyframes.push(KeyframeAnalysis {
                    timestamp_seconds: timestamp,
                    frame_number: (timestamp * request.fps) as u64,
                    scene_description: analysis.scene_description,
                    objects: analysis.objects,
                    labels: analysis.labels,
                    confidence: analysis.confidence,
                });

                debug!("分析关键帧 {} (时间: {:.2}s)", i, timestamp);
            }
        }

        Ok(keyframes)
    }

    /// 检测场景
    async fn detect_scenes(
        &self,
        request: &VideoAnalysisRequest,
        keyframes: &[KeyframeAnalysis],
    ) -> Result<Vec<SceneAnalysis>> {
        let mut scenes = Vec::new();
        let mut current_scene_start = 0.0;
        let mut current_scene_labels: Vec<String> = Vec::new();

        for (i, keyframe) in keyframes.iter().enumerate() {
            let keyframe_labels: Vec<String> =
                keyframe.labels.iter().map(|l| l.name.clone()).collect();

            // 检测场景变化
            let similarity =
                self.calculate_label_similarity(&current_scene_labels, &keyframe_labels);

            if similarity < self.config.scene_change_threshold || i == 0 {
                // 场景变化
                if i > 0 {
                    // 保存前一个场景
                    scenes.push(SceneAnalysis {
                        start_time: current_scene_start,
                        end_time: keyframe.timestamp_seconds,
                        scene_type: self.infer_scene_type(&current_scene_labels),
                        description: self.generate_scene_description(&current_scene_labels),
                        dominant_objects: current_scene_labels.clone(),
                        confidence: 0.8,
                    });
                }

                current_scene_start = keyframe.timestamp_seconds;
                current_scene_labels = keyframe_labels;
            } else {
                // 合并标签
                for label in keyframe_labels {
                    if !current_scene_labels.contains(&label) {
                        current_scene_labels.push(label);
                    }
                }
            }
        }

        // 添加最后一个场景
        if !current_scene_labels.is_empty() {
            scenes.push(SceneAnalysis {
                start_time: current_scene_start,
                end_time: request.duration_seconds,
                scene_type: self.infer_scene_type(&current_scene_labels),
                description: self.generate_scene_description(&current_scene_labels),
                dominant_objects: current_scene_labels,
                confidence: 0.8,
            });
        }

        Ok(scenes)
    }

    /// 转录音频
    async fn transcribe_audio(&self, audio_data: &str) -> Result<AudioTranscriptionResponse> {
        let audio_client = self
            .audio_client
            .as_ref()
            .ok_or_else(|| AgentMemError::internal_error("音频分析客户端未初始化".to_string()))?;

        let request = AudioTranscriptionRequest {
            audio_data: audio_data.to_string(),
            format: AudioFormat::Mp3, // 假设为 MP3
            language: None,
            enable_timestamps: true,
            enable_speaker_diarization: true,
        };

        audio_client.transcribe_audio(&request).await
    }

    /// 生成视频摘要
    async fn generate_summary(&self, response: &VideoAnalysisResponse) -> Result<String> {
        let mut summary_parts = Vec::new();

        // 基于场景生成摘要
        if !response.scenes.is_empty() {
            summary_parts.push(format!("视频包含 {} 个场景", response.scenes.len()));

            for (i, scene) in response.scenes.iter().enumerate() {
                summary_parts.push(format!(
                    "场景 {}: {} ({:.1}s - {:.1}s)",
                    i + 1,
                    scene.description,
                    scene.start_time,
                    scene.end_time
                ));
            }
        }

        // 基于音频转录生成摘要
        if let Some(transcription) = &response.audio_transcription {
            if !transcription.text.is_empty() {
                summary_parts.push(format!("音频内容: {}", transcription.text));
            }
        }

        Ok(summary_parts.join(". "))
    }

    /// 提取标签
    async fn extract_tags(&self, response: &VideoAnalysisResponse) -> Result<Vec<String>> {
        let mut tags = std::collections::HashSet::new();

        // 从关键帧提取标签
        for keyframe in &response.keyframes {
            for label in &keyframe.labels {
                tags.insert(label.name.clone());
            }
        }

        // 从场景提取标签
        for scene in &response.scenes {
            tags.insert(scene.scene_type.clone());
            for obj in &scene.dominant_objects {
                tags.insert(obj.clone());
            }
        }

        Ok(tags.into_iter().collect())
    }

    /// 计算置信度
    fn calculate_confidence(&self, response: &VideoAnalysisResponse) -> f32 {
        let mut total_confidence = 0.0;
        let mut count = 0;

        for keyframe in &response.keyframes {
            total_confidence += keyframe.confidence;
            count += 1;
        }

        for scene in &response.scenes {
            total_confidence += scene.confidence;
            count += 1;
        }

        if count > 0 {
            total_confidence / count as f32
        } else {
            0.0
        }
    }

    /// 计算标签相似度
    fn calculate_label_similarity(&self, labels1: &[String], labels2: &[String]) -> f32 {
        if labels1.is_empty() || labels2.is_empty() {
            return 0.0;
        }

        let set1: std::collections::HashSet<_> = labels1.iter().collect();
        let set2: std::collections::HashSet<_> = labels2.iter().collect();

        let intersection = set1.intersection(&set2).count();
        let union = set1.union(&set2).count();

        intersection as f32 / union as f32
    }

    /// 推断场景类型
    fn infer_scene_type(&self, labels: &[String]) -> String {
        // 简单的场景类型推断
        for label in labels {
            let label_lower = label.to_lowercase();
            if label_lower.contains("indoor") || label_lower.contains("room") {
                return "indoor".to_string();
            }
            if label_lower.contains("outdoor") || label_lower.contains("street") {
                return "outdoor".to_string();
            }
        }
        "general".to_string()
    }

    /// 生成场景描述
    fn generate_scene_description(&self, labels: &[String]) -> String {
        if labels.is_empty() {
            return "未知场景".to_string();
        }
        format!("包含: {}", labels.join(", "))
    }
}

/// 视频分析请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoAnalysisRequest {
    /// 视频 ID
    pub video_id: String,
    /// 视频时长（秒）
    pub duration_seconds: f64,
    /// 宽度
    pub width: u32,
    /// 高度
    pub height: u32,
    /// 帧率
    pub fps: f64,
    /// 关键帧图像数据（base64）
    pub keyframes: Vec<String>,
    /// 音频轨道数据（base64）
    pub audio_track: Option<String>,
}

/// 视频分析响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoAnalysisResponse {
    /// 视频 ID
    pub video_id: String,
    /// 视频时长（秒）
    pub duration_seconds: f64,
    /// 宽度
    pub width: u32,
    /// 高度
    pub height: u32,
    /// 帧率
    pub fps: f64,
    /// 关键帧分析
    pub keyframes: Vec<KeyframeAnalysis>,
    /// 场景分析
    pub scenes: Vec<SceneAnalysis>,
    /// 音频转录
    pub audio_transcription: Option<AudioTranscriptionResponse>,
    /// 视频摘要
    pub summary: Option<String>,
    /// 标签
    pub tags: Vec<String>,
    /// 置信度
    pub confidence: f32,
}

/// 关键帧分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyframeAnalysis {
    /// 时间戳（秒）
    pub timestamp_seconds: f64,
    /// 帧号
    pub frame_number: u64,
    /// 场景描述
    pub scene_description: Option<String>,
    /// 检测到的对象
    pub objects: Vec<DetectedObject>,
    /// 标签
    pub labels: Vec<Label>,
    /// 置信度
    pub confidence: f32,
}

/// 场景分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneAnalysis {
    /// 开始时间（秒）
    pub start_time: f64,
    /// 结束时间（秒）
    pub end_time: f64,
    /// 场景类型
    pub scene_type: String,
    /// 场景描述
    pub description: String,
    /// 主要对象
    pub dominant_objects: Vec<String>,
    /// 置信度
    pub confidence: f32,
}
