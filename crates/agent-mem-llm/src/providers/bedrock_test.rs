//! AWS Bedrock 提供商测试

#[cfg(test)]
mod tests {
    use super::super::bedrock::{
        BedrockClaudeRequest, BedrockLlamaRequest, BedrockProvider, BedrockTitanRequest,
        TitanTextConfig,
    };
    use agent_mem_traits::{LLMConfig, LLMProvider, Message};

    fn create_test_config() -> LLMConfig {
        LLMConfig {
            provider: "bedrock".to_string(),
            model: "anthropic.claude-3-sonnet-20240229-v1:0".to_string(),
            api_key: Some("test-access-key:test-secret-key:us-east-1".to_string()),
            base_url: None,
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
    fn test_bedrock_provider_creation() {
        let config = create_test_config();
        let provider = BedrockProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_bedrock_provider_creation_invalid_credentials() {
        let mut config = create_test_config();
        config.api_key = Some("invalid-format".to_string());

        let provider = BedrockProvider::new(config);
        assert!(provider.is_err());
        assert!(provider
            .unwrap_err()
            .to_string()
            .contains("AWS credentials format"));
    }

    #[test]
    fn test_bedrock_provider_creation_missing_credentials() {
        let mut config = create_test_config();
        config.api_key = None;

        let provider = BedrockProvider::new(config);
        assert!(provider.is_err());
        assert!(provider
            .unwrap_err()
            .to_string()
            .contains("AWS access key is required"));
    }

    #[test]
    fn test_build_api_url() {
        let config = create_test_config();
        let provider = BedrockProvider::new(config).unwrap();

        let url = provider.build_api_url("anthropic.claude-3-sonnet-20240229-v1:0");
        assert_eq!(
            url,
            "https://bedrock-runtime.us-east-1.amazonaws.com/model/anthropic.claude-3-sonnet-20240229-v1:0/invoke"
        );
    }

    #[test]
    fn test_convert_messages_to_prompt() {
        let config = create_test_config();
        let provider = BedrockProvider::new(config).unwrap();
        let messages = create_test_messages();

        let prompt = provider.convert_messages_to_prompt(&messages);

        assert!(prompt.contains("System: You are a helpful assistant."));
        assert!(prompt.contains("Human: Hello, how are you?"));
        assert!(prompt.ends_with("Assistant: "));
    }

    #[test]
    fn test_detect_model_type_claude() {
        let config = create_test_config();
        let provider = BedrockProvider::new(config).unwrap();

        assert_eq!(provider.detect_model_type(), "claude");
    }

    #[test]
    fn test_detect_model_type_llama() {
        let mut config = create_test_config();
        config.model = "meta.llama2-70b-chat-v1".to_string();
        let provider = BedrockProvider::new(config).unwrap();

        assert_eq!(provider.detect_model_type(), "llama");
    }

    #[test]
    fn test_detect_model_type_titan() {
        let mut config = create_test_config();
        config.model = "amazon.titan-text-large-v1".to_string();
        let provider = BedrockProvider::new(config).unwrap();

        assert_eq!(provider.detect_model_type(), "titan");
    }

    #[test]
    fn test_detect_model_type_unknown() {
        let mut config = create_test_config();
        config.model = "unknown.model-v1".to_string();
        let provider = BedrockProvider::new(config).unwrap();

        // 未知模型默认使用 Claude 格式
        assert_eq!(provider.detect_model_type(), "claude");
    }

    #[test]
    fn test_claude_request_serialization() {
        let request = BedrockClaudeRequest {
            prompt: "Human: Hello\n\nAssistant: ".to_string(),
            max_tokens_to_sample: 1000,
            temperature: 0.7,
            top_p: 0.9,
            stop_sequences: vec!["Human:".to_string()],
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"prompt\""));
        assert!(json.contains("\"max_tokens_to_sample\""));
        assert!(json.contains("\"temperature\""));
        assert!(json.contains("\"top_p\""));
        assert!(json.contains("\"stop_sequences\""));
    }

    #[test]
    fn test_llama_request_serialization() {
        let request = BedrockLlamaRequest {
            prompt: "Human: Hello\n\nAssistant: ".to_string(),
            max_gen_len: 1000,
            temperature: 0.7,
            top_p: 0.9,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"prompt\""));
        assert!(json.contains("\"max_gen_len\""));
        assert!(json.contains("\"temperature\""));
        assert!(json.contains("\"top_p\""));
    }

    #[test]
    fn test_titan_request_serialization() {
        let request = BedrockTitanRequest {
            input_text: "Human: Hello\n\nAssistant: ".to_string(),
            text_generation_config: TitanTextConfig {
                max_token_count: 1000,
                temperature: 0.7,
                top_p: 0.9,
                stop_sequences: vec!["Human:".to_string()],
            },
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"inputText\""));
        assert!(json.contains("\"textGenerationConfig\""));
        assert!(json.contains("\"maxTokenCount\""));
        assert!(json.contains("\"stopSequences\""));
    }

    #[test]
    fn test_model_info_claude() {
        let config = create_test_config();
        let provider = BedrockProvider::new(config).unwrap();

        let model_info = provider.get_model_info();
        assert_eq!(model_info.model, "anthropic.claude-3-sonnet-20240229-v1:0");
        assert_eq!(model_info.provider, "bedrock");
        assert_eq!(model_info.max_tokens, 200_000); // Claude 3 支持 200K tokens
        assert!(!model_info.supports_streaming);
        assert!(!model_info.supports_functions);
    }

    #[test]
    fn test_model_info_llama() {
        let mut config = create_test_config();
        config.model = "meta.llama2-70b-chat-v1".to_string();
        let provider = BedrockProvider::new(config).unwrap();

        let model_info = provider.get_model_info();
        assert_eq!(model_info.model, "meta.llama2-70b-chat-v1");
        assert_eq!(model_info.provider, "bedrock");
        assert_eq!(model_info.max_tokens, 4_096); // Llama 2 支持 4K tokens
    }

