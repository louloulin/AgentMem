//! AgentMem Client SDK Demo
//!
//! This demo showcases the AgentMem HTTP client SDK functionality.

use agent_mem_client::{
    AddMemoryRequest, AsyncAgentMemClient, ClientConfig, SearchMemoriesRequest,
};
use agent_mem_core::MemoryType;
use anyhow::Result;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("🚀 Starting AgentMem Client SDK Demo");

    // Create client configuration
    let config = ClientConfig::new("http://localhost:8082")
        .with_timeout(std::time::Duration::from_secs(30))
        .with_max_retries(3)
        .with_logging(true);

    info!("📋 Client Configuration:");
    info!("  - Base URL: {}", config.base_url);
    info!("  - Timeout: {:?}", config.timeout);
    info!("  - Max Retries: {}", config.max_retries);
    info!("  - Logging: {}", config.enable_logging);

    // Create client
    let client = AsyncAgentMemClient::new(config)?;
    info!("✅ Client created successfully");

    // Test health check
    info!("🔍 Testing health check...");
    match client.health_check().await {
        Ok(health) => {
            info!("✅ Health check passed: {}", health.status);
            info!("   Version: {}", health.version);
            info!("   Timestamp: {}", health.timestamp);
        }
        Err(e) => {
            error!("❌ Health check failed: {}", e);
            info!("💡 Make sure the AgentMem server is running on http://localhost:8082");
            info!("   You can start it with: cargo run -p server-demo");
            return Ok(());
        }
    }

    // Test metrics
    info!("📊 Testing metrics endpoint...");
    match client.get_metrics().await {
        Ok(metrics) => {
            info!("✅ Metrics retrieved successfully");
            info!("   Timestamp: {}", metrics.timestamp);
            info!("   Metrics count: {}", metrics.metrics.len());
        }
        Err(e) => {
            error!("❌ Metrics failed: {}", e);
        }
    }

    // Test adding memories
    info!("💾 Testing memory operations...");

    // Create test memories
    let memories = vec![
        AddMemoryRequest::new("demo_agent", "I learned about Rust programming today")
            .with_user_id("demo_user")
            .with_memory_type(MemoryType::Episodic)
            .with_importance(0.8),
        AddMemoryRequest::new("demo_agent", "The weather is sunny and warm")
            .with_user_id("demo_user")
            .with_memory_type(MemoryType::Episodic)
            .with_importance(0.5),
        AddMemoryRequest::new("demo_agent", "I need to remember to buy groceries")
            .with_user_id("demo_user")
            .with_memory_type(MemoryType::Procedural)
            .with_importance(0.7),
    ];

    let mut memory_ids = Vec::new();

    for (i, memory_request) in memories.iter().enumerate() {
        info!("📝 Adding memory {}: {}", i + 1, memory_request.content);

        match client.add_memory(memory_request.clone()).await {
            Ok(response) => {
                info!("✅ Memory added successfully: {}", response.message);
                memory_ids.push(response.id);
            }
            Err(e) => {
                error!("❌ Failed to add memory: {}", e);
            }
        }
    }

    // Test searching memories
    if !memory_ids.is_empty() {
        info!("🔍 Testing memory search...");

        let search_queries = vec!["Rust programming", "weather", "groceries", "learning"];

        for query in search_queries {
            info!("🔎 Searching for: '{}'", query);

            let search_request = SearchMemoriesRequest::new(query)
                .with_agent_id("demo_agent")
                .with_limit(5)
                .with_threshold(0.1);

            match client.search_memories(search_request).await {
                Ok(results) => {
                    info!(
                        "✅ Search completed: {} results found",
                        results.results.len()
                    );
                    for (i, result) in results.results.iter().enumerate() {
                        info!(
                            "   {}. {} (score: {:.3})",
                            i + 1,
                            result.memory.content,
                            result.score
                        );
                    }
                }
                Err(e) => {
                    error!("❌ Search failed: {}", e);
                }
            }
        }

        // Test getting individual memories
        info!("📖 Testing individual memory retrieval...");
        for (i, memory_id) in memory_ids.iter().enumerate() {
            info!("📄 Getting memory {}: {}", i + 1, memory_id);

            match client.get_memory(memory_id).await {
                Ok(memory) => {
                    info!("✅ Memory retrieved: {}", memory.content);
                    info!("   Type: {:?}", memory.memory_type);
                    info!("   Importance: {:?}", memory.importance);
                    info!("   Created: {}", memory.created_at);
                }
                Err(e) => {
                    error!("❌ Failed to get memory: {}", e);
                }
            }
        }
    }

    info!("🎉 Client SDK demo completed successfully!");
    info!("📚 The AgentMem client SDK provides:");
    info!("   - Async and sync interfaces");
    info!("   - Automatic retry with exponential backoff");
    info!("   - Connection pooling and timeout handling");
    info!("   - Type-safe request/response models");
    info!("   - Comprehensive error handling");
    info!("   - Built-in logging and metrics");

    Ok(())
}
