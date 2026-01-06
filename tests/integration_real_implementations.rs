//! 真实实现集成测试
//! 
//! 这个测试套件验证所有 Mock 实现已经被真实实现替换，
//! 并且系统能够正常工作。

use agent_mem_core::client::AgentMemClient;
use agent_mem_compat::client::{Mem0Client, Messages};
use agent_mem_compat::types::{AddMemoryRequest, SearchMemoryRequest, MemoryFilter};
use agent_mem_traits::{Message, VectorData};
use agent_mem_llm::factory::LLMFactory;
use agent_mem_llm::config::LLMConfig;
use agent_mem_embeddings::factory::EmbeddingFactory;
use agent_mem_embeddings::config::EmbeddingConfig;
use agent_mem_storage::factory::StorageFactory;
use agent_mem_storage::config::VectorStoreConfig;
use std::collections::HashMap;
use tokio;

/// 测试 LLM 提供商真实实现
#[tokio::test]
async fn test_real_llm_providers() {
    // 测试 DeepSeek 提供商（使用真实 API key）
    let deepseek_config = LLMConfig {
        provider: "deepseek".to_string(),
        model: "deepseek-chat".to_string(),
        api_key: Some("sk-498fd5f3041f4466a43fa2b9bbbec250".to_string()),
        temperature: Some(0.7),
        max_tokens: Some(100),
        ..Default::default()
    };

    let llm_provider = LLMFactory::create_provider(&deepseek_config).await;
    assert!(llm_provider.is_ok(), "DeepSeek provider should be created successfully");

    // 测试真实的文本生成（如果环境变量允许）
    if std::env::var("ENABLE_REAL_API_TESTS").is_ok() {
        let provider = llm_provider.unwrap();
        let messages = vec![Message {
            role: "user".to_string(),
            content: "Hello, this is a test message.".to_string(),
        }];
        
        let response = provider.generate(&messages).await;
        assert!(response.is_ok(), "Real LLM generation should work");
        
        let text = response.unwrap();
        assert!(!text.is_empty(), "Generated text should not be empty");
        assert!(!text.contains("Mock"), "Response should not contain 'Mock'");
    }
}

/// 测试嵌入提供商真实实现
#[tokio::test]
async fn test_real_embedding_providers() {
    // 测试本地嵌入提供商
    let local_config = EmbeddingConfig {
        provider: "local".to_string(),
        model: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
        dimension: 384,
        ..Default::default()
    };

    let embedder = EmbeddingFactory::create_embedder(&local_config).await;
    assert!(embedder.is_ok(), "Local embedder should be created successfully");

    let embedder = embedder.unwrap();
    let embedding = embedder.embed("This is a test sentence for embedding.").await;
    assert!(embedding.is_ok(), "Embedding generation should work");
    
    let vector = embedding.unwrap();
    assert_eq!(vector.len(), 384, "Embedding dimension should be 384");
    
    // 验证不是 Mock 实现（Mock 通常返回全零或固定值）
    let non_zero_count = vector.iter().filter(|&&x| x != 0.0).count();
    assert!(non_zero_count > 0, "Real embedding should have non-zero values");
}

/// 测试存储后端真实实现
#[tokio::test]
async fn test_real_storage_backends() {
    // 测试内存存储（这是真实实现，不是 Mock）
    let memory_config = VectorStoreConfig {
        provider: "memory".to_string(),
        ..Default::default()
    };

    let store = StorageFactory::create_store(&memory_config).await;
    assert!(store.is_ok(), "Memory store should be created successfully");

    let store = store.unwrap();
    
    // 测试真实的向量操作
    let test_vector = VectorData {
        id: "test_real_vector".to_string(),
        vector: vec![0.1, 0.2, 0.3, 0.4, 0.5],
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("test_type".to_string(), serde_json::Value::String("real_implementation".to_string()));
            meta
        },
    };

    let add_result = store.add_vector(&test_vector).await;
    assert!(add_result.is_ok(), "Vector addition should work");

    let search_result = store.search_vectors(&test_vector.vector, 5, None).await;
    assert!(search_result.is_ok(), "Vector search should work");
    
    let results = search_result.unwrap();
    assert!(!results.is_empty(), "Search should return results");
    assert_eq!(results[0].id, "test_real_vector", "Should find the added vector");
}

