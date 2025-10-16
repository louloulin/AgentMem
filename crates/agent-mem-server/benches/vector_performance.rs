//! 向量存储性能基准测试
//!
//! 测试 LanceDB 和其他向量存储的性能

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::Rng;
use std::time::Duration;

// 生成随机向量
fn generate_random_vector(dimension: usize) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    (0..dimension).map(|_| rng.gen::<f32>()).collect()
}

// 生成多个随机向量
fn generate_random_vectors(count: usize, dimension: usize) -> Vec<Vec<f32>> {
    (0..count)
        .map(|_| generate_random_vector(dimension))
        .collect()
}

// LanceDB 插入性能测试
fn bench_lancedb_insert(c: &mut Criterion) {
    #[cfg(feature = "lancedb")]
    {
        use agent_mem_storage::backends::lancedb_store::LanceDBStore;
        use agent_mem_traits::{VectorStore, VectorData};
        use tempfile::TempDir;
        use std::collections::HashMap;

        let rt = tokio::runtime::Runtime::new().unwrap();

        let mut group = c.benchmark_group("lancedb_insert");
        group.measurement_time(Duration::from_secs(10));
        group.sample_size(10);

        for size in [10, 100, 1000].iter() {
            group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
                b.iter_batched(
                    || {
                        // Setup: 为每次迭代创建新的临时目录和 store
                        let temp_dir = TempDir::new().unwrap();
                        let store = rt
                            .block_on(async {
                                LanceDBStore::new(
                                    temp_dir.path().to_str().unwrap(),
                                    "vectors",
                                )
                                .await
                            })
                            .unwrap();

                        // 创建 VectorData
                        let vectors: Vec<VectorData> = (0..size)
                            .map(|i| VectorData {
                                id: format!("vec_{}", i),
                                vector: generate_random_vector(1536),
                                metadata: HashMap::new(),
                            })
                            .collect();

                        (store, vectors, temp_dir)
                    },
                    |(store, vectors, _temp_dir)| {
                        // Benchmark: 插入向量
                        rt.block_on(async {
                            store
                                .add_vectors(black_box(vectors))
                                .await
                                .unwrap();
                        });
                    },
                    criterion::BatchSize::SmallInput,
                );
            });
        }

        group.finish();
    }

    #[cfg(not(feature = "lancedb"))]
    {
        println!("LanceDB feature not enabled, skipping bench_lancedb_insert");
    }
}

// LanceDB 搜索性能测试
fn bench_lancedb_search(c: &mut Criterion) {
    #[cfg(feature = "lancedb")]
    {
        use agent_mem_storage::backends::lancedb_store::LanceDBStore;
        use agent_mem_traits::{VectorStore, VectorData};
        use tempfile::TempDir;
        use std::collections::HashMap;

        let rt = tokio::runtime::Runtime::new().unwrap();

        // 预先创建并填充数据
        let temp_dir = TempDir::new().unwrap();
        let store = rt
            .block_on(async {
                let store = LanceDBStore::new(temp_dir.path().to_str().unwrap(), "vectors")
                    .await
                    .unwrap();

                // 插入 10000 个向量
                let vectors: Vec<VectorData> = (0..10000)
                    .map(|i| VectorData {
                        id: format!("vec_{}", i),
                        vector: generate_random_vector(1536),
                        metadata: HashMap::new(),
                    })
                    .collect();
                store.add_vectors(vectors).await.unwrap();

                store
            });

        let mut group = c.benchmark_group("lancedb_search");
        group.measurement_time(Duration::from_secs(10));
        group.sample_size(20);

        for limit in [10, 100, 1000].iter() {
            group.bench_with_input(BenchmarkId::from_parameter(limit), limit, |b, &limit| {
                b.iter(|| {
                    let query = generate_random_vector(1536);
                    rt.block_on(async {
                        store
                            .search_vectors(black_box(query), black_box(limit), None)
                            .await
                            .unwrap();
                    });
                });
            });
        }

        group.finish();

        // 清理
        drop(store);
        drop(temp_dir);
    }

    #[cfg(not(feature = "lancedb"))]
    {
        println!("LanceDB feature not enabled, skipping bench_lancedb_search");
    }
}

