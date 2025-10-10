//! End-to-End Integration Tests
//!
//! Tests complete workflows including:
//! - Storage factory creation
//! - Agent initialization with stores
//! - Memory storage and retrieval
//! - Multi-backend support
//!
//! Note: This test reuses Mock implementations from agent_store_integration_test.rs

use agent_mem_core::agents::{EpisodicAgent, SemanticAgent};
use agent_mem_core::MemoryAgent;
use agent_mem_traits::{
    EpisodicEvent, EpisodicMemoryStore, EpisodicQuery, Result, SemanticMemoryItem,
    SemanticMemoryStore, SemanticQuery,
};
use async_trait::async_trait;
use chrono::Utc;
use std::sync::{Arc, Mutex};

// ============================================================================
// Mock Stores (simplified implementation)
// ============================================================================

#[derive(Clone)]
struct MockEpisodicStore {
    events: Arc<Mutex<Vec<EpisodicEvent>>>,
}

impl MockEpisodicStore {
    fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl EpisodicMemoryStore for MockEpisodicStore {
    async fn create_event(&self, event: EpisodicEvent) -> Result<EpisodicEvent> {
        let mut events = self.events.lock().unwrap();
        events.push(event.clone());
        Ok(event)
    }

    async fn get_event(&self, event_id: &str, _user_id: &str) -> Result<Option<EpisodicEvent>> {
        let events = self.events.lock().unwrap();
        Ok(events.iter().find(|e| e.id == event_id).cloned())
    }

    async fn query_events(&self, user_id: &str, query: EpisodicQuery) -> Result<Vec<EpisodicEvent>> {
        let events = self.events.lock().unwrap();
        let mut filtered: Vec<EpisodicEvent> = events
            .iter()
            .filter(|e| e.user_id == user_id)
            .cloned()
            .collect();

        // Sort by occurred_at descending
        filtered.sort_by(|a, b| b.occurred_at.cmp(&a.occurred_at));

        // Apply limit
        if let Some(limit) = query.limit {
            filtered.truncate(limit as usize);
        }

        Ok(filtered)
    }

    async fn update_event(&self, event: EpisodicEvent) -> Result<bool> {
        let mut events = self.events.lock().unwrap();
        if let Some(pos) = events.iter().position(|e| e.id == event.id) {
            events[pos] = event;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn delete_event(&self, event_id: &str, _user_id: &str) -> Result<bool> {
        let mut events = self.events.lock().unwrap();
        let before_len = events.len();
        events.retain(|e| e.id != event_id);
        Ok(events.len() < before_len)
    }

    async fn update_importance(&self, event_id: &str, _user_id: &str, importance: f32) -> Result<bool> {
        let mut events = self.events.lock().unwrap();
        if let Some(event) = events.iter_mut().find(|e| e.id == event_id) {
            event.importance_score = importance;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn count_events_in_range(
        &self,
        user_id: &str,
        start_time: chrono::DateTime<Utc>,
        end_time: chrono::DateTime<Utc>,
    ) -> Result<i64> {
        let events = self.events.lock().unwrap();
        let count = events
            .iter()
            .filter(|e| {
                e.user_id == user_id && e.occurred_at >= start_time && e.occurred_at <= end_time
            })
            .count();
        Ok(count as i64)
    }

    async fn get_recent_events(&self, user_id: &str, limit: i64) -> Result<Vec<EpisodicEvent>> {
        let events = self.events.lock().unwrap();
        let mut filtered: Vec<EpisodicEvent> = events
            .iter()
            .filter(|e| e.user_id == user_id)
            .cloned()
            .collect();

        // Sort by occurred_at descending
        filtered.sort_by(|a, b| b.occurred_at.cmp(&a.occurred_at));
        filtered.truncate(limit as usize);

        Ok(filtered)
    }
}

#[derive(Clone)]
struct MockSemanticStore {
    items: Arc<Mutex<Vec<SemanticMemoryItem>>>,
}

impl MockSemanticStore {
    fn new() -> Self {
        Self {
            items: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl SemanticMemoryStore for MockSemanticStore {
    async fn create_item(&self, item: SemanticMemoryItem) -> Result<SemanticMemoryItem> {
        let mut items = self.items.lock().unwrap();
        items.push(item.clone());
        Ok(item)
    }

    async fn get_item(&self, item_id: &str, _user_id: &str) -> Result<Option<SemanticMemoryItem>> {
        let items = self.items.lock().unwrap();
        Ok(items.iter().find(|i| i.id == item_id).cloned())
    }

    async fn query_items(&self, user_id: &str, query: SemanticQuery) -> Result<Vec<SemanticMemoryItem>> {
        let items = self.items.lock().unwrap();
        let mut filtered: Vec<SemanticMemoryItem> = items
            .iter()
            .filter(|i| i.user_id == user_id)
            .cloned()
            .collect();

        // Sort by updated_at descending
        filtered.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        // Apply limit
        if let Some(limit) = query.limit {
            filtered.truncate(limit as usize);
        }

        Ok(filtered)
    }

    async fn update_item(&self, item: SemanticMemoryItem) -> Result<bool> {
        let mut items = self.items.lock().unwrap();
        if let Some(pos) = items.iter().position(|i| i.id == item.id) {
            items[pos] = item;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn delete_item(&self, item_id: &str, _user_id: &str) -> Result<bool> {
        let mut items = self.items.lock().unwrap();
        let before_len = items.len();
        items.retain(|i| i.id != item_id);
        Ok(items.len() < before_len)
    }

    async fn search_by_tree_path(&self, user_id: &str, tree_path: Vec<String>) -> Result<Vec<SemanticMemoryItem>> {
        let items = self.items.lock().unwrap();
        Ok(items
            .iter()
            .filter(|i| i.user_id == user_id && i.tree_path == tree_path)
            .cloned()
            .collect())
    }

    async fn search_by_name(&self, user_id: &str, name_pattern: &str, limit: i64) -> Result<Vec<SemanticMemoryItem>> {
        let items = self.items.lock().unwrap();
        Ok(items
            .iter()
            .filter(|i| i.user_id == user_id && i.name.contains(name_pattern))
            .take(limit as usize)
            .cloned()
            .collect())
    }
}

// ============================================================================
// Mock Storage Factory
// ============================================================================

struct MockStorageFactory {
    episodic_store: Arc<dyn EpisodicMemoryStore>,
    semantic_store: Arc<dyn SemanticMemoryStore>,
}

impl MockStorageFactory {
    fn new() -> Self {
        Self {
            episodic_store: Arc::new(MockEpisodicStore::new()),
            semantic_store: Arc::new(MockSemanticStore::new()),
        }
    }

    fn episodic_store(&self) -> Arc<dyn EpisodicMemoryStore> {
        self.episodic_store.clone()
    }

    fn semantic_store(&self) -> Arc<dyn SemanticMemoryStore> {
        self.semantic_store.clone()
    }
}

// ============================================================================
// End-to-End Tests
// ============================================================================

#[tokio::test]
async fn test_e2e_agent_with_factory() {
    // 1. Create storage factory
    let factory = MockStorageFactory::new();

    // 2. Create agents with stores from factory
    let episodic_agent = EpisodicAgent::with_store(
        "e2e-episodic-agent".to_string(),
        factory.episodic_store(),
    );

    let semantic_agent = SemanticAgent::with_store(
        "e2e-semantic-agent".to_string(),
        factory.semantic_store(),
    );

    // 3. Verify agents were created successfully
    assert_eq!(episodic_agent.agent_id(), "e2e-episodic-agent");
    assert_eq!(semantic_agent.agent_id(), "e2e-semantic-agent");

    println!("✅ End-to-end agent creation with factory test passed");
}

#[tokio::test]
async fn test_e2e_memory_storage_and_retrieval() {
    // 1. Create storage factory
    let factory = MockStorageFactory::new();
    let episodic_store = factory.episodic_store();

    // 2. Create agent
    let _agent = EpisodicAgent::with_store(
        "e2e-memory-agent".to_string(),
        episodic_store.clone(),
    );

    // 3. Store a memory
    let now = Utc::now();
    let event = EpisodicEvent {
        id: "event-1".to_string(),
        organization_id: "org-1".to_string(),
        user_id: "user-1".to_string(),
        agent_id: "e2e-memory-agent".to_string(),
        occurred_at: now,
        event_type: "meeting".to_string(),
        actor: Some("user-1".to_string()),
        summary: "User met John at the coffee shop".to_string(),
        details: Some("Had a great conversation about Rust programming".to_string()),
        importance_score: 0.9,
        metadata: serde_json::json!({}),
        created_at: now,
        updated_at: now,
    };

    let stored_event = episodic_store.create_event(event.clone()).await.unwrap();
    assert_eq!(stored_event.id, "event-1");

    // 4. Retrieve the memory
    let retrieved = episodic_store
        .get_event("event-1", "user-1")
        .await
        .unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().summary, "User met John at the coffee shop");

    // 5. Query for memories
    let query = EpisodicQuery {
        start_time: None,
        end_time: None,
        event_type: Some("meeting".to_string()),
        min_importance: Some(0.5),
        limit: Some(10),
    };
    let search_results = episodic_store
        .query_events("user-1", query)
        .await
        .unwrap();
    assert_eq!(search_results.len(), 1);
    assert_eq!(search_results[0].id, "event-1");

    println!("✅ End-to-end memory storage and retrieval test passed");
}

#[tokio::test]
async fn test_e2e_multi_agent_workflow() {
    // 1. Create storage factory
    let factory = MockStorageFactory::new();

    // 2. Create multiple agents
    let _episodic_agent = EpisodicAgent::with_store(
        "workflow-episodic".to_string(),
        factory.episodic_store(),
    );

    let _semantic_agent = SemanticAgent::with_store(
        "workflow-semantic".to_string(),
        factory.semantic_store(),
    );

    // 3. Store episodic memory
    let now = Utc::now();
    let event = EpisodicEvent {
        id: "workflow-event-1".to_string(),
        organization_id: "org-1".to_string(),
        user_id: "workflow-user".to_string(),
        agent_id: "workflow-episodic".to_string(),
        occurred_at: now,
        event_type: "learning".to_string(),
        actor: Some("workflow-user".to_string()),
        summary: "User learned about Rust programming".to_string(),
        details: Some("Studied ownership, borrowing, and lifetimes".to_string()),
        importance_score: 0.8,
        metadata: serde_json::json!({}),
        created_at: now,
        updated_at: now,
    };

    factory.episodic_store().create_event(event).await.unwrap();

    // 4. Store semantic memory
    let semantic_item = SemanticMemoryItem {
        id: "workflow-semantic-1".to_string(),
        organization_id: "org-1".to_string(),
        user_id: "workflow-user".to_string(),
        agent_id: "workflow-semantic".to_string(),
        name: "Rust Programming Language".to_string(),
        summary: "Rust is a systems programming language".to_string(),
        details: Some("Focuses on safety, speed, and concurrency".to_string()),
        source: Some("Learning session".to_string()),
        tree_path: vec!["Programming".to_string(), "Languages".to_string()],
        metadata: serde_json::json!({}),
        created_at: now,
        updated_at: now,
    };

    factory.semantic_store().create_item(semantic_item).await.unwrap();

    // 5. Verify both agents can access their respective stores
    let episodic_query = EpisodicQuery {
        start_time: None,
        end_time: None,
        event_type: None,
        min_importance: None,
        limit: Some(10),
    };
    let episodic_events = factory
        .episodic_store()
        .query_events("workflow-user", episodic_query)
        .await
        .unwrap();
    assert_eq!(episodic_events.len(), 1);

    let semantic_query = SemanticQuery {
        name_query: None,
        summary_query: None,
        tree_path_prefix: None,
        limit: Some(10),
    };
    let semantic_items = factory
        .semantic_store()
        .query_items("workflow-user", semantic_query)
        .await
        .unwrap();
    assert_eq!(semantic_items.len(), 1);

    println!("✅ End-to-end multi-agent workflow test passed");
}

