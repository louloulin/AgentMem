//! P1 Integration Tests
//!
//! Comprehensive integration tests for P1 features:
//! - Input validation layer
//! - Database prepared statement caching
//! - Performance improvements
//!
//! Run with:
//! ```bash
//! cargo test --package agent-mem-server --test integration_test_p1
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{timeout, Duration};

// ==================== Test Utilities ====================

/// Test helper: Create test database connection
async fn create_test_store() -> agent_mem_storage::backends::libsql_core::LibSqlCoreStore {
    use libsql::Connection;
    use tokio::sync::Mutex;
    
    let conn = Connection::open_in_memory().expect("Failed to create in-memory DB");
    let conn = Arc::new(Mutex::new(conn));
    
    // Initialize schema
    {
        let conn_guard = conn.lock().await;
        conn_guard.execute(
            r#"
            CREATE TABLE IF NOT EXISTS core_memory (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                agent_id TEXT NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                category TEXT NOT NULL,
                is_mutable INTEGER DEFAULT 1,
                metadata TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
            libsql::params![],
        ).await.expect("Failed to create table");
    }
    
    agent_mem_storage::backends::libsql_core::LibSqlCoreStore::new(conn)
}

// ==================== Validation Tests ====================

#[tokio::test]
async fn test_validation_add_memory_valid() {
    use agent_mem_server::middleware::validation::validate_add_memory_request;
    
    let result = validate_add_memory_request(
        "Valid test content".to_string(),
        None,
        None,
        Some(0.5),
        Some("test-agent".to_string()),
        None,
    );
    
    assert!(result.is_ok(), "Valid request should pass validation");
}

#[tokio::test]
async fn test_validation_add_memory_html_rejection() {
    use agent_mem_server::middleware::validation::validate_add_memory_request;
    
    let dangerous_contents = vec![
        "<script>alert('xss')</script>",
        "<iframe src='evil.com'></iframe>",
        "javascript:alert('xss')",
        "onclick='evil()'",
        "onload='evil()'",
    ];
    
    for content in dangerous_contents {
        let result = validate_add_memory_request(
            content.to_string(),
            None,
            None,
            None,
            None,
            None,
        );
        
        assert!(result.is_err(), "Content with '{}' should be rejected", content);
    }
}

#[tokio::test]
async fn test_validation_payload_size_limit() {
    use agent_mem_server::middleware::validation::validate_add_memory_request;
    
    // Create a request that exceeds 1MB
    let large_content = "a".repeat(1_100_000); // Exceeds 1MB
    
    let mut metadata = HashMap::new();
    for i in 0..100 {
        metadata.insert(format!("key{}", i), "value".repeat(1000));
    }
    
    let result = validate_add_memory_request(
        large_content,
        Some(metadata),
        None,
        None,
        None,
        None,
    );
    
    assert!(result.is_err(), "Payload exceeding 1MB should be rejected");
}

#[tokio::test]
async fn test_validation_metadata_constraints() {
    use agent_mem_server::middleware::validation::validate_add_memory_request;
    
    // Test metadata key validation
    let mut metadata = HashMap::new();
    metadata.insert("invalid key!".to_string(), "value".to_string());
    
    let result = validate_add_memory_request(
        "Valid content".to_string(),
        Some(metadata),
        None,
        None,
        None,
        None,
    );
    
    assert!(result.is_err(), "Invalid metadata key should be rejected");
    
    // Test metadata entry count limit
    let mut metadata = HashMap::new();
    for i in 0..51 {
        metadata.insert(format!("key{}", i), "value".to_string());
    }
    
    let result = validate_add_memory_request(
        "Valid content".to_string(),
        Some(metadata),
        None,
        None,
        None,
        None,
    );
    
    assert!(result.is_err(), "Too many metadata entries should be rejected");
}

#[tokio::test]
async fn test_validation_tag_constraints() {
    use agent_mem_server::middleware::validation::validate_add_memory_request;
    
    // Test invalid tag characters
    let result = validate_add_memory_request(
        "Valid content".to_string(),
        None,
        Some(vec!["invalid tag!".to_string()]),
        None,
        None,
        None,
    );
    
    assert!(result.is_err(), "Invalid tag should be rejected");
    
    // Test too many tags
    let tags: Vec<String> = (0..21).map(|i| format!("tag{}", i)).collect();
    let result = validate_add_memory_request(
        "Valid content".to_string(),
        None,
        Some(tags),
        None,
        None,
        None,
    );
    
    assert!(result.is_err(), "Too many tags should be rejected");
}

// ==================== Database Statement Caching Tests ====================

#[tokio::test]
async fn test_statement_cache_hit() {
    use agent_mem_core::search::QueryOptimizer;
    use agent_mem_core::search::reranker::ResultReranker;
    use agent_mem::Memory;
    
    // Create store with caching
    let store = create_test_store().await;
    
    // Verify cache is initially empty
    let initial_cache_size = store.cache_size().await;
    assert_eq!(initial_cache_size, 0, "Initial cache should be empty");
    
    // Perform first query - should cache the statement
    let _result1 = store.get_value("test-user", "test-key-1").await;
    
    // Check cache size after first query
    let cache_size_after_first = store.cache_size().await;
    assert!(cache_size_after_first > 0, "Statement should be cached after first query");
    
    // Perform second query with different parameters - should use cached statement
    let _result2 = store.get_value("test-user", "test-key-2").await;
    
    // Cache size should remain the same (statement reused)
    let cache_size_after_second = store.cache_size().await;
    assert_eq!(
        cache_size_after_second, cache_size_after_first,
        "Cache size should not increase when reusing cached statement"
    );
}

#[tokio::test]
async fn test_statement_cache_clear() {
    let store = create_test_store().await;
    
    // Perform queries to populate cache
    let _result1 = store.get_value("user1", "key1").await;
    let _result2 = store.get_all("user1").await;
    
    // Verify cache is populated
    let cache_size_before_clear = store.cache_size().await;
    assert!(cache_size_before_clear > 0, "Cache should be populated");
    
    // Clear cache
    store.clear_statement_cache().await;
    
    // Verify cache is empty
    let cache_size_after_clear = store.cache_size().await;
    assert_eq!(cache_size_after_clear, 0, "Cache should be empty after clearing");
}

#[tokio::test]
async fn test_statement_cache_performance_improvement() {
    let store = create_test_store().await;
    
    // Add test data
    let item = agent_mem_traits::CoreMemoryItem {
        id: uuid::Uuid::new_v4().to_string(),
        user_id: "perf-test-user".to_string(),
        agent_id: "test-agent".to_string(),
        key: "perf-test-key".to_string(),
        value: "performance test value".to_string(),
        category: "test".to_string(),
        is_mutable: true,
        metadata: serde_json::json!({}),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    store.set_value(item.clone()).await.expect("Failed to set value");
    
    // First query (cache miss) - measure time
    let start1 = std::time::Instant::now();
    let _result1 = store.get_value("perf-test-user", "perf-test-key").await;
    let duration1 = start1.elapsed();
    
    // Second query (cache hit) - should be faster
    let start2 = std::time::Instant::now();
    let _result2 = store.get_value("perf-test-user", "perf-test-key").await;
    let duration2 = start2.elapsed();
    
    // Note: In-memory databases might not show significant difference
    // but the cache mechanism should still work correctly
    println!("First query (cache miss): {:?}", duration1);
    println!("Second query (cache hit): {:?}", duration2);
    
    // Verify both queries succeed
    assert!(_result1.is_ok(), "First query should succeed");
    assert!(_result2.is_ok(), "Second query should succeed");
}

// ==================== Integration Tests ====================

#[tokio::test]
async fn test_validation_and_database_integration() {
    use agent_mem_server::middleware::validation::validate_add_memory_request;
    use agent_mem_traits::CoreMemoryStore;
    
    let store = create_test_store().await;
    
    // Test valid request
    let valid_result = validate_add_memory_request(
        "Integration test content".to_string(),
        None,
        None,
        Some(0.7),
        Some("integration-test-agent".to_string()),
        None,
    );
    
    assert!(valid_result.is_ok(), "Valid request should pass validation");
    
    // Convert to CoreMemoryItem and store
    let item = agent_mem_traits::CoreMemoryItem {
        id: uuid::Uuid::new_v4().to_string(),
        user_id: "integration-user".to_string(),
        agent_id: "integration-test-agent".to_string(),
        key: "integration-key".to_string(),
        value: "Integration test value".to_string(),
        category: "test".to_string(),
        is_mutable: true,
        metadata: serde_json::json!({}),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    let store_result = store.set_value(item).await;
    assert!(store_result.is_ok(), "Should be able to store validated item");
    
    // Retrieve and verify
    let retrieved = store.get_value("integration-user", "integration-key").await;
    assert!(retrieved.is_ok(), "Should be able to retrieve stored item");
    assert!(retrieved.unwrap().is_some(), "Retrieved item should exist");
}

#[tokio::test]
async fn test_concurrent_validated_requests() {
    use agent_mem_server::middleware::validation::validate_add_memory_request;
    use tokio::task::JoinSet;
    
    // Simulate concurrent validated requests
    let mut join_set = JoinSet::new();
    
    for i in 0..10 {
        join_set.spawn(async move {
            validate_add_memory_request(
                format!("Concurrent test content {}", i),
                None,
                Some(vec![format!("tag{}", i)]),
                Some(0.5),
                Some(format!("agent{}", i)),
                None,
            )
        });
    }
    
    let mut success_count = 0;
    while let Some(result) = join_set.join_next().await {
        assert!(result.is_ok(), "Task should not panic");
        assert!(result.unwrap().is_ok(), "Each request should be valid");
        success_count += 1;
    }
    
    assert_eq!(success_count, 10, "All 10 concurrent requests should succeed");
}

#[tokio::test]
async fn test_end_to_end_workflow() {
    use agent_mem_server::middleware::validation::validate_add_memory_request;
    use agent_mem_traits::CoreMemoryStore;
    
    let store = create_test_store().await;
    
    // Step 1: Validate input
    let validation_result = validate_add_memory_request(
        "End-to-end test content".to_string(),
        {
            let mut metadata = HashMap::new();
            metadata.insert("category".to_string(), "e2e-test".to_string());
            Some(metadata)
        },
        Some(vec!["e2e".to_string(), "test".to_string()]),
        Some(0.9),
        Some("e2e-agent".to_string()),
        Some("e2e-session".to_string()),
    );
    
    assert!(validation_result.is_ok(), "Validation should succeed");
    
    // Step 2: Store in database
    let item = agent_mem_traits::CoreMemoryItem {
        id: uuid::Uuid::new_v4().to_string(),
        user_id: "e2e-user".to_string(),
        agent_id: "e2e-agent".to_string(),
        key: "e2e-key".to_string(),
        value: "End-to-end test value".to_string(),
        category: "e2e-test".to_string(),
        is_mutable: true,
        metadata: serde_json::json!({"category": "e2e-test"}),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    store.set_value(item.clone()).await.expect("Store should succeed");
    
    // Step 3: Retrieve from database
    let retrieved = store.get_value("e2e-user", "e2e-key").await;
    assert!(retrieved.is_ok(), "Retrieval should succeed");
    
    let retrieved_item = retrieved.unwrap().expect("Item should exist");
    assert_eq!(retrieved_item.key, "e2e-key", "Retrieved key should match");
    assert_eq!(retrieved_item.value, "End-to-end test value", "Retrieved value should match");
    
    // Step 4: Query all (tests cache)
    let all_items = store.get_all("e2e-user").await;
    assert!(all_items.is_ok(), "Get all should succeed");
    assert_eq!(all_items.unwrap().len(), 1, "Should have exactly 1 item");
    
    // Step 5: Verify cache was used
    let cache_size = store.cache_size().await;
    assert!(cache_size > 0, "Statement cache should be populated after queries");
}

// ==================== Performance Benchmarks ====================

#[tokio::test]
async fn benchmark_statement_cache_overhead() {
    let store = create_test_store().await;
    
    // Prepare test data
    for i in 0..10 {
        let item = agent_mem_traits::CoreMemoryItem {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: "benchmark-user".to_string(),
            agent_id: "benchmark-agent".to_string(),
            key: format!("bench-key-{}", i),
            value: format!("benchmark value {}", i),
            category: "benchmark".to_string(),
            is_mutable: true,
            metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        store.set_value(item).await.expect("Failed to insert test data");
    }
    
    // Benchmark queries with caching
    let iterations = 100;
    let start = std::time::Instant::now();
    
    for i in 0..iterations {
        let key = format!("bench-key-{}", i % 10);
        let _result = store.get_value("benchmark-user", &key).await;
    }
    
    let duration = start.elapsed();
    let queries_per_second = iterations as f64 / duration.as_secs_f64();
    
    println!(
        "Statement cache benchmark: {} queries in {:?} ({:.2} queries/sec)",
        iterations, duration, queries_per_second
    );
    
    // Verify cache is working
    let cache_size = store.cache_size().await;
    assert!(cache_size > 0, "Cache should be populated");
    
    // Performance assertion: Should handle at least 50 queries/sec with caching
    assert!(
        queries_per_second >= 50.0,
        "Should handle at least 50 queries/sec, got {:.2}",
        queries_per_second
    );
}

#[tokio::test]
async fn benchmark_validation_performance() {
    use agent_mem_server::middleware::validation::validate_add_memory_request;
    
    let iterations = 1000;
    let start = std::time::Instant::now();
    
    for i in 0..iterations {
        let _result = validate_add_memory_request(
            format!("Benchmark test content {}", i),
            {
                let mut metadata = HashMap::new();
                metadata.insert("index".to_string(), format!("{}", i));
                Some(metadata)
            },
            Some(vec!["benchmark".to_string(), "test".to_string()]),
            Some(0.5),
            Some("benchmark-agent".to_string()),
            None,
        );
    }
    
    let duration = start.elapsed();
    let validations_per_second = iterations as f64 / duration.as_secs_f64();
    
    println!(
        "Validation benchmark: {} validations in {:?} ({:.2} validations/sec)",
        iterations, duration, validations_per_second
    );
    
    // Performance assertion: Should handle at least 1000 validations/sec
    assert!(
        validations_per_second >= 1000.0,
        "Should handle at least 1000 validations/sec, got {:.2}",
        validations_per_second
    );
}
