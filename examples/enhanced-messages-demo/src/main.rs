//! Enhanced Messages Demo
//!
//! This demo showcases the enhanced Messages type that supports:
//! - Single string messages
//! - Structured Message objects
//! - Multiple string messages
//! - Proper validation and conversion

use agent_mem_client::Mem5Client;
use agent_mem_traits::{Message, MessageRole, Messages};
use anyhow::Result;
use serde_json::json;
use std::collections::HashMap;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 Starting Enhanced Messages Demo");

    // Initialize Mem5Client
    let client = Mem5Client::new().await?;
    info!("✅ Mem5Client initialized successfully");

    // Test 1: Single string message
    info!("📝 Test 1: Single string message");
    let single_message = Messages::Single("I love learning new programming languages".to_string());

    let memory_id1 = client
        .add(
            single_message,
            Some("user123".to_string()),
            Some("agent456".to_string()),
            Some("session789".to_string()),
            Some(HashMap::from([
                ("category".to_string(), json!("learning")),
                ("type".to_string(), json!("preference")),
            ])),
            true,
            Some("episodic".to_string()),
            None,
        )
        .await?;

    info!("✅ Added single message memory with ID: {}", memory_id1);

    // Test 2: Structured message
    info!("📝 Test 2: Structured message");
    let structured_message = Messages::Structured(Message {
        role: MessageRole::User,
        content: "I am a senior software engineer with expertise in Rust and distributed systems"
            .to_string(),
        timestamp: None,
    });

    let memory_id2 = client
        .add(
            structured_message,
            Some("user123".to_string()),
            Some("agent456".to_string()),
            Some("session789".to_string()),
            Some(HashMap::from([
                ("category".to_string(), json!("professional")),
                ("importance".to_string(), json!("high")),
            ])),
            true,
            Some("semantic".to_string()),
            None,
        )
        .await?;

    info!("✅ Added structured message memory with ID: {}", memory_id2);

    // Test 3: Multiple messages
    info!("📝 Test 3: Multiple messages");
    let multiple_messages = Messages::Multiple(vec![
        Message::user("I work remotely from San Francisco"),
        Message::user("I prefer flexible working hours"),
        Message::user("I enjoy collaborative team environments"),
    ]);

    let memory_id3 = client
        .add(
            multiple_messages,
            Some("user123".to_string()),
            Some("agent456".to_string()),
            Some("session789".to_string()),
            Some(HashMap::from([
                ("category".to_string(), json!("work_style")),
                ("context".to_string(), json!("preferences")),
            ])),
            true,
            Some("episodic".to_string()),
            None,
        )
        .await?;

    info!("✅ Added multiple messages memory with ID: {}", memory_id3);

    // Test 4: Search for memories
    info!("🔍 Test 4: Searching for memories");
    let search_results = client
        .search(
            "programming".to_string(),
            Some("user123".to_string()),
            Some("agent456".to_string()),
            None,
            10,
            Some(HashMap::from([("category".to_string(), json!("learning"))])),
            Some(0.3),
        )
        .await?;

    info!(
        "🔍 Found {} memories related to programming:",
        search_results.len()
    );
    for (i, memory) in search_results.iter().enumerate() {
        info!(
            "  {}. ID: {}, Content: {}",
            i + 1,
            memory.id,
            memory.content
        );
    }

    // Test 5: Search for professional memories
    info!("🔍 Test 5: Searching for professional memories");
    let professional_results = client
        .search(
            "software engineer".to_string(),
            Some("user123".to_string()),
            Some("agent456".to_string()),
            None,
            10,
            Some(HashMap::from([(
                "category".to_string(),
                json!("professional"),
            )])),
            Some(0.3),
        )
        .await?;

    info!(
        "🔍 Found {} professional memories:",
        professional_results.len()
    );
    for (i, memory) in professional_results.iter().enumerate() {
        info!(
            "  {}. ID: {}, Content: {}",
            i + 1,
            memory.id,
            memory.content
        );
    }

    // Test 6: Validation tests
    info!("🧪 Test 6: Message validation");

    // Test empty single message (should fail)
    let empty_single = Messages::Single("".to_string());
    match empty_single.validate() {
        Ok(_) => warn!("❌ Empty single message validation should have failed"),
        Err(_) => info!("✅ Empty single message validation correctly failed"),
    }

    // Test empty multiple messages (should fail)
    let empty_multiple = Messages::Multiple(vec![]);
    match empty_multiple.validate() {
        Ok(_) => warn!("❌ Empty multiple messages validation should have failed"),
        Err(_) => info!("✅ Empty multiple messages validation correctly failed"),
    }

    // Test multiple messages with empty string (should fail)
    let multiple_with_empty = Messages::Multiple(vec![
        Message::user("Valid message"),
        Message::user(""),
        Message::user("Another valid message"),
    ]);
    match multiple_with_empty.validate() {
        Ok(_) => warn!("❌ Multiple messages with empty string validation should have failed"),
        Err(_) => info!("✅ Multiple messages with empty string validation correctly failed"),
    }

    // Test 7: Message conversion
    info!("🔄 Test 7: Message conversion");

    let single = Messages::Single("Test single".to_string());
    info!("Single message content: {}", single.to_content_string());
    info!(
        "Single message list length: {}",
        single.to_message_list().len()
    );

    let structured = Messages::Structured(Message::user("Test structured"));
    info!(
        "Structured message content: {}",
        structured.to_content_string()
    );
    info!(
        "Structured message list length: {}",
        structured.to_message_list().len()
    );

    let multiple = Messages::Multiple(vec![Message::user("First"), Message::user("Second")]);
    info!(
        "Multiple messages content: {}",
        multiple.to_content_string()
    );
    info!(
        "Multiple messages list length: {}",
        multiple.to_message_list().len()
    );

    // Test 8: Health check
    info!("🏥 Test 8: Health check");
    let health = client.health_check().await?;
    info!("🏥 Client health: {}", health.status);
    info!("🏥 Version: {}", health.version);
    for (component, status) in &health.checks {
        info!("  - {}: {}", component, status);
    }

    info!("🎉 Enhanced Messages Demo completed successfully!");
    Ok(())
}
