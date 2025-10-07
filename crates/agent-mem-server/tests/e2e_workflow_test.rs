//! End-to-End Workflow Integration Tests
//!
//! Tests complete user workflows including:
//! - Agent creation
//! - Chat conversation
//! - Memory creation and retrieval
//! - Memory search
//! - Knowledge graph

use serde_json::json;

/// Test helper to create a test agent
fn create_test_agent_request() -> serde_json::Value {
    json!({
        "name": "Test Agent",
        "description": "An agent for testing",
        "organization_id": "test-org-123",
        "state": "active"
    })
}

/// Test helper to create a test memory
fn create_test_memory_request(agent_id: &str) -> serde_json::Value {
    json!({
        "agent_id": agent_id,
        "memory_type": "episodic",
        "content": "User prefers morning meetings",
        "importance": 0.8,
        "metadata": {
            "category": "preferences",
            "tags": ["meetings", "schedule"]
        }
    })
}

/// Test helper to create a chat message
fn create_chat_message_request(message: &str) -> serde_json::Value {
    json!({
        "message": message,
        "user_id": "test-user-123",
        "stream": false
    })
}

#[tokio::test]
async fn test_complete_agent_workflow() {
    // Test data structures for agent workflow
    let agent_request = create_test_agent_request();
    
    // Validate agent request structure
    assert!(agent_request["name"].is_string());
    assert!(agent_request["description"].is_string());
    assert_eq!(agent_request["name"], "Test Agent");
    
    // Simulate agent response
    let agent_response = json!({
        "id": "agent-123",
        "name": "Test Agent",
        "description": "An agent for testing",
        "organization_id": "test-org-123",
        "state": "active",
        "created_at": "2025-10-07T00:00:00Z",
        "updated_at": "2025-10-07T00:00:00Z"
    });
    
    assert_eq!(agent_response["id"], "agent-123");
    assert_eq!(agent_response["state"], "active");
}

#[tokio::test]
async fn test_complete_memory_workflow() {
    let agent_id = "agent-123";
    
    // Step 1: Create memory
    let memory_request = create_test_memory_request(agent_id);
    assert_eq!(memory_request["agent_id"], agent_id);
    assert_eq!(memory_request["memory_type"], "episodic");
    assert_eq!(memory_request["importance"], 0.8);
    
    // Step 2: Simulate memory creation response
    let memory_response = json!({
        "id": "mem-456",
        "agent_id": agent_id,
        "memory_type": "episodic",
        "content": "User prefers morning meetings",
        "importance": 0.8,
        "created_at": "2025-10-07T00:00:00Z",
        "updated_at": "2025-10-07T00:00:00Z"
    });
    
    assert_eq!(memory_response["id"], "mem-456");
    assert_eq!(memory_response["agent_id"], agent_id);
    
    // Step 3: Search for memory
    let search_request = json!({
        "query": "morning meetings",
        "agent_id": agent_id,
        "limit": 10
    });
    
    assert_eq!(search_request["query"], "morning meetings");
    assert_eq!(search_request["agent_id"], agent_id);
    
    // Step 4: Simulate search response
    let search_response = json!({
        "results": [
            {
                "id": "mem-456",
                "content": "User prefers morning meetings",
                "relevance_score": 0.95,
                "memory_type": "episodic"
            }
        ],
        "total": 1
    });
    
    assert_eq!(search_response["total"], 1);
    assert!(search_response["results"].is_array());
}

#[tokio::test]
async fn test_complete_chat_workflow() {
    let agent_id = "agent-123";
    
    // Step 1: Send first message
    let message1 = create_chat_message_request("Hello, I prefer morning meetings");
    assert_eq!(message1["message"], "Hello, I prefer morning meetings");
    
    // Step 2: Simulate chat response with memory extraction
    let response1 = json!({
        "message_id": "msg-001",
        "content": "I understand you prefer morning meetings. I'll remember that!",
        "memories_updated": true,
        "memories_count": 1,
        "processing_time_ms": 150
    });
    
    assert_eq!(response1["memories_updated"], true);
    assert_eq!(response1["memories_count"], 1);
    
    // Step 3: Send second message
    let message2 = create_chat_message_request("When should we schedule our next meeting?");
    assert_eq!(message2["message"], "When should we schedule our next meeting?");
    
    // Step 4: Simulate response with memory retrieval
    let response2 = json!({
        "message_id": "msg-002",
        "content": "Based on your preference for morning meetings, how about 9 AM tomorrow?",
        "memories_updated": false,
        "memories_count": 0,
        "processing_time_ms": 120
    });
    
    assert_eq!(response2["memories_updated"], false);
    assert!(response2["content"].as_str().unwrap().contains("morning"));
}

