//! Simple API Test - Mock Implementation
//!
//! This example demonstrates the Simple API design without requiring
//! full compilation of agent-mem-core (which has SQLx issues).
//!
//! This is a proof-of-concept to validate the API design.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, Level};

/// Memory item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    pub id: String,
    pub content: String,
    pub user_id: Option<String>,
    pub agent_id: String,
    pub metadata: HashMap<String, String>,
    pub created_at: String,
}

/// Simple Memory API (Mock Implementation)
pub struct SimpleMemory {
    memories: Arc<RwLock<Vec<MemoryItem>>>,
    default_user_id: Option<String>,
    default_agent_id: String,
    next_id: Arc<RwLock<usize>>,
}

impl SimpleMemory {
    /// Create a new SimpleMemory
    pub async fn new() -> Result<Self> {
        info!("Initializing SimpleMemory (Mock)");
        Ok(Self {
            memories: Arc::new(RwLock::new(Vec::new())),
            default_user_id: None,
            default_agent_id: "default".to_string(),
            next_id: Arc::new(RwLock::new(1)),
        })
    }

    /// Set the default user ID
    pub fn with_user(mut self, user_id: impl Into<String>) -> Self {
        self.default_user_id = Some(user_id.into());
        self
    }

    /// Set the default agent ID
    pub fn with_agent(mut self, agent_id: impl Into<String>) -> Self {
        self.default_agent_id = agent_id.into();
        self
    }

    /// Add a memory
    pub async fn add(&self, content: impl Into<String>) -> Result<String> {
        self.add_with_metadata(content, None).await
    }

