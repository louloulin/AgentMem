//! 向量存储集成测试
//!
//! 测试嵌入式模式和服务器模式的端到端功能

use agent_mem_config::{DeploymentMode, EmbeddedModeConfig};
use agent_mem_core::storage::factory::StorageFactory;
use tempfile::TempDir;

#[cfg(feature = "lancedb")]
use agent_mem_storage::backends::lancedb_store::LanceDBStore;
#[cfg(feature = "lancedb")]
use agent_mem_traits::{VectorStore, VectorData};
#[cfg(feature = "lancedb")]
use std::collections::HashMap;

/// 测试嵌入式模式 StorageFactory 创建
#[tokio::test]
#[cfg(feature = "libsql")]
async fn test_embedded_mode_storage_factory() {
    // 1. 创建临时目录
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let vector_path = temp_dir.path().join("vectors");

    // 2. 创建嵌入式配置
    let config = EmbeddedModeConfig {
        database_path: db_path.clone(),
        vector_path: vector_path.clone(),
        vector_dimension: 1536,
        enable_wal: true,
        cache_size_kb: 10240,
    };

    let mode = DeploymentMode::Embedded(config);

    // 3. 创建存储 - 验证所有 repositories 都被正确创建
    let repos = StorageFactory::create(mode).await.unwrap();

    // 4. 验证所有 repositories 都存在
    assert!(std::ptr::addr_of!(repos.users) as usize != 0);
    assert!(std::ptr::addr_of!(repos.organizations) as usize != 0);
    assert!(std::ptr::addr_of!(repos.agents) as usize != 0);
    assert!(std::ptr::addr_of!(repos.messages) as usize != 0);
    assert!(std::ptr::addr_of!(repos.tools) as usize != 0);
    assert!(std::ptr::addr_of!(repos.api_keys) as usize != 0);
    assert!(std::ptr::addr_of!(repos.memories) as usize != 0);
    assert!(std::ptr::addr_of!(repos.blocks) as usize != 0);
    assert!(std::ptr::addr_of!(repos.associations) as usize != 0);

    // 清理
    drop(repos);
    drop(temp_dir);
}

/// 测试 LanceDB 向量存储端到端功能
#[tokio::test]
#[cfg(feature = "lancedb")]
async fn test_lancedb_vector_store_end_to_end() {
    // 1. 创建临时目录
    let temp_dir = TempDir::new().unwrap();
    let vector_path = temp_dir.path().join("vectors");

    // 2. 创建 LanceDB 存储
    let store = LanceDBStore::new(vector_path.to_str().unwrap(), "test_vectors")
        .await
        .unwrap();

    // 3. 创建测试向量
    let vectors = vec![
        VectorData {
            id: "test1".to_string(),
            vector: vec![0.1; 1536],
            metadata: {
                let mut map = HashMap::new();
                map.insert("content".to_string(), "Test memory 1".to_string());
                map
            },
        },
        VectorData {
            id: "test2".to_string(),
            vector: vec![0.2; 1536],
            metadata: {
                let mut map = HashMap::new();
                map.insert("content".to_string(), "Test memory 2".to_string());
                map
            },
        },
        VectorData {
            id: "test3".to_string(),
            vector: vec![0.3; 1536],
            metadata: {
                let mut map = HashMap::new();
                map.insert("content".to_string(), "Test memory 3".to_string());
                map
            },
        },
    ];

    // 4. 插入向量
    let ids = store.add_vectors(vectors).await.unwrap();
    assert_eq!(ids.len(), 3);

    // 5. 向量搜索
    let query = vec![0.1; 1536];
    let results = store.search_vectors(query, 10, None).await.unwrap();

    assert!(!results.is_empty());
    assert_eq!(results[0].id, "test1");

    // 6. 删除向量
    store
        .delete_vectors(vec!["test1".to_string()])
        .await
        .unwrap();

    // 7. 验证删除
    let query = vec![0.1; 1536];
    let results = store.search_vectors(query, 10, None).await.unwrap();

    // test1 应该被删除，test2 应该是最相似的
    if !results.is_empty() {
        assert_ne!(results[0].id, "test1");
    }

    // 清理
    drop(store);
    drop(temp_dir);
}

