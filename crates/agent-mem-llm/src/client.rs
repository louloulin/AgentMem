//! LLM客户端实现

use crate::{prompts::PromptManager, LLMFactory};
use agent_mem_traits::{
    llm::{FunctionCallResponse, FunctionDefinition},
    LLMConfig, LLMProvider, Message, ModelInfo, Result,
};
use std::sync::Arc;
use std::time::Duration;

/// LLM客户端配置
#[derive(Debug, Clone)]
pub struct LLMClientConfig {
    /// 重试次数
    pub max_retries: u32,
    /// 重试间隔（秒）
    pub retry_delay_seconds: u64,
    /// 请求超时（秒）
    pub timeout_seconds: u64,
    /// 是否启用日志
    pub enable_logging: bool,
}

impl Default for LLMClientConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay_seconds: 1,
            timeout_seconds: 30,
            enable_logging: false,
        }
    }
}

/// LLM客户端，提供高级的LLM交互功能
pub struct LLMClient {
    provider: Arc<dyn LLMProvider + Send + Sync>,
    prompt_manager: PromptManager,
    config: LLMClientConfig,
}

impl LLMClient {
    /// 创建新的LLM客户端
    pub fn new(llm_config: &LLMConfig) -> Result<Self> {
        let provider = LLMFactory::create_provider(llm_config)?;
        let prompt_manager = PromptManager::new();
        let config = LLMClientConfig::default();

        Ok(Self {
            provider,
            prompt_manager,
            config,
        })
    }

    /// 使用自定义配置创建LLM客户端
    pub fn with_config(llm_config: &LLMConfig, client_config: LLMClientConfig) -> Result<Self> {
        let provider = LLMFactory::create_provider(llm_config)?;
        let prompt_manager = PromptManager::new();

        Ok(Self {
            provider,
            prompt_manager,
            config: client_config,
        })
    }

    /// 使用自定义提示词管理器创建LLM客户端
    pub fn with_prompt_manager(
        llm_config: &LLMConfig,
        prompt_manager: PromptManager,
    ) -> Result<Self> {
        let provider = LLMFactory::create_provider(llm_config)?;
        let config = LLMClientConfig::default();

        Ok(Self {
            provider,
            prompt_manager,
            config,
        })
    }

    /// 生成文本响应
    pub async fn generate(&self, messages: &[Message]) -> Result<String> {
        self.generate_with_retry(messages, self.config.max_retries)
            .await
    }

    /// 带函数调用的生成响应
    pub async fn generate_with_functions(
        &self,
        messages: &[Message],
        functions: &[FunctionDefinition],
    ) -> Result<FunctionCallResponse> {
        self.provider
            .generate_with_functions(messages, functions)
            .await
    }

    /// 生成流式响应
    pub async fn generate_stream(
        &self,
        messages: &[Message],
    ) -> Result<Box<dyn futures::Stream<Item = Result<String>> + Send + Unpin>> {
        self.provider.generate_stream(messages).await
    }

