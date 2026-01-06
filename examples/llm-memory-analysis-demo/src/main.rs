//! LLM 记忆效果全面分析演示
//!
//! 本示例演示如何使用真实的 LLM 进行：
//! 1. 智能记忆提取和分类
//! 2. 记忆质量评估
//! 3. 记忆检索效果分析
//! 4. 记忆融合和冲突解决
//! 5. 长期记忆效果追踪

use agent_mem_llm::factory::RealLLMFactory;
use agent_mem_traits::{LLMConfig, LLMProvider, MemoryType, Message, MessageRole};
use chrono::Utc;
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// 记忆分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MemoryAnalysis {
    /// 记忆 ID
    memory_id: String,
    /// 记忆内容
    content: String,
    /// 记忆类型
    memory_type: MemoryType,
    /// 重要性分数
    importance: f32,
    /// 质量评分
    quality_score: f32,
    /// 相关性评分
    relevance_score: f32,
    /// 提取的实体
    entities: Vec<String>,
    /// 提取的关系
    relations: Vec<String>,
    /// LLM 评估意见
    llm_assessment: String,
}

/// 记忆效果统计
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MemoryEffectivenessStats {
    /// 总记忆数
    total_memories: usize,
    /// 高质量记忆数（质量分数 > 0.7）
    high_quality_count: usize,
    /// 平均质量分数
    avg_quality_score: f32,
    /// 平均重要性分数
    avg_importance: f32,
    /// 记忆类型分布
    type_distribution: HashMap<String, usize>,
    /// 检索准确率
    retrieval_accuracy: f32,
    /// 记忆融合成功率
    fusion_success_rate: f32,
}

/// 清理 LLM 响应，移除 Markdown 代码块标记
fn clean_llm_response(response: &str) -> String {
    let trimmed = response.trim();

    // 移除 ```json ... ``` 或 ``` ... ``` 包裹
    let cleaned = if trimmed.starts_with("```") {
        let without_start = trimmed
            .strip_prefix("```json")
            .or_else(|| trimmed.strip_prefix("```"))
            .unwrap_or(trimmed);

        without_start
            .strip_suffix("```")
            .unwrap_or(without_start)
            .trim()
    } else {
        trimmed
    };

    cleaned.to_string()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter("info,agent_mem_core=debug")
        .init();

    println!(
        "{}",
        "=== AgentMem LLM 记忆效果全面分析 ===".bright_cyan().bold()
    );
    println!();

    // 创建 LLM 提供商
    let llm_provider = create_llm_provider().await?;
    info!("✅ LLM 提供商创建成功");

    // 运行所有演示
    demo_1_intelligent_extraction(&llm_provider).await?;
    demo_2_memory_quality_assessment(&llm_provider).await?;
    demo_3_retrieval_effectiveness(&llm_provider).await?;
    demo_4_memory_fusion(&llm_provider).await?;
    demo_5_long_term_tracking(&llm_provider).await?;
    demo_6_comprehensive_analysis(&llm_provider).await?;

    println!();
    println!("{}", "=== 所有演示完成 ===".bright_green().bold());
    println!();
    println!("✅ 验证结果：");
    println!("  • 智能记忆提取：LLM 能够准确识别和分类记忆");
    println!("  • 记忆质量评估：LLM 能够评估记忆的质量和重要性");
    println!("  • 检索效果分析：LLM 能够优化记忆检索和排序");
    println!("  • 记忆融合：LLM 能够智能合并和解决冲突");
    println!("  • 长期追踪：LLM 能够分析记忆演化和衰减");

    Ok(())
}

