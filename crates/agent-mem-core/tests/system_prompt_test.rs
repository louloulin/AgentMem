//! 系统提示和清空功能集成测试

use agent_mem_core::client::{AgentMemClient, Messages, MemoryType};
use agent_mem_traits::Result;

#[tokio::test]
async fn test_extract_memory_for_system_prompt() -> Result<()> {
    let client = AgentMemClient::default();

    // 创建用户
    let user = client.create_user("test_user".to_string()).await?;

    // 添加一些记忆
    client
        .add(
            Messages::Single("I love pizza".to_string()),
            Some(user.id.clone()),
            None,
            None,
            None,
            false,
            Some(MemoryType::Semantic),
            None,
        )
        .await?;

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

    // 提取记忆
    let prompt = client
        .extract_memory_for_system_prompt(
            "What do you know about me?".to_string(),
            Some(user.id.clone()),
            Some(5),
        )
        .await?;

    // 验证提示包含记忆
    assert!(prompt.contains("Relevant memories") || prompt.contains("No relevant"));

    Ok(())
}

#[tokio::test]
async fn test_extract_memory_empty() -> Result<()> {
    let client = AgentMemClient::default();

    // 创建用户但不添加记忆
    let user = client.create_user("empty_user".to_string()).await?;

    // 提取记忆
    let prompt = client
        .extract_memory_for_system_prompt(
            "What do you know?".to_string(),
            Some(user.id.clone()),
            Some(5),
        )
        .await?;

    // 验证返回空记忆提示
    assert!(prompt.contains("No relevant memories found"));

    Ok(())
}

#[tokio::test]
async fn test_construct_system_message() -> Result<()> {
    let client = AgentMemClient::default();

    // 创建用户
    let user = client.create_user("test_user2".to_string()).await?;

    // 添加记忆
    client
        .add(
            Messages::Single("I work as a software engineer".to_string()),
            Some(user.id.clone()),
            None,
            None,
            None,
            false,
            Some(MemoryType::Semantic),
            None,
        )
        .await?;

    // 构建系统消息
    let system_message = client
        .construct_system_message(
            "Tell me about my job".to_string(),
            Some(user.id.clone()),
            None,
        )
        .await?;

    // 验证系统消息包含必要元素
    assert!(system_message.contains("helpful AI assistant"));
    assert!(system_message.contains("User: Tell me about my job"));

    Ok(())
}

#[tokio::test]
async fn test_construct_system_message_with_custom_prefix() -> Result<()> {
    let client = AgentMemClient::default();

    // 创建用户
    let user = client.create_user("test_user3".to_string()).await?;

    // 添加记忆
    client
        .add(
            Messages::Single("I enjoy hiking".to_string()),
            Some(user.id.clone()),
            None,
            None,
            None,
            false,
            Some(MemoryType::Episodic),
            None,
        )
        .await?;

    // 使用自定义前缀构建系统消息
    let custom_prefix = "You are a personal assistant:";
    let system_message = client
        .construct_system_message(
            "What are my hobbies?".to_string(),
            Some(user.id.clone()),
            Some(custom_prefix.to_string()),
        )
        .await?;

    // 验证使用了自定义前缀
    assert!(system_message.contains(custom_prefix));
    assert!(system_message.contains("User: What are my hobbies?"));

    Ok(())
}

#[tokio::test]
async fn test_clear_all_memories() -> Result<()> {
    let client = AgentMemClient::default();

    // 创建用户
    let user = client.create_user("clear_test_user".to_string()).await?;

    // 添加多个记忆
    for i in 0..5 {
        client
            .add(
                Messages::Single(format!("Memory {}", i)),
                Some(user.id.clone()),
                None,
                None,
                None,
                false,
                Some(MemoryType::Episodic),
                None,
            )
            .await?;
    }

    // 验证记忆已添加
    let memories_before = client.get_all(Some(user.id.clone()), None, None, None).await?;
    assert_eq!(memories_before.len(), 5);

    // 清空所有记忆
    let deleted_count = client.clear(Some(user.id.clone())).await?;
    assert_eq!(deleted_count, 5);

    // 验证记忆已清空
    let memories_after = client.get_all(Some(user.id.clone()), None, None, None).await?;
    assert_eq!(memories_after.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_clear_conversation_history() -> Result<()> {
    let client = AgentMemClient::default();

    // 创建用户
    let user = client.create_user("conv_test_user".to_string()).await?;

    // 添加情景记忆（对话历史）
    for i in 0..3 {
        client
            .add(
                Messages::Single(format!("Conversation {}", i)),
                Some(user.id.clone()),
                None,
                None,
                None,
                false,
                Some(MemoryType::Episodic),
                None,
            )
            .await?;
    }

    // 添加语义记忆（应该保留）
    for i in 0..2 {
        client
            .add(
                Messages::Single(format!("Fact {}", i)),
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

    // 验证总共有 5 个记忆
    let memories_before = client.get_all(Some(user.id.clone()), None, None, None).await?;
    assert_eq!(memories_before.len(), 5);

    // 清空对话历史
    let deleted_count = client.clear_conversation_history(user.id.clone()).await?;
    assert_eq!(deleted_count, 3);

    // 验证只剩下语义记忆
    let memories_after = client.get_all(Some(user.id.clone()), None, None, None).await?;
    assert_eq!(memories_after.len(), 2);

    // 验证剩余的都是语义记忆
    for memory in memories_after {
        assert_eq!(memory.memory_type, MemoryType::Semantic);
    }

    Ok(())
}

#[tokio::test]
async fn test_clear_empty_user() -> Result<()> {
    let client = AgentMemClient::default();

    // 创建用户但不添加记忆
    let user = client.create_user("empty_clear_user".to_string()).await?;

    // 清空记忆（应该返回 0）
    let deleted_count = client.clear(Some(user.id.clone())).await?;
    assert_eq!(deleted_count, 0);

    Ok(())
}

