//! Phase 4: 智能处理集成增强演示
//!
//! 本演示展示了 AgentMem Phase 4 的核心功能：
//! 1. 高级事实提取
//! 2. 智能决策引擎
//! 3. 冲突解决系统
//! 4. 重要性评估器
//! 5. 集成智能处理流水线

use agent_mem_intelligence::{
    conflict_resolution::{ConflictResolver, ConflictResolverConfig},
    decision_engine::{ExistingMemory, MemoryDecisionEngine},
    fact_extraction::{ExtractedFact, FactCategory, FactExtractor},
    importance_evaluator::{ImportanceEvaluator, ImportanceEvaluatorConfig},
    intelligent_processor::IntelligentMemoryProcessor,
};
use agent_mem_llm::factory::RealLLMFactory;
use agent_mem_traits::{
    LLMConfig, MemoryItem, MemoryType, Message, MessageRole, ProcessingResult, Session,
};
use anyhow::Result;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 启动 Phase 4: 智能处理集成增强演示");

    // 演示各个组件
    demo_fact_extraction().await?;
    demo_decision_engine().await?;
    demo_conflict_resolution().await?;
    demo_importance_evaluation().await?;
    demo_integrated_processing().await?;

    info!("✅ Phase 4 演示完成！");
    Ok(())
}

/// 创建真实的 LLM 提供商（移除 Mock 降级）
async fn create_llm_provider() -> Arc<dyn agent_mem_traits::LLMProvider + Send + Sync> {
    // 尝试多个提供商的配置，按优先级排序
    let provider_configs = vec![
        // 1. 尝试 Ollama (本地)
        LLMConfig {
            provider: "ollama".to_string(),
            model: "llama3.2:3b".to_string(),
            api_key: None,
            base_url: Some("http://localhost:11434".to_string()),
            temperature: Some(0.7),
            max_tokens: Some(4000),
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            response_format: None,
        },
        // 2. 尝试 OpenAI (如果有 API key)
        LLMConfig {
            provider: "openai".to_string(),
            model: "gpt-3.5-turbo".to_string(),
            api_key: std::env::var("OPENAI_API_KEY").ok(),
            base_url: None,
            temperature: Some(0.7),
            max_tokens: Some(4000),
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            response_format: None,
        },
        // 3. 尝试 Anthropic (如果有 API key)
        LLMConfig {
            provider: "anthropic".to_string(),
            model: "claude-3-haiku-20240307".to_string(),
            api_key: std::env::var("ANTHROPIC_API_KEY").ok(),
            base_url: None,
            temperature: Some(0.7),
            max_tokens: Some(4000),
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            response_format: None,
        },
    ];

    // 尝试每个配置，直到找到可用的提供商
    for config in provider_configs {
        // 跳过没有 API key 的云提供商
        if (config.provider == "openai" || config.provider == "anthropic")
            && config.api_key.is_none()
        {
            continue;
        }

        match LLMFactory::create_provider(&config) {
            Ok(provider) => {
                info!("✅ 成功创建 LLM 提供商: {}", config.provider);
                return provider;
            }
            Err(e) => {
                warn!("❌ 无法创建 {} 提供商: {}", config.provider, e);
                continue;
            }
        }
    }

    // 如果所有提供商都失败，返回错误而不是 Mock
    panic!(
        "❌ 无法创建任何 LLM 提供商。请确保：\n\
           1. Ollama 服务正在运行 (http://localhost:11434)\n\
           2. 或设置 OPENAI_API_KEY 环境变量\n\
           3. 或设置 ANTHROPIC_API_KEY 环境变量"
    );
}

