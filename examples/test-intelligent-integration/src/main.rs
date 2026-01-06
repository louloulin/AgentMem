//! 测试智能功能集成
//!
//! 这个示例程序测试 FactExtractor 和 DecisionEngine 的 trait 实现

use agent_mem_intelligence::{FactExtractor, MemoryDecisionEngine};
use agent_mem_llm::providers::LocalTestProvider;
use agent_mem_traits::{
    DecisionEngine, ExtractedFact, FactExtractor as FactExtractorTrait, LLMProvider, Message,
    MessageRole, MemoryItem, Session,
};
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    println!("🧪 测试智能功能集成\n");

    // 创建 LLM 提供商 (使用本地测试提供商)
    let llm_provider: Arc<dyn LLMProvider> = Arc::new(LocalTestProvider::new());

    // 测试 1: FactExtractor trait 实现
    println!("📝 测试 1: FactExtractor trait 实现");
    test_fact_extractor(llm_provider.clone()).await?;

    // 测试 2: DecisionEngine trait 实现
    println!("\n🤖 测试 2: DecisionEngine trait 实现");
    test_decision_engine(llm_provider.clone()).await?;

    // 测试 3: 完整流程
    println!("\n🔄 测试 3: 完整智能处理流程");
    test_full_flow(llm_provider.clone()).await?;

    println!("\n✅ 所有测试通过！");
    Ok(())
}

/// 测试 FactExtractor trait 实现
async fn test_fact_extractor(llm_provider: Arc<dyn LLMProvider>) -> anyhow::Result<()> {
    // 创建 FactExtractor
    let fact_extractor = FactExtractor::new(llm_provider);

    // 创建测试消息
    let messages = vec![
        Message {
            role: MessageRole::User,
            content: "我的名字是张三，我住在北京。".to_string(),
            timestamp: Some(chrono::Utc::now()),
        },
        Message {
            role: MessageRole::User,
            content: "我喜欢编程，特别是 Rust 语言。".to_string(),
            timestamp: Some(chrono::Utc::now()),
        },
    ];

    // 调用 trait 方法
    let facts = fact_extractor.extract_facts(&messages).await?;

    println!("  提取到 {} 个事实:", facts.len());
    for (i, fact) in facts.iter().enumerate() {
        println!("    {}. {} (置信度: {:.2}, 类别: {})",
            i + 1, fact.content, fact.confidence, fact.category);
    }

    assert!(!facts.is_empty(), "应该提取到至少一个事实");
    println!("  ✅ FactExtractor trait 测试通过");

    Ok(())
}

/// 测试 DecisionEngine trait 实现
async fn test_decision_engine(llm_provider: Arc<dyn LLMProvider>) -> anyhow::Result<()> {
    // 创建 DecisionEngine
    let decision_engine = MemoryDecisionEngine::new(llm_provider);

    // 创建测试事实
    let fact = ExtractedFact {
        content: "用户喜欢 Rust 编程".to_string(),
        confidence: 0.9,
        category: "preference".to_string(),
        metadata: HashMap::new(),
    };

    // 创建现有记忆
    let existing_memories = vec![
        MemoryItem {
            id: "mem1".to_string(),
            content: "用户喜欢编程".to_string(),
            hash: None,
            metadata: HashMap::new(),
            score: Some(0.8),
            created_at: chrono::Utc::now(),
            updated_at: None,
            session: Session {
                id: "session1".to_string(),
                user_id: Some("user1".to_string()),
                agent_id: Some("agent1".to_string()),
                run_id: None,
                actor_id: None,
                created_at: chrono::Utc::now(),
                metadata: HashMap::new(),
            },
            memory_type: agent_mem_traits::MemoryType::Episodic,
            entities: vec![],
            relations: vec![],
            agent_id: "agent1".to_string(),
            user_id: Some("user1".to_string()),
            importance: 0.7,
            embedding: None,
            last_accessed_at: chrono::Utc::now(),
            access_count: 1,
            expires_at: None,
            version: 1,
        },
    ];

    // 调用 trait 方法
    let decision = decision_engine.decide(&fact, &existing_memories).await?;

    println!("  决策结果:");
    println!("    操作: {:?}", decision.action);
    println!("    置信度: {:.2}", decision.confidence);
    println!("    推理: {}", decision.reasoning);

    assert!(decision.confidence > 0.0, "决策置信度应该大于 0");
    println!("  ✅ DecisionEngine trait 测试通过");

    Ok(())
}

/// 测试完整智能处理流程
async fn test_full_flow(llm_provider: Arc<dyn LLMProvider>) -> anyhow::Result<()> {
    println!("  模拟完整的智能记忆处理流程:");

    // 1. 创建组件
    let fact_extractor = FactExtractor::new(llm_provider.clone());
    let decision_engine = MemoryDecisionEngine::new(llm_provider);

    // 2. 提取事实
    let messages = vec![Message {
        role: MessageRole::User,
        content: "我最近在学习 Rust 的异步编程，特别是 tokio 框架。".to_string(),
        timestamp: Some(chrono::Utc::now()),
    }];

    println!("  步骤 1: 提取事实");
    let facts = fact_extractor.extract_facts(&messages).await?;
    println!("    提取到 {} 个事实", facts.len());

    // 3. 为每个事实做决策
    println!("  步骤 2: 为每个事实做决策");
    let existing_memories = vec![]; // 假设没有现有记忆

    for (i, fact) in facts.iter().enumerate() {
        let decision = decision_engine.decide(fact, &existing_memories).await?;
        println!("    事实 {}: {:?} (置信度: {:.2})",
            i + 1, decision.action, decision.confidence);
    }

    println!("  ✅ 完整流程测试通过");

    Ok(())
}

