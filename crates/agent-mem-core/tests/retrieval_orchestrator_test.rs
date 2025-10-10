use agent_mem_core::retrieval::{
    ActiveRetrievalConfig, ActiveRetrievalSystem, RetrievalRequest, RetrievalStrategy,
};
use agent_mem_core::types::MemoryType;

#[tokio::test]
async fn test_retrieval_orchestrator_basic() {
    // 创建配置
    let config = ActiveRetrievalConfig::default();

    // 创建检索系统
    let system = ActiveRetrievalSystem::new(config)
        .await
        .expect("Failed to create retrieval system");

    // 创建检索请求
    let request = RetrievalRequest {
        query: "test query".to_string(),
        target_memory_types: Some(vec![MemoryType::Core, MemoryType::Episodic]),
        max_results: 10,
        preferred_strategy: Some(RetrievalStrategy::StringMatch),
        context: None,
        enable_topic_extraction: false,
        enable_context_synthesis: false,
    };

    // 执行检索
    let response = system
        .retrieve(request)
        .await
        .expect("Failed to retrieve memories");

    // 验证结果
    assert!(response.memories.len() > 0, "Should return some memories");
    assert!(
        response.memories.len() <= 10,
        "Should not exceed max_results"
    );
    assert!(response.processing_time_ms > 0, "Should have processing time");
    assert!(
        response.confidence_score >= 0.0 && response.confidence_score <= 1.0,
        "Confidence score should be between 0 and 1"
    );

    println!("✅ Basic retrieval test passed");
    println!("   Retrieved {} memories", response.memories.len());
    println!("   Processing time: {}ms", response.processing_time_ms);
    println!("   Confidence score: {:.2}", response.confidence_score);
}

#[tokio::test]
async fn test_retrieval_orchestrator_multiple_memory_types() {
    let config = ActiveRetrievalConfig::default();
    let system = ActiveRetrievalSystem::new(config)
        .await
        .expect("Failed to create retrieval system");

    let request = RetrievalRequest {
        query: "multi-type query".to_string(),
        target_memory_types: Some(vec![
            MemoryType::Core,
            MemoryType::Episodic,
            MemoryType::Semantic,
        ]),
        max_results: 15,
        preferred_strategy: Some(RetrievalStrategy::Hybrid),
        context: None,
        enable_topic_extraction: false,
        enable_context_synthesis: false,
    };

    let response = system
        .retrieve(request)
        .await
        .expect("Failed to retrieve memories");

    // 验证结果包含多种记忆类型
    let memory_types: std::collections::HashSet<_> = response
        .memories
        .iter()
        .map(|m| m.memory_type.clone())
        .collect();

    assert!(
        memory_types.len() > 1,
        "Should retrieve from multiple memory types"
    );
    assert!(
        response.memories.len() <= 15,
        "Should not exceed max_results"
    );

    println!("✅ Multiple memory types test passed");
    println!("   Retrieved {} memories", response.memories.len());
    println!("   Memory types: {:?}", memory_types);
}

#[tokio::test]
async fn test_retrieval_orchestrator_relevance_scoring() {
    let config = ActiveRetrievalConfig::default();
    let system = ActiveRetrievalSystem::new(config)
        .await
        .expect("Failed to create retrieval system");

    let request = RetrievalRequest {
        query: "relevance test".to_string(),
        target_memory_types: Some(vec![MemoryType::Semantic]),
        max_results: 5,
        preferred_strategy: Some(RetrievalStrategy::Embedding),
        context: None,
        enable_topic_extraction: false,
        enable_context_synthesis: false,
    };

    let response = system
        .retrieve(request)
        .await
        .expect("Failed to retrieve memories");

    // 验证结果按相关性降序排序
    for i in 1..response.memories.len() {
        assert!(
            response.memories[i - 1].relevance_score >= response.memories[i].relevance_score,
            "Results should be sorted by relevance score in descending order"
        );
    }

    // 验证所有结果都有相关性分数
    for memory in &response.memories {
        assert!(
            memory.relevance_score > 0.0 && memory.relevance_score <= 1.0,
            "Relevance score should be between 0 and 1"
        );
    }

    println!("✅ Relevance scoring test passed");
    println!("   Retrieved {} memories", response.memories.len());
    if !response.memories.is_empty() {
        println!(
            "   Top score: {:.3}",
            response.memories[0].relevance_score
        );
        println!(
            "   Lowest score: {:.3}",
            response.memories.last().unwrap().relevance_score
        );
    }
}

