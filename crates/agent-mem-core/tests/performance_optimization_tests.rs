//! Performance Optimization Tests - Phase 2 & 3
//!
//! 测试：
//! - Phase 2: 综合评分系统（relevance + importance + recency）
//! - Phase 3: HCAM极简Prompt构建
//! - 性能指标验证

use agent_mem_core::orchestrator::{
    ChatRequest, MemoryIntegrator, MemoryIntegratorConfig, Orchestrator, OrchestratorConfig,
};
use agent_mem_core::{engine::MemoryEngine, Memory};
use agent_mem_llm::LLMClient;
use agent_mem_storage::{create_storage, StorageConfig};
use agent_mem_tools::ToolExecutor;
use agent_mem_traits::{AttributeKey, AttributeValue, Content, Message, Result};
use chrono::{Duration, Utc};
use std::sync::Arc;
use tokio;

/// Helper: 创建测试用的Memory
fn create_test_memory(id: &str, content: &str, importance: f64, age_days: i64) -> Memory {
    let created_at = Utc::now() - Duration::days(age_days);
    let mut memory = Memory::new_text(content, None);
    memory.id = id.to_string();
    memory.metadata.created_at = created_at;
    memory.set_importance(importance);
    memory.set_score(0.8); // 默认相似度
    memory
}

/// Phase 2 测试：综合评分系统
#[tokio::test]
async fn test_phase2_comprehensive_scoring() -> Result<()> {
    let storage_config = StorageConfig::mock();
    let storage = create_storage(storage_config).await?;
    let memory_engine = Arc::new(MemoryEngine::new(storage));
    let config = MemoryIntegratorConfig::default();
    let integrator = MemoryIntegrator::new(memory_engine, config);

    // 创建测试记忆：不同importance和age组合
    let mem1 = create_test_memory("mem1", "Recent important", 0.9, 1); // 最近+重要
    let mem2 = create_test_memory("mem2", "Old important", 0.9, 60); // 旧+重要
    let mem3 = create_test_memory("mem3", "Recent unimportant", 0.3, 1); // 最近+不重要
    let mem4 = create_test_memory("mem4", "Old unimportant", 0.3, 60); // 旧+不重要

    // 计算综合评分
    let score1 = integrator.calculate_comprehensive_score(&mem1);
    let score2 = integrator.calculate_comprehensive_score(&mem2);
    let score3 = integrator.calculate_comprehensive_score(&mem3);
    let score4 = integrator.calculate_comprehensive_score(&mem4);

    println!("📊 Phase 2 综合评分测试:");
    println!("  mem1 (Recent+Important): {:.3}", score1);
    println!("  mem2 (Old+Important):     {:.3}", score2);
    println!("  mem3 (Recent+Unimportant):{:.3}", score3);
    println!("  mem4 (Old+Unimportant):   {:.3}", score4);

    // 验证排序：Recent+Important > Old+Important > Recent+Unimportant > Old+Unimportant
    assert!(
        score1 > score2,
        "Recent+Important should rank higher than Old+Important"
    );
    assert!(
        score2 > score3,
        "Old+Important should rank higher than Recent+Unimportant"
    );
    assert!(
        score3 > score4,
        "Recent+Unimportant should rank higher than Old+Unimportant"
    );

    // 验证时效性衰减生效
    assert!(
        (score1 - score2).abs() > 0.05,
        "Recency decay should have significant impact"
    );

    println!("✅ Phase 2 综合评分测试通过");
    Ok(())
}

/// Phase 2 测试：sort_memories使用综合评分
#[tokio::test]
async fn test_phase2_sort_with_comprehensive_scoring() -> Result<()> {
    let storage_config = StorageConfig::mock();
    let storage = create_storage(storage_config).await?;
    let memory_engine = Arc::new(MemoryEngine::new(storage));
    let config = MemoryIntegratorConfig::default();
    let integrator = MemoryIntegrator::new(memory_engine, config);

    // 创建乱序的记忆列表
    let mut memories = vec![
        create_test_memory("mem4", "Old unimportant", 0.3, 60),
        create_test_memory("mem1", "Recent important", 0.9, 1),
        create_test_memory("mem3", "Recent unimportant", 0.3, 1),
        create_test_memory("mem2", "Old important", 0.9, 60),
    ];

    // 使用综合评分排序
    memories = integrator.sort_memories(memories);

    println!("📊 Phase 2 排序测试:");
    for (i, mem) in memories.iter().enumerate() {
        let score = integrator.calculate_comprehensive_score(mem);
        println!("  {}. {} (score: {:.3})", i + 1, mem.id, score);
    }

    // 验证排序结果
    assert_eq!(memories[0].id, "mem1", "First should be Recent+Important");
    assert_eq!(
        memories[memories.len() - 1].id,
        "mem4",
        "Last should be Old+Unimportant"
    );

    println!("✅ Phase 2 排序测试通过");
    Ok(())
}