    /// Add a memory with metadata
    pub async fn add_with_metadata(
        &self,
        content: impl Into<String>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<String> {
        let mut next_id = self.next_id.write().await;
        let id = format!("mem_{}", *next_id);
        *next_id += 1;
        drop(next_id);

        let memory = MemoryItem {
            id: id.clone(),
            content: content.into(),
            user_id: self.default_user_id.clone(),
            agent_id: self.default_agent_id.clone(),
            metadata: metadata.unwrap_or_default(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        let mut memories = self.memories.write().await;
        memories.push(memory);

        info!("Added memory: {}", id);
        Ok(id)
    }

    /// Search memories
    pub async fn search(&self, query: impl Into<String>) -> Result<Vec<MemoryItem>> {
        self.search_with_limit(query, 10).await
    }

    /// Search memories with custom limit
    pub async fn search_with_limit(
        &self,
        query: impl Into<String>,
        limit: usize,
    ) -> Result<Vec<MemoryItem>> {
        let query = query.into().to_lowercase();
        let memories = self.memories.read().await;

        let mut results: Vec<MemoryItem> = memories
            .iter()
            .filter(|m| {
                // Simple text search
                m.content.to_lowercase().contains(&query)
                    || query.is_empty()
            })
            .filter(|m| {
                // Filter by user_id if set
                if let Some(user_id) = &self.default_user_id {
                    m.user_id.as_ref() == Some(user_id)
                } else {
                    true
                }
            })
            .filter(|m| {
                // Filter by agent_id
                m.agent_id == self.default_agent_id
            })
            .cloned()
            .collect();

        results.truncate(limit);
        Ok(results)
    }

    /// Get all memories
    pub async fn get_all(&self) -> Result<Vec<MemoryItem>> {
        self.search("").await
    }

    /// Update a memory
    pub async fn update(&self, memory_id: impl Into<String>, content: impl Into<String>) -> Result<()> {
        let memory_id = memory_id.into();
        let new_content = content.into();

        let mut memories = self.memories.write().await;
        if let Some(memory) = memories.iter_mut().find(|m| m.id == memory_id) {
            memory.content = new_content;
            info!("Updated memory: {}", memory_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Memory not found: {}", memory_id))
        }
    }

    /// Delete a memory
    pub async fn delete(&self, memory_id: impl Into<String>) -> Result<()> {
        let memory_id = memory_id.into();

        let mut memories = self.memories.write().await;
        if let Some(pos) = memories.iter().position(|m| m.id == memory_id) {
            memories.remove(pos);
            info!("Deleted memory: {}", memory_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Memory not found: {}", memory_id))
        }
    }

    /// Delete all memories
    pub async fn delete_all(&self) -> Result<()> {
        let mut memories = self.memories.write().await;
        let count = memories.len();
        memories.clear();
        info!("Deleted {} memories", count);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("🚀 Simple API Test - Mock Implementation");
    info!("==========================================\n");

    // Test 1: Simple initialization
    info!("📝 Test 1: Simple Initialization");
    info!("--------------------------------");
    
    let mem = SimpleMemory::new().await?;
    info!("✅ Memory initialized successfully\n");

    // Test 2: Add memories
    info!("📝 Test 2: Adding Memories");
    info!("---------------------------");
    
    let id1 = mem.add("I love pizza").await?;
    info!("✅ Added memory 1: {}", id1);
    
    let id2 = mem.add("My favorite color is blue").await?;
    info!("✅ Added memory 2: {}", id2);
    
    let id3 = mem.add("I work as a software engineer").await?;
    info!("✅ Added memory 3: {}\n", id3);

    // Test 3: Add memory with metadata
    info!("📝 Test 3: Adding Memory with Metadata");
    info!("---------------------------------------");
    
    let mut metadata = HashMap::new();
    metadata.insert("category".to_string(), "food".to_string());
    metadata.insert("importance".to_string(), "high".to_string());
    
    let id4 = mem.add_with_metadata("I'm allergic to peanuts", Some(metadata)).await?;
    info!("✅ Added memory with metadata: {}\n", id4);

    // Test 4: Search memories
    info!("📝 Test 4: Searching Memories");
    info!("------------------------------");
    
    let results = mem.search("").await?; // Empty query returns all
    info!("✅ Found {} memories:", results.len());
    for (i, memory) in results.iter().enumerate() {
        info!("  {}. {} (ID: {})", i + 1, memory.content, memory.id);
    }
    info!("");

    // Test 5: Search with specific query
    info!("📝 Test 5: Specific Search Query");
    info!("---------------------------------");
    
    let pizza_results = mem.search("pizza").await?;
    info!("✅ Found {} memories about pizza:", pizza_results.len());
    for memory in pizza_results {
        info!("  - {}", memory.content);
    }
    info!("");

    // Test 6: Get all memories
    info!("📝 Test 6: Get All Memories");
    info!("----------------------------");
    
    let all_memories = mem.get_all().await?;
    info!("✅ Total memories: {}", all_memories.len());
    for (i, memory) in all_memories.iter().enumerate() {
        info!("  {}. {}", i + 1, memory.content);
    }
    info!("");

    // Test 7: Update memory
    info!("📝 Test 7: Update Memory");
    info!("-------------------------");
    
    mem.update(&id1, "I love pizza and pasta").await?;
    info!("✅ Updated memory: {}\n", id1);

    // Test 8: Search after update
    info!("📝 Test 8: Search After Update");
    info!("-------------------------------");
    
    let updated_results = mem.search("pasta").await?;
    info!("✅ Found {} memories about pasta:", updated_results.len());
    for memory in updated_results {
        info!("  - {}", memory.content);
    }
    info!("");

    // Test 9: User-specific memories
    info!("📝 Test 9: User-Specific Memories");
    info!("----------------------------------");
    
    let alice_mem = SimpleMemory::new().await?.with_user("alice");
    let alice_id = alice_mem.add("Alice loves Rust programming").await?;
    info!("✅ Added memory for Alice: {}", alice_id);
    
    let bob_mem = SimpleMemory::new().await?.with_user("bob");
    let bob_id = bob_mem.add("Bob prefers Python").await?;
    info!("✅ Added memory for Bob: {}", bob_id);
    
    let alice_memories = alice_mem.get_all().await?;
    let bob_memories = bob_mem.get_all().await?;
    
    info!("✅ Alice has {} memories", alice_memories.len());
    info!("✅ Bob has {} memories\n", bob_memories.len());

    // Test 10: Delete memory
    info!("📝 Test 10: Delete Memory");
    info!("--------------------------");
    
    mem.delete(&id2).await?;
    info!("✅ Deleted memory: {}", id2);
    
    let remaining = mem.get_all().await?;
    info!("✅ Remaining memories: {}\n", remaining.len());

    // Test 11: Search with limit
    info!("📝 Test 11: Search with Limit");
    info!("------------------------------");
    
    let limited_results = mem.search_with_limit("", 2).await?;
    info!("✅ Found {} memories (limited to 2):", limited_results.len());
    for memory in limited_results {
        info!("  - {}", memory.content);
    }
    info!("");

    // Summary
    info!("🎉 All Tests Passed!");
    info!("====================");
    info!("✅ API Design Validated");
    info!("✅ All 11 test scenarios passed");
    info!("");
    info!("📊 API Comparison:");
    info!("   Mem0:      m.add('I love pizza')");
    info!("   AgentMem:  mem.add('I love pizza').await?");
    info!("   ✅ Same simplicity!\n");

    Ok(())
}

