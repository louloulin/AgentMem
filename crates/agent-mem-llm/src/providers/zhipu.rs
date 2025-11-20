//! Zhipu AI (智谱AI) LLM提供商实现
//! 支持 GLM-4, glm-4.6, GLM-4-Air 等模型

use agent_mem_traits::llm::{FunctionCall, FunctionCallResponse, FunctionDefinition};
use agent_mem_traits::{
    AgentMemError, LLMConfig, LLMProvider, Message, MessageRole, ModelInfo, Result,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Zhipu AI API请求结构
#[derive(Debug, Serialize)]
struct ZhipuRequest {
    model: String,
    messages: Vec<ZhipuMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ZhipuTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<String>,
}

/// Zhipu消息格式
#[derive(Debug, Serialize, Deserialize)]
struct ZhipuMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<ZhipuToolCall>>,
}

/// Zhipu工具调用
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ZhipuToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: ZhipuFunctionCall,
}

/// Zhipu API响应结构
#[derive(Debug, Deserialize)]
struct ZhipuResponse {
    id: String,
    created: u64,
    model: String,
    choices: Vec<ZhipuChoice>,
    usage: ZhipuUsage,
}

/// Zhipu选择结构
#[derive(Debug, Deserialize)]
struct ZhipuChoice {
    index: u32,
    message: ZhipuMessage,
    finish_reason: Option<String>,
}

/// Zhipu使用统计
#[derive(Debug, Deserialize)]
struct ZhipuUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// Zhipu流式响应结构
#[derive(Debug, Deserialize)]
struct ZhipuStreamResponse {
    id: String,
    created: u64,
    model: String,
    choices: Vec<ZhipuStreamChoice>,
}

/// Zhipu流式选择结构
#[derive(Debug, Deserialize)]
struct ZhipuStreamChoice {
    index: u32,
    delta: ZhipuStreamDelta,
    finish_reason: Option<String>,
}

/// Zhipu流式增量数据
#[derive(Debug, Deserialize)]
struct ZhipuStreamDelta {
    #[serde(default)]
    role: Option<String>,
    #[serde(default)]
    content: Option<String>,
}

/// Zhipu函数定义
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ZhipuFunction {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Zhipu函数调用
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ZhipuFunctionCall {
    pub name: String,
    pub arguments: String,
}

/// Zhipu工具定义
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ZhipuTool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: ZhipuFunction,
}

/// Zhipu错误响应
#[derive(Debug, Deserialize)]
struct ZhipuErrorResponse {
    error: ZhipuErrorDetail,
}

#[derive(Debug, Deserialize)]
struct ZhipuErrorDetail {
    message: String,
    #[serde(rename = "type")]
    error_type: String,
    code: Option<String>,
}

/// Zhipu AI LLM提供商
pub struct ZhipuProvider {
    config: LLMConfig,
    client: Client,
}

impl ZhipuProvider {
    /// 创建新的Zhipu提供商实例
    pub fn new(config: LLMConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .map_err(|e| AgentMemError::LLMError(format!("Failed to create HTTP client: {e}")))?;

        Ok(Self { config, client })
    }

    /// 将内部消息格式转换为Zhipu格式
    fn convert_messages(&self, messages: &[Message]) -> Vec<ZhipuMessage> {
        messages
            .iter()
            .map(|msg| {
                let role = match msg.role {
                    MessageRole::System => "system",
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                };

                ZhipuMessage {
                    role: role.to_string(),
                    content: msg.content.clone(),
                    tool_calls: None,
                }
            })
            .collect()
    }

    /// 将函数定义转换为Zhipu工具格式
    fn convert_tools(&self, functions: &[FunctionDefinition]) -> Vec<ZhipuTool> {
        functions
            .iter()
            .map(|func| ZhipuTool {
                tool_type: "function".to_string(),
                function: ZhipuFunction {
                    name: func.name.clone(),
                    description: func.description.clone(),
                    parameters: func.parameters.clone(),
                },
            })
            .collect()
    }
}

