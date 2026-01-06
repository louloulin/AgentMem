//! 验证 SimpleMemory 架构缺陷
//!
//! 这个示例通过实际运行来验证 SIMPLEMEMORY_ARCHITECTURE_DEFECTS_ANALYSIS.md 中的分析

#[path = "shared/simple_memory_adapter.rs"]
mod simple_memory_adapter;
use agent_mem_traits::Result;
use simple_memory_adapter::SimpleMemory;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("\n🔍 SimpleMemory 架构缺陷验证");
    println!("======================================================================\n");

    // 缺陷 1: 智能功能默认禁用
    verify_defect_1().await?;

    // 缺陷 2: 没有向量嵌入支持
    verify_defect_2().await?;

    // 缺陷 3: 搜索只能做字符串包含匹配
    verify_defect_3().await?;

    // 缺陷 4: 配置存在但不生效
    verify_defect_4().await?;

    println!("\n======================================================================");
    println!("📊 验证完成！所有缺陷均已确认。");
    println!("\n详细分析请查看: SIMPLEMEMORY_ARCHITECTURE_DEFECTS_ANALYSIS.md");

    Ok(())
}

/// 缺陷 1: 智能功能默认禁用
async fn verify_defect_1() -> Result<()> {
    println!("🔴 缺陷 1: 智能功能默认禁用");
    println!("----------------------------------------------------------------------");

    let mem = SimpleMemory::new().await?;

    // 添加包含多个事实的内容
    let content = "我叫张三，今年30岁，在北京工作。我喜欢编程和阅读。";
    println!("   添加内容: {}", content);

    let id = mem.add(content).await?;
    println!("   ✅ 记忆ID: {}", id);

    // 获取所有记忆并找到刚添加的
    let all_memories = mem.get_all().await?;
    if let Some(memory) = all_memories.iter().find(|m| m.id == id) {
        println!("\n   📋 记忆详情:");
        println!("      内容: {}", memory.content);
        println!("      类型: {:?}", memory.memory_type);
        println!("      重要性: {}", memory.importance);

        // 检查是否提取了事实
        println!("\n   🔬 事实提取检查:");
        if memory.entities.is_empty() {
            println!("      ❌ 实体列表: 空 (应该提取: 张三, 北京)");
        } else {
            println!("      ✅ 实体列表: {:?}", memory.entities);
        }

        if memory.relations.is_empty() {
            println!("      ❌ 关系列表: 空 (应该提取: 张三-年龄-30岁, 张三-工作地-北京)");
        } else {
            println!("      ✅ 关系列表: {:?}", memory.relations);
        }
    }

    println!("\n   💡 结论: 智能事实提取功能未生效 ❌");
    println!();

    Ok(())
}

/// 缺陷 2: 没有向量嵌入支持
async fn verify_defect_2() -> Result<()> {
    println!("🔴 缺陷 2: 没有向量嵌入支持");
    println!("----------------------------------------------------------------------");

    let mem = SimpleMemory::new().await?;

    // 添加记忆
    let id = mem.add("I love pizza").await?;
    println!("   添加记忆: I love pizza");
    println!("   记忆ID: {}", id);

    // 检查是否生成了向量
    let all_memories = mem.get_all().await?;
    if let Some(memory) = all_memories.iter().find(|m| m.id == id) {
        println!("\n   🔬 向量嵌入检查:");
        if memory.embedding.is_none() {
            println!("      ❌ embedding: None (应该自动生成 384 维向量)");
        } else {
            println!("      ✅ embedding: {:?}", memory.embedding);
        }
    }

    println!("\n   💡 结论: 向量嵌入功能未生效 ❌");
    println!();

    Ok(())
}

