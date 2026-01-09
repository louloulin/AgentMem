//! EventBus Demo - Demonstrates pub/sub event system
//!
//! This example shows how to:
//! - Create an event bus
//! - Subscribe to events
//! - Publish events
//! - Handle events with custom handlers

use agent_mem_event_bus::{EventBus, LoggingHandler};
use agent_mem_performance::telemetry::{MemoryEvent, EventType};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🚀 EventBus Demo\n");

    // Create event bus with capacity 100
    let bus = EventBus::new(100);

    // Subscribe to all events
    println!("📡 Subscribing to events...");
    let mut subscriber = bus.subscribe().await;

    // Spawn a task to handle events
    let handle = tokio::spawn(async move {
        println!("🎧 Listening for events...");
        while let Some(event) = subscriber.recv().await {
            println!("✅ Received: {:?}", event.event_type);
            if let Some(mem_id) = &event.memory_id {
                println!("   Memory ID: {}", mem_id);
            }
        }
        println!("🔚 Event stream ended");
    });

    // Publish some events
    println!("\n📤 Publishing events...\n");

    for i in 1..=5 {
        let event = MemoryEvent::new(EventType::MemoryCreated)
            .with_memory_id(format!("mem-{}", i))
            .with_user_id("user-123".to_string());

        match bus.publish(event).await {
            Ok(_) => println!("📨 Published event {}", i),
            Err(e) => println!("❌ Failed to publish event {}: {}", i, e),
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Publish different event types
    println!("\n📤 Publishing different event types...\n");

    let update_event = MemoryEvent::new(EventType::MemoryUpdated)
        .with_memory_id("mem-1".to_string());
    bus.publish(update_event).await?;

    let search_event = MemoryEvent::new(EventType::MemorySearched)
        .with_user_id("user-456".to_string());
    bus.publish(search_event).await?;

    // Wait a bit for events to be processed
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Show statistics
    println!("\n📊 Event Bus Statistics:\n");
    let stats = bus.get_stats().await;
    println!("   Events Published: {}", stats.events_published);
    println!("   Events Received: {}", stats.events_received);
    println!("   Subscribers: {}", stats.subscriber_count);
    println!("   Uptime: {:?}", bus.uptime());

    // Show history
    println!("\n📜 Event History:\n");
    let history = bus.get_history().await;
    println!("   Total events in history: {}", history.len());

    let created_events = bus.get_history_by_type(EventType::MemoryCreated).await;
    println!("   MemoryCreated events: {}", created_events.len());

    // Shutdown gracefully
    println!("\n👋 Shutting down...");
    bus.shutdown().await;
    handle.abort();

    println!("\n✅ Demo completed!");
    Ok(())
}