#[async_trait]
impl LLMProvider for ZhipuProvider {
    async fn generate(&self, messages: &[Message]) -> Result<String> {
        let start_time = std::time::Instant::now();

        let api_key = self.config.api_key.as_ref().ok_or_else(|| {
            AgentMemError::ConfigError("Zhipu API key not configured".to_string())
        })?;

        let base_url = self
            .config
            .base_url
            .as_deref()
            .unwrap_or("https://open.bigmodel.cn/api/paas/v4");

        let url = format!("{base_url}/chat/completions");

        info!("🔵 Zhipu API 请求开始");
        info!("   模型: {}", self.config.model);
        info!("   URL: {}", url);
        info!("   消息数量: {}", messages.len());

        // 🔍 详细记录每条消息的内容和长度 (UTF-8安全截断)
        for (idx, msg) in messages.iter().enumerate() {
            let content_preview = if msg.content.chars().count() > 200 {
                let truncated: String = msg.content.chars().take(200).collect();
                format!(
                    "{}... (总长度: {}字符)",
                    truncated,
                    msg.content.chars().count()
                )
            } else {
                msg.content.clone()
            };
            info!(
                "   📝 消息[{}] role={:?}, 长度={}字符, 内容=\"{}\"",
                idx,
                msg.role,
                msg.content.chars().count(),
                content_preview
            );
        }

        debug!("   消息内容（完整）: {:?}", messages);

        let converted_messages = self.convert_messages(messages);

        // 🔍 打印完整的prompt内容（所有消息合并）
        info!("📋 === 完整Prompt内容（所有消息） ===");
        let total_chars: usize = converted_messages.iter().map(|m| m.content.len()).sum();
        info!("   总字符数: {}", total_chars);

        // 合并所有消息内容
        let full_prompt: String = converted_messages
            .iter()
            .map(|m| format!("[{}] {}\n", m.role, m.content))
            .collect();
        info!("{}", full_prompt);
        info!("📋 === Prompt内容结束 ===");

        let request = ZhipuRequest {
            model: self.config.model.clone(),
            messages: converted_messages,
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            top_p: self.config.top_p,
            stream: Some(false),
            tools: None,
            tool_choice: None,
        };

        debug!(
            "   请求体JSON: {}",
            serde_json::to_string_pretty(&request).unwrap_or_default()
        );

        info!("🔵 发送 HTTP 请求...");
        let http_start = std::time::Instant::now();
        
        // 详细记录请求开始时间
        info!("   ⏱️  请求开始时间戳: {:?}", std::time::SystemTime::now());
        info!("   📦 请求体大小: {} bytes", serde_json::to_string(&request).unwrap_or_default().len());
        info!("   🌐 目标URL: {}", url);
        info!("   🔍 开始DNS解析和TCP连接...");

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {api_key}"))
            .header("Content-Type", "application/json")
            .json(&request)
            .timeout(std::time::Duration::from_secs(60)) // 添加60秒超时
            .send()
            .await
            .map_err(|e| {
                let elapsed = http_start.elapsed();
                warn!("❌ HTTP 请求失败，已耗时: {:?}", elapsed);
                warn!("   错误详情: {}", e);
                if e.is_timeout() {
                    warn!("   ⚠️  请求超时！");
                } else if e.is_connect() {
                    warn!("   ⚠️  连接失败！");
                } else if e.is_request() {
                    warn!("   ⚠️  请求构建失败！");
                }
                AgentMemError::LLMError(format!("Failed to send request: {e}"))
            })?;

        let http_duration = http_start.elapsed();
        info!("✅ HTTP 响应收到，总耗时: {:?}", http_duration);
        info!("   ⏱️  响应到达时间戳: {:?}", std::time::SystemTime::now());

        let status = response.status();
        info!("   HTTP 状态码: {}", status);
        
        // 记录响应头信息
        info!("   📊 响应头信息:");
        if let Some(content_length) = response.headers().get("content-length") {
            info!("      Content-Length: {:?}", content_length);
        }
        if let Some(content_type) = response.headers().get("content-type") {
            info!("      Content-Type: {:?}", content_type);
        }
        if let Some(server) = response.headers().get("server") {
            info!("      Server: {:?}", server);
        }
        if let Some(date) = response.headers().get("date") {
            info!("      Date: {:?}", date);
        }
        
        // 计算网络传输速度
        let response_size = response.content_length().unwrap_or(0);
        if response_size > 0 && http_duration.as_secs_f64() > 0.0 {
            let speed_kbps = (response_size as f64 / 1024.0) / http_duration.as_secs_f64();
            info!("   📈 传输速度: {:.2} KB/s", speed_kbps);
        }

        if !status.is_success() {
            warn!("❌ HTTP 状态码错误: {}", status);
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            warn!("   错误响应: {}", error_text);

            if let Ok(error_response) = serde_json::from_str::<ZhipuErrorResponse>(&error_text) {
                return Err(AgentMemError::LLMError(format!(
                    "Zhipu API error: {} (type: {})",
                    error_response.error.message, error_response.error.error_type
                )));
            }

            return Err(AgentMemError::LLMError(format!(
                "Zhipu API error: {error_text}"
            )));
        }

        info!("🔵 解析 JSON 响应...");
        let parse_start = std::time::Instant::now();
        
        // 先读取原始响应文本以便分析
        let response_text = response.text().await.map_err(|e| {
            warn!("❌ 读取响应体失败: {}", e);
            AgentMemError::LLMError(format!("Failed to read response body: {e}"))
        })?;
        
        let body_read_duration = parse_start.elapsed();
        info!("   📥 响应体读取完成，耗时: {:?}, 大小: {} bytes", body_read_duration, response_text.len());

        let zhipu_response: ZhipuResponse = serde_json::from_str(&response_text).map_err(|e| {
            warn!("❌ JSON 解析失败: {}", e);
            warn!("   响应文本前500字符: {}", &response_text.chars().take(500).collect::<String>());
            AgentMemError::LLMError(format!("Failed to parse Zhipu response: {e}"))
        })?;

        let parse_duration = parse_start.elapsed();
        info!("✅ JSON 解析完成，总耗时: {:?}", parse_duration);

        let total_duration = start_time.elapsed();
        info!("✅ Zhipu API 调用完成，总耗时: {:?}", total_duration);
        
        // 详细的时间分解
        info!("   ⏱️  时间分解:");
        info!("      - HTTP等待: {:?} ({:.1}%)", http_duration, (http_duration.as_secs_f64() / total_duration.as_secs_f64()) * 100.0);
        info!("      - JSON解析: {:?} ({:.1}%)", parse_duration, (parse_duration.as_secs_f64() / total_duration.as_secs_f64()) * 100.0);
        
        info!(
            "   📊 Token 使用: prompt={}, completion={}, total={}",
            zhipu_response.usage.prompt_tokens,
            zhipu_response.usage.completion_tokens,
            zhipu_response.usage.total_tokens
        );
        
        // 计算生成速度
        let tokens_per_second = zhipu_response.usage.completion_tokens as f64 / http_duration.as_secs_f64();
        info!("   ⚡ 生成速度: {:.2} tokens/s", tokens_per_second);
        
        // 如果速度异常慢，给出警告
        if tokens_per_second < 10.0 {
            warn!("   ⚠️  生成速度异常慢！正常应该 >20 tokens/s");
        }

        let result = zhipu_response
            .choices
            .first()
            .map(|choice| {
                let content = choice.message.content.clone();
                info!("   响应长度: {} 字符", content.len());
                debug!("   响应内容: {}", content);
                content
            })
            .ok_or_else(|| {
                warn!("❌ Zhipu 响应中没有内容");
                AgentMemError::LLMError("No response from Zhipu".to_string())
            })?;

        Ok(result)
    }

