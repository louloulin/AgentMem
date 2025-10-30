//! OpenAI Vision API 集成
//!
//! 使用 GPT-4 Vision 进行图像分析

use super::ai_models::*;
use agent_mem_traits::{AgentMemError, Result};
use serde::Deserialize;
use serde_json::json;
use tracing::{debug, error, info};

#[cfg(feature = "multimodal")]
use reqwest::Client;

/// OpenAI Vision 客户端
pub struct OpenAIVisionClient {
    /// HTTP 客户端
    client: Client,
    /// 配置
    config: AIModelConfig,
}

impl OpenAIVisionClient {
    /// 创建新的 OpenAI Vision 客户端
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

    /// 分析图像
    pub async fn analyze_image(
        &self,
        request: &ImageAnalysisRequest,
    ) -> Result<ImageAnalysisResponse> {
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

        // 构建提示词
        let prompt = self.build_analysis_prompt(&request.tasks);

        // 构建请求体
        let image_content = if let Some(url) = &request.image_url {
            json!({
                "type": "image_url",
                "image_url": {
                    "url": url,
                    "detail": self.map_detail_level(&request.detail_level)
                }
            })
        } else {
            json!({
                "type": "image_url",
                "image_url": {
                    "url": format!("data:image/jpeg;base64,{}", request.image_data),
                    "detail": self.map_detail_level(&request.detail_level)
                }
            })
        };

        let body = json!({
            "model": self.config.model_name.as_ref().unwrap_or(&"gpt-4-vision-preview".to_string()),
            "messages": [
                {
                    "role": "user",
                    "content": [
                        {
                            "type": "text",
                            "text": prompt
                        },
                        image_content
                    ]
                }
            ],
            "max_tokens": request.max_tokens.unwrap_or(1000)
        });

        info!("发送图像分析请求到 OpenAI Vision API");
        debug!("请求体: {}", serde_json::to_string_pretty(&body).unwrap());

        // 发送请求
        let response = self
            .client
            .post(format!("{base_url}/chat/completions"))
            .header("Authorization", format!("Bearer {api_key}"))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| AgentMemError::internal_error(format!("发送请求失败: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "无法读取错误响应".to_string());
            error!("OpenAI API 错误 ({}): {}", status, error_text);
            return Err(AgentMemError::internal_error(format!(
                "OpenAI API 错误 ({status}): {error_text}"
            )));
        }

        let api_response: OpenAIVisionResponse = response
            .json()
            .await
            .map_err(|e| AgentMemError::internal_error(format!("解析响应失败: {e}")))?;

        debug!("OpenAI Vision API 响应: {:?}", api_response);

        // 解析响应
        self.parse_response(&api_response, &request.tasks)
    }

    /// 构建分析提示词
    fn build_analysis_prompt(&self, tasks: &[ImageAnalysisTask]) -> String {
        let mut prompt = String::from("请分析这张图像，并提供以下信息：\n\n");

        for task in tasks {
            match task {
                ImageAnalysisTask::SceneDescription => {
                    prompt.push_str("1. 场景描述：详细描述图像中的场景、环境和整体氛围\n");
                }
                ImageAnalysisTask::ObjectDetection => {
                    prompt.push_str("2. 对象检测：列出图像中的所有主要对象及其位置\n");
                }
                ImageAnalysisTask::TextRecognition => {
                    prompt.push_str("3. 文本识别：提取图像中的所有可见文本\n");
                }
                ImageAnalysisTask::FaceDetection => {
                    prompt.push_str("4. 人脸检测：识别图像中的人脸及其特征\n");
                }
                ImageAnalysisTask::LabelExtraction => {
                    prompt.push_str("5. 标签提取：为图像生成相关的标签和关键词\n");
                }
                ImageAnalysisTask::ColorAnalysis => {
                    prompt.push_str("6. 颜色分析：分析图像的主要颜色和色调\n");
                }
            }
        }

        prompt.push_str("\n请以结构化的 JSON 格式返回结果。");
        prompt
    }

    /// 映射详细程度
    fn map_detail_level(&self, level: &DetailLevel) -> &str {
        match level {
            DetailLevel::Low => "low",
            DetailLevel::High => "high",
            DetailLevel::Medium | DetailLevel::Auto => "auto",
        }
    }

    /// 解析响应
    fn parse_response(
        &self,
        response: &OpenAIVisionResponse,
        tasks: &[ImageAnalysisTask],
    ) -> Result<ImageAnalysisResponse> {
        let content = response
            .choices
            .first()
            .and_then(|c| c.message.content.as_ref())
            .ok_or_else(|| AgentMemError::internal_error("响应中没有内容".to_string()))?;

        info!("OpenAI Vision 分析结果: {}", content);

        // 尝试解析 JSON 响应
        if let Ok(json_response) = serde_json::from_str::<serde_json::Value>(content) {
            self.parse_json_response(&json_response, tasks)
        } else {
            // 如果不是 JSON，则解析文本响应
            self.parse_text_response(content, tasks)
        }
    }

    /// 解析 JSON 响应
    fn parse_json_response(
        &self,
        json: &serde_json::Value,
        tasks: &[ImageAnalysisTask],
    ) -> Result<ImageAnalysisResponse> {
        let mut response = ImageAnalysisResponse {
            scene_description: None,
            objects: Vec::new(),
            text: None,
            faces: Vec::new(),
            labels: Vec::new(),
            dominant_colors: Vec::new(),
            confidence: 0.9,
        };

        if tasks.contains(&ImageAnalysisTask::SceneDescription) {
            response.scene_description = json
                .get("scene_description")
                .or_else(|| json.get("description"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
        }

        if tasks.contains(&ImageAnalysisTask::ObjectDetection) {
            if let Some(objects) = json.get("objects").and_then(|v| v.as_array()) {
                for obj in objects {
                    if let Some(name) = obj.get("name").and_then(|v| v.as_str()) {
                        response.objects.push(DetectedObject {
                            name: name.to_string(),
                            confidence: obj
                                .get("confidence")
                                .and_then(|v| v.as_f64())
                                .unwrap_or(0.8) as f32,
                            bounding_box: None,
                        });
                    }
                }
            }
        }

        if tasks.contains(&ImageAnalysisTask::TextRecognition) {
            response.text = json
                .get("text")
                .or_else(|| json.get("ocr_text"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
        }

        if tasks.contains(&ImageAnalysisTask::LabelExtraction) {
            if let Some(labels) = json.get("labels").and_then(|v| v.as_array()) {
                for label in labels {
                    if let Some(name) = label.as_str().or_else(|| label.get("name").and_then(|v| v.as_str())) {
                        response.labels.push(Label {
                            name: name.to_string(),
                            confidence: 0.85,
                            category: None,
                        });
                    }
                }
            }
        }

        Ok(response)
    }

    /// 解析文本响应
    fn parse_text_response(
        &self,
        text: &str,
        tasks: &[ImageAnalysisTask],
    ) -> Result<ImageAnalysisResponse> {
        let mut response = ImageAnalysisResponse {
            scene_description: None,
            objects: Vec::new(),
            text: None,
            faces: Vec::new(),
            labels: Vec::new(),
            dominant_colors: Vec::new(),
            confidence: 0.85,
        };

        // 如果需要场景描述，使用整个文本
        if tasks.contains(&ImageAnalysisTask::SceneDescription) {
            response.scene_description = Some(text.to_string());
        }

        // 简单的文本解析逻辑
        // 在实际应用中，可以使用更复杂的 NLP 技术

        Ok(response)
    }
}

/// OpenAI Vision API 响应
#[derive(Debug, Deserialize)]
struct OpenAIVisionResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: MessageContent,
}

#[derive(Debug, Deserialize)]
struct MessageContent {
    content: Option<String>,
}

