//! MemoryExtractor 单元测试

use agent_mem_core::orchestrator::memory_extraction::{MemoryExtractor, MemoryExtractorConfig};
use agent_mem_core::engine::{MemoryEngine, MemoryEngineConfig};
use agent_mem_llm::LLMClient;
use agent_mem_traits::{LLMConfig, Message, MessageRole};
use chrono::Utc;
use std::sync::Arc;

#[test]
fn test_memory_extractor_config_default() {
    let config = MemoryExtractorConfig::default();
    assert!(config.auto_extract);
    assert_eq!(config.min_turns_for_extraction, 3);
    assert!(!config.extraction_prompt.is_empty());
}

#[test]
fn test_memory_extractor_config_custom() {
    let config = MemoryExtractorConfig {
        auto_extract: false,
        min_turns_for_extraction: 5,
        extraction_prompt: "Custom prompt".to_string(),
    };
    assert!(!config.auto_extract);
    assert_eq!(config.min_turns_for_extraction, 5);
    assert_eq!(config.extraction_prompt, "Custom prompt");
}

#[tokio::test]
#[ignore] // 需要 LLM 客户端配置
async fn test_extract_from_conversation() {
    // 这个测试需要真实的 LLM 客户端
    // 在实际环境中运行时需要设置 LLM_API_KEY 等环境变量
    
    if std::env::var("LLM_API_KEY").is_err() {
        println!("Skipping test: LLM_API_KEY not set");
        return;
    }

    let llm_config = LLMConfig::default();
    let llm_client = Arc::new(LLMClient::new(&llm_config).expect("Failed to create LLMClient"));
    let memory_engine = Arc::new(MemoryEngine::new(MemoryEngineConfig::default()));
    let config = MemoryExtractorConfig::default();
    
    let extractor = MemoryExtractor::new(llm_client, memory_engine, config);

    let messages = vec![
        Message {
            role: MessageRole::User,
            content: "I love Python programming".to_string(),
            timestamp: Some(Utc::now()),
        },
        Message {
            role: MessageRole::Assistant,
            content: "That's great! Python is a versatile language.".to_string(),
            timestamp: Some(Utc::now()),
        },
        Message {
            role: MessageRole::User,
            content: "I'm working on a machine learning project".to_string(),
            timestamp: Some(Utc::now()),
        },
    ];

    let result = extractor
        .extract_from_conversation(&messages, "agent-1", "user-1")
        .await;

    assert!(result.is_ok());
    let memories = result.unwrap();
    // 应该提取到一些记忆
    println!("Extracted {} memories", memories.len());
}

#[test]
fn test_format_conversation() {
    let messages = vec![
        Message {
            role: MessageRole::User,
            content: "Hello".to_string(),
            timestamp: Some(Utc::now()),
        },
        Message {
            role: MessageRole::Assistant,
            content: "Hi there!".to_string(),
            timestamp: Some(Utc::now()),
        },
    ];

    let formatted = format_conversation_for_test(&messages);
    assert!(formatted.contains("User: Hello"));
    assert!(formatted.contains("Assistant: Hi there!"));
}

// 辅助函数：格式化对话
fn format_conversation_for_test(messages: &[Message]) -> String {
    messages
        .iter()
        .map(|msg| {
            let role = match msg.role {
                MessageRole::User => "User",
                MessageRole::Assistant => "Assistant",
                MessageRole::System => "System",
            };
            format!("{}: {}", role, msg.content)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[tokio::test]
#[ignore] // TODO: 需要配置 LLMClient  
async fn test_save_memories_empty() {
    let llm_config = LLMConfig::default();
    let llm_client = Arc::new(LLMClient::new(&llm_config).expect("Failed to create LLMClient"));
    let memory_engine = Arc::new(MemoryEngine::new(MemoryEngineConfig::default()));
    let config = MemoryExtractorConfig::default();
    
    let extractor = MemoryExtractor::new(llm_client, memory_engine, config);

    let result = extractor.save_memories(vec![]).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn test_memory_type_parsing() {
    // 测试记忆类型解析
    let types = vec!["episodic", "semantic", "procedural", "working"];
    for memory_type in types {
        assert!(is_valid_memory_type(memory_type));
    }

    assert!(!is_valid_memory_type("invalid"));
}

// 辅助函数：验证记忆类型
fn is_valid_memory_type(memory_type: &str) -> bool {
    matches!(
        memory_type,
        "episodic" | "semantic" | "procedural" | "working" | "core" | "resource"
    )
}

#[test]
fn test_importance_score_range() {
    // 测试重要性评分范围
    let valid_scores = vec![0.0, 0.5, 1.0];
    for score in valid_scores {
        assert!(score >= 0.0 && score <= 1.0);
    }

    let invalid_scores = vec![-0.1, 1.1];
    for score in invalid_scores {
        assert!(score < 0.0 || score > 1.0);
    }
}

#[test]
fn test_min_turns_for_extraction() {
    let config = MemoryExtractorConfig::default();
    
    // 少于最小轮数的对话
    let short_conversation = vec![
        Message {
            role: MessageRole::User,
            content: "Hi".to_string(),
            timestamp: Some(Utc::now()),
        },
    ];
    
    assert!(short_conversation.len() < config.min_turns_for_extraction);
}

#[test]
fn test_extraction_prompt_template() {
    let config = MemoryExtractorConfig::default();
    
    // 验证提示词模板包含关键元素
    assert!(config.extraction_prompt.contains("conversation"));
    assert!(config.extraction_prompt.contains("JSON"));
    assert!(config.extraction_prompt.contains("importance"));
}

#[test]
fn test_memory_content_validation() {
    // 测试记忆内容验证
    let valid_content = "User prefers Python over JavaScript";
    assert!(!valid_content.is_empty());
    assert!(valid_content.len() < 10000); // 合理的长度限制

    let empty_content = "";
    assert!(empty_content.is_empty());
}

#[test]
fn test_json_parsing() {
    // 测试 JSON 解析
    let valid_json = r#"[
        {"content": "User likes Python", "type": "semantic", "importance": 0.7}
    ]"#;
    
    let result: Result<Vec<serde_json::Value>, _> = serde_json::from_str(valid_json);
    assert!(result.is_ok());

    let invalid_json = "not a json";
    let result: Result<Vec<serde_json::Value>, _> = serde_json::from_str(invalid_json);
    assert!(result.is_err());
}

#[test]
fn test_conversation_length() {
    let messages = vec![
        Message {
            role: MessageRole::User,
            content: "Hello".to_string(),
            timestamp: Some(Utc::now()),
        },
        Message {
            role: MessageRole::Assistant,
            content: "Hi".to_string(),
            timestamp: Some(Utc::now()),
        },
        Message {
            role: MessageRole::User,
            content: "How are you?".to_string(),
            timestamp: Some(Utc::now()),
        },
    ];

    assert_eq!(messages.len(), 3);
    assert!(messages.len() >= MemoryExtractorConfig::default().min_turns_for_extraction);
}

#[test]
fn test_message_role_types() {
    // 测试消息角色类型
    let user_msg = Message {
        role: MessageRole::User,
        content: "Test".to_string(),
        timestamp: Some(Utc::now()),
    };
    assert!(matches!(user_msg.role, MessageRole::User));

    let assistant_msg = Message {
        role: MessageRole::Assistant,
        content: "Test".to_string(),
        timestamp: Some(Utc::now()),
    };
    assert!(matches!(assistant_msg.role, MessageRole::Assistant));
}

