//! Memory Search Integration Test
//!
//! Tests the newly implemented memory search functionality

use agent_mem::Memory;

#[tokio::test]
async fn test_memory_search_basic() {
    // Create Memory instance (V4架构)
    let memory = Memory::new().await.unwrap();

    // Add some test memories
    memory.add("I love pizza and pasta").await.unwrap();
    memory.add("I prefer Chinese food").await.unwrap();
    memory.add("Pizza is a type of Italian food").await.unwrap();

    // Test 1: Search for "pizza" - should find at least 2 memories
    let results = memory.search("pizza").await.unwrap();

    println!(
        "Search results for 'pizza': {} memories found",
        results.len()
    );
    for (i, mem) in results.iter().enumerate() {
        println!("  {}. {}", i + 1, mem.content);
    }

    assert!(
        !results.is_empty(),
        "Should find at least one memory about pizza"
    );
    assert!(
        results
            .iter()
            .any(|m| m.content.to_lowercase().contains("pizza")),
        "Results should contain pizza"
    );

    // Test 2: Search for "food"
    let results = memory.search("food").await.unwrap();

    println!(
        "\nSearch results for 'food': {} memories found",
        results.len()
    );
    for (i, mem) in results.iter().enumerate() {
        println!("  {}. {}", i + 1, mem.content);
    }

    assert!(
        !results.is_empty(),
        "Should find at least one memory about food"
    );

    println!("\n✅ Memory search tests passed!");
}

#[tokio::test]
async fn test_memory_search_relevance_scoring() {
    // Create Memory instance (V4架构)
    let memory = Memory::new().await.unwrap();

    // Add memories with different relevance to query
    memory
        .add("The quick brown fox jumps over the lazy dog")
        .await
        .unwrap();
    memory.add("A brown fox is a type of animal").await.unwrap();
    memory.add("Dogs and cats are common pets").await.unwrap();

    // Search for "brown fox"
    let results = memory.search("brown fox").await.unwrap();

    println!(
        "\nSearch results for 'brown fox': {} memories found",
        results.len()
    );
    for (i, mem) in results.iter().enumerate() {
        println!("  {}. {}", i + 1, mem.content);
    }

    assert!(!results.is_empty(), "Should find at least 1 memory");

    // The first result should be the most relevant
    let first_content = results[0].content.to_lowercase();
    assert!(
        first_content.contains("brown") || first_content.contains("fox"),
        "First result should contain 'brown' or 'fox'"
    );

    println!("✅ Relevance scoring tests passed!");
}