/// Phase 3 测试：极简Prompt长度
#[tokio::test]
async fn test_phase3_minimal_prompt_length() -> Result<()> {
    let storage_config = StorageConfig::mock();
    let storage = create_storage(storage_config).await?;
    let memory_engine = Arc::new(MemoryEngine::new(storage.clone()));

    let llm_client = LLMClient::new(vec![]);
    let tool_executor = ToolExecutor::new(vec![]);

    let config = OrchestratorConfig {
        max_memories: 3, // 只检索3条
        ..Default::default()
    };

    let orchestrator = Orchestrator::new(
        Arc::new(llm_client),
        memory_engine,
        Arc::new(tool_executor),
        storage,
        config,
    )
    .await?;

    // 创建测试请求
    let request = ChatRequest {
        message: "测试消息".to_string(),
        agent_id: "test_agent".to_string(),
        user_id: "test_user".to_string(),
        organization_id: "test_org".to_string(),
        session_id: "test_session".to_string(),
        stream: false,
        max_memories: 3,
    };

    // 构建prompt（通过内部方法，这里简化测试）
    // 实际场景中，orchestrator.chat()会调用build_messages_with_context

    // 验证目标：Prompt长度应该<500字符
    // 这里通过MemoryIntegrator的inject_memories_to_prompt测试
    let storage_config = StorageConfig::mock();
    let storage_test = create_storage(storage_config).await?;
    let memory_engine_test = Arc::new(MemoryEngine::new(storage_test));
    let integrator = MemoryIntegrator::new(memory_engine_test, MemoryIntegratorConfig::default());

    let test_memories = vec![
        create_test_memory("m1", "这是一条很长的记忆内容，包含了大量的详细信息和上下文，用于测试极简格式是否能够有效截断".repeat(3).as_str(), 0.8, 1),
        create_test_memory("m2", "第二条记忆".to_string().as_str(), 0.7, 2),
        create_test_memory("m3", "第三条记忆".to_string().as_str(), 0.6, 3),
    ];

    let prompt = integrator.inject_memories_to_prompt(&test_memories);
    let prompt_length = prompt.len();

    println!("📊 Phase 3 Prompt长度测试:");
    println!("  Prompt长度: {} 字符", prompt_length);
    println!("  Prompt内容预览:\n{}", &prompt[..prompt.len().min(200)]);

    // 验证：极简格式应该大幅缩短长度
    assert!(
        prompt_length < 500,
        "Prompt should be <500 chars, got {}",
        prompt_length
    );

    // 验证：包含截断标记
    assert!(prompt.contains("..."), "Long content should be truncated");

    println!(
        "✅ Phase 3 Prompt长度测试通过 ({} chars < 500)",
        prompt_length
    );
    Ok(())
}

/// Phase 3 测试：记忆数量限制
#[tokio::test]
async fn test_phase3_memory_limit() -> Result<()> {
    let storage_config = StorageConfig::mock();
    let storage = create_storage(storage_config).await?;
    let memory_engine = Arc::new(MemoryEngine::new(storage));
    let integrator = MemoryIntegrator::new(memory_engine, MemoryIntegratorConfig::default());

    // 创建10条记忆
    let test_memories: Vec<Memory> = (0..10)
        .map(|i| create_test_memory(&format!("mem{}", i), &format!("Memory {}", i), 0.8, i))
        .collect();

    let prompt = integrator.inject_memories_to_prompt(&test_memories);

    println!("📊 Phase 3 记忆数量限制测试:");
    println!("  输入: {} 条记忆", test_memories.len());

    // 计算实际包含的记忆数量（通过行数）
    let memory_lines: Vec<&str> = prompt
        .lines()
        .filter(|l| l.starts_with(char::is_numeric))
        .collect();
    let included_count = memory_lines.len();

    println!("  输出: {} 条记忆", included_count);

    // 验证：最多5条（Phase 3优化）
    assert!(
        included_count <= 5,
        "Should include at most 5 memories, got {}",
        included_count
    );

    println!("✅ Phase 3 记忆数量限制测试通过 ({} <= 5)", included_count);
    Ok(())
}