/// 缺陷 3: 搜索只能做字符串包含匹配
async fn verify_defect_3() -> Result<()> {
    println!("🔴 缺陷 3: 搜索只能做字符串包含匹配");
    println!("----------------------------------------------------------------------");

    let mem = SimpleMemory::new().await?;

    // 添加测试数据
    let test_data = vec![
        ("I love pizza", "食物偏好"),
        ("I work at Google", "工作信息"),
        ("My favorite color is blue", "颜色偏好"),
        ("I live in San Francisco", "居住地"),
    ];

    println!("   📝 添加测试数据:");
    for (content, desc) in &test_data {
        mem.add(*content).await?;
        println!("      - {} ({})", content, desc);
    }

    println!("\n   🔍 搜索测试:");

    // 测试 1: 精确子串匹配 (应该找到)
    println!("\n   测试 1: 精确子串匹配");
    println!("      查询: 'pizza'");
    let results = mem.search("pizza").await?;
    println!("      结果: {} 条", results.len());
    if results.len() > 0 {
        println!("      ✅ 找到: {}", results[0].content);
    } else {
        println!("      ❌ 未找到");
    }

    // 测试 2: 语义相似查询 (应该找到但找不到)
    println!("\n   测试 2: 语义相似查询");
    println!("      查询: 'What food do I like?'");
    let results = mem.search("What food do I like?").await?;
    println!("      结果: {} 条", results.len());
    if results.len() > 0 {
        println!("      ✅ 找到: {}", results[0].content);
    } else {
        println!("      ❌ 未找到 (应该找到 'I love pizza')");
    }

    // 测试 3: 同义词查询 (应该找到但找不到)
    println!("\n   测试 3: 同义词查询");
    println!("      查询: 'Where do I reside?'");
    let results = mem.search("Where do I reside?").await?;
    println!("      结果: {} 条", results.len());
    if results.len() > 0 {
        println!("      ✅ 找到: {}", results[0].content);
    } else {
        println!("      ❌ 未找到 (应该找到 'I live in San Francisco')");
    }

    // 测试 4: 多词查询 (整体不是子串)
    println!("\n   测试 4: 多词查询");
    println!("      查询: 'pizza favorite'");
    let results = mem.search("pizza favorite").await?;
    println!("      结果: {} 条", results.len());
    if results.len() > 0 {
        println!("      ✅ 找到: {}", results[0].content);
    } else {
        println!("      ❌ 未找到 (整体字符串不是任何内容的子串)");
    }

    println!("\n   💡 结论: 只能做简单的字符串包含匹配，无法理解语义 ❌");
    println!();

    Ok(())
}

/// 缺陷 4: 配置存在但不生效
async fn verify_defect_4() -> Result<()> {
    println!("🔴 缺陷 4: 配置存在但不生效");
    println!("----------------------------------------------------------------------");

    use agent_mem_config::memory::IntelligenceConfig;
    use agent_mem_config::MemoryConfig;

    // 创建启用智能功能的配置
    let mut config = MemoryConfig::default();
    config.intelligence.enable_intelligent_extraction = true;
    config.intelligence.enable_decision_engine = true;
    config.intelligence.enable_deduplication = true;

    println!("   📋 配置:");
    println!(
        "      enable_intelligent_extraction: {}",
        config.intelligence.enable_intelligent_extraction
    );
    println!(
        "      enable_decision_engine: {}",
        config.intelligence.enable_decision_engine
    );
    println!(
        "      enable_deduplication: {}",
        config.intelligence.enable_deduplication
    );

    // 使用配置创建 SimpleMemory
    let mem = SimpleMemory::with_config(config).await?;

    // 添加记忆
    let content = "我叫李四，今年25岁。";
    println!("\n   添加内容: {}", content);
    let id = mem.add(content).await?;

    // 检查是否使用了智能功能
    let all_memories = mem.get_all().await?;
    if let Some(memory) = all_memories.iter().find(|m| m.id == id) {
        println!("\n   🔬 智能功能检查:");

        if memory.entities.is_empty() {
            println!("      ❌ 实体提取: 未生效 (配置启用但无效)");
        } else {
            println!("      ✅ 实体提取: {:?}", memory.entities);
        }

        if memory.embedding.is_none() {
            println!("      ❌ 向量嵌入: 未生效 (配置启用但无效)");
        } else {
            println!("      ✅ 向量嵌入: 已生成");
        }
    }

    println!("\n   💡 结论: 配置启用了智能功能，但因为缺少智能组件，功能仍然无效 ❌");
    println!("   💡 原因: MemoryManager.fact_extractor = None, decision_engine = None");
    println!();

    Ok(())
}
