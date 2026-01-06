//! Mem0 Compatibility Layer Demo
//!
//! This demo shows how to use AgentMem's Mem0 compatibility layer as a drop-in
//! replacement for Mem0. The API is designed to be identical to Mem0's Python API.

use agent_mem_compat::{Mem0Client, Mem0Config};
use anyhow::Result;
use serde_json::json;
use std::collections::HashMap;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🧠 AgentMem Mem0 Compatibility Layer Demo");
    println!("==========================================");

    // Create a Mem0 client with default configuration
    let client = Mem0Client::new().await?;
    info!("✅ Mem0Client initialized successfully");

    // Demo user
    let user_id = "demo-user-123";
    let agent_id = "demo-agent";

    println!("\n📝 Adding memories...");

    // Add some memories
    let memory1_id = client
        .add(user_id, "I love pizza, especially margherita", None)
        .await?;
    println!("   ✅ Added memory 1: {}", memory1_id);

    let memory2_id = client
        .add(
            user_id,
            "My favorite programming language is Rust",
            Some(HashMap::from([
                ("category".to_string(), json!("preference")),
                ("importance".to_string(), json!(0.8)),
            ])),
        )
        .await?;
    println!("   ✅ Added memory 2: {}", memory2_id);

    let memory3_id = client
        .add(
            user_id,
            "I have a meeting with the team tomorrow at 3 PM",
            Some(HashMap::from([
                ("category".to_string(), json!("schedule")),
                ("urgent".to_string(), json!(true)),
            ])),
        )
        .await?;
    println!("   ✅ Added memory 3: {}", memory3_id);

    // Add a memory with agent_id and run_id
    let memory4_id = client
        .add_with_options(agent_mem_compat::types::AddMemoryRequest {
            memory: "The user prefers dark mode in their IDE".to_string(),
            user_id: user_id.to_string(),
            agent_id: Some(agent_id.to_string()),
            run_id: Some("session-001".to_string()),
            metadata: HashMap::from([
                ("category".to_string(), json!("ui_preference")),
                ("confidence".to_string(), json!(0.9)),
            ]),
        })
        .await?;
    println!("   ✅ Added memory 4: {}", memory4_id);

    println!("\n🔍 Searching memories...");

    // Search for food-related memories
    let food_memories = client.search("food pizza", user_id, None).await?;
    println!(
        "   🍕 Food search results: {} memories",
        food_memories.total
    );
    for memory in &food_memories.memories {
        println!("      - {}: {}", memory.id, memory.memory);
    }

    // Search for programming-related memories
    let prog_memories = client.search("programming language", user_id, None).await?;
    println!(
        "   💻 Programming search results: {} memories",
        prog_memories.total
    );
    for memory in &prog_memories.memories {
        println!("      - {}: {}", memory.id, memory.memory);
    }

    // Search with filters
    let filtered_memories = client
        .search_with_options(agent_mem_compat::types::SearchMemoryRequest {
            query: "preference".to_string(),
            user_id: user_id.to_string(),
            filters: Some(agent_mem_compat::types::MemoryFilter {
                agent_id: Some(agent_id.to_string()),
                ..Default::default()
            }),
            limit: Some(10),
        })
        .await?;
    println!(
        "   🎯 Filtered search results: {} memories",
        filtered_memories.total
    );
    for memory in &filtered_memories.memories {
        println!("      - {}: {}", memory.id, memory.memory);
    }

    println!("\n📊 Getting memory statistics...");

    let stats = client.get_stats(user_id).await?;
    println!("   📈 User statistics:");
    for (key, value) in &stats {
        println!("      - {}: {}", key, value);
    }

    println!("\n📋 Getting all memories...");

    let all_memories = client.get_all(user_id, None).await?;
    println!("   📚 Total memories: {}", all_memories.len());
    for (i, memory) in all_memories.iter().enumerate() {
        println!("      {}. [{}] {}", i + 1, memory.id, memory.memory);
        if let Some(score) = memory.score {
            println!("         Importance: {:.2}", score);
        }
        if !memory.metadata.is_empty() {
            println!(
                "         Metadata: {}",
                serde_json::to_string_pretty(&memory.metadata)?
            );
        }
    }

    println!("\n✏️  Updating a memory...");

    let updated_memory = client
        .update(
            &memory2_id,
            user_id,
            agent_mem_compat::types::UpdateMemoryRequest {
                memory: Some(
                    "My favorite programming language is Rust - it's fast and safe!".to_string(),
                ),
                metadata: Some(HashMap::from([
                    ("category".to_string(), json!("preference")),
                    ("importance".to_string(), json!(0.9)),
                    ("updated".to_string(), json!(true)),
                ])),
            },
        )
        .await?;
    println!("   ✅ Updated memory: {}", updated_memory.memory);

    println!("\n🗑️  Deleting a memory...");

    let delete_result = client.delete(&memory3_id, user_id).await?;
    if delete_result.success {
        println!("   ✅ Deleted memory successfully");
    } else {
        warn!(
            "   ⚠️  Failed to delete memory: {:?}",
            delete_result.message
        );
    }

    println!("\n🔄 Testing configuration options...");

    // Test different configurations
    let openai_config = Mem0Config::openai();
    println!(
        "   🤖 OpenAI config: provider={}, model={}",
        openai_config.llm.provider, openai_config.llm.model
    );

    let anthropic_config = Mem0Config::anthropic();
    println!(
        "   🧠 Anthropic config: provider={}, model={}",
        anthropic_config.llm.provider, anthropic_config.llm.model
    );

    let local_config = Mem0Config::local();
    println!(
        "   🏠 Local config: provider={}, model={}",
        local_config.llm.provider, local_config.llm.model
    );

    println!("\n🧹 Cleaning up...");

    // Delete all memories for the user
    let cleanup_result = client.delete_all(user_id).await?;
    if cleanup_result.success {
        println!(
            "   ✅ Cleaned up all memories: {}",
            cleanup_result.message.unwrap_or_default()
        );
    }

    println!("\n🎉 Demo completed successfully!");
    println!("\n💡 Key Features Demonstrated:");
    println!("   • Drop-in Mem0 API compatibility");
    println!("   • Memory CRUD operations (Create, Read, Update, Delete)");
    println!("   • Semantic search with filtering");
    println!("   • Importance scoring and metadata support");
    println!("   • Multiple configuration options");
    println!("   • Statistics and analytics");
    println!("   • Session and agent tracking");

    Ok(())
}
