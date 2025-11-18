//! LumosAI + AgentMem 集成测试

use agent_mem_core::engine::{MemoryEngine, MemoryEngineConfig};
use agent_mem_lumosai::memory_adapter::AgentMemBackend;
use agent_mem_traits::{AttributeKey, AttributeValue, Content, MemoryId};
use agent_mem_traits::abstractions::Memory as AgentMemMemory;
use lumosai_core::llm::{Message as LumosMessage, Role as LumosRole};
use lumosai_core::memory::{Memory as LumosMemory, MemoryConfig};
use std::sync::Arc;
use chrono::Utc;

/// 创建测试用的 MemoryEngine
async fn create_test_memory_engine() -> Arc<MemoryEngine> {
    let config = MemoryEngineConfig::default();
    let engine = MemoryEngine::new(config);
    Arc::new(engine)
}


#[tokio::test]
async fn test_memory_adapter_store() {
    // 创建 MemoryEngine
    let engine = create_test_memory_engine().await;
    
    // 创建 AgentMemBackend
    let backend = AgentMemBackend::new(
        engine.clone(),
        "test_agent".to_string(),
        "test_user".to_string(),
    );
    
    // 创建测试消息
    let message = LumosMessage {
        role: LumosRole::User,
        content: "Hello, this is a test message".to_string(),
        metadata: None,
        name: None,
    };
    
    // 测试 store 方法
    let result = backend.store(&message).await;
    assert!(result.is_ok(), "Store should succeed: {:?}", result.err());
    
    // 验证记忆已保存
    let memories = engine.search_memories(
        "test message",
        Some(agent_mem_core::hierarchy::MemoryScope::User {
            agent_id: "test_agent".to_string(),
            user_id: "test_user".to_string(),
        }),
        Some(10),
    ).await.expect("Search should succeed");
    
    assert!(!memories.is_empty(), "Should find stored memory");
    assert!(memories[0].content.contains("test message"));
}

#[tokio::test]
async fn test_memory_adapter_retrieve() {
    let engine = create_test_memory_engine().await;
    
    let backend = AgentMemBackend::new(
        engine.clone(),
        "test_agent".to_string(),
        "test_user".to_string(),
    );
    
    // 先存储一些测试数据
    let test_messages = vec![
        ("Hello", LumosRole::User),
        ("Hi there!", LumosRole::Assistant),
        ("How are you?", LumosRole::User),
    ];
    
    for (content, role) in test_messages {
        let message = LumosMessage {
            role,
            content: content.to_string(),
            metadata: None,
            name: None,
        };
        backend.store(&message).await.expect("Store should succeed");
    }
    
    // 测试 retrieve 方法
    let config = MemoryConfig {
        query: Some("Hello".to_string()),
        ..Default::default()
    };
    
    let result = backend.retrieve(&config).await;
    assert!(result.is_ok(), "Retrieve should succeed: {:?}", result.err());
    
    let messages = result.unwrap();
    assert!(!messages.is_empty(), "Should retrieve messages");
    
    // 验证消息内容
    let has_hello = messages.iter().any(|m| m.content.contains("Hello"));
    assert!(has_hello, "Should find message with 'Hello'");
}

#[tokio::test]
async fn test_memory_adapter_all_content_types() {
    let engine = create_test_memory_engine().await;
    
    // 直接测试所有 Content 类型的处理
    let mut memory = AgentMemMemory {
        id: MemoryId::new(),
        content: Content::Text("text content".to_string()),
        attributes: Default::default(),
        relations: Default::default(),
        metadata: Default::default(),
    };
    
    memory.attributes.set(
        AttributeKey::core("agent_id"),
        AttributeValue::String("test_agent".to_string())
    );
    
    engine.add_memory(memory.clone()).await.expect("Should store text content");
    
    // 测试 Structured content
    memory.id = MemoryId::new();
    memory.content = Content::Structured(serde_json::json!({"key": "value"}));
    engine.add_memory(memory.clone()).await.expect("Should store structured content");
    
    // 测试 Vector content
    memory.id = MemoryId::new();
    memory.content = Content::Vector(vec![0.1, 0.2, 0.3]);
    engine.add_memory(memory.clone()).await.expect("Should store vector content");
    
    // 测试 Binary content
    memory.id = MemoryId::new();
    memory.content = Content::Binary(vec![1, 2, 3, 4]);
    engine.add_memory(memory.clone()).await.expect("Should store binary content");
    
    // 验证所有类型都能正确存储
    let all_memories = engine.search_memories("", None, Some(10))
        .await.expect("Should search memories");
    
    assert!(all_memories.len() >= 4, "Should have all content types stored");
}

