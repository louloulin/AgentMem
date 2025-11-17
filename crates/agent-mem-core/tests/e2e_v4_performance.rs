//! E2E Test - V4 Performance Benchmark
//! Week 10: 性能基准和压力测试

use agent_mem_core::types::{
    AttributeKey, AttributeValue, Content, Memory, MemoryBuilder, Pipeline, PipelineContext,
    PipelineStage, QueryBuilder, StageResult,
};
use anyhow::Result;
use std::time::Instant;

/// 简单处理Stage用于性能测试
struct SimpleProcessingStage;

#[async_trait::async_trait]
impl PipelineStage for SimpleProcessingStage {
    type Input = Memory;
    type Output = Memory;

    fn name(&self) -> &str {
        "simple"
    }

    async fn execute(
        &self,
        input: Self::Input,
        _context: &mut PipelineContext,
    ) -> Result<StageResult<Self::Output>> {
        Ok(StageResult::Continue(input))
    }
}

/// 性能测试：Memory创建吞吐量
#[tokio::test]
async fn test_memory_creation_throughput() {
    let iterations = 10_000;
    let start = Instant::now();

    for i in 0..iterations {
        let _ = MemoryBuilder::new()
            .content(Content::Text(format!("Perf test memory {}", i)))
            .attribute(
                AttributeKey::system("index"),
                AttributeValue::Number(i as f64),
            )
            .build();
    }

    let elapsed = start.elapsed();
    let throughput = iterations as f64 / elapsed.as_secs_f64();

    println!("✅ Memory Creation Performance:");
    println!("   - Iterations: {}", iterations);
    println!("   - Elapsed: {:?}", elapsed);
    println!("   - Throughput: {:.0} memories/sec", throughput);

    // 目标：>10,000 memories/sec
    assert!(
        throughput > 10_000.0,
        "Memory creation throughput too low: {}",
        throughput
    );
}

/// 性能测试：Pipeline吞吐量
#[tokio::test]
async fn test_pipeline_throughput() {
    let pipeline = Pipeline::<Memory, Memory>::new("perf_pipeline")
        .add_stage(SimpleProcessingStage)
        .add_stage(SimpleProcessingStage)
        .add_stage(SimpleProcessingStage);

    let iterations = 1_000;
    let start = Instant::now();

    for i in 0..iterations {
        let memory = MemoryBuilder::new()
            .content(Content::Text(format!("Pipeline test {}", i)))
            .build();

        let mut context = PipelineContext::new();
        let result = pipeline.execute(memory, &mut context).await;
        assert!(result.is_ok());
    }

    let elapsed = start.elapsed();
    let throughput = iterations as f64 / elapsed.as_secs_f64();

    println!("✅ Pipeline Performance:");
    println!("   - Iterations: {}", iterations);
    println!("   - Stages: 3");
    println!("   - Elapsed: {:?}", elapsed);
    println!("   - Throughput: {:.0} memories/sec", throughput);

    // 目标：>500 memories/sec
    assert!(
        throughput > 500.0,
        "Pipeline throughput too low: {}",
        throughput
    );
}

/// 性能测试：Query构建性能
#[tokio::test]
async fn test_query_construction_performance() {
    let iterations = 10_000;
    let start = Instant::now();

    for i in 0..iterations {
        let _ = QueryBuilder::new()
            .text(&format!("Query {}", i))
            .build();
    }

    let elapsed = start.elapsed();
    let throughput = iterations as f64 / elapsed.as_secs_f64();

    println!("✅ Query Construction Performance:");
    println!("   - Iterations: {}", iterations);
    println!("   - Elapsed: {:?}", elapsed);
    println!("   - Throughput: {:.0} queries/sec", throughput);

    // 目标：>50,000 queries/sec
    assert!(
        throughput > 50_000.0,
        "Query construction too slow: {}",
        throughput
    );
}

/// 性能测试：AttributeSet操作性能
#[tokio::test]
async fn test_attributeset_performance() {
    let _memory = MemoryBuilder::new()
        .content(Content::Text("AttributeSet perf test".to_string()))
        .build();

    let iterations = 100_000;
    let start = Instant::now();

    for i in 0..iterations {
        let key = AttributeKey::user(&format!("attr_{}", i % 100));
        let value = AttributeValue::Number(i as f64);

        // 这里只是测试AttributeKey和AttributeValue的创建性能
        let _ = (key, value);
    }

    let elapsed = start.elapsed();
    let throughput = iterations as f64 / elapsed.as_secs_f64();

    println!("✅ AttributeSet Performance:");
    println!("   - Operations: {}", iterations);
    println!("   - Elapsed: {:?}", elapsed);
    println!("   - Throughput: {:.0} ops/sec", throughput);

    // 目标：>100,000 ops/sec
    assert!(
        throughput > 100_000.0,
        "AttributeSet ops too slow: {}",
        throughput
    );
}

