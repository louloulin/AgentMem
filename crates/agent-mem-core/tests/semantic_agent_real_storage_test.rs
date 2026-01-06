//! Integration tests for SemanticAgent with real storage

use agent_mem_core::agents::{MemoryAgent, SemanticAgent};
use agent_mem_core::coordination::TaskRequest;
use agent_mem_core::types::MemoryType;
use agent_mem_traits::{SemanticMemoryItem, SemanticMemoryStore, SemanticQuery};
use async_trait::async_trait;
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Mock implementation of SemanticMemoryStore for testing
#[derive(Clone)]
struct MockSemanticStore {
    items: Arc<Mutex<HashMap<String, SemanticMemoryItem>>>,
}

impl MockSemanticStore {
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
impl SemanticMemoryStore for MockSemanticStore {
    async fn create_item(
        &self,
        item: SemanticMemoryItem,
    ) -> agent_mem_traits::Result<SemanticMemoryItem> {
        let key = Self::make_key(&item.user_id, &item.id);
        self.items.lock().await.insert(key, item.clone());
        Ok(item)
    }

    async fn get_item(
        &self,
        item_id: &str,
        user_id: &str,
    ) -> agent_mem_traits::Result<Option<SemanticMemoryItem>> {
        let key = Self::make_key(user_id, item_id);
        Ok(self.items.lock().await.get(&key).cloned())
    }

    async fn query_items(
        &self,
        user_id: &str,
        query: SemanticQuery,
    ) -> agent_mem_traits::Result<Vec<SemanticMemoryItem>> {
        let items = self.items.lock().await;
        let mut results: Vec<_> = items
            .values()
            .filter(|item| {
                item.user_id == user_id
                    && query.name_query.as_ref().map_or(true, |nq| {
                        // Remove % wildcards if present
                        let pattern = nq.trim_matches('%');
                        item.name.contains(pattern)
                    })
                    && query.summary_query.as_ref().map_or(true, |sq| {
                        let pattern = sq.trim_matches('%');
                        item.summary.contains(pattern)
                    })
                    && query.tree_path_prefix.as_ref().map_or(true, |prefix| {
                        item.tree_path.len() >= prefix.len()
                            && item.tree_path[..prefix.len()] == prefix[..]
                    })
            })
            .cloned()
            .collect();

        // Sort by created_at descending
        results.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        if let Some(limit) = query.limit {
            results.truncate(limit as usize);
        }

        Ok(results)
    }