    async fn generate_with_functions(
        &self,
        messages: &[Message],
        functions: &[FunctionDefinition],
    ) -> Result<FunctionCallResponse> {
        let api_key = self.config.api_key.as_ref().ok_or_else(|| {
            AgentMemError::ConfigError("Zhipu API key not configured".to_string())
        })?;

        let base_url = self
            .config
            .base_url
            .as_deref()
            .unwrap_or("https://open.bigmodel.cn/api/paas/v4");

        let url = format!("{base_url}/chat/completions");

        let tools = self.convert_tools(functions);

        let request = ZhipuRequest {
            model: self.config.model.clone(),
            messages: self.convert_messages(messages),
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            top_p: self.config.top_p,
            stream: Some(false),
            tools: Some(tools),
            tool_choice: Some("auto".to_string()),
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {api_key}"))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AgentMemError::LLMError(format!("Failed to send request: {e}")))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AgentMemError::LLMError(format!(
                "Zhipu API error: {error_text}"
            )));
        }

        let zhipu_response: ZhipuResponse = response
            .json()
            .await
            .map_err(|e| AgentMemError::LLMError(format!("Failed to parse Zhipu response: {e}")))?;

        let choice = zhipu_response
            .choices
            .first()
            .ok_or_else(|| AgentMemError::LLMError("No response from Zhipu".to_string()))?;

        let mut function_calls = Vec::new();

        if let Some(tool_calls) = &choice.message.tool_calls {
            for tool_call in tool_calls {
                function_calls.push(FunctionCall {
                    name: tool_call.function.name.clone(),
                    arguments: tool_call.function.arguments.clone(),
                });
            }
        }

        Ok(FunctionCallResponse {
            text: if function_calls.is_empty() {
                Some(choice.message.content.clone())
            } else {
                None
            },
            function_calls,
        })
    }

    async fn generate_stream(
        &self,
        messages: &[Message],
    ) -> Result<Pin<Box<dyn futures::Stream<Item = Result<String>> + Send>>> {
        use futures::stream::StreamExt;

        let api_key = self.config.api_key.as_ref().ok_or_else(|| {
            AgentMemError::ConfigError("Zhipu API key not configured".to_string())
        })?;

        let base_url = self
            .config
            .base_url
            .as_deref()
            .unwrap_or("https://open.bigmodel.cn/api/paas/v4");

        let url = format!("{base_url}/chat/completions");

        info!("🌊 Zhipu 真实流式请求开始");
        info!("   模型: {}", self.config.model);
        info!("   URL: {}", url);
        info!("   消息数量: {}", messages.len());

        let converted_messages = self.convert_messages(messages);

        let request = ZhipuRequest {
            model: self.config.model.clone(),
            messages: converted_messages,
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            top_p: self.config.top_p,
            stream: Some(true), // ✅ 启用真实流式
            tools: None,
            tool_choice: None,
        };

        info!("🔵 发送流式HTTP请求...");
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {api_key}"))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                warn!("❌ 流式HTTP请求失败: {}", e);
                AgentMemError::LLMError(format!("Failed to send streaming request: {e}"))
            })?;

        let status = response.status();
        info!("   HTTP 状态码: {}", status);

        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            warn!("❌ 流式请求失败: {}", error_text);
            return Err(AgentMemError::LLMError(format!(
                "Zhipu streaming API error: {error_text}"
            )));
        }

        info!("✅ 开始接收SSE流式数据...");

        // ✅ 创建真实的SSE流式处理器
        let stream = response
            .bytes_stream()
            .map(|chunk_result| {
                match chunk_result {
                    Ok(chunk) => {
                        // 解析SSE格式的数据
                        let chunk_str = String::from_utf8_lossy(&chunk);
                        let mut content_parts = Vec::new();

                        // SSE格式：data: {...}\n\n
                        for line in chunk_str.lines() {
                            let line = line.trim();
                            
                            // 跳过空行和注释
                            if line.is_empty() || line.starts_with(':') {
                                continue;
                            }

                            // 解析 data: 行
                            if let Some(data) = line.strip_prefix("data: ") {
                                let data = data.trim();
                                
                                // 检查是否是结束标记
                                if data == "[DONE]" {
                                    info!("✅ SSE流式数据传输完成");
                                    break;
                                }

                                // 解析JSON
                                match serde_json::from_str::<ZhipuStreamResponse>(data) {
                                    Ok(stream_resp) => {
                                        if let Some(choice) = stream_resp.choices.first() {
                                            if let Some(content) = &choice.delta.content {
                                                if !content.is_empty() {
                                                    debug!("   📦 收到内容块: {}", content);
                                                    content_parts.push(content.clone());
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        debug!("⚠️  解析流式响应失败 (非关键): {}, 数据: {}", e, data);
                                        // 非关键错误，继续处理下一块
                                    }
                                }
                            }
                        }

                        if content_parts.is_empty() {
                            Ok("".to_string())
                        } else {
                            let joined = content_parts.join("");
                            debug!("   ✅ 返回内容块: {} 字符", joined.len());
                            Ok(joined)
                        }
                    }
                    Err(e) => {
                        warn!("❌ 流式数据块接收失败: {}", e);
                        Err(AgentMemError::LLMError(format!("Stream chunk error: {e}")))
                    }
                }
            })
            .filter(|result| {
                // 过滤掉空字符串
                futures::future::ready(match result {
                    Ok(s) => !s.is_empty(),
                    Err(_) => true,
                })
            });

        Ok(Box::pin(stream))
    }

    fn get_model_info(&self) -> ModelInfo {
        ModelInfo {
            provider: "zhipu".to_string(),
            model: self.config.model.clone(),
            max_tokens: self.config.max_tokens.unwrap_or(8192),
            supports_functions: true,
            supports_streaming: false,
        }
    }

    fn validate_config(&self) -> Result<()> {
        if self.config.api_key.is_none() {
            return Err(AgentMemError::ConfigError(
                "Zhipu API key is required".to_string(),
            ));
        }
        Ok(())
    }
}
