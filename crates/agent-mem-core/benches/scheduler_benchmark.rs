//! Memory Scheduler Benchmarks
//!
//! 基准测试 AgentMem 2.6 记忆调度器的性能
//!
//! ## 测试目标
//!
//! ### 延迟目标
//! - scheduler.select_memories() 延迟增加 <20% vs. search_memories()
//! - 调度分数计算 < 1ms per memory
//!
//! ### 精度目标
//! - 检索精度提升 +30-50%（基于相关性排序）
//! - Top-10 结果相关性分数提升
//!
//! ## 测试场景
//!
//! 1. **基准测试**: 无 scheduler vs. 有 scheduler 的性能对比
//! 2. **候选数量**: 不同候选数量下的性能（10, 50, 100, 500）
//! 3. **Top-K 选择**: 不同 top-k 值的性能（5, 10, 20, 50）
//! 4. **配置对比**: 不同调度策略的性能差异
//!
//! ## 参考文献
//!
//! - [Benchmarking Your Rust Code with Criterion](https://medium.com/rustaceans/benchmarking-your-rust-code-with-criterion-a-comprehensive-guide-fa38366870a6)
//! - [How to benchmark Rust code with Criterion](https://bencher.dev/learn/benchmarking/rust/criterion/)
//! - MemOS: A Memory OS for AI System (ACL 2025)

use agent_mem_core::scheduler::{DefaultMemoryScheduler, ExponentialDecayModel};
use agent_mem_traits::{
    AttributeKey, AttributeValue, Content, MemoryBuilder, MemoryScheduler, ScheduleConfig,
};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::sync::Arc;

/// 创建测试记忆
fn create_test_memory(id: usize, importance: f64, days_ago: f64) -> agent_mem_traits::MemoryV4 {
    let created_at = (chrono::Utc::now() - chrono::Duration::days(days_ago as i64)).timestamp();

    MemoryBuilder::new()
        .content(Content::Text(format!(
            "Test memory {} with importance {} from {} days ago",
            id, importance, days_ago
        )))
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

/// 创建候选记忆集合
fn create_candidate_memories(count: usize) -> Vec<agent_mem_traits::MemoryV4> {
    (0..count)
        .map(|i| {
            // 生成多样化的记忆：
            // - 重要性：0.3-0.9
            // - 时间：0-30天
            let importance = 0.3 + (i as f64 % 7.0) * 0.1;
            let days_ago = (i as f64 % 31.0);
            create_test_memory(i, importance, days_ago)
        })
        .collect()
}

/// 基准测试: 调度器选择性能（不同候选数量）
fn bench_scheduler_selection_by_candidate_count(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let scheduler = DefaultMemoryScheduler::new(
        ScheduleConfig::balanced(),
        ExponentialDecayModel::default(),
    );

    let candidate_counts = vec![10, 50, 100, 200, 500];

    let mut group = c.benchmark_group("scheduler_selection");

    for count in candidate_counts {
        let memories = create_candidate_memories(count);
        let query = "test query for benchmarking";

        group.throughput(Throughput::Elements(count as u64));

        group.bench_with_input(
            BenchmarkId::new("candidates", count),
            &count,
            |b, &_count| {
                b.to_async(tokio::runtime::Runtime::new().unwrap())
                    .iter(|| {
                        let selected = rt.block_on(async {
                            scheduler
                                .select_memories(
                                    black_box(query),
                                    black_box(memories.clone()),
                                    black_box(10),
                                )
                                .await
                                .unwrap()
                        });
                        black_box(selected)
                    });
            },
        );
    }

    group.finish();
}

/// 基准测试: 调度器选择性能（不同 top-k 值）
fn bench_scheduler_selection_by_top_k(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let scheduler = DefaultMemoryScheduler::new(
        ScheduleConfig::balanced(),
        ExponentialDecayModel::default(),
    );

    let memories = create_candidate_memories(100);
    let query = "test query for benchmarking";

    let top_k_values = vec![5, 10, 20, 50];

    let mut group = c.benchmark_group("scheduler_top_k");

    for top_k in top_k_values {
        group.bench_with_input(
            BenchmarkId::new("top_k", top_k),
            &top_k,
            |b, &_top_k| {
                b.to_async(tokio::runtime::Runtime::new().unwrap())
                    .iter(|| {
                        let selected = rt.block_on(async {
                            scheduler
                                .select_memories(
                                    black_box(query),
                                    black_box(memories.clone()),
                                    black_box(top_k),
                                )
                                .await
                                .unwrap()
                        });
                        black_box(selected)
                    });
            },
        );
    }

    group.finish();
}

/// 基准测试: 不同调度策略的性能对比
fn bench_scheduler_strategies(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let strategies = vec![
        ("balanced", ScheduleConfig::balanced()),
        ("relevance_focused", ScheduleConfig::relevance_focused()),
        ("importance_focused", ScheduleConfig::importance_focused()),
        ("recency_focused", ScheduleConfig::recency_focused()),
    ];

    let memories = create_candidate_memories(100);
    let query = "test query for benchmarking";

    let mut group = c.benchmark_group("scheduler_strategies");

    for (name, config) in strategies {
        let scheduler = DefaultMemoryScheduler::new(config.clone(), ExponentialDecayModel::default());

        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &name,
            |b, &_name| {
                b.to_async(tokio::runtime::Runtime::new().unwrap())
                    .iter(|| {
                        let selected = rt.block_on(async {
                            scheduler
                                .select_memories(
                                    black_box(query),
                                    black_box(memories.clone()),
                                    black_box(10),
                                )
                                .await
                                .unwrap()
                        });
                        black_box(selected)
                    });
            },
        );
    }

    group.finish();
}

