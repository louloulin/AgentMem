//! Azure OpenAI 提供商测试

#[cfg(test)]
mod tests {
    use super::super::azure::{
        AzureChoice, AzureMessage, AzureProvider, AzureRequest, AzureResponse, AzureUsage,
    };
    use agent_mem_traits::{LLMConfig, LLMProvider, Message};

    fn create_test_config() -> LLMConfig {
        LLMConfig {
            provider: "azure".to_string(),
            model: "gpt-4".to_string(),
            api_key: Some("test-api-key".to_string()),
            base_url: Some("https://test-resource.openai.azure.com".to_string()),
            temperature: Some(0.7),
            max_tokens: Some(1000),
            top_p: Some(0.9),
            frequency_penalty: Some(0.0),
            presence_penalty: Some(0.0),
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
    fn test_azure_provider_creation() {
        let config = create_test_config();
        let provider = AzureProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_azure_provider_creation_missing_api_key() {
        let mut config = create_test_config();
        config.api_key = None;

        let provider = AzureProvider::new(config);
        assert!(provider.is_err());
        assert!(provider
            .unwrap_err()
            .to_string()
            .contains("API key is required"));
    }

    #[test]
    fn test_azure_provider_creation_missing_base_url() {
        let mut config = create_test_config();
        config.base_url = None;

        let provider = AzureProvider::new(config);
        assert!(provider.is_err());
        assert!(provider
            .unwrap_err()
            .to_string()
            .contains("endpoint URL is required"));
    }

    #[test]
    fn test_azure_provider_creation_empty_model() {
        let mut config = create_test_config();
        config.model = "".to_string();

        let provider = AzureProvider::new(config);
        assert!(provider.is_err());
        assert!(provider
            .unwrap_err()
            .to_string()
            .contains("Deployment name"));
    }

    #[test]
    fn test_build_api_url() {
        let config = create_test_config();
        let provider = AzureProvider::new(config).unwrap();

        let url = provider.build_api_url();
        assert_eq!(
            url,
            "https://test-resource.openai.azure.com/openai/deployments/gpt-4/chat/completions?api-version=2024-02-01"
        );
    }

    #[test]
    fn test_build_api_url_with_trailing_slash() {
        let mut config = create_test_config();
        config.base_url = Some("https://test-resource.openai.azure.com/".to_string());
        let provider = AzureProvider::new(config).unwrap();

        let url = provider.build_api_url();
        assert_eq!(
            url,
            "https://test-resource.openai.azure.com/openai/deployments/gpt-4/chat/completions?api-version=2024-02-01"
        );
    }

    #[test]
    fn test_convert_messages() {
        let config = create_test_config();
        let provider = AzureProvider::new(config).unwrap();
        let messages = create_test_messages();

        let azure_messages = provider.convert_messages(&messages);

        assert_eq!(azure_messages.len(), 2);

        // 第一条消息（System）
        assert_eq!(azure_messages[0].role, "system");
        assert_eq!(azure_messages[0].content, "You are a helpful assistant.");

        // 第二条消息（User）
        assert_eq!(azure_messages[1].role, "user");
        assert_eq!(azure_messages[1].content, "Hello, how are you?");
    }

    #[test]
    fn test_convert_messages_all_roles() {
        let config = create_test_config();
        let provider = AzureProvider::new(config).unwrap();

        let messages = vec![
            Message::system("System message"),
            Message::user("User message"),
            Message::assistant("Assistant message"),
        ];

        let azure_messages = provider.convert_messages(&messages);

        assert_eq!(azure_messages.len(), 3);
        assert_eq!(azure_messages[0].role, "system");
        assert_eq!(azure_messages[1].role, "user");
        assert_eq!(azure_messages[2].role, "assistant");
    }

    #[test]
    fn test_extract_response_text() {
        let config = create_test_config();
        let provider = AzureProvider::new(config).unwrap();

        // 创建真实响应结构用于测试
        let response = AzureResponse {
            id: "chatcmpl-test".to_string(),
            object: "chat.completion".to_string(),
            created: 1234567890,
            model: "gpt-4".to_string(),
            choices: vec![AzureChoice {
                index: 0,
                message: AzureMessage {
                    role: "assistant".to_string(),
                    content: "Hello! I'm doing well, thank you for asking.".to_string(),
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage: Some(AzureUsage {
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
        let provider = AzureProvider::new(config).unwrap();

        let response = AzureResponse {
            id: "chatcmpl-test".to_string(),
            object: "chat.completion".to_string(),
            created: 1234567890,
            model: "gpt-4".to_string(),
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
    fn test_extract_response_text_content_filter() {
        let config = create_test_config();
        let provider = AzureProvider::new(config).unwrap();

        let response = AzureResponse {
            id: "chatcmpl-test".to_string(),
            object: "chat.completion".to_string(),
            created: 1234567890,
            model: "gpt-4".to_string(),
            choices: vec![AzureChoice {
                index: 0,
                message: AzureMessage {
                    role: "assistant".to_string(),
                    content: "".to_string(),
                },
                finish_reason: Some("content_filter".to_string()),
            }],
            usage: None,
        };

        let result = provider.extract_response_text(&response);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Content was filtered"));
    }

    #[test]
    fn test_extract_response_text_length_finish() {
        let config = create_test_config();
        let provider = AzureProvider::new(config).unwrap();

        let response = AzureResponse {
            id: "chatcmpl-test".to_string(),
            object: "chat.completion".to_string(),
            created: 1234567890,
            model: "gpt-4".to_string(),
            choices: vec![AzureChoice {
                index: 0,
                message: AzureMessage {
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
    fn test_azure_request_serialization() {
        let request = AzureRequest {
            messages: vec![AzureMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            max_tokens: Some(1000),
            temperature: Some(0.7),
            top_p: Some(0.9),
            frequency_penalty: Some(0.0),
            presence_penalty: Some(0.0),
            stop: None,
            stream: Some(false),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"messages\""));
        assert!(json.contains("\"max_tokens\""));
        assert!(json.contains("\"temperature\""));
        assert!(json.contains("\"top_p\""));
        assert!(json.contains("\"frequency_penalty\""));
        assert!(json.contains("\"presence_penalty\""));
        assert!(json.contains("\"stream\""));
    }

    #[test]
    #[ignore] // Requires Azure OpenAI credentials
    fn test_model_info() {
        let config = create_test_config();
        let provider = AzureProvider::new(config).unwrap();

        let model_info = provider.get_model_info();
        assert_eq!(model_info.model, "gpt-4");
        assert_eq!(model_info.provider, "azure");
        assert_eq!(model_info.max_tokens, 1000);
        assert!(!model_info.supports_streaming);
        assert!(model_info.supports_functions);
    }

    #[test]
    fn test_validate_config_success() {
        let config = create_test_config();
        let provider = AzureProvider::new(config).unwrap();

        let result = provider.validate_config();
        assert!(result.is_ok());
    }

    #[test]
    fn test_with_api_version() {
        let config = create_test_config();
        let provider = AzureProvider::new(config)
            .unwrap()
            .with_api_version("2023-05-15");

        let url = provider.build_api_url();
        assert!(url.contains("api-version=2023-05-15"));
    }

    #[test]
    fn test_empty_messages() {
        let config = create_test_config();
        let provider = AzureProvider::new(config).unwrap();

        let messages = vec![];
        let azure_messages = provider.convert_messages(&messages);

        assert_eq!(azure_messages.len(), 0);
    }

    #[test]
    fn test_long_message() {
        let config = create_test_config();
        let provider = AzureProvider::new(config).unwrap();

        let long_content = "A".repeat(10000); // 10K 字符的长消息
        let messages = vec![Message::user(&long_content)];

        let azure_messages = provider.convert_messages(&messages);

        assert_eq!(azure_messages.len(), 1);
        assert_eq!(azure_messages[0].content, long_content);
    }

    #[test]
    fn test_special_characters() {
        let config = create_test_config();
        let provider = AzureProvider::new(config).unwrap();

        let special_content = "Hello! 你好 🌟 \n\t Special chars: @#$%^&*()";
        let messages = vec![Message::user(special_content)];

        let azure_messages = provider.convert_messages(&messages);

        assert_eq!(azure_messages.len(), 1);
        assert_eq!(azure_messages[0].content, special_content);
    }

    #[test]
    fn test_different_deployment_names() {
        let deployment_names = vec!["gpt-4", "gpt-35-turbo", "my-custom-deployment"];

        for deployment_name in deployment_names {
            let mut config = create_test_config();
            config.model = deployment_name.to_string();

            let provider = AzureProvider::new(config).unwrap();
            let url = provider.build_api_url();

            assert!(url.contains(&format!("/deployments/{}/", deployment_name)));
        }
    }

    #[test]
    fn test_different_regions() {
        let base_urls = vec![
            "https://eastus.api.cognitive.microsoft.com",
            "https://westus2.api.cognitive.microsoft.com",
            "https://northeurope.api.cognitive.microsoft.com",
        ];

        for base_url in base_urls {
            let mut config = create_test_config();
            config.base_url = Some(base_url.to_string());

            let provider = AzureProvider::new(config).unwrap();
            let url = provider.build_api_url();

            assert!(url.starts_with(base_url));
        }
    }

    // 真实 API 集成测试 (需要环境变量)
    #[cfg(feature = "integration-tests")]
    mod integration_tests {
        use super::*;
        use std::env;

        fn create_real_config() -> Option<LLMConfig> {
            let api_key = env::var("AZURE_OPENAI_API_KEY").ok()?;
            let base_url = env::var("AZURE_OPENAI_ENDPOINT").ok()?;

            Some(LLMConfig {
                provider: "azure".to_string(),
                model: "gpt-4".to_string(),
                api_key: Some(api_key),
                base_url: Some(base_url),
                temperature: Some(0.7),
                max_tokens: Some(100),
                top_p: Some(0.9),
                frequency_penalty: Some(0.0),
                presence_penalty: Some(0.0),
                response_format: None,
            })
        }

        #[tokio::test]
        async fn test_real_azure_api_generate() {
            let Some(config) = create_real_config() else {
                println!("Skipping real API test - AZURE_OPENAI_API_KEY or AZURE_OPENAI_ENDPOINT not set");
                return;
            };

            let provider = AzureProvider::new(config).unwrap();
            let messages = vec![
                Message::system("You are a helpful assistant."),
                Message::user("Say hello in exactly 3 words."),
            ];

            let result = provider.generate(&messages).await;
            assert!(result.is_ok(), "Real API call should succeed: {:?}", result);

            let response = result.unwrap();
            assert!(!response.is_empty(), "Response should not be empty");
            println!("Azure OpenAI Response: {}", response);
        }

        #[tokio::test]
        async fn test_real_azure_api_health_check() {
            let Some(config) = create_real_config() else {
                println!("Skipping real API test - AZURE_OPENAI_API_KEY or AZURE_OPENAI_ENDPOINT not set");
                return;
            };

            let provider = AzureProvider::new(config).unwrap();
            let health = provider.health_check().await;

            assert!(health.is_ok(), "Health check should succeed: {:?}", health);
            let health_status = health.unwrap();
            assert_eq!(health_status.status, "healthy");
            println!("Azure OpenAI Health: {:?}", health_status);
        }

        #[tokio::test]
        async fn test_real_azure_api_error_handling() {
            // 使用无效的 API key 测试错误处理
            let config = LLMConfig {
                provider: "azure".to_string(),
                model: "gpt-4".to_string(),
                api_key: Some("invalid-api-key".to_string()),
                base_url: Some("https://invalid-resource.openai.azure.com".to_string()),
                temperature: Some(0.7),
                max_tokens: Some(100),
                top_p: Some(0.9),
                frequency_penalty: Some(0.0),
                presence_penalty: Some(0.0),
                response_format: None,
            };

            let provider = AzureProvider::new(config).unwrap();
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
