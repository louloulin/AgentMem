//! Integration tests for ProceduralAgent with real storage
//!
//! These tests verify that ProceduralAgent actually uses the ProceduralMemoryStore
//! instead of returning mock responses.

use agent_mem_core::agents::{MemoryAgent, ProceduralAgent};
use agent_mem_core::coordination::TaskRequest;
use agent_mem_core::types::MemoryType;
use agent_mem_traits::{ProceduralMemoryItem, ProceduralMemoryStore, ProceduralQuery};
use async_trait::async_trait;
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Mock implementation of ProceduralMemoryStore for testing
#[derive(Clone)]
struct MockProceduralStore {
    items: Arc<Mutex<HashMap<String, ProceduralMemoryItem>>>,
}

impl MockProceduralStore {
    fn new() -> Self {
        Self {
            items: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn make_key(user_id: &str, item_id: &str) -> String {
        format!("{}:{}", user_id, item_id)
    }
}

#[async_trait]
impl ProceduralMemoryStore for MockProceduralStore {
    async fn create_item(
        &self,
        item: ProceduralMemoryItem,
    ) -> agent_mem_traits::Result<ProceduralMemoryItem> {
        let key = Self::make_key(&item.user_id, &item.id);
        self.items.lock().await.insert(key, item.clone());
        Ok(item)
    }

    async fn get_item(
        &self,
        item_id: &str,
        user_id: &str,
    ) -> agent_mem_traits::Result<Option<ProceduralMemoryItem>> {
        let key = Self::make_key(user_id, item_id);
        Ok(self.items.lock().await.get(&key).cloned())
    }

    async fn query_items(
        &self,
        user_id: &str,
        query: ProceduralQuery,
    ) -> agent_mem_traits::Result<Vec<ProceduralMemoryItem>> {
        let items = self.items.lock().await;
        let mut results: Vec<ProceduralMemoryItem> = items
            .values()
            .filter(|item| {
                // Filter by user_id
                if item.user_id != user_id {
                    return false;
                }

                // Filter by skill_name_pattern if provided
                if let Some(ref pattern) = query.skill_name_pattern {
                    if !item.skill_name.contains(pattern) {
                        return false;
                    }
                }

                // Filter by min_success_rate if provided
                if let Some(min_rate) = query.min_success_rate {
                    if item.success_rate < min_rate {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();

        // Apply limit if provided
        if let Some(limit) = query.limit {
            results.truncate(limit as usize);
        }

        Ok(results)
    }

    async fn update_item(&self, item: ProceduralMemoryItem) -> agent_mem_traits::Result<bool> {
        let key = Self::make_key(&item.user_id, &item.id);
        let mut items = self.items.lock().await;
        if items.contains_key(&key) {
            items.insert(key, item);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn delete_item(&self, item_id: &str, user_id: &str) -> agent_mem_traits::Result<bool> {
        let key = Self::make_key(user_id, item_id);
        Ok(self.items.lock().await.remove(&key).is_some())
    }

    async fn update_execution_stats(
        &self,
        item_id: &str,
        user_id: &str,
        success: bool,
    ) -> agent_mem_traits::Result<bool> {
        let key = Self::make_key(user_id, item_id);
        let mut items = self.items.lock().await;
        if let Some(item) = items.get_mut(&key) {
            item.execution_count += 1;
            if success {
                // Update success rate using exponential moving average
                let alpha = 0.1;
                item.success_rate = item.success_rate * (1.0 - alpha) + alpha;
            } else {
                let alpha = 0.1;
                item.success_rate = item.success_rate * (1.0 - alpha);
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn get_top_skills(
        &self,
        user_id: &str,
        limit: i64,
    ) -> agent_mem_traits::Result<Vec<ProceduralMemoryItem>> {
        let items = self.items.lock().await;
        let mut user_items: Vec<ProceduralMemoryItem> = items
            .values()
            .filter(|item| item.user_id == user_id)
            .cloned()
            .collect();

        // Sort by success_rate descending
        user_items.sort_by(|a, b| {
            b.success_rate
                .partial_cmp(&a.success_rate)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Apply limit
        user_items.truncate(limit as usize);

        Ok(user_items)
    }
}

#[tokio::test]
async fn test_procedural_agent_insert_with_real_store() {
    // Create mock store
    let store = Arc::new(MockProceduralStore::new());

    // Create ProceduralAgent with store
    let mut agent = ProceduralAgent::new("test-procedural-agent".to_string());
    agent.set_store(store.clone());
    agent.initialize().await.unwrap();

    // Test insert
    let params = json!({
        "user_id": "user123",
        "skill_name": "make_coffee",
        "description": "How to make a perfect cup of coffee",
        "steps": [
            "Boil water to 200°F",
            "Grind coffee beans",
            "Pour water over grounds",
            "Wait 4 minutes",
            "Press and serve"
        ],
        "metadata": {
            "difficulty": "easy",
            "time_minutes": 5
        }
    });

    let task = TaskRequest {
        task_id: "task1".to_string(),
        memory_type: MemoryType::Procedural,
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
    assert_eq!(result["skill_name"], "make_coffee");

    // Verify data was actually stored
    let procedure_id = result["procedure_id"].as_str().unwrap();
    let stored_item = store.get_item(procedure_id, "user123").await.unwrap();
    assert!(stored_item.is_some());
    let item = stored_item.unwrap();
    assert_eq!(item.skill_name, "make_coffee");
    assert_eq!(item.description, "How to make a perfect cup of coffee");
    assert_eq!(item.steps.len(), 5);
}

#[tokio::test]
async fn test_procedural_agent_search_with_real_store() {
    // Create mock store
    let store = Arc::new(MockProceduralStore::new());

    // Pre-populate with test data
    let now = Utc::now();
    let item1 = ProceduralMemoryItem {
        id: "proc1".to_string(),
        organization_id: "default".to_string(),
        user_id: "user123".to_string(),
        agent_id: "test-agent".to_string(),
        skill_name: "make_coffee".to_string(),
        description: "How to make coffee".to_string(),
        steps: vec!["Step 1".to_string(), "Step 2".to_string()],
        success_rate: 0.9,
        execution_count: 10,
        metadata: json!({}),
        created_at: now,
        updated_at: now,
    };
    store.create_item(item1).await.unwrap();

    let item2 = ProceduralMemoryItem {
        id: "proc2".to_string(),
        organization_id: "default".to_string(),
        user_id: "user123".to_string(),
        agent_id: "test-agent".to_string(),
        skill_name: "make_tea".to_string(),
        description: "How to make tea".to_string(),
        steps: vec!["Step 1".to_string()],
        success_rate: 0.8,
        execution_count: 5,
        metadata: json!({}),
        created_at: now,
        updated_at: now,
    };
    store.create_item(item2).await.unwrap();

    // Create ProceduralAgent with store
    let mut agent = ProceduralAgent::new("test-procedural-agent".to_string());
    agent.set_store(store.clone());
    agent.initialize().await.unwrap();

    // Test search
    let params = json!({
        "user_id": "user123",
        "skill_name_pattern": "coffee"
    });

    let task = TaskRequest {
        task_id: "task2".to_string(),
        memory_type: MemoryType::Procedural,
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
    assert_eq!(result["total_count"], 1);

    let results = result["results"].as_array().unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["skill_name"], "make_coffee");
}

#[tokio::test]
async fn test_procedural_agent_update_with_real_store() {
    // Create mock store
    let store = Arc::new(MockProceduralStore::new());

    // Pre-populate with test data
    let now = Utc::now();
    let item = ProceduralMemoryItem {
        id: "proc1".to_string(),
        organization_id: "default".to_string(),
        user_id: "user123".to_string(),
        agent_id: "test-agent".to_string(),
        skill_name: "make_coffee".to_string(),
        description: "Old description".to_string(),
        steps: vec!["Step 1".to_string()],
        success_rate: 0.5,
        execution_count: 1,
        metadata: json!({}),
        created_at: now,
        updated_at: now,
    };
    store.create_item(item).await.unwrap();

    // Create ProceduralAgent with store
    let mut agent = ProceduralAgent::new("test-procedural-agent".to_string());
    agent.set_store(store.clone());
    agent.initialize().await.unwrap();

    // Test update
    let params = json!({
        "user_id": "user123",
        "item_id": "proc1",
        "description": "New description",
        "steps": ["New Step 1", "New Step 2"]
    });

    let task = TaskRequest {
        task_id: "task3".to_string(),
        memory_type: MemoryType::Procedural,
        operation: "update".to_string(),
        parameters: params,
        priority: 1,
        timeout: None,
        retry_count: 0,
    };

    let response = agent.execute_task(task).await.unwrap();
    assert!(response.success);

    // Verify data was actually updated
    let stored_item = store.get_item("proc1", "user123").await.unwrap();
    assert!(stored_item.is_some());
    let item = stored_item.unwrap();
    assert_eq!(item.description, "New description");
    assert_eq!(item.steps.len(), 2);
    assert_eq!(item.steps[0], "New Step 1");
}

#[tokio::test]
async fn test_procedural_agent_delete_with_real_store() {
    // Create mock store
    let store = Arc::new(MockProceduralStore::new());

    // Pre-populate with test data
    let now = Utc::now();
    let item = ProceduralMemoryItem {
        id: "proc1".to_string(),
        organization_id: "default".to_string(),
        user_id: "user123".to_string(),
        agent_id: "test-agent".to_string(),
        skill_name: "make_coffee".to_string(),
        description: "How to make coffee".to_string(),
        steps: vec!["Step 1".to_string()],
        success_rate: 0.9,
        execution_count: 10,
        metadata: json!({}),
        created_at: now,
        updated_at: now,
    };
    store.create_item(item).await.unwrap();

    // Create ProceduralAgent with store
    let mut agent = ProceduralAgent::new("test-procedural-agent".to_string());
    agent.set_store(store.clone());
    agent.initialize().await.unwrap();

    // Test delete
    let params = json!({
        "user_id": "user123",
        "item_id": "proc1"
    });

    let task = TaskRequest {
        task_id: "task4".to_string(),
        memory_type: MemoryType::Procedural,
        operation: "delete".to_string(),
        parameters: params,
        priority: 1,
        timeout: None,
        retry_count: 0,
    };

    let response = agent.execute_task(task).await.unwrap();
    assert!(response.success);

    // Verify data was actually deleted
    let stored_item = store.get_item("proc1", "user123").await.unwrap();
    assert!(stored_item.is_none());
}

