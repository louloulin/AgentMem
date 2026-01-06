//! Context-Aware Memory Demo
//!
//! This demo showcases the context-aware memory management capabilities of AgentMem,
//! including intelligent context extraction, context-based search, and adaptive learning.

use agent_mem_compat::{ContextAwareSearchRequest, ContextInfo, ContextPattern, Mem0Client};
use agent_mem_traits::Session;
use std::collections::HashMap;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 启动上下文感知记忆演示\n");

    // Create Mem0Client
    let client = Mem0Client::new().await?;

    // Create a test session
    let session = Session {
        id: "context_demo_session".to_string(),
        user_id: Some("demo_user".to_string()),
        agent_id: Some("demo_agent".to_string()),
        run_id: None,
        actor_id: None,
        created_at: chrono::Utc::now(),
        metadata: HashMap::new(),
    };

    // Demo 1: Context Extraction
    println!("🎯 演示 1: 上下文提取");
    demo_context_extraction(&client, &session).await?;

    // Demo 2: Context-Aware Search
    println!("\n🎯 演示 2: 上下文感知搜索");
    demo_context_aware_search(&client, &session).await?;

    // Demo 3: Context Learning
    println!("\n🎯 演示 3: 上下文学习");
    demo_context_learning(&client).await?;

    // Demo 4: Context Patterns
    println!("\n🎯 演示 4: 上下文模式识别");
    demo_context_patterns(&client).await?;

    // Demo 5: Context Statistics
    println!("\n🎯 演示 5: 上下文统计");
    demo_context_statistics(&client).await?;

    println!("\n✅ 所有上下文感知演示完成！");
    Ok(())
}

async fn demo_context_extraction(
    client: &Mem0Client,
    session: &Session,
) -> Result<(), Box<dyn std::error::Error>> {
    let test_contents = vec![
        "今天早上我需要完成编程项目的开发工作",
        "昨天晚上我很开心地和朋友们一起看电影",
        "明天我要去办公室参加重要的会议",
        "我正在学习 Rust 编程语言，感觉很有挑战性",
        "在家里我喜欢听音乐和阅读技术书籍",
    ];

    for (i, content) in test_contents.iter().enumerate() {
        println!("  📝 内容 {}: {}", i + 1, content);

        match client.extract_context(content, session).await {
            Ok(contexts) => {
                if contexts.is_empty() {
                    println!("    ❌ 未提取到上下文信息");
                } else {
                    println!("    ✅ 提取到 {} 个上下文:", contexts.len());
                    for context in &contexts {
                        println!(
                            "      - {}: {} (置信度: {:.2})",
                            context.context_type, context.value, context.confidence
                        );
                    }
                }
            }
            Err(e) => {
                warn!("上下文提取失败: {}", e);
            }
        }
        println!();
    }

    Ok(())
}

async fn demo_context_aware_search(
    client: &Mem0Client,
    session: &Session,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create some sample contexts
    let current_contexts = vec![
        ContextInfo {
            id: uuid::Uuid::new_v4().to_string(),
            context_type: "topic".to_string(),
            value: "programming".to_string(),
            confidence: 0.9,
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
            entities: Vec::new(),
            relations: Vec::new(),
        },
        ContextInfo {
            id: uuid::Uuid::new_v4().to_string(),
            context_type: "temporal".to_string(),
            value: "morning".to_string(),
            confidence: 0.8,
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
            entities: Vec::new(),
            relations: Vec::new(),
        },
        ContextInfo {
            id: uuid::Uuid::new_v4().to_string(),
            context_type: "emotional".to_string(),
            value: "focused".to_string(),
            confidence: 0.7,
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
            entities: Vec::new(),
            relations: Vec::new(),
        },
    ];

    let search_request = ContextAwareSearchRequest {
        query: "编程学习".to_string(),
        current_context: current_contexts.clone(),
        session: session.clone(),
        limit: Some(10),
        min_relevance: Some(0.5),
        context_weight: Some(0.4),
        enable_pattern_matching: true,
    };

    println!("  🔍 搜索查询: {}", search_request.query);
    println!("  📋 当前上下文:");
    for context in &current_contexts {
        println!("    - {}: {}", context.context_type, context.value);
    }

    match client.search_with_context(search_request).await {
        Ok(results) => {
            println!("  ✅ 搜索完成，找到 {} 条结果", results.len());
            for (i, result) in results.iter().enumerate() {
                println!(
                    "    {}. 记忆: {} (相关性: {:.2}, 上下文: {:.2}, 综合: {:.2})",
                    i + 1,
                    result.memory.memory,
                    result.relevance_score,
                    result.context_score,
                    result.combined_score
                );
                if !result.context_explanation.is_empty() {
                    println!("       解释: {}", result.context_explanation);
                }
            }
        }
        Err(e) => {
            warn!("上下文感知搜索失败: {}", e);
        }
    }

    Ok(())
}

