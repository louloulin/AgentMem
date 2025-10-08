//! 内存向量存储测试
//!
//! 测试 MemoryVectorStore 的基本功能

use agent_mem_storage::backends::MemoryVectorStore;
use agent_mem_traits::{VectorData, VectorStore, VectorStoreConfig};
use std::collections::HashMap;

fn create_test_config() -> VectorStoreConfig {
    VectorStoreConfig {
        provider: "memory".to_string(),
        path: ":memory:".to_string(),
        table_name: "test_vectors".to_string(),
        dimension: Some(3),
        api_key: None,
        index_name: None,
        url: None,
        collection_name: None,
    }
}

#[tokio::test]
async fn test_memory_vector_basic_operations() {
    let store = MemoryVectorStore::new(create_test_config()).await.unwrap();

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
    assert_eq!(ids[0], "vec-1");
    assert_eq!(ids[1], "vec-2");
    assert_eq!(ids[2], "vec-3");

    // 测试计数
    let count = store.count_vectors().await.unwrap();
    assert_eq!(count, 3);

    // 测试获取向量
    let vec = store.get_vector("vec-1").await.unwrap();
    assert!(vec.is_some());
    let vec = vec.unwrap();
    assert_eq!(vec.id, "vec-1");
    assert_eq!(vec.vector, vec![1.0, 0.0, 0.0]);

    // 测试获取不存在的向量
    let vec = store.get_vector("non-existent").await.unwrap();
    assert!(vec.is_none());
}

#[tokio::test]
async fn test_memory_vector_search() {
    let store = MemoryVectorStore::new(create_test_config()).await.unwrap();

    // 创建测试向量
    let vectors = vec![
        VectorData {
            id: "vec-1".to_string(),
            vector: vec![1.0, 0.0, 0.0],
            metadata: HashMap::new(),
        },
        VectorData {
            id: "vec-2".to_string(),
            vector: vec![0.9, 0.1, 0.0],
            metadata: HashMap::new(),
        },
        VectorData {
            id: "vec-3".to_string(),
            vector: vec![0.0, 1.0, 0.0],
            metadata: HashMap::new(),
        },
    ];

    store.add_vectors(vectors).await.unwrap();

    // 测试向量搜索
    let query = vec![1.0, 0.0, 0.0];
    let results = store.search_vectors(query, 2, None).await.unwrap();
    
    assert_eq!(results.len(), 2);
    // 第一个结果应该是 vec-1 (完全匹配)
    assert_eq!(results[0].id, "vec-1");
    assert!(results[0].similarity > 0.99);
    
    // 第二个结果应该是 vec-2 (相似度次高)
    assert_eq!(results[1].id, "vec-2");
    assert!(results[1].similarity > 0.8);
}

#[tokio::test]
async fn test_memory_vector_search_with_threshold() {
    let store = MemoryVectorStore::new(create_test_config()).await.unwrap();

    let vectors = vec![
        VectorData {
            id: "vec-1".to_string(),
            vector: vec![1.0, 0.0, 0.0],
            metadata: HashMap::new(),
        },
        VectorData {
            id: "vec-2".to_string(),
            vector: vec![0.5, 0.5, 0.0],
            metadata: HashMap::new(),
        },
        VectorData {
            id: "vec-3".to_string(),
            vector: vec![0.0, 1.0, 0.0],
            metadata: HashMap::new(),
        },
    ];

    store.add_vectors(vectors).await.unwrap();

    // 使用阈值搜索
    let query = vec![1.0, 0.0, 0.0];
    let results = store.search_vectors(query, 10, Some(0.8)).await.unwrap();
    
    // 只有相似度 >= 0.8 的结果
    assert!(results.len() <= 2);
    for result in &results {
        assert!(result.similarity >= 0.8);
    }
}

