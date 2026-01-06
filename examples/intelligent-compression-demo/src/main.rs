//! 智能记忆压缩引擎演示
//!
//! 展示基于学术研究的智能压缩算法，包括：
//! - 重要性驱动压缩
//! - 语义保持压缩  
//! - 时间感知压缩
//! - 自适应压缩策略

use agent_mem_core::compression::{
    CompressionConfig, CompressionContext, IntelligentCompressionEngine,
};
use agent_mem_traits::MemoryItem;
use std::collections::HashMap;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    println!("🧠 启动智能记忆压缩引擎演示");

    // 创建压缩引擎配置
    let config = CompressionConfig {
        enable_importance_compression: true,
        enable_semantic_compression: true,
        enable_temporal_compression: true,
        enable_adaptive_compression: true,
        min_importance_threshold: 0.3,
        target_compression_ratio: 0.6,
        semantic_similarity_threshold: 0.8,
        temporal_decay_factor: 0.95,
        adaptive_learning_rate: 0.1,
    };

    // 创建智能压缩引擎
    let compression_engine = IntelligentCompressionEngine::new(config);

    // 演示 1: 重要性驱动压缩
    println!("\n🎯 演示 1: 重要性驱动压缩");
    demo_importance_driven_compression(&compression_engine).await?;

    // 演示 2: 时间感知压缩
    println!("\n🎯 演示 2: 时间感知压缩");
    demo_temporal_aware_compression(&compression_engine).await?;

    // 演示 3: 语义保持压缩
    println!("\n🎯 演示 3: 语义保持压缩");
    demo_semantic_compression(&compression_engine).await?;

    // 演示 4: 自适应压缩策略
    println!("\n🎯 演示 4: 自适应压缩策略");
    demo_adaptive_compression(&compression_engine).await?;

    // 演示 5: 压缩统计和性能分析
    println!("\n🎯 演示 5: 压缩统计和性能分析");
    demo_compression_stats(&compression_engine).await?;

    println!("\n✅ 所有智能压缩演示完成！");

    println!("\n🎉 智能记忆压缩引擎特点:");
    println!("  - 🎯 重要性驱动压缩：基于访问频率和重要性评分");
    println!("  - 🧠 语义保持压缩：使用 PCA 和语义分析降维");
    println!("  - ⏰ 时间感知压缩：基于时间衰减的压缩率调整");
    println!("  - 🔄 自适应压缩：根据查询模式动态优化");
    println!("  - 📊 性能监控：实时压缩效果评估和策略调整");

    Ok(())
}

async fn demo_importance_driven_compression(
    engine: &IntelligentCompressionEngine,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Testing importance-driven compression");

    // 创建测试记忆数据
    let memories = create_test_memories_with_importance().await?;

    // 创建压缩上下文（模拟访问统计）
    let mut context = CompressionContext::new();
    context.update_access_stats("memory_1".to_string(), 100); // 高访问频率
    context.update_access_stats("memory_2".to_string(), 50); // 中等访问频率
    context.update_access_stats("memory_3".to_string(), 5); // 低访问频率
    context.update_access_stats("memory_4".to_string(), 1); // 极低访问频率

    println!("  📝 原始记忆数量: {}", memories.len());
    println!("  📊 访问统计:");
    for (id, count) in &context.access_stats {
        println!("    - {id}: {count} 次访问");
    }

    // 执行重要性驱动压缩
    let compressed = engine.compress_memories(&memories, &context).await?;

    println!("  🗜️ 压缩后记忆数量: {}", compressed.len());
    println!("  📈 压缩结果:");

    for (i, comp_memory) in compressed.iter().enumerate() {
        let original_len = memories
            .iter()
            .find(|m| m.id == comp_memory.original_id)
            .map(|m| m.content.len())
            .unwrap_or(0);

        println!("    {}. ID: {}", i + 1, comp_memory.original_id);
        println!("       重要性分数: {:.3}", comp_memory.importance_score);
        println!("       压缩比率: {:.3}", comp_memory.compression_ratio);
        println!(
            "       原始长度: {} → 压缩长度: {}",
            original_len,
            comp_memory.compressed_content.len()
        );
        println!(
            "       内容预览: {}...",
            comp_memory
                .compressed_content
                .chars()
                .take(50)
                .collect::<String>()
        );
        println!();
    }

    Ok(())
}

