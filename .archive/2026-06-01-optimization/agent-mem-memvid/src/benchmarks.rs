// Performance benchmarks for agent-mem-memvid
//
// These are simple benchmarks to establish performance baselines.
// For more comprehensive benchmarking, use criterion.

use crate::{MemvidConfig, MemvidStore, RealMemvidStore};
use agent_mem_traits::{AttributeSet, Content, Memory, MemoryId, MetadataV4};

fn create_test_memory(id: usize) -> Memory {
    Memory {
        id: MemoryId::from_string(format!("bench-memory-{}", id)),
        content: Content::text(&format!("Test memory content number {}", id)),
        attributes: AttributeSet::new(),
        relations: Default::default(),
        metadata: MetadataV4::default(),
    }
}

#[cfg(test)]
mod benchmarks {
    use super::*;

    #[tokio::test]
    async fn bench_sequential_writes() {
        let config = MemvidConfig::new("bench_sequential.mv2");
        let store = MemvidStore::create(config).await.unwrap();

        let start = std::time::Instant::now();
        let count = 100;

        for i in 0..count {
            let memory = create_test_memory(i);
            store.add(&memory).await.unwrap();
        }

        let duration = start.elapsed();
        let ops_per_sec = count as f64 / duration.as_secs_f64();

        println!("\n=== Sequential Write Benchmark ===");
        println!("Operations: {}", count);
        println!("Duration: {:?}", duration);
        println!("Throughput: {:.2} ops/sec", ops_per_sec);
        println!("Target: >10,000 ops/sec");
        println!(
            "Status: {}",
            if ops_per_sec > 1000.0 {
                "✓ PASS"
            } else {
                "✗ FAIL"
            }
        );

        // Cleanup
        let _ = tokio::fs::remove_file("bench_sequential.mv2").await;
    }

    #[tokio::test]
    async fn bench_sequential_reads() {
        let config = MemvidConfig::new("bench_reads.mv2");
        let store = MemvidStore::create(config).await.unwrap();

        // Add 100 memories
        for i in 0..100 {
            let memory = create_test_memory(i);
            store.add(&memory).await.unwrap();
        }

        let start = std::time::Instant::now();
        let iterations = 100;

        for _ in 0..iterations {
            let id = MemoryId::from_string("bench-memory-50".to_string());
            let _ = store.get(&id).await.unwrap();
        }

        let duration = start.elapsed();
        let avg_latency_ms = duration.as_secs_f64() * 1000.0 / iterations as f64;

        println!("\n=== Sequential Read Benchmark ===");
        println!("Iterations: {}", iterations);
        println!("Duration: {:?}", duration);
        println!("Average latency: {:.3} ms", avg_latency_ms);
        println!("Target: <5ms (P95)");
        println!(
            "Status: {}",
            if avg_latency_ms < 5.0 {
                "✓ PASS"
            } else {
                "✗ FAIL"
            }
        );

        // Cleanup
        let _ = tokio::fs::remove_file("bench_reads.mv2").await;
    }

    #[tokio::test]
    async fn bench_search_performance() {
        let config = MemvidConfig::new("bench_search.mv2");
        let store = MemvidStore::create(config).await.unwrap();

        // Add memories with searchable content
        let keywords = vec!["rust", "memory", "database", "search", "performance"];
        for i in 0..50 {
            let keyword = keywords[i % keywords.len()];
            let memory = Memory {
                id: MemoryId::from_string(format!("search-{}", i)),
                content: Content::text(&format!("This is about {}", keyword)),
                attributes: AttributeSet::new(),
                relations: Default::default(),
                metadata: MetadataV4::default(),
            };
            store.add(&memory).await.unwrap();
        }

        // Benchmark search
        let start = std::time::Instant::now();
        let iterations = 50;

        for _ in 0..iterations {
            let _ = store.search("rust", 10).await.unwrap();
        }

        let duration = start.elapsed();
        let avg_latency_ms = duration.as_secs_f64() * 1000.0 / iterations as f64;

        println!("\n=== Search Performance Benchmark ===");
        println!("Iterations: {}", iterations);
        println!("Dataset size: 50 memories");
        println!("Duration: {:?}", duration);
        println!("Average latency: {:.3} ms", avg_latency_ms);
        println!("Target: <5ms (with Tantivy integration)");
        println!("Note: Current implementation uses linear search (O(n))");
        println!("Status: ⏳ BASELINE (Tantivy integration needed)");

        // Cleanup
        let _ = tokio::fs::remove_file("bench_search.mv2").await;
    }