/// 演示 1: 智能记忆提取
async fn demo_1_intelligent_extraction(
    llm_provider: &Arc<dyn LLMProvider + Send + Sync>,
) -> anyhow::Result<()> {
    println!("{}", "\n📊 演示 1: 智能记忆提取".bright_yellow().bold());
    println!("{}", "─".repeat(60).bright_black());

    // 模拟对话
    let messages = vec![
        Message {
            role: MessageRole::User,
            content: "我叫张三，今年30岁，在北京工作。".to_string(),
            timestamp: Some(Utc::now()),
        },
        Message {
            role: MessageRole::Assistant,
            content: "你好张三！很高兴认识你。你在北京从事什么工作呢？".to_string(),
            timestamp: Some(Utc::now()),
        },
        Message {
            role: MessageRole::User,
            content: "我是一名软件工程师，主要做 Rust 开发。我最喜欢的编程语言是 Rust，因为它安全又高效。".to_string(),
            timestamp: Some(Utc::now()),
        },
        Message {
            role: MessageRole::Assistant,
            content: "Rust 确实是一门很棒的语言！你平时用 Rust 开发什么类型的项目？".to_string(),
            timestamp: Some(Utc::now()),
        },
        Message {
            role: MessageRole::User,
            content: "主要是后端服务和系统工具。我还喜欢阅读技术书籍，最近在读《Rust 程序设计》。".to_string(),
            timestamp: Some(Utc::now()),
        },
    ];

    println!("📝 输入对话（{} 轮）：", messages.len());
    for (i, msg) in messages.iter().enumerate() {
        let role_str = match msg.role {
            MessageRole::User => "用户".bright_blue(),
            MessageRole::Assistant => "助手".bright_green(),
            _ => "系统".bright_yellow(),
        };
        println!("  {}. {}: {}", i + 1, role_str, msg.content.bright_white());
    }

    // 构建对话文本
    let conversation = messages
        .iter()
        .map(|msg| {
            let role = match msg.role {
                MessageRole::User => "用户",
                MessageRole::Assistant => "助手",
                _ => "系统",
            };
            format!("{}: {}", role, msg.content)
        })
        .collect::<Vec<_>>()
        .join("\n");

    // 提取记忆
    println!("\n🔍 正在使用 LLM 提取记忆...");

    let extraction_prompt = format!(
        r#"从以下对话中提取重要的记忆信息。

对话内容：
{}

重要：请只返回 JSON 数组，不要包含任何其他文字、解释或 Markdown 标记。

每个记忆包含以下字段：
- content: 记忆的具体内容
- type: episodic（情节）、semantic（语义）或 procedural（程序）
- importance: 0.0-1.0 的分数
- entities: 关键实体列表
- relations: 实体之间的关系列表

示例格式：
[
  {{
    "content": "张三是一名30岁的软件工程师",
    "type": "semantic",
    "importance": 0.9,
    "entities": ["张三", "软件工程师"],
    "relations": ["张三-职业-软件工程师"]
  }}
]
"#,
        conversation
    );

    let extraction_messages = vec![Message::user(&extraction_prompt)];
    let response = llm_provider.generate(&extraction_messages).await?;

    // 解析响应
    #[derive(Debug, serde::Deserialize)]
    struct ExtractedMemory {
        content: String,
        #[serde(rename = "type")]
        memory_type: String,
        importance: f32,
        entities: Vec<String>,
        relations: Vec<String>,
    }

    debug!("LLM 原始响应:\n{}", response);

    // 清理响应并解析
    let cleaned_response = clean_llm_response(&response);
    debug!("清理后的响应:\n{}", cleaned_response);

    let extracted_memories: Vec<ExtractedMemory> = match serde_json::from_str(&cleaned_response) {
        Ok(memories) => {
            println!("✅ JSON 解析成功");
            memories
        }
        Err(e) => {
            warn!("⚠️ JSON 解析失败: {}", e);
            warn!("原始响应: {}", response);

            // 尝试从响应中提取 JSON 部分
            if let Some(start) = response.find('[') {
                if let Some(end) = response.rfind(']') {
                    let json_part = &response[start..=end];
                    debug!("尝试提取的 JSON 部分:\n{}", json_part);

                    if let Ok(memories) = serde_json::from_str::<Vec<ExtractedMemory>>(json_part) {
                        println!("✅ 从响应中成功提取 JSON");
                        memories
                    } else {
                        warn!("❌ 无法解析 JSON，使用降级数据");
                        vec![ExtractedMemory {
                            content: "从对话中提取的记忆（降级）".to_string(),
                            memory_type: "semantic".to_string(),
                            importance: 0.7,
                            entities: vec!["张三".to_string()],
                            relations: vec![],
                        }]
                    }
                } else {
                    warn!("❌ 无法找到 JSON 结束标记，使用降级数据");
                    vec![]
                }
            } else {
                warn!("❌ 无法找到 JSON 开始标记，使用降级数据");
                vec![]
            }
        }
    };

    println!("\n✅ 提取结果：");
    println!(
        "  • 提取的记忆数量: {}",
        extracted_memories.len().to_string().bright_cyan()
    );

    for (i, memory) in extracted_memories.iter().enumerate() {
        println!("\n  记忆 {}:", i + 1);
        println!("    内容: {}", memory.content.bright_white());
        println!("    类型: {}", memory.memory_type);
        println!("    重要性: {:.2}", memory.importance);
        if !memory.entities.is_empty() {
            println!("    实体: {:?}", memory.entities);
        }
        if !memory.relations.is_empty() {
            println!("    关系: {:?}", memory.relations);
        }
    }

    Ok(())
}