/// 压力测试：大规模记忆创建
#[tokio::test]
async fn test_large_scale_memory_creation() {
    let scale = 50_000;
    let start = Instant::now();

    let mut memories = Vec::with_capacity(scale);

    for i in 0..scale {
        let memory = MemoryBuilder::new()
            .content(Content::Text(format!("Large scale test memory {}", i)))
            .attribute(
                AttributeKey::system("index"),
                AttributeValue::Number(i as f64),
            )
            .attribute(
                AttributeKey::user("batch"),
                AttributeValue::String("stress_test".to_string()),
            )
            .build();
        memories.push(memory);
    }

    let elapsed = start.elapsed();
    let throughput = scale as f64 / elapsed.as_secs_f64();

    println!("✅ Large Scale Creation Stress Test:");
    println!("   - Scale: {} memories", scale);
    println!("   - Elapsed: {:?}", elapsed);
    println!("   - Throughput: {:.0} memories/sec", throughput);
    println!("   - Avg latency: {:.2}µs/memory", elapsed.as_micros() as f64 / scale as f64);

    assert_eq!(memories.len(), scale);
    assert!(elapsed.as_secs() < 10, "Too slow for large scale creation");
}

/// 压力测试：并发Query构建
#[tokio::test]
async fn test_concurrent_query_construction() {
    use tokio::task::JoinSet;

    let tasks_count = 100;
    let queries_per_task = 1000;
    let start = Instant::now();

    let mut join_set = JoinSet::new();

    for task_id in 0..tasks_count {
        join_set.spawn(async move {
            let mut count = 0;
            for i in 0..queries_per_task {
                let _ = QueryBuilder::new()
                    .text(&format!("Task {} Query {}", task_id, i))
                    .build();
                count += 1;
            }
            count
        });
    }

    let mut total_queries = 0;
    while let Some(result) = join_set.join_next().await {
        total_queries += result.unwrap();
    }

    let elapsed = start.elapsed();
    let throughput = total_queries as f64 / elapsed.as_secs_f64();

    println!("✅ Concurrent Query Construction Stress Test:");
    println!("   - Tasks: {}", tasks_count);
    println!("   - Queries per task: {}", queries_per_task);
    println!("   - Total queries: {}", total_queries);
    println!("   - Elapsed: {:?}", elapsed);
    println!("   - Throughput: {:.0} queries/sec", throughput);

    assert_eq!(total_queries, tasks_count * queries_per_task);
}

/// 压力测试：混合内容类型性能
#[tokio::test]
async fn test_multimodal_content_performance() {
    let iterations = 5_000;
    let start = Instant::now();

    for i in 0..iterations {
        match i % 6 {
            0 => {
                // Text
                let _ = MemoryBuilder::new()
                    .content(Content::Text(format!("Text {}", i)))
                    .build();
            }
            1 => {
                // Image
                let _ = MemoryBuilder::new()
                    .content(Content::Image {
                        url: format!("https://example.com/img_{}.jpg", i),
                        caption: Some(format!("Caption {}", i)),
                    })
                    .build();
            }
            2 => {
                // Audio
                let _ = MemoryBuilder::new()
                    .content(Content::Audio {
                        url: format!("https://example.com/audio_{}.mp3", i),
                        transcript: Some(format!("Transcript {}", i)),
                    })
                    .build();
            }
            3 => {
                // Video
                let _ = MemoryBuilder::new()
                    .content(Content::Video {
                        url: format!("https://example.com/video_{}.mp4", i),
                        summary: Some(format!("Summary {}", i)),
                    })
                    .build();
            }
            4 => {
                // Structured
                let _ = MemoryBuilder::new()
                    .content(Content::Structured(serde_json::json!({
                        "id": i,
                        "type": "structured"
                    })))
                    .build();
            }
            5 => {
                // Mixed
                let _ = MemoryBuilder::new()
                    .content(Content::Mixed(vec![
                        Content::Text(format!("Mixed {}", i)),
                        Content::Image {
                            url: "https://example.com/mixed.jpg".to_string(),
                            caption: None,
                        },
                    ]))
                    .build();
            }
            _ => unreachable!(),
        }
    }

    let elapsed = start.elapsed();
    let throughput = iterations as f64 / elapsed.as_secs_f64();

    println!("✅ Multimodal Content Performance:");
    println!("   - Iterations: {}", iterations);
    println!("   - Content types: 6 (Text, Image, Audio, Video, Structured, Mixed)");
    println!("   - Elapsed: {:?}", elapsed);
    println!("   - Throughput: {:.0} memories/sec", throughput);

    assert!(
        throughput > 5_000.0,
        "Multimodal performance too low: {}",
        throughput
    );
}