    #[tokio::test]
    async fn bench_mixed_workload() {
        let config = MemvidConfig::new("bench_mixed.mv2");
        let store = MemvidStore::create(config).await.unwrap();

        let start = std::time::Instant::now();

        // Mixed workload: 70% reads, 20% writes, 10% searches
        for i in 0..100 {
            if i % 10 < 7 {
                // Read
                if i > 0 {
                    let id = MemoryId::from_string(format!("bench-memory-{}", i / 10));
                    let _ = store.get(&id).await;
                }
            } else if i % 10 < 9 {
                // Write
                let memory = create_test_memory(i);
                store.add(&memory).await.unwrap();
            } else {
                // Search
                let _ = store.search("test", 5).await;
            }
        }

        let duration = start.elapsed();

        println!("\n=== Mixed Workload Benchmark ===");
        println!("Operations: 100 (70% read, 20% write, 10% search)");
        println!("Duration: {:?}", duration);
        println!(
            "Average: {:.3} ms/op",
            duration.as_secs_f64() * 1000.0 / 100.0
        );

        // Cleanup
        let _ = tokio::fs::remove_file("bench_mixed.mv2").await;
    }

    #[tokio::test]
    async fn bench_batch_add_vs_individual() {
        // Test individual adds
        let store1 = RealMemvidStore::create("bench_batch_individual.mv2")
            .await
            .unwrap();

        let individual_memories: Vec<Memory> = (0..100).map(|i| create_test_memory(i)).collect();

        let start_individual = std::time::Instant::now();
        for memory in &individual_memories {
            store1.add(memory).await.unwrap();
        }
        let duration_individual = start_individual.elapsed();

        // Test batch add
        let store2 = RealMemvidStore::create("bench_batch_batch.mv2")
            .await
            .unwrap();

        let start_batch = std::time::Instant::now();
        let _ = store2.batch_add(&individual_memories).await.unwrap();
        let duration_batch = start_batch.elapsed();

        let speedup = duration_individual.as_secs_f64() / duration_batch.as_secs_f64();

        println!("\n=== Batch Add vs Individual Benchmark ===");
        println!("Operations: 100");
        println!("Individual adds: {:?}", duration_individual);
        println!("Batch add: {:?}", duration_batch);
        println!("Speedup: {:.2}x", speedup);
        println!("Target: >5x speedup");
        println!(
            "Status: {}",
            if speedup > 2.0 {
                "✓ PASS"
            } else {
                "⚠ IMPROVEMENT NEEDED"
            }
        );

        // Cleanup
        let _ = tokio::fs::remove_file("bench_batch_individual.mv2").await;
        let _ = tokio::fs::remove_file("bench_batch_batch.mv2").await;
    }

    #[tokio::test]
    async fn bench_batch_get_vs_individual() {
        // First, populate a store
        let store = RealMemvidStore::create("bench_batch_get.mv2")
            .await
            .unwrap();

        let memories: Vec<Memory> = (0..100).map(|i| create_test_memory(i)).collect();
        store.batch_add(&memories).await.unwrap();

        let ids: Vec<MemoryId> = memories.iter().map(|m| m.id.clone()).collect();

        // Test individual gets
        let start_individual = std::time::Instant::now();
        for id in &ids {
            let _ = store.get(id).await.unwrap();
        }
        let duration_individual = start_individual.elapsed();

        // Test batch get
        let start_batch = std::time::Instant::now();
        let _ = store.batch_get(&ids).await.unwrap();
        let duration_batch = start_batch.elapsed();

        let speedup = duration_individual.as_secs_f64() / duration_batch.as_secs_f64();

        println!("\n=== Batch Get vs Individual Benchmark ===");
        println!("Operations: 100");
        println!("Individual gets: {:?}", duration_individual);
        println!("Batch get: {:?}", duration_batch);
        println!("Speedup: {:.2}x", speedup);
        println!("Target: >2x speedup");
        println!(
            "Status: {}",
            if speedup > 1.5 {
                "✓ PASS"
            } else {
                "⚠ IMPROVEMENT NEEDED"
            }
        );

        // Cleanup
        let _ = tokio::fs::remove_file("bench_batch_get.mv2").await;
    }