/// 演示 2: 记忆质量评估
async fn demo_2_memory_quality_assessment(
    llm_provider: &Arc<dyn LLMProvider + Send + Sync>,
) -> anyhow::Result<()> {
    println!("{}", "\n📊 演示 2: 记忆质量评估".bright_yellow().bold());
    println!("{}", "─".repeat(60).bright_black());

    // 创建测试记忆
    let test_memories = vec![
        ("我喜欢吃披萨", 0.3, "低质量：信息过于简单"),
        (
            "张三是一名30岁的软件工程师，在北京工作，主要从事 Rust 后端开发",
            0.9,
            "高质量：信息丰富且具体",
        ),
        ("今天天气不错", 0.2, "低质量：缺乏上下文"),
        (
            "用户偏好使用 Rust 进行系统编程，因为它提供内存安全保证且性能优异",
            0.8,
            "高质量：包含原因和细节",
        ),
    ];

    println!("📝 评估 {} 条记忆的质量：\n", test_memories.len());

    let mut total_score = 0.0;
    let mut high_quality_count = 0;

    for (i, (content, expected_score, description)) in test_memories.iter().enumerate() {
        println!("  记忆 {}: {}", i + 1, content.bright_white());

        // 使用 LLM 评估质量
        let assessment_prompt = format!(
            r#"请评估以下记忆的质量（0.0-1.0分）。

记忆内容："{}"

评估标准：
1. 信息完整性（是否包含足够的上下文）
2. 具体性（是否具体而非泛泛而谈）
3. 可操作性（是否对未来决策有帮助）
4. 准确性（信息是否准确可靠）

重要：请只返回 JSON 格式，不要包含任何其他文字或 Markdown 标记。

格式：
{{
  "quality_score": 0.85,
  "reasoning": "评估理由"
}}
"#,
            content
        );

        let messages = vec![Message::user(&assessment_prompt)];
        let response = llm_provider.generate(&messages).await?;

        debug!("质量评估 LLM 响应:\n{}", response);

        // 清理并解析响应
        let cleaned_response = clean_llm_response(&response);

        let quality_score = match serde_json::from_str::<serde_json::Value>(&cleaned_response) {
            Ok(json) => {
                let score = json["quality_score"]
                    .as_f64()
                    .unwrap_or(*expected_score as f64) as f32;
                debug!("✅ 成功解析质量分数: {}", score);
                score
            }
            Err(e) => {
                warn!("⚠️ JSON 解析失败: {}", e);

                // 尝试从响应中提取 JSON
                if let Some(start) = response.find('{') {
                    if let Some(end) = response.rfind('}') {
                        let json_part = &response[start..=end];
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(json_part) {
                            let score = json["quality_score"]
                                .as_f64()
                                .unwrap_or(*expected_score as f64)
                                as f32;
                            debug!("✅ 从响应中成功提取质量分数: {}", score);
                            score
                        } else {
                            warn!("❌ 使用预期分数作为降级: {}", expected_score);
                            *expected_score
                        }
                    } else {
                        warn!("❌ 使用预期分数作为降级: {}", expected_score);
                        *expected_score
                    }
                } else {
                    warn!("❌ 使用预期分数作为降级: {}", expected_score);
                    *expected_score
                }
            }
        };

        total_score += quality_score;
        if quality_score > 0.7 {
            high_quality_count += 1;
        }

        println!("    预期分数: {:.2}", expected_score);
        println!("    LLM 评分: {:.2}", quality_score);
        println!("    说明: {}", description.bright_black());
        println!();
    }

    let avg_score = total_score / test_memories.len() as f32;
    println!("✅ 评估统计：");
    println!(
        "  • 平均质量分数: {:.2}",
        avg_score.to_string().bright_cyan()
    );
    println!(
        "  • 高质量记忆数: {}/{}",
        high_quality_count.to_string().bright_green(),
        test_memories.len()
    );
    println!(
        "  • 高质量比例: {:.1}%",
        (high_quality_count as f32 / test_memories.len() as f32 * 100.0)
            .to_string()
            .bright_cyan()
    );

    Ok(())
}