/// 测试 Mem0 兼容性真实实现
#[tokio::test]
async fn test_real_mem0_compatibility() {
    let client = Mem0Client::new().await;
    assert!(client.is_ok(), "Mem0 client should be created successfully");

    let client = client.unwrap();
    
    // 测试真实的记忆添加
    let add_request = AddMemoryRequest {
        user_id: "test_user_real".to_string(),
        memory: "I love programming in Rust because it's safe and fast.".to_string(),
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("category".to_string(), serde_json::Value::String("preference".to_string()));
            meta.insert("test_type".to_string(), serde_json::Value::String("real_implementation".to_string()));
            meta
        },
    };

    let memory_id = client.add_with_options(add_request).await;
    assert!(memory_id.is_ok(), "Memory addition should work");
    
    let memory_id = memory_id.unwrap();
    assert!(!memory_id.is_empty(), "Memory ID should not be empty");
    assert!(!memory_id.contains("mock"), "Memory ID should not contain 'mock'");

    // 测试真实的记忆搜索
    let search_request = SearchMemoryRequest {
        query: "programming language".to_string(),
        user_id: "test_user_real".to_string(),
        filters: Some(MemoryFilter {
            category: Some("preference".to_string()),
            limit: Some(10),
            ..Default::default()
        }),
        limit: Some(10),
    };

    let search_result = client.search_with_options(search_request).await;
    assert!(search_result.is_ok(), "Memory search should work");
    
    let results = search_result.unwrap();
    assert!(!results.memories.is_empty(), "Search should return memories");
    
    // 验证返回的记忆不是 Mock 数据
    let memory = &results.memories[0];
    assert!(!memory.content.contains("Mock"), "Memory content should not contain 'Mock'");
    assert!(!memory.content.contains("mock"), "Memory content should not contain 'mock'");
}

/// 测试批量操作真实实现
#[tokio::test]
async fn test_real_batch_operations() {
    let client = Mem0Client::new().await;
    assert!(client.is_ok(), "Mem0 client should be created successfully");

    let client = client.unwrap();
    
    // 测试真实的批量添加
    let batch_memories = vec![
        "I enjoy reading science fiction books.".to_string(),
        "My favorite programming language is Rust.".to_string(),
        "I prefer working in quiet environments.".to_string(),
    ];

    let mut batch_requests = Vec::new();
    for (i, memory) in batch_memories.iter().enumerate() {
        batch_requests.push(AddMemoryRequest {
            user_id: "batch_test_user".to_string(),
            memory: memory.clone(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("batch_index".to_string(), serde_json::Value::Number(i.into()));
                meta.insert("test_type".to_string(), serde_json::Value::String("batch_real".to_string()));
                meta
            },
        });
    }

    // 执行批量添加
    let mut successful_adds = 0;
    for request in batch_requests {
        let result = client.add_with_options(request).await;
        if result.is_ok() {
            successful_adds += 1;
        }
    }

    assert_eq!(successful_adds, 3, "All batch additions should succeed");

    // 测试批量搜索
    let search_result = client.get_all("batch_test_user", None).await;
    assert!(search_result.is_ok(), "Batch retrieval should work");
    
    let memories = search_result.unwrap();
    assert!(memories.len() >= 3, "Should retrieve at least 3 memories");
    
    // 验证不是 Mock 数据
    for memory in &memories {
        assert!(!memory.content.contains("Mock"), "Memory content should not be Mock");
        assert!(!memory.content.contains("mock"), "Memory content should not be mock");
    }
}

/// 测试性能监控真实实现
#[tokio::test]
async fn test_real_performance_monitoring() {
    use agent_mem_performance::monitor::PerformanceMonitor;
    use agent_mem_performance::config::PerformanceConfig;

    let config = PerformanceConfig::default();
    let monitor = PerformanceMonitor::new(config);

    // 测试真实的性能指标收集
    let metrics = monitor.collect_metrics().await;
    assert!(metrics.is_ok(), "Performance metrics collection should work");
    
    let metrics = metrics.unwrap();
    
    // 验证指标不是 Mock 数据
    assert!(metrics.memory_usage_mb > 0.0, "Memory usage should be positive");
    assert!(metrics.cpu_usage_percent >= 0.0, "CPU usage should be non-negative");
    assert!(metrics.active_connections >= 0, "Active connections should be non-negative");
    
    // 验证指标值在合理范围内（不是固定的 Mock 值）
    assert!(metrics.memory_usage_mb < 10000.0, "Memory usage should be reasonable");
    assert!(metrics.cpu_usage_percent <= 100.0, "CPU usage should not exceed 100%");
}