    #[tokio::test]
    async fn bench_batch_delete_vs_individual() {
        let ids: Vec<MemoryId> = (0..100)
            .map(|i| MemoryId::from_string(format!("bench-del-{}", i)))
            .collect();

        // Test individual deletes
        let store1 = RealMemvidStore::create("bench_batch_del_individual.mv2")
            .await
            .unwrap();

        let memories: Vec<Memory> = ids
            .iter()
            .enumerate()
            .map(|(i, id)| Memory {
                id: id.clone(),
                content: Content::text(&format!("Memory {}", i)),
                attributes: AttributeSet::new(),
                relations: Default::default(),
                metadata: MetadataV4::default(),
            })
            .collect();
        store1.batch_add(&memories).await.unwrap();

        let start_individual = std::time::Instant::now();
        for id in &ids {
            let _ = store1.delete(id).await;
        }
        let duration_individual = start_individual.elapsed();

        // Test batch delete
        let store2 = RealMemvidStore::create("bench_batch_del_batch.mv2")
            .await
            .unwrap();

        store2.batch_add(&memories).await.unwrap();
        let start_batch = std::time::Instant::now();
        let _ = store2.batch_delete(&ids).await.unwrap();
        let duration_batch = start_batch.elapsed();

        let speedup = duration_individual.as_secs_f64() / duration_batch.as_secs_f64();

        println!("\n=== Batch Delete vs Individual Benchmark ===");
        println!("Operations: 100");
        println!("Individual deletes: {:?}", duration_individual);
        println!("Batch delete: {:?}", duration_batch);
        println!("Speedup: {:.2}x", speedup);
        println!("Target: >5x speedup");
        println!(
            "Status: {}",
            if speedup > 2.0 {
                "✓ PASS"
            } else {
                "⚠ IMPROVEMENT NEEDED"
            }
        );

        // Cleanup
        let _ = tokio::fs::remove_file("bench_batch_del_individual.mv2").await;
        let _ = tokio::fs::remove_file("bench_batch_del_batch.mv2").await;
    }

    #[tokio::test]
    async fn bench_large_batch_operations() {
        let store = RealMemvidStore::create("bench_large_batch.mv2")
            .await
            .unwrap();

        // Test different batch sizes
        let batch_sizes = vec![10, 50, 100, 500, 1000];

        println!("\n=== Large Batch Operations Benchmark ===");
        println!("Testing various batch sizes...\n");

        for size in batch_sizes {
            let memories: Vec<Memory> = (0..size)
                .map(|i| Memory {
                    id: MemoryId::from_string(format!("large-batch-{}", i)),
                    content: Content::text(&format!("Memory content {}", i)),
                    attributes: AttributeSet::new(),
                    relations: Default::default(),
                    metadata: MetadataV4::default(),
                })
                .collect();

            let start = std::time::Instant::now();
            let _ = store.batch_add(&memories).await.unwrap();
            let duration = start.elapsed();

            let ops_per_sec = size as f64 / duration.as_secs_f64();

            println!(
                "Batch size: {:>4} | Time: {:>8.2?} | Throughput: {:>8.0} ops/sec",
                size, duration, ops_per_sec
            );

            // Cleanup for next iteration
            let ids: Vec<MemoryId> = memories.iter().map(|m| m.id.clone()).collect();
            let _ = store.batch_delete(&ids).await;
        }

        // Cleanup
        let _ = tokio::fs::remove_file("bench_large_batch.mv2").await;
    }

    // ============================================================
    // 向量搜索基准测试
    // ============================================================

    use crate::embedding::LocalEmbedding;
    use crate::vector_search::{EmbeddingGenerator, VectorIndex, VectorSearchConfig};
    use std::sync::Arc;