/// 演示高级事实提取功能
async fn demo_fact_extraction() -> Result<()> {
    info!("\n📊 === 高级事实提取演示 ===");

    let messages = vec![
        Message {
            role: MessageRole::User,
            content: "我叫张三，今年30岁，是一名软件工程师，住在北京。我喜欢编程和阅读。"
                .to_string(),
            timestamp: Some(Utc::now()),
        },
        Message {
            role: MessageRole::User,
            content: "我在阿里巴巴工作，主要负责后端开发，使用Java和Python。".to_string(),
            timestamp: Some(Utc::now()),
        },
    ];

    // 创建事实提取器
    let llm = create_llm_provider().await;
    let fact_extractor = FactExtractor::new(llm);

    // 提取事实
    let facts = fact_extractor.extract_facts_internal(&messages).await?;

    info!("提取到 {} 个事实:", facts.len());
    for (i, fact) in facts.iter().enumerate() {
        info!(
            "  {}. {} (置信度: {:.2}, 类别: {:?})",
            i + 1,
            fact.content,
            fact.confidence,
            fact.category
        );
    }

    Ok(())
}

/// 演示智能决策引擎
async fn demo_decision_engine() -> Result<()> {
    info!("\n🧠 === 智能决策引擎演示 ===");

    // 创建决策引擎
    let llm = create_llm_provider().await;
    let decision_engine = MemoryDecisionEngine::new(llm);

    // 模拟提取的事实
    let facts = vec![
        ExtractedFact {
            content: "用户姓名：张三".to_string(),
            confidence: 0.95,
            category: FactCategory::Personal,
            entities: vec![],
            temporal_info: None,
            source_message_id: Some("0".to_string()),
            metadata: HashMap::new(),
        },
        ExtractedFact {
            content: "用户职业：软件工程师".to_string(),
            confidence: 0.90,
            category: FactCategory::Professional,
            entities: vec![],
            temporal_info: None,
            source_message_id: Some("0".to_string()),
            metadata: HashMap::new(),
        },
    ];

    // 模拟现有记忆
    let existing_memories = vec![ExistingMemory {
        id: Uuid::new_v4().to_string(),
        content: "用户姓名：李四".to_string(),
        importance: 0.8,
        created_at: Utc::now().to_rfc3339(),
        updated_at: None,
        metadata: HashMap::new(),
    }];

    // 生成决策
    let decisions = decision_engine
        .make_decisions(&facts, &existing_memories)
        .await?;

    info!("生成 {} 个记忆决策:", decisions.len());
    for (i, decision) in decisions.iter().enumerate() {
        info!(
            "  {}. 动作: {:?}, 置信度: {:.2}, 原因: {}",
            i + 1,
            decision.action,
            decision.confidence,
            decision.reasoning
        );
    }

    Ok(())
}

/// 演示冲突解决系统
async fn demo_conflict_resolution() -> Result<()> {
    info!("\n⚔️ === 冲突解决系统演示 ===");

    // 创建冲突解决器
    let llm = create_llm_provider().await;
    let conflict_resolver = ConflictResolver::new(llm, ConflictResolverConfig::default());

    // 创建测试记忆
    let new_memories = vec![
        create_test_memory("用户姓名：张三", 0.9),
        create_test_memory("用户年龄：30岁", 0.8),
    ];

    let existing_memories = vec![
        create_test_memory("用户姓名：李四", 0.7),
        create_test_memory("用户年龄：25岁", 0.6),
    ];

    // 检测冲突
    let conflicts = conflict_resolver
        .detect_conflicts(&new_memories, &existing_memories)
        .await?;

    info!("检测到 {} 个潜在冲突:", conflicts.len());
    for (i, conflict) in conflicts.iter().enumerate() {
        info!(
            "  {}. 冲突类型: {:?}, 置信度: {:.2}",
            i + 1,
            conflict.conflict_type,
            conflict.confidence
        );
        info!("     描述: {}", conflict.description);
    }

    Ok(())
}

/// 演示重要性评估器
async fn demo_importance_evaluation() -> Result<()> {
    info!("\n⭐ === 重要性评估器演示 ===");

    // 创建重要性评估器
    let llm = create_llm_provider().await;
    let importance_evaluator = ImportanceEvaluator::new(llm, ImportanceEvaluatorConfig::default());

    // 创建测试记忆
    let memory = create_test_memory("用户是资深软件工程师，有10年经验", 0.8);

    // 评估重要性
    let evaluation = importance_evaluator
        .evaluate_importance(&memory, &[], &[])
        .await?;

    info!("重要性评估结果:");
    info!("  重要性分数: {:.2}", evaluation.importance_score);
    info!("  置信度: {:.2}", evaluation.confidence);
    info!("  推理: {}", evaluation.reasoning);

    Ok(())
}

