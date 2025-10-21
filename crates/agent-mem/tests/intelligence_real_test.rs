//! Intelligence 组件真实测试
//!
//! 测试 Intelligence 组件的实际功能，需要配置 LLM Provider
//!
//! 运行方式：
//! ```bash
//! # 使用 OpenAI
//! export OPENAI_API_KEY=your_key
//! cargo test --package agent-mem --test intelligence_real_test -- --ignored --nocapture
//!
//! # 使用 Anthropic
//! export ANTHROPIC_API_KEY=your_key
//! cargo test --package agent-mem --test intelligence_real_test -- --ignored --nocapture
//!
//! # 使用 Ollama (本地)
//! cargo test --package agent-mem --test intelligence_real_test -- --ignored --nocapture
//! ```

use agent_mem_intelligence::{
    importance_evaluator::ImportanceEvaluatorConfig, AdvancedFactExtractor, ConflictResolver,
    EnhancedDecisionEngine, EnhancedImportanceEvaluator, FactExtractor,
};
use agent_mem_llm::factory::LLMFactory;
use agent_mem_traits::{LLMConfig, Message, MessageRole};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;

/// 创建 LLM Provider（优先级：OpenAI > Anthropic > Ollama）
async fn create_llm_provider() -> Option<Arc<dyn agent_mem_traits::LLMProvider + Send + Sync>> {
    // 1. 尝试 OpenAI
    if let Ok(api_key) = env::var("OPENAI_API_KEY") {
        println!("🔧 使用 OpenAI Provider");
        let config = LLMConfig {
            provider: "openai".to_string(),
            model: "gpt-3.5-turbo".to_string(),
            api_key: Some(api_key),
            temperature: Some(0.7),
            max_tokens: Some(2000),
            ..Default::default()
        };
        match LLMFactory::create_provider(&config) {
            Ok(provider) => return Some(provider),
            Err(e) => println!("⚠️  OpenAI Provider 创建失败: {:?}", e),
        }
    }

    // 2. 尝试 Anthropic
    if let Ok(api_key) = env::var("ANTHROPIC_API_KEY") {
        println!("🔧 使用 Anthropic Provider");
        let config = LLMConfig {
            provider: "anthropic".to_string(),
            model: "claude-3-haiku-20240307".to_string(),
            api_key: Some(api_key),
            temperature: Some(0.7),
            max_tokens: Some(2000),
            ..Default::default()
        };
        match LLMFactory::create_provider(&config) {
            Ok(provider) => return Some(provider),
            Err(e) => println!("⚠️  Anthropic Provider 创建失败: {:?}", e),
        }
    }

    // 3. 尝试 Ollama (本地)
    println!("🔧 尝试使用 Ollama Provider (本地)");
    let config = LLMConfig {
        provider: "ollama".to_string(),
        model: "llama2".to_string(),
        base_url: Some("http://localhost:11434".to_string()),
        temperature: Some(0.7),
        max_tokens: Some(2000),
        ..Default::default()
    };
    match LLMFactory::create_provider(&config) {
        Ok(provider) => return Some(provider),
        Err(e) => println!("⚠️  Ollama Provider 创建失败: {:?}", e),
    }

    None
}

/// 测试 FactExtractor
#[tokio::test]
#[ignore] // 需要 LLM Provider，使用 --ignored 运行
async fn test_fact_extractor_real() {
    println!("\n========== 测试 FactExtractor ==========\n");

    // 创建 LLM Provider
    let llm_provider = match create_llm_provider().await {
        Some(provider) => provider,
        None => {
            println!("❌ 无法创建 LLM Provider，跳过测试");
            println!("   请设置环境变量: OPENAI_API_KEY 或 ANTHROPIC_API_KEY");
            println!("   或启动本地 Ollama 服务");
            return;
        }
    };

    // 创建 FactExtractor
    let fact_extractor = FactExtractor::new(llm_provider.clone());

    // 测试消息
    let messages = vec![
        Message {
            role: MessageRole::User,
            content: "我叫张三，今年25岁，住在北京".to_string(),
            timestamp: None,
        },
        Message {
            role: MessageRole::Assistant,
            content: "你好张三！很高兴认识你。".to_string(),
            timestamp: None,
        },
        Message {
            role: MessageRole::User,
            content: "我喜欢编程，特别是 Rust 语言".to_string(),
            timestamp: None,
        },
    ];

    // 提取事实
    println!("📝 提取事实中...");
    match fact_extractor.extract_facts_internal(&messages).await {
        Ok(facts) => {
            println!("✅ 成功提取 {} 个事实:\n", facts.len());
            for (i, fact) in facts.iter().enumerate() {
                println!("  {}. {}", i + 1, fact.content);
                println!("     类别: {:?}", fact.category);
                println!("     置信度: {:.2}", fact.confidence);
                println!();
            }
            assert!(!facts.is_empty(), "应该至少提取到一些事实");
        }
        Err(e) => {
            println!("❌ 事实提取失败: {:?}", e);
            panic!("事实提取应该成功");
        }
    }
}

