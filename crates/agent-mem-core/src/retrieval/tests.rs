//! 主动检索系统测试

use super::*;
use crate::types::MemoryType;
use std::collections::HashMap;
use tokio;

/// 创建测试用的检索请求
fn create_test_retrieval_request() -> RetrievalRequest {
    RetrievalRequest {
        query: "测试查询内容".to_string(),
        target_memory_types: Some(vec![MemoryType::Semantic, MemoryType::Episodic]),
        max_results: 10,
        preferred_strategy: Some(RetrievalStrategy::Embedding),
        context: Some({
            let mut ctx = HashMap::new();
            ctx.insert(
                "user_id".to_string(),
                serde_json::Value::String("test_user".to_string()),
            );
            ctx
        }),
        enable_topic_extraction: true,
        enable_context_synthesis: true,
    }
}

/// 创建测试用的检索记忆项
fn create_test_retrieved_memories() -> Vec<RetrievedMemory> {
    vec![
        RetrievedMemory {
            id: "memory_1".to_string(),
            memory_type: MemoryType::Semantic,
            content: "这是一个语义记忆内容".to_string(),
            relevance_score: 0.9,
            source_agent: "semantic_agent".to_string(),
            retrieval_strategy: RetrievalStrategy::Embedding,
            metadata: HashMap::new(),
        },
        RetrievedMemory {
            id: "memory_2".to_string(),
            memory_type: MemoryType::Episodic,
            content: "这是一个情节记忆内容".to_string(),
            relevance_score: 0.8,
            source_agent: "episodic_agent".to_string(),
            retrieval_strategy: RetrievalStrategy::BM25,
            metadata: HashMap::new(),
        },
    ]
}

#[tokio::test]
async fn test_topic_extractor_creation() {
    let config = TopicExtractorConfig::default();
    let extractor = TopicExtractor::new(config).await;
    assert!(extractor.is_ok());
}

#[tokio::test]
async fn test_topic_extraction() {
    let config = TopicExtractorConfig::default();
    let extractor = TopicExtractor::new(config).await?;

    let text = "这是一个关于人工智能和机器学习的技术讨论";
    let topics = extractor.extract_topics(text, None).await?;

    assert!(!topics.is_empty());
    assert!(topics
        .iter()
        .any(|t| matches!(t.category, TopicCategory::Technical)));
}

#[tokio::test]
async fn test_topic_extraction_with_context() {
    let config = TopicExtractorConfig::default();
    let extractor = TopicExtractor::new(config).await?;

    let text = "讨论项目进展和业务需求";
    let mut context = HashMap::new();
    context.insert(
        "domain".to_string(),
        serde_json::Value::String("business".to_string()),
    );

    let topics = extractor
        .extract_topics(text, Some(&context))
        .await
        .unwrap();

    assert!(!topics.is_empty());
}

#[tokio::test]
async fn test_topic_extractor_stats() {
    let config = TopicExtractorConfig::default();
    let extractor = TopicExtractor::new(config).await?;

    // 执行一些提取操作
    let _ = extractor.extract_topics("测试文本", None).await?;

    let stats = extractor.get_stats().await?;
    assert!(stats.is_object());
}

#[tokio::test]
async fn test_retrieval_router_creation() {
    let config = RetrievalRouterConfig::default();
    let router = RetrievalRouter::new(config).await;
    assert!(router.is_ok());
}

#[tokio::test]
async fn test_retrieval_routing() {
    let config = RetrievalRouterConfig::default();
    let router = RetrievalRouter::new(config).await?;

    let request = create_test_retrieval_request();
    let topics = vec![ExtractedTopic {
        name: "技术讨论".to_string(),
        category: TopicCategory::Technical,
        confidence: 0.8,
        keywords: vec!["技术".to_string(), "讨论".to_string()],
        description: Some("技术相关主题".to_string()),
        hierarchy_level: 0,
        parent_topic_id: None,
        relevance_score: 0.9,
    }];

    let result = router.route_retrieval(&request, &topics).await?;

    assert!(result.success);
    assert!(!result.decision.selected_strategies.is_empty());
    assert!(!result.decision.target_memory_types.is_empty());
    assert!(result.decision.confidence > 0.0);
}

#[tokio::test]
async fn test_router_strategy_selection() {
    let config = RetrievalRouterConfig::default();
    let router = RetrievalRouter::new(config).await?;

    let request = RetrievalRequest {
        query: "查找相关文档".to_string(),
        target_memory_types: None,
        max_results: 5,
        preferred_strategy: Some(RetrievalStrategy::BM25),
        context: None,
        enable_topic_extraction: false,
        enable_context_synthesis: false,
    };

    let result = router.route_retrieval(&request, &[]).await?;

    assert!(result.success);
    assert!(!result.decision.selected_strategies.is_empty());
}

#[tokio::test]
async fn test_router_stats() {
    let config = RetrievalRouterConfig::default();
    let router = RetrievalRouter::new(config).await?;

    let request = create_test_retrieval_request();
    let _ = router.route_retrieval(&request, &[]).await?;

    let stats = router.get_stats().await?;
    assert!(stats.is_object());
}

#[tokio::test]
async fn test_context_synthesizer_creation() {
    let config = ContextSynthesizerConfig::default();
    let synthesizer = ContextSynthesizer::new(config).await;
    assert!(synthesizer.is_ok());
}

