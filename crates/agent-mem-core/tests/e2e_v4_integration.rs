//! E2E Test - V4 Integration
//! Week 10: 多模块集成测试

use agent_mem_core::config::AgentMemConfig;
use agent_mem_core::performance::cache::QueryCacheConfig;
use agent_mem_core::search::adaptive_router::AdaptiveRouter;
use agent_mem_core::types::{
    AttributeKey, AttributeValue, Content, Memory, MemoryBuilder, Pipeline, PipelineContext,
    PipelineStage, QueryBuilder, QueryIntent, StageResult,
};
use anyhow::Result;

/// 集成Stage：完整的记忆处理流程
struct FullMemoryProcessingStage;

#[async_trait::async_trait]
impl PipelineStage for FullMemoryProcessingStage {
    type Input = Memory;
    type Output = Memory;

    fn name(&self) -> &str {
        "full_processing"
    }

    async fn execute(
        &self,
        input: Self::Input,
        context: &mut PipelineContext,
    ) -> Result<StageResult<Self::Output>> {
        let mut output = input;

        // 1. 内容清理
        if let Content::Text(ref text) = output.content {
            let cleaned = text.trim().to_string();
            output.content = Content::Text(cleaned);
        }

        // 2. 重要性评估
        let importance = if let Content::Text(ref text) = output.content {
            (text.len() as f64 / 100.0).min(1.0)
        } else {
            0.5
        };
        output.attributes.set(
            AttributeKey::system("importance"),
            AttributeValue::Number(importance),
        );

        // 3. 自动分类
        if let Content::Text(ref text) = output.content {
            if text.contains("食物") || text.contains("火锅") || text.contains("菜") {
                output.attributes.set(
                    AttributeKey::domain("category"),
                    AttributeValue::String("food".to_string()),
                );
            }
        }

        context.set("processed", true);
        Ok(StageResult::Continue(output))
    }
}

/// E2E集成测试：Memory + Pipeline + Query 完整流程
#[tokio::test]
async fn test_end_to_end_memory_pipeline_query() {
    // 1. 创建记忆
    let memory = MemoryBuilder::new()
        .content(Content::Text(
            "  用户喜欢吃四川火锅，辣度偏好中辣  ".to_string(),
        ))
        .attribute(
            AttributeKey::system("user_id"),
            AttributeValue::String("user_001".to_string()),
        )
        .build();

    println!("✅ Step 1: Memory created");

    // 2. Pipeline处理
    let pipeline = Pipeline::<Memory, Memory>::new("full_pipeline")
        .add_stage(FullMemoryProcessingStage);

    let mut context = PipelineContext::new();
    let processed_memory = pipeline.execute(memory, &mut context).await.unwrap();

    // 验证Pipeline处理结果
    assert_eq!(context.get::<bool>("processed"), Some(true));
    assert!(processed_memory
        .attributes
        .get(&AttributeKey::system("importance"))
        .is_some());
    assert!(matches!(
        processed_memory
            .attributes
            .get(&AttributeKey::domain("category")),
        Some(AttributeValue::String(s)) if s == "food"
    ));

    println!("✅ Step 2: Pipeline processed");

    // 3. 构建Query
    let query = QueryBuilder::new()
        .text("四川火锅")
        .build();

    assert!(matches!(query.intent, QueryIntent::SemanticSearch { .. }));

    println!("✅ Step 3: Query constructed");

    // 4. 完整流程验证
    println!("\n=== Full Integration Test Summary ===");
    println!("✅ Memory creation: Content + Attributes");
    println!("✅ Pipeline processing: Cleaning + Importance + Classification");
    println!("✅ Query construction: Intent-based query");
    println!("✅ All modules integrated successfully!");
}

