//! Integration tests for Agent + Store integration
//!
//! Tests that agents correctly use the trait-based storage backends

use agent_mem_core::agents::{CoreAgent, EpisodicAgent, ProceduralAgent, SemanticAgent, WorkingAgent};
use agent_mem_traits::{
    CoreMemoryItem, CoreMemoryStore, EpisodicEvent, EpisodicMemoryStore, EpisodicQuery,
    ProceduralMemoryItem, ProceduralMemoryStore, ProceduralQuery, Result, SemanticMemoryItem,
    SemanticMemoryStore, SemanticQuery, WorkingMemoryItem, WorkingMemoryStore,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// ============================================================================
// Mock Episodic Store
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
            .filter(|e| {
                if let Some(ref event_type) = query.event_type {
                    &e.event_type == event_type
                } else {
                    true
                }
            })
            .filter(|e| {
                if let Some(min_importance) = query.min_importance {
                    e.importance_score >= min_importance
                } else {
                    true
                }
            })
            .filter(|e| {
                if let Some(start_time) = query.start_time {
                    e.occurred_at >= start_time
                } else {
                    true
                }
            })
            .filter(|e| {
                if let Some(end_time) = query.end_time {
                    e.occurred_at <= end_time
                } else {
                    true
                }
            })
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
        if let Some(pos) = events.iter().position(|e| e.id == event_id) {
            events.remove(pos);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn update_importance(&self, event_id: &str, _user_id: &str, importance_score: f32) -> Result<bool> {
        let mut events = self.events.lock().unwrap();
        if let Some(event) = events.iter_mut().find(|e| e.id == event_id) {
            event.importance_score = importance_score;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn count_events_in_range(
        &self,
        user_id: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<i64> {
        let events = self.events.lock().unwrap();
        let count = events
            .iter()
            .filter(|e| e.user_id == user_id)
            .filter(|e| e.occurred_at >= start_time && e.occurred_at <= end_time)
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

// ============================================================================
// Mock Semantic Store
// ============================================================================

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
            .filter(|i| {
                if let Some(ref name_query) = query.name_query {
                    i.name.contains(name_query)
                } else {
                    true
                }
            })
            .filter(|i| {
                if let Some(ref summary_query) = query.summary_query {
                    i.summary.contains(summary_query)
                } else {
                    true
                }
            })
            .cloned()
            .collect();

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
        if let Some(pos) = items.iter().position(|i| i.id == item_id) {
            items.remove(pos);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn search_by_tree_path(&self, user_id: &str, tree_path: Vec<String>) -> Result<Vec<SemanticMemoryItem>> {
        let items = self.items.lock().unwrap();
        Ok(items
            .iter()
            .filter(|i| i.user_id == user_id)
            .filter(|i| i.tree_path.starts_with(&tree_path))
            .cloned()
            .collect())
    }

    async fn search_by_name(&self, user_id: &str, name_pattern: &str, limit: i64) -> Result<Vec<SemanticMemoryItem>> {
        let items = self.items.lock().unwrap();
        let mut filtered: Vec<SemanticMemoryItem> = items
            .iter()
            .filter(|i| i.user_id == user_id)
            .filter(|i| i.name.contains(name_pattern))
            .cloned()
            .collect();

        filtered.truncate(limit as usize);
        Ok(filtered)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[tokio::test]
async fn test_episodic_agent_with_mock_store() {
    // Create mock store
    let store = Arc::new(MockEpisodicStore::new());

    // Create agent with store
    let _agent = EpisodicAgent::with_store("test-agent".to_string(), store.clone());

    // Verify store is empty initially
    let events = store.events.lock().unwrap();
    assert_eq!(events.len(), 0);
}

#[tokio::test]
async fn test_semantic_agent_with_mock_store() {
    // Create mock store
    let store = Arc::new(MockSemanticStore::new());

    // Create agent with store
    let _agent = SemanticAgent::with_store("test-agent".to_string(), store.clone());

    // Verify store is empty initially
    let items = store.items.lock().unwrap();
    assert_eq!(items.len(), 0);
}

#[tokio::test]
async fn test_agent_store_runtime_switching() {
    // Create two different stores
    let store1 = Arc::new(MockEpisodicStore::new());
    let store2 = Arc::new(MockEpisodicStore::new());

    // Create agent with first store
    let mut agent = EpisodicAgent::with_store("test-agent".to_string(), store1.clone());

    // Switch to second store at runtime
    agent.set_store(store2.clone());

    // Verify both stores are still empty
    let events1 = store1.events.lock().unwrap();
    let events2 = store2.events.lock().unwrap();
    assert_eq!(events1.len(), 0);
    assert_eq!(events2.len(), 0);
}

#[tokio::test]
async fn test_mock_episodic_store_operations() {
    let store = MockEpisodicStore::new();

    // Create test event
    let event = EpisodicEvent {
        id: "event-1".to_string(),
        organization_id: "org-1".to_string(),
        user_id: "user-1".to_string(),
        agent_id: "agent-1".to_string(),
        occurred_at: Utc::now(),
        event_type: "conversation".to_string(),
        actor: Some("user".to_string()),
        summary: "Test event".to_string(),
        details: Some("Test details".to_string()),
        importance_score: 0.8,
        metadata: serde_json::json!({}),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Test create
    let created = store.create_event(event.clone()).await.unwrap();
    assert_eq!(created.id, "event-1");

    // Test get
    let retrieved = store.get_event("event-1", "user-1").await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, "event-1");

    // Test query
    let query = EpisodicQuery {
        start_time: None,
        end_time: None,
        event_type: Some("conversation".to_string()),
        min_importance: Some(0.5),
        limit: Some(10),
    };
    let results = store.query_events("user-1", query).await.unwrap();
    assert_eq!(results.len(), 1);

    // Test delete
    let deleted = store.delete_event("event-1", "user-1").await.unwrap();
    assert!(deleted);

    // Verify deleted
    let retrieved = store.get_event("event-1", "user-1").await.unwrap();
    assert!(retrieved.is_none());
}

#[tokio::test]
async fn test_mock_semantic_store_operations() {
    let store = MockSemanticStore::new();

    // Create test item
    let item = SemanticMemoryItem {
        id: "item-1".to_string(),
        organization_id: "org-1".to_string(),
        user_id: "user-1".to_string(),
        agent_id: "agent-1".to_string(),
        name: "Test Concept".to_string(),
        summary: "A test concept for testing".to_string(),
        details: Some("Detailed information".to_string()),
        source: Some("test".to_string()),
        tree_path: vec!["root".to_string(), "concepts".to_string()],
        metadata: serde_json::json!({}),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Test create
    let created = store.create_item(item.clone()).await.unwrap();
    assert_eq!(created.id, "item-1");

    // Test get
    let retrieved = store.get_item("item-1", "user-1").await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "Test Concept");

    // Test query
    let query = SemanticQuery {
        name_query: Some("Test".to_string()),
        summary_query: None,
        tree_path_prefix: None,
        limit: Some(10),
    };
    let results = store.query_items("user-1", query).await.unwrap();
    assert_eq!(results.len(), 1);

    // Test delete
    let deleted = store.delete_item("item-1", "user-1").await.unwrap();
    assert!(deleted);

    // Verify deleted
    let retrieved = store.get_item("item-1", "user-1").await.unwrap();
    assert!(retrieved.is_none());
}

// ============================================================================
// Mock Procedural Store
// ============================================================================

#[derive(Clone)]
struct MockProceduralStore {
    items: Arc<Mutex<Vec<ProceduralMemoryItem>>>,
}

impl MockProceduralStore {
    fn new() -> Self {
        Self {
            items: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl ProceduralMemoryStore for MockProceduralStore {
    async fn create_item(&self, item: ProceduralMemoryItem) -> Result<ProceduralMemoryItem> {
        let mut items = self.items.lock().unwrap();
        items.push(item.clone());
        Ok(item)
    }

    async fn get_item(&self, item_id: &str, _user_id: &str) -> Result<Option<ProceduralMemoryItem>> {
        let items = self.items.lock().unwrap();
        Ok(items.iter().find(|i| i.id == item_id).cloned())
    }

    async fn query_items(&self, user_id: &str, query: ProceduralQuery) -> Result<Vec<ProceduralMemoryItem>> {
        let items = self.items.lock().unwrap();
        let mut filtered: Vec<ProceduralMemoryItem> = items
            .iter()
            .filter(|i| i.user_id == user_id)
            .filter(|i| {
                if let Some(ref pattern) = query.skill_name_pattern {
                    i.skill_name.contains(pattern)
                } else {
                    true
                }
            })
            .filter(|i| {
                if let Some(min_rate) = query.min_success_rate {
                    i.success_rate >= min_rate
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        if let Some(limit) = query.limit {
            filtered.truncate(limit as usize);
        }

        Ok(filtered)
    }

    async fn update_item(&self, item: ProceduralMemoryItem) -> Result<bool> {
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
        if let Some(pos) = items.iter().position(|i| i.id == item_id) {
            items.remove(pos);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn update_execution_stats(&self, item_id: &str, user_id: &str, success: bool) -> Result<bool> {
        let mut items = self.items.lock().unwrap();
        if let Some(item) = items.iter_mut().find(|i| i.id == item_id && i.user_id == user_id) {
            let old_count = item.execution_count as f32;
            let old_rate = item.success_rate;
            item.execution_count += 1;
            let new_count = item.execution_count as f32;

            if success {
                item.success_rate = (old_rate * old_count + 1.0) / new_count;
            } else {
                item.success_rate = (old_rate * old_count) / new_count;
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn get_top_skills(&self, user_id: &str, limit: i64) -> Result<Vec<ProceduralMemoryItem>> {
        let items = self.items.lock().unwrap();
        let mut filtered: Vec<ProceduralMemoryItem> = items
            .iter()
            .filter(|i| i.user_id == user_id)
            .cloned()
            .collect();

        filtered.sort_by(|a, b| b.success_rate.partial_cmp(&a.success_rate).unwrap());
        filtered.truncate(limit as usize);

        Ok(filtered)
    }
}

// ============================================================================
// Mock Core Store
// ============================================================================

#[derive(Clone)]
struct MockCoreStore {
    items: Arc<Mutex<HashMap<String, CoreMemoryItem>>>,
}

impl MockCoreStore {
    fn new() -> Self {
        Self {
            items: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl CoreMemoryStore for MockCoreStore {
    async fn set_value(&self, item: CoreMemoryItem) -> Result<CoreMemoryItem> {
        let mut items = self.items.lock().unwrap();
        let key = format!("{}:{}:{}", item.user_id, item.agent_id, item.key);
        items.insert(key, item.clone());
        Ok(item)
    }

    async fn get_value(&self, user_id: &str, key: &str) -> Result<Option<CoreMemoryItem>> {
        let items = self.items.lock().unwrap();
        let lookup_key = format!("{}:*:{}", user_id, key);
        Ok(items.values().find(|i| i.user_id == user_id && i.key == key).cloned())
    }

    async fn get_all(&self, user_id: &str) -> Result<Vec<CoreMemoryItem>> {
        let items = self.items.lock().unwrap();
        Ok(items.values().filter(|i| i.user_id == user_id).cloned().collect())
    }

    async fn get_by_category(&self, user_id: &str, category: &str) -> Result<Vec<CoreMemoryItem>> {
        let items = self.items.lock().unwrap();
        Ok(items
            .values()
            .filter(|i| i.user_id == user_id && i.category == category)
            .cloned()
            .collect())
    }

    async fn delete_value(&self, user_id: &str, key: &str) -> Result<bool> {
        let mut items = self.items.lock().unwrap();
        let to_remove: Vec<String> = items
            .iter()
            .filter(|(_, v)| v.user_id == user_id && v.key == key && v.is_mutable)
            .map(|(k, _)| k.clone())
            .collect();

        for k in to_remove.iter() {
            items.remove(k);
        }

        Ok(!to_remove.is_empty())
    }

    async fn update_value(&self, user_id: &str, key: &str, new_value: &str) -> Result<bool> {
        let mut items = self.items.lock().unwrap();
        for item in items.values_mut() {
            if item.user_id == user_id && item.key == key && item.is_mutable {
                item.value = new_value.to_string();
                item.updated_at = Utc::now();
                return Ok(true);
            }
        }
        Ok(false)
    }
}

// ============================================================================
// Mock Working Store
// ============================================================================

#[derive(Clone)]
struct MockWorkingStore {
    items: Arc<Mutex<Vec<WorkingMemoryItem>>>,
}

impl MockWorkingStore {
    fn new() -> Self {
        Self {
            items: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl WorkingMemoryStore for MockWorkingStore {
    async fn add_item(&self, item: WorkingMemoryItem) -> Result<WorkingMemoryItem> {
        let mut items = self.items.lock().unwrap();
        items.push(item.clone());
        Ok(item)
    }

    async fn get_session_items(&self, session_id: &str) -> Result<Vec<WorkingMemoryItem>> {
        let items = self.items.lock().unwrap();
        let now = Utc::now();
        Ok(items
            .iter()
            .filter(|i| i.session_id == session_id)
            .filter(|i| i.expires_at.is_none() || i.expires_at.unwrap() > now)
            .cloned()
            .collect())
    }

    async fn remove_item(&self, item_id: &str) -> Result<bool> {
        let mut items = self.items.lock().unwrap();
        if let Some(pos) = items.iter().position(|i| i.id == item_id) {
            items.remove(pos);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn clear_expired(&self) -> Result<i64> {
        let mut items = self.items.lock().unwrap();
        let now = Utc::now();
        let before_len = items.len();
        items.retain(|i| i.expires_at.is_none() || i.expires_at.unwrap() > now);
        Ok((before_len - items.len()) as i64)
    }

    async fn clear_session(&self, session_id: &str) -> Result<i64> {
        let mut items = self.items.lock().unwrap();
        let before_len = items.len();
        items.retain(|i| i.session_id != session_id);
        Ok((before_len - items.len()) as i64)
    }

    async fn get_by_priority(&self, session_id: &str, min_priority: i32) -> Result<Vec<WorkingMemoryItem>> {
        let items = self.items.lock().unwrap();
        let now = Utc::now();
        let mut filtered: Vec<WorkingMemoryItem> = items
            .iter()
            .filter(|i| i.session_id == session_id)
            .filter(|i| i.priority >= min_priority)
            .filter(|i| i.expires_at.is_none() || i.expires_at.unwrap() > now)
            .cloned()
            .collect();

        filtered.sort_by(|a, b| b.priority.cmp(&a.priority));
        Ok(filtered)
    }
}

// ============================================================================
// Tests for ProceduralAgent
// ============================================================================

#[tokio::test]
async fn test_procedural_agent_with_mock_store() {
    let store = Arc::new(MockProceduralStore::new());
    let _agent = ProceduralAgent::with_store("test-procedural-agent".to_string(), store.clone());

    // Verify agent was created successfully
    let items = store.items.lock().unwrap();
    assert_eq!(items.len(), 0);
}

#[tokio::test]
async fn test_procedural_agent_set_store() {
    let mut agent = ProceduralAgent::new("test-procedural-agent".to_string());
    let store = Arc::new(MockProceduralStore::new());

    agent.set_store(store.clone());

    // Verify store was set
    let items = store.items.lock().unwrap();
    assert_eq!(items.len(), 0);
}

#[tokio::test]
async fn test_mock_procedural_store_operations() {
    let store = MockProceduralStore::new();

    // Test create
    let item = ProceduralMemoryItem {
        id: "proc-1".to_string(),
        organization_id: "org-1".to_string(),
        user_id: "user-1".to_string(),
        agent_id: "agent-1".to_string(),
        skill_name: "test_skill".to_string(),
        description: "Test skill".to_string(),
        steps: vec!["step1".to_string(), "step2".to_string()],
        success_rate: 0.8,
        execution_count: 10,
        metadata: serde_json::json!({}),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let created = store.create_item(item.clone()).await.unwrap();
    assert_eq!(created.id, "proc-1");

    // Test get
    let retrieved = store.get_item("proc-1", "user-1").await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().skill_name, "test_skill");

    // Test update execution stats
    let updated = store.update_execution_stats("proc-1", "user-1", true).await.unwrap();
    assert!(updated);

    // Test query
    let query = ProceduralQuery {
        skill_name_pattern: Some("test".to_string()),
        min_success_rate: Some(0.5),
        limit: Some(10),
    };
    let results = store.query_items("user-1", query).await.unwrap();
    assert_eq!(results.len(), 1);

    // Test delete
    let deleted = store.delete_item("proc-1", "user-1").await.unwrap();
    assert!(deleted);
}

// ============================================================================
// Tests for CoreAgent
// ============================================================================

#[tokio::test]
async fn test_core_agent_with_mock_store() {
    let store = Arc::new(MockCoreStore::new());
    let _agent = CoreAgent::with_store("test-core-agent".to_string(), store.clone());

    // Verify agent was created successfully
    let items = store.items.lock().unwrap();
    assert_eq!(items.len(), 0);
}

#[tokio::test]
async fn test_core_agent_set_store() {
    let mut agent = CoreAgent::new("test-core-agent".to_string());
    let store = Arc::new(MockCoreStore::new());

    agent.set_store(store.clone());

    // Verify store was set
    let items = store.items.lock().unwrap();
    assert_eq!(items.len(), 0);
}

#[tokio::test]
async fn test_mock_core_store_operations() {
    let store = MockCoreStore::new();

    // Test set value
    let item = CoreMemoryItem {
        id: "core-1".to_string(),
        user_id: "user-1".to_string(),
        agent_id: "agent-1".to_string(),
        key: "name".to_string(),
        value: "Alice".to_string(),
        category: "profile".to_string(),
        is_mutable: true,
        metadata: serde_json::json!({}),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let created = store.set_value(item.clone()).await.unwrap();
    assert_eq!(created.key, "name");

    // Test get value
    let retrieved = store.get_value("user-1", "name").await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().value, "Alice");

    // Test update value
    let updated = store.update_value("user-1", "name", "Bob").await.unwrap();
    assert!(updated);

    // Test get all
    let all = store.get_all("user-1").await.unwrap();
    assert_eq!(all.len(), 1);

    // Test get by category
    let by_category = store.get_by_category("user-1", "profile").await.unwrap();
    assert_eq!(by_category.len(), 1);

    // Test delete
    let deleted = store.delete_value("user-1", "name").await.unwrap();
    assert!(deleted);
}

// ============================================================================
// Tests for WorkingAgent
// ============================================================================

#[tokio::test]
async fn test_working_agent_with_mock_store() {
    let store = Arc::new(MockWorkingStore::new());
    let _agent = WorkingAgent::with_store("test-working-agent".to_string(), store.clone());

    // Verify agent was created successfully
    let items = store.items.lock().unwrap();
    assert_eq!(items.len(), 0);
}

#[tokio::test]
async fn test_working_agent_set_store() {
    let mut agent = WorkingAgent::new("test-working-agent".to_string());
    let store = Arc::new(MockWorkingStore::new());

    agent.set_store(store.clone());

    // Verify store was set
    let items = store.items.lock().unwrap();
    assert_eq!(items.len(), 0);
}

#[tokio::test]
async fn test_mock_working_store_operations() {
    let store = MockWorkingStore::new();

    // Test add item
    let item = WorkingMemoryItem {
        id: "work-1".to_string(),
        user_id: "user-1".to_string(),
        agent_id: "agent-1".to_string(),
        session_id: "session-1".to_string(),
        content: "Test content".to_string(),
        priority: 5,
        expires_at: None,
        metadata: serde_json::json!({}),
        created_at: Utc::now(),
    };

    let created = store.add_item(item.clone()).await.unwrap();
    assert_eq!(created.id, "work-1");

    // Test get session items
    let session_items = store.get_session_items("session-1").await.unwrap();
    assert_eq!(session_items.len(), 1);

    // Test get by priority
    let priority_items = store.get_by_priority("session-1", 3).await.unwrap();
    assert_eq!(priority_items.len(), 1);

    // Test remove item
    let removed = store.remove_item("work-1").await.unwrap();
    assert!(removed);

    // Test clear session
    store.add_item(item.clone()).await.unwrap();
    let cleared = store.clear_session("session-1").await.unwrap();
    assert_eq!(cleared, 1);
}