#[tokio::test]
async fn test_memory_vector_search_with_filters() {
    let store = MemoryVectorStore::new(create_test_config()).await.unwrap();

    // 创建带元数据的向量
    let mut metadata1 = HashMap::new();
    metadata1.insert("category".to_string(), "A".to_string());
    metadata1.insert("priority".to_string(), "high".to_string());
    
    let mut metadata2 = HashMap::new();
    metadata2.insert("category".to_string(), "B".to_string());
    metadata2.insert("priority".to_string(), "low".to_string());
    
    let mut metadata3 = HashMap::new();
    metadata3.insert("category".to_string(), "A".to_string());
    metadata3.insert("priority".to_string(), "low".to_string());

    let vectors = vec![
        VectorData {
            id: "vec-1".to_string(),
            vector: vec![1.0, 0.0, 0.0],
            metadata: metadata1,
        },
        VectorData {
            id: "vec-2".to_string(),
            vector: vec![0.9, 0.1, 0.0],
            metadata: metadata2,
        },
        VectorData {
            id: "vec-3".to_string(),
            vector: vec![0.8, 0.2, 0.0],
            metadata: metadata3,
        },
    ];

    store.add_vectors(vectors).await.unwrap();

    // 测试带过滤器的搜索
    let mut filters = HashMap::new();
    filters.insert("category".to_string(), serde_json::json!("A"));

    let query = vec![1.0, 0.0, 0.0];
    let results = store
        .search_with_filters(query, 10, &filters, None)
        .await
        .unwrap();

    // 应该只返回 category=A 的向量
    assert_eq!(results.len(), 2);
    for result in &results {
        assert!(result.id == "vec-1" || result.id == "vec-3");
    }
}

#[tokio::test]
async fn test_memory_vector_update() {
    let store = MemoryVectorStore::new(create_test_config()).await.unwrap();

    let vectors = vec![VectorData {
        id: "vec-1".to_string(),
        vector: vec![1.0, 0.0, 0.0],
        metadata: HashMap::new(),
    }];

    store.add_vectors(vectors).await.unwrap();

    // 更新向量
    let mut new_metadata = HashMap::new();
    new_metadata.insert("updated".to_string(), "true".to_string());
    
    let updated_vectors = vec![VectorData {
        id: "vec-1".to_string(),
        vector: vec![0.0, 1.0, 0.0],
        metadata: new_metadata,
    }];

    store.update_vectors(updated_vectors).await.unwrap();

    // 验证更新
    let vec = store.get_vector("vec-1").await.unwrap().unwrap();
    assert_eq!(vec.vector, vec![0.0, 1.0, 0.0]);
    assert_eq!(vec.metadata.get("updated").unwrap(), "true");
}

#[tokio::test]
async fn test_memory_vector_delete() {
    let store = MemoryVectorStore::new(create_test_config()).await.unwrap();

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
    ];

    store.add_vectors(vectors).await.unwrap();

    // 删除一个向量
    store.delete_vectors(vec!["vec-1".to_string()]).await.unwrap();

    // 验证删除
    let count = store.count_vectors().await.unwrap();
    assert_eq!(count, 1);

    let vec = store.get_vector("vec-1").await.unwrap();
    assert!(vec.is_none());

    let vec = store.get_vector("vec-2").await.unwrap();
    assert!(vec.is_some());
}

#[tokio::test]
async fn test_memory_vector_clear() {
    let store = MemoryVectorStore::new(create_test_config()).await.unwrap();

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
    ];

    store.add_vectors(vectors).await.unwrap();

    // 清空
    store.clear().await.unwrap();

    // 验证清空
    let count = store.count_vectors().await.unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn test_memory_vector_batch_operations() {
    let store = MemoryVectorStore::new(create_test_config()).await.unwrap();

    // 批量添加
    let batch1 = vec![VectorData {
        id: "vec-1".to_string(),
        vector: vec![1.0, 0.0, 0.0],
        metadata: HashMap::new(),
    }];

    let batch2 = vec![VectorData {
        id: "vec-2".to_string(),
        vector: vec![0.0, 1.0, 0.0],
        metadata: HashMap::new(),
    }];

    let results = store
        .add_vectors_batch(vec![batch1, batch2])
        .await
        .unwrap();

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].len(), 1);
    assert_eq!(results[1].len(), 1);

    // 批量删除
    let delete_results = store
        .delete_vectors_batch(vec![
            vec!["vec-1".to_string()],
            vec!["vec-2".to_string()],
        ])
        .await
        .unwrap();

    assert_eq!(delete_results.len(), 2);
    assert!(delete_results[0]);
    assert!(delete_results[1]);

    let count = store.count_vectors().await.unwrap();
    assert_eq!(count, 0);
}