// LanceDB 删除性能测试
fn bench_lancedb_delete(c: &mut Criterion) {
    #[cfg(feature = "lancedb")]
    {
        use agent_mem_storage::backends::lancedb_store::LanceDBStore;
        use agent_mem_traits::{VectorStore, VectorData};
        use tempfile::TempDir;
        use std::collections::HashMap;

        let rt = tokio::runtime::Runtime::new().unwrap();

        let mut group = c.benchmark_group("lancedb_delete");
        group.measurement_time(Duration::from_secs(10));
        group.sample_size(10);

        for size in [10, 100, 1000].iter() {
            group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
                b.iter_batched(
                    || {
                        // Setup: 创建 store 并插入数据
                        let temp_dir = TempDir::new().unwrap();
                        let store = rt
                            .block_on(async {
                                let store = LanceDBStore::new(
                                    temp_dir.path().to_str().unwrap(),
                                    "vectors",
                                )
                                .await
                                .unwrap();

                                // 插入向量
                                let vectors: Vec<VectorData> = (0..size)
                                    .map(|i| VectorData {
                                        id: format!("vec_{}", i),
                                        vector: generate_random_vector(1536),
                                        metadata: HashMap::new(),
                                    })
                                    .collect();
                                store.add_vectors(vectors).await.unwrap();

                                store
                            });

                        let ids_to_delete: Vec<String> =
                            (0..size).map(|i| format!("vec_{}", i)).collect();
                        (store, ids_to_delete, temp_dir)
                    },
                    |(store, ids_to_delete, _temp_dir)| {
                        // Benchmark: 删除向量
                        rt.block_on(async {
                            store.delete_vectors(black_box(ids_to_delete)).await.unwrap();
                        });
                    },
                    criterion::BatchSize::SmallInput,
                );
            });
        }

        group.finish();
    }

    #[cfg(not(feature = "lancedb"))]
    {
        println!("LanceDB feature not enabled, skipping bench_lancedb_delete");
    }
}

// 向量相似度计算性能测试
fn bench_vector_similarity(c: &mut Criterion) {
    let mut group = c.benchmark_group("vector_similarity");

    // 测试不同维度的向量相似度计算
    for dimension in [384, 768, 1536].iter() {
        group.bench_with_input(
            BenchmarkId::new("cosine", dimension),
            dimension,
            |b, &dimension| {
                let vec1 = generate_random_vector(dimension);
                let vec2 = generate_random_vector(dimension);

                b.iter(|| {
                    // 余弦相似度计算
                    let dot_product: f32 = vec1
                        .iter()
                        .zip(vec2.iter())
                        .map(|(a, b)| a * b)
                        .sum();
                    let norm1: f32 = vec1.iter().map(|x| x * x).sum::<f32>().sqrt();
                    let norm2: f32 = vec2.iter().map(|x| x * x).sum::<f32>().sqrt();
                    black_box(dot_product / (norm1 * norm2));
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("euclidean", dimension),
            dimension,
            |b, &dimension| {
                let vec1 = generate_random_vector(dimension);
                let vec2 = generate_random_vector(dimension);

                b.iter(|| {
                    // 欧氏距离计算
                    let distance: f32 = vec1
                        .iter()
                        .zip(vec2.iter())
                        .map(|(a, b)| (a - b).powi(2))
                        .sum::<f32>()
                        .sqrt();
                    black_box(distance);
                });
            },
        );
    }

    group.finish();
}

// 批量向量操作性能测试
fn bench_batch_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_operations");

    for batch_size in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("generate_vectors", batch_size),
            batch_size,
            |b, &batch_size| {
                b.iter(|| {
                    black_box(generate_random_vectors(batch_size, 1536));
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_lancedb_insert,
    bench_lancedb_search,
    bench_lancedb_delete,
    bench_vector_similarity,
    bench_batch_operations
);
criterion_main!(benches);