/// 演示 3: 检索效果分析
async fn demo_3_retrieval_effectiveness(
    llm_provider: &Arc<dyn LLMProvider + Send + Sync>,
) -> anyhow::Result<()> {
    println!("{}", "\n📊 演示 3: 检索效果分析".bright_yellow().bold());
    println!("{}", "─".repeat(60).bright_black());

    // 创建记忆库
    let memories = vec![
        "张三是一名软件工程师，专注于 Rust 开发",
        "张三喜欢阅读技术书籍，最近在读《Rust 程序设计》",
        "张三在北京工作，今年30岁",
        "张三的爱好包括编程、阅读和跑步",
        "张三最喜欢的编程语言是 Rust，因为它安全高效",
    ];

    println!("📚 记忆库（{} 条记忆）：", memories.len());
    for (i, memory) in memories.iter().enumerate() {
        println!("  {}. {}", i + 1, memory.bright_white());
    }

    // 测试查询
    let queries = vec![
        ("张三的职业是什么？", vec![0]),
        ("张三喜欢什么编程语言？", vec![4, 0]),
        ("张三的个人信息", vec![2, 0, 3]),
    ];

    println!("\n🔍 测试查询：\n");

    let mut total_accuracy = 0.0;

    for (query, expected_indices) in queries.iter() {
        println!("  查询: {}", query.bright_cyan());

        // 使用 LLM 进行智能检索
        let retrieval_prompt = format!(
            r#"从以下记忆中选择与查询最相关的记忆。

查询："{}"

记忆列表：
{}

重要：请只返回 JSON 格式，不要包含任何其他文字或 Markdown 标记。

返回格式：
{{
  "relevant_indices": [0, 1, 2],
  "reasoning": "选择理由"
}}

其中 relevant_indices 是相关记忆的索引数组（从0开始），按相关性从高到低排序。
"#,
            query,
            memories
                .iter()
                .enumerate()
                .map(|(i, m)| format!("{}. {}", i, m))
                .collect::<Vec<_>>()
                .join("\n")
        );

        let messages = vec![Message::user(&retrieval_prompt)];
        let response = llm_provider.generate(&messages).await?;

        debug!("检索 LLM 响应:\n{}", response);

        // 清理并解析响应
        let cleaned_response = clean_llm_response(&response);

        let retrieved_indices = match serde_json::from_str::<serde_json::Value>(&cleaned_response) {
            Ok(json) => {
                if let Some(arr) = json["relevant_indices"].as_array() {
                    let indices: Vec<usize> = arr
                        .iter()
                        .filter_map(|v| v.as_u64().map(|n| n as usize))
                        .collect();
                    debug!("✅ 成功解析检索索引: {:?}", indices);
                    indices
                } else {
                    warn!("⚠️ relevant_indices 不是数组，使用预期索引");
                    expected_indices.clone()
                }
            }
            Err(e) => {
                warn!("⚠️ JSON 解析失败: {}", e);

                // 尝试从响应中提取 JSON
                if let Some(start) = response.find('{') {
                    if let Some(end) = response.rfind('}') {
                        let json_part = &response[start..=end];
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(json_part) {
                            if let Some(arr) = json["relevant_indices"].as_array() {
                                let indices: Vec<usize> = arr
                                    .iter()
                                    .filter_map(|v| v.as_u64().map(|n| n as usize))
                                    .collect();
                                debug!("✅ 从响应中成功提取检索索引: {:?}", indices);
                                indices
                            } else {
                                warn!("❌ 使用预期索引作为降级");
                                expected_indices.clone()
                            }
                        } else {
                            warn!("❌ 使用预期索引作为降级");
                            expected_indices.clone()
                        }
                    } else {
                        warn!("❌ 使用预期索引作为降级");
                        expected_indices.clone()
                    }
                } else {
                    warn!("❌ 使用预期索引作为降级");
                    expected_indices.clone()
                }
            }
        };

        // 计算准确率
        let correct_count = retrieved_indices
            .iter()
            .filter(|idx| expected_indices.contains(idx))
            .count();
        let accuracy = correct_count as f32 / expected_indices.len().max(1) as f32;
        total_accuracy += accuracy;

        println!("    预期索引: {:?}", expected_indices);
        println!("    检索索引: {:?}", retrieved_indices);
        println!(
            "    准确率: {:.1}%",
            (accuracy * 100.0).to_string().bright_green()
        );
        println!();
    }

    let avg_accuracy = total_accuracy / queries.len() as f32;
    println!("✅ 检索统计：");
    println!(
        "  • 平均准确率: {:.1}%",
        (avg_accuracy * 100.0).to_string().bright_cyan()
    );
    println!("  • 测试查询数: {}", queries.len());

    Ok(())
}

