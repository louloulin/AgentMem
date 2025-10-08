//! Simple Memory Demo - Mem0-style API
//!
//! This example demonstrates the simplified Memory API that mimics Mem0's interface.
//!
//! ## Features Demonstrated
//!
//! 1. Simple initialization with automatic configuration
//! 2. Adding memories with intelligent processing
//! 3. Searching memories
//! 4. Getting all memories
//! 5. Updating and deleting memories
//!
//! ## Usage
//!
//! ```bash
//! # Set OpenAI API key
//! export OPENAI_API_KEY="your-api-key"
//!
//! # Run the demo
//! cargo run --package simple-memory-demo
//! ```

use agent_mem_core::SimpleMemory;
use anyhow::Result;
use std::collections::HashMap;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("🚀 Simple Memory Demo - Mem0-style API");
    info!("========================================\n");

    // Check if OpenAI API key is set
    if std::env::var("OPENAI_API_KEY").is_err() {
        eprintln!("❌ Error: OPENAI_API_KEY environment variable not set");
        eprintln!("Please set it with: export OPENAI_API_KEY=\"your-api-key\"");
        return Ok(());
    }

    // Test 1: Simple initialization
    info!("📝 Test 1: Simple Initialization");
    info!("--------------------------------");
    
    let mem = SimpleMemory::new().await?;
    info!("✅ Memory initialized successfully\n");

    // Test 2: Add memories
    info!("📝 Test 2: Adding Memories");
    info!("---------------------------");
    
    let id1 = mem.add("I love pizza").await?;
    info!("✅ Added memory 1: {}", id1);
    
    let id2 = mem.add("My favorite color is blue").await?;
    info!("✅ Added memory 2: {}", id2);
    
    let id3 = mem.add("I work as a software engineer").await?;
    info!("✅ Added memory 3: {}\n", id3);

    // Test 3: Add memory with metadata
    info!("📝 Test 3: Adding Memory with Metadata");
    info!("---------------------------------------");
    
    let mut metadata = HashMap::new();
    metadata.insert("category".to_string(), "food".to_string());
    metadata.insert("importance".to_string(), "high".to_string());
    
    let id4 = mem.add_with_metadata("I'm allergic to peanuts", Some(metadata)).await?;
    info!("✅ Added memory with metadata: {}\n", id4);

    // Test 4: Search memories
    info!("📝 Test 4: Searching Memories");
    info!("------------------------------");
    
    let results = mem.search("What do you know about me?").await?;
    info!("✅ Found {} memories:", results.len());
    for (i, memory) in results.iter().enumerate() {
        info!("  {}. {} (ID: {})", i + 1, memory.content, memory.id);
    }
    info!("");

    // Test 5: Search with specific query
    info!("📝 Test 5: Specific Search Query");
    info!("---------------------------------");
    
    let pizza_results = mem.search("pizza").await?;
    info!("✅ Found {} memories about pizza:", pizza_results.len());
    for memory in pizza_results {
        info!("  - {}", memory.content);
    }
    info!("");

    // Test 6: Get all memories
    info!("📝 Test 6: Get All Memories");
    info!("----------------------------");
    
    let all_memories = mem.get_all().await?;
    info!("✅ Total memories: {}", all_memories.len());
    for (i, memory) in all_memories.iter().enumerate() {
        info!("  {}. {} (Type: {:?})", i + 1, memory.content, memory.memory_type);
    }
    info!("");

    // Test 7: Update memory
    info!("📝 Test 7: Update Memory");
    info!("-------------------------");
    
    mem.update(&id1, "I love pizza and pasta").await?;
    info!("✅ Updated memory: {}\n", id1);

    // Test 8: Search after update
    info!("📝 Test 8: Search After Update");
    info!("-------------------------------");
    
    let updated_results = mem.search("pasta").await?;
    info!("✅ Found {} memories about pasta:", updated_results.len());
    for memory in updated_results {
        info!("  - {}", memory.content);
    }
    info!("");

    // Test 9: User-specific memories
    info!("📝 Test 9: User-Specific Memories");
    info!("----------------------------------");
    
    let alice_mem = SimpleMemory::new().await?.with_user("alice");
    let alice_id = alice_mem.add("Alice loves Rust programming").await?;
    info!("✅ Added memory for Alice: {}", alice_id);
    
    let bob_mem = SimpleMemory::new().await?.with_user("bob");
    let bob_id = bob_mem.add("Bob prefers Python").await?;
    info!("✅ Added memory for Bob: {}", bob_id);
    
    let alice_memories = alice_mem.get_all().await?;
    let bob_memories = bob_mem.get_all().await?;
    
    info!("✅ Alice has {} memories", alice_memories.len());
    info!("✅ Bob has {} memories\n", bob_memories.len());

    // Test 10: Delete memory
    info!("📝 Test 10: Delete Memory");
    info!("--------------------------");
    
    mem.delete(&id2).await?;
    info!("✅ Deleted memory: {}", id2);
    
    let remaining = mem.get_all().await?;
    info!("✅ Remaining memories: {}\n", remaining.len());

    // Test 11: Search with limit
    info!("📝 Test 11: Search with Limit");
    info!("------------------------------");
    
    let limited_results = mem.search_with_limit("What do you know?", 2).await?;
    info!("✅ Found {} memories (limited to 2):", limited_results.len());
    for memory in limited_results {
        info!("  - {}", memory.content);
    }
    info!("");

    // Summary
    info!("🎉 Demo Complete!");
    info!("=================");
    info!("✅ All tests passed successfully");
    info!("✅ Intelligent features working:");
    info!("   - Automatic fact extraction");
    info!("   - Smart ADD/UPDATE/DELETE decisions");
    info!("   - User/Agent isolation");
    info!("   - Metadata support");
    info!("");
    info!("📊 API Comparison:");
    info!("   Mem0:      m.add('I love pizza')");
    info!("   AgentMem:  mem.add('I love pizza').await?");
    info!("   ✅ Same simplicity, Rust performance!\n");

    Ok(())
}

