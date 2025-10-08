//! 智能记忆管理演示
//!
//! 展示 AgentMem Phase 1.1 的智能功能集成：
//! - 智能事实提取 (FactExtractor)
//! - 智能决策引擎 (DecisionEngine)
//! - ADD/UPDATE/DELETE/MERGE 自动决策
//! - 降级处理和错误恢复

use agent_mem_config::MemoryConfig;
use agent_mem_core::MemoryManager;
use agent_mem_llm::factory::RealLLMFactory;
use agent_mem_traits::LLMConfig;
use anyhow::Result;
use std::sync::Arc;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 启动智能记忆管理演示");

    // 演示场景
    demo_intelligent_extraction().await?;
    demo_decision_engine().await?;
    demo_fallback_handling().await?;

    info!("✅ 智能记忆管理演示完成！");
    Ok(())
}

/// 演示 1: 智能事实提取
async fn demo_intelligent_extraction() -> Result<()> {
    info!("\n📊 === 演示 1: 智能事实提取 ===");

    // 创建 LLM 提供商
    let llm_provider = create_llm_provider().await?;

    // 创建配置 (启用智能提取)
    let mut config = MemoryConfig::default();
    config.intelligence.enable_intelligent_extraction = true;
    config.intelligence.enable_decision_engine = true;
    config.intelligence.fact_extraction.min_confidence = 0.7;

    // 创建 MemoryManager
    let manager = MemoryManager::with_llm_provider(config, llm_provider);

    // 添加包含多个事实的内容
    let content = "我叫张三，今年30岁，在北京工作。我喜欢编程和阅读，最喜欢的编程语言是 Rust。";

    info!("添加记忆: {}", content);

    match manager
        .add_memory(
            "agent_001".to_string(),
            Some("user_123".to_string()),
            content.to_string(),
            None,
            None,
            None,
        )
        .await
    {
        Ok(memory_id) => {
            info!("✅ 记忆添加成功，ID: {}", memory_id);

            // 获取记忆
            if let Some(memory) = manager.get_memory(&memory_id).await? {
                info!("记忆内容: {}", memory.content);
                info!("重要性: {}", memory.importance);
            }
        }
        Err(e) => {
            warn!("⚠️ 记忆添加失败: {}", e);
        }
    }

    Ok(())
}

/// 演示 2: 智能决策引擎
async fn demo_decision_engine() -> Result<()> {
    info!("\n🧠 === 演示 2: 智能决策引擎 ===");

    let llm_provider = create_llm_provider().await?;

    let mut config = MemoryConfig::default();
    config.intelligence.enable_intelligent_extraction = true;
    config.intelligence.enable_decision_engine = true;
    config.intelligence.decision_engine.similarity_threshold = 0.85;

    let manager = MemoryManager::with_llm_provider(config, llm_provider);

    // 场景 1: 添加新记忆
    info!("\n场景 1: 添加新记忆");
    let memory_id_1 = manager
        .add_memory(
            "agent_001".to_string(),
            Some("user_123".to_string()),
            "我喜欢吃披萨".to_string(),
            None,
            None,
            None,
        )
        .await?;
    info!("✅ 添加记忆 1: {}", memory_id_1);

    // 场景 2: 添加相似记忆 (应该触发 UPDATE 或 MERGE)
    info!("\n场景 2: 添加相似记忆 (可能触发 UPDATE/MERGE)");
    let memory_id_2 = manager
        .add_memory(
            "agent_001".to_string(),
            Some("user_123".to_string()),
            "我最喜欢的食物是意大利披萨".to_string(),
            None,
            None,
            None,
        )
        .await?;
    info!("✅ 添加记忆 2: {}", memory_id_2);

    // 场景 3: 添加矛盾记忆 (应该触发 UPDATE 或 DELETE)
    info!("\n场景 3: 添加矛盾记忆 (可能触发 UPDATE/DELETE)");
    let memory_id_3 = manager
        .add_memory(
            "agent_001".to_string(),
            Some("user_123".to_string()),
            "我不喜欢吃披萨了".to_string(),
            None,
            None,
            None,
        )
        .await?;
    info!("✅ 添加记忆 3: {}", memory_id_3);

    // 查看最终记忆
    info!("\n最终记忆列表:");
    let memories = manager
        .get_agent_memories("agent_001", Some(10))
        .await?;
    for (idx, memory) in memories.iter().enumerate() {
        info!("  {}. {} (重要性: {})", idx + 1, memory.content, memory.importance);
    }

    Ok(())
}

/// 演示 3: 降级处理
async fn demo_fallback_handling() -> Result<()> {
    info!("\n🔄 === 演示 3: 降级处理 ===");

    // 场景 1: 没有 LLM 提供商 (应该降级到简单流程)
    info!("\n场景 1: 没有 LLM 提供商 (降级到简单流程)");

    let config = MemoryConfig::default();
    let manager = MemoryManager::with_config(config);

    let memory_id = manager
        .add_memory(
            "agent_002".to_string(),
            Some("user_456".to_string()),
            "这是一个简单的记忆".to_string(),
            None,
            None,
            None,
        )
        .await?;

    info!("✅ 使用简单流程添加记忆: {}", memory_id);

    // 场景 2: 禁用智能功能
    info!("\n场景 2: 禁用智能功能");

    let llm_provider = create_llm_provider().await?;
    let mut config = MemoryConfig::default();
    config.intelligence.enable_intelligent_extraction = false;
    config.intelligence.enable_decision_engine = false;

    let manager = MemoryManager::with_llm_provider(config, llm_provider);

    let memory_id = manager
        .add_memory(
            "agent_003".to_string(),
            Some("user_789".to_string()),
            "智能功能已禁用".to_string(),
            None,
            None,
            None,
        )
        .await?;

    info!("✅ 智能功能禁用，使用简单流程: {}", memory_id);

    Ok(())
}

/// 创建 LLM 提供商
async fn create_llm_provider() -> Result<Arc<dyn agent_mem_traits::LLMProvider + Send + Sync>> {
    // 尝试多个提供商配置
    let provider_configs = vec![
        // 1. Ollama (本地)
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
        // 2. OpenAI
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
    ];

    for config in provider_configs {
        if (config.provider == "openai") && config.api_key.is_none() {
            continue;
        }

        match RealLLMFactory::create_provider(&config).await {
            Ok(provider) => {
                info!("✅ 成功创建 LLM 提供商: {}", config.provider);
                return Ok(provider);
            }
            Err(e) => {
                warn!("⚠️ 创建 {} 提供商失败: {}", config.provider, e);
                continue;
            }
        }
    }

    Err(anyhow::anyhow!("无法创建任何 LLM 提供商"))
}

