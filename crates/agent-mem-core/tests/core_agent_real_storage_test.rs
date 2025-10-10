//! Test CoreAgent with real storage integration
//!
//! This test verifies that CoreAgent actually uses CoreMemoryStore
//! instead of returning mock responses.

use agent_mem_core::agents::{CoreAgent, MemoryAgent};
use agent_mem_traits::{AgentMemError, CoreMemoryItem, CoreMemoryStore, Result};
use async_trait::async_trait;
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Mock CoreMemoryStore for testing
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

    fn make_key(user_id: &str, key: &str) -> String {
        format!("{}:{}", user_id, key)
    }
}

#[async_trait]
impl CoreMemoryStore for MockCoreStore {
    async fn set_value(&self, item: CoreMemoryItem) -> Result<CoreMemoryItem> {
        let key = Self::make_key(&item.user_id, &item.key);
        self.items.lock().await.insert(key, item.clone());
        Ok(item)
    }

    async fn get_value(&self, user_id: &str, key: &str) -> Result<Option<CoreMemoryItem>> {
        let lookup_key = Self::make_key(user_id, key);
        Ok(self.items.lock().await.get(&lookup_key).cloned())
    }

    async fn get_all(&self, user_id: &str) -> Result<Vec<CoreMemoryItem>> {
        let prefix = format!("{}:", user_id);
        Ok(self
            .items
            .lock()
            .await
            .iter()
            .filter(|(k, _)| k.starts_with(&prefix))
            .map(|(_, v)| v.clone())
            .collect())
    }

    async fn get_by_category(
        &self,
        user_id: &str,
        category: &str,
    ) -> Result<Vec<CoreMemoryItem>> {
        let prefix = format!("{}:", user_id);
        Ok(self
            .items
            .lock()
            .await
            .iter()
            .filter(|(k, v)| k.starts_with(&prefix) && v.category == category)
            .map(|(_, v)| v.clone())
            .collect())
    }

    async fn delete_value(&self, user_id: &str, key: &str) -> Result<bool> {
        let lookup_key = Self::make_key(user_id, key);
        Ok(self.items.lock().await.remove(&lookup_key).is_some())
    }

