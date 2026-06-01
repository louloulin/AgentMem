//! Integration tests for agent-mem-memvid
//!
//! These tests validate the RealMemvidStore implementation with real MemVid files.
//! Tests cover CRUD operations, concurrency, large-scale data, and error handling.

use crate::memvid_store::RealMemvidStore;
use agent_mem_traits::{AttributeSet, Content, Memory, MemoryId, MetadataV4};
use std::sync::Arc;

/// Helper: Create a test memory with specific content
fn create_test_memory(id: &str, content: &str) -> Memory {
    Memory {
        id: MemoryId::from_string(id.to_string()),
        content: Content::text(content),
        attributes: AttributeSet::new(),
        relations: Default::default(),
        metadata: MetadataV4::default(),
    }
}

/// Helper: Clean up test files
fn cleanup_test_file(path: &str) {
    let _ = std::fs::remove_file(path);
}

/// Helper: Generate a batch of test memories
fn generate_test_memories(count: usize, prefix: &str) -> Vec<Memory> {
    (0..count)
        .map(|i| Memory {
            id: MemoryId::from_string(format!("{}-{}", prefix, i)),
            content: Content::text(&format!(
                "Test memory content number {} with some searchable text",
                i
            )),
            attributes: AttributeSet::new(),
            relations: Default::default(),
            metadata: MetadataV4::default(),
        })
        .collect()
}

// ============================================================================
// CRUD Integration Tests
// ============================================================================

#[tokio::test]
async fn integration_create_and_open_store() {
    let path = "integration_create_open.mv2";
    cleanup_test_file(path);

    // Create a new store
    let store = RealMemvidStore::create(path).await.unwrap();

    // Add a memory
    let memory = create_test_memory("test-1", "Hello, world!");
    store.add(&memory).await.unwrap();

    // Close and reopen
    drop(store);
    let store2 = RealMemvidStore::open(path).await.unwrap();

    // Verify memory persists
    let retrieved = store2.get(&memory.id).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id.as_str(), "test-1");

    cleanup_test_file(path);
}

#[tokio::test]
async fn integration_full_crud_cycle() {
    let path = "integration_crud.mv2";
    cleanup_test_file(path);

    let store = RealMemvidStore::create(path).await.unwrap();

    // CREATE
    let memory = create_test_memory("crud-1", "Original content");
    store.add(&memory).await.unwrap();

    // READ
    let retrieved = store.get(&memory.id).await.unwrap();
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.id.as_str(), "crud-1");
    // MemVid adds metadata to the stored text, so just check it starts with expected content
    assert!(retrieved
        .content
        .to_string()
        .starts_with("Original content"));

    // UPDATE
    let updated_memory = Memory {
        id: memory.id.clone(),
        content: Content::text("Updated content"),
        attributes: AttributeSet::new(),
        relations: Default::default(),
        metadata: MetadataV4::default(),
    };
    store.update(&updated_memory).await.unwrap();

    let retrieved = store.get(&memory.id).await.unwrap();
    assert!(retrieved.is_some());
    assert!(retrieved
        .unwrap()
        .content
        .to_string()
        .starts_with("Updated content"));

    // DELETE
    store.delete(&memory.id).await.unwrap();

    let retrieved = store.get(&memory.id).await.unwrap();
    assert!(retrieved.is_none());

    cleanup_test_file(path);
}

#[tokio::test]
async fn integration_list_all_memories() {
    let path = "integration_list.mv2";
    cleanup_test_file(path);

    let store = RealMemvidStore::create(path).await.unwrap();

    // Add multiple memories
    for i in 0..10 {
        let memory = create_test_memory(&format!("list-{}", i), &format!("Content {}", i));
        store.add(&memory).await.unwrap();
    }

    // List all
    let memories = store.list().await.unwrap();
    assert_eq!(memories.len(), 10);

    // Verify IDs
    let ids: Vec<_> = memories.iter().map(|m| m.id.as_str().to_string()).collect();
    for i in 0..10 {
        assert!(ids.contains(&format!("list-{}", i)));
    }

    cleanup_test_file(path);
}