/// 基准测试: 调度分数计算性能
fn bench_scheduler_scoring(c: &mut Criterion) {
    let scheduler = DefaultMemoryScheduler::new(
        ScheduleConfig::balanced(),
        ExponentialDecayModel::default(),
    );

    let memory = create_test_memory(0, 0.8, 1.0);
    let context = agent_mem_traits::ScheduleContext::new(0.7);

    c.bench_function("schedule_score_calculation", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let score = rt.block_on(async {
                scheduler
                    .schedule_score(
                        black_box(&memory),
                        black_box("test query"),
                        black_box(&context),
                    )
                    .await
                    .unwrap()
            });
            black_box(score)
        });
    });
}

/// 基准测试: 时间衰减计算性能
fn bench_time_decay(c: &mut Criterion) {
    let decay_model = ExponentialDecayModel::default();

    let ages = vec![0.0, 1.0, 7.0, 30.0, 100.0];

    let mut group = c.benchmark_group("time_decay");

    for age in ages {
        group.bench_with_input(
            BenchmarkId::new("age_days", age as u64),
            &age,
            |b, &_age| {
                b.iter(|| {
                    let score = decay_model.decay_score(black_box(age));
                    black_box(score)
                });
            },
        );
    }

    group.finish();
}

/// 对比测试: 有 scheduler vs. 无 scheduler（模拟）
fn bench_with_vs_without_scheduler(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let scheduler = DefaultMemoryScheduler::new(
        ScheduleConfig::balanced(),
        ExponentialDecayModel::default(),
    );

    let memories = create_candidate_memories(100);
    let query = "test query";

    let mut group = c.benchmark_group("with_vs_without_scheduler");

    // 无 scheduler（直接取 top-k）
    group.bench_function("without_scheduler", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| {
                let selected = memories.clone().into_iter().take(10).collect();
                black_box(selected)
            });
    });

    // 有 scheduler
    group.bench_function("with_scheduler", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| {
                let selected = rt.block_on(async {
                    scheduler
                        .select_memories(
                            black_box(query),
                            black_box(memories.clone()),
                            black_box(10),
                        )
                        .await
                        .unwrap()
                });
                black_box(selected)
            });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_scheduler_selection_by_candidate_count,
    bench_scheduler_selection_by_top_k,
    bench_scheduler_strategies,
    bench_scheduler_scoring,
    bench_time_decay,
    bench_with_vs_without_scheduler
);

criterion_main!(benches);