    /// 带重试的生成文本响应
    fn generate_with_retry<'a>(
        &'a self,
        messages: &'a [Message],
        retries_left: u32,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String>> + Send + 'a>> {
        Box::pin(async move {
            match self.provider.generate(messages).await {
                Ok(response) => {
                    if self.config.enable_logging {
                        println!("LLM Response: {}", response);
                    }
                    Ok(response)
                }
                Err(e) => {
                    if retries_left > 0 {
                        if self.config.enable_logging {
                            println!(
                                "LLM request failed, retrying... ({} retries left): {}",
                                retries_left, e
                            );
                        }

                        // 等待重试间隔
                        tokio::time::sleep(Duration::from_secs(self.config.retry_delay_seconds))
                            .await;

                        self.generate_with_retry(messages, retries_left - 1).await
                    } else {
                        Err(e)
                    }
                }
            }
        })
    }

    /// 提取记忆
    pub async fn extract_memories(&self, conversation: &str) -> Result<String> {
        let messages = self
            .prompt_manager
            .build_memory_extraction_prompt(conversation)?;
        self.generate(&messages).await
    }

    /// 摘要记忆
    pub async fn summarize_memories(&self, memories: &str) -> Result<String> {
        let messages = self
            .prompt_manager
            .build_memory_summarization_prompt(memories)?;
        self.generate(&messages).await
    }

    /// 检测记忆冲突
    pub async fn detect_memory_conflicts(&self, memory_a: &str, memory_b: &str) -> Result<String> {
        let messages = self
            .prompt_manager
            .build_memory_conflict_detection_prompt(memory_a, memory_b)?;
        self.generate(&messages).await
    }

    /// 评估记忆重要性
    pub async fn score_memory_importance(
        &self,
        memory_content: &str,
        user_context: &str,
    ) -> Result<String> {
        let messages = self
            .prompt_manager
            .build_memory_importance_scoring_prompt(memory_content, user_context)?;
        self.generate(&messages).await
    }

    /// 增强查询
    pub async fn enhance_query(&self, user_query: &str) -> Result<String> {
        let messages = self
            .prompt_manager
            .build_memory_query_enhancement_prompt(user_query)?;
        self.generate(&messages).await
    }

    /// 自定义提示词生成
    pub async fn generate_with_template(
        &self,
        template_name: &str,
        variables: std::collections::HashMap<String, String>,
    ) -> Result<String> {
        let messages = self
            .prompt_manager
            .build_custom_prompt(template_name, variables)?;
        self.generate(&messages).await
    }

    /// 简单的文本生成
    pub async fn generate_simple(&self, prompt: &str) -> Result<String> {
        let messages = vec![Message {
            role: agent_mem_traits::MessageRole::User,
            content: prompt.to_string(),
            timestamp: None,
        }];
        self.generate(&messages).await
    }

    /// 带系统提示的文本生成
    pub async fn generate_with_system(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String> {
        let messages = self
            .prompt_manager
            .build_system_prompt(system_prompt, user_prompt);
        self.generate(&messages).await
    }

    /// 获取模型信息
    pub fn get_model_info(&self) -> ModelInfo {
        self.provider.get_model_info()
    }

    /// 验证配置
    pub fn validate_config(&self) -> Result<()> {
        self.provider.validate_config()
    }

    /// 获取提示词管理器的引用
    pub fn prompt_manager(&self) -> &PromptManager {
        &self.prompt_manager
    }

    /// 获取可变的提示词管理器引用
    pub fn prompt_manager_mut(&mut self) -> &mut PromptManager {
        &mut self.prompt_manager
    }

    /// 设置客户端配置
    pub fn set_config(&mut self, config: LLMClientConfig) {
        self.config = config;
    }

    /// 获取客户端配置
    pub fn get_config(&self) -> &LLMClientConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> LLMConfig {
        LLMConfig {
            provider: "openai".to_string(),
            model: "gpt-3.5-turbo".to_string(),
            api_key: Some("test-key".to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn test_llm_client_creation() {
        let config = create_test_config();
        let client = LLMClient::new(&config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_llm_client_with_config() {
        let llm_config = create_test_config();
        let client_config = LLMClientConfig {
            max_retries: 5,
            retry_delay_seconds: 2,
            timeout_seconds: 60,
            enable_logging: true,
        };

        let client = LLMClient::with_config(&llm_config, client_config);
        assert!(client.is_ok());

        let client = client.unwrap();
        assert_eq!(client.config.max_retries, 5);
        assert_eq!(client.config.retry_delay_seconds, 2);
    }

    #[test]
    fn test_get_model_info() {
        let config = create_test_config();
        let client = LLMClient::new(&config).unwrap();

        let model_info = client.get_model_info();
        assert_eq!(model_info.provider, "openai");
        assert_eq!(model_info.model, "gpt-3.5-turbo");
    }

    #[test]
    fn test_validate_config() {
        let config = create_test_config();
        let client = LLMClient::new(&config).unwrap();

        let result = client.validate_config();
        assert!(result.is_ok());
    }

    #[test]
    fn test_prompt_manager_access() {
        let config = create_test_config();
        let mut client = LLMClient::new(&config).unwrap();

        // 测试只读访问
        let templates = client.prompt_manager().get_available_templates();
        assert!(!templates.is_empty());

        // 测试可变访问
        client
            .prompt_manager_mut()
            .add_template("test".to_string(), "Test template: {variable}".to_string());

        let templates = client.prompt_manager().get_available_templates();
        assert!(templates.contains(&&"test".to_string()));
    }

    #[tokio::test]
    async fn test_extract_memories_prompt_building() {
        let config = create_test_config();
        let client = LLMClient::new(&config).unwrap();

        // 这个测试只验证提示词构建，不实际调用LLM
        let messages = client
            .prompt_manager
            .build_memory_extraction_prompt("Hello, I love tennis!");
        assert!(messages.is_ok());

        let messages = messages.unwrap();
        assert_eq!(messages.len(), 1);
        assert!(messages[0].content.contains("Hello, I love tennis!"));
    }
}