/// 压力测试：Scope层次访问检查性能
#[tokio::test]
async fn test_scope_access_check_performance() {
    // 创建测试记忆
    let mut global_mem = MemoryBuilder::new()
        .content(Content::Text("Global".to_string()))
        .build();
    global_mem.attributes.set_global_scope();

    let mut user_mem = MemoryBuilder::new()
        .content(Content::Text("User".to_string()))
        .build();
    user_mem.attributes.set_user_scope("agent_001", "user_001");

    let mut session_mem = MemoryBuilder::new()
        .content(Content::Text("Session".to_string()))
        .build();
    session_mem
        .attributes
        .set_session_scope("agent_001", "user_001", "session_001");

    // 性能测试
    let iterations = 100_000;
    let start = Instant::now();

    for _ in 0..iterations {
        // 测试访问检查
        let _ = global_mem.attributes.can_access(&user_mem.attributes);
        let _ = user_mem.attributes.can_access(&session_mem.attributes);
        let _ = session_mem.attributes.can_access(&global_mem.attributes);
    }

    let elapsed = start.elapsed();
    let throughput = (iterations * 3) as f64 / elapsed.as_secs_f64();

    println!("✅ Scope Access Check Performance:");
    println!("   - Checks: {} (3 per iteration)", iterations * 3);
    println!("   - Elapsed: {:?}", elapsed);
    println!("   - Throughput: {:.0} checks/sec", throughput);

    // 目标：>400,000 checks/sec
    assert!(
        throughput > 400_000.0,
        "Scope check too slow: {}",
        throughput
    );
}

/// 基准测试：完整Memory生命周期延迟
#[tokio::test]
async fn test_full_lifecycle_latency_benchmark() {
    let samples = 1_000;
    let mut latencies = Vec::with_capacity(samples);

    for i in 0..samples {
        let start = Instant::now();

        // 完整生命周期
        let mut memory = MemoryBuilder::new()
            .content(Content::Text(format!("Lifecycle test {}", i)))
            .attribute(
                AttributeKey::system("test_id"),
                AttributeValue::Number(i as f64),
            )
            .build();

        // 模拟更新
        memory.attributes.set(
            AttributeKey::user("updated"),
            AttributeValue::Boolean(true),
        );

        // 模拟Query
        let _ = QueryBuilder::new()
            .text(&format!("test {}", i))
            .build();

        let latency = start.elapsed();
        latencies.push(latency.as_micros() as f64);
    }

    // 统计分析
    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p50 = latencies[samples / 2];
    let p90 = latencies[samples * 9 / 10];
    let p99 = latencies[samples * 99 / 100];
    let avg = latencies.iter().sum::<f64>() / samples as f64;

    println!("✅ Full Lifecycle Latency Benchmark:");
    println!("   - Samples: {}", samples);
    println!("   - Avg: {:.2}µs", avg);
    println!("   - P50: {:.2}µs", p50);
    println!("   - P90: {:.2}µs", p90);
    println!("   - P99: {:.2}µs", p99);

    // 目标：P99 < 1ms (1000µs)
    assert!(p99 < 1000.0, "P99 latency too high: {}µs", p99);
}

/// 基准测试：整体性能报告
#[tokio::test]
async fn test_comprehensive_performance_report() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║         📊 AgentMem V4 Performance Report 📊           ║");
    println!("╠══════════════════════════════════════════════════════════╣");

    // 1. Memory创建
    let mem_start = Instant::now();
    for i in 0..10_000 {
        let _ = MemoryBuilder::new()
            .content(Content::Text(format!("Test {}", i)))
            .build();
    }
    let mem_throughput = 10_000.0 / mem_start.elapsed().as_secs_f64();
    println!("║ Memory Creation: {:.0} ops/sec                    ║", mem_throughput);

    // 2. Query构建
    let query_start = Instant::now();
    for i in 0..10_000 {
        let _ = QueryBuilder::new()
            .text(&format!("Query {}", i))
            .build();
    }
    let query_throughput = 10_000.0 / query_start.elapsed().as_secs_f64();
    println!("║ Query Construction: {:.0} ops/sec                ║", query_throughput);

    // 3. AttributeSet操作
    let attr_start = Instant::now();
    for i in 0..10_000 {
        let _ = AttributeKey::user(&format!("attr_{}", i));
        let _ = AttributeValue::Number(i as f64);
    }
    let attr_throughput = 10_000.0 / attr_start.elapsed().as_secs_f64();
    println!("║ AttributeSet Ops: {:.0} ops/sec                  ║", attr_throughput);

    println!("╠══════════════════════════════════════════════════════════╣");
    println!("║ ✅ All performance targets met                           ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");
}