async fn demo_context_learning(client: &Mem0Client) -> Result<(), Box<dyn std::error::Error>> {
    // Create learning contexts
    let learning_contexts = vec![
        ContextInfo {
            id: uuid::Uuid::new_v4().to_string(),
            context_type: "topic".to_string(),
            value: "programming".to_string(),
            confidence: 0.9,
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
            entities: Vec::new(),
            relations: Vec::new(),
        },
        ContextInfo {
            id: uuid::Uuid::new_v4().to_string(),
            context_type: "temporal".to_string(),
            value: "evening".to_string(),
            confidence: 0.8,
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
            entities: Vec::new(),
            relations: Vec::new(),
        },
        ContextInfo {
            id: uuid::Uuid::new_v4().to_string(),
            context_type: "location".to_string(),
            value: "home".to_string(),
            confidence: 0.7,
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
            entities: Vec::new(),
            relations: Vec::new(),
        },
    ];

    println!("  📚 学习上下文模式:");
    for context in &learning_contexts {
        println!("    - {}: {}", context.context_type, context.value);
    }

    match client.learn_from_context(&learning_contexts).await {
        Ok(result) => {
            println!("  ✅ 学习完成 (置信度: {:.2})", result.confidence);

            if !result.new_patterns.is_empty() {
                println!("    🆕 新发现的模式:");
                for pattern in &result.new_patterns {
                    println!("      - {}: {:?}", pattern.name, pattern.context_types);
                }
            }

            if !result.updated_patterns.is_empty() {
                println!("    🔄 更新的模式:");
                for pattern in &result.updated_patterns {
                    println!("      - {}: 频率 {}", pattern.name, pattern.frequency);
                }
            }

            if !result.insights.is_empty() {
                println!("    💡 学习洞察:");
                for insight in &result.insights {
                    println!("      - {}", insight);
                }
            }
        }
        Err(e) => {
            warn!("上下文学习失败: {}", e);
        }
    }

    Ok(())
}

async fn demo_context_patterns(client: &Mem0Client) -> Result<(), Box<dyn std::error::Error>> {
    match client.get_context_patterns().await {
        Ok(patterns) => {
            println!("  📊 发现的上下文模式 ({} 个):", patterns.len());

            if patterns.is_empty() {
                println!("    ❌ 暂无学习到的模式");
            } else {
                for (i, pattern) in patterns.iter().enumerate() {
                    println!(
                        "    {}. {} (频率: {}, 置信度: {:.2})",
                        i + 1,
                        pattern.name,
                        pattern.frequency,
                        pattern.confidence
                    );
                    println!("       上下文类型: {:?}", pattern.context_types);
                    if !pattern.triggers.is_empty() {
                        println!("       触发条件: {:?}", pattern.triggers);
                    }
                }
            }
        }
        Err(e) => {
            warn!("获取上下文模式失败: {}", e);
        }
    }

    Ok(())
}

async fn demo_context_statistics(client: &Mem0Client) -> Result<(), Box<dyn std::error::Error>> {
    match client.get_context_statistics().await {
        Ok(stats) => {
            println!("  📈 上下文统计信息:");

            if stats.is_empty() {
                println!("    ❌ 暂无统计数据");
            } else {
                for (context_type, count) in &stats {
                    println!("    - {}: {} 次", context_type, count);
                }
            }
        }
        Err(e) => {
            warn!("获取上下文统计失败: {}", e);
        }
    }

    Ok(())
}
