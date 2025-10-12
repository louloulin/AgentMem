//! 高级推理功能演示
//!
//! 演示如何使用高级推理功能：
//! 1. 多跳因果推理
//! 2. 反事实推理
//! 3. 类比推理

use agent_mem_intelligence::reasoning::{AdvancedReasoner, MemoryData};
use chrono::{Duration, Utc};
use tracing::Level;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化 tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    println!("=== 高级推理功能演示 ===\n");

    // 演示 1: 多跳因果推理
    demo_multi_hop_causal_reasoning().await?;

    // 演示 2: 反事实推理
    demo_counterfactual_reasoning().await?;

    // 演示 3: 类比推理
    demo_analogical_reasoning().await?;

    println!("\n=== 演示完成 ===");
    Ok(())
}

/// 演示 1: 多跳因果推理
async fn demo_multi_hop_causal_reasoning() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- 演示 1: 多跳因果推理 ---");
    println!("场景: 学习编程的因果链\n");

    let reasoner = AdvancedReasoner::default();

    // 创建因果链: 开始学习 -> 完成课程 -> 做项目 -> 获得工作
    let memories = vec![
        create_memory(
            "mem1",
            "决定学习编程，开始学习 Python 基础",
            100,
            vec![1.0, 0.0, 0.0, 0.0],
        ),
        create_memory(
            "mem2",
            "完成了 Python 编程课程，掌握了基础语法",
            80,
            vec![0.9, 0.1, 0.0, 0.0],
        ),
        create_memory(
            "mem3",
            "开始做个人项目，构建了一个 Web 应用",
            60,
            vec![0.8, 0.2, 0.0, 0.0],
        ),
        create_memory(
            "mem4",
            "获得了软件工程师的工作机会",
            10,
            vec![0.7, 0.3, 0.0, 0.0],
        ),
    ];

    println!("✓ 创建了 {} 个记忆", memories.len());
    for (i, mem) in memories.iter().enumerate() {
        println!("  记忆 {}: {}", i + 1, mem.content);
    }

    // 执行多跳因果推理
    println!("\n执行多跳因果推理...");
    let results = reasoner.multi_hop_causal_reasoning(&memories[0], &memories[3], &memories)?;

    if results.is_empty() {
        println!("✗ 未找到因果链");
    } else {
        println!("\n✓ 找到 {} 条因果推理链:\n", results.len());

        for (i, result) in results.iter().enumerate() {
            println!("推理链 {}:", i + 1);
            println!("  深度: {} 步", result.depth);
            println!("  置信度: {:.2}", result.overall_confidence);
            println!("  推理步骤:");

            for (j, step) in result.reasoning_chain.iter().enumerate() {
                println!(
                    "    步骤 {}: {} → {}",
                    j + 1,
                    step.cause_id,
                    step.effect_id
                );
                println!("      类型: {:?}", step.relation_type);
                println!("      置信度: {:.2}", step.confidence);
                if !step.evidence.is_empty() {
                    println!("      证据: {}", step.evidence[0]);
                }
            }

            println!("\n  解释:\n{}", result.explanation);
            println!();
        }
    }

    Ok(())
}

/// 演示 2: 反事实推理
async fn demo_counterfactual_reasoning() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- 演示 2: 反事实推理 ---");
    println!("场景: 如果当初选择了不同的专业\n");

    let reasoner = AdvancedReasoner::default();

    // 创建记忆链
    let memories = vec![
        create_memory(
            "decision",
            "决定学习计算机科学专业",
            100,
            vec![1.0, 0.0, 0.0],
        ),
        create_memory(
            "skill1",
            "学会了编程和算法",
            80,
            vec![0.9, 0.1, 0.0],
        ),
        create_memory(
            "skill2",
            "掌握了软件工程技能",
            60,
            vec![0.8, 0.2, 0.0],
        ),
        create_memory(
            "career",
            "成为了一名软件工程师",
            10,
            vec![0.7, 0.3, 0.0],
        ),
    ];

    println!("✓ 创建了 {} 个记忆", memories.len());
    println!("  原始决定: {}", memories[0].content);
    println!("  最终结果: {}", memories[3].content);

    // 执行反事实推理
    println!("\n执行反事实推理...");
    let hypothesis = "决定学习医学专业";
    println!("假设: {}", hypothesis);

    let result = reasoner.counterfactual_reasoning(&memories[0], hypothesis, &memories)?;

    println!("\n✓ 反事实推理结果:");
    println!("  原始情况: {}", result.original_scenario);
    println!("  反事实假设: {}", result.counterfactual_hypothesis);
    println!("  预测结果: {}", result.predicted_outcome);
    println!("  置信度: {:.2}", result.confidence);
    println!("  影响的记忆数: {}", result.affected_memory_ids.len());

    if !result.affected_memory_ids.is_empty() {
        println!("  受影响的记忆:");
        for id in &result.affected_memory_ids {
            if let Some(mem) = memories.iter().find(|m| &m.id == id) {
                println!("    - {}: {}", id, mem.content);
            }
        }
    }

    println!("\n  详细解释:\n{}", result.explanation);
    println!();

    Ok(())
}

