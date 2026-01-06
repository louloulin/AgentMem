//! AgentMem Server Demo
//!
//! This demo starts the AgentMem server and demonstrates basic API usage.

use agent_mem_server::{MemoryServer, ServerConfig};
use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::try_init().ok(); // Ignore if already initialized

    info!("🚀 Starting AgentMem Server Demo");

    // Create server configuration
    let mut config = ServerConfig::default();
    config.port = 8082; // Use different port for demo
    config.enable_auth = false;
    config.enable_logging = false; // Disable server logging to avoid conflicts

    info!("📋 Server Configuration:");
    info!("  - Port: {}", config.port);
    info!("  - CORS: {}", config.enable_cors);
    info!("  - Auth: {}", config.enable_auth);
    info!("  - Logging: {}", config.enable_logging);

    // Create and start server
    let server = MemoryServer::new(config).await?;

    // Start server in background
    let server_handle = tokio::spawn(async move {
        if let Err(e) = server.start().await {
            error!("Server error: {}", e);
        }
    });

    // Wait for server to start
    info!("⏳ Waiting for server to start...");
    sleep(Duration::from_secs(2)).await;

    // Test the API
    let client = Client::new();
    let base_url = "http://localhost:8082";

    info!("🔍 Testing API endpoints...");

    // Test health check
    match test_health_check(&client, base_url).await {
        Ok(_) => info!("✅ Health check passed"),
        Err(e) => error!("❌ Health check failed: {}", e),
    }

    // Test metrics endpoint
    match test_metrics(&client, base_url).await {
        Ok(_) => info!("✅ Metrics endpoint passed"),
        Err(e) => error!("❌ Metrics endpoint failed: {}", e),
    }

    // Test memory operations
    match test_memory_operations(&client, base_url).await {
        Ok(_) => info!("✅ Memory operations passed"),
        Err(e) => error!("❌ Memory operations failed: {}", e),
    }

    info!("🎉 Demo completed! Server is still running...");
    info!("📖 Visit http://localhost:8082/swagger-ui/ for API documentation");
    info!("🔍 Visit http://localhost:8082/health for health check");
    info!("📊 Visit http://localhost:8082/metrics for metrics");
    info!("Press Ctrl+C to stop the server");

    // Keep the server running
    server_handle.await?;

    Ok(())
}

async fn test_health_check(
    client: &Client,
    base_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = client.get(&format!("{}/health", base_url)).send().await?;

    if response.status().is_success() {
        let body: serde_json::Value = response.json().await?;
        info!(
            "Health check response: {}",
            serde_json::to_string_pretty(&body)?
        );
        Ok(())
    } else {
        Err(format!("Health check failed with status: {}", response.status()).into())
    }
}

async fn test_metrics(client: &Client, base_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let response = client.get(&format!("{}/metrics", base_url)).send().await?;

    if response.status().is_success() {
        let body: serde_json::Value = response.json().await?;
        info!("Metrics response: {}", serde_json::to_string_pretty(&body)?);
        Ok(())
    } else {
        Err(format!("Metrics failed with status: {}", response.status()).into())
    }
}

async fn test_memory_operations(
    client: &Client,
    base_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Test adding a memory
    let memory_request = json!({
        "agent_id": "demo_agent",
        "user_id": "demo_user",
        "content": "This is a demo memory from the server test",
        "memory_type": "Episodic",
        "importance": 0.8
    });

    info!(
        "Adding memory: {}",
        serde_json::to_string_pretty(&memory_request)?
    );

    let response = client
        .post(&format!("{}/api/v1/memories", base_url))
        .json(&memory_request)
        .send()
        .await?;

    if response.status().is_success() {
        let body: serde_json::Value = response.json().await?;
        info!(
            "Add memory response: {}",
            serde_json::to_string_pretty(&body)?
        );

        // Test searching memories
        let search_request = json!({
            "query": "demo memory",
            "agent_id": "demo_agent",
            "limit": 10
        });

        info!(
            "Searching memories: {}",
            serde_json::to_string_pretty(&search_request)?
        );

        let search_response = client
            .post(&format!("{}/api/v1/memories/search", base_url))
            .json(&search_request)
            .send()
            .await?;

        if search_response.status().is_success() {
            let search_body: serde_json::Value = search_response.json().await?;
            info!(
                "Search response: {}",
                serde_json::to_string_pretty(&search_body)?
            );
        } else {
            error!("Search failed with status: {}", search_response.status());
        }

        Ok(())
    } else {
        Err(format!("Add memory failed with status: {}", response.status()).into())
    }
}
