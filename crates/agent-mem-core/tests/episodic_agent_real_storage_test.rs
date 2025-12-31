//! Integration tests for EpisodicAgent with real storage

use agent_mem_core::agents::{EpisodicAgent, MemoryAgent};
use agent_mem_core::coordination::TaskRequest;
use agent_mem_core::types::MemoryType;
use agent_mem_traits::{EpisodicEvent, EpisodicMemoryStore, EpisodicQuery};
use async_trait::async_trait;
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Mock implementation of EpisodicMemoryStore for testing
#[derive(Clone)]
struct MockEpisodicStore {
    events: Arc<Mutex<HashMap<String, EpisodicEvent>>>,
}

impl MockEpisodicStore {
    fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn make_key(user_id: &str, event_id: &str) -> String {
        format!("{user_id}:{event_id}")
    }
}

#[async_trait]
impl EpisodicMemoryStore for MockEpisodicStore {
    async fn create_event(&self, event: EpisodicEvent) -> agent_mem_traits::Result<EpisodicEvent> {
        let key = Self::make_key(&event.user_id, &event.id);
        self.events.lock().await.insert(key, event.clone());
        Ok(event)
    }

    async fn get_event(
        &self,
        event_id: &str,
        user_id: &str,
    ) -> agent_mem_traits::Result<Option<EpisodicEvent>> {
        let key = Self::make_key(user_id, event_id);
        Ok(self.events.lock().await.get(&key).cloned())
    }

    async fn query_events(
        &self,
        user_id: &str,
        query: EpisodicQuery,
    ) -> agent_mem_traits::Result<Vec<EpisodicEvent>> {
        let events = self.events.lock().await;
        let mut results: Vec<_> = events
            .values()
            .filter(|event| {
                event.user_id == user_id
                    && query
                        .start_time
                        .is_none_or(|start| event.occurred_at >= start)
                    && query.end_time.is_none_or(|end| event.occurred_at <= end)
                    && query
                        .event_type
                        .as_ref()
                        .is_none_or(|et| &event.event_type == et)
                    && query
                        .min_importance
                        .is_none_or(|min| event.importance_score >= min)
            })
            .cloned()
            .collect();

        // Sort by occurred_at descending
        results.sort_by(|a, b| b.occurred_at.cmp(&a.occurred_at));

        if let Some(limit) = query.limit {
            results.truncate(limit as usize);
        }

        Ok(results)
    }

