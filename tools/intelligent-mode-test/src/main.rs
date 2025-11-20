//! 智能模式性能测试工具
//!
//! 用于验证 Phase 2 优化效果（并行LLM调用）

use agent_mem::orchestrator::{MemoryOrchestrator, OrchestratorConfig};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("\n🚀 AgentMem Phase 2 性能测试 - 智能模式");
    println!("================================\n");

    // 创建 Orchestrator（启用智能功能）
    // 优先使用 Zhipu AI，如果没有配置则使用 OpenAI
    let (llm_provider, llm_model) = if std::env::var("ZHIPU_API_KEY").is_ok() {
        println!("🔧 使用 Zhipu AI (GLM-4.6)");
        ("zhipu".to_string(), "glm-4.6".to_string())
    } else if std::env::var("OPENAI_API_KEY").is_ok() {
        println!("🔧 使用 OpenAI (GPT-3.5-Turbo)");
        ("openai".to_string(), "gpt-3.5-turbo".to_string())
    } else {
        println!("⚠️  未配置 LLM API Key，将自动降级到快速模式");
        ("openai".to_string(), "gpt-3.5-turbo".to_string())
    };

    let config = OrchestratorConfig {
        storage_url: Some("libsql://./data/intelligent_test.db".to_string()),
        llm_provider: Some(llm_provider),
        llm_model: Some(llm_model),
        embedder_provider: Some("fastembed".to_string()),
        embedder_model: Some("all-MiniLM-L6-v2".to_string()),
        vector_store_url: Some("./data/intelligent_lancedb".to_string()),
        enable_intelligent_features: true,
    };

    println!("📦 初始化 Orchestrator（智能模式）...");
    let orchestrator = MemoryOrchestrator::new_with_config(config).await?;
    println!("✅ Orchestrator 初始化完成\n");

    // 测试 1: 智能模式单个添加性能（Phase 2 验证）
    println!("📊 测试 1: 智能模式单个添加性能");
    println!("─────────────────────────────────────");
    println!("⚠️  注意: 此测试需要配置有效的 OpenAI API Key");
    println!("⚠️  如果未配置，将自动降级到快速模式\n");

    let test_count = 5; // 减少测试次数，因为LLM调用较慢
    let mut durations = Vec::new();

    for i in 0..test_count {
        let start = Instant::now();

        let result = orchestrator
            .add_memory_v2(
                format!("The user likes programming in Rust. Test {}", i),
                "test_agent".to_string(),
                Some("test_user".to_string()),
                None, // run_id
                None, // metadata
                true, // infer=true，启用智能模式
                None, // memory_type
                None, // prompt
            )
            .await;

        let duration = start.elapsed();
        durations.push(duration);

        match result {
            Ok(add_result) => {
                println!(
                    "  ✅ 记忆 {} 添加成功: {} 个事件, 耗时: {:?}",
                    i,
                    add_result.results.len(),
                    duration
                );
            }
            Err(e) => {
                println!("  ❌ 记忆 {} 添加失败: {}, 耗时: {:?}", i, e, duration);
            }
        }
    }

    let total_time: std::time::Duration = durations.iter().sum();
    let avg_time = total_time / test_count as u32;
    let throughput = 1000.0 / avg_time.as_millis() as f64;

    println!("\n✅ 测试完成");
    println!("   记忆数量: {}", test_count);
    println!("   总时间: {:?}", total_time);
    println!("   平均延迟: {:?}", avg_time);
    println!("   吞吐量: {:.2} ops/s", throughput);
    println!("   目标: 1,000 ops/s (需要并行LLM调用优化)");

    // 测试 2: 对比快速模式和智能模式
    println!("\n📊 测试 2: 快速模式 vs 智能模式对比");
    println!("─────────────────────────────────────");

    // 快速模式
    println!("\n🔹 快速模式 (infer=false):");
    let start = Instant::now();
    for i in 0..5 {
        orchestrator
            .add_memory_v2(
                format!("Fast mode test {}", i),
                "test_agent".to_string(),
                Some("test_user".to_string()),
                None,  // run_id
                None,  // metadata
                false, // infer=false
                None,  // memory_type
                None,  // prompt
            )
            .await?;
    }
    let fast_time = start.elapsed();
    let fast_throughput = 5000.0 / fast_time.as_millis() as f64;

    println!("   总时间: {:?}", fast_time);
    println!("   吞吐量: {:.2} ops/s", fast_throughput);

    // 智能模式
    println!("\n🔹 智能模式 (infer=true):");
    let start = Instant::now();
    for i in 0..5 {
        let _ = orchestrator
            .add_memory_v2(
                format!("Intelligent mode test {}", i),
                "test_agent".to_string(),
                Some("test_user".to_string()),
                None, // run_id
                None, // metadata
                true, // infer=true
                None, // memory_type
                None, // prompt
            )
            .await;
    }
    let intelligent_time = start.elapsed();
    let intelligent_throughput = 5000.0 / intelligent_time.as_millis() as f64;

    println!("   总时间: {:?}", intelligent_time);
    println!("   吞吐量: {:.2} ops/s", intelligent_throughput);

    // 对比
    println!("\n📈 性能对比:");
    println!("   快速模式: {:.2} ops/s", fast_throughput);
    println!("   智能模式: {:.2} ops/s", intelligent_throughput);
    if fast_time > intelligent_time {
        let speedup = fast_time.as_millis() as f64 / intelligent_time.as_millis() as f64;
        println!("   智能模式更快: {:.2}x", speedup);
    } else {
        let slowdown = intelligent_time.as_millis() as f64 / fast_time.as_millis() as f64;
        println!("   快速模式更快: {:.2}x", slowdown);
    }

    println!("\n================================");
    println!("✅ 所有测试完成！");
    println!("\n📈 Phase 2 优化总结:");
    println!("   - 并行LLM调用已实现（Step 1-4）");
    println!("   - 预期性能提升: 3x（150ms → 50ms）");
    println!("   - 目标吞吐量: 1,000 ops/s");
    println!("\n💡 下一步:");
    println!("   - 实现 LLM 结果缓存（Task 2.2）");
    println!("   - 进一步优化决策执行（Task 2.3）");
    println!("   - 运行真实压测验证（Task 2.4）");

    Ok(())
}
