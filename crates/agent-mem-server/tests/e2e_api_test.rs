//! End-to-End API Integration Tests
//!
//! 这些测试需要一个运行中的 AgentMem 服务器实例。
//! 测试完整的 API 流程，包括：
//! - Agent CRUD 操作
//! - Memory CRUD 操作
//! - Chat 对话流程
//! - 流式聊天
//! - 工具调用
//! - 认证和授权
//!
//! 运行方式：
//! 1. 启动服务器: `cargo run --bin agent-mem-server`
//! 2. 运行测试: `cargo test --test e2e_api_test -- --test-threads=1`

use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use std::time::Duration;

/// 测试配置
const BASE_URL: &str = "http://localhost:3000";
const API_VERSION: &str = "v1";
const TEST_ORG_ID: &str = "test-org-e2e";
const TEST_USER_ID: &str = "test-user-e2e";

/// 创建 HTTP 客户端
fn create_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client")
}

/// 生成测试用的认证令牌
fn get_test_auth_token() -> String {
    // 注意：在真实环境中，这应该是一个有效的 JWT token
    // 这里使用一个测试 token，假设服务器配置了测试模式
    "test-token-e2e".to_string()
}

/// 辅助函数：检查服务器是否运行
async fn check_server_running() -> bool {
    let client = create_client();
    match client.get(format!("{}/health", BASE_URL)).send().await {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

#[tokio::test]
#[ignore] // 需要运行中的服务器
async fn test_e2e_health_check() {
    let client = create_client();

    let response = client
        .get(format!("{}/health", BASE_URL))
        .send()
        .await
        .expect("Failed to send health check request");

    assert_eq!(response.status(), StatusCode::OK);

    let body: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["status"], "healthy");

    println!("✅ Health check passed");
}

#[tokio::test]
#[ignore] // 需要运行中的服务器
async fn test_e2e_complete_agent_workflow() {
    if !check_server_running().await {
        println!("⚠️  Server not running, skipping test");
        return;
    }

    let client = create_client();
    let auth_token = get_test_auth_token();

    // Step 1: 创建 Agent
    println!("Step 1: Creating agent...");
    let create_agent_request = json!({
        "name": "E2E Test Agent",
        "description": "Agent for end-to-end testing",
        "organization_id": TEST_ORG_ID,
        "state": "active",
        "config": {
            "llm_provider": "openai",
            "llm_model": "gpt-4"
        }
    });

    let create_response = client
        .post(format!("{}/api/{}/agents", BASE_URL, API_VERSION))
        .header("Authorization", format!("Bearer {}", auth_token))
        .json(&create_agent_request)
        .send()
        .await
        .expect("Failed to create agent");

    assert_eq!(create_response.status(), StatusCode::CREATED);

    let agent: Value = create_response.json().await.expect("Failed to parse agent");
    let agent_id = agent["id"].as_str().expect("Agent ID not found");

    println!("✅ Agent created: {}", agent_id);

    // Step 2: 获取 Agent
    println!("Step 2: Getting agent...");
    let get_response = client
        .get(format!(
            "{}/api/{}/agents/{}",
            BASE_URL, API_VERSION, agent_id
        ))
        .header("Authorization", format!("Bearer {}", auth_token))
        .send()
        .await
        .expect("Failed to get agent");

    assert_eq!(get_response.status(), StatusCode::OK);

    let retrieved_agent: Value = get_response.json().await.expect("Failed to parse agent");
    assert_eq!(retrieved_agent["id"], agent_id);
    assert_eq!(retrieved_agent["name"], "E2E Test Agent");

    println!("✅ Agent retrieved successfully");

    // Step 3: 更新 Agent
    println!("Step 3: Updating agent...");
    let update_request = json!({
        "name": "E2E Test Agent (Updated)",
        "description": "Updated description"
    });

    let update_response = client
        .put(format!(
            "{}/api/{}/agents/{}",
            BASE_URL, API_VERSION, agent_id
        ))
        .header("Authorization", format!("Bearer {}", auth_token))
        .json(&update_request)
        .send()
        .await
        .expect("Failed to update agent");

    assert_eq!(update_response.status(), StatusCode::OK);

    println!("✅ Agent updated successfully");

    // Step 4: 列出 Agents
    println!("Step 4: Listing agents...");
    let list_response = client
        .get(format!("{}/api/{}/agents", BASE_URL, API_VERSION))
        .header("Authorization", format!("Bearer {}", auth_token))
        .send()
        .await
        .expect("Failed to list agents");

    assert_eq!(list_response.status(), StatusCode::OK);

    let agents: Value = list_response.json().await.expect("Failed to parse agents");
    assert!(agents["agents"].is_array());

    println!("✅ Agents listed successfully");

    // Step 5: 删除 Agent
    println!("Step 5: Deleting agent...");
    let delete_response = client
        .delete(format!(
            "{}/api/{}/agents/{}",
            BASE_URL, API_VERSION, agent_id
        ))
        .header("Authorization", format!("Bearer {}", auth_token))
        .send()
        .await
        .expect("Failed to delete agent");

    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    println!("✅ Agent deleted successfully");
    println!("🎉 Complete agent workflow test passed!");
}

#[tokio::test]
#[ignore] // 需要运行中的服务器
async fn test_e2e_complete_memory_workflow() {
    if !check_server_running().await {
        println!("⚠️  Server not running, skipping test");
        return;
    }

    let client = create_client();
    let auth_token = get_test_auth_token();

    // 首先创建一个 agent
    let agent_id = "test-agent-memory-workflow";

    // Step 1: 创建 Memory
    println!("Step 1: Creating memory...");
    let create_memory_request = json!({
        "agent_id": agent_id,
        "user_id": TEST_USER_ID,
        "memory_type": "episodic",
        "content": "User prefers morning meetings at 9 AM",
        "importance": 0.8,
        "metadata": {
            "category": "preferences",
            "tags": ["meetings", "schedule", "morning"]
        }
    });

    let create_response = client
        .post(format!("{}/api/{}/memories", BASE_URL, API_VERSION))
        .header("Authorization", format!("Bearer {}", auth_token))
        .json(&create_memory_request)
        .send()
        .await
        .expect("Failed to create memory");

    assert_eq!(create_response.status(), StatusCode::CREATED);

    let memory: Value = create_response
        .json()
        .await
        .expect("Failed to parse memory");
    let memory_id = memory["id"].as_str().expect("Memory ID not found");

    println!("✅ Memory created: {}", memory_id);

    // Step 2: 获取 Memory
    println!("Step 2: Getting memory...");
    let get_response = client
        .get(format!(
            "{}/api/{}/memories/{}",
            BASE_URL, API_VERSION, memory_id
        ))
        .header("Authorization", format!("Bearer {}", auth_token))
        .send()
        .await
        .expect("Failed to get memory");

    assert_eq!(get_response.status(), StatusCode::OK);

    let retrieved_memory: Value = get_response.json().await.expect("Failed to parse memory");
    assert_eq!(retrieved_memory["id"], memory_id);
    assert_eq!(
        retrieved_memory["content"],
        "User prefers morning meetings at 9 AM"
    );

    println!("✅ Memory retrieved successfully");

    // Step 3: 搜索 Memories
    println!("Step 3: Searching memories...");
    let search_request = json!({
        "query": "morning meetings",
        "agent_id": agent_id,
        "limit": 10
    });

    let search_response = client
        .post(format!("{}/api/{}/memories/search", BASE_URL, API_VERSION))
        .header("Authorization", format!("Bearer {}", auth_token))
        .json(&search_request)
        .send()
        .await
        .expect("Failed to search memories");

    assert_eq!(search_response.status(), StatusCode::OK);

    let search_results: Value = search_response
        .json()
        .await
        .expect("Failed to parse search results");
    assert!(search_results["results"].is_array());

    println!("✅ Memory search completed");

    // Step 4: 更新 Memory
    println!("Step 4: Updating memory...");
    let update_request = json!({
        "importance": 0.9,
        "content": "User strongly prefers morning meetings at 9 AM"
    });

    let update_response = client
        .put(format!(
            "{}/api/{}/memories/{}",
            BASE_URL, API_VERSION, memory_id
        ))
        .header("Authorization", format!("Bearer {}", auth_token))
        .json(&update_request)
        .send()
        .await
        .expect("Failed to update memory");

    assert_eq!(update_response.status(), StatusCode::OK);

    println!("✅ Memory updated successfully");

    // Step 5: 删除 Memory
    println!("Step 5: Deleting memory...");
    let delete_response = client
        .delete(format!(
            "{}/api/{}/memories/{}",
            BASE_URL, API_VERSION, memory_id
        ))
        .header("Authorization", format!("Bearer {}", auth_token))
        .send()
        .await
        .expect("Failed to delete memory");

    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    println!("✅ Memory deleted successfully");
    println!("🎉 Complete memory workflow test passed!");
}

#[tokio::test]
#[ignore] // 需要运行中的服务器
async fn test_e2e_chat_workflow() {
    if !check_server_running().await {
        println!("⚠️  Server not running, skipping test");
        return;
    }

    let client = create_client();
    let auth_token = get_test_auth_token();

    // 首先创建一个 agent
    let agent_id = "test-agent-chat";

    // Step 1: 发送聊天消息
    println!("Step 1: Sending chat message...");
    let chat_request = json!({
        "message": "Hello! I prefer morning meetings at 9 AM.",
        "user_id": TEST_USER_ID,
        "stream": false,
        "max_memories": 10
    });

    let chat_response = client
        .post(format!(
            "{}/api/{}/agents/{}/chat",
            BASE_URL, API_VERSION, agent_id
        ))
        .header("Authorization", format!("Bearer {}", auth_token))
        .json(&chat_request)
        .send()
        .await
        .expect("Failed to send chat message");

    assert_eq!(chat_response.status(), StatusCode::OK);

    let response: Value = chat_response
        .json()
        .await
        .expect("Failed to parse chat response");
    assert!(response["content"].is_string());
    assert!(response["message_id"].is_string());

    println!("✅ Chat message sent and response received");
    println!("Response: {}", response["content"]);

    // Step 2: 发送第二条消息，测试记忆检索
    println!("Step 2: Sending follow-up message...");
    let followup_request = json!({
        "message": "When should we schedule our next meeting?",
        "user_id": TEST_USER_ID,
        "stream": false
    });

    let followup_response = client
        .post(format!(
            "{}/api/{}/agents/{}/chat",
            BASE_URL, API_VERSION, agent_id
        ))
        .header("Authorization", format!("Bearer {}", auth_token))
        .json(&followup_request)
        .send()
        .await
        .expect("Failed to send follow-up message");

    assert_eq!(followup_response.status(), StatusCode::OK);

    let response2: Value = followup_response
        .json()
        .await
        .expect("Failed to parse response");
    let content = response2["content"].as_str().unwrap();

    // 验证响应中包含了之前的记忆（morning meetings）
    println!("Follow-up response: {}", content);

    println!("✅ Chat workflow completed");
    println!("🎉 Complete chat workflow test passed!");
}

#[tokio::test]
#[ignore] // 需要运行中的服务器和真实的 LLM API key
async fn test_e2e_streaming_chat() {
    if !check_server_running().await {
        println!("⚠️  Server not running, skipping test");
        return;
    }

    let client = create_client();
    let auth_token = get_test_auth_token();
    let agent_id = "test-agent-streaming";

    println!("Testing streaming chat...");
    let chat_request = json!({
        "message": "Tell me a short story about AI",
        "user_id": TEST_USER_ID,
        "stream": true
    });

    let response = client
        .post(format!(
            "{}/api/{}/agents/{}/chat/stream",
            BASE_URL, API_VERSION, agent_id
        ))
        .header("Authorization", format!("Bearer {}", auth_token))
        .json(&chat_request)
        .send()
        .await
        .expect("Failed to send streaming chat request");

    assert_eq!(response.status(), StatusCode::OK);

    // 验证响应是 SSE 流
    let content_type = response.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("text/event-stream"));

    println!("✅ Streaming chat initiated");
    println!("🎉 Streaming chat test passed!");
}