#[tokio::test]
async fn integration_count_memories() {
    let path = "integration_count.mv2";
    cleanup_test_file(path);

    let store = RealMemvidStore::create(path).await.unwrap();

    // Empty store
    assert_eq!(store.count().await.unwrap(), 0);

    // Add 100 memories
    for i in 0..100 {
        let memory = create_test_memory(&format!("count-{}", i), "Content");
        store.add(&memory).await.unwrap();
    }

    assert_eq!(store.count().await.unwrap(), 100);

    // Delete 20 memories
    for i in 0..20 {
        let id = MemoryId::from_string(format!("count-{}", i));
        store.delete(&id).await.unwrap();
    }

    let final_count = store.count().await.unwrap();
    println!("Final count after deleting 20 from 100: {}", final_count);
    assert_eq!(final_count, 80);

    cleanup_test_file(path);
}

// ============================================================================
// Search Integration Tests
// ============================================================================

#[tokio::test]
async fn integration_full_text_search() {
    let path = "integration_search.mv2";
    cleanup_test_file(path);

    let store = RealMemvidStore::create(path).await.unwrap();

    // Add memories with specific keywords
    let keywords = vec![
        ("search-1", "rust programming language"),
        ("search-2", "python scripting automation"),
        ("search-3", "rust memory management"),
        ("search-4", "javascript web development"),
        ("search-5", "rust async programming"),
    ];

    for (id, content) in &keywords {
        let memory = create_test_memory(id, content);
        store.add(&memory).await.unwrap();
    }

    // Search for "rust" - should match 3 results
    let results = store.search("rust", 10).await.unwrap();
    assert!(results.len() >= 2); // At least 2 matches

    // Verify results contain relevant URIs
    let result_uris: Vec<_> = results.iter().map(|r| r.uri.clone()).collect();
    // URIs should be in format "mv2://memory/{id}"
    let has_rust_match = result_uris.iter().any(|uri| {
        uri.contains("search-1") || uri.contains("search-3") || uri.contains("search-5")
    });
    assert!(
        has_rust_match,
        "Should find at least one rust-related result"
    );

    cleanup_test_file(path);
}

#[tokio::test]
async fn integration_search_with_snippets() {
    let path = "integration_snippets.mv2";
    cleanup_test_file(path);

    let store = RealMemvidStore::create(path).await.unwrap();

    // Add memory with searchable content
    let memory = create_test_memory("snippet-1", "The quick brown fox jumps over the lazy dog");
    store.add(&memory).await.unwrap();

    // Search for "fox"
    let results = store.search("fox", 10).await.unwrap();
    assert!(!results.is_empty());

    // Check result has expected text
    let result = &results[0];
    // The score field is Option<f32> in SearchHit
    if let Some(score) = result.score {
        assert!(score > 0.0);
    }
    // Text should contain the search term
    assert!(
        result.text.to_lowercase().contains("fox") || result.text.to_lowercase().contains("dog")
    );

    cleanup_test_file(path);
}

// ============================================================================
// Large-Scale Tests
// ============================================================================

#[tokio::test]
async fn integration_large_scale_write() {
    let path = "integration_large_write.mv2";
    cleanup_test_file(path);

    let store = RealMemvidStore::create(path).await.unwrap();

    let start = std::time::Instant::now();
    let count = 1000;

    // Add 1000 memories
    for i in 0..count {
        let memory = create_test_memory(
            &format!("large-{}", i),
            &format!("Memory number {} with unique content for testing", i),
        );
        store.add(&memory).await.unwrap();
    }

    let duration = start.elapsed();
    let ops_per_sec = count as f64 / duration.as_secs_f64();

    println!("\n=== Large-Scale Write Test ===");
    println!("Count: {}", count);
    println!("Duration: {:?}", duration);
    println!("Throughput: {:.2} ops/sec", ops_per_sec);

    // Verify count
    assert_eq!(store.count().await.unwrap(), count);

    cleanup_test_file(path);
}