#[tokio::test]
async fn test_retrieval_orchestrator_max_results() {
    let config = ActiveRetrievalConfig::default();
    let system = ActiveRetrievalSystem::new(config)
        .await
        .expect("Failed to create retrieval system");

    // 测试不同的 max_results 值
    for max_results in [1, 3, 5, 10] {
        let request = RetrievalRequest {
            query: format!("test query {}", max_results),
            target_memory_types: Some(vec![
                MemoryType::Core,
                MemoryType::Episodic,
                MemoryType::Semantic,
            ]),
            max_results,
            preferred_strategy: Some(RetrievalStrategy::StringMatch),
            context: None,
            enable_topic_extraction: false,
            enable_context_synthesis: false,
        };

        let response = system
            .retrieve(request)
            .await
            .expect("Failed to retrieve memories");

        assert!(
            response.memories.len() <= max_results,
            "Should not exceed max_results of {}",
            max_results
        );

        println!(
            "   max_results={}: retrieved {} memories",
            max_results,
            response.memories.len()
        );
    }

    println!("✅ Max results test passed");
}

#[tokio::test]
async fn test_retrieval_orchestrator_caching() {
    let mut config = ActiveRetrievalConfig::default();
    config.enable_caching = true;
    config.cache_ttl_seconds = 60;

    let system = ActiveRetrievalSystem::new(config)
        .await
        .expect("Failed to create retrieval system");

    let request = RetrievalRequest {
        query: "cached query".to_string(),
        target_memory_types: Some(vec![MemoryType::Core]),
        max_results: 5,
        preferred_strategy: Some(RetrievalStrategy::StringMatch),
        context: None,
        enable_topic_extraction: false,
        enable_context_synthesis: false,
    };

    // 第一次检索
    let response1 = system
        .retrieve(request.clone())
        .await
        .expect("Failed to retrieve memories");

    // 第二次检索（应该从缓存返回）
    let response2 = system
        .retrieve(request.clone())
        .await
        .expect("Failed to retrieve memories");

    // 验证结果一致
    assert_eq!(
        response1.memories.len(),
        response2.memories.len(),
        "Cached results should be identical"
    );

    // 第二次检索应该更快（从缓存）
    // 注意：由于缓存，第二次的 processing_time_ms 可能为 0 或很小
    println!("✅ Caching test passed");
    println!("   First retrieval: {}ms", response1.processing_time_ms);
    println!("   Second retrieval: {}ms", response2.processing_time_ms);
}

#[tokio::test]
async fn test_retrieval_orchestrator_metadata() {
    let config = ActiveRetrievalConfig::default();
    let system = ActiveRetrievalSystem::new(config)
        .await
        .expect("Failed to create retrieval system");

    let request = RetrievalRequest {
        query: "metadata test".to_string(),
        target_memory_types: Some(vec![MemoryType::Procedural]),
        max_results: 3,
        preferred_strategy: Some(RetrievalStrategy::SemanticGraph),
        context: None,
        enable_topic_extraction: false,
        enable_context_synthesis: false,
    };

    let response = system
        .retrieve(request)
        .await
        .expect("Failed to retrieve memories");

    // 验证每个结果都有元数据
    for memory in &response.memories {
        assert!(!memory.metadata.is_empty(), "Should have metadata");
        assert!(
            memory.metadata.contains_key("mock"),
            "Should have 'mock' metadata"
        );
        assert!(
            memory.metadata.contains_key("query"),
            "Should have 'query' metadata"
        );
        assert!(
            memory.metadata.contains_key("memory_type"),
            "Should have 'memory_type' metadata"
        );
        assert!(
            memory.metadata.contains_key("strategy"),
            "Should have 'strategy' metadata"
        );
    }

    println!("✅ Metadata test passed");
    println!("   All {} memories have complete metadata", response.memories.len());
}

