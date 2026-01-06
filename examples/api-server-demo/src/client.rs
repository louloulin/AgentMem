//! AgentMem API Client Demo
//!
//! This demo shows how to interact with the AgentMem REST API.

use agent_mem_traits::MemoryType;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use tracing::{error, info};

#[derive(Debug, Serialize)]
struct MemoryRequest {
    agent_id: String,
    user_id: Option<String>,
    content: String,
    memory_type: Option<MemoryType>,
    importance: Option<f32>,
    metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
struct MemoryResponse {
    id: String,
    message: String,
}

#[derive(Debug, Serialize)]
struct SearchRequest {
    agent_id: Option<String>,
    user_id: Option<String>,
    query: String,
    memory_type: Option<MemoryType>,
    limit: Option<usize>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("🔗 Starting AgentMem API Client Demo");

    let client = Client::new();
    let base_url = "http://localhost:8080";

    // Test health check
    info!("🏥 Testing health check...");
    match client.get(&format!("{}/health", base_url)).send().await {
        Ok(response) => {
            if response.status().is_success() {
                let health: serde_json::Value = response.json().await?;
                info!("✅ Health check passed: {}", health);
            } else {
                error!("❌ Health check failed with status: {}", response.status());
                return Ok(());
            }
        }
        Err(e) => {
            error!("❌ Failed to connect to server: {}", e);
            error!("💡 Make sure the server is running with: cargo run --bin server");
            return Ok(());
        }
    }

    // Test adding memories
    info!("💾 Testing memory creation...");

    let memory1 = MemoryRequest {
        agent_id: "demo-agent".to_string(),
        user_id: Some("user123".to_string()),
        content: "The user prefers coffee over tea in the morning.".to_string(),
        memory_type: Some(MemoryType::Episodic),
        importance: Some(0.8),
        metadata: Some({
            let mut map = HashMap::new();
            map.insert("category".to_string(), "preference".to_string());
            map.insert("time".to_string(), "morning".to_string());
            map
        }),
    };

    let response = client
        .post(&format!("{}/api/v1/memories", base_url))
        .json(&memory1)
        .send()
        .await?;

    if response.status().is_success() {
        let memory_response: MemoryResponse = response.json().await?;
        info!("✅ Memory created: {}", memory_response.id);

        // Test getting the memory
        info!("🔍 Testing memory retrieval...");
        let get_response = client
            .get(&format!(
                "{}/api/v1/memories/{}",
                base_url, memory_response.id
            ))
            .send()
            .await?;

        if get_response.status().is_success() {
            let memory_data: serde_json::Value = get_response.json().await?;
            info!(
                "✅ Memory retrieved: {}",
                serde_json::to_string_pretty(&memory_data)?
            );
        } else {
            error!("❌ Failed to retrieve memory: {}", get_response.status());
        }
    } else {
        error!("❌ Failed to create memory: {}", response.status());
        let error_text = response.text().await?;
        error!("Error details: {}", error_text);
    }

    // Add more memories for search testing
    info!("📚 Adding more memories for search testing...");

    let memories = vec![
        MemoryRequest {
            agent_id: "demo-agent".to_string(),
            user_id: Some("user123".to_string()),
            content: "The user is learning Rust programming language.".to_string(),
            memory_type: Some(MemoryType::Semantic),
            importance: Some(0.9),
            metadata: None,
        },
        MemoryRequest {
            agent_id: "demo-agent".to_string(),
            user_id: Some("user123".to_string()),
            content: "Remember to send the weekly report every Friday.".to_string(),
            memory_type: Some(MemoryType::Procedural),
            importance: Some(0.7),
            metadata: None,
        },
    ];

    for memory in memories {
        let response = client
            .post(&format!("{}/api/v1/memories", base_url))
            .json(&memory)
            .send()
            .await?;

        if response.status().is_success() {
            let memory_response: MemoryResponse = response.json().await?;
            info!("✅ Additional memory created: {}", memory_response.id);
        }
    }

    // Test search
    info!("🔍 Testing memory search...");

    let search_request = SearchRequest {
        agent_id: Some("demo-agent".to_string()),
        user_id: Some("user123".to_string()),
        query: "user".to_string(),
        memory_type: None,
        limit: Some(10),
    };

    let search_response = client
        .post(&format!("{}/api/v1/memories/search", base_url))
        .json(&search_request)
        .send()
        .await?;

    if search_response.status().is_success() {
        let search_results: serde_json::Value = search_response.json().await?;
        info!(
            "✅ Search completed: {}",
            serde_json::to_string_pretty(&search_results)?
        );
    } else {
        error!("❌ Search failed: {}", search_response.status());
    }

    // Test metrics
    info!("📊 Testing metrics endpoint...");
    let metrics_response = client.get(&format!("{}/metrics", base_url)).send().await?;

    if metrics_response.status().is_success() {
        let metrics: serde_json::Value = metrics_response.json().await?;
        info!(
            "✅ Metrics retrieved: {}",
            serde_json::to_string_pretty(&metrics)?
        );
    } else {
        error!("❌ Failed to get metrics: {}", metrics_response.status());
    }

    info!("🎉 API Client Demo completed successfully!");
    info!("🌐 You can also test the API manually at: http://localhost:8080/swagger-ui/");

    Ok(())
}