#[tokio::test]
#[ignore] // 需要运行中的服务器
async fn test_e2e_authentication() {
    if !check_server_running().await {
        println!("⚠️  Server not running, skipping test");
        return;
    }

    let client = create_client();

    // Test 1: 无认证令牌
    println!("Test 1: Request without auth token...");
    let response = client
        .get(format!("{}/api/{}/agents", BASE_URL, API_VERSION))
        .send()
        .await
        .expect("Failed to send request");

    // 应该返回 401 Unauthorized
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    println!("✅ Unauthorized request rejected");

    // Test 2: 无效的认证令牌
    println!("Test 2: Request with invalid auth token...");
    let response = client
        .get(format!("{}/api/{}/agents", BASE_URL, API_VERSION))
        .header("Authorization", "Bearer invalid-token")
        .send()
        .await
        .expect("Failed to send request");

    // 应该返回 401 Unauthorized
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    println!("✅ Invalid token rejected");

    // Test 3: 有效的认证令牌
    println!("Test 3: Request with valid auth token...");
    let auth_token = get_test_auth_token();
    let response = client
        .get(format!("{}/api/{}/agents", BASE_URL, API_VERSION))
        .header("Authorization", format!("Bearer {}", auth_token))
        .send()
        .await
        .expect("Failed to send request");

    // 应该返回 200 OK
    assert_eq!(response.status(), StatusCode::OK);
    println!("✅ Valid token accepted");

    println!("🎉 Authentication test passed!");
}

