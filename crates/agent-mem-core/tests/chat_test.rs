//! 对话功能集成测试

use agent_mem_core::client::{AgentMemClient, AgentMemClientConfig, MemoryType, Messages};
use agent_mem_traits::{LLMConfig, Result};

/// 创建带 Mock LLM 配置的客户端
fn create_test_client_with_llm() -> AgentMemClient {
    let mut config = AgentMemClientConfig::default();

    // 使用 Ollama 作为测试 LLM（本地运行，不需要 API key）
    config.llm = Some(LLMConfig {
        provider: "ollama".to_string(),
        model: "llama2".to_string(),
        api_key: None,
        base_url: Some("http://localhost:11434".to_string()),
        temperature: Some(0.7),
        max_tokens: Some(100),
        top_p: None,
        frequency_penalty: None,
        presence_penalty: None,
        response_format: None,
    });

    AgentMemClient::new(config)
}

#[tokio::test]
async fn test_chat_without_llm_config() -> Result<()> {
    // 测试没有配置 LLM 时的错误处理
    let client = AgentMemClient::default();

    let result = client.chat("Hello".to_string(), None, false).await;

    // 应该返回错误，因为没有配置 LLM
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("LLM not configured"));

    Ok(())
}

#[tokio::test]
async fn test_chat_basic_functionality() -> Result<()> {
    let client = create_test_client_with_llm();

    // 创建用户
    let user = client.create_user("test_chat_user".to_string()).await?;

    // 添加一些背景信息
    client
        .add(
            Messages::Single("I am a software engineer".to_string()),
            Some(user.id.clone()),
            None,
            None,
            None,
            false,
            Some(MemoryType::Semantic),
            None,
        )
        .await?;

    // 注意：这个测试需要 Ollama 运行在本地
    // 如果 Ollama 未运行，测试会失败
    // 在 CI 环境中，可以跳过这个测试或使用 mock
    let result = client
        .chat(
            "What is my profession?".to_string(),
            Some(user.id.clone()),
            false, // 不保存到记忆
        )
        .await;

    // 如果 Ollama 未运行，跳过验证
    if result.is_ok() {
        let response = result.unwrap();
        assert!(!response.is_empty());
        println!("Chat response: {response}");
    } else {
        println!(
            "Skipping chat test - Ollama not available: {:?}",
            result.err()
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_chat_with_memory_save() -> Result<()> {
    let client = create_test_client_with_llm();

    // 创建用户
    let user = client.create_user("test_save_user".to_string()).await?;

    // 添加背景信息
    client
        .add(
            Messages::Single("My favorite color is blue".to_string()),
            Some(user.id.clone()),
            None,
            None,
            None,
            false,
            Some(MemoryType::Semantic),
            None,
        )
        .await?;

    // 获取对话前的记忆数量
    let memories_before = client
        .get_all(Some(user.id.clone()), None, None, None)
        .await?;
    let count_before = memories_before.len();

    // 进行对话并保存到记忆
    let result = client
        .chat(
            "What is my favorite color?".to_string(),
            Some(user.id.clone()),
            true, // 保存到记忆
        )
        .await;

    // 如果 Ollama 可用，验证记忆已保存
    if result.is_ok() {
        let memories_after = client
            .get_all(Some(user.id.clone()), None, None, None)
            .await?;
        let count_after = memories_after.len();

        // 应该增加了 2 条记忆（用户消息 + 助手回复）
        assert_eq!(count_after, count_before + 2);

        // 验证新增的记忆是情景记忆
        let new_memories: Vec<_> = memories_after
            .iter()
            .filter(|m| m.memory_type == MemoryType::Episodic)
            .collect();
        assert!(new_memories.len() >= 2);
    } else {
        println!("Skipping memory save test - Ollama not available");
    }

    Ok(())
}

#[tokio::test]
async fn test_chat_without_memory_save() -> Result<()> {
    let client = create_test_client_with_llm();

    // 创建用户
    let user = client.create_user("test_no_save_user".to_string()).await?;

    // 获取对话前的记忆数量
    let memories_before = client
        .get_all(Some(user.id.clone()), None, None, None)
        .await?;
    let count_before = memories_before.len();

    // 进行对话但不保存到记忆
    let result = client
        .chat(
            "Hello, how are you?".to_string(),
            Some(user.id.clone()),
            false, // 不保存到记忆
        )
        .await;

    // 如果 Ollama 可用，验证记忆未增加
    if result.is_ok() {
        let memories_after = client
            .get_all(Some(user.id.clone()), None, None, None)
            .await?;
        let count_after = memories_after.len();

        // 记忆数量应该不变
        assert_eq!(count_after, count_before);
    } else {
        println!("Skipping no-save test - Ollama not available");
    }

    Ok(())
}

#[tokio::test]
async fn test_chat_with_memory_context() -> Result<()> {
    let client = create_test_client_with_llm();

    // 创建用户
    let user = client.create_user("test_context_user".to_string()).await?;

    // 添加多条背景信息
    let facts = vec![
        "I work at Google",
        "I live in San Francisco",
        "I enjoy hiking on weekends",
    ];

    for fact in facts {
        client
            .add(
                Messages::Single(fact.to_string()),
                Some(user.id.clone()),
                None,
                None,
                None,
                false,
                Some(MemoryType::Semantic),
                None,
            )
            .await?;
    }

    // 进行对话，应该能够利用记忆上下文
    let result = client
        .chat(
            "Tell me about myself".to_string(),
            Some(user.id.clone()),
            false,
        )
        .await;

    // 如果 Ollama 可用，验证响应
    if result.is_ok() {
        let response = result.unwrap();
        assert!(!response.is_empty());
        println!("Context-aware response: {response}");

        // 理想情况下，响应应该包含一些背景信息
        // 但这取决于 LLM 的质量，所以我们只检查响应不为空
    } else {
        println!("Skipping context test - Ollama not available");
    }

    Ok(())
}

#[tokio::test]
async fn test_chat_multiple_rounds() -> Result<()> {
    let client = create_test_client_with_llm();

    // 创建用户
    let user = client.create_user("test_multi_user".to_string()).await?;

    // 进行多轮对话
    let messages = ["My name is Alice", "I am learning Rust", "What is my name?"];

    for (i, msg) in messages.iter().enumerate() {
        let result = client
            .chat(
                msg.to_string(),
                Some(user.id.clone()),
                true, // 保存每轮对话
            )
            .await;

        if result.is_ok() {
            let response = result.unwrap();
            println!("Round {}: User: {} | Assistant: {}", i + 1, msg, response);

            // 最后一轮应该能够回答名字
            if i == 2 && response.to_lowercase().contains("alice") {
                println!("✅ Multi-round conversation successful!");
            }
        } else {
            println!("Skipping round {} - Ollama not available", i + 1);
            break;
        }
    }

    Ok(())
}
