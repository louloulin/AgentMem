//! 智能记忆管理演示
//!
//! 展示 AgentMem 的智能功能集成：
//! - 基于 Memory 统一 API 的智能功能
//! - 自动事实提取和记忆去重
//! - 智能决策和冲突检测
//! - 多种 LLM 提供商支持

use agent_mem::Memory;
use anyhow::Result;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 启动智能记忆管理演示");

    // 演示场景
    demo_basic_operations().await?;
    demo_intelligent_operations().await?;
    demo_search_and_retrieval().await?;

    info!("✅ 智能记忆管理演示完成！");
    Ok(())
}

/// 演示 1: 基础记忆操作
async fn demo_basic_operations() -> Result<()> {
    info!("\n📊 === 演示 1: 基础记忆操作 ===");

    // 创建 Memory 实例 (零配置模式)
    let memory = Memory::new().await?;
    info!("✅ Memory 实例创建成功");

    // 添加记忆
    let memory_id_1 = memory.add("我喜欢吃披萨").await?;
    info!("✅ 添加记忆 1: {}", memory_id_1);

    let memory_id_2 = memory.add("今天天气很好").await?;
    info!("✅ 添加记忆 2: {}", memory_id_2);

    let memory_id_3 = memory.add("我正在学习 Rust 编程").await?;
    info!("✅ 添加记忆 3: {}", memory_id_3);

    // 搜索记忆
    info!("\n搜索记忆: '披萨'");
    let results = memory.search("披萨", None, None, None).await?;
    info!("找到 {} 条相关记忆", results.len());
    for result in results {
        info!("  - {}", result.memory.content);
    }

    // 获取所有记忆
    let all_memories = memory.get_all(None, None, None, None).await?;
    info!("\n当前共有 {} 条记忆", all_memories.memories.len());

    Ok(())
}

/// 演示 2: 智能记忆操作
async fn demo_intelligent_operations() -> Result<()> {
    info!("\n🧠 === 演示 2: 智能记忆操作 ===");

    // 创建支持智能功能的 Memory 实例
    let memory = Memory::builder()
        .with_llm_from_env()  // 从环境变量读取 LLM 配置
        .build()
        .await?;

    info!("✅ 启用智能功能的 Memory 创建成功");

    // 添加包含多个事实的复杂内容
    let complex_content = "我叫张三，今年30岁，在北京工作。我喜欢编程和阅读，最喜欢的编程语言是 Rust。";
    
    info!("添加复杂记忆: {}", complex_content);
    
    match memory.add(complex_content).await {
        Ok(memory_id) => {
            info!("✅ 记忆添加成功: {}", memory_id);
            
            // 获取该记忆
            if let Ok(results) = memory.get_all(None, None, None, None).await {
                if let Some(mem) = results.memories.last() {
                    info!("记忆内容: {}", mem.content);
                    info!("重要性: {}", mem.importance);
                }
            }
        }
        Err(e) => {
            warn!("⚠️ 记忆添加失败 (可能因为没有配置 LLM): {}", e);
            info!("提示: 设置环境变量 OPENAI_API_KEY 或 DEEPSEEK_API_KEY 以启用智能功能");
        }
    }

    // 添加相关记忆
    info!("\n添加相关记忆...");
    let _ = memory.add("我最喜欢的食物是意大利披萨").await;
    let _ = memory.add("Rust 是一门很棒的编程语言").await;

    // 搜索相关记忆
    info!("\n搜索 'Rust'");
    match memory.search("Rust", None, None, None).await {
        Ok(results) => {
            info!("找到 {} 条相关记忆", results.len());
            for result in results.iter().take(3) {
                info!("  - {} (相似度: {:.2})", result.memory.content, result.score);
            }
        }
        Err(e) => {
            warn!("⚠️ 搜索失败: {}", e);
        }
    }

    Ok(())
}

/// 演示 3: 搜索和检索
async fn demo_search_and_retrieval() -> Result<()> {
    info!("\n🔍 === 演示 3: 搜索和检索 ===");

    let memory = Memory::new().await?;

    // 添加多个记忆
    let memories = vec![
        "我昨天去了公园",
        "Python 是一门流行的编程语言",
        "我喜欢在周末看电影",
        "Rust 提供了内存安全保证",
        "今天的午餐很美味",
    ];

    info!("添加 {} 条测试记忆", memories.len());
    for content in memories {
        memory.add(content).await?;
    }

    // 多次搜索测试
    let search_queries = vec![
        "编程语言",
        "周末活动",
        "食物",
    ];

    for query in search_queries {
        info!("\n搜索: '{}'", query);
        match memory.search(query, None, Some(3), None).await {
            Ok(results) => {
                if results.is_empty() {
                    info!("  未找到相关记忆");
                } else {
                    for result in results {
                        info!("  - {}", result.memory.content);
                    }
                }
            }
            Err(e) => {
                warn!("  搜索失败: {}", e);
            }
        }
    }

    // 获取统计信息
    info!("\n记忆统计:");
    let all_memories = memory.get_all(None, None, None, None).await?;
    info!("  总计: {} 条记忆", all_memories.memories.len());

    Ok(())
}
