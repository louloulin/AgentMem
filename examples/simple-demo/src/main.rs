//! Simple AgentMem Core Demo
//!
//! This example demonstrates basic usage of the AgentMem core functionality,
//! including memory creation, storage, and retrieval.

use agent_mem_core::{MemoryEngine, MemoryEngineConfig};
use agent_mem_traits::{MemoryItem, MemoryType, Session};
use chrono::Utc;
use std::collections::HashMap;
use tracing::{error, info};
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting AgentMem Simple Demo");

    // Create memory engine with default configuration
    let config = MemoryEngineConfig::default();
    let engine = MemoryEngine::new(config);

    info!("Memory engine initialized");

    // Create some test memories
    let memories = create_test_memories();

    // Add memories to the engine
    for memory in memories {
        match engine.add_memory(memory.clone()).await {
            Ok(id) => {
                info!("Added memory with ID: {}", id);
            }
            Err(e) => {
                error!("Failed to add memory: {}", e);
            }
        }
    }

    // Get engine statistics
    match engine.get_statistics().await {
        Ok(stats) => {
            info!("Engine Statistics:");
            info!("  Total memories: {}", stats.total_memories);
            info!("  Memories by level: {:?}", stats.memories_by_level);
            info!(
                "  Average importance by level: {:?}",
                stats.avg_importance_by_level
            );
        }
        Err(e) => {
            error!("Failed to get statistics: {}", e);
        }
    }

    // Process memories for optimization
    match engine.process_memories().await {
        Ok(report) => {
            info!("Processing Report:");
            info!("  Total memories processed: {}", report.total_memories);
            info!("  Conflicts detected: {}", report.conflicts_detected);
            info!("  Conflicts resolved: {}", report.conflicts_resolved);
            info!("  Memories promoted: {}", report.memories_promoted);
            info!("  Memories demoted: {}", report.memories_demoted);
            info!("  Processing errors: {}", report.errors);
        }
        Err(e) => {
            error!("Failed to process memories: {}", e);
        }
    }

    info!("Demo completed successfully");
    Ok(())
}

fn create_test_memories() -> Vec<MemoryItem> {
    let now = Utc::now();

    vec![
        MemoryItem {
            id: Uuid::new_v4().to_string(),
            content: "Remember to implement the new authentication system".to_string(),
            hash: None,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert(
                    "priority".to_string(),
                    serde_json::Value::String("high".to_string()),
                );
                meta.insert(
                    "category".to_string(),
                    serde_json::Value::String("development".to_string()),
                );
                meta
            },
            score: Some(0.8),
            created_at: now,
            updated_at: Some(now),
            session: Session::new(),
            memory_type: MemoryType::Procedural,
            entities: Vec::new(),
            relations: Vec::new(),
            agent_id: "demo_agent".to_string(),
            user_id: Some("demo_user".to_string()),
            importance: 0.8,
            embedding: None,
            last_accessed_at: now,
            access_count: 0,
            expires_at: None,
            version: 1,
        },
        MemoryItem {
            id: Uuid::new_v4().to_string(),
            content: "User John Doe prefers dark mode interface".to_string(),
            hash: None,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert(
                    "user_id".to_string(),
                    serde_json::Value::String("john-doe".to_string()),
                );
                meta.insert(
                    "preference_type".to_string(),
                    serde_json::Value::String("ui".to_string()),
                );
                meta
            },
            score: Some(0.6),
            created_at: now,
            updated_at: Some(now),
            session: Session::new().with_user_id(Some("john-doe".to_string())),
            memory_type: MemoryType::Episodic,
            entities: Vec::new(),
            relations: Vec::new(),
            agent_id: "demo_agent".to_string(),
            user_id: Some("john-doe".to_string()),
            importance: 0.6,
            embedding: None,
            last_accessed_at: now,
            access_count: 0,
            expires_at: None,
            version: 1,
        },
        MemoryItem {
            id: Uuid::new_v4().to_string(),
            content: "The capital of France is Paris".to_string(),
            hash: None,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert(
                    "category".to_string(),
                    serde_json::Value::String("general-knowledge".to_string()),
                );
                meta
            },
            score: Some(0.4),
            created_at: now,
            updated_at: Some(now),
            session: Session::new(),
            memory_type: MemoryType::Semantic,
            entities: Vec::new(),
            relations: Vec::new(),
            agent_id: "demo_agent".to_string(),
            user_id: None,
            importance: 0.4,
            embedding: None,
            last_accessed_at: now,
            access_count: 0,
            expires_at: None,
            version: 1,
        },
    ]
}