#[tokio::test]
async fn test_memory_adapter_all_roles() {
    let engine = create_test_memory_engine().await;
    
    let backend = AgentMemBackend::new(
        engine.clone(),
        "test_agent".to_string(),
        "test_user".to_string(),
    );
    
    // 测试所有 Role 类型
    let roles = vec![
        LumosRole::System,
        LumosRole::User,
        LumosRole::Assistant,
        LumosRole::Tool,
        LumosRole::Function,
        LumosRole::Custom("custom_role".to_string()),
    ];
    
    for role in roles {
        let message = LumosMessage {
            role: role.clone(),
            content: format!("Test message for {:?}", role),
            metadata: None,
            name: None,
        };
        
        let result = backend.store(&message).await;
        assert!(result.is_ok(), "Should store message with role {:?}", role);
    }
    
    // 验证所有角色的消息都已存储
    let all_memories = engine.search_memories("Test message", None, Some(20))
        .await.expect("Should search memories");
    
    assert!(all_memories.len() >= 6, "Should have messages for all roles");
}

#[tokio::test]
async fn test_agent_factory_basic() {
    // 这个测试需要真实的 Repositories，暂时跳过
    // 在实际环境中需要设置数据库
    
    // TODO: 添加 AgentFactory 的集成测试
    // 需要 mock Repositories 或使用测试数据库
}

#[tokio::test]
async fn test_memory_persistence() {
    let engine = create_test_memory_engine().await;
    
    let backend = AgentMemBackend::new(
        engine.clone(),
        "persist_agent".to_string(),
        "persist_user".to_string(),
    );
    
    // 存储消息
    let message = LumosMessage {
        role: LumosRole::User,
        content: "This should persist".to_string(),
        metadata: None,
        name: None,
    };
    
    backend.store(&message).await.expect("Store should succeed");
    
    // 创建新的 backend 实例（模拟应用重启）
    let backend2 = AgentMemBackend::new(
        engine.clone(),
        "persist_agent".to_string(),
        "persist_user".to_string(),
    );
    
    // 验证可以检索到之前存储的数据
    let config = MemoryConfig {
        query: Some("persist".to_string()),
        ..Default::default()
    };
    
    let messages = backend2.retrieve(&config).await.expect("Retrieve should succeed");
    assert!(!messages.is_empty(), "Should retrieve persisted message");
    assert!(messages[0].content.contains("persist"));
}

#[tokio::test]
async fn test_concurrent_operations() {
    let engine = create_test_memory_engine().await;
    
    let backend = Arc::new(AgentMemBackend::new(
        engine.clone(),
        "concurrent_agent".to_string(),
        "concurrent_user".to_string(),
    ));
    
    // 并发存储多个消息
    let mut handles = vec![];
    
    for i in 0..10 {
        let backend_clone = backend.clone();
        let handle = tokio::spawn(async move {
            let message = LumosMessage {
                role: LumosRole::User,
                content: format!("Concurrent message {}", i),
                metadata: None,
                name: None,
            };
            backend_clone.store(&message).await
        });
        handles.push(handle);
    }
    
    // 等待所有任务完成
    for handle in handles {
        let result = handle.await.expect("Task should complete");
        assert!(result.is_ok(), "Concurrent store should succeed");
    }
    
    // 验证所有消息都已存储
    let config = MemoryConfig {
        query: Some("Concurrent".to_string()),
        ..Default::default()
    };
    
    let messages = backend.retrieve(&config).await.expect("Retrieve should succeed");
    assert!(messages.len() >= 10, "Should have all concurrent messages");
}
