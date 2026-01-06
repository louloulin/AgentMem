//! Together AI 提供商测试

#[cfg(test)]
mod tests {
    use super::super::together::{
        TogetherChoice, TogetherMessage, TogetherProvider, TogetherRequest, TogetherResponse,
        TogetherUsage,
    };
    use agent_mem_traits::{LLMConfig, LLMProvider, Message};

    fn create_test_config() -> LLMConfig {
        LLMConfig {
            provider: "together".to_string(),
            model: "meta-llama/Meta-Llama-3-8B-Instruct".to_string(),
            api_key: Some("test-api-key".to_string()),
            base_url: Some("https://api.together.xyz/v1".to_string()),
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
    fn test_together_provider_creation() {
        let config = create_test_config();
        let provider = TogetherProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_together_provider_creation_missing_api_key() {
        let mut config = create_test_config();
        config.api_key = None;

        let provider = TogetherProvider::new(config);
        assert!(provider.is_err());
        assert!(provider
            .unwrap_err()
            .to_string()
            .contains("API key is required"));
    }

    #[test]
    fn test_together_provider_creation_empty_model() {
        let mut config = create_test_config();
        config.model = "".to_string();

        let provider = TogetherProvider::new(config);
        assert!(provider.is_err());
        assert!(provider
            .unwrap_err()
            .to_string()
            .contains("Model name is required"));
    }

    #[test]
    fn test_build_api_url() {
        let config = create_test_config();
        let provider = TogetherProvider::new(config).unwrap();

        let url = provider.build_api_url();
        assert_eq!(url, "https://api.together.xyz/v1/chat/completions");
    }

    #[test]
    fn test_build_api_url_with_trailing_slash() {
        let mut config = create_test_config();
        config.base_url = Some("https://api.together.xyz/v1/".to_string());
        let provider = TogetherProvider::new(config).unwrap();

        let url = provider.build_api_url();
        assert_eq!(url, "https://api.together.xyz/v1/chat/completions");
    }

    #[test]
    fn test_convert_messages() {
        let config = create_test_config();
        let provider = TogetherProvider::new(config).unwrap();
        let messages = create_test_messages();

        let together_messages = provider.convert_messages(&messages);

        assert_eq!(together_messages.len(), 2);

        // 第一条消息（System）
        assert_eq!(together_messages[0].role, "system");
        assert_eq!(together_messages[0].content, "You are a helpful assistant.");

        // 第二条消息（User）
        assert_eq!(together_messages[1].role, "user");
        assert_eq!(together_messages[1].content, "Hello, how are you?");
    }

    #[test]
    fn test_convert_messages_all_roles() {
        let config = create_test_config();
        let provider = TogetherProvider::new(config).unwrap();

        let messages = vec![
            Message::system("System message"),
            Message::user("User message"),
            Message::assistant("Assistant message"),
        ];

        let together_messages = provider.convert_messages(&messages);

        assert_eq!(together_messages.len(), 3);
        assert_eq!(together_messages[0].role, "system");
        assert_eq!(together_messages[1].role, "user");
        assert_eq!(together_messages[2].role, "assistant");
    }

    #[test]
    fn test_extract_response_text() {
        let config = create_test_config();
        let provider = TogetherProvider::new(config).unwrap();

        // 创建真实响应结构用于测试
        let response = TogetherResponse {
            id: "chatcmpl-test".to_string(),
            object: "chat.completion".to_string(),
            created: 1234567890,
            model: "meta-llama/Meta-Llama-3-8B-Instruct".to_string(),
            choices: vec![TogetherChoice {
                index: 0,
                message: TogetherMessage {
                    role: "assistant".to_string(),
                    content: "Hello! I'm doing well, thank you for asking.".to_string(),
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage: Some(TogetherUsage {
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
        let provider = TogetherProvider::new(config).unwrap();

        let response = TogetherResponse {
            id: "chatcmpl-test".to_string(),
            object: "chat.completion".to_string(),
            created: 1234567890,
            model: "meta-llama/Meta-Llama-3-8B-Instruct".to_string(),
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
    fn test_extract_response_text_eos_finish() {
        let config = create_test_config();
        let provider = TogetherProvider::new(config).unwrap();

        let response = TogetherResponse {
            id: "chatcmpl-test".to_string(),
            object: "chat.completion".to_string(),
            created: 1234567890,
            model: "meta-llama/Meta-Llama-3-8B-Instruct".to_string(),
            choices: vec![TogetherChoice {
                index: 0,
                message: TogetherMessage {
                    role: "assistant".to_string(),
                    content: "This response ended with EOS token.".to_string(),
                },
                finish_reason: Some("eos".to_string()),
            }],
            usage: None,
        };

        let result = provider.extract_response_text(&response);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "This response ended with EOS token.");
    }

    #[test]
    fn test_together_request_serialization() {
        let request = TogetherRequest {
            model: "meta-llama/Meta-Llama-3-8B-Instruct".to_string(),
            messages: vec![TogetherMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            max_tokens: Some(1000),
            temperature: Some(0.7),
            top_p: Some(0.9),
            top_k: Some(50),
            repetition_penalty: Some(1.1),
            stop: None,
            stream: Some(false),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"model\""));
        assert!(json.contains("\"messages\""));
        assert!(json.contains("\"max_tokens\""));
        assert!(json.contains("\"temperature\""));
        assert!(json.contains("\"top_p\""));
        assert!(json.contains("\"top_k\""));
        assert!(json.contains("\"repetition_penalty\""));
        assert!(json.contains("\"stream\""));
    }

    #[test]
    fn test_supported_models() {
        let models = TogetherProvider::supported_models();
        assert!(models.contains(&"meta-llama/Meta-Llama-3-8B-Instruct"));
        assert!(models.contains(&"meta-llama/Llama-2-7b-chat-hf"));
        assert!(models.contains(&"mistralai/Mistral-7B-Instruct-v0.2"));
        assert!(models.contains(&"codellama/CodeLlama-7b-Instruct-hf"));
        assert!(models.len() > 10); // 确保有足够多的支持模型
    }

    #[test]
    fn test_is_model_supported() {
        let config = create_test_config();
        let provider = TogetherProvider::new(config).unwrap();

        assert!(provider.is_model_supported());

        // 测试不支持的模型
        let mut config = create_test_config();
        config.model = "unsupported/model".to_string();
        let provider = TogetherProvider::new(config).unwrap();

        assert!(!provider.is_model_supported());
    }

    #[test]
    fn test_model_info_llama3() {
        let config = create_test_config();
        let provider = TogetherProvider::new(config).unwrap();

        let model_info = provider.get_model_info();
        assert_eq!(model_info.model, "meta-llama/Meta-Llama-3-8B-Instruct");
        assert_eq!(model_info.provider, "together");
        assert_eq!(model_info.max_tokens, 8_192);
        assert!(!model_info.supports_streaming);
        assert!(!model_info.supports_functions);
    }

    #[test]
    fn test_model_info_mixtral() {
        let mut config = create_test_config();
        config.model = "mistralai/Mixtral-8x7B-Instruct-v0.1".to_string();
        let provider = TogetherProvider::new(config).unwrap();

        let model_info = provider.get_model_info();
        assert_eq!(model_info.model, "mistralai/Mixtral-8x7B-Instruct-v0.1");
        assert_eq!(model_info.provider, "together");
        assert_eq!(model_info.max_tokens, 32_768);
    }

    #[test]
    fn test_model_info_codellama() {
        let mut config = create_test_config();
        config.model = "codellama/CodeLlama-13b-Instruct-hf".to_string();
        let provider = TogetherProvider::new(config).unwrap();

        let model_info = provider.get_model_info();
        assert_eq!(model_info.model, "codellama/CodeLlama-13b-Instruct-hf");
        assert_eq!(model_info.provider, "together");
        assert_eq!(model_info.max_tokens, 16_384);
    }

    #[test]
    fn test_validate_config_success() {
        let config = create_test_config();
        let provider = TogetherProvider::new(config).unwrap();

        let result = provider.validate_config();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_config_missing_api_key() {
        let mut config = create_test_config();
        config.api_key = None;

        // 测试创建提供商时的错误
        let result = TogetherProvider::new(config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("API key is required"));
    }

    #[test]
    fn test_empty_messages() {
        let config = create_test_config();
        let provider = TogetherProvider::new(config).unwrap();

        let messages = vec![];
        let together_messages = provider.convert_messages(&messages);

        assert_eq!(together_messages.len(), 0);
    }

    #[test]
    fn test_long_message() {
        let config = create_test_config();
        let provider = TogetherProvider::new(config).unwrap();

        let long_content = "A".repeat(8000); // 8K 字符的长消息
        let messages = vec![Message::user(&long_content)];

        let together_messages = provider.convert_messages(&messages);

        assert_eq!(together_messages.len(), 1);
        assert_eq!(together_messages[0].content, long_content);
    }

    #[test]
    fn test_special_characters() {
        let config = create_test_config();
        let provider = TogetherProvider::new(config).unwrap();

        let special_content = "Hello! 你好 🌟 \n\t Special chars: @#$%^&*()";
        let messages = vec![Message::user(special_content)];

        let together_messages = provider.convert_messages(&messages);

        assert_eq!(together_messages.len(), 1);
        assert_eq!(together_messages[0].content, special_content);
    }

    #[test]
    fn test_different_model_families() {
        let model_families = vec![
            ("meta-llama/Llama-2-7b-chat-hf", 4_096),
            ("meta-llama/Meta-Llama-3-8B-Instruct", 8_192),
            ("mistralai/Mistral-7B-Instruct-v0.2", 8_192),
            ("mistralai/Mixtral-8x7B-Instruct-v0.1", 32_768),
            ("codellama/CodeLlama-7b-Instruct-hf", 16_384),
        ];

        for (model, expected_max_tokens) in model_families {
            let mut config = create_test_config();
            config.model = model.to_string();

            let provider = TogetherProvider::new(config).unwrap();
            let model_info = provider.get_model_info();

            assert_eq!(model_info.model, model);
            assert_eq!(model_info.provider, "together");
            assert_eq!(model_info.max_tokens, expected_max_tokens);
        }
    }

    #[test]
    fn test_custom_base_url() {
        let mut config = create_test_config();
        config.base_url = Some("https://custom.together.endpoint.com/v1".to_string());

        let provider = TogetherProvider::new(config).unwrap();
        let url = provider.build_api_url();

        assert_eq!(
            url,
            "https://custom.together.endpoint.com/v1/chat/completions"
        );
    }

    #[test]
    fn test_default_base_url() {
        let mut config = create_test_config();
        config.base_url = None;

        let provider = TogetherProvider::new(config).unwrap();
        let url = provider.build_api_url();

        assert_eq!(url, "https://api.together.xyz/v1/chat/completions");
    }

    #[test]
    fn test_model_max_tokens_calculation() {
        let test_cases = vec![
            ("meta-llama/Llama-2-7b-chat-hf", 4_096),
            ("meta-llama/Meta-Llama-3-70B-Instruct", 8_192),
            ("mistralai/Mistral-7B-Instruct-v0.1", 8_192),
            ("mistralai/Mixtral-8x7B-Instruct-v0.1", 32_768),
            ("codellama/CodeLlama-34b-Instruct-hf", 16_384),
            ("unknown/model", 4_096), // 默认值
        ];

        for (model, expected_tokens) in test_cases {
            let mut config = create_test_config();
            config.model = model.to_string();

            let provider = TogetherProvider::new(config).unwrap();
            assert_eq!(provider.get_model_max_tokens(), expected_tokens);
        }
    }

    // 真实 API 集成测试 (需要环境变量)
    #[cfg(feature = "integration-tests")]
    mod integration_tests {
        use super::*;
        use std::env;

        fn create_real_config() -> Option<LLMConfig> {
            env::var("TOGETHER_API_KEY").ok().map(|api_key| LLMConfig {
                provider: "together".to_string(),
                model: "meta-llama/Meta-Llama-3-8B-Instruct".to_string(),
                api_key: Some(api_key),
                base_url: Some("https://api.together.xyz/v1".to_string()),
                temperature: Some(0.7),
                max_tokens: Some(100),
                top_p: Some(0.9),
                frequency_penalty: None,
                presence_penalty: None,
                response_format: None,
            })
        }

        #[tokio::test]
        async fn test_real_together_api_generate() {
            let Some(config) = create_real_config() else {
                println!("Skipping real API test - TOGETHER_API_KEY not set");
                return;
            };

            let provider = TogetherProvider::new(config).unwrap();
            let messages = vec![
                Message::system("You are a helpful assistant."),
                Message::user("Say hello in exactly 3 words."),
            ];

            let result = provider.generate(&messages).await;
            assert!(result.is_ok(), "Real API call should succeed: {:?}", result);

            let response = result.unwrap();
            assert!(!response.is_empty(), "Response should not be empty");
            println!("Together API Response: {}", response);
        }

        #[tokio::test]
        async fn test_real_together_api_health_check() {
            let Some(config) = create_real_config() else {
                println!("Skipping real API test - TOGETHER_API_KEY not set");
                return;
            };

            let provider = TogetherProvider::new(config).unwrap();
            let health = provider.health_check().await;

            assert!(health.is_ok(), "Health check should succeed: {:?}", health);
            let health_status = health.unwrap();
            assert_eq!(health_status.status, "healthy");
            println!("Together API Health: {:?}", health_status);
        }

        #[tokio::test]
        async fn test_real_together_api_error_handling() {
            // 使用无效的 API key 测试错误处理
            let config = LLMConfig {
                provider: "together".to_string(),
                model: "meta-llama/Meta-Llama-3-8B-Instruct".to_string(),
                api_key: Some("invalid-api-key".to_string()),
                base_url: Some("https://api.together.xyz/v1".to_string()),
                temperature: Some(0.7),
                max_tokens: Some(100),
                top_p: Some(0.9),
                frequency_penalty: None,
                presence_penalty: None,
                response_format: None,
            };

            let provider = TogetherProvider::new(config).unwrap();
            let messages = vec![Message::user("Test message")];

            let result = provider.generate(&messages).await;
            assert!(result.is_err(), "Invalid API key should cause error");

            let error = result.unwrap_err();
            assert!(
                error.to_string().contains("401") || error.to_string().contains("unauthorized")
            );
            println!("Expected error for invalid API key: {}", error);
        }
    }
}
