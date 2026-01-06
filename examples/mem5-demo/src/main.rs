//! Mem5 Enhanced AgentMem Demo
//!
//! This demo showcases the new Mem5Client with enhanced features:
//! - Full Mem0 API compatibility
//! - Batch operations
//! - Advanced search and filtering
//! - Error recovery and retry mechanisms
//! - Performance monitoring

use agent_mem_client::Mem5Client;
use agent_mem_traits::{EnhancedAddRequest, Message, Messages};
use anyhow::Result;
use serde_json::json;
use std::collections::HashMap;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("🚀 Starting Mem5 Enhanced AgentMem Demo");

    // Create a new Mem5Client
    let client = Mem5Client::new().await?;
    info!("✅ Mem5Client initialized successfully");

    // Test 1: Basic add operation with single message
    info!("📝 Test 1: Adding a single memory");
    let memory_id = client
        .add(
            Messages::Single("I love programming in Rust because it's fast and safe".to_string()),
            Some("user123".to_string()),
            Some("agent456".to_string()),
            Some("session789".to_string()),
            Some({
                let mut metadata = HashMap::new();
                metadata.insert("category".to_string(), json!("preference"));
                metadata.insert("confidence".to_string(), json!(0.9));
                metadata
            }),
            true, // infer
            Some("episodic".to_string()),
            None,
        )
        .await?;
    info!("✅ Added memory with ID: {}", memory_id);

    // Test 2: Add operation with multiple messages
    info!("📝 Test 2: Adding multiple messages as one memory");
    let memory_id2 = client
        .add(
            Messages::Multiple(vec![
                Message::user("I work as a software engineer"),
                Message::user("I specialize in backend development"),
                Message::user("I have 5 years of experience"),
            ]),
            Some("user123".to_string()),
            Some("agent456".to_string()),
            Some("session789".to_string()),
            Some({
                let mut metadata = HashMap::new();
                metadata.insert("category".to_string(), json!("professional"));
                metadata.insert("importance".to_string(), json!("high"));
                metadata
            }),
            true,
            Some("semantic".to_string()),
            None,
        )
        .await?;
    info!("✅ Added multi-message memory with ID: {}", memory_id2);

    // Test 3: Search memories
    info!("🔍 Test 3: Searching for memories");
    let search_results = client
        .search(
            "programming".to_string(),
            Some("user123".to_string()),
            Some("agent456".to_string()),
            None,
            10, // limit
            Some({
                let mut filters = HashMap::new();
                filters.insert("category".to_string(), json!("preference"));
                filters
            }),
            Some(0.5), // threshold
        )
        .await?;

    info!("🔍 Found {} memories:", search_results.len());
    for (i, memory) in search_results.iter().enumerate() {
        info!(
            "  {}. ID: {}, Content: {}, Importance: {:?}",
            i + 1,
            memory.id,
            memory.content,
            memory.importance
        );
    }

    // Test 4: Batch add operation
    info!("📦 Test 4: Batch adding memories");
    let batch_requests = vec![
        EnhancedAddRequest {
            messages: Messages::Single("I enjoy hiking on weekends".to_string()),
            user_id: Some("user123".to_string()),
            agent_id: Some("agent456".to_string()),
            run_id: Some("session789".to_string()),
            metadata: Some({
                let mut metadata = HashMap::new();
                metadata.insert("category".to_string(), json!("hobby"));
                metadata
            }),
            infer: true,
            memory_type: Some("episodic".to_string()),
            prompt: None,
        },
        EnhancedAddRequest {
            messages: Messages::Single("I prefer tea over coffee".to_string()),
            user_id: Some("user123".to_string()),
            agent_id: Some("agent456".to_string()),
            run_id: Some("session789".to_string()),
            metadata: Some({
                let mut metadata = HashMap::new();
                metadata.insert("category".to_string(), json!("preference"));
                metadata
            }),
            infer: true,
            memory_type: Some("episodic".to_string()),
            prompt: None,
        },
        EnhancedAddRequest {
            messages: Messages::Single("I live in San Francisco".to_string()),
            user_id: Some("user123".to_string()),
            agent_id: Some("agent456".to_string()),
            run_id: Some("session789".to_string()),
            metadata: Some({
                let mut metadata = HashMap::new();
                metadata.insert("category".to_string(), json!("location"));
                metadata
            }),
            infer: true,
            memory_type: Some("semantic".to_string()),
            prompt: None,
        },
    ];

    let batch_result = client.add_batch(batch_requests).await?;
    info!(
        "📦 Batch add completed: {} successful, {} failed",
        batch_result.successful, batch_result.failed
    );

    for (i, result_id) in batch_result.results.iter().enumerate() {
        info!("  {}. Added memory ID: {}", i + 1, result_id);
    }

    if !batch_result.errors.is_empty() {
        for (i, error) in batch_result.errors.iter().enumerate() {
            error!("  Error {}: {}", i + 1, error);
        }
    }

    // Test 5: Advanced search with different filters
    info!("🔍 Test 5: Advanced search with category filter");
    let hobby_search = client
        .search(
            "weekend".to_string(),
            Some("user123".to_string()),
            None,
            None,
            5,
            Some({
                let mut filters = HashMap::new();
                filters.insert("category".to_string(), json!("hobby"));
                filters
            }),
            None,
        )
        .await?;

    info!("🔍 Found {} hobby-related memories:", hobby_search.len());
    for memory in hobby_search {
        info!("  - {}", memory.content);
    }

    // Test 6: Health check
    info!("🏥 Test 6: Health check");
    let health = client.health_check().await?;
    info!("🏥 Client health: {}", health.status);
    info!("🏥 Version: {}", health.version);
    for (check, status) in health.checks {
        info!("  - {}: {}", check, status);
    }

    // Test 7: Get metrics
    info!("📊 Test 7: Getting client metrics");
    let metrics = client.get_metrics().await?;
    info!("📊 Client metrics at {}:", metrics.timestamp);
    for (metric, value) in metrics.metrics {
        info!("  - {}: {:.2}", metric, value);
    }

    info!("🎉 Mem5 Enhanced AgentMem Demo completed successfully!");
    Ok(())
}
