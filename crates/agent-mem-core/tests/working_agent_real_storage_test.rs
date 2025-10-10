//! Integration tests for WorkingAgent with real storage

use agent_mem_core::agents::{MemoryAgent, WorkingAgent};
use agent_mem_core::coordination::TaskRequest;
use agent_mem_core::types::MemoryType;
use agent_mem_traits::{WorkingMemoryItem, WorkingMemoryStore};
use async_trait::async_trait;
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Mock implementation of WorkingMemoryStore for testing
#[derive(Clone)]
struct MockWorkingStore {
    items: Arc<Mutex<HashMap<String, WorkingMemoryItem>>>,
}

impl MockWorkingStore {
    fn new() -> Self {
        Self {
            items: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl WorkingMemoryStore for MockWorkingStore {
    async fn add_item(
        &self,
        item: WorkingMemoryItem,
    ) -> agent_mem_traits::Result<WorkingMemoryItem> {
        self.items.lock().await.insert(item.id.clone(), item.clone());
        Ok(item)
    }

    async fn get_session_items(
        &self,
        session_id: &str,
    ) -> agent_mem_traits::Result<Vec<WorkingMemoryItem>> {
        let items = self.items.lock().await;
        let results: Vec<WorkingMemoryItem> = items
            .values()
            .filter(|item| item.session_id == session_id)
            .cloned()
            .collect();
        Ok(results)
    }

    async fn remove_item(&self, item_id: &str) -> agent_mem_traits::Result<bool> {
        Ok(self.items.lock().await.remove(item_id).is_some())
    }

    async fn clear_expired(&self) -> agent_mem_traits::Result<i64> {
        let now = Utc::now();
        let mut items = self.items.lock().await;
        let before_count = items.len();
        items.retain(|_, item| {
            if let Some(expires_at) = item.expires_at {
                expires_at > now
            } else {
                true
            }
        });
        let after_count = items.len();
        Ok((before_count - after_count) as i64)
    }

    async fn clear_session(&self, session_id: &str) -> agent_mem_traits::Result<i64> {
        let mut items = self.items.lock().await;
        let before_count = items.len();
        items.retain(|_, item| item.session_id != session_id);
        let after_count = items.len();
        Ok((before_count - after_count) as i64)
    }

    async fn get_by_priority(
        &self,
        session_id: &str,
        min_priority: i32,
    ) -> agent_mem_traits::Result<Vec<WorkingMemoryItem>> {
        let items = self.items.lock().await;
        let results: Vec<WorkingMemoryItem> = items
            .values()
            .filter(|item| item.session_id == session_id && item.priority >= min_priority)
            .cloned()
            .collect();
        Ok(results)
    }
}

#[tokio::test]
async fn test_working_agent_insert_with_real_store() {
    // Create mock store
    let store = Arc::new(MockWorkingStore::new());

    // Create WorkingAgent with store
    let mut agent = WorkingAgent::new("test-working-agent".to_string());
    agent.set_store(store.clone());
    agent.initialize().await.unwrap();

    // Test insert
    let params = json!({
        "user_id": "user123",
        "session_id": "session456",
        "content": "Remember to buy milk",
        "priority": 5
    });

    let task = TaskRequest {
        task_id: "task1".to_string(),
        memory_type: MemoryType::Working,
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
    assert_eq!(result["session_id"], "session456");

    // Verify data was actually stored
    let session_items = store.get_session_items("session456").await.unwrap();
    assert_eq!(session_items.len(), 1);
    assert_eq!(session_items[0].content, "Remember to buy milk");
    assert_eq!(session_items[0].priority, 5);
}

#[tokio::test]
async fn test_working_agent_search_with_real_store() {
    // Create mock store
    let store = Arc::new(MockWorkingStore::new());

    // Pre-populate with test data
    let now = Utc::now();
    let item1 = WorkingMemoryItem {
        id: "item1".to_string(),
        user_id: "user123".to_string(),
        agent_id: "test-agent".to_string(),
        session_id: "session456".to_string(),
        content: "Task 1".to_string(),
        priority: 5,
        expires_at: None,
        metadata: json!({}),
        created_at: now,
    };
    store.add_item(item1).await.unwrap();

    let item2 = WorkingMemoryItem {
        id: "item2".to_string(),
        user_id: "user123".to_string(),
        agent_id: "test-agent".to_string(),
        session_id: "session456".to_string(),
        content: "Task 2".to_string(),
        priority: 3,
        expires_at: None,
        metadata: json!({}),
        created_at: now,
    };
    store.add_item(item2).await.unwrap();

    // Create WorkingAgent with store
    let mut agent = WorkingAgent::new("test-working-agent".to_string());
    agent.set_store(store.clone());
    agent.initialize().await.unwrap();

    // Test search
    let params = json!({
        "session_id": "session456"
    });

    let task = TaskRequest {
        task_id: "task2".to_string(),
        memory_type: MemoryType::Working,
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
    assert_eq!(result["total_count"], 2);

    let results = result["results"].as_array().unwrap();
    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn test_working_agent_delete_with_real_store() {
    // Create mock store
    let store = Arc::new(MockWorkingStore::new());

    // Pre-populate with test data
    let now = Utc::now();
    let item = WorkingMemoryItem {
        id: "item1".to_string(),
        user_id: "user123".to_string(),
        agent_id: "test-agent".to_string(),
        session_id: "session456".to_string(),
        content: "Task 1".to_string(),
        priority: 5,
        expires_at: None,
        metadata: json!({}),
        created_at: now,
    };
    store.add_item(item).await.unwrap();

    // Create WorkingAgent with store
    let mut agent = WorkingAgent::new("test-working-agent".to_string());
    agent.set_store(store.clone());
    agent.initialize().await.unwrap();

    // Test delete
    let params = json!({
        "item_id": "item1"
    });

    let task = TaskRequest {
        task_id: "task3".to_string(),
        memory_type: MemoryType::Working,
        operation: "delete".to_string(),
        parameters: params,
        priority: 1,
        timeout: None,
        retry_count: 0,
    };

    let response = agent.execute_task(task).await.unwrap();
    assert!(response.success);

    // Verify data was actually deleted
    let session_items = store.get_session_items("session456").await.unwrap();
    assert_eq!(session_items.len(), 0);
}

