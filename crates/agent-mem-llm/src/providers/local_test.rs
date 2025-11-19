//! 本地测试 LLM 提供商实现
//!
//! 这个模块提供了一个本地测试 LLM 提供商，用于开发和测试环境。
//! 它实现了确定性的响应生成，不依赖外部 API 服务。

use agent_mem_traits::{AgentMemError, LLMConfig, LLMProvider, Message, ModelInfo, Result};
use async_trait::async_trait;
use futures::stream::{self};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;
use std::time::Duration;
use tracing::{debug, info};

/// 本地测试 LLM 提供商配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalTestConfig {
    /// 模型名称
    pub model_name: String,
    /// 响应延迟（毫秒）
    pub response_delay_ms: u64,
    /// 最大 token 数
    pub max_tokens: u32,
    /// 温度参数
    pub temperature: f32,
    /// 是否启用详细日志
    pub verbose: bool,
}

impl Default for LocalTestConfig {
    fn default() -> Self {
        Self {
            model_name: "local-test-model".to_string(),
            response_delay_ms: 100,
            max_tokens: 1000,
            temperature: 0.7,
            verbose: false,
        }
    }
}

/// 本地测试 LLM 提供商
pub struct LocalTestProvider {
    config: LLMConfig,
    local_config: LocalTestConfig,
    response_templates: HashMap<String, String>,
}

impl LocalTestProvider {
    /// 创建新的本地测试提供商实例
    pub fn new(config: LLMConfig) -> Result<Self> {
        let local_config = LocalTestConfig::default();
        let response_templates = Self::create_response_templates();

        info!("初始化本地测试 LLM 提供商: {}", local_config.model_name);

        Ok(Self {
            config,
            local_config,
            response_templates,
        })
    }

    /// 使用自定义配置创建提供商
    pub fn with_config(config: LLMConfig, local_config: LocalTestConfig) -> Result<Self> {
        let response_templates = Self::create_response_templates();

        info!(
            "初始化本地测试 LLM 提供商（自定义配置）: {}",
            local_config.model_name
        );

        Ok(Self {
            config,
            local_config,
            response_templates,
        })
    }

    /// 创建响应模板
    fn create_response_templates() -> HashMap<String, String> {
        let mut templates = HashMap::new();

        // 通用响应模板
        templates.insert(
            "default".to_string(),
            "这是一个来自本地测试 LLM 的响应。我理解了您的请求，并将尽力提供有用的信息。"
                .to_string(),
        );

        // 问候响应
        templates.insert(
            "greeting".to_string(),
            "您好！我是本地测试 LLM 助手。很高兴为您服务！有什么我可以帮助您的吗？".to_string(),
        );

        // 总结响应
        templates.insert(
            "summary".to_string(),
            "根据提供的信息，我可以总结如下要点：\n1. 主要内容已被理解和处理\n2. 相关信息已被提取和分析\n3. 响应已根据上下文生成".to_string(),
        );

        // 分析响应
        templates.insert(
            "analysis".to_string(),
            "经过分析，我发现以下几个关键点：\n• 输入内容具有明确的结构\n• 上下文信息丰富且相关\n• 可以提供有价值的洞察".to_string(),
        );

        // 错误处理响应
        templates.insert(
            "error".to_string(),
            "抱歉，在处理您的请求时遇到了一些问题。请检查输入格式或稍后重试。".to_string(),
        );

        templates
    }

    /// 根据消息内容选择合适的响应模板
    fn select_response_template(&self, messages: &[Message]) -> String {
        if messages.is_empty() {
            return self.response_templates.get("default").unwrap().clone();
        }

        let last_message = &messages[messages.len() - 1];
        let content = last_message.content.to_lowercase();

        // 简单的关键词匹配
        if content.contains("你好") || content.contains("hello") || content.contains("hi") {
            self.response_templates.get("greeting").unwrap().clone()
        } else if content.contains("总结") || content.contains("summary") {
            self.response_templates.get("summary").unwrap().clone()
        } else if content.contains("分析") || content.contains("analysis") {
            self.response_templates.get("analysis").unwrap().clone()
        } else {
            // 生成基于内容的动态响应
            format!(
                "我理解您提到了：「{}」。这是一个很有趣的话题。基于我的理解，我可以提供以下见解：\n\n1. 您的输入包含了 {} 个字符\n2. 消息类型：{:?}\n3. 这是第 {} 条消息\n\n如果您需要更具体的帮助，请告诉我更多详细信息。",
                if content.len() > 50 { &content[..50] } else { &content },
                content.len(),
                last_message.role,
                messages.len()
            )
        }
    }

    /// 生成模拟的使用统计
    fn generate_usage_stats(
        &self,
        input_tokens: usize,
        output_tokens: usize,
    ) -> HashMap<String, serde_json::Value> {
        let mut usage = HashMap::new();
        usage.insert(
            "prompt_tokens".to_string(),
            serde_json::Value::Number(input_tokens.into()),
        );
        usage.insert(
            "completion_tokens".to_string(),
            serde_json::Value::Number(output_tokens.into()),
        );
        usage.insert(
            "total_tokens".to_string(),
            serde_json::Value::Number((input_tokens + output_tokens).into()),
        );
        usage
    }

    /// 估算 token 数量（简单实现）
    fn estimate_tokens(&self, text: &str) -> usize {
        // 简单的 token 估算：大约每 4 个字符一个 token
        text.len().div_ceil(4)
    }
}