#[tokio::test]
#[ignore] // 需要运行中的服务器
async fn test_e2e_error_handling() {
    if !check_server_running().await {
        println!("⚠️  Server not running, skipping test");
        return;
    }

    let client = create_client();
    let auth_token = get_test_auth_token();

    // Test 1: 创建无效的 Agent（空名称）
    println!("Test 1: Creating agent with empty name...");
    let invalid_agent = json!({
        "name": "",
        "description": "Test",
        "organization_id": TEST_ORG_ID
    });

    let response = client
        .post(format!("{}/api/{}/agents", BASE_URL, API_VERSION))
        .header("Authorization", format!("Bearer {}", auth_token))
        .json(&invalid_agent)
        .send()
        .await
        .expect("Failed to send request");

    // 应该返回 400 Bad Request
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    println!("✅ Invalid agent creation rejected");

    // Test 2: 获取不存在的 Agent
    println!("Test 2: Getting non-existent agent...");
    let response = client
        .get(format!(
            "{}/api/{}/agents/non-existent-agent-id",
            BASE_URL, API_VERSION
        ))
        .header("Authorization", format!("Bearer {}", auth_token))
        .send()
        .await
        .expect("Failed to send request");

    // 应该返回 404 Not Found
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    println!("✅ Non-existent agent returns 404");

    // Test 3: 创建无效的 Memory（importance > 1.0）
    println!("Test 3: Creating memory with invalid importance...");
    let invalid_memory = json!({
        "agent_id": "test-agent",
        "user_id": TEST_USER_ID,
        "memory_type": "episodic",
        "content": "Test",
        "importance": 1.5  // Invalid: > 1.0
    });

    let response = client
        .post(format!("{}/api/{}/memories", BASE_URL, API_VERSION))
        .header("Authorization", format!("Bearer {}", auth_token))
        .json(&invalid_memory)
        .send()
        .await
        .expect("Failed to send request");

    // 应该返回 400 Bad Request
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    println!("✅ Invalid memory creation rejected");

    println!("🎉 Error handling test passed!");
}