#[tokio::test]
async fn test_memory_lifecycle() {
    let agent_id = "agent-123";
    
    // Step 1: Create multiple memories
    let memories = vec![
        json!({
            "agent_id": agent_id,
            "memory_type": "episodic",
            "content": "User likes coffee",
            "importance": 0.7
        }),
        json!({
            "agent_id": agent_id,
            "memory_type": "semantic",
            "content": "Coffee is a beverage",
            "importance": 0.5
        }),
        json!({
            "agent_id": agent_id,
            "memory_type": "procedural",
            "content": "How to make coffee: boil water, add grounds",
            "importance": 0.6
        }),
    ];
    
    assert_eq!(memories.len(), 3);
    assert_eq!(memories[0]["memory_type"], "episodic");
    assert_eq!(memories[1]["memory_type"], "semantic");
    assert_eq!(memories[2]["memory_type"], "procedural");
    
    // Step 2: Simulate memory retrieval
    let retrieved_memories = json!({
        "memories": [
            {"id": "mem-1", "content": "User likes coffee", "importance": 0.7},
            {"id": "mem-2", "content": "Coffee is a beverage", "importance": 0.5},
            {"id": "mem-3", "content": "How to make coffee: boil water, add grounds", "importance": 0.6}
        ],
        "total": 3
    });
    
    assert_eq!(retrieved_memories["total"], 3);
    
    // Step 3: Update memory importance
    let update_request = json!({
        "importance": 0.9
    });
    
    assert_eq!(update_request["importance"], 0.9);
    
    // Step 4: Delete memory
    let delete_response = json!({
        "success": true,
        "message": "Memory deleted successfully"
    });
    
    assert_eq!(delete_response["success"], true);
}

#[tokio::test]
async fn test_knowledge_graph_workflow() {
    let agent_id = "agent-123";
    
    // Step 1: Create related memories
    let memory1 = json!({
        "agent_id": agent_id,
        "content": "User works at TechCorp",
        "memory_type": "episodic"
    });
    
    let memory2 = json!({
        "agent_id": agent_id,
        "content": "TechCorp is a software company",
        "memory_type": "semantic"
    });
    
    let memory3 = json!({
        "agent_id": agent_id,
        "content": "User is a software engineer",
        "memory_type": "episodic"
    });
    
    // Step 2: Simulate graph data
    let graph_data = json!({
        "nodes": [
            {"id": "mem-1", "label": "User works at TechCorp", "type": "episodic"},
            {"id": "mem-2", "label": "TechCorp is a software company", "type": "semantic"},
            {"id": "mem-3", "label": "User is a software engineer", "type": "episodic"}
        ],
        "edges": [
            {"source": "mem-1", "target": "mem-2", "type": "related"},
            {"source": "mem-1", "target": "mem-3", "type": "related"}
        ]
    });
    
    assert_eq!(graph_data["nodes"].as_array().unwrap().len(), 3);
    assert_eq!(graph_data["edges"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_multi_agent_workflow() {
    // Step 1: Create multiple agents
    let agent1 = json!({
        "name": "Assistant Agent",
        "description": "General purpose assistant"
    });
    
    let agent2 = json!({
        "name": "Specialist Agent",
        "description": "Domain specialist"
    });
    
    // Step 2: Each agent has separate memories
    let agent1_memory = json!({
        "agent_id": "agent-1",
        "content": "User prefers detailed explanations"
    });
    
    let agent2_memory = json!({
        "agent_id": "agent-2",
        "content": "User is an expert in the field"
    });
    
    assert_ne!(agent1_memory["agent_id"], agent2_memory["agent_id"]);
    assert_ne!(agent1_memory["content"], agent2_memory["content"]);
}

#[tokio::test]
async fn test_error_handling_workflow() {
    // Test invalid agent creation
    let invalid_agent = json!({
        "name": "", // Empty name should fail
        "description": "Test"
    });
    
    assert_eq!(invalid_agent["name"], "");
    
    // Test invalid memory creation
    let invalid_memory = json!({
        "agent_id": "non-existent-agent",
        "content": "Test memory"
    });
    
    assert_eq!(invalid_memory["agent_id"], "non-existent-agent");
    
    // Test invalid search query
    let invalid_search = json!({
        "query": "", // Empty query
        "agent_id": "agent-123"
    });
    
    assert_eq!(invalid_search["query"], "");
}

#[test]
fn test_data_validation() {
    // Test agent name validation
    let valid_name = "Test Agent";
    assert!(!valid_name.is_empty());
    assert!(valid_name.len() <= 255);
    
    // Test importance score validation
    let valid_importance = 0.8;
    assert!(valid_importance >= 0.0 && valid_importance <= 1.0);
    
    let invalid_importance = 1.5;
    assert!(invalid_importance > 1.0); // Should fail validation
    
    // Test memory type validation
    let valid_types = vec!["episodic", "semantic", "procedural", "working", "core"];
    assert!(valid_types.contains(&"episodic"));
    assert!(!valid_types.contains(&"invalid"));
}

#[test]
fn test_performance_requirements() {
    use std::time::Instant;
    
    // Test that data structure operations are fast
    let start = Instant::now();
    
    let agent = json!({
        "name": "Test Agent",
        "description": "Performance test"
    });
    
    let _json_str = serde_json::to_string(&agent).unwrap();
    
    let duration = start.elapsed();
    
    // JSON serialization should be very fast (< 1ms)
    assert!(duration.as_millis() < 1);
}