    #[test]
    fn test_model_info_titan() {
        let mut config = create_test_config();
        config.model = "amazon.titan-text-large-v1".to_string();
        let provider = BedrockProvider::new(config).unwrap();

        let model_info = provider.get_model_info();
        assert_eq!(model_info.model, "amazon.titan-text-large-v1");
        assert_eq!(model_info.provider, "bedrock");
        assert_eq!(model_info.max_tokens, 8_000); // Titan 支持 8K tokens
    }

    #[test]
    fn test_validate_config_success() {
        let config = create_test_config();
        let provider = BedrockProvider::new(config).unwrap();

        let result = provider.validate_config();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_config_missing_model() {
        let mut config = create_test_config();
        config.model = "".to_string();
        let provider = BedrockProvider::new(config).unwrap();

        let result = provider.validate_config();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Model ID is required"));
    }

    #[test]
    fn test_prompt_formatting_system_message() {
        let config = create_test_config();
        let provider = BedrockProvider::new(config).unwrap();

        let messages = vec![
            Message::system("You are a helpful assistant."),
            Message::user("What is the capital of France?"),
        ];

        let prompt = provider.convert_messages_to_prompt(&messages);

        assert!(prompt.starts_with("System: You are a helpful assistant."));
        assert!(prompt.contains("Human: What is the capital of France?"));
        assert!(prompt.ends_with("Assistant: "));
    }

    #[test]
    fn test_prompt_formatting_conversation() {
        let config = create_test_config();
        let provider = BedrockProvider::new(config).unwrap();

        let messages = vec![
            Message::user("Hello"),
            Message::assistant("Hi there! How can I help you?"),
            Message::user("What's the weather like?"),
        ];

        let prompt = provider.convert_messages_to_prompt(&messages);

        assert!(prompt.contains("Human: Hello"));
        assert!(prompt.contains("Assistant: Hi there! How can I help you?"));
        assert!(prompt.contains("Human: What's the weather like?"));
        assert!(prompt.ends_with("Assistant: "));
    }

    #[test]
    fn test_prompt_formatting_empty_messages() {
        let config = create_test_config();
        let provider = BedrockProvider::new(config).unwrap();

        let messages = vec![];
        let prompt = provider.convert_messages_to_prompt(&messages);

        assert_eq!(prompt, "Assistant: ");
    }

    #[test]
    fn test_prompt_formatting_only_assistant() {
        let config = create_test_config();
        let provider = BedrockProvider::new(config).unwrap();

        let messages = vec![Message::assistant("I'm ready to help!")];

        let prompt = provider.convert_messages_to_prompt(&messages);

        assert!(prompt.contains("Assistant: I'm ready to help!"));
        assert!(prompt.ends_with("Assistant: "));
    }

    #[test]
    fn test_aws_credentials_parsing() {
        let config = LLMConfig {
            provider: "bedrock".to_string(),
            model: "anthropic.claude-3-sonnet-20240229-v1:0".to_string(),
            api_key: Some(
                "AKIAIOSFODNN7EXAMPLE:wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY:us-west-2"
                    .to_string(),
            ),
            base_url: None,
            temperature: Some(0.7),
            max_tokens: Some(1000),
            top_p: Some(0.9),
            frequency_penalty: None,
            presence_penalty: None,
            response_format: None,
        };

        let provider = BedrockProvider::new(config).unwrap();

        assert_eq!(provider.access_key, "AKIAIOSFODNN7EXAMPLE");
        assert_eq!(
            provider.secret_key,
            "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
        );
        assert_eq!(provider.region, "us-west-2");
    }

    #[test]
    fn test_different_regions() {
        let regions = vec!["us-east-1", "us-west-2", "eu-west-1", "ap-southeast-1"];

        for region in regions {
            let config = LLMConfig {
                provider: "bedrock".to_string(),
                model: "anthropic.claude-3-sonnet-20240229-v1:0".to_string(),
                api_key: Some(format!("access:secret:{region}")),
                base_url: None,
                temperature: Some(0.7),
                max_tokens: Some(1000),
                top_p: Some(0.9),
                frequency_penalty: None,
                presence_penalty: None,
                response_format: None,
            };

            let provider = BedrockProvider::new(config).unwrap();
            let url = provider.build_api_url("test-model");

            assert!(url.contains(&format!("bedrock-runtime.{region}.amazonaws.com")));
        }
    }

    #[test]
    fn test_special_characters_in_prompt() {
        let config = create_test_config();
        let provider = BedrockProvider::new(config).unwrap();

        let messages = vec![Message::user(
            "Hello! 你好 🌟 \n\t Special chars: @#$%^&*()",
        )];

        let prompt = provider.convert_messages_to_prompt(&messages);

        assert!(prompt.contains("Hello! 你好 🌟 \n\t Special chars: @#$%^&*()"));
        assert!(prompt.ends_with("Assistant: "));
    }

    #[test]
    fn test_long_conversation() {
        let config = create_test_config();
        let provider = BedrockProvider::new(config).unwrap();

        let mut messages = vec![];
        for i in 0..10 {
            messages.push(Message::user(&format!("User message {i}")));
            messages.push(Message::assistant(&format!("Assistant response {i}")));
        }
        messages.push(Message::user("Final question"));

        let prompt = provider.convert_messages_to_prompt(&messages);

        assert!(prompt.contains("Human: User message 0"));
        assert!(prompt.contains("Assistant: Assistant response 0"));
        assert!(prompt.contains("Human: Final question"));
        assert!(prompt.ends_with("Assistant: "));
    }
}
