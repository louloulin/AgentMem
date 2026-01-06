//! 记忆可视化功能集成测试

use agent_mem_core::client::{AgentMemClient, MemoryType};
use agent_mem_traits::Result;

#[tokio::test]
async fn test_visualize_empty_memories() -> Result<()> {
    let client = AgentMemClient::default();

    // 创建用户
    let user = client.create_user("test_user".to_string()).await?;

    // 可视化空记忆
    let viz = client.visualize_memories(Some(user.id.clone())).await?;

    // 验证
    assert_eq!(viz.user_id, user.id);
    assert_eq!(viz.user_name, "test_user");
    assert_eq!(viz.summary.total_count, 0);
    assert_eq!(viz.summary.episodic_count, 0);
    assert_eq!(viz.summary.semantic_count, 0);
    assert_eq!(viz.summary.procedural_count, 0);
    assert_eq!(viz.summary.core_count, 0);
    assert_eq!(viz.summary.resource_count, 0);
    assert_eq!(viz.summary.knowledge_count, 0);
    assert_eq!(viz.summary.working_count, 0);
    assert_eq!(viz.summary.contextual_count, 0);

    assert!(viz.memories.episodic.is_empty());
    assert!(viz.memories.semantic.is_empty());
    assert!(viz.memories.procedural.is_empty());
    assert!(viz.memories.core.is_empty());
    assert!(viz.memories.resource.is_empty());
    assert!(viz.memories.knowledge.is_empty());
    assert!(viz.memories.working.is_empty());
    assert!(viz.memories.contextual.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_visualize_with_memories() -> Result<()> {
    let client = AgentMemClient::default();

    // 创建用户
    let user = client.create_user("alice".to_string()).await?;

    // 添加不同类型的记忆
    client
        .add_simple(
            "Episodic memory 1".to_string(),
            Some(user.id.clone()),
            None,
            Some(MemoryType::Episodic),
        )
        .await?;

    client
        .add_simple(
            "Episodic memory 2".to_string(),
            Some(user.id.clone()),
            None,
            Some(MemoryType::Episodic),
        )
        .await?;

    client
        .add_simple(
            "Semantic memory 1".to_string(),
            Some(user.id.clone()),
            None,
            Some(MemoryType::Semantic),
        )
        .await?;

    client
        .add_simple(
            "Procedural memory 1".to_string(),
            Some(user.id.clone()),
            None,
            Some(MemoryType::Procedural),
        )
        .await?;

    client
        .add_simple(
            "Core memory 1".to_string(),
            Some(user.id.clone()),
            None,
            Some(MemoryType::Core),
        )
        .await?;

    // 可视化记忆
    let viz = client.visualize_memories(Some(user.id.clone())).await?;

    // 验证摘要
    assert_eq!(viz.user_id, user.id);
    assert_eq!(viz.user_name, "alice");
    assert_eq!(viz.summary.total_count, 5);
    assert_eq!(viz.summary.episodic_count, 2);
    assert_eq!(viz.summary.semantic_count, 1);
    assert_eq!(viz.summary.procedural_count, 1);
    assert_eq!(viz.summary.core_count, 1);
    assert_eq!(viz.summary.resource_count, 0);
    assert_eq!(viz.summary.knowledge_count, 0);

    // 验证记忆内容
    assert_eq!(viz.memories.episodic.len(), 2);
    assert_eq!(viz.memories.semantic.len(), 1);
    assert_eq!(viz.memories.procedural.len(), 1);
    assert_eq!(viz.memories.core.len(), 1);
    assert!(viz.memories.resource.is_empty());
    assert!(viz.memories.knowledge.is_empty());

    // 验证记忆内容
    assert!(viz.memories.episodic[0].content.contains("Episodic memory"));
    assert!(viz.memories.semantic[0].content.contains("Semantic memory"));
    assert!(viz.memories.procedural[0]
        .content
        .contains("Procedural memory"));
    assert!(viz.memories.core[0].content.contains("Core memory"));

    Ok(())
}

#[tokio::test]
async fn test_visualize_all_memory_types() -> Result<()> {
    let client = AgentMemClient::default();

    // 创建用户
    let user = client.create_user("bob".to_string()).await?;

    // 添加所有类型的记忆
    client
        .add_simple(
            "Episodic".to_string(),
            Some(user.id.clone()),
            None,
            Some(MemoryType::Episodic),
        )
        .await?;

    client
        .add_simple(
            "Semantic".to_string(),
            Some(user.id.clone()),
            None,
            Some(MemoryType::Semantic),
        )
        .await?;

    client
        .add_simple(
            "Procedural".to_string(),
            Some(user.id.clone()),
            None,
            Some(MemoryType::Procedural),
        )
        .await?;

    client
        .add_simple(
            "Core".to_string(),
            Some(user.id.clone()),
            None,
            Some(MemoryType::Core),
        )
        .await?;

    client
        .add_simple(
            "Resource".to_string(),
            Some(user.id.clone()),
            None,
            Some(MemoryType::Resource),
        )
        .await?;

    client
        .add_simple(
            "Knowledge".to_string(),
            Some(user.id.clone()),
            None,
            Some(MemoryType::Knowledge),
        )
        .await?;

    client
        .add_simple(
            "Working".to_string(),
            Some(user.id.clone()),
            None,
            Some(MemoryType::Working),
        )
        .await?;

    client
        .add_simple(
            "Contextual".to_string(),
            Some(user.id.clone()),
            None,
            Some(MemoryType::Contextual),
        )
        .await?;

    // 可视化记忆
    let viz = client.visualize_memories(Some(user.id.clone())).await?;

    // 验证所有类型都有记忆
    assert_eq!(viz.summary.total_count, 8);
    assert_eq!(viz.summary.episodic_count, 1);
    assert_eq!(viz.summary.semantic_count, 1);
    assert_eq!(viz.summary.procedural_count, 1);
    assert_eq!(viz.summary.core_count, 1);
    assert_eq!(viz.summary.resource_count, 1);
    assert_eq!(viz.summary.knowledge_count, 1);
    assert_eq!(viz.summary.working_count, 1);
    assert_eq!(viz.summary.contextual_count, 1);

    // 验证每个类型都有一条记忆
    assert_eq!(viz.memories.episodic.len(), 1);
    assert_eq!(viz.memories.semantic.len(), 1);
    assert_eq!(viz.memories.procedural.len(), 1);
    assert_eq!(viz.memories.core.len(), 1);
    assert_eq!(viz.memories.resource.len(), 1);
    assert_eq!(viz.memories.knowledge.len(), 1);
    assert_eq!(viz.memories.working.len(), 1);
    assert_eq!(viz.memories.contextual.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_visualize_default_user() -> Result<()> {
    let client = AgentMemClient::default();

    // 可视化默认用户（无 user_id）
    let viz = client.visualize_memories(None).await?;

    // 验证
    assert_eq!(viz.user_id, "default");
    assert_eq!(viz.user_name, "Default");

    Ok(())
}

#[tokio::test]
async fn test_visualize_nonexistent_user() -> Result<()> {
    let client = AgentMemClient::default();

    // 可视化不存在的用户
    let viz = client
        .visualize_memories(Some("nonexistent_user".to_string()))
        .await?;

    // 验证
    assert_eq!(viz.user_id, "nonexistent_user");
    assert_eq!(viz.user_name, "Unknown");
    assert_eq!(viz.summary.total_count, 0);

    Ok(())
}

#[tokio::test]
async fn test_visualize_multiple_users() -> Result<()> {
    let client = AgentMemClient::default();

    // 创建两个用户
    let user1 = client.create_user("user1".to_string()).await?;
    let user2 = client.create_user("user2".to_string()).await?;

    // 为 user1 添加记忆
    client
        .add_simple(
            "User1 memory 1".to_string(),
            Some(user1.id.clone()),
            None,
            Some(MemoryType::Episodic),
        )
        .await?;

    client
        .add_simple(
            "User1 memory 2".to_string(),
            Some(user1.id.clone()),
            None,
            Some(MemoryType::Semantic),
        )
        .await?;

    // 为 user2 添加记忆
    client
        .add_simple(
            "User2 memory 1".to_string(),
            Some(user2.id.clone()),
            None,
            Some(MemoryType::Episodic),
        )
        .await?;

    // 可视化 user1 的记忆
    let viz1 = client.visualize_memories(Some(user1.id.clone())).await?;
    assert_eq!(viz1.user_name, "user1");
    assert_eq!(viz1.summary.total_count, 2);
    assert_eq!(viz1.summary.episodic_count, 1);
    assert_eq!(viz1.summary.semantic_count, 1);

    // 可视化 user2 的记忆
    let viz2 = client.visualize_memories(Some(user2.id.clone())).await?;
    assert_eq!(viz2.user_name, "user2");
    assert_eq!(viz2.summary.total_count, 1);
    assert_eq!(viz2.summary.episodic_count, 1);
    assert_eq!(viz2.summary.semantic_count, 0);

    Ok(())
}