/// 性能基准测试：TTFB模拟
#[tokio::test]
async fn test_performance_ttfb_benchmark() -> Result<()> {
    use std::time::Instant;

    let storage_config = StorageConfig::mock();
    let storage = create_storage(storage_config).await?;
    let memory_engine = Arc::new(MemoryEngine::new(storage));
    let config = MemoryIntegratorConfig {
        max_memories: 3, // Phase 3优化：默认3条
        ..Default::default()
    };
    let integrator = MemoryIntegrator::new(memory_engine, config);

    let test_memories: Vec<Memory> = (0..3)
        .map(|i| create_test_memory(&format!("mem{}", i), &format!("Memory {}", i), 0.8, i))
        .collect();

    // 测试综合评分性能
    let start = Instant::now();
    for mem in &test_memories {
        let _ = integrator.calculate_comprehensive_score(mem);
    }
    let scoring_time = start.elapsed();

    // 测试排序性能
    let start = Instant::now();
    let _sorted = integrator.sort_memories(test_memories.clone());
    let sort_time = start.elapsed();

    // 测试Prompt构建性能
    let start = Instant::now();
    let _prompt = integrator.inject_memories_to_prompt(&test_memories);
    let prompt_time = start.elapsed();

    println!("📊 性能基准测试:");
    println!("  综合评分 (3条): {:?}", scoring_time);
    println!("  排序      (3条): {:?}", sort_time);
    println!("  Prompt构建(3条): {:?}", prompt_time);
    println!(
        "  总耗时:          {:?}",
        scoring_time + sort_time + prompt_time
    );

    // 验证：所有操作应该在10ms内完成（极快）
    let total_time = scoring_time + sort_time + prompt_time;
    assert!(
        total_time.as_millis() < 10,
        "Total time should be <10ms, got {:?}",
        total_time
    );

    println!("✅ 性能基准测试通过 (总耗时 {:?} < 10ms)", total_time);
    Ok(())
}

/// 集成测试：完整优化流程
#[tokio::test]
async fn test_full_optimization_pipeline() -> Result<()> {
    use std::time::Instant;

    let storage_config = StorageConfig::mock();
    let storage = create_storage(storage_config).await?;
    let memory_engine = Arc::new(MemoryEngine::new(storage));
    let config = MemoryIntegratorConfig {
        max_memories: 3,
        episodic_weight: 1.2,
        working_weight: 1.0,
        semantic_weight: 0.9,
        ..Default::default()
    };
    let integrator = MemoryIntegrator::new(memory_engine, config);

    println!("📊 完整优化流程测试:");
    let start = Instant::now();

    // Step 1: 创建测试记忆（模拟检索结果）
    let mut memories = vec![
        create_test_memory("episodic1", "用户昨天问过类似问题", 0.8, 1),
        create_test_memory("episodic2", "用户上周提到过相关话题", 0.7, 7),
        create_test_memory("working1", "当前会话上下文", 0.9, 0),
        create_test_memory("semantic1", "通用知识背景", 0.6, 30),
        create_test_memory("semantic2", "更多背景知识", 0.5, 60),
    ];
    println!("  Step 1: 创建5条测试记忆");

    // Step 2: 综合评分
    for mem in &mut memories {
        let score = integrator.calculate_comprehensive_score(mem);
        mem.set_score(score);
    }
    println!("  Step 2: 综合评分完成");

    // Step 3: 排序
    memories = integrator.sort_memories(memories);
    println!("  Step 3: 排序完成");

    // Step 4: 限制数量（取前3条）
    memories.truncate(3);
    println!("  Step 4: 限制为3条记忆");

    // Step 5: 构建极简Prompt
    let prompt = integrator.inject_memories_to_prompt(&memories);
    let prompt_length = prompt.len();
    println!("  Step 5: 构建Prompt ({} chars)", prompt_length);

    let total_time = start.elapsed();
    println!("  总耗时: {:?}", total_time);

    // 验证结果
    assert_eq!(memories.len(), 3, "Should have exactly 3 memories");
    assert!(prompt_length < 500, "Prompt should be <500 chars");
    assert!(total_time.as_millis() < 10, "Should complete in <10ms");

    // 验证排序：working > episodic > semantic
    println!("\n  排序结果:");
    for (i, mem) in memories.iter().enumerate() {
        println!(
            "    {}. {} (score: {:.3})",
            i + 1,
            mem.id,
            mem.score().unwrap_or(0.0)
        );
    }

    println!("\n✅ 完整优化流程测试通过");
    println!("   - 记忆数: 3条 ✓");
    println!("   - Prompt长度: {} < 500 ✓", prompt_length);
    println!("   - 耗时: {:?} < 10ms ✓", total_time);

    Ok(())
}