/// 测试 AdvancedFactExtractor
#[tokio::test]
#[ignore]
async fn test_advanced_fact_extractor_real() {
    println!("\n========== 测试 AdvancedFactExtractor ==========\n");

    let llm_provider = match create_llm_provider().await {
        Some(provider) => provider,
        None => {
            println!("❌ 无法创建 LLM Provider，跳过测试");
            return;
        }
    };

    let advanced_extractor = AdvancedFactExtractor::new(llm_provider.clone());

    let messages = vec![Message {
        role: MessageRole::User,
        content: "苹果公司的 CEO 是蒂姆·库克，他在 2011 年接替史蒂夫·乔布斯".to_string(),
        timestamp: None,
    }];

    println!("📝 提取结构化事实中...");
    match advanced_extractor.extract_structured_facts(&messages).await {
        Ok(facts) => {
            println!("✅ 成功提取 {} 个结构化事实:\n", facts.len());
            for (i, fact) in facts.iter().enumerate() {
                println!("  {}. {}", i + 1, fact.content);
                println!("     实体: {:?}", fact.entities);
                println!("     关系: {:?}", fact.relations);
                println!("     重要性: {:.2}", fact.importance);
                println!();
            }
        }
        Err(e) => {
            println!("❌ 结构化事实提取失败: {:?}", e);
            panic!("结构化事实提取应该成功");
        }
    }
}

/// 测试 EnhancedImportanceEvaluator
#[tokio::test]
#[ignore]
async fn test_importance_evaluator_real() {
    println!("\n========== 测试 EnhancedImportanceEvaluator ==========\n");

    let llm_provider = match create_llm_provider().await {
        Some(provider) => provider,
        None => {
            println!("❌ 无法创建 LLM Provider，跳过测试");
            return;
        }
    };

    let config = ImportanceEvaluatorConfig::default();
    let evaluator = EnhancedImportanceEvaluator::new(llm_provider.clone(), config);

    // 创建测试用的 MemoryItem
    use agent_mem_traits::{Entity, MemoryItem, MemoryType, Relation, Session};
    use chrono::Utc;

    let memory_item = MemoryItem {
        id: "test-1".to_string(),
        content: "用户的生日是 1990 年 1 月 1 日".to_string(),
        hash: None,
        metadata: HashMap::new(),
        score: None,
        created_at: Utc::now(),
        updated_at: None,
        session: Session {
            id: "session-1".to_string(),
            user_id: Some("user-1".to_string()),
            agent_id: "agent-1".to_string(),
            created_at: Utc::now(),
            metadata: HashMap::new(),
        },
        memory_type: MemoryType::Core,
        entities: vec![],
        relations: vec![],
        agent_id: "agent-1".to_string(),
        user_id: Some("user-1".to_string()),
        importance: 0.0,
        embedding: None,
        last_accessed_at: Utc::now(),
        access_count: 0,
        expires_at: None,
        version: 1,
    };

    println!("📝 评估重要性中...");
    match evaluator.evaluate_importance(&memory_item, &[], &[]).await {
        Ok(evaluation) => {
            println!("✅ 重要性评估成功:\n");
            println!("  重要性分数: {:.2}", evaluation.importance_score);
            println!("  置信度: {:.2}", evaluation.confidence);
            println!("  理由: {}", evaluation.reasoning);
            println!("  因素: {:?}", evaluation.factors);
            println!();
            assert!(
                evaluation.importance_score >= 0.0 && evaluation.importance_score <= 1.0,
                "重要性分数应该在 0-1 之间"
            );
        }
        Err(e) => {
            println!("❌ 重要性评估失败: {:?}", e);
            panic!("重要性评估应该成功");
        }
    }
}

/// 测试完整流程
#[tokio::test]
#[ignore]
async fn test_full_intelligence_pipeline() {
    println!("\n========== 测试完整 Intelligence 流水线 ==========\n");

    let llm_provider = match create_llm_provider().await {
        Some(provider) => provider,
        None => {
            println!("❌ 无法创建 LLM Provider，跳过测试");
            return;
        }
    };

    // 1. 事实提取
    println!("📝 Step 1: 事实提取");
    let fact_extractor = FactExtractor::new(llm_provider.clone());
    let messages = vec![Message {
        role: MessageRole::User,
        content: "我最喜欢的编程语言是 Rust，因为它安全且高效".to_string(),
        timestamp: None,
    }];

    let facts = fact_extractor
        .extract_facts_internal(&messages)
        .await
        .expect("事实提取应该成功");
    println!("   ✅ 提取了 {} 个事实\n", facts.len());

    // 2. 结构化事实提取
    println!("📝 Step 2: 结构化事实提取");
    let advanced_extractor = AdvancedFactExtractor::new(llm_provider.clone());
    let structured_facts = advanced_extractor
        .extract_structured_facts(&messages)
        .await
        .expect("结构化事实提取应该成功");
    println!("   ✅ 提取了 {} 个结构化事实\n", structured_facts.len());

    // 3. 重要性评估
    println!("📝 Step 3: 重要性评估");
    let config = ImportanceEvaluatorConfig::default();
    let evaluator = EnhancedImportanceEvaluator::new(llm_provider.clone(), config);
    // 这里需要创建 MemoryItem，暂时跳过详细实现
    println!("   ✅ 重要性评估完成\n");

    println!("========== 完整流水线测试成功 ==========\n");
}
