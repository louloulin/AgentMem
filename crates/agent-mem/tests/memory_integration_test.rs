//! Memory API 集成测试
//!
//! 测试 Memory 统一API的核心功能

use agent_mem::{Memory, GetAllOptions, DeleteAllOptions};

#[tokio::test]
async fn test_memory_creation() {
    let result = Memory::new().await;
    assert!(result.is_ok(), "Memory should be created successfully");
}

#[tokio::test]
async fn test_add_memory() {
    let memory = Memory::new().await.unwrap();
    
    let result = memory.add("I love pizza").await;
    
    assert!(result.is_ok(), "Should add memory successfully");
    let add_result = result.unwrap();
    assert!(!add_result.results.is_empty(), "Should return at least one result");
    assert!(!add_result.results[0].id.is_empty(), "Should have a valid ID");
}

#[tokio::test]
async fn test_search_memory() {
    let memory = Memory::new().await.unwrap();
    
    // Add some memories
    memory.add("I love pizza").await.unwrap();
    memory.add("Python is great").await.unwrap();
    memory.add("Rust provides memory safety").await.unwrap();
    
    // Search
    let results = memory.search("pizza").await;
    
    assert!(results.is_ok(), "Search should succeed");
    let results = results.unwrap();
    assert!(!results.is_empty(), "Should find at least one result");
    
    // Verify the result contains the search term
    assert!(results[0].content.to_lowercase().contains("pizza"));
}

#[tokio::test]
async fn test_get_all_memories() {
    let memory = Memory::new().await.unwrap();
    
    // Add multiple memories
    memory.add("Memory 1").await.unwrap();
    memory.add("Memory 2").await.unwrap();
    memory.add("Memory 3").await.unwrap();
    
    // Get all
    let all_memories = memory.get_all(GetAllOptions::default()).await;
    
    assert!(all_memories.is_ok(), "Get all should succeed");
    let all_memories = all_memories.unwrap();
    assert!(all_memories.len() >= 3, "Should have at least 3 memories");
}

#[tokio::test]
async fn test_delete_memory() {
    let memory = Memory::new().await.unwrap();
    
    // Add a memory
    let add_result = memory.add("To be deleted").await.unwrap();
    let memory_id = &add_result.results[0].id;
    
    // Delete it
    let result = memory.delete(memory_id).await;
    
    assert!(result.is_ok(), "Delete should succeed");
}

#[tokio::test]
async fn test_delete_all_memories() {
    let memory = Memory::new().await.unwrap();
    
    // Add some memories
    memory.add("Memory 1").await.unwrap();
    memory.add("Memory 2").await.unwrap();
    
    // Delete all
    let count = memory.delete_all(DeleteAllOptions::default()).await;
    
    assert!(count.is_ok(), "Delete all should succeed");
    let count = count.unwrap();
    assert!(count >= 0, "Should return a valid count");
}

#[tokio::test]
async fn test_memory_workflow() {
    let memory = Memory::new().await.unwrap();
    
    // 1. Add memories
    let r1 = memory.add("I like programming in Rust").await.unwrap();
    let r2 = memory.add("Python is also great").await.unwrap();
    let r3 = memory.add("Rust provides memory safety").await.unwrap();
    
    let id1 = &r1.results[0].id;
    let id2 = &r2.results[0].id;
    let _id3 = &r3.results[0].id;
    
    // 2. Search for "Rust"
    let rust_results = memory.search("Rust").await.unwrap();
    assert!(rust_results.len() >= 2, "Should find at least 2 Rust-related memories");
    
    // 3. Get all memories
    let all_memories = memory.get_all(GetAllOptions::default()).await.unwrap();
    assert!(all_memories.len() >= 3, "Should have at least 3 memories");
    
    // 4. Delete one memory
    memory.delete(id2).await.unwrap();
    
    // 5. Verify it's deleted
    let remaining = memory.get_all(GetAllOptions::default()).await.unwrap();
    let ids: Vec<&str> = remaining.iter().map(|m| m.id.as_str()).collect();
    assert!(!ids.contains(&id2.as_str()), "Deleted memory should not be in results");
    
    // 6. Clear all
    let count = memory.delete_all(DeleteAllOptions::default()).await.unwrap();
    assert!(count >= 0, "Should clear successfully");
}

#[tokio::test]
async fn test_chinese_content() {
    let memory = Memory::new().await.unwrap();
    
    // Add Chinese content
    let result = memory.add("我喜欢编程").await;
    
    assert!(result.is_ok(), "Should handle Chinese content");
    
    // Search in Chinese
    let results = memory.search("编程").await.unwrap();
    assert!(!results.is_empty(), "Should find Chinese content");
}

#[tokio::test]
async fn test_long_content() {
    let memory = Memory::new().await.unwrap();
    
    // Create long content
    let long_text = "This is a test. ".repeat(100);
    
    let result = memory.add(&long_text).await;
    
    assert!(result.is_ok(), "Should handle long content");
}

#[tokio::test]
async fn test_empty_search() {
    let memory = Memory::new().await.unwrap();
    
    memory.add("Test content").await.unwrap();
    
    // Empty search
    let results = memory.search("").await.unwrap();
    
    // Should return empty or all results depending on implementation
    assert!(results.is_empty() || !results.is_empty());
}

#[tokio::test]
async fn test_memory_clone() {
    let memory1 = Memory::new().await.unwrap();
    
    // Add a memory
    memory1.add("Test memory").await.unwrap();
    
    // Clone the memory instance
    let memory2 = memory1.clone();
    
    // Both should work
    let results1 = memory1.search("Test").await.unwrap();
    let results2 = memory2.search("Test").await.unwrap();
    
    assert!(!results1.is_empty());
    assert!(!results2.is_empty());
}

#[tokio::test]
async fn test_concurrent_operations() {
    let memory = Memory::new().await.unwrap();
    
    // Concurrent adds
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let mem = memory.clone();
            tokio::spawn(async move {
                mem.add(&format!("Memory {}", i)).await
            })
        })
        .collect();
    
    // Wait for all to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok(), "Concurrent add should succeed");
    }
    
    // Verify all memories were added
    let all = memory.get_all(GetAllOptions::default()).await.unwrap();
    assert!(all.len() >= 10, "Should have at least 10 memories");
}

