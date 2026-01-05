// AgentMem Performance Benchmarks
//
// 使用 criterion 进行性能基准测试
//
// 运行方式：
// ```bash
// cargo bench --bench memory_benchmarks
// ```

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use agent_mem::Memory;
use tokio::runtime::Runtime;

/// 基础操作基准测试
fn bench_basic_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    // 测试添加记忆的性能
    c.bench_function("add_memory", |b| {
        b.to_async(&rt).iter(|| {
            async {
                let memory = Memory::quick();
                black_box(memory.add(black_box("测试记忆内容")).await)
            }
        })
    });

    // 测试搜索记忆的性能
    c.bench_function("search_memory", |b| {
        b.to_async(&rt).iter(|| {
            async {
                let memory = Memory::quick();
                // 预先添加一些记忆
                memory.add("我喜欢喝咖啡").await.unwrap();
                memory.add("我是一名程序员").await.unwrap();
                black_box(memory.search(black_box("咖啡")).await)
            }
        })
    });

    // 测试更新记忆的性能
    c.bench_function("update_memory", |b| {
        b.to_async(&rt).iter(|| {
            async {
                let memory = Memory::quick();
                let add_result = memory.add("原始内容").await.unwrap();
                let memory_id = &add_result.results[0].id;
                black_box(memory.update(black_box(memory_id), black_box("更新内容")).await)
            }
        })
    });

    // 测试删除记忆的性能
    c.bench_function("delete_memory", |b| {
        b.to_async(&rt).iter(|| {
            async {
                let memory = Memory::quick();
                let add_result = memory.add("待删除内容").await.unwrap();
                let memory_id = &add_result.results[0].id;
                black_box(memory.delete(black_box(memory_id)).await)
            }
        })
    });
}

/// 批量操作基准测试
fn bench_batch_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("batch_add");

    for size in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.to_async(&rt).iter(|| {
                async {
                    let memory = Memory::quick();
                    for i in 0..size {
                        black_box(memory.add(&format!("测试记忆{}", i)).await);
                    }
                }
            })
        });
    }

    group.finish();
}

/// 搜索性能基准测试
fn bench_search_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("search_with_dataset_size");

    for size in [100, 500, 1000, 5000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.to_async(&rt).iter(|| {
                async {
                    let memory = Memory::quick();

                    // 预先填充数据
                    for i in 0..size {
                        memory.add(&format!("这是第{}条测试记忆", i)).await.unwrap();
                    }

                    // 测试搜索性能
                    black_box(memory.search("测试").await);
                }
            })
        });
    }

    group.finish();
}

/// 并发操作基准测试
fn bench_concurrent_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("concurrent_adds", |b| {
        b.to_async(&rt).iter(|| {
            async {
                let memory = Memory::quick();

                let handles: Vec<_> = (0..10)
                    .map(|i| {
                        let memory_clone = memory.clone();
                        tokio::spawn(async move {
                            memory_clone.add(&format!("并发记忆{}", i)).await
                        })
                    })
                    .collect();

                for handle in handles {
                    black_box(handle.await.unwrap());
                }
            }
        })
    });

    c.bench_function("concurrent_searches", |b| {
        b.to_async(&rt).iter(|| {
            async {
                let memory = Memory::quick();
                memory.add("测试搜索内容").await.unwrap();

                let handles: Vec<_> = (0..10)
                    .map(|_| {
                        let memory_clone = memory.clone();
                        tokio::spawn(async move {
                            memory_clone.search("测试").await
                        })
                    })
                    .collect();

                for handle in handles {
                    black_box(handle.await.unwrap());
                }
            }
        })
    });
}

/// 不同内容长度基准测试
fn bench_content_length(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("add_memory_by_content_length");

    let contents = vec![
        ("short", "简短内容"),
        ("medium", "这是一段中等长度的内容，包含了一些描述性的文字，大约有几十个字符"),
        ("long", &"这是一段较长的内容。".repeat(50)),
        ("very_long", &"这是一段非常长的内容。".repeat(200)),
    ];

    for (name, content) in contents.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(name), content, |b, content| {
            b.to_async(&rt).iter(|| {
                async {
                    let memory = Memory::quick();
                    black_box(memory.add(black_box(*content)).await)
                }
            })
        });
    }

    group.finish();
}

/// Embedding 性能基准测试
fn bench_embedding_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("embedding_generation", |b| {
        b.to_async(&rt).iter(|| {
            async {
                // 这里假设有 embedding 功能
                // 实际测试需要根据 embedding 实现调整
                black_box(async { "embedding_test".to_string() }).await
            }
        })
    });
}

/// 内存使用基准测试
fn bench_memory_usage(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("memory_usage_1000_items", |b| {
        b.to_async(&rt).iter(|| {
            async {
                let memory = Memory::quick();

                for i in 0..1000 {
                    memory.add(&format!("记忆内容{}", i)).await.unwrap();
                }

                black_box(&memory);
            }
        })
    });
}

criterion_group!(
    benches,
    bench_basic_operations,
    bench_batch_operations,
    bench_search_performance,
    bench_concurrent_operations,
    bench_content_length,
    bench_embedding_performance,
    bench_memory_usage
);

criterion_main!(benches);