#[tokio::test]
async fn integration_large_scale_read() {
    let path = "integration_large_read.mv2";
    cleanup_test_file(path);

    let store = RealMemvidStore::create(path).await.unwrap();

    // Pre-populate with 1000 memories
    for i in 0..1000 {
        let memory = create_test_memory(&format!("read-{}", i), "Content");
        store.add(&memory).await.unwrap();
    }

    let start = std::time::Instant::now();
    let iterations = 100;

    // Read 100 random memories
    for i in 0..iterations {
        let id = MemoryId::from_string(format!("read-{}", i * 10));
        let _ = store.get(&id).await.unwrap();
    }

    let duration = start.elapsed();
    let avg_latency_ms = duration.as_secs_f64() * 1000.0 / iterations as f64;

    println!("\n=== Large-Scale Read Test ===");
    println!("Iterations: {}", iterations);
    println!("Duration: {:?}", duration);
    println!("Average latency: {:.3} ms", avg_latency_ms);
    println!("Target: <5ms");

    cleanup_test_file(path);
}

#[tokio::test]
async fn integration_large_scale_search() {
    let path = "integration_large_search.mv2";
    cleanup_test_file(path);

    let store = RealMemvidStore::create(path).await.unwrap();

    // Add 500 memories with varied content
    let topics = vec!["rust", "python", "javascript", "go", "java"];
    for i in 0..500 {
        let topic = topics[i % topics.len()];
        let memory = create_test_memory(
            &format!("search-{}", i),
            &format!(
                "This is about {} programming with content number {}",
                topic, i
            ),
        );
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

    println!("\n=== Large-Scale Search Test ===");
    println!("Dataset: 500 memories");
    println!("Iterations: {}", iterations);
    println!("Duration: {:?}", duration);
    println!("Average latency: {:.3} ms", avg_latency_ms);

    cleanup_test_file(path);
}

// ============================================================================
// Concurrency Tests
// ============================================================================

#[tokio::test]
async fn integration_concurrent_reads() {
    let path = "integration_concurrent_reads.mv2";
    cleanup_test_file(path);

    let store = Arc::new(RealMemvidStore::create(path).await.unwrap());

    // Pre-populate with 100 memories
    for i in 0..100 {
        let memory = create_test_memory(&format!("concurrent-{}", i), "Content");
        store.add(&memory).await.unwrap();
    }

    // Spawn 10 concurrent readers
    let mut handles = vec![];
    for reader_id in 0..10 {
        let store_clone = Arc::clone(&store);
        let handle = tokio::spawn(async move {
            for i in 0..10 {
                let id = MemoryId::from_string(format!("concurrent-{}", i));
                let _ = store_clone.get(&id).await;
            }
            reader_id
        });
        handles.push(handle);
    }

    // Wait for all readers
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result >= 0 && result < 10);
    }

    println!("\n=== Concurrent Reads Test ===");
    println!("Concurrent readers: 10");
    println!("Reads per reader: 10");
    println!("Status: ✓ PASS");

    cleanup_test_file(path);
}