    #[tokio::test]
    async fn bench_vector_upsert_single() {
        let embedding_gen = Arc::new(LocalEmbedding::new(128)) as Arc<dyn EmbeddingGenerator>;
        let index = VectorIndex::new(embedding_gen);

        let iterations = 100;
        let start = std::time::Instant::now();

        for i in 0..iterations {
            let _ = index
                .upsert(
                    &format!("id-{}", i),
                    &format!("Test memory content number {}", i),
                )
                .await;
        }

        let duration = start.elapsed();
        let ops_per_sec = iterations as f64 / duration.as_secs_f64();

        println!("\n=== Vector Upsert Single Benchmark ===");
        println!("Iterations: {}", iterations);
        println!("Total time: {:?}", duration);
        println!("Throughput: {:.0} ops/sec", ops_per_sec);
        println!(
            "Average: {:.2} ms/op",
            duration.as_millis() as f64 / iterations as f64
        );
    }

    #[tokio::test]
    async fn bench_vector_upsert_batch() {
        let embedding_gen = Arc::new(LocalEmbedding::new(128)) as Arc<dyn EmbeddingGenerator>;
        let index = VectorIndex::new(embedding_gen);

        let batch_size = 100;
        let items: Vec<(String, String)> = (0..batch_size)
            .map(|i| {
                (
                    format!("batch-id-{}", i),
                    format!("Batch test content {}", i),
                )
            })
            .collect();

        let start = std::time::Instant::now();
        let _ = index.upsert_batch(items).await;
        let duration = start.elapsed();

        let ops_per_sec = batch_size as f64 / duration.as_secs_f64();

        println!("\n=== Vector Upsert Batch Benchmark ===");
        println!("Batch size: {}", batch_size);
        println!("Total time: {:?}", duration);
        println!("Throughput: {:.0} ops/sec", ops_per_sec);
        println!(
            "Average: {:.2} ms/op",
            duration.as_millis() as f64 / batch_size as f64
        );
    }

    #[tokio::test]
    async fn bench_vector_search_scales() {
        let embedding_gen = Arc::new(LocalEmbedding::new(128)) as Arc<dyn EmbeddingGenerator>;
        let index = VectorIndex::new(embedding_gen);

        let scales = vec![10, 50, 100, 500, 1000];

        println!("\n=== Vector Search Scaling Benchmark ===");
        println!(
            "{:>6} | {:>10} | {:>10} | {:>10}",
            "Size", "Build(ms)", "Search(ms)", "Throughput"
        );
        println!("{:-<54}", "");

        for size in scales {
            // Build index
            let items: Vec<(String, String)> = (0..size)
                .map(|i| {
                    (
                        format!("scale-{}-{}", size, i),
                        format!("Content {} for scale {}", i, size),
                    )
                })
                .collect();

            let build_start = std::time::Instant::now();
            let _ = index.upsert_batch(items).await;
            let build_duration = build_start.elapsed();

            // Perform searches
            let search_iterations = 10;
            let search_start = std::time::Instant::now();

            for _ in 0..search_iterations {
                let config = VectorSearchConfig {
                    top_k: 10,
                    min_similarity: 0.0,
                    enable_cache: false,
                };
                let _ = index.search("test query", &config).await;
            }

            let search_duration = search_start.elapsed();
            let avg_search_ms = search_duration.as_millis() as f64 / search_iterations as f64;

            println!(
                "{:>6} | {:>10.2} | {:>10.2} | {:>10.0}",
                size,
                build_duration.as_millis(),
                avg_search_ms,
                1000.0 / avg_search_ms
            );

            // Clear for next scale
            let _ = index.clear().await;
        }
    }

    #[tokio::test]
    async fn bench_vector_similarity_computation() {
        use crate::embedding::cosine_similarity;

        let dimension = 128;
        let iterations = 1000;

        // Generate test vectors
        let vectors: Vec<Vec<f32>> = (0..iterations)
            .map(|_| (0..dimension).map(|_| rand::random::<f32>()).collect())
            .collect();

        let query_vec: Vec<f32> = (0..dimension).map(|_| rand::random::<f32>()).collect();

        let start = std::time::Instant::now();

        for vec in &vectors {
            let _ = cosine_similarity(&query_vec, vec);
        }

        let duration = start.elapsed();

        println!("\n=== Vector Similarity Computation Benchmark ===");
        println!("Dimension: {}", dimension);
        println!("Iterations: {}", iterations);
        println!("Total time: {:?}", duration);
        println!(
            "Throughput: {:.0} comps/sec",
            iterations as f64 / duration.as_secs_f64()
        );
        println!(
            "Average: {:.2} µs/comp",
            duration.as_micros() as f64 / iterations as f64
        );
    }
}
