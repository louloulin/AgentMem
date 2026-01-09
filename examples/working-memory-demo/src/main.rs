//! Working Memory Demo
//!
//! Demonstrates the WorkingMemoryService capabilities:
//! - Session-based memory isolation
//! - Priority-based retrieval
//! - Automatic expiration
//! - Statistics tracking

use agent_mem_traits::WorkingMemoryItem;
use agent_mem_working_memory::{WorkingMemoryConfig, WorkingMemoryService};
use chrono::Utc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🧠 Working Memory Demo\n");

    // Create service with custom config
    let config = WorkingMemoryConfig::default()
        .with_max_items(50)
        .with_ttl(10) // 10 seconds TTL
        .with_cleanup_interval(5); // Cleanup every 5 seconds

    println!("⚙️  Creating WorkingMemoryService...");
    let service = WorkingMemoryService::new(config).await?;
    println!("✅ Service created\n");

    // Scenario 1: Add items to a session
    println!("📝 Scenario 1: Adding conversation context\n");

    let session_id = "chat-session-123";

    // Add user message
    let user_msg = WorkingMemoryItem {
        id: String::new(),
        session_id: session_id.to_string(),
        content: "User asks about Rust memory management".to_string(),
        priority: 5,
        expires_at: None,
        created_at: Utc::now(),
        user_id: "user-456".to_string(),
        agent_id: "agent-789".to_string(),
        metadata: serde_json::json!({"type": "user_message"}),
    };
    service.add_item(user_msg).await?;

    // Add context
    let context = WorkingMemoryItem {
        id: String::new(),
        session_id: session_id.to_string(),
        content: "User is learning Rust and interested in memory safety".to_string(),
        priority: 7,
        expires_at: None,
        created_at: Utc::now(),
        user_id: "user-456".to_string(),
        agent_id: "agent-789".to_string(),
        metadata: serde_json::json!({"type": "context"}),
    };
    service.add_item(context).await?;

    println!("✅ Added 2 items to session: {}\n", session_id);

    // Scenario 2: Retrieve session items
    println!("📂 Scenario 2: Retrieving session items\n");

    let items = service.get_session_items(session_id).await?;
    println!("Found {} items:", items.len());
    for item in &items {
        println!("  - [Priority {}] {}", item.priority, item.content);
    }
    println!();

    // Scenario 3: Priority-based retrieval
    println!("🎯 Scenario 3: Priority-based retrieval\n");

    let high_priority_items = service.get_by_priority(session_id, 6).await?;
    println!("Items with priority >= 6: {}", high_priority_items.len());
    for item in &high_priority_items {
        println!("  - [Priority {}] {}", item.priority, item.content);
    }
    println!();

    // Scenario 4: Statistics
    println!("📊 Scenario 4: Service statistics\n");

    let stats = service.get_stats().await;
    println!("Total items: {}", stats.total_items);
    println!("Total sessions: {}", stats.total_sessions);
    println!("Items added: {}", stats.items_added);
    println!();

    // Scenario 5: Expiration cleanup
    println!("⏰ Scenario 5: Testing expiration\n");

    println!("Waiting for items to expire (11 seconds)...");
    tokio::time::sleep(Duration::from_secs(11)).await;

    let cleared = service.clear_expired().await?;
    println!("Cleared {} expired items", cleared);

    let items_after = service.get_session_items(session_id).await?;
    println!("Remaining items: {}", items_after.len());
    println!();

    // Final statistics
    println!("📊 Final statistics:\n");

    let final_stats = service.get_stats().await;
    println!("Total items: {}", final_stats.total_items);
    println!("Expired items cleaned: {}", final_stats.expired_items_cleaned);

    if let Some(cleanup_time) = final_stats.last_cleanup_at {
        println!("Last cleanup: {:?}", cleanup_time);
    }

    println!("\n✅ Demo completed!");
    Ok(())
}
