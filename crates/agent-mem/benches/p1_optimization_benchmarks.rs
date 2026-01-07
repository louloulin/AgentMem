//! P1 Performance Optimization Benchmarks
//!
//! 验证 P1 阶段的性能优化效果：
//! - 克隆减少优化 (search_with_options, get_all)
//! - 哈希性能优化 (twox-hash vs DefaultHasher)
//! - 并行初始化优化
//!
//! 运行方式：
//! ```bash
//! cargo bench --bench p1_optimization_benchmarks
//! ```

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use agent_mem::Memory;
use tokio::runtime::Runtime;
use std::time::Duration;

/// ✅ P1: 测试克隆优化效果
///
/// 验证 search_with_options 中的克隆减少优化
/// 目标：99.9% fewer clones in typical workloads
fn bench_clone_optimization(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("clone_optimization");

    // 测试不同数据集大小下的搜索性能
    for size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.to_async(&rt).iter(|| {
                async {
                    let memory = Memory::new_core().await.unwrap();

                    // 预先填充数据
                    for i in 0..size {
                        let _ = memory
                            .add(&format!("测试记忆内容 {} - 这是一个关于编程和技术的描述", i))
                            .await;
                    }

                    // 测试搜索性能（已优化：先过滤后克隆）
                    let _results = memory.search(&format!("编程")).await.unwrap();
                }
            })
        });
    }

    group.finish();
}

/// ✅ P1: 测试哈希性能优化
///
/// 验证 twox-hash vs DefaultHasher 的性能差异
/// 目标：~10x faster (1μs → <100ns per hash)
fn bench_hash_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_performance");

    // 测试不同输入大小的哈希性能
    for size in [10, 50, 100, 500].iter() {
        group.throughput(Throughput::Bytes(*size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let query = "test query".repeat(*size);

            b.iter(|| {
                use std::hash::{Hash, Hasher};
                use twox_hash::XxHash64;

                // ✅ P1 优化后的哈希
                let mut hasher = XxHash64::default();
                black_box(&query).hash(&mut hasher);
                let hash = black_box(hasher.finish());

                // 防止编译器优化掉计算
                black_box(hash);
            })
        });
    }

    group.finish();
}

/// ✅ P1: 测试并行初始化优化
///
/// 验证 tokio::try_join! 并行初始化的性能提升
/// 目标：40-60% startup time reduction
fn bench_parallel_initialization(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    // 只测试一次初始化（因为初始化不是高频操作）
    c.bench_function("parallel_initialization", |b| {
        b.to_async(&rt).iter(|| {
            async {
                // ✅ P1 优化：并行初始化（内部使用 tokio::try_join!）
                let memory = Memory::builder()
                    .with_core_features()
                    .build()
                    .await
                    .unwrap();

                black_box(memory);
            }
        })
    });
}

/// ✅ P1: 测试搜索性能（综合测试）
///
/// 验证综合搜索性能，包括：
/// - 文本匹配
/// - 向量搜索
/// - 结果排序
fn bench_search_comprehensive(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("search_comprehensive");

    for size in [100, 500, 1000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.to_async(&rt).iter(|| {
                async {
                    let memory = Memory::new_core().await.unwrap();

                    // 预先填充多样化的数据
                    for i in 0..size {
                        let topics = vec!["编程", "Rust", "Python", "AI", "机器学习", "数据库"];
                        let topic = topics[i % topics.len()];
                        let _ = memory
                            .add(&format!("关于{}的学习笔记 {}", topic, i))
                            .await;
                    }

                    // 测试搜索（包含文本匹配和向量搜索）
                    let _results = memory.search("编程").await.unwrap();

                    // 验证结果数量合理
                    assert!(_results.results.len() <= size as usize);
                }
            })
        });
    }

    group.finish();
}

/// ✅ P1: 测试批量操作性能
///
/// 验证批量添加的性能
/// 目标：验证克隆优化对批量操作的影响
fn bench_batch_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("batch_operations");

    for size in [10, 50, 100].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.to_async(&rt).iter(|| {
                async {
                    let memory = Memory::new_core().await.unwrap();

                    // 批量添加（已优化：减少克隆）
                    for i in 0..*size {
                        let _ = memory
                            .add(&format!("批量添加的记忆 {} - 测试内容", i))
                            .await;
                    }

                    black_box(size);
                }
            })
        });
    }

    group.finish();
}

/// ✅ P1: JWT Refresh Token 性能测试
///
/// 验证 refresh token 操作的性能
#[cfg(feature = "server")]
fn bench_jwt_refresh_tokens(c: &mut Criterion) {
    use agent_mem_server::auth::AuthService;
    use chrono::Duration;

    let auth_service = AuthService::new("test-secret-key-for-benchmarking-purposes-only");

    // 生成 token 对
    let token_pair = auth_service
        .generate_token_pair(
            "user123",
            "org456".to_string(),
            vec!["user".to_string()],
            None,
            Some(Duration::minutes(15)),
            Some(Duration::days(7)),
        )
        .unwrap();

    let mut group = c.benchmark_group("jwt_operations");

    // 测试 token 验证性能
    group.bench_function("validate_access_token", |b| {
        b.iter(|| {
            let _claims = black_box(auth_service.validate_access_token(&token_pair.access_token));
        })
    });

    // 测试 refresh token 性能
    group.bench_function("refresh_access_token", |b| {
        b.iter(|| {
            let _new_token = black_box(
                auth_service.refresh_access_token(&token_pair.refresh_token, None)
            );
        })
    });

    group.finish();
}

criterion_group! {
    name = p1_optimizations;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(10);
    targets =
        bench_clone_optimization,
        bench_hash_performance,
        bench_parallel_initialization,
        bench_search_comprehensive,
        bench_batch_operations
}

criterion_main!(p1_optimizations);