#[async_trait]
impl LLMProvider for LocalTestProvider {
    async fn generate(&self, messages: &[Message]) -> Result<String> {
        if self.local_config.verbose {
            debug!("本地测试 LLM 开始生成响应，消息数量: {}", messages.len());
        }

        // 模拟处理延迟
        if self.local_config.response_delay_ms > 0 {
            tokio::time::sleep(Duration::from_millis(self.local_config.response_delay_ms)).await;
        }

        // 验证输入
        if messages.is_empty() {
            return Err(AgentMemError::llm_error("消息列表不能为空"));
        }

        // 生成响应
        let response = self.select_response_template(messages);

        // 应用 max_tokens 限制
        let limited_response = if response.len() > self.local_config.max_tokens as usize {
            format!(
                "{}...",
                &response[..self.local_config.max_tokens as usize - 3]
            )
        } else {
            response
        };

        if self.local_config.verbose {
            debug!(
                "本地测试 LLM 生成响应完成，长度: {}",
                limited_response.len()
            );
        }

        Ok(limited_response)
    }

    async fn generate_stream(
        &self,
        messages: &[Message],
    ) -> Result<Pin<Box<dyn futures::Stream<Item = Result<String>> + Send>>> {
        // 简单实现：将完整响应分块流式返回
        let response = self.generate(messages).await?;
        let chunks: Vec<String> = response
            .chars()
            .collect::<Vec<char>>()
            .chunks(10)
            .map(|chunk| chunk.iter().collect())
            .collect();

        let stream = stream::iter(chunks.into_iter().map(Ok));
        Ok(Box::pin(stream))
    }

    fn get_model_info(&self) -> ModelInfo {
        ModelInfo {
            provider: "local_test".to_string(),
            model: self.local_config.model_name.clone(),
            max_tokens: self.local_config.max_tokens,
            supports_streaming: true,
            supports_functions: false,
        }
    }

    fn validate_config(&self) -> Result<()> {
        if self.local_config.model_name.is_empty() {
            return Err(AgentMemError::llm_error("模型名称不能为空"));
        }
        if self.local_config.max_tokens == 0 {
            return Err(AgentMemError::llm_error("max_tokens 必须大于 0"));
        }
        Ok(())
    }
}

impl LocalTestProvider {
    /// 生成带元数据的响应（辅助方法）
    pub async fn generate_with_metadata(
        &self,
        messages: &[Message],
    ) -> Result<(String, HashMap<String, serde_json::Value>)> {
        let response = self.generate(messages).await?;

        // 计算 token 使用量
        let input_tokens: usize = messages
            .iter()
            .map(|msg| self.estimate_tokens(&msg.content))
            .sum();
        let output_tokens = self.estimate_tokens(&response);

        // 生成元数据
        let mut metadata = HashMap::new();
        metadata.insert(
            "model".to_string(),
            serde_json::Value::String(self.local_config.model_name.clone()),
        );
        metadata.insert(
            "provider".to_string(),
            serde_json::Value::String("local_test".to_string()),
        );
        metadata.insert(
            "temperature".to_string(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(self.local_config.temperature as f64)
                    .unwrap_or_else(|| serde_json::Number::from(0)),
            ),
        );
        metadata.insert(
            "max_tokens".to_string(),
            serde_json::Value::Number(self.local_config.max_tokens.into()),
        );
        metadata.insert(
            "usage".to_string(),
            serde_json::Value::Object(
                self.generate_usage_stats(input_tokens, output_tokens)
                    .into_iter()
                    .collect(),
            ),
        );

        Ok((response, metadata))
    }

    /// 健康检查（辅助方法）
    pub async fn health_check(&self) -> Result<bool> {
        // 本地测试提供商总是健康的
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_mem_traits::MessageRole;

    #[tokio::test]
    async fn test_local_test_provider_creation() {
        let config = LLMConfig::default();
        let provider = LocalTestProvider::new(config).unwrap();

        let model_info = provider.get_model_info();
        assert_eq!(model_info.provider, "local_test");
        assert_eq!(model_info.model, "local-test-model");
    }

    #[tokio::test]
    async fn test_generate_response() {
        let config = LLMConfig::default();
        let provider = LocalTestProvider::new(config).unwrap();

        let messages = vec![Message {
            role: MessageRole::User,
            content: "你好".to_string(),
            timestamp: None,
        }];

        let response = provider.generate(&messages).await.unwrap();
        assert!(!response.is_empty());
        assert!(response.contains("您好"));
    }

    #[tokio::test]
    async fn test_generate_with_metadata() {
        let config = LLMConfig::default();
        let provider = LocalTestProvider::new(config).unwrap();

        let messages = vec![Message {
            role: MessageRole::User,
            content: "请分析这个问题".to_string(),
            timestamp: None,
        }];

        let (response, metadata) = provider.generate_with_metadata(&messages).await.unwrap();
        assert!(!response.is_empty());
        assert!(metadata.contains_key("model"));
        assert!(metadata.contains_key("usage"));
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = LLMConfig::default();
        let provider = LocalTestProvider::new(config).unwrap();

        let is_healthy = provider.health_check().await.unwrap();
        assert!(is_healthy);
    }

    #[tokio::test]
    async fn test_empty_messages_error() {
        let config = LLMConfig::default();
        let provider = LocalTestProvider::new(config).unwrap();

        let result = provider.generate(&[]).await;
        assert!(result.is_err());
    }
}