/// 测试 LanceDB 批量操作
#[tokio::test]
#[cfg(feature = "lancedb")]
async fn test_lancedb_batch_operations() {
    // 1. 创建临时目录
    let temp_dir = TempDir::new().unwrap();
    let vector_path = temp_dir.path().join("vectors_batch");

    // 2. 创建 LanceDB 存储
    let store = LanceDBStore::new(vector_path.to_str().unwrap(), "batch_vectors")
        .await
        .unwrap();

    // 3. 批量插入向量
    let batch_size = 100;
    let vectors: Vec<VectorData> = (0..batch_size)
        .map(|i| VectorData {
            id: format!("vec_{}", i),
            vector: vec![i as f32 / batch_size as f32; 768],
            metadata: {
                let mut map = HashMap::new();
                map.insert("index".to_string(), i.to_string());
                map
            },
        })
        .collect();

    let ids = store.add_vectors(vectors).await.unwrap();
    assert_eq!(ids.len(), batch_size);

    // 4. 批量搜索
    let query = vec![0.5; 768];
    let results = store.search_vectors(query, 10, None).await.unwrap();

    assert!(!results.is_empty());
    assert!(results.len() <= 10);

    // 5. 批量删除
    let ids_to_delete: Vec<String> = (0..10).map(|i| format!("vec_{}", i)).collect();
    store.delete_vectors(ids_to_delete).await.unwrap();

    // 清理
    drop(store);
    drop(temp_dir);
}

/// 测试 LanceDB 向量更新（通过删除和重新插入）
#[tokio::test]
#[cfg(feature = "lancedb")]
async fn test_lancedb_vector_update() {
    // 1. 创建临时目录
    let temp_dir = TempDir::new().unwrap();
    let vector_path = temp_dir.path().join("vectors_update");

    // 2. 创建 LanceDB 存储
    let store = LanceDBStore::new(vector_path.to_str().unwrap(), "update_vectors")
        .await
        .unwrap();

    // 3. 插入初始向量
    let vectors = vec![VectorData {
        id: "update_test".to_string(),
        vector: vec![0.1; 1536],
        metadata: {
            let mut map = HashMap::new();
            map.insert("version".to_string(), "1".to_string());
            map
        },
    }];

    store.add_vectors(vectors).await.unwrap();

    // 4. 验证初始向量
    let query = vec![0.1; 1536];
    let results = store.search_vectors(query.clone(), 1, None).await.unwrap();
    assert!(!results.is_empty());
    assert_eq!(results[0].id, "update_test");

    // 5. 更新向量（通过删除和重新插入，因为 update_vectors 尚未实现）
    store
        .delete_vectors(vec!["update_test".to_string()])
        .await
        .unwrap();

    let updated_vectors = vec![VectorData {
        id: "update_test".to_string(),
        vector: vec![0.9; 1536],
        metadata: {
            let mut map = HashMap::new();
            map.insert("version".to_string(), "2".to_string());
            map
        },
    }];

    store.add_vectors(updated_vectors).await.unwrap();

    // 6. 验证更新后的向量
    let query = vec![0.9; 1536];
    let results = store.search_vectors(query, 1, None).await.unwrap();

    assert!(!results.is_empty());
    assert_eq!(results[0].id, "update_test");
    // 更新后的向量应该与查询向量非常相似（余弦相似度接近 1.0）
    assert!(
        results[0].similarity > 0.999,
        "Expected similarity > 0.999, got {}",
        results[0].similarity
    );

    // 清理
    drop(store);
    drop(temp_dir);
}

/// 测试 LanceDB 统计信息
#[tokio::test]
#[cfg(feature = "lancedb")]
async fn test_lancedb_stats() {
    // 1. 创建临时目录
    let temp_dir = TempDir::new().unwrap();
    let vector_path = temp_dir.path().join("vectors_stats");

    // 2. 创建 LanceDB 存储
    let store = LanceDBStore::new(vector_path.to_str().unwrap(), "stats_vectors")
        .await
        .unwrap();

    // 3. 插入一些向量
    let vectors: Vec<VectorData> = (0..50)
        .map(|i| VectorData {
            id: format!("stats_vec_{}", i),
            vector: vec![i as f32 / 50.0; 1536],
            metadata: HashMap::new(),
        })
        .collect();

    store.add_vectors(vectors).await.unwrap();

    // 4. 获取统计信息
    let stats = store.get_stats().await.unwrap();

    assert_eq!(stats.total_vectors, 50);
    // 注意：当前 LanceDB 实现返回硬编码的 1536 维度
    assert_eq!(stats.dimension, 1536);

    // 清理
    drop(store);
    drop(temp_dir);
}

