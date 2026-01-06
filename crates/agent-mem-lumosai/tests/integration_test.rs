//! LumosAI + AgentMem 集成测试

use agent_mem::Memory as AgentMemApi;
use agent_mem_lumosai::memory_adapter::AgentMemBackend;
use lumosai_core::llm::{Message as LumosMessage, Role as LumosRole};
use lumosai_core::memory::{Memory as LumosMemory, MemoryConfig};
use std::sync::Arc;

async fn create_test_memory() -> Arc<AgentMemApi> {
    let memory = AgentMemApi::new().await.expect("Failed to create Memory");
    Arc::new(memory)
}

#[tokio::test]
async fn test_memory_adapter_store() {
    let memory = create_test_memory().await;
    let backend = AgentMemBackend::new(
        memory.clone(),
        "test_agent".to_string(),
        "test_user".to_string(),
    );

    let message = LumosMessage {
        role: LumosRole::User,
        content: "Hello, this is a test message".to_string(),
        metadata: None,
        name: None,
    };

    let result = backend.store(&message).await;
    assert!(result.is_ok(), "Store should succeed: {:?}", result.err());
    println!("✅ Memory store test passed");
}

#[tokio::test]
async fn test_memory_adapter_retrieve() {
    let memory = create_test_memory().await;
    let backend = AgentMemBackend::new(
        memory.clone(),
        "test_agent_retrieve".to_string(),
        "test_user_retrieve".to_string(),
    );

    for i in 1..=3 {
        let message = LumosMessage {
            role: if i % 2 == 0 {
                LumosRole::Assistant
            } else {
                LumosRole::User
            },
            content: format!("Test message {}", i),
            metadata: None,
            name: None,
        };
        backend.store(&message).await.expect("Store should succeed");
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    let config = MemoryConfig {
        last_messages: Some(10),
        ..Default::default()
    };
    let result = backend.retrieve(&config).await;
    assert!(result.is_ok(), "Retrieve should succeed");

    let messages = result.unwrap();
    println!("Retrieved {} messages", messages.len());
    assert!(messages.len() <= 10);
    println!("✅ Memory retrieve test passed");
}