async fn demo_temporal_aware_compression(
    engine: &IntelligentCompressionEngine,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Testing temporal-aware compression");

    // 创建不同时间的测试记忆
    let memories = create_test_memories_with_timestamps().await?;
    let context = CompressionContext::new();

    println!("  📝 原始记忆（按时间排序）:");
    for (i, memory) in memories.iter().enumerate() {
        println!(
            "    {}. {} - 创建时间: {}",
            i + 1,
            memory.id,
            memory.created_at.format("%Y-%m-%d %H:%M")
        );
    }

    // 执行时间感知压缩
    let compressed = engine.compress_memories(&memories, &context).await?;

    println!("  🗜️ 时间感知压缩结果:");
    for (i, comp_memory) in compressed.iter().enumerate() {
        println!("    {}. ID: {}", i + 1, comp_memory.original_id);
        println!("       时间权重: {:.3}", comp_memory.importance_score);
        println!("       压缩比率: {:.3}", comp_memory.compression_ratio);
        println!("       内容: {}", comp_memory.compressed_content);
        println!();
    }

    Ok(())
}

async fn demo_semantic_compression(
    engine: &IntelligentCompressionEngine,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Testing semantic compression");

    // 创建语义相似的测试记忆
    let memories = create_semantically_similar_memories().await?;
    let context = CompressionContext::new();

    println!("  📝 原始记忆（语义相似）:");
    for (i, memory) in memories.iter().enumerate() {
        println!("    {}. {}: {}", i + 1, memory.id, memory.content);
    }

    // 执行语义压缩
    let compressed = engine.compress_memories(&memories, &context).await?;

    println!("  🗜️ 语义压缩结果:");
    println!(
        "    原始记忆数: {} → 压缩后: {}",
        memories.len(),
        compressed.len()
    );

    for (i, comp_memory) in compressed.iter().enumerate() {
        println!("    {}. ID: {}", i + 1, comp_memory.original_id);
        println!("       语义哈希: {}", comp_memory.semantic_hash);
        println!("       压缩内容: {}", comp_memory.compressed_content);
        println!();
    }

    Ok(())
}

async fn demo_adaptive_compression(
    engine: &IntelligentCompressionEngine,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Testing adaptive compression");

    let memories = create_test_memories_with_importance().await?;

    // 模拟不同的查询模式
    let scenarios = vec![
        ("高频查询场景", create_high_query_context()),
        ("大量记忆场景", create_large_memory_context()),
        ("平衡场景", create_balanced_context()),
    ];

    for (scenario_name, context) in scenarios {
        println!("  📊 场景: {scenario_name}");

        let compressed = engine.compress_memories(&memories, &context).await?;

        let total_original_size: usize = memories.iter().map(|m| m.content.len()).sum();
        let total_compressed_size: usize =
            compressed.iter().map(|m| m.compressed_content.len()).sum();
        let compression_ratio = total_compressed_size as f32 / total_original_size as f32;

        println!(
            "    - 压缩记忆数: {} → {}",
            memories.len(),
            compressed.len()
        );
        println!("    - 总体压缩比: {compression_ratio:.3}");
        println!(
            "    - 平均重要性: {:.3}",
            compressed.iter().map(|m| m.importance_score).sum::<f32>() / compressed.len() as f32
        );
        println!();
    }

    Ok(())
}

async fn demo_compression_stats(
    engine: &IntelligentCompressionEngine,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Analyzing compression statistics");

    // 获取压缩统计信息
    let stats = engine.get_compression_stats().await?;

    println!("  📊 压缩引擎统计信息:");
    println!("    - 总压缩次数: {}", stats.total_compressions);
    println!("    - 平均压缩比: {:.3}", stats.average_compression_ratio);
    println!(
        "    - 平均信息保留率: {:.3}",
        stats.average_information_retention
    );
    println!("    - 启用的策略: {:?}", stats.enabled_strategies);

    println!("  🎯 策略权重:");
    for (strategy, weight) in &stats.strategy_weights {
        println!("    - {strategy}: {weight:.3}");
    }

    Ok(())
}

// 辅助函数：创建测试数据

