use agent_mem_core::{MemoryEngine, MemoryEngineConfig};
use agent_mem_traits::{Content, MemoryItem, MemoryType, MemoryV4, Session};
use chrono::Utc;
use std::collections::HashMap;
use tracing::{error, info};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("🚀 Starting AgentMem Comprehensive Test");

    // Test 1: Engine Creation and Configuration
    info!("📋 Test 1: Engine Creation and Configuration");
    let config = MemoryEngineConfig::default();
    let engine = MemoryEngine::new(config);
    info!("✅ Memory engine created successfully");

    // Test 2: Memory Creation and Addition
    info!("📋 Test 2: Memory Creation and Addition");
    let memories = create_test_memories();
    let mut memory_ids = Vec::new();

    for memory_item in memories {
        let id = memory_item.id.clone();
        let memory = MemoryV4::from_legacy_item(&memory_item);
        match engine.add_memory(memory).await {
            Ok(_) => {
                info!("✅ Added memory: {}", id);
                memory_ids.push(id);
            }
            Err(e) => {
                error!("❌ Failed to add memory {}: {}", id, e);
            }
        }
    }

    // Test 3: Memory Retrieval
    info!("📋 Test 3: Memory Retrieval");
    for id in &memory_ids {
        match engine.get_memory(id).await {
            Ok(Some(memory)) => {
                let content_str = match &memory.content {
                    Content::Text(t) => t.clone(),
                    Content::Structured(v) => v.to_string(),
                    _ => String::from("(binary content)"),
                };
                info!("✅ Retrieved memory: {} - {}", id, content_str);
            }
            Ok(None) => {
                error!("❌ Memory not found: {}", id);
            }
            Err(e) => {
                error!("❌ Failed to retrieve memory {}: {}", id, e);
            }
        }
    }

    // Test 4: Memory Update
    info!("📋 Test 4: Memory Update");
    if let Some(first_id) = memory_ids.first() {
        if let Ok(Some(mut memory)) = engine.get_memory(first_id).await {
            memory.content = Content::Text("Updated content for comprehensive test".to_string());
            memory.metadata.updated_at = Utc::now();

            match engine.update_memory(memory).await {
                Ok(_) => info!("✅ Updated memory: {}", first_id),
                Err(e) => error!("❌ Failed to update memory {}: {}", first_id, e),
            }
        }
    }

    // Test 5: Engine Statistics
    info!("📋 Test 5: Engine Statistics");
    match engine.get_statistics().await {
        Ok(stats) => {
            info!("✅ Engine Statistics:");
            info!("   Total memories: {}", stats.total_memories);
            info!("   Memories by level: {:?}", stats.memories_by_level);
            info!(
                "   Average importance by level: {:?}",
                stats.avg_importance_by_level
            );
        }
        Err(e) => {
            error!("❌ Failed to get statistics: {}", e);
        }
    }

    // Test 6: Memory Processing
    info!("📋 Test 6: Memory Processing");
    match engine.process_memories().await {
        Ok(report) => {
            info!("✅ Processing Report:");
            info!("   Total memories processed: {}", report.total_memories);
            info!("   Conflicts detected: {}", report.conflicts_detected);
            info!("   Conflicts resolved: {}", report.conflicts_resolved);
            info!("   Memories promoted: {}", report.memories_promoted);
            info!("   Memories demoted: {}", report.memories_demoted);
            info!("   Processing errors: {}", report.errors);
            info!("   Duration: {}ms", report.duration_ms);
        }
        Err(e) => {
            error!("❌ Failed to process memories: {}", e);
        }
    }

    // Test 7: Memory Deletion
    info!("📋 Test 7: Memory Deletion");
    if let Some(last_id) = memory_ids.last() {
        match engine.remove_memory(last_id).await {
            Ok(true) => info!("✅ Deleted memory: {}", last_id),
            Ok(false) => error!("❌ Memory not found for deletion: {}", last_id),
            Err(e) => error!("❌ Failed to delete memory {}: {}", last_id, e),
        }
    }

    info!("🎉 Comprehensive test completed successfully!");
    Ok(())
}

fn create_test_memories() -> Vec<MemoryItem> {
    let now = Utc::now();

    vec![
        MemoryItem {
            id: Uuid::new_v4().to_string(),
            content: "High priority task: Implement authentication system".to_string(),
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
            score: Some(0.9),
            created_at: now,
            updated_at: Some(now),
            session: Session::new(),
            memory_type: MemoryType::Procedural,
            entities: Vec::new(),
            relations: Vec::new(),
            agent_id: "test_agent".to_string(),
            user_id: Some("test_user".to_string()),
            importance: 0.9,
            embedding: None,
            last_accessed_at: now,
            access_count: 0,
            expires_at: None,
            version: 1,
        },
        MemoryItem {
            id: Uuid::new_v4().to_string(),
            content: "User John prefers dark mode interface".to_string(),
            hash: None,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert(
                    "user_id".to_string(),
                    serde_json::Value::String("john".to_string()),
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
            session: Session::new().with_user_id(Some("john".to_string())),
            memory_type: MemoryType::Episodic,
            entities: Vec::new(),
            relations: Vec::new(),
            agent_id: "test_agent".to_string(),
            user_id: Some("john".to_string()),
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
                meta.insert("verified".to_string(), serde_json::Value::Bool(true));
                meta
            },
            score: Some(0.4),
            created_at: now,
            updated_at: Some(now),
            session: Session::new(),
            memory_type: MemoryType::Semantic,
            entities: Vec::new(),
            relations: Vec::new(),
            agent_id: "test_agent".to_string(),
            user_id: None,
            importance: 0.4,
            embedding: None,
            last_accessed_at: now,
            access_count: 0,
            expires_at: None,
            version: 1,
        },
        MemoryItem {
            id: Uuid::new_v4().to_string(),
            content: "Current working on feature branch: auth-system".to_string(),
            hash: None,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert(
                    "branch".to_string(),
                    serde_json::Value::String("auth-system".to_string()),
                );
                meta.insert(
                    "status".to_string(),
                    serde_json::Value::String("active".to_string()),
                );
                meta
            },
            score: Some(0.8),
            created_at: now,
            updated_at: Some(now),
            session: Session::new().with_run_id(Some("dev-session-123".to_string())),
            memory_type: MemoryType::Working,
            entities: Vec::new(),
            relations: Vec::new(),
            agent_id: "test_agent".to_string(),
            user_id: Some("test_user".to_string()),
            importance: 0.8,
            embedding: None,
            last_accessed_at: now,
            access_count: 0,
            expires_at: None,
            version: 1,
        },
        MemoryItem {
            id: Uuid::new_v4().to_string(),
            content: "Meeting scheduled with client at 3 PM tomorrow".to_string(),
            hash: None,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert(
                    "type".to_string(),
                    serde_json::Value::String("meeting".to_string()),
                );
                meta.insert(
                    "client".to_string(),
                    serde_json::Value::String("acme-corp".to_string()),
                );
                meta
            },
            score: Some(0.7),
            created_at: now,
            updated_at: Some(now),
            session: Session::new(),
            memory_type: MemoryType::Episodic,
            entities: Vec::new(),
            relations: Vec::new(),
            agent_id: "test_agent".to_string(),
            user_id: Some("test_user".to_string()),
            importance: 0.7,
            embedding: None,
            last_accessed_at: now,
            access_count: 0,
            expires_at: None,
            version: 1,
        },
    ]
}
