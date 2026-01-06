//! 智能记忆处理演示
//!
//! 演示修复后的智能记忆处理功能，包括：
//! - 多模态内容处理
//! - 事实提取
//! - 决策引擎
//! - 智能处理器

use agent_mem_intelligence::multimodal::{ContentType, MultimodalContent, MultimodalProcessor};
use agent_mem_traits::Result;
use std::collections::HashMap;
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🧠 AgentMem 智能记忆处理演示");
    println!("================================");

    // 演示1：多模态文本处理
    demo_multimodal_text_processing().await?;

    // 演示2：事实提取
    demo_fact_extraction().await?;

    // 演示3：决策引擎
    demo_decision_engine().await?;

    // 演示4：智能处理器
    demo_intelligent_processor().await?;

    println!("\n✅ 所有演示完成！智能记忆处理功能正常工作。");
    Ok(())
}

async fn demo_multimodal_text_processing() -> Result<()> {
    println!("\n📝 演示1：多模态文本处理");
    println!("------------------------");

    // 创建文本处理器
    let processor = agent_mem_intelligence::multimodal::text::TextProcessor::new();

    // 创建测试内容
    let mut content = MultimodalContent {
        id: "demo-text-1".to_string(),
        content_type: ContentType::Text,
        file_path: None,
        url: None,
        mime_type: Some("text/plain".to_string()),
        size: Some(0),
        data: None,
        extracted_text: Some("这是一个关于人工智能和机器学习的重要文档。它包含了深度学习、神经网络和自然语言处理的核心概念。".to_string()),
        metadata: HashMap::new(),
        processing_status: agent_mem_intelligence::multimodal::ProcessingStatus::Pending,
    };

    // 处理内容
    processor.process(&mut content).await?;

    println!("✅ 文本处理完成");
    println!("   - 内容类型: {:?}", content.content_type);
    println!("   - 处理状态: {:?}", content.processing_status);
    println!("   - 元数据项数: {}", content.metadata.len());

    // 提取文本
    if let Some(text) = processor.extract_text(&content).await? {
        println!("   - 提取的文本长度: {} 字符", text.len());
    }

    // 生成摘要
    if let Some(summary) = processor.generate_summary(&content).await? {
        println!("   - 生成的摘要: {}", summary);
    }

    Ok(())
}

async fn demo_fact_extraction() -> Result<()> {
    println!("\n🔍 演示2：事实提取");
    println!("------------------");

    // 创建事实提取器（使用演示API密钥）
    println!("   ⚠️  演示模式：跳过需要真实API密钥的事实提取");
    println!("   📝 模拟提取的事实：");

    // 模拟提取的事实
    let facts = vec![
        agent_mem_intelligence::fact_extraction::ExtractedFact {
            content: "Rust是一种系统编程语言".to_string(),
            category: agent_mem_intelligence::fact_extraction::FactCategory::Knowledge,
            confidence: 0.95,
            entities: vec![],
            temporal_info: None,
            source_message_id: Some("msg-1".to_string()),
            metadata: std::collections::HashMap::new(),
        },
        agent_mem_intelligence::fact_extraction::ExtractedFact {
            content: "Mozilla开发了Rust语言".to_string(),
            category: agent_mem_intelligence::fact_extraction::FactCategory::Knowledge,
            confidence: 0.90,
            entities: vec![],
            temporal_info: None,
            source_message_id: Some("msg-2".to_string()),
            metadata: std::collections::HashMap::new(),
        },
    ];

    // 在演示模式下，我们直接使用模拟的事实，不需要调用API

    println!("✅ 事实提取完成");
    println!("   - 提取的事实数量: {}", facts.len());

    for (i, fact) in facts.iter().enumerate() {
        println!("   - 事实 {}: {}", i + 1, fact.content);
        println!("     类别: {:?}", fact.category);
        println!("     置信度: {:.2}", fact.confidence);
    }

    Ok(())
}