/// E2E集成测试：Adaptive Router + Config + Cache
#[tokio::test]
async fn test_adaptive_router_config_cache_integration() {
    // 1. 加载配置
    let config = AgentMemConfig::default();
    let cache_config = QueryCacheConfig::default();

    println!("✅ Step 1: Config loaded");

    // 2. 创建自适应路由器
    let router = AdaptiveRouter::new(config);

    println!("✅ Step 2: Adaptive router created");

    // 3. 创建Query并决策
    let query = agent_mem_core::search::SearchQuery {
        query: "测试查询".to_string(),
        limit: 10,
        threshold: Some(0.7),
        vector_weight: 0.0,
        fulltext_weight: 0.0,
        filters: None,
        metadata_filters: None,
    };

    let (strategy, weights) = router.decide_strategy(&query).await.unwrap();

    println!("✅ Step 3: Strategy selected: {:?}", strategy);
    println!("   - Vector weight: {:.2}", weights.vector_weight);
    println!("   - Fulltext weight: {:.2}", weights.fulltext_weight);

    // 验证权重和为1
    assert!((weights.vector_weight + weights.fulltext_weight - 1.0).abs() < 0.01);

    // 4. 模拟反馈
    router
        .record_performance(&query, strategy, 0.85, 100)
        .await
        .unwrap();

    println!("✅ Step 4: Performance feedback recorded");

    // 5. 验证缓存配置
    assert_eq!(cache_config.max_entries, 10000);
    assert_eq!(cache_config.default_ttl_seconds, 300);

    println!("✅ Step 5: Cache config validated");

    println!("\n=== Adaptive + Config + Cache Integration Test Summary ===");
    println!("✅ Configuration system working");
    println!("✅ Adaptive router functional");
    println!("✅ Performance tracking active");
    println!("✅ Cache configuration valid");
}

/// E2E集成测试：多记忆类型处理
#[tokio::test]
async fn test_multimodal_memory_types_integration() {
    let mut memories = Vec::new();

    // 文本记忆
    let text_mem = MemoryBuilder::new()
        .content(Content::Text("这是文本记忆".to_string()))
        .attribute(
            AttributeKey::domain("type"),
            AttributeValue::String("text".to_string()),
        )
        .build();
    memories.push(text_mem);

    // 图片记忆
    let image_mem = MemoryBuilder::new()
        .content(Content::Image {
            url: "https://example.com/img.jpg".to_string(),
            caption: Some("图片说明".to_string()),
        })
        .attribute(
            AttributeKey::domain("type"),
            AttributeValue::String("image".to_string()),
        )
        .build();
    memories.push(image_mem);

    // 音频记忆
    let audio_mem = MemoryBuilder::new()
        .content(Content::Audio {
            url: "https://example.com/audio.mp3".to_string(),
            transcript: Some("音频转录".to_string()),
        })
        .attribute(
            AttributeKey::domain("type"),
            AttributeValue::String("audio".to_string()),
        )
        .build();
    memories.push(audio_mem);

    // 视频记忆
    let video_mem = MemoryBuilder::new()
        .content(Content::Video {
            url: "https://example.com/video.mp4".to_string(),
            summary: Some("视频摘要".to_string()),
        })
        .attribute(
            AttributeKey::domain("type"),
            AttributeValue::String("video".to_string()),
        )
        .build();
    memories.push(video_mem);

    // 结构化数据
    let structured_mem = MemoryBuilder::new()
        .content(Content::Structured(serde_json::json!({
            "name": "测试数据",
            "value": 123,
            "tags": ["test", "structured"]
        })))
        .attribute(
            AttributeKey::domain("type"),
            AttributeValue::String("structured".to_string()),
        )
        .build();
    memories.push(structured_mem);

    // 混合内容
    let mixed_mem = MemoryBuilder::new()
        .content(Content::Mixed(vec![
            Content::Text("混合内容示例".to_string()),
            Content::Image {
                url: "https://example.com/mixed.jpg".to_string(),
                caption: Some("配图".to_string()),
            },
        ]))
        .attribute(
            AttributeKey::domain("type"),
            AttributeValue::String("mixed".to_string()),
        )
        .build();
    memories.push(mixed_mem);

    // 验证所有类型
    assert_eq!(memories.len(), 6);
    assert!(matches!(memories[0].content, Content::Text(_)));
    assert!(matches!(memories[1].content, Content::Image { .. }));
    assert!(matches!(memories[2].content, Content::Audio { .. }));
    assert!(matches!(memories[3].content, Content::Video { .. }));
    assert!(matches!(memories[4].content, Content::Structured(_)));
    assert!(matches!(memories[5].content, Content::Mixed(_)));

    println!("✅ Multimodal memory types integration test passed");
    println!("   - Text: ✅");
    println!("   - Image: ✅");
    println!("   - Audio: ✅");
    println!("   - Video: ✅");
    println!("   - Structured: ✅");
    println!("   - Mixed: ✅");
}

