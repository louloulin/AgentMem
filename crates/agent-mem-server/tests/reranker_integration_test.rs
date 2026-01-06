//! Reranker集成测试
//!
//! 验证QueryOptimizer和ResultReranker在实际搜索流程中的集成效果

#[tokio::test]
async fn test_reranker_initialization() -> Result<(), Box<dyn std::error::Error>> {
    use agent_mem_server::routes::memory::MemoryManager;

    // 测试MemoryManager能否正确初始化（包含QueryOptimizer和Reranker）
    let manager = MemoryManager::new(None, None).await;

    assert!(manager.is_ok(), "MemoryManager应该成功初始化");

    println!("✅ Reranker初始化测试通过");

    Ok(())
}

#[tokio::test]
async fn test_search_with_optimizer_and_reranker() -> Result<(), Box<dyn std::error::Error>> {
    use agent_mem_server::routes::memory::MemoryManager;

    let manager = MemoryManager::new(None, None).await?;

    // 执行搜索（即使没有数据，也应该正常执行，不崩溃）
    let results = manager
        .search_memories(
            "test query".to_string(),
            Some("test_agent".to_string()),
            Some("test_user".to_string()),
            Some(5),
            None,
        )
        .await;

    // 应该返回成功（即使是空结果）
    assert!(results.is_ok(), "搜索应该成功执行");

    let results = results.unwrap();
    println!(
        "✅ QueryOptimizer和Reranker集成测试通过，返回{}个结果",
        results.len()
    );

    Ok(())
}

#[tokio::test]
async fn test_different_query_types() -> Result<(), Box<dyn std::error::Error>> {
    use agent_mem_server::routes::memory::MemoryManager;

    let manager = MemoryManager::new(None, None).await?;

    // 测试不同类型的查询都能正常处理
    let test_queries = vec![
        "simple query",
        "what is machine learning?",
        "test@example.com",
        "很长的查询字符串 with multiple words and 中文",
        "短",
    ];

    for query in test_queries {
        let results = manager
            .search_memories(
                query.to_string(),
                Some("test_agent".to_string()),
                Some("test_user".to_string()),
                Some(10),
                None,
            )
            .await;

        assert!(results.is_ok(), "查询 '{}' 应该成功", query);
    }

    println!("✅ 不同查询类型测试通过");

    Ok(())
}

#[tokio::test]
async fn test_different_limit_values() -> Result<(), Box<dyn std::error::Error>> {
    use agent_mem_server::routes::memory::MemoryManager;

    let manager = MemoryManager::new(None, None).await?;

    // 测试不同的limit值
    let limits = vec![1, 5, 10, 20, 50];

    for limit in limits {
        let results = manager
            .search_memories(
                "test query".to_string(),
                Some("test_agent".to_string()),
                Some("test_user".to_string()),
                Some(limit),
                None,
            )
            .await?;

        // 结果数量不应超过limit
        assert!(
            results.len() <= limit,
            "结果数量 {} 不应超过 limit {}",
            results.len(),
            limit
        );
    }

    println!("✅ 不同limit值测试通过");

    Ok(())
}

#[tokio::test]
async fn test_optimizer_components_exist() -> Result<(), Box<dyn std::error::Error>> {
    use agent_mem_core::search::IndexStatistics;
    use agent_mem_core::search::{QueryOptimizer, ResultReranker};
    use std::sync::{Arc, RwLock};

    // 验证QueryOptimizer可以创建
    let stats = Arc::new(RwLock::new(IndexStatistics::new(1000, 1536)));
    let optimizer = QueryOptimizer::with_default_config(stats);

    // 验证可以优化查询
    let query = agent_mem_core::search::SearchQuery {
        query: "test".to_string(),
        limit: 10,
        threshold: Some(0.7),
        vector_weight: 0.7,
        fulltext_weight: 0.3,
        filters: None,
        metadata_filters: None,
    };

    let plan = optimizer.optimize_query(&query);
    assert!(plan.is_ok(), "查询优化应该成功");

    // 验证ResultReranker可以创建
    let reranker = ResultReranker::with_default_config();

    println!("✅ QueryOptimizer和ResultReranker组件创建测试通过");
    println!("   优化计划: {:?}", plan.unwrap());

    Ok(())
}
