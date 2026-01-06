//! Groq 提供商测试

#[cfg(test)]
mod tests {
    use super::super::groq::{
        GroqChoice, GroqMessage, GroqProvider, GroqRequest, GroqResponse, GroqUsage,
    };
    use agent_mem_traits::{LLMConfig, LLMProvider, Message};

    fn create_test_config() -> LLMConfig {
        LLMConfig {
            provider: "groq".to_string(),
            model: "llama3-8b-8192".to_string(),
            api_key: Some("test-api-key".to_string()),
            base_url: Some("https://api.groq.com/openai/v1".to_string()),
            temperature: Some(0.7),
            max_tokens: Some(1000),
            top_p: Some(0.9),
            frequency_penalty: None,
            presence_penalty: None,
            response_format: None,
        }
    }

    fn create_test_messages() -> Vec<Message> {
        vec![
            Message::system("You are a helpful assistant."),
            Message::user("Hello, how are you?"),
        ]
    }

    #[test]
    fn test_groq_provider_creation() {
        let config = create_test_config();
        let provider = GroqProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_groq_provider_creation_missing_api_key() {
        let mut config = create_test_config();
        config.api_key = None;

        let provider = GroqProvider::new(config);
        assert!(provider.is_err());
        assert!(provider
            .unwrap_err()
            .to_string()
            .contains("API key is required"));
    }

    #[test]
    fn test_groq_provider_creation_empty_model() {
        let mut config = create_test_config();
        config.model = "".to_string();

        let provider = GroqProvider::new(config);
        assert!(provider.is_err());
        assert!(provider
            .unwrap_err()
            .to_string()
            .contains("Model name is required"));
    }

    #[test]
    fn test_build_api_url() {
        let config = create_test_config();
        let provider = GroqProvider::new(config).unwrap();

        let url = provider.build_api_url();
        assert_eq!(url, "https://api.groq.com/openai/v1/chat/completions");
    }

    #[test]
    fn test_build_api_url_with_trailing_slash() {
        let mut config = create_test_config();
        config.base_url = Some("https://api.groq.com/openai/v1/".to_string());
        let provider = GroqProvider::new(config).unwrap();

        let url = provider.build_api_url();
        assert_eq!(url, "https://api.groq.com/openai/v1/chat/completions");
    }

    #[test]
    fn test_convert_messages() {
        let config = create_test_config();
        let provider = GroqProvider::new(config).unwrap();
        let messages = create_test_messages();

        let groq_messages = provider.convert_messages(&messages);

        assert_eq!(groq_messages.len(), 2);

        // 第一条消息（System）
        assert_eq!(groq_messages[0].role, "system");
        assert_eq!(groq_messages[0].content, "You are a helpful assistant.");

        // 第二条消息（User）
        assert_eq!(groq_messages[1].role, "user");
        assert_eq!(groq_messages[1].content, "Hello, how are you?");
    }

    #[test]
    fn test_convert_messages_all_roles() {
        let config = create_test_config();
        let provider = GroqProvider::new(config).unwrap();

        let messages = vec![
            Message::system("System message"),
            Message::user("User message"),
            Message::assistant("Assistant message"),
        ];

        let groq_messages = provider.convert_messages(&messages);

        assert_eq!(groq_messages.len(), 3);
        assert_eq!(groq_messages[0].role, "system");
        assert_eq!(groq_messages[1].role, "user");
        assert_eq!(groq_messages[2].role, "assistant");
    }

    #[test]
    fn test_extract_response_text() {
        let config = create_test_config();
        let provider = GroqProvider::new(config).unwrap();

        // 创建真实响应结构用于测试
        let response = GroqResponse {
            id: "chatcmpl-test".to_string(),
            object: "chat.completion".to_string(),
            created: 1234567890,
            model: "llama3-8b-8192".to_string(),
            choices: vec![GroqChoice {
                index: 0,
                message: GroqMessage {
                    role: "assistant".to_string(),
                    content: "Hello! I'm doing well, thank you for asking.".to_string(),
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage: Some(GroqUsage {
                prompt_tokens: 20,
                completion_tokens: 15,
                total_tokens: 35,
            }),
        };

        let result = provider.extract_response_text(&response);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "Hello! I'm doing well, thank you for asking."
        );
    }

    #[test]
    fn test_extract_response_text_empty_choices() {
        let config = create_test_config();
        let provider = GroqProvider::new(config).unwrap();

        let response = GroqResponse {
            id: "chatcmpl-test".to_string(),
            object: "chat.completion".to_string(),
            created: 1234567890,
            model: "llama3-8b-8192".to_string(),
            choices: vec![],
            usage: None,
        };

        let result = provider.extract_response_text(&response);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No choices in response"));
    }

    #[test]
    fn test_extract_response_text_length_finish() {
        let config = create_test_config();
        let provider = GroqProvider::new(config).unwrap();

        let response = GroqResponse {
            id: "chatcmpl-test".to_string(),
            object: "chat.completion".to_string(),
            created: 1234567890,
            model: "llama3-8b-8192".to_string(),
            choices: vec![GroqChoice {
                index: 0,
                message: GroqMessage {
                    role: "assistant".to_string(),
                    content: "This is a truncated response...".to_string(),
                },
                finish_reason: Some("length".to_string()),
            }],
            usage: None,
        };

        let result = provider.extract_response_text(&response);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "This is a truncated response...");
    }

    #[test]
    fn test_groq_request_serialization() {
        let request = GroqRequest {
            messages: vec![GroqMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            model: "llama3-8b-8192".to_string(),
            max_tokens: Some(1000),
            temperature: Some(0.7),
            top_p: Some(0.9),
            stop: None,
            stream: Some(false),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"messages\""));
        assert!(json.contains("\"model\""));
        assert!(json.contains("\"max_tokens\""));
        assert!(json.contains("\"temperature\""));
        assert!(json.contains("\"top_p\""));
        assert!(json.contains("\"stream\""));
    }

    #[test]
    fn test_supported_models() {
        let models = GroqProvider::supported_models();
        assert!(models.contains(&"llama3-8b-8192"));
        assert!(models.contains(&"llama3-70b-8192"));
        assert!(models.contains(&"mixtral-8x7b-32768"));
        assert!(models.contains(&"gemma-7b-it"));
        assert!(models.contains(&"gemma2-9b-it"));
    }

    #[test]
    fn test_is_model_supported() {
        let config = create_test_config();
        let provider = GroqProvider::new(config).unwrap();

        assert!(provider.is_model_supported());

        // 测试不支持的模型
        let mut config = create_test_config();
        config.model = "unsupported-model".to_string();
        let provider = GroqProvider::new(config).unwrap();

        assert!(!provider.is_model_supported());
    }

    #[test]
    fn test_model_info_llama3_8b() {
        let config = create_test_config();
        let provider = GroqProvider::new(config).unwrap();

        let model_info = provider.get_model_info();
        assert_eq!(model_info.model, "llama3-8b-8192");
        assert_eq!(model_info.provider, "groq");
        assert_eq!(model_info.max_tokens, 8_192);
        assert!(!model_info.supports_streaming);
        assert!(!model_info.supports_functions);
    }

    #[test]
    fn test_model_info_mixtral() {
        let mut config = create_test_config();
        config.model = "mixtral-8x7b-32768".to_string();
        let provider = GroqProvider::new(config).unwrap();

        let model_info = provider.get_model_info();
        assert_eq!(model_info.model, "mixtral-8x7b-32768");
        assert_eq!(model_info.provider, "groq");
        assert_eq!(model_info.max_tokens, 32_768);
    }

    #[test]
    fn test_validate_config_success() {
        let config = create_test_config();
        let provider = GroqProvider::new(config).unwrap();

        let result = provider.validate_config();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_config_missing_api_key() {
        let mut config = create_test_config();
        config.api_key = None;

        // 测试创建提供商时的错误
        let result = GroqProvider::new(config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("API key is required"));
    }

    #[test]
    fn test_validate_config_unsupported_model() {
        let mut config = create_test_config();
        config.model = "unsupported-model".to_string();
        let provider = GroqProvider::new(config).unwrap();

        let result = provider.validate_config();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not supported"));
    }

    #[test]
    fn test_empty_messages() {
        let config = create_test_config();
        let provider = GroqProvider::new(config).unwrap();

        let messages = vec![];
        let groq_messages = provider.convert_messages(&messages);

        assert_eq!(groq_messages.len(), 0);
    }

    #[test]
    fn test_long_message() {
        let config = create_test_config();
        let provider = GroqProvider::new(config).unwrap();

        let long_content = "A".repeat(5000); // 5K 字符的长消息
        let messages = vec![Message::user(&long_content)];

        let groq_messages = provider.convert_messages(&messages);

        assert_eq!(groq_messages.len(), 1);
        assert_eq!(groq_messages[0].content, long_content);
    }

    #[test]
    fn test_special_characters() {
        let config = create_test_config();
        let provider = GroqProvider::new(config).unwrap();

        let special_content = "Hello! 你好 🌟 \n\t Special chars: @#$%^&*()";
        let messages = vec![Message::user(special_content)];

        let groq_messages = provider.convert_messages(&messages);

        assert_eq!(groq_messages.len(), 1);
        assert_eq!(groq_messages[0].content, special_content);
    }

    #[test]
    fn test_all_supported_models() {
        let supported_models = GroqProvider::supported_models();

        for model in supported_models {
            let mut config = create_test_config();
            config.model = model.to_string();

            let provider = GroqProvider::new(config).unwrap();
            assert!(provider.is_model_supported());

            let model_info = provider.get_model_info();
            assert_eq!(model_info.model, model);
            assert_eq!(model_info.provider, "groq");
            assert!(model_info.max_tokens > 0);
        }
    }

    #[test]
    fn test_custom_base_url() {
        let mut config = create_test_config();
        config.base_url = Some("https://custom.groq.endpoint.com/v1".to_string());

        let provider = GroqProvider::new(config).unwrap();
        let url = provider.build_api_url();

        assert_eq!(url, "https://custom.groq.endpoint.com/v1/chat/completions");
    }

    #[test]
    fn test_default_base_url() {
        let mut config = create_test_config();
        config.base_url = None;

        let provider = GroqProvider::new(config).unwrap();
        let url = provider.build_api_url();

        assert_eq!(url, "https://api.groq.com/openai/v1/chat/completions");
    }
}
