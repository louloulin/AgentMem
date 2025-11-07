//! Memory API 集成测试
//!
//! 测试 Memory 统一API的核心功能（使用 FastEmbed 本地嵌入）

use agent_mem::MemoryBuilder;

/// 创建测试用的 Memory 实例
/// 使用 FastEmbed 本地嵌入（384维，无需 API key）
/// VectorStore 会自动使用与 Embedder 相同的维度
async fn create_test_memory() -> agent_mem::Memory {
    MemoryBuilder::new()
        .with_agent("test_agent")
        .with_embedder("fastembed", "all-MiniLM-L6-v2") // 384维本地模型
        .build()
        .await
        .expect("Failed to create memory with fastembed")
}

#[tokio::test]
async fn test_memory_creation() {
    let memory = create_test_memory().await;
    // Memory 创建成功
    drop(memory);
}

#[tokio::test]
async fn test_add_memory() {
    let memory = create_test_memory().await;

    let result = memory.add("I love pizza").await;

    assert!(
        result.is_ok(),
        "Should add memory successfully: {:?}",
        result.err()
    );
    let add_result = result.unwrap();
    assert!(
        !add_result.results.is_empty(),
        "Should return at least one result"
    );
    assert!(
        !add_result.results[0].id.is_empty(),
        "Should have a valid ID"
    );
}

#[tokio::test]
async fn test_search_memory() {
    let memory = create_test_memory().await;

    // Add some memories
    memory
        .add("I love pizza")
        .await
        .expect("Failed to add memory 1");
    memory
        .add("Python is great")
        .await
        .expect("Failed to add memory 2");
    memory
        .add("Rust provides memory safety")
        .await
        .expect("Failed to add memory 3");

    // Search
    let results = memory.search("pizza").await;

    assert!(
        results.is_ok(),
        "Search should succeed: {:?}",
        results.err()
    );
    let results = results.unwrap();
    assert!(!results.is_empty(), "Should find at least one result");

    // Verify the result contains the search term
    let has_pizza = results
        .iter()
        .any(|r| r.content.to_lowercase().contains("pizza"));
    assert!(has_pizza, "Results should contain 'pizza'");
}

#[tokio::test]
async fn test_get_all_memories() {
    let memory = create_test_memory().await;

    // Add multiple memories
    memory
        .add("Memory 1")
        .await
        .expect("Failed to add memory 1");
    memory
        .add("Memory 2")
        .await
        .expect("Failed to add memory 2");
    memory
        .add("Memory 3")
        .await
        .expect("Failed to add memory 3");

    // Get all
    let all_memories = memory.get_all(agent_mem::GetAllOptions::default()).await;

    assert!(
        all_memories.is_ok(),
        "Get all should succeed: {:?}",
        all_memories.err()
    );
    let all_memories = all_memories.unwrap();
    assert!(
        all_memories.len() >= 3,
        "Should have at least 3 memories, got {}",
        all_memories.len()
    );
}

#[tokio::test]
async fn test_delete_memory() {
    let memory = create_test_memory().await;

    // Add a memory
    let add_result = memory
        .add("To be deleted")
        .await
        .expect("Failed to add memory");
    let memory_id = &add_result.results[0].id;

    // Delete it
    let result = memory.delete(memory_id).await;

    assert!(result.is_ok(), "Delete should succeed: {:?}", result.err());
}

#[tokio::test]
async fn test_delete_all_memories() {
    let memory = create_test_memory().await;

    // Add some memories
    memory
        .add("Memory 1")
        .await
        .expect("Failed to add memory 1");
    memory
        .add("Memory 2")
        .await
        .expect("Failed to add memory 2");

    // Delete all
    let count = memory
        .delete_all(agent_mem::DeleteAllOptions::default())
        .await;

    assert!(
        count.is_ok(),
        "Delete all should succeed: {:?}",
        count.err()
    );
    let count = count.unwrap();
    assert!(count >= 0, "Should return a valid count");
}

#[tokio::test]
async fn test_memory_workflow() {
    let memory = create_test_memory().await;

    // 1. Add memories
    let r1 = memory
        .add("I like programming in Rust")
        .await
        .expect("Failed to add r1");
    let r2 = memory
        .add("Python is also great")
        .await
        .expect("Failed to add r2");
    let r3 = memory
        .add("Rust provides memory safety")
        .await
        .expect("Failed to add r3");

    let id1 = &r1.results[0].id;
    let id2 = &r2.results[0].id;
    let _id3 = &r3.results[0].id;

    // 2. Search for "Rust"
    let rust_results = memory.search("Rust").await.expect("Failed to search");
    assert!(
        rust_results.len() >= 2,
        "Should find at least 2 Rust-related memories"
    );

    // 3. Get all memories
    let all_memories = memory
        .get_all(agent_mem::GetAllOptions::default())
        .await
        .expect("Failed to get all");
    assert!(all_memories.len() >= 3, "Should have at least 3 memories");

    // 4. Delete one memory
    memory.delete(id2).await.expect("Failed to delete");

    // 5. Verify it's deleted
    let remaining = memory
        .get_all(agent_mem::GetAllOptions::default())
        .await
        .expect("Failed to get remaining");
    let ids: Vec<&str> = remaining.iter().map(|m| m.id.as_str()).collect();
    assert!(
        !ids.contains(&id2.as_str()),
        "Deleted memory should not be in results"
    );
    assert!(
        ids.contains(&id1.as_str()),
        "Non-deleted memory should still exist"
    );
}

