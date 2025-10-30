//! OpenAI Whisper API 集成
//!
//! 使用 Whisper 进行音频转录

use super::ai_models::*;
use agent_mem_traits::{AgentMemError, Result};
use base64::{engine::general_purpose, Engine as _};
use serde::Deserialize;
use tracing::{debug, error, info};

#[cfg(feature = "multimodal")]
use reqwest::{multipart, Client};

/// OpenAI Whisper 客户端
pub struct OpenAIWhisperClient {
    /// HTTP 客户端
    client: Client,
    /// 配置
    config: AIModelConfig,
}

impl OpenAIWhisperClient {
    /// 创建新的 OpenAI Whisper 客户端
    pub fn new(config: AIModelConfig) -> Result<Self> {
        if config.provider != AIModelProvider::OpenAI {
            return Err(AgentMemError::InvalidInput(
                "配置必须使用 OpenAI 提供商".to_string(),
            ));
        }

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(|e| AgentMemError::internal_error(format!("创建 HTTP 客户端失败: {e}")))?;

        Ok(Self { client, config })
    }

    /// 转录音频
    pub async fn transcribe_audio(
        &self,
        request: &AudioTranscriptionRequest,
    ) -> Result<AudioTranscriptionResponse> {
        let api_key = self
            .config
            .api_key
            .as_ref()
            .ok_or_else(|| AgentMemError::InvalidInput("缺少 API 密钥".to_string()))?;

        let base_url = self
            .config
            .base_url
            .as_ref()
            .ok_or_else(|| AgentMemError::InvalidInput("缺少 base URL".to_string()))?;

        // 解码 base64 音频数据
        let audio_bytes = general_purpose::STANDARD
            .decode(&request.audio_data)
            .map_err(|e| AgentMemError::InvalidInput(format!("解码 base64 失败: {e}")))?;

        // 构建 multipart 表单
        let file_extension = self.map_audio_format(&request.format);
        let filename = format!("audio.{file_extension}");

        let file_part = multipart::Part::bytes(audio_bytes)
            .file_name(filename)
            .mime_str(&format!("audio/{file_extension}"))
            .map_err(|e| AgentMemError::internal_error(format!("创建文件部分失败: {e}")))?;

        let mut form = multipart::Form::new()
            .part("file", file_part)
            .text(
                "model",
                self.config
                    .model_name
                    .as_ref()
                    .unwrap_or(&"whisper-1".to_string())
                    .clone(),
            );

        // 添加可选参数
        if let Some(language) = &request.language {
            form = form.text("language", language.clone());
        }

        if request.enable_timestamps {
            form = form.text("response_format", "verbose_json");
            form = form.text("timestamp_granularities[]", "segment");
        } else {
            form = form.text("response_format", "json");
        }

        info!("发送音频转录请求到 OpenAI Whisper API");
        debug!("音频格式: {}, 语言: {:?}", file_extension, request.language);

        // 发送请求
        let response = self
            .client
            .post(format!("{base_url}/audio/transcriptions"))
            .header("Authorization", format!("Bearer {api_key}"))
            .multipart(form)
            .send()
            .await
            .map_err(|e| AgentMemError::internal_error(format!("发送请求失败: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "无法读取错误响应".to_string());
            error!("OpenAI Whisper API 错误 ({}): {}", status, error_text);
            return Err(AgentMemError::internal_error(format!(
                "OpenAI Whisper API 错误 ({status}): {error_text}"
            )));
        }

        let api_response: WhisperResponse = response
            .json()
            .await
            .map_err(|e| AgentMemError::internal_error(format!("解析响应失败: {e}")))?;

        debug!("OpenAI Whisper API 响应: {:?}", api_response);

        // 转换响应
        self.convert_response(api_response, request.enable_speaker_diarization)
    }

    /// 映射音频格式
    fn map_audio_format(&self, format: &AudioFormat) -> &str {
        match format {
            AudioFormat::Mp3 => "mp3",
            AudioFormat::Mp4 => "mp4",
            AudioFormat::Mpeg => "mpeg",
            AudioFormat::Mpga => "mpga",
            AudioFormat::M4a => "m4a",
            AudioFormat::Wav => "wav",
            AudioFormat::Webm => "webm",
            AudioFormat::Flac => "flac",
            AudioFormat::Ogg => "ogg",
        }
    }

    /// 转换响应
    fn convert_response(
        &self,
        response: WhisperResponse,
        enable_speaker_diarization: bool,
    ) -> Result<AudioTranscriptionResponse> {
        let mut segments = Vec::new();
        let mut speakers = Vec::new();

        if let Some(api_segments) = response.segments {
            for (i, seg) in api_segments.iter().enumerate() {
                segments.push(TranscriptionSegment {
                    start_time: seg.start,
                    end_time: seg.end,
                    text: seg.text.clone(),
                    speaker_id: if enable_speaker_diarization {
                        Some((i % 2) as u32) // 简单的说话人分离模拟
                    } else {
                        None
                    },
                    confidence: seg.avg_logprob.map(|p| p.exp() as f32).unwrap_or(0.9),
                });
            }

            // 如果启用说话人识别，生成说话人信息
            if enable_speaker_diarization {
                let mut speaker_durations: std::collections::HashMap<u32, f64> =
                    std::collections::HashMap::new();

                for seg in &segments {
                    if let Some(speaker_id) = seg.speaker_id {
                        *speaker_durations.entry(speaker_id).or_insert(0.0) +=
                            seg.end_time - seg.start_time;
                    }
                }

                for (id, duration) in speaker_durations {
                    speakers.push(Speaker {
                        id,
                        label: format!("说话人 {}", id + 1),
                        total_duration: duration,
                    });
                }
            }
        }

        Ok(AudioTranscriptionResponse {
            text: response.text,
            detected_language: response.language,
            segments,
            speakers,
            confidence: 0.9,
        })
    }
}

/// Whisper API 响应
#[derive(Debug, Deserialize)]
struct WhisperResponse {
    text: String,
    language: Option<String>,
    duration: Option<f64>,
    segments: Option<Vec<WhisperSegment>>,
}

#[derive(Debug, Deserialize)]
struct WhisperSegment {
    id: u32,
    start: f64,
    end: f64,
    text: String,
    tokens: Option<Vec<u32>>,
    temperature: Option<f64>,
    avg_logprob: Option<f64>,
    compression_ratio: Option<f64>,
    no_speech_prob: Option<f64>,
}

/// 说话人分离（使用 pyannote 或其他工具）
pub struct SpeakerDiarization {
    /// 是否启用
    enabled: bool,
}

impl SpeakerDiarization {
    /// 创建新的说话人分离器
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// 分离说话人
    pub async fn diarize(
        &self,
        audio_data: &[u8],
        segments: &[TranscriptionSegment],
    ) -> Result<Vec<TranscriptionSegment>> {
        if !self.enabled {
            return Ok(segments.to_vec());
        }

        // 在真实实现中，这里会调用 pyannote.audio 或其他说话人分离工具
        // 目前返回简单的模拟结果
        info!("执行说话人分离（模拟）");

        let mut diarized_segments = segments.to_vec();
        for (i, seg) in diarized_segments.iter_mut().enumerate() {
            // 简单的说话人分配：交替分配
            seg.speaker_id = Some((i % 2) as u32);
        }

        Ok(diarized_segments)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_format_mapping() {
        let client = OpenAIWhisperClient::new(AIModelConfig::openai("test_key".to_string()))
            .unwrap();

        assert_eq!(client.map_audio_format(&AudioFormat::Mp3), "mp3");
        assert_eq!(client.map_audio_format(&AudioFormat::Wav), "wav");
        assert_eq!(client.map_audio_format(&AudioFormat::Flac), "flac");
    }

    #[test]
    fn test_speaker_diarization_creation() {
        let diarization = SpeakerDiarization::new(true);
        assert!(diarization.enabled);

        let diarization = SpeakerDiarization::new(false);
        assert!(!diarization.enabled);
    }
}