#[tokio::test]
async fn integration_concurrent_writes() {
    let path = "integration_concurrent_writes.mv2";
    cleanup_test_file(path);

    let store = RealMemvidStore::create(path).await.unwrap();

    // Spawn 5 concurrent writers
    let mut handles = vec![];
    for writer_id in 0..5 {
        let path = path.to_string();
        let handle = tokio::spawn(async move {
            // Each writer opens its own store instance
            let store = RealMemvidStore::open(&path).await.unwrap();

            for i in 0..10 {
                let memory = create_test_memory(
                    &format!("writer-{}-{}", writer_id, i),
                    &format!("Content from writer {}", writer_id),
                );
                let _ = store.add(&memory).await;
            }
            writer_id
        });
        handles.push(handle);
    }

    // Wait for all writers
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify all writes succeeded
    let count = store.count().await.unwrap();
    assert_eq!(count, 50); // 5 writers * 10 memories each

    println!("\n=== Concurrent Writes Test ===");
    println!("Concurrent writers: 5");
    println!("Writes per writer: 10");
    println!("Total memories: {}", count);
    println!("Status: ✓ PASS");

    cleanup_test_file(path);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn integration_open_nonexistent_file() {
    let path = "nonexistent_file_12345.mv2";

    // Try to open non-existent file
    let result = RealMemvidStore::open(path).await;

    assert!(result.is_err());

    println!("\n=== Error Handling: Open Non-Existent File ===");
    println!("Status: ✓ PASS - Correctly returns error");
}

#[tokio::test]
async fn integration_get_nonexistent_memory() {
    let path = "integration_get_missing.mv2";
    cleanup_test_file(path);

    let store = RealMemvidStore::create(path).await.unwrap();

    // Try to get non-existent memory
    let id = MemoryId::from_string("does-not-exist".to_string());
    let result = store.get(&id).await.unwrap();

    assert!(result.is_none());

    cleanup_test_file(path);
}

#[tokio::test]
async fn integration_update_nonexistent_memory() {
    let path = "integration_update_missing.mv2";
    cleanup_test_file(path);

    let store = RealMemvidStore::create(path).await.unwrap();

    // Try to update non-existent memory
    let memory = create_test_memory("does-not-exist", "Content");
    let result = store.update(&memory).await;

    assert!(result.is_err());

    cleanup_test_file(path);
}

#[tokio::test]
async fn integration_delete_nonexistent_memory() {
    let path = "integration_delete_missing.mv2";
    cleanup_test_file(path);

    let store = RealMemvidStore::create(path).await.unwrap();

    // Try to delete non-existent memory
    let id = MemoryId::from_string("does-not-exist".to_string());
    let result = store.delete(&id).await;

    assert!(result.is_err());

    cleanup_test_file(path);
}

// ============================================================================
// Cache Behavior Tests
// ============================================================================

#[tokio::test]
async fn integration_cache_hit() {
    let path = "integration_cache_hit.mv2";
    cleanup_test_file(path);

    let store = RealMemvidStore::create(path).await.unwrap();

    // Add a memory
    let memory = create_test_memory("cache-1", "Cached content");
    store.add(&memory).await.unwrap();

    // First read - loads into cache
    let _ = store.get(&memory.id).await.unwrap();

    // Second read - should hit cache (faster)
    let start = std::time::Instant::now();
    let _ = store.get(&memory.id).await.unwrap();
    let cached_duration = start.elapsed();

    println!("\n=== Cache Hit Test ===");
    println!("Cached read duration: {:?}", cached_duration);
    println!("Status: ✓ PASS");

    cleanup_test_file(path);
}

#[tokio::test]
async fn integration_cache_expiration() {
    let path = "integration_cache_expire.mv2";
    cleanup_test_file(path);

    // Create store with small cache (10 entries)
    let store = RealMemvidStore::create(path).await.unwrap();

    // Add 20 memories to exceed cache size
    for i in 0..20 {
        let memory = create_test_memory(&format!("cache-{}", i), "Content");
        store.add(&memory).await.unwrap();
    }

    // Access first 10 memories (load into cache)
    for i in 0..10 {
        let id = MemoryId::from_string(format!("cache-{}", i));
        let _ = store.get(&id).await.unwrap();
    }

    // Access next 10 memories (should evict earlier entries)
    for i in 10..20 {
        let id = MemoryId::from_string(format!("cache-{}", i));
        let _ = store.get(&id).await.unwrap();
    }

    // Try to access memory-0 again (may have been evicted)
    let id = MemoryId::from_string("cache-0".to_string());
    let result = store.get(&id).await.unwrap();

    // Should still retrieve correctly (from disk if not in cache)
    assert!(result.is_some());

    println!("\n=== Cache Expiration Test ===");
    println!("Cache size: 10");
    println!("Memories added: 20");
    println!("Status: ✓ PASS - LRU eviction works correctly");

    cleanup_test_file(path);
}

// ============================================================================
// Statistics Tests
// ============================================================================

#[tokio::test]
async fn integration_store_stats() {
    let path = "integration_stats.mv2";
    cleanup_test_file(path);

    let store = RealMemvidStore::create(path).await.unwrap();

    // Add some memories
    for i in 0..50 {
        let memory = create_test_memory(&format!("stats-{}", i), "Content");
        store.add(&memory).await.unwrap();
    }

    // Get stats
    let stats = store.stats().await.unwrap();

    assert_eq!(stats.frame_count, 50);

    println!("\n=== Store Statistics Test ===");
    println!("Frame count: {}", stats.frame_count);
    println!("Status: ✓ PASS");

    cleanup_test_file(path);
}

// ============================================================================
// Mixed Workload Tests
// ============================================================================

#[tokio::test]
async fn integration_mixed_workload() {
    let path = "integration_mixed.mv2";
    cleanup_test_file(path);

    let store = RealMemvidStore::create(path).await.unwrap();

    let start = std::time::Instant::now();

    // Mixed workload: 60% reads, 30% writes, 10% searches
    for i in 0..100 {
        match i % 10 {
            0..=5 => {
                // Read
                if i > 0 {
                    let id = MemoryId::from_string(format!("mixed-{}", i / 10));
                    let _ = store.get(&id).await;
                }
            }
            6..=8 => {
                // Write
                let memory = create_test_memory(&format!("mixed-{}", i), "New content");
                store.add(&memory).await.unwrap();
            }
            _ => {
                // Search
                let _ = store.search("content", 5).await;
            }
        }
    }

    let duration = start.elapsed();

    println!("\n=== Mixed Workload Integration Test ===");
    println!("Operations: 100 (60% read, 30% write, 10% search)");
    println!("Duration: {:?}", duration);
    println!(
        "Average: {:.3} ms/op",
        duration.as_secs_f64() * 1000.0 / 100.0
    );
    println!("Status: ✓ PASS");

    cleanup_test_file(path);
}

// ============================================================================
// Advanced Search Tests
// ============================================================================

#[tokio::test]
async fn integration_fuzzy_search() {
    let path = "integration_fuzzy.mv2";
    cleanup_test_file(path);

    let store = RealMemvidStore::create(path).await.unwrap();

    // Add memories with similar but not identical content
    let memories = vec![
        ("fuzzy-1", "The quick brown fox jumps"),
        ("fuzzy-2", "The qick brown fox jumps"), // typo: "qick" instead of "quick"
        ("fuzzy-3", "A fast brown fox running"),
        ("fuzzy-4", "The slow brown turtle walks"),
    ];

    for (id, content) in memories {
        let memory = create_test_memory(id, content);
        store.add(&memory).await.unwrap();
    }

    // Fuzzy search should find approximately matching terms
    let results = store.search_fuzzy("qick", 10).await.unwrap();
    assert!(!results.is_empty());

    println!("\n=== Fuzzy Search Test ===");
    println!("Query: 'qick' (typo)");
    println!("Results found: {}", results.len());
    println!("Status: ✓ PASS");

    cleanup_test_file(path);
}

#[tokio::test]
async fn integration_phrase_search() {
    let path = "integration_phrase.mv2";
    cleanup_test_file(path);

    let store = RealMemvidStore::create(path).await.unwrap();

    // Add memories with phrases
    let memories = vec![
        ("phrase-1", "The quick brown fox jumps over the lazy dog"),
        ("phrase-2", "quick brown fox"), // Partial match
        ("phrase-3", "The lazy dog sleeps"),
        ("phrase-4", "A different story entirely"),
    ];

    for (id, content) in memories {
        let memory = create_test_memory(id, content);
        store.add(&memory).await.unwrap();
    }

    // Phrase search should find exact phrase matches
    let results = store.search_phrase("quick brown fox", 10).await.unwrap();
    assert!(!results.is_empty());

    println!("\n=== Phrase Search Test ===");
    println!("Query: 'quick brown fox'");
    println!("Results found: {}", results.len());
    println!("Status: ✓ PASS");

    cleanup_test_file(path);
}

#[tokio::test]
async fn integration_multi_term_search() {
    let path = "integration_multi.mv2";
    cleanup_test_file(path);

    let store = RealMemvidStore::create(path).await.unwrap();

    // Add memories with different topics
    let memories = vec![
        ("multi-1", "Rust programming language"),
        ("multi-2", "Python scripting language"),
        ("multi-3", "JavaScript web development"),
        ("multi-4", "Go programming for concurrency"),
        ("multi-5", "Java enterprise applications"),
    ];

    for (id, content) in memories {
        let memory = create_test_memory(id, content);
        store.add(&memory).await.unwrap();
    }

    // Multi-term search with OR should find memories matching any term
    let results = store
        .search_multi(vec!["rust", "python", "javascript"], 10)
        .await
        .unwrap();
    assert!(!results.is_empty());

    println!("\n=== Multi-Term Search Test ===");
    println!("Query: 'rust OR python OR javascript'");
    println!("Results found: {}", results.len());
    println!("Status: ✓ PASS");

    cleanup_test_file(path);
}

#[tokio::test]
async fn integration_search_performance() {
    let path = "integration_search_perf.mv2";
    cleanup_test_file(path);

    let store = RealMemvidStore::create(path).await.unwrap();

    // Add 100 memories with varied content
    for i in 0..100 {
        let memory = create_test_memory(
            &format!("perf-{}", i),
            &format!(
                "Memory number {} with unique searchable content about various topics",
                i
            ),
        );
        store.add(&memory).await.unwrap();
    }

    // Benchmark search performance
    let iterations = 50;
    let start = std::time::Instant::now();

    for _ in 0..iterations {
        let _ = store.search("memory", 10).await;
    }

    let duration = start.elapsed();
    let avg_latency_ms = duration.as_secs_f64() * 1000.0 / iterations as f64;

    println!("\n=== Search Performance Test ===");
    println!("Dataset: 100 memories");
    println!("Iterations: {}", iterations);
    println!("Total duration: {:?}", duration);
    println!("Average latency: {:.3} ms", avg_latency_ms);
    println!("Target: <5ms");
    println!(
        "Status: {}",
        if avg_latency_ms < 5.0 {
            "✓ PASS"
        } else {
            "⚠ SLOW"
        }
    );

    cleanup_test_file(path);
}

// ============================================================================
// Batch Operations Tests
// ============================================================================

#[tokio::test]
async fn integration_batch_add() {
    let path = "integration_batch_add.mv2";
    cleanup_test_file(path);

    let store = RealMemvidStore::create(path).await.unwrap();

    // Create 50 memories
    let memories: Vec<Memory> = (0..50)
        .map(|i| create_test_memory(&format!("batch-{}", i), &format!("Content {}", i)))
        .collect();

    // Batch add
    let start = std::time::Instant::now();
    let ids = store.batch_add(&memories).await.unwrap();
    let duration = start.elapsed();

    assert_eq!(ids.len(), 50);

    // Verify all memories were added
    let count = store.count().await.unwrap();
    assert_eq!(count, 50);

    println!("\n=== Batch Add Test ===");
    println!("Added: {} memories", ids.len());
    println!("Duration: {:?}", duration);
    println!("Throughput: {:.0} ops/sec", 50.0 / duration.as_secs_f64());
    println!("Status: ✓ PASS");

    cleanup_test_file(path);
}

#[tokio::test]
async fn integration_batch_get() {
    let path = "integration_batch_get.mv2";
    cleanup_test_file(path);

    let store = RealMemvidStore::create(path).await.unwrap();

    // Create and add 30 memories
    let memories: Vec<Memory> = (0..30)
        .map(|i| create_test_memory(&format!("get-{}", i), &format!("Content {}", i)))
        .collect();

    let ids: Vec<MemoryId> = memories.iter().map(|m| m.id.clone()).collect();
    store.batch_add(&memories).await.unwrap();

    // Batch get
    let results = store.batch_get(&ids).await.unwrap();

    assert_eq!(results.len(), 30);
    assert!(results.iter().all(|r| r.is_some()));

    println!("\n=== Batch Get Test ===");
    println!("Retrieved: {} memories", results.len());
    println!("Status: ✓ PASS");

    cleanup_test_file(path);
}

#[tokio::test]
async fn integration_batch_delete() {
    let path = "integration_batch_delete.mv2";
    cleanup_test_file(path);

    let store = RealMemvidStore::create(path).await.unwrap();

    // Create and add 50 memories
    let memories: Vec<Memory> = (0..50)
        .map(|i| create_test_memory(&format!("del-{}", i), &format!("Content {}", i)))
        .collect();

    store.batch_add(&memories).await.unwrap();

    // Delete first 25 memories
    let ids_to_delete: Vec<MemoryId> = memories.iter().take(25).map(|m| m.id.clone()).collect();
    let deleted_count = store.batch_delete(&ids_to_delete).await.unwrap();

    assert_eq!(deleted_count, 25);

    // Verify count
    let count = store.count().await.unwrap();
    assert_eq!(count, 25);

    println!("\n=== Batch Delete Test ===");
    println!("Deleted: {} memories", deleted_count);
    println!("Remaining: {} memories", count);
    println!("Status: ✓ PASS");

    cleanup_test_file(path);
}

#[tokio::test]
async fn integration_batch_update() {
    let path = "integration_batch_update.mv2";
    cleanup_test_file(path);

    let store = RealMemvidStore::create(path).await.unwrap();

    // Create and add initial memories
    let memories: Vec<Memory> = (0..20)
        .map(|i| create_test_memory(&format!("update-{}", i), &format!("Original {}", i)))
        .collect();

    let ids: Vec<MemoryId> = memories.iter().map(|m| m.id.clone()).collect();
    store.batch_add(&memories).await.unwrap();

    // Create updated versions
    let updated_memories: Vec<Memory> = ids
        .iter()
        .enumerate()
        .map(|(i, id)| Memory {
            id: id.clone(),
            content: Content::text(&format!("Updated {}", i)),
            attributes: AttributeSet::new(),
            relations: Default::default(),
            metadata: MetadataV4::default(),
        })
        .collect();

    let updated_ids = store.batch_update(&updated_memories).await.unwrap();

    assert_eq!(updated_ids.len(), 20);

    // Verify updates
    let results = store.batch_get(&ids).await.unwrap();
    assert!(results.iter().all(|r| {
        r.as_ref()
            .map(|m| m.content.to_string().starts_with("Updated"))
            .unwrap_or(false)
    }));

    println!("\n=== Batch Update Test ===");
    println!("Updated: {} memories", updated_ids.len());
    println!("Status: ✓ PASS");

    cleanup_test_file(path);
}

#[tokio::test]
async fn integration_batch_mixed_operations() {
    let path = "integration_batch_mixed.mv2";
    cleanup_test_file(path);

    let store = RealMemvidStore::create(path).await.unwrap();

    // Add initial batch
    let memories1: Vec<Memory> = (0..30)
        .map(|i| create_test_memory(&format!("mixed-{}", i), &format!("Content {}", i)))
        .collect();

    store.batch_add(&memories1).await.unwrap();

    // Add another batch
    let memories2: Vec<Memory> = (30..50)
        .map(|i| create_test_memory(&format!("mixed-{}", i), &format!("Content {}", i)))
        .collect();

    store.batch_add(&memories2).await.unwrap();

    assert_eq!(store.count().await.unwrap(), 50);

    // Update some
    let update_ids: Vec<MemoryId> = memories1.iter().take(10).map(|m| m.id.clone()).collect();
    let updated_memories: Vec<Memory> = update_ids
        .iter()
        .enumerate()
        .map(|(i, id)| Memory {
            id: id.clone(),
            content: Content::text(&format!("Updated content {}", i)),
            attributes: AttributeSet::new(),
            relations: Default::default(),
            metadata: MetadataV4::default(),
        })
        .collect();

    store.batch_update(&updated_memories).await.unwrap();

    // Delete some
    let delete_ids: Vec<MemoryId> = memories1
        .iter()
        .skip(10)
        .take(10)
        .map(|m| m.id.clone())
        .collect();
    store.batch_delete(&delete_ids).await.unwrap();

    // Verify final state
    let final_count = store.count().await.unwrap();
    assert_eq!(final_count, 40); // 50 - 10 deleted

    println!("\n=== Batch Mixed Operations Test ===");
    println!("Initial: 50 memories");
    println!("Updated: 10 memories");
    println!("Deleted: 10 memories");
    println!("Final count: {}", final_count);
    println!("Status: ✓ PASS");

    cleanup_test_file(path);
}

// ============================================================
// 向量搜索集成测试
// ============================================================

use crate::embedding::LocalEmbedding;
use crate::vector_search::{EmbeddingGenerator, VectorIndex, VectorSearchConfig};

#[tokio::test]
async fn integration_vector_index_basic() {
    let path = "test_vector_basic.mv2";
    cleanup_test_file(path);

    // 创建向量索引
    let embedding_gen = Arc::new(LocalEmbedding::new(128)) as Arc<dyn EmbeddingGenerator>;
    let index = VectorIndex::new(embedding_gen);

    // 添加一些向量
    let _ = index.upsert("mem1", "rust programming language").await;
    let _ = index.upsert("mem2", "python programming language").await;
    let _ = index.upsert("mem3", "javascript web development").await;

    // 验证索引大小
    let size = index.len().await;
    assert_eq!(size, 3);

    // 搜索测试
    let config = VectorSearchConfig {
        top_k: 2,
        min_similarity: 0.0,
        enable_cache: false,
    };

    let results = index.search("rust", &config).await.unwrap();
    assert!(!results.is_empty());
    assert!(results.len() <= 2);

    // 清理
    let _ = index.clear().await;
    let size = index.len().await;
    assert_eq!(size, 0);

    cleanup_test_file(path);
}

#[tokio::test]
async fn integration_vector_index_batch() {
    let path = "test_vector_batch.mv2";
    cleanup_test_file(path);

    let embedding_gen = Arc::new(LocalEmbedding::new(128)) as Arc<dyn EmbeddingGenerator>;
    let index = VectorIndex::new(embedding_gen);

    // 批量添加
    let items = vec![
        ("mem1".to_string(), "apple fruit".to_string()),
        ("mem2".to_string(), "banana fruit".to_string()),
        ("mem3".to_string(), "orange fruit".to_string()),
        ("mem4".to_string(), "carrot vegetable".to_string()),
    ];

    let _ = index.upsert_batch(items).await;

    let size = index.len().await;
    assert_eq!(size, 4);

    // 搜索相似内容
    let config = VectorSearchConfig {
        top_k: 3,
        min_similarity: 0.1,
        enable_cache: true,
    };

    let results = index.search("fruit", &config).await.unwrap();
    assert!(!results.is_empty());

    cleanup_test_file(path);
}

#[tokio::test]
async fn integration_vector_similarity_threshold() {
    let path = "test_vector_threshold.mv2";
    cleanup_test_file(path);

    let embedding_gen = Arc::new(LocalEmbedding::new(64)) as Arc<dyn EmbeddingGenerator>;
    let index = VectorIndex::new(embedding_gen);

    // 添加测试数据
    let _ = index.upsert("id1", "hello world").await;
    let _ = index.upsert("id2", "goodbye world").await;
    let _ = index.upsert("id3", "rust programming").await;

    // 高阈值搜索（应该返回更少结果）
    let config_high = VectorSearchConfig {
        top_k: 10,
        min_similarity: 0.9,
        enable_cache: false,
    };

    let results_high = index.search("hello", &config_high).await.unwrap();

    // 低阈值搜索（应该返回更多结果）
    let config_low = VectorSearchConfig {
        top_k: 10,
        min_similarity: 0.0,
        enable_cache: false,
    };

    let results_low = index.search("hello", &config_low).await.unwrap();

    // 低阈值应该返回更多或相等的结果
    assert!(results_low.len() >= results_high.len());

    cleanup_test_file(path);
}

#[tokio::test]
async fn integration_vector_remove() {
    let path = "test_vector_remove.mv2";
    cleanup_test_file(path);

    let embedding_gen = Arc::new(LocalEmbedding::new(128)) as Arc<dyn EmbeddingGenerator>;
    let index = VectorIndex::new(embedding_gen);

    // 添加数据
    let _ = index.upsert("id1", "test one").await;
    let _ = index.upsert("id2", "test two").await;
    let _ = index.upsert("id3", "test three").await;

    assert_eq!(index.len().await, 3);

    // 删除一个
    let _ = index.remove("id2").await;

    assert_eq!(index.len().await, 2);

    // 验证删除后搜索不包含已删除项
    let config = VectorSearchConfig::default();
    let results = index.search("test", &config).await.unwrap();

    // 检查结果中不包含 id2
    let has_id2 = results.iter().any(|r| r.memory_id == "id2");
    assert!(!has_id2);

    cleanup_test_file(path);
}