#[tokio::test]
async fn test_chinese_content() {
    let memory = create_test_memory().await;

    // Add Chinese content
    let result = memory.add("我喜欢编程").await;

    assert!(
        result.is_ok(),
        "Should handle Chinese content: {:?}",
        result.err()
    );
    let add_result = result.unwrap();
    assert!(!add_result.results.is_empty());
}

#[tokio::test]
async fn test_long_content() {
    let memory = create_test_memory().await;

    // Create long content
    let long_text = "This is a test. ".repeat(100);

    let result = memory.add(&long_text).await;

    assert!(
        result.is_ok(),
        "Should handle long content: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_empty_search() {
    let memory = create_test_memory().await;

    memory
        .add("Test content")
        .await
        .expect("Failed to add test content");

    // Empty search
    let results = memory.search("").await;

    // Empty search should return empty results
    assert!(
        results.is_ok(),
        "Empty search should not crash: {:?}",
        results.err()
    );
}

#[tokio::test]
async fn test_memory_clone() {
    let memory1 = create_test_memory().await;

    // Add a memory
    memory1
        .add("Test memory")
        .await
        .expect("Failed to add test memory");

    // Clone the memory instance
    let memory2 = memory1.clone();

    // Both should work
    let results1 = memory1.search("Test").await;
    let results2 = memory2.search("Test").await;

    assert!(
        results1.is_ok(),
        "Memory1 search should work: {:?}",
        results1.err()
    );
    assert!(
        results2.is_ok(),
        "Memory2 search should work: {:?}",
        results2.err()
    );
}

#[tokio::test]
async fn test_concurrent_operations() {
    let memory = create_test_memory().await;

    // Concurrent adds
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let mem = memory.clone();
            tokio::spawn(async move { mem.add(&format!("Memory {}", i)).await })
        })
        .collect();

    // Wait for all to complete
    let mut success_count = 0;
    let mut error_count = 0;

    for handle in handles {
        match handle.await {
            Ok(Ok(_)) => success_count += 1,
            Ok(Err(e)) => {
                error_count += 1;
                eprintln!("Add failed: {:?}", e);
            }
            Err(e) => {
                error_count += 1;
                eprintln!("Task failed: {:?}", e);
            }
        }
    }

    println!(
        "Concurrent test: {} successful, {} failed",
        success_count, error_count
    );
    assert_eq!(
        success_count, 10,
        "All concurrent operations should succeed"
    );
    assert_eq!(error_count, 0, "No operations should fail");
}

#[tokio::test]
async fn test_special_characters() {
    let memory = create_test_memory().await;

    // Test various special characters
    let test_cases = vec![
        "email@example.com",
        "https://example.com",
        "Price: $99.99",
        "Math: 2 + 2 = 4",
        "Quote: \"Hello World\"",
    ];

    for case in test_cases {
        let result = memory.add(case).await;
        assert!(
            result.is_ok(),
            "Should handle special chars in: {} - {:?}",
            case,
            result.err()
        );
    }
}

#[tokio::test]
async fn test_update_memory() {
    let memory = create_test_memory().await;

    // Add a memory
    let add_result = memory.add("Original content").await.expect("Failed to add");
    let memory_id = &add_result.results[0].id;

    // Update it
    let mut update_data = std::collections::HashMap::new();
    update_data.insert("content".to_string(), serde_json::json!("Updated content"));

    let update_result = memory.update(memory_id, update_data).await;

    // Update should work
    assert!(
        update_result.is_ok(),
        "Update should succeed: {:?}",
        update_result.err()
    );
}

#[tokio::test]
async fn test_multiple_searches() {
    let memory = create_test_memory().await;

    // Add diverse content
    memory
        .add("Rust programming language")
        .await
        .expect("Failed to add");
    memory
        .add("Python data science")
        .await
        .expect("Failed to add");
    memory
        .add("JavaScript web development")
        .await
        .expect("Failed to add");

    // Search for different terms
    let rust_results = memory.search("Rust").await;
    let python_results = memory.search("Python").await;
    let web_results = memory.search("web").await;

    assert!(rust_results.is_ok(), "Rust search should work");
    assert!(python_results.is_ok(), "Python search should work");
    assert!(web_results.is_ok(), "Web search should work");

    let rust_results = rust_results.unwrap();
    let python_results = python_results.unwrap();
    let web_results = web_results.unwrap();

    assert!(!rust_results.is_empty(), "Should find Rust results");
    assert!(!python_results.is_empty(), "Should find Python results");
    assert!(!web_results.is_empty(), "Should find web results");
}

#[tokio::test]
async fn test_builder_pattern() {
    // Test custom configuration with fastembed
    let memory = MemoryBuilder::new()
        .with_agent("custom_agent")
        .with_user("custom_user")
        .with_embedder("fastembed", "all-MiniLM-L6-v2")
        .build()
        .await;

    assert!(
        memory.is_ok(),
        "Builder pattern should work: {:?}",
        memory.err()
    );
}

#[tokio::test]
async fn test_multiple_instances() {
    let mem1 = create_test_memory().await;
    let mem2 = create_test_memory().await;

    // Add to both
    mem1.add("Memory in instance 1")
        .await
        .expect("Failed to add to mem1");
    mem2.add("Memory in instance 2")
        .await
        .expect("Failed to add to mem2");

    // Both should have their own data
    let count1 = mem1
        .get_all(agent_mem::GetAllOptions::default())
        .await
        .expect("Failed to get all from mem1");
    let count2 = mem2
        .get_all(agent_mem::GetAllOptions::default())
        .await
        .expect("Failed to get all from mem2");

    assert!(count1.len() >= 1, "Instance 1 should have memories");
    assert!(count2.len() >= 1, "Instance 2 should have memories");
}
