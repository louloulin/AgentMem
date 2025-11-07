//! 重要性评分和智能功能演示
//!
//! 演示 Phase 2 任务 2.2 完成的智能功能增强：
//! - 基于访问模式的频率评分
//! - 基于访问类型的重要性更新
//! - 多维度重要性计算

use agent_mem_core::intelligence::{
    AccessType, DefaultImportanceScorer, ImportanceScorer, IntelligenceConfig,
};
use agent_mem_traits::{MemoryItem, MemoryType, Session};
use chrono::{Duration, Utc};
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 AgentMem 重要性评分和智能功能演示");
    println!("========================================\n");

    // 创建智能评分器
    let config = IntelligenceConfig::default();
    let scorer = DefaultImportanceScorer::new(config);

    // 演示1：新创建的记忆
    demo_new_memory(&scorer).await?;

    // 演示2：频繁访问的记忆
    demo_frequently_accessed_memory(&scorer).await?;

    // 演示3：旧记忆的衰减
    demo_old_memory_decay(&scorer).await?;

    // 演示4：重要性更新
    demo_importance_updates(&scorer).await?;

    println!("\n✅ 所有演示完成！智能功能正常工作。");
    Ok(())
}

/// 演示1：新创建的记忆
async fn demo_new_memory(
    scorer: &DefaultImportanceScorer,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 演示1：新创建的记忆");
    println!("------------------------");

    let memory = create_test_memory(
        "新创建的记忆",
        0,          // 0 次访问
        Utc::now(), // 刚创建
        0.5,        // 基础重要性
    );

    let factors = scorer.calculate_importance(&memory).await?;

    println!("记忆内容: {}", memory.content);
    println!(
        "创建时间: {}",
        memory.created_at.format("%Y-%m-%d %H:%M:%S")
    );
    println!("访问次数: {}", memory.access_count);
    println!("\n重要性因子:");
    println!("  - 时效性评分: {:.3}", factors.recency_score);
    println!("  - 频率评分:   {:.3}", factors.frequency_score);
    println!("  - 相关性评分: {:.3}", factors.relevance_score);
    println!("  - 交互评分:   {:.3}", factors.interaction_score);
    println!("  - 综合评分:   {:.3}", factors.final_score);
    println!("\n💡 新记忆具有高时效性，但频率和交互评分较低\n");

    Ok(())
}

/// 演示2：频繁访问的记忆
async fn demo_frequently_accessed_memory(
    scorer: &DefaultImportanceScorer,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 演示2：频繁访问的记忆");
    println!("------------------------");

    let memory = create_test_memory(
        "频繁访问的重要记忆",
        100,                             // 100 次访问
        Utc::now() - Duration::days(30), // 30 天前创建
        0.8,                             // 高重要性
    );

    let factors = scorer.calculate_importance(&memory).await?;

    println!("记忆内容: {}", memory.content);
    println!(
        "创建时间: {}",
        memory.created_at.format("%Y-%m-%d %H:%M:%S")
    );
    println!("访问次数: {}", memory.access_count);
    println!("访问频率: {:.2} 次/天", memory.access_count as f64 / 30.0);
    println!("\n重要性因子:");
    println!("  - 时效性评分: {:.3}", factors.recency_score);
    println!("  - 频率评分:   {:.3}", factors.frequency_score);
    println!("  - 相关性评分: {:.3}", factors.relevance_score);
    println!("  - 交互评分:   {:.3}", factors.interaction_score);
    println!("  - 综合评分:   {:.3}", factors.final_score);
    println!("\n💡 频繁访问的记忆具有高频率评分和交互评分\n");

    Ok(())
}

/// 演示3：旧记忆的衰减
async fn demo_old_memory_decay(
    scorer: &DefaultImportanceScorer,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("⏰ 演示3：旧记忆的时效性衰减");
    println!("----------------------------");

    // 创建不同年龄的记忆
    let ages = vec![
        ("1 小时前", Duration::hours(1)),
        ("1 天前", Duration::days(1)),
        ("1 周前", Duration::weeks(1)),
        ("1 个月前", Duration::days(30)),
        ("3 个月前", Duration::days(90)),
        ("1 年前", Duration::days(365)),
    ];

    println!("记忆年龄 vs 时效性评分:");
    println!("年龄\t\t时效性\t频率\t综合");
    println!("----------------------------------------");

    for (label, age) in ages {
        let memory = create_test_memory(
            &format!("{}的记忆", label),
            10,               // 固定访问次数
            Utc::now() - age, // 不同创建时间
            0.5,              // 固定基础重要性
        );

        let factors = scorer.calculate_importance(&memory).await?;

        println!(
            "{}\t{:.3}\t{:.3}\t{:.3}",
            label, factors.recency_score, factors.frequency_score, factors.final_score
        );
    }

    println!("\n💡 记忆的时效性随时间呈指数衰减\n");

    Ok(())
}

/// 演示4：重要性更新
async fn demo_importance_updates(
    scorer: &DefaultImportanceScorer,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 演示4：基于访问类型的重要性更新");
    println!("----------------------------------");

    let access_types = vec![
        AccessType::Read,
        AccessType::Update,
        AccessType::Reference,
        AccessType::Decision,
    ];

    println!("访问类型\t\t重要性提升");
    println!("--------------------------------");

    for access_type in access_types {
        let boost = scorer
            .update_importance("test-memory-id", access_type.clone())
            .await?;

        println!("{:?}\t\t+{:.3}", access_type, boost);
    }

    println!("\n💡 不同访问类型对重要性的影响不同：");
    println!("   - Decision (决策) 影响最大 (+0.08)");
    println!("   - Update (更新) 影响中等 (+0.03)");
    println!("   - Reference (引用) 影响较小 (+0.02)");
    println!("   - Read (读取) 影响最小 (+0.01)\n");

    Ok(())
}

/// 创建测试记忆
fn create_test_memory(
    content: &str,
    access_count: u32,
    created_at: chrono::DateTime<Utc>,
    importance: f32,
) -> MemoryItem {
    let now = Utc::now();
    let last_accessed = if access_count > 0 {
        now - Duration::hours(1) // 最后访问在1小时前
    } else {
        created_at
    };

    MemoryItem {
        id: Uuid::new_v4().to_string(),
        content: content.to_string(),
        hash: None,
        metadata: HashMap::new(),
        score: Some(0.5),
        created_at,
        updated_at: Some(now),
        session: Session {
            id: Uuid::new_v4().to_string(),
            user_id: Some("demo-user".to_string()),
            agent_id: Some("demo-agent".to_string()),
            run_id: None,
            actor_id: None,
            created_at: now,
            metadata: HashMap::new(),
        },
        memory_type: MemoryType::Episodic,
        entities: Vec::new(),
        relations: Vec::new(),
        agent_id: "demo-agent".to_string(),
        user_id: Some("demo-user".to_string()),
        importance,
        embedding: None,
        last_accessed_at: last_accessed,
        access_count,
        expires_at: None,
        version: 1,
    }
}
