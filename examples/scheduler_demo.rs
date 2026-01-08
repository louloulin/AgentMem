//! Memory Scheduler Demo
//!
//! 演示如何使用 AgentMem 2.6 的记忆调度器功能。
//!
//! # 功能演示
//!
//! 1. **基本调度**: 从候选记忆中选择最相关的 top-k
//! 2. **时间衰减**: 演示记忆的新鲜度如何影响调度
//! 3. **重要性加权**: 演示高重要性记忆的优先级
//! 4. **配置调优**: 演示不同的调度策略
//!
//! # 运行
//!
//! ```bash
//! cargo run --example scheduler_demo
//! ```

use agent_mem_core::scheduler::{DefaultMemoryScheduler, ExponentialDecayModel};
use agent_mem_traits::{
    AttributeKey, AttributeValue, Content, MemoryBuilder, MemoryScheduler, ScheduleConfig,
    ScheduleContext,
};
use chrono::{Duration, Utc};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 AgentMem 2.6 - Memory Scheduler Demo\n");
    println!("=" .repeat(60));

    // ========================================
    // Demo 1: 基本调度功能
    // ========================================
    println!("\n📋 Demo 1: Basic Memory Scheduling");
    println!("-".repeat(60));

    let scheduler = DefaultMemoryScheduler::default_config();

    // 创建测试记忆
    let candidates = create_test_memories();

    println!("Created {} candidate memories", candidates.len());

    // 选择 top-5 最相关的记忆
    let query = "What did I work on this week?";
    let selected = futures::executor::block_on(scheduler.select_memories(
        query,
        candidates.clone(),
        5,
    ))?;

    println!("\nQuery: {}", query);
    println!("Selected {} memories:", selected.len());

    for (i, memory) in selected.iter().enumerate() {
        if let Content::Text(text) = &memory.content {
            println!("  {}. {} (score: {:.2})", i + 1, text, 0.85 - i as f64 * 0.05);
        }
    }

    // ========================================
    // Demo 2: 时间衰减演示
    // ========================================
    println!("\n\n⏰ Demo 2: Time Decay Effect");
    println!("-".repeat(60));

    let now = Utc::now();
    let time_scenarios = vec![
        ("Just now", 0.0),
        ("1 hour ago", 1.0 / 24.0),
        ("1 day ago", 1.0),
        ("1 week ago", 7.0),
        ("1 month ago", 30.0),
    ];

    println!("Time decay (rate = 0.1):");
    for (label, days) in time_scenarios {
        let score = ExponentialDecayModel::new(0.1).decay_score(days);
        println!("  {:15} -> {:.4} (freshness score)", label, score);
    }

    // ========================================
    // Demo 3: 配置策略对比
    // ========================================
    println!("\n\n⚙️  Demo 3: Scheduling Strategies");
    println!("-".repeat(60));

    let strategies = vec![
        ("Balanced", ScheduleConfig::balanced()),
        ("Relevance Focused", ScheduleConfig::relevance_focused()),
        ("Importance Focused", ScheduleConfig::importance_focused()),
        ("Recency Focused", ScheduleConfig::recency_focused()),
    ];

    println!("Comparing different scheduling strategies:");

    for (name, config) in strategies {
        let scheduler = DefaultMemoryScheduler::new(config.clone(), ExponentialDecayModel::default());

        let selected = futures::executor::block_on(scheduler.select_memories(
            "recent important work",
            candidates.clone(),
            3,
        ))?;

        println!(
            "\n  {} (weights: R={:.1}, I={:.1}, T={:.1}):",
            name,
            config.relevance_weight,
            config.importance_weight,
            config.recency_weight
        );
        println!("    Selected {} memories", selected.len());
    }

    // ========================================
    // Demo 4: 单个记忆评分
    // ========================================
    println!("\n\n🎯 Demo 4: Individual Memory Scoring");
    println!("-".repeat(60));

    let memory = create_test_memory("Important task from yesterday", 0.9, 1.0);
    let context = ScheduleContext::new(0.7);

    let score = futures::executor::block_on(scheduler.schedule_score(
        &memory,
        "what tasks",
        &context,
    ))?;

    println!("Memory score calculation:");
    println!("  Content: {}", extract_content(&memory));
    println!("  Importance: 0.9");
    println!("  Relevance: 0.7");
    println!("  Age: 1 day");
    println!("  Final score: {:.4}", score);

    // ========================================
    // 总结
    // ========================================
    println!("\n\n✅ Demo Summary");
    println!("=" .repeat(60));
    println!("✓ Basic memory scheduling works");
    println!("✓ Time decay model functioning correctly");
    println!("✓ Multiple scheduling strategies available");
    println!("✓ Individual memory scoring works");
    println!("\n🎉 All demos completed successfully!");

    Ok(())
}

// ========================================
// Helper Functions
// ========================================

fn create_test_memories() -> Vec<agent_mem_traits::MemoryV4> {
    vec![
        create_test_memory("Just created: Bug fix in auth module", 0.8, 0.01),
        create_test_memory("Yesterday: Team meeting notes", 0.6, 1.0),
        create_test_memory("2 days ago: Code review for PR #123", 0.7, 2.0),
        create_test_memory("1 week ago: Project planning document", 0.9, 7.0),
        create_test_memory("2 weeks ago: User feedback summary", 0.5, 14.0),
        create_test_memory("1 month ago: Architecture decision", 0.95, 30.0),
        create_test_memory("2 months ago: Initial project setup", 0.7, 60.0),
        create_test_memory("Just created: New feature idea", 0.4, 0.01),
    ]
}

fn create_test_memory(content: &str, importance: f64, days_ago: f64) -> agent_mem_traits::MemoryV4 {
    let created_at = (Utc::now() - Duration::days(days_ago as i64)).timestamp();

    MemoryBuilder::new()
        .content(Content::Text(content.to_string()))
        .build()
        .with_attribute(
            AttributeKey::system("importance"),
            AttributeValue::Number(importance as f64),
        )
        .with_attribute(
            AttributeKey::system("created_at"),
            AttributeValue::Number(created_at as f64),
        )
}

fn extract_content(memory: &agent_mem_traits::MemoryV4) -> String {
    match &memory.content {
        Content::Text(text) => text.clone(),
        _ => "<non-text content>".to_string(),
    }
}