fn create_memory_item(
    id: String,
    content: String,
    importance: f32,
    access_count: u32,
    created_at: chrono::DateTime<chrono::Utc>,
) -> MemoryItem {
    use agent_mem_traits::Session;

    MemoryItem {
        id,
        content,
        hash: None,
        metadata: HashMap::new(),
        score: Some(importance),
        created_at,
        updated_at: Some(created_at),
        session: Session::new(),
        memory_type: agent_mem_traits::MemoryType::Episodic,
        entities: Vec::new(),
        relations: Vec::new(),
        agent_id: "test_agent".to_string(),
        user_id: Some("test_user".to_string()),
        importance,
        embedding: None,
        last_accessed_at: created_at,
        access_count,
        expires_at: None,
        version: 1,
    }
}

async fn create_test_memories_with_importance(
) -> Result<Vec<MemoryItem>, Box<dyn std::error::Error>> {
    let now = chrono::Utc::now();

    Ok(vec![
        create_memory_item(
            "memory_1".to_string(),
            "这是一个非常重要的会议记录，包含了关键的商业决策和战略规划。会议中讨论了公司未来三年的发展方向，包括新产品开发、市场扩张策略、人才招聘计划等重要议题。".to_string(),
            0.9,
            100,
            now,
        ),
        create_memory_item(
            "memory_2".to_string(),
            "今天的天气很好，阳光明媚。我去了公园散步，看到了很多人在锻炼。公园里的花开得很漂亮，春天真是个美好的季节。".to_string(),
            0.3,
            50,
            now,
        ),
        create_memory_item(
            "memory_3".to_string(),
            "学习了新的编程技术，包括 Rust 语言的高级特性。异步编程、所有权系统、生命周期管理等概念都很有趣。".to_string(),
            0.7,
            25,
            now,
        ),
        create_memory_item(
            "memory_4".to_string(),
            "买了一杯咖啡。".to_string(),
            0.1,
            1,
            now,
        ),
    ])
}

async fn create_test_memories_with_timestamps(
) -> Result<Vec<MemoryItem>, Box<dyn std::error::Error>> {
    let now = chrono::Utc::now();

    let one_week = chrono::Duration::weeks(1);
    let one_month = chrono::Duration::days(30);

    Ok(vec![
        create_memory_item(
            "recent_memory".to_string(),
            "今天刚刚发生的重要事件，需要完整保留所有细节信息。".to_string(),
            0.8,
            10,
            now,
        ),
        create_memory_item(
            "week_old_memory".to_string(),
            "一周前的会议记录，包含了一些重要的决策和讨论内容，但不如最新的信息重要。".to_string(),
            0.6,
            5,
            now - one_week,
        ),
        create_memory_item(
            "month_old_memory".to_string(),
            "一个月前的旧记录，信息可能已经过时，可以进行较大程度的压缩。".to_string(),
            0.4,
            2,
            now - one_month,
        ),
    ])
}

async fn create_semantically_similar_memories(
) -> Result<Vec<MemoryItem>, Box<dyn std::error::Error>> {
    let now = chrono::Utc::now();

    Ok(vec![
        create_memory_item(
            "weather_1".to_string(),
            "今天天气晴朗，阳光明媚，温度适宜。".to_string(),
            0.3,
            3,
            now,
        ),
        create_memory_item(
            "weather_2".to_string(),
            "今日天气很好，阳光充足，气温舒适。".to_string(),
            0.3,
            2,
            now,
        ),
        create_memory_item(
            "weather_3".to_string(),
            "天气不错，阳光灿烂，温度刚好。".to_string(),
            0.3,
            1,
            now,
        ),
    ])
}

fn create_high_query_context() -> CompressionContext {
    let mut context = CompressionContext::new();
    // 模拟高频查询场景
    for i in 0..15 {
        context.query_patterns.push(format!("query_{i}"));
    }
    context
}

fn create_large_memory_context() -> CompressionContext {
    let mut context = CompressionContext::new();
    // 模拟大量记忆场景
    for i in 0..1500 {
        context.update_access_stats(format!("memory_{i}"), 1);
    }
    context
}

fn create_balanced_context() -> CompressionContext {
    let mut context = CompressionContext::new();
    // 模拟平衡场景
    for i in 0..5 {
        context.query_patterns.push(format!("query_{i}"));
    }
    for i in 0..100 {
        context.update_access_stats(format!("memory_{i}"), i % 10 + 1);
    }
    context
}