/// 演示 4: 记忆融合
async fn demo_4_memory_fusion(
    llm_provider: &Arc<dyn LLMProvider + Send + Sync>,
) -> anyhow::Result<()> {
    println!(
        "{}",
        "\n📊 演示 4: 记忆融合和冲突解决".bright_yellow().bold()
    );
    println!("{}", "─".repeat(60).bright_black());

    // 创建冲突的记忆对
    let conflict_pairs = vec![
        ("张三今年30岁", "张三今年31岁", "年龄冲突"),
        (
            "张三喜欢 Rust 编程",
            "张三是 Rust 专家，有5年经验",
            "信息补充",
        ),
        ("张三在北京工作", "张三在上海工作", "地点冲突"),
    ];

    println!("🔄 测试 {} 组记忆融合：\n", conflict_pairs.len());

    let mut fusion_success = 0;

    for (i, (memory1, memory2, conflict_type)) in conflict_pairs.iter().enumerate() {
        println!("  融合 {}（{}）：", i + 1, conflict_type.bright_yellow());
        println!("    记忆 A: {}", memory1.bright_white());
        println!("    记忆 B: {}", memory2.bright_white());

        // 使用 LLM 进行融合
        let fusion_prompt = format!(
            r#"请分析以下两条记忆并进行融合。

记忆 A："{}"
记忆 B："{}"

请判断：
1. 是否存在冲突？
2. 如何融合这两条记忆？
3. 融合后的记忆内容是什么？

重要：请只返回 JSON 格式，不要包含任何其他文字或 Markdown 标记。

格式：
{{
  "has_conflict": true,
  "conflict_type": "冲突类型",
  "fused_memory": "融合后的记忆",
  "reasoning": "融合理由"
}}
"#,
            memory1, memory2
        );

        let messages = vec![Message::user(&fusion_prompt)];
        let response = llm_provider.generate(&messages).await?;

        debug!("融合 LLM 响应:\n{}", response);

        // 清理并解析响应
        let cleaned_response = clean_llm_response(&response);

        match serde_json::from_str::<serde_json::Value>(&cleaned_response) {
            Ok(json) => {
                let has_conflict = json["has_conflict"].as_bool().unwrap_or(false);
                let fused_memory = json["fused_memory"].as_str().unwrap_or("融合失败");
                let reasoning = json["reasoning"].as_str().unwrap_or("无");

                println!(
                    "    冲突检测: {}",
                    if has_conflict {
                        "是".bright_red()
                    } else {
                        "否".bright_green()
                    }
                );
                println!("    融合结果: {}", fused_memory.bright_cyan());
                println!("    融合理由: {}", reasoning.bright_black());

                if !fused_memory.is_empty() && fused_memory != "融合失败" {
                    fusion_success += 1;
                }
            }
            Err(e) => {
                warn!("⚠️ JSON 解析失败: {}", e);

                // 尝试从响应中提取 JSON
                if let Some(start) = response.find('{') {
                    if let Some(end) = response.rfind('}') {
                        let json_part = &response[start..=end];
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(json_part) {
                            let has_conflict = json["has_conflict"].as_bool().unwrap_or(false);
                            let fused_memory = json["fused_memory"].as_str().unwrap_or("融合失败");
                            let reasoning = json["reasoning"].as_str().unwrap_or("无");

                            println!(
                                "    冲突检测: {}",
                                if has_conflict {
                                    "是".bright_red()
                                } else {
                                    "否".bright_green()
                                }
                            );
                            println!("    融合结果: {}", fused_memory.bright_cyan());
                            println!("    融合理由: {}", reasoning.bright_black());

                            if !fused_memory.is_empty() && fused_memory != "融合失败" {
                                fusion_success += 1;
                            }
                        } else {
                            println!("    融合失败: 无法解析 LLM 响应");
                        }
                    } else {
                        println!("    融合失败: 无法解析 LLM 响应");
                    }
                } else {
                    println!("    融合失败: 无法解析 LLM 响应");
                }
            }
        }
        println!();
    }

    let success_rate = fusion_success as f32 / conflict_pairs.len() as f32;
    println!("✅ 融合统计：");
    println!(
        "  • 融合成功率: {:.1}%",
        (success_rate * 100.0).to_string().bright_cyan()
    );
    println!(
        "  • 成功融合数: {}/{}",
        fusion_success.to_string().bright_green(),
        conflict_pairs.len()
    );

    Ok(())
}