/// 演示 3: 类比推理
async fn demo_analogical_reasoning() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- 演示 3: 类比推理 ---");
    println!("场景: 学习编程 vs 学习音乐\n");

    let reasoner = AdvancedReasoner::default();

    // 源领域：学习编程
    let source_memories = vec![
        create_memory(
            "prog1",
            "学习编程基础语法和概念",
            100,
            vec![1.0, 0.0, 0.0],
        ),
        create_memory(
            "prog2",
            "通过练习编程题目提高技能",
            80,
            vec![0.9, 0.1, 0.0],
        ),
        create_memory(
            "prog3",
            "构建实际项目应用所学知识",
            60,
            vec![0.8, 0.2, 0.0],
        ),
        create_memory(
            "prog4",
            "成为熟练的程序员",
            40,
            vec![0.7, 0.3, 0.0],
        ),
    ];

    // 目标领域：学习音乐
    let target_memories = vec![
        create_memory(
            "music1",
            "学习音乐理论基础和乐理",
            50,
            vec![0.0, 1.0, 0.0],
        ),
        create_memory(
            "music2",
            "通过练习音阶和练习曲提高技能",
            40,
            vec![0.1, 0.9, 0.0],
        ),
        create_memory(
            "music3",
            "创作和演奏实际音乐作品",
            30,
            vec![0.2, 0.8, 0.0],
        ),
        create_memory(
            "music4",
            "成为熟练的音乐家",
            20,
            vec![0.3, 0.7, 0.0],
        ),
    ];

    println!("✓ 源领域（学习编程）包含 {} 个记忆:", source_memories.len());
    for mem in &source_memories {
        println!("  - {}", mem.content);
    }

    println!("\n✓ 目标领域（学习音乐）包含 {} 个记忆:", target_memories.len());
    for mem in &target_memories {
        println!("  - {}", mem.content);
    }

    // 执行类比推理
    println!("\n执行类比推理...");
    let result = reasoner.analogical_reasoning(&source_memories, &target_memories)?;

    println!("\n✓ 类比推理结果:");
    println!("  源领域: {}", result.source_domain.name);
    println!("  目标领域: {}", result.target_domain.name);
    println!("  类比强度: {:.2}", result.analogy_strength);
    println!("  建立的映射数: {}", result.mappings.len());

    if !result.mappings.is_empty() {
        println!("\n  类比映射:");
        for (i, mapping) in result.mappings.iter().take(5).enumerate() {
            println!("    映射 {}:", i + 1);
            println!("      源: {}", mapping.source_element);
            println!("      目标: {}", mapping.target_element);
            println!("      类型: {:?}", mapping.mapping_type);
            println!("      置信度: {:.2}", mapping.confidence);
        }

        if result.mappings.len() > 5 {
            println!("    ... 还有 {} 个映射", result.mappings.len() - 5);
        }
    }

    println!("\n  结论:\n{}", result.conclusion);
    println!();

    Ok(())
}

/// 创建测试记忆
fn create_memory(id: &str, content: &str, hours_ago: i64, embedding: Vec<f32>) -> MemoryData {
    MemoryData {
        id: id.to_string(),
        content: content.to_string(),
        created_at: Utc::now() - Duration::hours(hours_ago),
        embedding: Some(embedding),
    }
}