    async fn update_value(&self, user_id: &str, key: &str, value: &str) -> Result<bool> {
        let lookup_key = Self::make_key(user_id, key);
        let mut items = self.items.lock().await;
        if let Some(item) = items.get_mut(&lookup_key) {
            item.value = value.to_string();
            item.updated_at = Utc::now();
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[tokio::test]
async fn test_core_agent_insert_with_real_store() {
    // Create mock store
    let store = Arc::new(MockCoreStore::new());

    // Create CoreAgent with store
    let mut agent = CoreAgent::new("test-agent".to_string());
    agent.set_store(store.clone());
    agent.initialize().await.unwrap();
    agent.initialize().await.unwrap();

    // Test insert
    let params = json!({
        "label": "user_name",
        "content": "Alice",
        "block_type": "identity",
        "user_id": "user123"
    });

    let task = agent_mem_core::coordination::TaskRequest {
        task_id: "task1".to_string(),
        memory_type: agent_mem_core::types::MemoryType::Core,
        operation: "create_block".to_string(),
        parameters: params,
        priority: 1,
        timeout: None,
        retry_count: 0,
    };

    let response = agent.execute_task(task).await.unwrap();
    assert!(response.success);

    let result = response.data.unwrap();
    assert_eq!(result["success"], true);
    assert_eq!(result["label"], "user_name");
    assert_eq!(result["content"], "Alice");

    // Verify data was actually stored
    let stored_item = store.get_value("user123", "user_name").await.unwrap();
    assert!(stored_item.is_some());
    let item = stored_item.unwrap();
    assert_eq!(item.key, "user_name");
    assert_eq!(item.value, "Alice");
    assert_eq!(item.category, "identity");
}

#[tokio::test]
async fn test_core_agent_read_with_real_store() {
    // Create mock store and pre-populate data
    let store = Arc::new(MockCoreStore::new());
    let now = Utc::now();
    let item = CoreMemoryItem {
        id: "item1".to_string(),
        user_id: "user123".to_string(),
        agent_id: "test-agent".to_string(),
        key: "user_age".to_string(),
        value: "30".to_string(),
        category: "profile".to_string(),
        is_mutable: true,
        metadata: json!({}),
        created_at: now,
        updated_at: now,
    };
    store.set_value(item).await.unwrap();

    // Create CoreAgent with store
    let mut agent = CoreAgent::new("test-agent".to_string());
    agent.set_store(store.clone());
    agent.initialize().await.unwrap();

    // Test read
    let params = json!({
        "label": "user_age",
        "user_id": "user123"
    });

    let task = agent_mem_core::coordination::TaskRequest {
        task_id: "task2".to_string(),
        memory_type: agent_mem_core::types::MemoryType::Core,
        operation: "read_block".to_string(),
        parameters: params,
        priority: 1,
        timeout: None,
        retry_count: 0,
    };

    let response = agent.execute_task(task).await.unwrap();
    assert!(response.success);

    let result = response.data.unwrap();
    assert_eq!(result["success"], true);
    assert_eq!(result["block"]["label"], "user_age");
    assert_eq!(result["block"]["content"], "30");
    assert_eq!(result["block"]["block_type"], "profile");
}

#[tokio::test]
async fn test_core_agent_update_with_real_store() {
    // Create mock store and pre-populate data
    let store = Arc::new(MockCoreStore::new());
    let now = Utc::now();
    let item = CoreMemoryItem {
        id: "item1".to_string(),
        user_id: "user123".to_string(),
        agent_id: "test-agent".to_string(),
        key: "user_city".to_string(),
        value: "New York".to_string(),
        category: "location".to_string(),
        is_mutable: true,
        metadata: json!({}),
        created_at: now,
        updated_at: now,
    };
    store.set_value(item).await.unwrap();

    // Create CoreAgent with store
    let mut agent = CoreAgent::new("test-agent".to_string());
    agent.set_store(store.clone());
    agent.initialize().await.unwrap();

    // Test update
    let params = json!({
        "label": "user_city",
        "content": "San Francisco",
        "user_id": "user123"
    });

    let task = agent_mem_core::coordination::TaskRequest {
        task_id: "task3".to_string(),
        memory_type: agent_mem_core::types::MemoryType::Core,
        operation: "update_block".to_string(),
        parameters: params,
        priority: 1,
        timeout: None,
        retry_count: 0,
    };

    let response = agent.execute_task(task).await.unwrap();
    assert!(response.success);

    let result = response.data.unwrap();
    assert_eq!(result["success"], true);
    assert_eq!(result["content"], "San Francisco");

    // Verify data was actually updated
    let updated_item = store.get_value("user123", "user_city").await.unwrap();
    assert!(updated_item.is_some());
    assert_eq!(updated_item.unwrap().value, "San Francisco");
}

#[tokio::test]
async fn test_core_agent_delete_with_real_store() {
    // Create mock store and pre-populate data
    let store = Arc::new(MockCoreStore::new());
    let now = Utc::now();
    let item = CoreMemoryItem {
        id: "item1".to_string(),
        user_id: "user123".to_string(),
        agent_id: "test-agent".to_string(),
        key: "temp_data".to_string(),
        value: "temporary".to_string(),
        category: "temp".to_string(),
        is_mutable: true,
        metadata: json!({}),
        created_at: now,
        updated_at: now,
    };
    store.set_value(item).await.unwrap();

    // Create CoreAgent with store
    let mut agent = CoreAgent::new("test-agent".to_string());
    agent.set_store(store.clone());
    agent.initialize().await.unwrap();

    // Test delete
    let params = json!({
        "label": "temp_data",
        "user_id": "user123"
    });

    let task = agent_mem_core::coordination::TaskRequest {
        task_id: "task4".to_string(),
        memory_type: agent_mem_core::types::MemoryType::Core,
        operation: "delete_block".to_string(),
        parameters: params,
        priority: 1,
        timeout: None,
        retry_count: 0,
    };

    let response = agent.execute_task(task).await.unwrap();
    assert!(response.success);

    let result = response.data.unwrap();
    assert_eq!(result["success"], true);

    // Verify data was actually deleted
    let deleted_item = store.get_value("user123", "temp_data").await.unwrap();
    assert!(deleted_item.is_none());
}

#[tokio::test]
async fn test_core_agent_search_with_real_store() {
    // Create mock store and pre-populate data
    let store = Arc::new(MockCoreStore::new());
    let now = Utc::now();

    // Add multiple items
    for (key, value, category) in [
        ("name", "Alice", "identity"),
        ("age", "30", "profile"),
        ("city", "New York", "location"),
        ("hobby", "reading", "interests"),
    ] {
        let item = CoreMemoryItem {
            id: format!("item_{}", key),
            user_id: "user123".to_string(),
            agent_id: "test-agent".to_string(),
            key: key.to_string(),
            value: value.to_string(),
            category: category.to_string(),
            is_mutable: true,
            metadata: json!({}),
            created_at: now,
            updated_at: now,
        };
        store.set_value(item).await.unwrap();
    }

    // Create CoreAgent with store
    let mut agent = CoreAgent::new("test-agent".to_string());
    agent.set_store(store.clone());
    agent.initialize().await.unwrap();

    // Test search
    let params = json!({
        "query": "Alice",
        "user_id": "user123"
    });

    let task = agent_mem_core::coordination::TaskRequest {
        task_id: "task5".to_string(),
        memory_type: agent_mem_core::types::MemoryType::Core,
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
    assert!(result["total_count"].as_u64().unwrap() > 0);
    assert!(result["results"].as_array().unwrap().len() > 0);
}