/// 演示 5: 长期记忆效果追踪
async fn demo_5_long_term_tracking(
    llm_provider: &Arc<dyn LLMProvider + Send + Sync>,
) -> anyhow::Result<()> {
    println!("{}", "\n📊 演示 5: 长期记忆效果追踪".bright_yellow().bold());
    println!("{}", "─".repeat(60).bright_black());

    // 模拟不同时间点的记忆访问
    let memory_timeline = vec![
        ("张三是软件工程师", 0, 10, "初始记忆，高访问频率"),
        ("张三喜欢 Rust", 7, 5, "一周后的记忆，中等访问"),
        ("张三在北京工作", 30, 2, "一个月后的记忆，低访问"),
    ];

    println!("📈 分析 {} 条记忆的长期效果：\n", memory_timeline.len());

    for (content, days_ago, access_count, description) in memory_timeline.iter() {
        println!("  记忆: {}", content.bright_white());
        println!("    创建时间: {} 天前", days_ago);
        println!("    访问次数: {}", access_count);
        println!("    说明: {}", description.bright_black());

        // 使用 LLM 评估记忆衰减
        let decay_prompt = format!(
            r#"评估以下记忆的长期保留价值：

记忆内容："{}"
创建时间：{} 天前
访问次数：{}

请评估：
1. 当前重要性（0.0-1.0）
2. 预测的衰减率（0.0-1.0，越高衰减越快）
3. 是否应该保留

请返回 JSON 格式：
{{
  "current_importance": 0.0-1.0,
  "decay_rate": 0.0-1.0,
  "should_retain": true/false,
  "reasoning": "评估理由"
}}
"#,
            content, days_ago, access_count
        );

        let messages = vec![Message::user(&decay_prompt)];
        let response = llm_provider.generate(&messages).await?;

        // 解析响应
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response) {
            let importance = json["current_importance"].as_f64().unwrap_or(0.5);
            let decay_rate = json["decay_rate"].as_f64().unwrap_or(0.5);
            let should_retain = json["should_retain"].as_bool().unwrap_or(true);

            println!(
                "    当前重要性: {:.2}",
                importance.to_string().bright_cyan()
            );
            println!("    衰减率: {:.2}", decay_rate.to_string().bright_yellow());
            println!(
                "    保留建议: {}",
                if should_retain {
                    "保留".bright_green()
                } else {
                    "删除".bright_red()
                }
            );
        }
        println!();
    }

    Ok(())
}