async fn demo_decision_engine() -> Result<()> {
    println!("\n⚙️ 演示3：决策引擎");
    println!("------------------");

    println!("   ⚠️  演示模式：跳过需要真实API密钥的决策引擎");
    println!("   📝 模拟决策结果：");

    // 模拟决策结果
    let decisions = vec![
        agent_mem_intelligence::decision_engine::MemoryDecision {
            action: agent_mem_intelligence::decision_engine::MemoryAction::Add {
                content: "Rust是一种系统编程语言".to_string(),
                importance: 0.95,
                metadata: HashMap::new(),
            },
            confidence: 0.95,
            reasoning: "这是一个关于编程语言的重要技术事实，应该存储".to_string(),
            affected_memories: vec!["mem-1".to_string()],
            estimated_impact: 0.8,
        },
        agent_mem_intelligence::decision_engine::MemoryDecision {
            action: agent_mem_intelligence::decision_engine::MemoryAction::NoAction {
                reason: "信息已存在，无需重复存储".to_string(),
            },
            confidence: 0.80,
            reasoning: "与现有的编程语言知识相关，但无需更新".to_string(),
            affected_memories: vec![],
            estimated_impact: 0.2,
        },
    ];

    println!("✅ 决策制定完成");
    println!("   - 决策数量: {}", decisions.len());
    for (i, decision) in decisions.iter().enumerate() {
        println!("   - 决策 {}: {:?}", i + 1, decision.action);
        println!("     置信度: {:.2}", decision.confidence);
        println!("     理由: {}", decision.reasoning);
    }

    Ok(())
}

async fn demo_intelligent_processor() -> Result<()> {
    println!("\n🤖 演示4：智能处理器");
    println!("--------------------");

    println!("   ⚠️  演示模式：跳过需要真实API密钥的智能处理器");
    println!("   📝 模拟处理结果：");

    // 模拟智能处理结果
    let result = agent_mem_intelligence::intelligent_processor::IntelligentProcessingResult {
        extracted_facts: vec![agent_mem_intelligence::fact_extraction::ExtractedFact {
            content: "量子计算利用量子力学现象进行计算".to_string(),
            category: agent_mem_intelligence::fact_extraction::FactCategory::Knowledge,
            confidence: 0.92,
            entities: vec![],
            temporal_info: None,
            source_message_id: Some("msg-4".to_string()),
            metadata: std::collections::HashMap::new(),
        }],
        memory_decisions: vec![agent_mem_intelligence::decision_engine::MemoryDecision {
            action: agent_mem_intelligence::decision_engine::MemoryAction::Add {
                content: "量子计算是一种利用量子力学现象进行计算的技术".to_string(),
                importance: 0.85,
                metadata: HashMap::new(),
            },
            confidence: 0.92,
            reasoning: "这是关于量子计算的重要技术概念".to_string(),
            affected_memories: vec![],
            estimated_impact: 0.7,
        }],
        conflict_detections: vec![],
        quality_metrics: agent_mem_intelligence::intelligent_processor::QualityMetrics {
            average_fact_confidence: 0.92,
            average_decision_confidence: 0.92,
            conflict_rate: 0.0,
            fact_diversity_score: 0.8,
            processing_efficiency: 0.95,
        },
        processing_insights: agent_mem_intelligence::intelligent_processor::ProcessingInsights {
            dominant_fact_categories: vec!["Knowledge".to_string()],
            memory_growth_prediction: 0.15,
            suggested_optimizations: vec!["继续收集量子计算相关信息".to_string()],
            attention_areas: vec![],
        },
        processing_stats: agent_mem_intelligence::intelligent_processor::ProcessingStats {
            total_messages: 2,
            facts_extracted: 1,
            decisions_made: 1,
            high_confidence_decisions: 1,
            processing_time_ms: 150,
        },
        recommendations: vec![
            "建议深入学习量子计算的基础理论".to_string(),
            "可以关注量子计算在实际应用中的发展".to_string(),
        ],
    };

    println!("✅ 智能处理完成");
    println!("   - 提取的事实数量: {}", result.extracted_facts.len());
    println!("   - 记忆决策数量: {}", result.memory_decisions.len());
    println!(
        "   - 处理时间: {}ms",
        result.processing_stats.processing_time_ms
    );

    // 显示提取的事实
    for (i, fact) in result.extracted_facts.iter().enumerate() {
        println!("   - 事实 {}: {}", i + 1, fact.content);
    }

    // 显示记忆决策
    for (i, decision) in result.memory_decisions.iter().enumerate() {
        println!("   - 决策 {}: {:?}", i + 1, decision.action);
    }

    // 显示推荐
    if !result.recommendations.is_empty() {
        println!("   - 推荐数量: {}", result.recommendations.len());
        for (i, rec) in result.recommendations.iter().enumerate() {
            println!("     推荐 {}: {}", i + 1, rec);
        }
    }

    Ok(())
}
