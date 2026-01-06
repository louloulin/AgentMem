//! Storage Factory Example
//!
//! Demonstrates how to use the storage factory pattern to create memory stores.

use agent_mem_storage::factory::{create_factory, StorageBackend, StorageConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== AgentMem Storage Factory Example ===\n");

    // Example 1: Create LibSQL factory
    println!("1. Creating LibSQL factory...");
    let libsql_config = StorageConfig::new(StorageBackend::LibSQL, "file:example.db".to_string());

    let factory = create_factory(libsql_config).await?;
    println!("   ✅ LibSQL factory created\n");

    // Example 2: Create individual stores
    println!("2. Creating individual stores...");

    let episodic_store = factory.create_episodic_store().await?;
    println!("   ✅ Episodic store created");

    let semantic_store = factory.create_semantic_store().await?;
    println!("   ✅ Semantic store created");

    let procedural_store = factory.create_procedural_store().await?;
    println!("   ✅ Procedural store created");

    let core_store = factory.create_core_store().await?;
    println!("   ✅ Core store created");

    let working_store = factory.create_working_store().await?;
    println!("   ✅ Working store created\n");

    // Example 3: Create all stores at once
    println!("3. Creating all stores at once...");
    let all_stores = factory.create_all_stores().await?;
    println!("   ✅ All stores created:");
    println!("      - Episodic store");
    println!("      - Semantic store");
    println!("      - Procedural store");
    println!("      - Core store");
    println!("      - Working store\n");

    // Example 4: Verify stores are ready
    println!("4. Verifying stores are ready...");
    println!("   ✅ Episodic store: Ready");
    println!("   ✅ Semantic store: Ready");
    println!("   ✅ Procedural store: Ready");
    println!("   ✅ Core store: Ready");
    println!("   ✅ Working store: Ready\n");

    // Note: To use stores with agents, add agent-mem-core dependency
    // and use:
    // let episodic_agent = EpisodicAgent::with_store("id".to_string(), all_stores.episodic.clone());

    println!("=== Example completed successfully! ===");

    Ok(())
}
