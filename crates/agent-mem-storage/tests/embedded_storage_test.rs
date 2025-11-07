//! 嵌入式存储集成测试
//!
//! 测试 LibSQL + LanceDB 嵌入式存储方案

use agent_mem_storage::backends::{LibSQLStore, MemoryVectorStore};
use agent_mem_traits::{VectorData, VectorStore};
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_libsql_basic_operations() {
    // 创建临时目录
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db_path_str = db_path.to_str().unwrap();

    // 创建 LibSQL 存储
    let store = LibSQLStore::new(db_path_str).await.unwrap();

    // 测试插入
    let record = agent_mem_storage::backends::libsql_store::MemoryRecord {
        id: "test-1".to_string(),
        agent_id: "agent-1".to_string(),
        user_id: Some("user-1".to_string()),
        content: "Test memory content".to_string(),
        memory_type: "episodic".to_string(),
        importance: 0.8,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        metadata: HashMap::new(),
    };

    store.insert(&record).await.unwrap();

    // 测试查询
    let retrieved = store.get("test-1").await.unwrap();
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.id, "test-1");
    assert_eq!(retrieved.content, "Test memory content");
    assert_eq!(retrieved.importance, 0.8);

    // 测试更新
    let mut updated_record = retrieved.clone();
    updated_record.content = "Updated content".to_string();
    updated_record.importance = 0.9;
    store.update(&updated_record).await.unwrap();

    let retrieved = store.get("test-1").await.unwrap().unwrap();
    assert_eq!(retrieved.content, "Updated content");
    assert_eq!(retrieved.importance, 0.9);

    // 测试计数
    let count = store.count().await.unwrap();
    assert_eq!(count, 1);

    // 测试删除
    let deleted = store.delete("test-1").await.unwrap();
    assert!(deleted);

    let count = store.count().await.unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn test_libsql_search() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_search.db");
    let db_path_str = db_path.to_str().unwrap();

    let store = LibSQLStore::new(db_path_str).await.unwrap();

    // 插入多条记录
    for i in 1..=5 {
        let record = agent_mem_storage::backends::libsql_store::MemoryRecord {
            id: format!("test-{}", i),
            agent_id: "agent-1".to_string(),
            user_id: Some("user-1".to_string()),
            content: format!("Memory content {}", i),
            memory_type: "episodic".to_string(),
            importance: 0.5 + (i as f32 * 0.1),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        };
        store.insert(&record).await.unwrap();
    }

    // 测试搜索
    let results = store
        .search("agent-1", Some("user-1"), None, Some(3))
        .await
        .unwrap();
    assert_eq!(results.len(), 3);

    // 验证按重要性排序
    assert!(results[0].importance >= results[1].importance);
    assert!(results[1].importance >= results[2].importance);
}

#[tokio::test]
async fn test_libsql_memory_mode() {
    // 测试内存模式
    let store = LibSQLStore::new(":memory:").await.unwrap();

    let record = agent_mem_storage::backends::libsql_store::MemoryRecord {
        id: "mem-1".to_string(),
        agent_id: "agent-1".to_string(),
        user_id: None,
        content: "In-memory test".to_string(),
        memory_type: "semantic".to_string(),
        importance: 0.7,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        metadata: HashMap::new(),
    };

    store.insert(&record).await.unwrap();
    let retrieved = store.get("mem-1").await.unwrap();
    assert!(retrieved.is_some());
}

