// Phase 0 持久化验证测试
// 验证 ag25.md Phase 0 的核心功能：LibSQL持久化

use agent_mem_core::storage::libsql::{LibSqlConnectionManager, LibSqlMemoryRepository, LibSqlMemoryOperations};
use agent_mem_core::operations::MemoryOperations;
use agent_mem_traits::{MemoryV4 as Memory, Content, AttributeKey, AttributeValue, AttributeSet};

#[tokio::test]
async fn test_phase0_libsql_persistence() {
    // Step 1: 创建测试数据库连接
    let test_db = "./test_data/phase0_test.db";
    let _ = std::fs::remove_file(test_db); // 清理旧数据
    
    let conn_mgr = LibSqlConnectionManager::new(test_db)
        .await
        .expect("Failed to create connection manager");
    
    let conn = conn_mgr.get_connection()
        .await
        .expect("Failed to get connection");
    
    // Step 2: 创建 LibSqlMemoryOperations
    let repo = LibSqlMemoryRepository::new(conn);
    let mut operations = LibSqlMemoryOperations::new(repo);
    
    // Step 3: 创建测试记忆
    let mut attributes = AttributeSet::new();
    attributes.insert(
        AttributeKey::core("agent_id"),
        AttributeValue::String("test_agent".to_string())
    );
    attributes.insert(
        AttributeKey::core("user_id"),
        AttributeValue::String("test_user".to_string())
    );
    attributes.insert(
        AttributeKey::core("memory_type"),
        AttributeValue::String("episodic".to_string())
    );
    attributes.insert(
        AttributeKey::system("importance"),
        AttributeValue::Number(0.8)
    );
    
    let memory = Memory {
        id: agent_mem_traits::MemoryId::new(),
        content: Content::text("Phase 0 测试记忆"),
        attributes,
        relations: agent_mem_traits::RelationGraph::new(),
        metadata: agent_mem_traits::Metadata::default(),
    };
    
    let memory_id = memory.id.0.clone();
    
    // Step 4: 写入记忆
    let created_id = operations.create_memory(memory.clone())
        .await
        .expect("Failed to create memory");
    
    assert_eq!(created_id, memory_id, "返回的ID应该与创建的ID一致");
    
    // Step 5: 读取记忆（验证持久化）
    let retrieved = operations.get_memory(&memory_id)
        .await
        .expect("Failed to get memory")
        .expect("Memory should exist");
    
    assert_eq!(retrieved.id.0, memory_id);
    assert_eq!(retrieved.content.as_text(), Some("Phase 0 测试记忆"));
    assert_eq!(retrieved.agent_id(), Some("test_agent".to_string()));
    
    println!("✅ Phase 0 持久化测试通过！");
    println!("   - LibSqlMemoryOperations 创建成功");
    println!("   - 记忆写入 SQLite 成功");
    println!("   - 记忆读取验证成功");
}

#[tokio::test]
async fn test_phase0_batch_persistence() {
    // 测试批量操作
    let test_db = "./test_data/phase0_batch_test.db";
    let _ = std::fs::remove_file(test_db);
    
    let conn_mgr = LibSqlConnectionManager::new(test_db)
        .await
        .expect("Failed to create connection manager");
    
    let conn = conn_mgr.get_connection()
        .await
        .expect("Failed to get connection");
    
    let repo = LibSqlMemoryRepository::new(conn);
    let mut operations = LibSqlMemoryOperations::new(repo);
    
    // 创建多个记忆
    let mut memories = Vec::new();
    for i in 0..5 {
        let mut attributes = AttributeSet::new();
        attributes.insert(
            AttributeKey::core("agent_id"),
            AttributeValue::String("batch_agent".to_string())
        );
        attributes.insert(
            AttributeKey::core("memory_type"),
            AttributeValue::String("semantic".to_string())
        );
        
        let memory = Memory {
            id: agent_mem_traits::MemoryId::new(),
            content: Content::text(format!("Batch memory {}", i)),
            attributes,
            relations: agent_mem_traits::RelationGraph::new(),
            metadata: agent_mem_traits::Metadata::default(),
        };
        memories.push(memory);
    }
    
    // 批量写入
    let ids = operations.batch_create_memories(memories.clone())
        .await
        .expect("Failed to batch create memories");
    
    assert_eq!(ids.len(), 5, "应该创建5条记忆");
    
    // 验证读取
    let agent_memories = operations.get_agent_memories("batch_agent", Some(10))
        .await
        .expect("Failed to get agent memories");
    
    assert_eq!(agent_memories.len(), 5, "应该检索到5条记忆");
    
    println!("✅ Phase 0 批量操作测试通过！");
}
