//! 智能推理引擎演示
//!
//! 展示如何使用 DeepSeek 驱动的智能推理引擎进行事实提取和记忆决策

use agent_mem_intelligence::{ExistingMemory, IntelligentMemoryProcessor};
use agent_mem_traits::{Message, MessageRole};
use chrono;
use std::collections::HashMap;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    println!("🧠 AgentMem 智能推理引擎演示");
    println!("================================");

    // 使用提供的 DeepSeek API 密钥
    let api_key = "sk-8790bf8b4f6c4afca432e8661508119c".to_string();

    // 创建智能处理器
    let processor = match IntelligentMemoryProcessor::new(api_key).await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("❌ 创建智能处理器失败: {}", e);
            return Err(e.into());
        }
    };

    println!("✅ 智能处理器创建成功");

    // 准备简化的测试消息
    let messages = vec![
        Message {
            role: MessageRole::User,
            content: "Hi, I'm John from San Francisco. I love coffee.".to_string(),
            timestamp: Some(chrono::Utc::now()),
        },
        Message {
            role: MessageRole::User,
            content: "I work with Rust and Python. I enjoy hiking.".to_string(),
            timestamp: Some(chrono::Utc::now()),
        },
    ];

    // 准备简化的现有记忆
    let existing_memories = vec![ExistingMemory {
        id: "mem1".to_string(),
        content: "User likes tea".to_string(),
        importance: 0.5,
        created_at: "2023-12-01T00:00:00Z".to_string(),
        updated_at: None,
        metadata: HashMap::new(),
    }];

    println!("\n📝 处理消息...");
    println!("消息数量: {}", messages.len());
    println!("现有记忆数量: {}", existing_memories.len());

    // 处理消息
    match processor
        .process_messages(&messages, &existing_memories)
        .await
    {
        Ok(result) => {
            println!("\n🎉 处理完成!");

            // 显示提取的事实
            println!("\n📊 提取的事实 ({}):", result.extracted_facts.len());
            for (i, fact) in result.extracted_facts.iter().enumerate() {
                println!(
                    "  {}. [{}] {} (置信度: {:.2})",
                    i + 1,
                    format!("{:?}", fact.category),
                    fact.content,
                    fact.confidence
                );
                if !fact.entities.is_empty() {
                    println!("     实体: {:?}", fact.entities);
                }
            }

            // 显示记忆决策
            println!("\n🎯 记忆决策 ({}):", result.memory_decisions.len());
            for (i, decision) in result.memory_decisions.iter().enumerate() {
                println!("  {}. 操作: {:?}", i + 1, decision.action);
                println!("     置信度: {:.2}", decision.confidence);
                println!("     原因: {}", decision.reasoning);
                if !decision.affected_memories.is_empty() {
                    println!("     影响的记忆: {:?}", decision.affected_memories);
                }
                println!();
            }

            // 显示处理统计
            println!("📈 处理统计:");
            println!("  - 总消息数: {}", result.processing_stats.total_messages);
            println!(
                "  - 提取事实数: {}",
                result.processing_stats.facts_extracted
            );
            println!("  - 生成决策数: {}", result.processing_stats.decisions_made);
            println!(
                "  - 高置信度决策: {}",
                result.processing_stats.high_confidence_decisions
            );
            println!(
                "  - 处理时间: {}ms",
                result.processing_stats.processing_time_ms
            );

            // 显示推荐
            if !result.recommendations.is_empty() {
                println!("\n💡 推荐:");
                for (i, rec) in result.recommendations.iter().enumerate() {
                    println!("  {}. {}", i + 1, rec);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ 处理失败: {}", e);
            return Err(e.into());
        }
    }

    // 测试记忆健康分析
    println!("\n🔍 分析记忆健康状况...");
    match processor.analyze_memory_health(&existing_memories).await {
        Ok(health_report) => {
            println!("✅ 记忆健康分析完成");
            println!("  - 总记忆数: {}", existing_memories.len());
            println!(
                "  - 低重要性记忆: {}",
                health_report.low_importance_memories.len()
            );
            println!("  - 短记忆: {}", health_report.short_memories.len());
            println!("  - 重复记忆对: {}", health_report.duplicate_memories.len());

            if !health_report.suggestions.is_empty() {
                println!("  建议:");
                for suggestion in &health_report.suggestions {
                    println!("    - {}", suggestion);
                }
            }
        }
        Err(e) => {
            eprintln!("⚠️  记忆健康分析失败: {}", e);
        }
    }

    println!("\n🎊 演示完成!");
    Ok(())
}