#[tokio::test]
async fn test_memory_vector_store_basic() {
    // 测试内存向量存储
    let store = MemoryVectorStore::new();

    // 创建测试向量
    let vectors = vec![
        VectorData {
            id: "vec-1".to_string(),
            vector: vec![1.0, 0.0, 0.0],
            metadata: HashMap::new(),
        },
        VectorData {
            id: "vec-2".to_string(),
            vector: vec![0.0, 1.0, 0.0],
            metadata: HashMap::new(),
        },
        VectorData {
            id: "vec-3".to_string(),
            vector: vec![0.0, 0.0, 1.0],
            metadata: HashMap::new(),
        },
    ];

    // 测试添加向量
    let ids = store.add_vectors(vectors.clone()).await.unwrap();
    assert_eq!(ids.len(), 3);

    // 测试计数
    let count = store.count_vectors().await.unwrap();
    assert_eq!(count, 3);

    // 测试获取向量
    let vec = store.get_vector("vec-1").await.unwrap();
    assert!(vec.is_some());
    let vec = vec.unwrap();
    assert_eq!(vec.id, "vec-1");
    assert_eq!(vec.vector, vec![1.0, 0.0, 0.0]);

    // 测试向量搜索
    let query = vec![1.0, 0.0, 0.0];
    let results = store.search_vectors(query, 2, None).await.unwrap();
    assert_eq!(results.len(), 2);
    // 第一个结果应该是 vec-1 (完全匹配)
    assert_eq!(results[0].id, "vec-1");

    // 测试删除
    store
        .delete_vectors(vec!["vec-1".to_string()])
        .await
        .unwrap();
    let count = store.count_vectors().await.unwrap();
    assert_eq!(count, 2);
}

#[tokio::test]
async fn test_memory_vector_store_search_with_filters() {
    let store = MemoryVectorStore::new();

    // 创建带元数据的向量
    let mut metadata1 = HashMap::new();
    metadata1.insert("category".to_string(), "A".to_string());

    let mut metadata2 = HashMap::new();
    metadata2.insert("category".to_string(), "B".to_string());

    let vectors = vec![
        VectorData {
            id: "vec-1".to_string(),
            vector: vec![1.0, 0.0],
            metadata: metadata1,
        },
        VectorData {
            id: "vec-2".to_string(),
            vector: vec![0.9, 0.1],
            metadata: metadata2,
        },
    ];

    store.add_vectors(vectors).await.unwrap();

    // 测试带过滤器的搜索
    let mut filters = HashMap::new();
    filters.insert("category".to_string(), serde_json::json!("A"));

    let query = vec![1.0, 0.0];
    let results = store
        .search_with_filters(query, 10, &filters, None)
        .await
        .unwrap();

    // 应该只返回 category=A 的向量
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "vec-1");
}

#[tokio::test]
async fn test_libsql_concurrent_operations() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_concurrent.db");
    let db_path_str = db_path.to_str().unwrap();

    let store = Arc::new(LibSQLStore::new(db_path_str).await.unwrap());

    // 并发插入
    let mut handles = vec![];
    for i in 0..10 {
        let store_clone = Arc::clone(&store);
        let handle = tokio::spawn(async move {
            let record = agent_mem_storage::backends::libsql_store::MemoryRecord {
                id: format!("concurrent-{}", i),
                agent_id: "agent-1".to_string(),
                user_id: Some("user-1".to_string()),
                content: format!("Concurrent content {}", i),
                memory_type: "episodic".to_string(),
                importance: 0.5,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                metadata: HashMap::new(),
            };
            store_clone.insert(&record).await
        });
        handles.push(handle);
    }

    // 等待所有任务完成
    for handle in handles {
        handle.await.unwrap().unwrap();
    }

    // 验证所有记录都已插入
    let count = store.count().await.unwrap();
    assert_eq!(count, 10);
}

#[tokio::test]
async fn test_libsql_clear() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_clear.db");
    let db_path_str = db_path.to_str().unwrap();

    let store = LibSQLStore::new(db_path_str).await.unwrap();

    // 插入一些记录
    for i in 1..=5 {
        let record = agent_mem_storage::backends::libsql_store::MemoryRecord {
            id: format!("test-{}", i),
            agent_id: "agent-1".to_string(),
            user_id: Some("user-1".to_string()),
            content: format!("Content {}", i),
            memory_type: "episodic".to_string(),
            importance: 0.5,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        };
        store.insert(&record).await.unwrap();
    }

    let count = store.count().await.unwrap();
    assert_eq!(count, 5);

    // 清空
    store.clear().await.unwrap();

    let count = store.count().await.unwrap();
    assert_eq!(count, 0);
}
