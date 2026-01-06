//! Google Gemini 提供商测试

#[cfg(test)]
mod tests {
    use super::super::gemini::{
        GeminiCandidate, GeminiGenerationConfig, GeminiMessage, GeminiPart, GeminiProvider,
        GeminiRequest, GeminiResponse,
    };
    use agent_mem_traits::{LLMConfig, LLMProvider, Message};

    fn create_test_config() -> LLMConfig {
        LLMConfig {
            provider: "gemini".to_string(),
            model: "gemini-1.5-pro".to_string(),
            api_key: Some("test-api-key".to_string()),
            base_url: Some("https://generativelanguage.googleapis.com".to_string()),
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
    fn test_gemini_provider_creation() {
        let config = create_test_config();
        let provider = GeminiProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_convert_messages() {
        let config = create_test_config();
        let provider = GeminiProvider::new(config).unwrap();
        let messages = create_test_messages();

        let gemini_messages = provider.convert_messages(&messages);

        assert_eq!(gemini_messages.len(), 2);

        // 第一条消息（System -> User）
        assert_eq!(gemini_messages[0].role, "user");
        assert_eq!(gemini_messages[0].parts.len(), 1);
        assert_eq!(
            gemini_messages[0].parts[0].text,
            "You are a helpful assistant."
        );

        // 第二条消息（User -> User）
        assert_eq!(gemini_messages[1].role, "user");
        assert_eq!(gemini_messages[1].parts.len(), 1);
        assert_eq!(gemini_messages[1].parts[0].text, "Hello, how are you?");
    }

    #[test]
    fn test_build_api_url() {
        let config = create_test_config();
        let provider = GeminiProvider::new(config).unwrap();

        let url = provider.build_api_url("generateContent");
        assert_eq!(
            url,
            "https://generativelanguage.googleapis.com/models/gemini-1.5-pro:generateContent"
        );
    }

    #[test]
    fn test_extract_response_text() {
        let config = create_test_config();
        let provider = GeminiProvider::new(config).unwrap();

        // 创建真实响应结构用于测试
        let response = GeminiResponse {
            candidates: vec![GeminiCandidate {
                content: GeminiMessage {
                    role: "model".to_string(),
                    parts: vec![GeminiPart {
                        text: "Hello! I'm doing well, thank you for asking.".to_string(),
                    }],
                },
                finish_reason: Some("STOP".to_string()),
            }],
            usage_metadata: None,
        };

        let result = provider.extract_response_text(&response);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "Hello! I'm doing well, thank you for asking."
        );
    }

    #[test]
    fn test_extract_response_text_empty_candidates() {
        let config = create_test_config();
        let provider = GeminiProvider::new(config).unwrap();

        let response = GeminiResponse {
            candidates: vec![],
            usage_metadata: None,
        };

        let result = provider.extract_response_text(&response);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No candidates in response"));
    }

    #[test]
    fn test_extract_response_text_bad_finish_reason() {
        let config = create_test_config();
        let provider = GeminiProvider::new(config).unwrap();

        let response = GeminiResponse {
            candidates: vec![GeminiCandidate {
                content: GeminiMessage {
                    role: "model".to_string(),
                    parts: vec![GeminiPart {
                        text: "Partial response".to_string(),
                    }],
                },
                finish_reason: Some("SAFETY".to_string()),
            }],
            usage_metadata: None,
        };

        let result = provider.extract_response_text(&response);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Generation stopped due to: SAFETY"));
    }

    #[test]
    fn test_extract_response_text_multiple_parts() {
        let config = create_test_config();
        let provider = GeminiProvider::new(config).unwrap();

        let response = GeminiResponse {
            candidates: vec![GeminiCandidate {
                content: GeminiMessage {
                    role: "model".to_string(),
                    parts: vec![
                        GeminiPart {
                            text: "Hello!".to_string(),
                        },
                        GeminiPart {
                            text: "How can I help you today?".to_string(),
                        },
                    ],
                },
                finish_reason: Some("STOP".to_string()),
            }],
            usage_metadata: None,
        };

        let result = provider.extract_response_text(&response);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello! How can I help you today?");
    }

    #[test]
    fn test_extract_response_text_no_text_parts() {
        let config = create_test_config();
        let provider = GeminiProvider::new(config).unwrap();

        let response = GeminiResponse {
            candidates: vec![GeminiCandidate {
                content: GeminiMessage {
                    role: "model".to_string(),
                    parts: vec![], // 空的 parts
                },
                finish_reason: Some("STOP".to_string()),
            }],
            usage_metadata: None,
        };

        let result = provider.extract_response_text(&response);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No text content in response"));
    }

    #[test]
    fn test_message_role_conversion() {
        let config = create_test_config();
        let provider = GeminiProvider::new(config).unwrap();

        let messages = vec![
            Message::system("System message"),
            Message::user("User message"),
            Message::assistant("Assistant message"),
        ];

        let gemini_messages = provider.convert_messages(&messages);

        assert_eq!(gemini_messages.len(), 3);
        assert_eq!(gemini_messages[0].role, "user"); // System -> User
        assert_eq!(gemini_messages[1].role, "user"); // User -> User
        assert_eq!(gemini_messages[2].role, "model"); // Assistant -> Model
    }

    #[test]
    fn test_generation_config() {
        let config = create_test_config();
        let provider = GeminiProvider::new(config).unwrap();
        let messages = create_test_messages();

        let gemini_messages = provider.convert_messages(&messages);

        let request = GeminiRequest {
            contents: gemini_messages,
            generation_config: GeminiGenerationConfig {
                temperature: 0.7,
                top_p: 0.9,
                top_k: 40,
                max_output_tokens: 1000,
            },
        };

        assert_eq!(request.generation_config.temperature, 0.7);
        assert_eq!(request.generation_config.top_p, 0.9);
        assert_eq!(request.generation_config.top_k, 40);
        assert_eq!(request.generation_config.max_output_tokens, 1000);
    }

    #[test]
    #[ignore] // Requires Google Gemini credentials
    fn test_provider_traits() {
        let config = create_test_config();
        let provider = GeminiProvider::new(config).unwrap();

        // 测试 LLMProvider trait 方法
        let model_info = provider.get_model_info();
        assert_eq!(model_info.model, "gemini-1.5-pro");
        assert_eq!(model_info.provider, "gemini");
        assert!(!model_info.supports_streaming); // 当前不支持流式
        assert!(model_info.supports_functions); // Gemini 支持函数调用

        // 测试配置验证
        let validation_result = provider.validate_config();
        assert!(validation_result.is_ok());
    }

    #[test]
    #[ignore] // Requires Google Gemini credentials
    fn test_model_info() {
        let config = create_test_config();
        let provider = GeminiProvider::new(config).unwrap();

        let model_info = provider.get_model_info();
        assert_eq!(model_info.model, "gemini-1.5-pro");
        assert_eq!(model_info.provider, "gemini");
        assert!(model_info.max_tokens > 0);
        assert!(model_info.supports_functions);
        assert!(!model_info.supports_streaming);
    }

    #[test]
    fn test_config_validation() {
        // 测试缺少 API 密钥的配置
        let mut config = create_test_config();
        config.api_key = None;

        let provider = GeminiProvider::new(config);
        assert!(provider.is_err()); // 创建时验证 API 密钥
    }

    #[test]
    fn test_empty_messages() {
        let config = create_test_config();
        let provider = GeminiProvider::new(config).unwrap();

        let messages = vec![];
        let gemini_messages = provider.convert_messages(&messages);

        assert_eq!(gemini_messages.len(), 0);
    }

    #[test]
    fn test_long_message() {
        let config = create_test_config();
        let provider = GeminiProvider::new(config).unwrap();

        let long_content = "A".repeat(10000); // 10K 字符的长消息
        let messages = vec![Message::user(&long_content)];

        let gemini_messages = provider.convert_messages(&messages);

        assert_eq!(gemini_messages.len(), 1);
        assert_eq!(gemini_messages[0].parts[0].text, long_content);
    }

    #[test]
    fn test_special_characters() {
        let config = create_test_config();
        let provider = GeminiProvider::new(config).unwrap();

        let special_content = "Hello! 你好 🌟 \n\t Special chars: @#$%^&*()";
        let messages = vec![Message::user(special_content)];

        let gemini_messages = provider.convert_messages(&messages);

        assert_eq!(gemini_messages.len(), 1);
        assert_eq!(gemini_messages[0].parts[0].text, special_content);
    }
}