#[tokio::test]
async fn test_context_synthesis() {
    let config = ContextSynthesizerConfig::default();
    let synthesizer = ContextSynthesizer::new(config).await?;

    let memories = create_test_retrieved_memories();
    let request = create_test_retrieval_request();

    let result = synthesizer
        .synthesize_context(&memories, &request)
        .await
        .unwrap();

    assert!(!result.synthesized_memories.is_empty());
    assert!(!result.synthesis_summary.is_empty());
    assert!(result.confidence_score >= 0.0 && result.confidence_score <= 1.0);
    assert!(!result.relevance_ranking.is_empty());
}

#[tokio::test]
async fn test_conflict_detection() {
    let config = ContextSynthesizerConfig {
        enable_conflict_detection: true,
        conflict_detection_threshold: 0.5,
        ..Default::default()
    };
    let synthesizer = ContextSynthesizer::new(config).await?;

    // 创建相似的记忆项来触发冲突检测
    let memories = vec![
        RetrievedMemory {
            id: "memory_1".to_string(),
            memory_type: MemoryType::Semantic,
            content: "相同的内容信息".to_string(),
            relevance_score: 0.9,
            source_agent: "agent_1".to_string(),
            retrieval_strategy: RetrievalStrategy::Embedding,
            metadata: HashMap::new(),
        },
        RetrievedMemory {
            id: "memory_2".to_string(),
            memory_type: MemoryType::Semantic,
            content: "相同的内容信息".to_string(),
            relevance_score: 0.8,
            source_agent: "agent_2".to_string(),
            retrieval_strategy: RetrievalStrategy::BM25,
            metadata: HashMap::new(),
        },
    ];

    let request = create_test_retrieval_request();
    let result = synthesizer
        .synthesize_context(&memories, &request)
        .await
        .unwrap();

    // 应该检测到冲突
    assert!(!result.detected_conflicts.is_empty());
}

#[tokio::test]
async fn test_synthesizer_stats() {
    let config = ContextSynthesizerConfig::default();
    let synthesizer = ContextSynthesizer::new(config).await?;

    let memories = create_test_retrieved_memories();
    let request = create_test_retrieval_request();
    let _ = synthesizer
        .synthesize_context(&memories, &request)
        .await
        .unwrap();

    let stats = synthesizer.get_stats().await?;
    assert!(stats.is_object());
}

#[tokio::test]
async fn test_active_retrieval_system_creation() {
    let config = ActiveRetrievalConfig::default();
    let system = ActiveRetrievalSystem::new(config).await;
    assert!(system.is_ok());
}

#[tokio::test]
async fn test_active_retrieval_system_retrieve() {
    let config = ActiveRetrievalConfig::default();
    let system = ActiveRetrievalSystem::new(config).await?;

    let request = create_test_retrieval_request();
    let result = system.retrieve(request).await?;

    assert!(result.processing_time_ms > 0);
    assert!(result.confidence_score >= 0.0 && result.confidence_score <= 1.0);
}

#[tokio::test]
async fn test_retrieval_system_caching() {
    let config = ActiveRetrievalConfig {
        enable_caching: true,
        cache_ttl_seconds: 60,
        ..Default::default()
    };
    let system = ActiveRetrievalSystem::new(config).await?;

    let request = create_test_retrieval_request();

    // 第一次检索
    let result1 = system.retrieve(request.clone()).await?;
    let time1 = result1.processing_time_ms;

    // 第二次检索（应该使用缓存）
    let result2 = system.retrieve(request).await?;
    let time2 = result2.processing_time_ms;

    // 缓存的检索应该更快
    assert!(time2 <= time1);
}

#[tokio::test]
async fn test_retrieval_system_stats() {
    let config = ActiveRetrievalConfig::default();
    let system = ActiveRetrievalSystem::new(config).await?;

    let request = create_test_retrieval_request();
    let _ = system.retrieve(request).await?;

    let stats = system.get_stats().await?;
    assert!(stats.cache_size >= 0);
}

#[tokio::test]
async fn test_cache_cleanup() {
    let config = ActiveRetrievalConfig {
        enable_caching: true,
        cache_ttl_seconds: 1, // 1秒过期
        ..Default::default()
    };
    let system = ActiveRetrievalSystem::new(config).await?;

    let request = create_test_retrieval_request();
    let _ = system.retrieve(request).await?;

    // 等待缓存过期
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // 清理缓存
    system.cleanup_cache().await?;

    let stats = system.get_stats().await?;
    assert_eq!(stats.cache_size, 0);
}

#[tokio::test]
async fn test_topic_category_description() {
    assert!(!TopicCategory::Technical.description().is_empty());
    assert!(!TopicCategory::Business.description().is_empty());
    assert!(!TopicCategory::Personal.description().is_empty());
}

#[tokio::test]
async fn test_retrieval_strategy_description() {
    assert!(!RetrievalStrategy::Embedding.description().is_empty());
    assert!(!RetrievalStrategy::BM25.description().is_empty());
    assert!(!RetrievalStrategy::Hybrid.description().is_empty());
}

#[tokio::test]
async fn test_conflict_resolution_description() {
    assert!(!ConflictResolution::KeepLatest.description().is_empty());
    assert!(!ConflictResolution::KeepMostRelevant
        .description()
        .is_empty());
    assert!(!ConflictResolution::Merge.description().is_empty());
}