    async fn update_item(&self, item: SemanticMemoryItem) -> agent_mem_traits::Result<bool> {
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

    async fn search_by_tree_path(
        &self,
        user_id: &str,
        tree_path: Vec<String>,
    ) -> agent_mem_traits::Result<Vec<SemanticMemoryItem>> {
        let items = self.items.lock().await;
        let results: Vec<_> = items
            .values()
            .filter(|item| item.user_id == user_id && item.tree_path == tree_path)
            .cloned()
            .collect();
        Ok(results)
    }

    async fn search_by_name(
        &self,
        user_id: &str,
        name_pattern: &str,
        limit: i64,
    ) -> agent_mem_traits::Result<Vec<SemanticMemoryItem>> {
        let items = self.items.lock().await;
        let mut results: Vec<_> = items
            .values()
            .filter(|item| item.user_id == user_id && item.name.contains(name_pattern))
            .cloned()
            .collect();

        results.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        results.truncate(limit as usize);
        Ok(results)
    }
}

#[tokio::test]
async fn test_semantic_agent_insert_with_real_store() {
    // Create mock store
    let store = Arc::new(MockSemanticStore::new());

    // Create SemanticAgent with store
    let mut agent = SemanticAgent::new("test-semantic-agent".to_string());
    agent.set_store(store.clone());
    agent.initialize().await.unwrap();

    // Test insert
    let now = Utc::now();
    let item_id = uuid::Uuid::new_v4().to_string();
    let params = json!({
        "id": item_id,
        "organization_id": "org1",
        "user_id": "user123",
        "agent_id": "test-agent",
        "name": "Rust Programming",
        "summary": "A systems programming language",
        "details": "Rust is a multi-paradigm programming language",
        "source": "Wikipedia",
        "tree_path": ["Programming", "Languages", "Rust"],
        "metadata": {},
        "created_at": now.to_rfc3339(),
        "updated_at": now.to_rfc3339()
    });

    let task = TaskRequest {
        task_id: "task1".to_string(),
        memory_type: MemoryType::Semantic,
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
    let returned_item_id = result["item_id"].as_str().unwrap();

    // Verify data was actually stored
    let stored_item = store.get_item(returned_item_id, "user123").await.unwrap();
    assert!(stored_item.is_some());
    let item = stored_item.unwrap();
    assert_eq!(item.name, "Rust Programming");
    assert_eq!(item.summary, "A systems programming language");
}

#[tokio::test]
async fn test_semantic_agent_search_with_real_store() {
    // Create mock store
    let store = Arc::new(MockSemanticStore::new());

    // Pre-populate with test data
    let now = Utc::now();
    let item1 = SemanticMemoryItem {
        id: "item1".to_string(),
        organization_id: "org1".to_string(),
        user_id: "user123".to_string(),
        agent_id: "test-agent".to_string(),
        name: "Rust Programming".to_string(),
        summary: "A systems programming language".to_string(),
        details: Some("Rust is fast and safe".to_string()),
        source: Some("Wikipedia".to_string()),
        tree_path: vec!["Programming".to_string(), "Languages".to_string()],
        metadata: json!({}),
        created_at: now,
        updated_at: now,
    };
    store.create_item(item1).await.unwrap();

    let item2 = SemanticMemoryItem {
        id: "item2".to_string(),
        organization_id: "org1".to_string(),
        user_id: "user123".to_string(),
        agent_id: "test-agent".to_string(),
        name: "Python Programming".to_string(),
        summary: "A high-level programming language".to_string(),
        details: Some("Python is easy to learn".to_string()),
        source: Some("Wikipedia".to_string()),
        tree_path: vec!["Programming".to_string(), "Languages".to_string()],
        metadata: json!({}),
        created_at: now,
        updated_at: now,
    };
    store.create_item(item2).await.unwrap();

    // Create SemanticAgent with store
    let mut agent = SemanticAgent::new("test-semantic-agent".to_string());
    agent.set_store(store.clone());
    agent.initialize().await.unwrap();

    // Test search
    let params = json!({
        "user_id": "user123",
        "name_query": "Rust",
        "limit": 10
    });

    let task = TaskRequest {
        task_id: "task2".to_string(),
        memory_type: MemoryType::Semantic,
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
    assert_eq!(results[0]["name"], "Rust Programming");
}

/// Test SemanticAgent update with real storage
#[tokio::test]
async fn test_semantic_agent_update_with_real_store() {
    let store = Arc::new(MockSemanticStore::new());
    let mut agent = SemanticAgent::with_store("semantic-agent-1".to_string(), store.clone());
    agent.initialize().await.unwrap();

    // First, insert an item
    let item_id = "item-update-1";
    let now = Utc::now();
    let params_insert = json!({
        "id": item_id,
        "organization_id": "org1",
        "user_id": "user123",
        "agent_id": "agent1",
        "name": "Original Name",
        "summary": "Original summary",
        "details": "Original details",
        "source": "test",
        "tree_path": ["root", "category"],
        "metadata": {},
        "created_at": now.to_rfc3339(),
        "updated_at": now.to_rfc3339()
    });

    let task_insert = TaskRequest {
        task_id: "task-insert".to_string(),
        memory_type: MemoryType::Semantic,
        operation: "insert".to_string(),
        parameters: params_insert,
        priority: 1,
        timeout: None,
        retry_count: 0,
    };

    agent.execute_task(task_insert).await.unwrap();

    // Now update the item
    let updated_at = Utc::now();
    let params_update = json!({
        "id": item_id,
        "organization_id": "org1",
        "user_id": "user123",
        "agent_id": "agent1",
        "name": "Updated Name",
        "summary": "Updated summary",
        "details": "Updated details",
        "source": "test",
        "tree_path": ["root", "category"],
        "metadata": {},
        "created_at": now.to_rfc3339(),
        "updated_at": updated_at.to_rfc3339()
    });

    let task_update = TaskRequest {
        task_id: "task-update".to_string(),
        memory_type: MemoryType::Semantic,
        operation: "update".to_string(),
        parameters: params_update,
        priority: 1,
        timeout: None,
        retry_count: 0,
    };

    let response = agent.execute_task(task_update).await.unwrap();
    if !response.success {
        eprintln!("Update failed: {:?}", response);
    }
    assert!(response.success);

    let result = response.data.unwrap();
    assert_eq!(result["success"], true);
    assert_eq!(result["item_id"], item_id);

    // Verify the item was actually updated in the store
    let stored_item = store.get_item(item_id, "user123").await.unwrap().unwrap();
    assert_eq!(stored_item.name, "Updated Name");
    assert_eq!(stored_item.summary, "Updated summary");
}

/// Test SemanticAgent delete with real storage
#[tokio::test]
async fn test_semantic_agent_delete_with_real_store() {
    let store = Arc::new(MockSemanticStore::new());
    let mut agent = SemanticAgent::with_store("semantic-agent-1".to_string(), store.clone());
    agent.initialize().await.unwrap();

    // First, insert an item
    let item_id = "item-delete-1";
    let now = Utc::now();
    let params_insert = json!({
        "id": item_id,
        "organization_id": "org1",
        "user_id": "user123",
        "agent_id": "agent1",
        "name": "To Be Deleted",
        "summary": "This item will be deleted",
        "details": null,
        "source": "test",
        "tree_path": ["root"],
        "metadata": {},
        "created_at": now.to_rfc3339(),
        "updated_at": now.to_rfc3339()
    });

    let task_insert = TaskRequest {
        task_id: "task-insert".to_string(),

        memory_type: MemoryType::Semantic,
        operation: "insert".to_string(),
        parameters: params_insert,
        priority: 1,
        timeout: None,
        retry_count: 0,
    };

    agent.execute_task(task_insert).await.unwrap();

    // Verify item exists
    assert!(store.get_item(item_id, "user123").await.unwrap().is_some());

    // Now delete the item
    let params_delete = json!({
        "id": item_id,
        "user_id": "user123"
    });

    let task_delete = TaskRequest {
        task_id: "task-delete".to_string(),

        memory_type: MemoryType::Semantic,
        operation: "delete".to_string(),
        parameters: params_delete,
        priority: 1,
        timeout: None,
        retry_count: 0,
    };

    let response = agent.execute_task(task_delete).await.unwrap();
    assert!(response.success);

    let result = response.data.unwrap();
    assert_eq!(result["success"], true);
    assert_eq!(result["item_id"], item_id);

    // Verify the item was actually deleted from the store
    assert!(store.get_item(item_id, "user123").await.unwrap().is_none());
}

/// Test SemanticAgent query_relationships with real storage
#[tokio::test]
async fn test_semantic_agent_query_relationships_with_real_store() {
    let store = Arc::new(MockSemanticStore::new());
    let mut agent = SemanticAgent::with_store("semantic-agent-1".to_string(), store.clone());
    agent.initialize().await.unwrap();

    // Insert a main concept
    let main_id = "concept-main";
    let now = Utc::now();
    let params_main = json!({
        "id": main_id,
        "organization_id": "org1",
        "user_id": "user123",
        "agent_id": "agent1",
        "name": "Main Concept",
        "summary": "The main concept",
        "details": null,
        "source": "test",
        "tree_path": ["root", "category", "subcategory"],
        "metadata": {},
        "created_at": now.to_rfc3339(),
        "updated_at": now.to_rfc3339()
    });

    let task_main = TaskRequest {
        task_id: "task-main".to_string(),

        memory_type: MemoryType::Semantic,
        operation: "insert".to_string(),
        parameters: params_main,
        priority: 1,
        timeout: None,
        retry_count: 0,
    };

    agent.execute_task(task_main).await.unwrap();

    // Insert related concepts with same tree_path
    let related_id = "concept-related";
    let params_related = json!({
        "id": related_id,
        "organization_id": "org1",
        "user_id": "user123",
        "agent_id": "agent1",
        "name": "Related Concept",
        "summary": "A related concept",
        "details": null,
        "source": "test",
        "tree_path": ["root", "category", "subcategory"],
        "metadata": {},
        "created_at": now.to_rfc3339(),
        "updated_at": now.to_rfc3339()
    });

    let task_related = TaskRequest {
        task_id: "task-related".to_string(),

        memory_type: MemoryType::Semantic,
        operation: "insert".to_string(),
        parameters: params_related,
        priority: 1,
        timeout: None,
        retry_count: 0,
    };

    agent.execute_task(task_related).await.unwrap();

    // Query relationships
    let params_query = json!({
        "concept_id": main_id,
        "user_id": "user123"
    });

    let task_query = TaskRequest {
        task_id: "task-query-rel".to_string(),

        memory_type: MemoryType::Semantic,
        operation: "relationship_query".to_string(),
        parameters: params_query,
        priority: 1,
        timeout: None,
        retry_count: 0,
    };

    let response = agent.execute_task(task_query).await.unwrap();
    assert!(response.success);

    let result = response.data.unwrap();
    assert_eq!(result["success"], true);
    assert_eq!(result["concept_id"], main_id);

    // Should find the related concept
    let relationships = result["relationships"].as_array().unwrap();
    assert_eq!(relationships.len(), 1);
    assert_eq!(relationships[0]["id"], related_id);
}

/// Test SemanticAgent graph_traversal with real storage
#[tokio::test]
async fn test_semantic_agent_graph_traversal_with_real_store() {
    let store = Arc::new(MockSemanticStore::new());
    let mut agent = SemanticAgent::with_store("semantic-agent-1".to_string(), store.clone());
    agent.initialize().await.unwrap();

    // Insert a start concept
    let start_id = "concept-start";
    let now = Utc::now();
    let params_start = json!({
        "id": start_id,
        "organization_id": "org1",
        "user_id": "user123",
        "agent_id": "agent1",
        "name": "Start Concept",
        "summary": "The starting point",
        "details": null,
        "source": "test",
        "tree_path": ["root", "level1", "level2"],
        "metadata": {},
        "created_at": now.to_rfc3339(),
        "updated_at": now.to_rfc3339()
    });

    let task_start = TaskRequest {
        task_id: "task-start".to_string(),

        memory_type: MemoryType::Semantic,
        operation: "insert".to_string(),
        parameters: params_start,
        priority: 1,
        timeout: None,
        retry_count: 0,
    };

    agent.execute_task(task_start).await.unwrap();

    // Traverse the graph
    let params_traverse = json!({
        "start_concept": start_id,
        "user_id": "user123",
        "max_depth": 2
    });

    let task_traverse = TaskRequest {
        task_id: "task-traverse".to_string(),

        memory_type: MemoryType::Semantic,
        operation: "graph_traversal".to_string(),
        parameters: params_traverse,
        priority: 1,
        timeout: None,
        retry_count: 0,
    };

    let response = agent.execute_task(task_traverse).await.unwrap();
    assert!(response.success);

    let result = response.data.unwrap();
    assert_eq!(result["success"], true);
    assert_eq!(result["start_concept"], start_id);
    assert_eq!(result["max_depth"], 2);

    // Should have traversal path
    let traversal_path = result["traversal_path"].as_array().unwrap();
    assert!(traversal_path.len() > 0);
    assert_eq!(traversal_path[0]["id"], start_id);
}