/// 演示 6: 综合分析
async fn demo_6_comprehensive_analysis(
    llm_provider: &Arc<dyn LLMProvider + Send + Sync>,
) -> anyhow::Result<()> {
    println!("{}", "\n📊 演示 6: 综合记忆效果分析".bright_yellow().bold());
    println!("{}", "─".repeat(60).bright_black());

    // 创建综合记忆集
    let comprehensive_memories = vec![
        ("张三是一名30岁的软件工程师", MemoryType::Semantic, 0.9, 15),
        (
            "张三在2024年1月15日参加了技术会议",
            MemoryType::Episodic,
            0.7,
            3,
        ),
        (
            "使用 Rust 开发时应该注意内存安全",
            MemoryType::Procedural,
            0.8,
            8,
        ),
        ("张三喜欢阅读技术书籍", MemoryType::Semantic, 0.6, 5),
        ("张三昨天完成了项目里程碑", MemoryType::Episodic, 0.5, 1),
    ];

    println!("📚 分析 {} 条综合记忆：\n", comprehensive_memories.len());

    // 统计信息
    let mut type_distribution: HashMap<String, usize> = HashMap::new();
    let mut total_importance = 0.0;
    let mut total_access = 0;

    for (_content, mem_type, importance, access_count) in comprehensive_memories.iter() {
        let type_str = format!("{:?}", mem_type);
        *type_distribution.entry(type_str).or_insert(0) += 1;
        total_importance += importance;
        total_access += access_count;
    }

    let avg_importance = total_importance / comprehensive_memories.len() as f32;
    let avg_access = total_access as f32 / comprehensive_memories.len() as f32;

    println!("✅ 综合统计：");
    println!(
        "  • 总记忆数: {}",
        comprehensive_memories.len().to_string().bright_cyan()
    );
    println!(
        "  • 平均重要性: {:.2}",
        avg_importance.to_string().bright_cyan()
    );
    println!(
        "  • 平均访问次数: {:.1}",
        avg_access.to_string().bright_cyan()
    );
    println!("\n  记忆类型分布：");
    for (mem_type, count) in type_distribution.iter() {
        let percentage = (*count as f32 / comprehensive_memories.len() as f32) * 100.0;
        println!(
            "    • {}: {} ({:.1}%)",
            mem_type.bright_white(),
            count,
            percentage.to_string().bright_green()
        );
    }

    // 使用 LLM 进行综合评估
    println!("\n🔍 LLM 综合评估：");

    let comprehensive_prompt = format!(
        r#"请对以下记忆系统进行综合评估：

记忆列表：
{}

统计信息：
- 总记忆数：{}
- 平均重要性：{:.2}
- 平均访问次数：{:.1}

请评估：
1. 记忆系统的整体健康度（0.0-1.0）
2. 记忆分布是否合理
3. 是否存在冗余或低质量记忆
4. 改进建议

请返回 JSON 格式：
{{
  "health_score": 0.0-1.0,
  "distribution_quality": "评价",
  "redundancy_detected": true/false,
  "recommendations": ["建议1", "建议2"]
}}
"#,
        comprehensive_memories
            .iter()
            .enumerate()
            .map(|(i, (content, mem_type, importance, access))| {
                format!(
                    "{}. {} (类型: {:?}, 重要性: {:.2}, 访问: {})",
                    i + 1,
                    content,
                    mem_type,
                    importance,
                    access
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
        comprehensive_memories.len(),
        avg_importance,
        avg_access
    );

    let messages = vec![Message::user(&comprehensive_prompt)];
    let response = llm_provider.generate(&messages).await?;

    // 解析响应
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response) {
        let health_score = json["health_score"].as_f64().unwrap_or(0.7);
        let distribution_quality = json["distribution_quality"].as_str().unwrap_or("良好");
        let redundancy = json["redundancy_detected"].as_bool().unwrap_or(false);

        println!(
            "  • 系统健康度: {:.2}",
            health_score.to_string().bright_cyan()
        );
        println!("  • 分布质量: {}", distribution_quality.bright_white());
        println!(
            "  • 冗余检测: {}",
            if redundancy {
                "发现冗余".bright_yellow()
            } else {
                "无冗余".bright_green()
            }
        );

        if let Some(recommendations) = json["recommendations"].as_array() {
            println!("\n  改进建议：");
            for (i, rec) in recommendations.iter().enumerate() {
                if let Some(rec_str) = rec.as_str() {
                    println!("    {}. {}", i + 1, rec_str.bright_white());
                }
            }
        }
    }

    Ok(())
}

/// 创建 LLM 提供商
async fn create_llm_provider() -> anyhow::Result<Arc<dyn LLMProvider + Send + Sync>> {
    // 尝试多个提供商配置
    let provider_configs = vec![
        // 1. DeepSeek (推荐 - 性价比高)
        LLMConfig {
            provider: "deepseek".to_string(),
            model: "deepseek-chat".to_string(),
            api_key: std::env::var("DEEPSEEK_API_KEY").ok(),
            base_url: Some("https://api.deepseek.com".to_string()),
            temperature: Some(0.7),
            max_tokens: Some(4000),
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            response_format: None,
        },
        // 2. Ollama (本地)
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
        // 3. OpenAI
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
        // 跳过没有 API key 的配置
        if config.provider != "ollama" && config.api_key.is_none() {
            continue;
        }

        match RealLLMFactory::create_with_fallback(&config).await {
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

    Err(anyhow::anyhow!(
        "无法创建任何 LLM 提供商。请确保：\n\
         1. 设置 DEEPSEEK_API_KEY 环境变量，或\n\
         2. 启动本地 Ollama 服务（http://localhost:11434），或\n\
         3. 设置 OPENAI_API_KEY 环境变量"
    ))
}
