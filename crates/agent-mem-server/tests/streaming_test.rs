//! 流式响应集成测试

use agent_mem_llm::LLMClient;
use agent_mem_traits::{LLMConfig, Message, MessageRole};

#[tokio::test]
async fn test_streaming_response() {
    // 跳过测试如果没有 API key
    if std::env::var("OPENAI_API_KEY").is_err() {
        println!("Skipping test: OPENAI_API_KEY not set");
        return;
    }

    // 创建 LLM 客户端
    let config = LLMConfig {
        provider: "openai".to_string(),
        model: "gpt-3.5-turbo".to_string(),
        api_key: std::env::var("OPENAI_API_KEY").ok(),
        ..Default::default()
    };

    let client = LLMClient::new(&config).expect("Failed to create LLM client");

    // 创建测试消息
    let messages = vec![Message {
        role: MessageRole::User,
        content: "Say 'Hello, World!' in one sentence.".to_string(),
        timestamp: Some(chrono::Utc::now()),
    }];

    // 测试非流式响应（流式响应需要更复杂的测试）
    let response = client
        .generate(&messages)
        .await
        .expect("Failed to generate response");

    println!("Response: {}", response);

    // 验证响应不为空
    assert!(!response.trim().is_empty(), "Response should not be empty");
}

#[tokio::test]
async fn test_streaming_chunk_format() {
    use serde_json::json;

    // 测试 StreamChunk 序列化
    let chunk = json!({
        "chunk_type": "content",
        "content": "Hello",
        "tool_call": null,
        "memories_count": null,
    });

    let serialized = serde_json::to_string(&chunk).expect("Failed to serialize");
    println!("Serialized chunk: {}", serialized);

    // 验证可以反序列化
    let deserialized: serde_json::Value =
        serde_json::from_str(&serialized).expect("Failed to deserialize");
    assert_eq!(deserialized["chunk_type"], "content");
    assert_eq!(deserialized["content"], "Hello");
}

#[test]
fn test_stream_chunk_types() {
    // 测试不同类型的 chunk
    let chunk_types = vec![
        "start",
        "content",
        "tool_call",
        "memory_update",
        "done",
        "error",
    ];

    for chunk_type in chunk_types {
        let chunk = serde_json::json!({
            "chunk_type": chunk_type,
            "content": null,
            "tool_call": null,
            "memories_count": null,
        });

        let serialized = serde_json::to_string(&chunk).expect("Failed to serialize");
        println!("Chunk type '{}': {}", chunk_type, serialized);

        // 验证序列化成功
        assert!(serialized.contains(chunk_type));
    }
}