/// 演示集成智能处理流水线
async fn demo_integrated_processing() -> Result<()> {
    info!("\n🔄 === 集成智能处理流水线演示 ===");

    // 创建真实的 LLM 提供商
    let llm = create_llm_provider().await;

    let fact_extractor = FactExtractor::new(llm.clone());
    let decision_engine = MemoryDecisionEngine::new(llm.clone());
    let conflict_resolver = ConflictResolver::new(llm.clone(), ConflictResolverConfig::default());
    let _importance_evaluator =
        ImportanceEvaluator::new(llm.clone(), ImportanceEvaluatorConfig::default());

    // 准备测试消息
    let messages = vec![
        Message {
            role: MessageRole::User,
            content: "我是王五，今年35岁，在腾讯工作，是一名高级架构师。".to_string(),
            timestamp: Some(Utc::now()),
        },
        Message {
            role: MessageRole::User,
            content: "我负责微服务架构设计，熟悉Kubernetes和Docker。".to_string(),
            timestamp: Some(Utc::now()),
        },
    ];

    // 模拟现有记忆
    let existing_memories = vec![ExistingMemory {
        id: Uuid::new_v4().to_string(),
        content: "用户是软件工程师".to_string(),
        importance: 0.7,
        created_at: Utc::now().to_rfc3339(),
        updated_at: None,
        metadata: HashMap::new(),
    }];

    // 手动执行智能处理流水线
    let start_time = std::time::Instant::now();

    // 1. 提取事实
    let extracted_facts = fact_extractor.extract_facts_internal(&messages).await?;

    // 2. 生成决策
    let memory_decisions = decision_engine
        .make_decisions(&extracted_facts, &existing_memories)
        .await?;

    // 3. 检测冲突
    let memories: Vec<_> = existing_memories
        .iter()
        .map(|m| create_test_memory(&m.content, m.importance))
        .collect();
    let conflict_detections = conflict_resolver
        .detect_conflicts(&memories, &memories)
        .await?;

    let processing_time = start_time.elapsed().as_millis() as f64;

    info!("智能处理结果:");
    info!("  提取事实数: {}", extracted_facts.len());
    info!("  记忆决策数: {}", memory_decisions.len());
    info!("  冲突检测数: {}", conflict_detections.len());
    info!("  处理时间: {:.2}ms", processing_time);

    // 显示详细结果
    if !extracted_facts.is_empty() {
        info!("  提取的事实:");
        for (i, fact) in extracted_facts.iter().enumerate() {
            info!(
                "    {}. {} (置信度: {:.2})",
                i + 1,
                fact.content,
                fact.confidence
            );
        }
    }

    if !memory_decisions.is_empty() {
        info!("  记忆决策:");
        for (i, decision) in memory_decisions.iter().enumerate() {
            info!(
                "    {}. {:?} (置信度: {:.2})",
                i + 1,
                decision.action,
                decision.confidence
            );
        }
    }

    Ok(())
}

/// 创建测试记忆项
fn create_test_memory(content: &str, importance: f32) -> MemoryItem {
    MemoryItem {
        id: Uuid::new_v4().to_string(),
        content: content.to_string(),
        hash: None,
        metadata: HashMap::new(),
        score: Some(importance),
        created_at: Utc::now(),
        updated_at: None,
        session: Session::default(),
        memory_type: MemoryType::Episodic,
        entities: vec![],
        relations: vec![],
        agent_id: "demo".to_string(),
        user_id: None,
        importance,
        embedding: None,
        last_accessed_at: Utc::now(),
        access_count: 0,
        expires_at: None,
        version: 1,
    }
}

// MockLLMProvider 已移除 - 现在只使用真实的 LLM 提供商