/// E2E集成测试：层次Scope + 访问控制
#[tokio::test]
async fn test_hierarchical_scope_access_control() {
    // 创建不同层次的记忆
    let mut global_mem = MemoryBuilder::new()
        .content(Content::Text("全局知识".to_string()))
        .build();
    global_mem.attributes.set_global_scope();

    let mut agent_mem = MemoryBuilder::new()
        .content(Content::Text("Agent专属".to_string()))
        .build();
    agent_mem.attributes.set_agent_scope("agent_001");

    let mut user_mem = MemoryBuilder::new()
        .content(Content::Text("用户个人数据".to_string()))
        .build();
    user_mem.attributes.set_user_scope("agent_001", "user_001");

    let mut session_mem = MemoryBuilder::new()
        .content(Content::Text("会话临时数据".to_string()))
        .build();
    session_mem
        .attributes
        .set_session_scope("agent_001", "user_001", "session_001");

    // 验证层次级别
    assert_eq!(global_mem.attributes.infer_scope_level(), 0);
    assert_eq!(agent_mem.attributes.infer_scope_level(), 1);
    assert_eq!(user_mem.attributes.infer_scope_level(), 2);
    assert_eq!(session_mem.attributes.infer_scope_level(), 3);

    // 验证访问控制（高权限可访问低权限）
    // Global (level 0) 可以访问所有其他级别
    assert!(global_mem.attributes.can_access(&agent_mem.attributes));
    assert!(global_mem.attributes.can_access(&user_mem.attributes));
    assert!(global_mem.attributes.can_access(&session_mem.attributes));
    
    // Agent (level 1) 可以访问 User 和 Session
    assert!(agent_mem.attributes.can_access(&user_mem.attributes));
    assert!(agent_mem.attributes.can_access(&session_mem.attributes));
    
    // User (level 2) 可以访问 Session
    assert!(user_mem.attributes.can_access(&session_mem.attributes));
    
    // 反向不可访问（低权限不能访问高权限）
    assert!(!session_mem.attributes.can_access(&global_mem.attributes));

    println!("✅ Hierarchical scope access control test passed");
    println!("   - Global scope: Level 0 (public)");
    println!("   - Agent scope: Level 1 (agent-specific)");
    println!("   - User scope: Level 2 (user-specific)");
    println!("   - Session scope: Level 3 (session-specific)");
}

/// E2E集成测试：Pipeline + Query + AttributeSet
#[tokio::test]
async fn test_pipeline_query_attributeset_integration() {
    // 1. 创建带丰富属性的记忆
    let memory = MemoryBuilder::new()
        .content(Content::Text("产品推荐：iPhone 15 Pro".to_string()))
        .attribute(
            AttributeKey::domain("product_id"),
            AttributeValue::String("iphone_15_pro".to_string()),
        )
        .attribute(
            AttributeKey::domain("price"),
            AttributeValue::Number(7999.0),
        )
        .attribute(
            AttributeKey::domain("category"),
            AttributeValue::String("electronics".to_string()),
        )
        .attribute(
            AttributeKey::user("preference"),
            AttributeValue::String("high_end".to_string()),
        )
        .build();

    // 2. Pipeline处理
    let pipeline = Pipeline::<Memory, Memory>::new("product_pipeline")
        .add_stage(FullMemoryProcessingStage);

    let mut context = PipelineContext::new();
    let processed = pipeline.execute(memory, &mut context).await.unwrap();

    // 3. Query with constraints
    use agent_mem_core::types::{ComparisonOperator, Constraint};

    let mut query = QueryBuilder::new().text("iPhone").build();
    query
        .constraints
        .push(Constraint::AttributeRange {
            key: AttributeKey::domain("price"),
            min: 5000.0,
            max: 10000.0,
        });
    query
        .constraints
        .push(Constraint::AttributeMatch {
            key: AttributeKey::domain("category"),
            operator: ComparisonOperator::Equal,
            value: AttributeValue::String("electronics".to_string()),
        });

    // 验证Query约束
    assert_eq!(query.constraints.len(), 2);

    // 验证AttributeSet
    assert!(matches!(
        processed.attributes.get(&AttributeKey::domain("price")),
        Some(AttributeValue::Number(n)) if (*n - 7999.0).abs() < 0.01
    ));
    assert!(matches!(
        processed
            .attributes
            .get(&AttributeKey::domain("category")),
        Some(AttributeValue::String(s)) if s == "electronics"
    ));

    println!("✅ Pipeline + Query + AttributeSet integration test passed");
    println!("   - AttributeSet: 4 attributes");
    println!("   - Pipeline: processed");
    println!("   - Query: 2 constraints");
}