    async fn update_event(&self, event: EpisodicEvent) -> agent_mem_traits::Result<bool> {
        let key = Self::make_key(&event.user_id, &event.id);
        let mut events = self.events.lock().await;
        if let std::collections::hash_map::Entry::Occupied(mut e) = events.entry(key) {
            e.insert(event);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn delete_event(&self, event_id: &str, user_id: &str) -> agent_mem_traits::Result<bool> {
        let key = Self::make_key(user_id, event_id);
        Ok(self.events.lock().await.remove(&key).is_some())
    }

    async fn update_importance(
        &self,
        event_id: &str,
        user_id: &str,
        importance_score: f32,
    ) -> agent_mem_traits::Result<bool> {
        let key = Self::make_key(user_id, event_id);
        let mut events = self.events.lock().await;
        if let Some(event) = events.get_mut(&key) {
            event.importance_score = importance_score;
            event.updated_at = Utc::now();
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
    ) -> agent_mem_traits::Result<i64> {
        let events = self.events.lock().await;
        let count = events
            .values()
            .filter(|event| {
                event.user_id == user_id
                    && event.occurred_at >= start_time
                    && event.occurred_at <= end_time
            })
            .count();
        Ok(count as i64)
    }

    async fn get_recent_events(
        &self,
        user_id: &str,
        limit: i64,
    ) -> agent_mem_traits::Result<Vec<EpisodicEvent>> {
        let events = self.events.lock().await;
        let mut user_events: Vec<_> = events
            .values()
            .filter(|event| event.user_id == user_id)
            .cloned()
            .collect();

        user_events.sort_by(|a, b| b.occurred_at.cmp(&a.occurred_at));
        user_events.truncate(limit as usize);
        Ok(user_events)
    }
}

#[tokio::test]
async fn test_episodic_agent_insert_with_real_store() {
    // Create mock store
    let store = Arc::new(MockEpisodicStore::new());

    // Create EpisodicAgent with store
    let mut agent = EpisodicAgent::new("test-episodic-agent".to_string());
    agent.set_store(store.clone());
    agent.initialize().await.unwrap();

    // Test insert
    let now = Utc::now();
    let event_id = uuid::Uuid::new_v4().to_string();
    let params = json!({
        "event": {
            "id": event_id,
            "organization_id": "org1",
            "user_id": "user123",
            "agent_id": "test-agent",
            "occurred_at": now.to_rfc3339(),
            "event_type": "meeting",
            "actor": "John Doe",
            "summary": "Team standup meeting",
            "details": "Discussed project progress",
            "importance_score": 0.8,
            "metadata": {},
            "created_at": now.to_rfc3339(),
            "updated_at": now.to_rfc3339()
        }
    });

    let task = TaskRequest {
        task_id: "task1".to_string(),
        memory_type: MemoryType::Episodic,
        operation: "insert".to_string(),
        parameters: params,
        priority: 1,
        timeout: None,
        retry_count: 0,
    };

    let response = agent.execute_task(task).await.unwrap();
    assert!(response.success);

    let result = response.data.unwrap();
    assert_eq!(result["success"], true);
    let returned_event_id = result["event_id"].as_str().unwrap();

    // Verify data was actually stored
    let stored_event = store.get_event(returned_event_id, "user123").await.unwrap();
    assert!(stored_event.is_some());
    let event = stored_event.unwrap();
    assert_eq!(event.summary, "Team standup meeting");
    assert_eq!(event.event_type, "meeting");
    assert_eq!(event.importance_score, 0.8);
}

#[tokio::test]
async fn test_episodic_agent_search_with_real_store() {
    // Create mock store
    let store = Arc::new(MockEpisodicStore::new());

    // Pre-populate with test data
    let now = Utc::now();
    let event1 = EpisodicEvent {
        id: "event1".to_string(),
        organization_id: "org1".to_string(),
        user_id: "user123".to_string(),
        agent_id: "test-agent".to_string(),
        occurred_at: now,
        event_type: "meeting".to_string(),
        actor: Some("John".to_string()),
        summary: "Team meeting".to_string(),
        details: Some("Discussed project".to_string()),
        importance_score: 0.8,
        metadata: json!({}),
        created_at: now,
        updated_at: now,
    };
    store.create_event(event1).await.unwrap();

    let event2 = EpisodicEvent {
        id: "event2".to_string(),
        organization_id: "org1".to_string(),
        user_id: "user123".to_string(),
        agent_id: "test-agent".to_string(),
        occurred_at: now,
        event_type: "task".to_string(),
        actor: Some("Jane".to_string()),
        summary: "Completed task".to_string(),
        details: Some("Finished coding".to_string()),
        importance_score: 0.6,
        metadata: json!({}),
        created_at: now,
        updated_at: now,
    };
    store.create_event(event2).await.unwrap();

    // Create EpisodicAgent with store
    let mut agent = EpisodicAgent::new("test-episodic-agent".to_string());
    agent.set_store(store.clone());
    agent.initialize().await.unwrap();

    // Test search
    let params = json!({
        "user_id": "user123",
        "event_type": "meeting",
        "limit": 10
    });

    let task = TaskRequest {
        task_id: "task2".to_string(),
        memory_type: MemoryType::Episodic,
        operation: "search".to_string(),
        parameters: params,
        priority: 1,
        timeout: None,
        retry_count: 0,
    };

    let response = agent.execute_task(task).await.unwrap();
    assert!(response.success);

    let result = response.data.unwrap();
    assert_eq!(result["success"], true);
    let results = result["results"].as_array().unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["event_type"], "meeting");
}

#[tokio::test]
async fn test_episodic_agent_update_with_real_store() {
    // Create mock store
    let store = Arc::new(MockEpisodicStore::new());

    // Pre-populate with test data
    let now = Utc::now();
    let event = EpisodicEvent {
        id: "event1".to_string(),
        organization_id: "org1".to_string(),
        user_id: "user123".to_string(),
        agent_id: "test-agent".to_string(),
        occurred_at: now,
        event_type: "meeting".to_string(),
        actor: Some("John".to_string()),
        summary: "Team meeting".to_string(),
        details: Some("Discussed project".to_string()),
        importance_score: 0.5,
        metadata: json!({}),
        created_at: now,
        updated_at: now,
    };
    store.create_event(event).await.unwrap();

    // Create EpisodicAgent with store
    let mut agent = EpisodicAgent::new("test-episodic-agent".to_string());
    agent.set_store(store.clone());
    agent.initialize().await.unwrap();

    // Test update importance
    let params = json!({
        "event_id": "event1",
        "user_id": "user123",
        "importance_score": 0.9
    });

    let task = TaskRequest {
        task_id: "task3".to_string(),
        memory_type: MemoryType::Episodic,
        operation: "update".to_string(),
        parameters: params,
        priority: 1,
        timeout: None,
        retry_count: 0,
    };

    let response = agent.execute_task(task).await.unwrap();
    assert!(response.success);

    // Verify data was actually updated
    let stored_event = store.get_event("event1", "user123").await.unwrap();
    assert!(stored_event.is_some());
    assert_eq!(stored_event.unwrap().importance_score, 0.9);
}
