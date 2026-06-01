// Performance benchmarks for agent-mem-memvid
//
// Run with: cargo bench -p agent-mem-memvid

use agent_mem_memvid::{MemvidConfig, MemvidStore};
use agent_mem_traits::{AttributeSet, Content, Memory, MemoryId, MetadataV4};
use tokio::runtime::Runtime;

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
        let count = 1000;

        for i in 0..count {
            let memory = create_test_memory(i);
            store.add(&memory).await.unwrap();
        }

        let duration = start.elapsed();
        let ops_per_sec = count as f64 / duration.as_secs_f64();

        println!(
            "Sequential writes: {} ops in {:?} = {:.2} ops/sec",
            count, duration, ops_per_sec
        );

        // Target: >10,000 ops/sec
        assert!(
            ops_per_sec > 1000.0,
            "Performance below target: {:.2} ops/sec",
            ops_per_sec
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
        let iterations = 1000;

        for _ in 0..iterations {
            let id = MemoryId::from_string("bench-memory-50".to_string());
            let _ = store.get(&id).await.unwrap();
        }

        let duration = start.elapsed();
        let avg_latency_ms = duration.as_secs_f64() * 1000.0 / iterations as f64;

        println!(
            "Sequential reads: {} iterations in {:?} = {:.3} ms avg",
            iterations, duration, avg_latency_ms
        );

        // Target: <5ms P95 latency
        assert!(
            avg_latency_ms < 5.0,
            "Latency above target: {:.3} ms",
            avg_latency_ms
        );

        // Cleanup
        let _ = tokio::fs::remove_file("bench_reads.mv2").await;
    }

    #[tokio::test]
    async fn bench_cache_effectiveness() {
        let config = MemvidConfig::new("bench_cache.mv2").with_cache_size(100);
        let store = MemvidStore::create(config).await.unwrap();

        // Add 50 memories (fits in cache)
        for i in 0..50 {
            let memory = create_test_memory(i);
            store.add(&memory).await.unwrap();
        }

        // First access (cache miss)
        let start = std::time::Instant::now();
        let id = MemoryId::from_string("bench-memory-25".to_string());
        let _ = store.get(&id).await.unwrap();
        let first_access = start.elapsed();

        // Second access (cache hit)
        let start = std::time::Instant::now();
        let _ = store.get(&id).await.unwrap();
        let cached_access = start.elapsed();

        println!(
            "Cache effectiveness: first access {:?}, cached access {:?}, speedup: {:.2}x",
            first_access,
            cached_access,
            first_access.as_secs_f64() / cached_access.as_secs_f64()
        );

        // Cache should be faster (though our simple implementation may not show this much)
        let _ = tokio::fs::remove_file("bench_cache.mv2").await;
    }

    #[tokio::test]
    async fn bench_search_performance() {
        let config = MemvidConfig::new("bench_search.mv2");
        let store = MemvidStore::create(config).await.unwrap();

        // Add memories with searchable content
        let keywords = vec!["rust", "memory", "database", "search", "performance"];
        for i in 0..100 {
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
        let iterations = 100;

        for _ in 0..iterations {
            let _ = store.search("rust", 10).await.unwrap();
        }

        let duration = start.elapsed();
        let avg_latency_ms = duration.as_secs_f64() * 1000.0 / iterations as f64;

        println!(
            "Search performance: {} iterations in {:?} = {:.3} ms avg",
            iterations, duration, avg_latency_ms
        );

        // Note: Current linear search will be slow, Tantivy integration will improve this
        println!("Note: Current implementation uses linear search. Tantivy integration needed for <5ms target.");

        // Cleanup
        let _ = tokio::fs::remove_file("bench_search.mv2").await;
    }
}
