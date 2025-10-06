//! Chat API 集成测试
//!
//! 测试 Chat API 的基本功能

use agent_mem_server::routes::chat::{ChatMessageRequest, ChatMessageResponse};
use serde_json::json;

#[tokio::test]
async fn test_chat_request_serialization() {
    let request = ChatMessageRequest {
        message: "Hello, agent!".to_string(),
        user_id: Some("user-123".to_string()),
        stream: false,
        metadata: Some(json!({"key": "value"})),
    };

    let json_str = serde_json::to_string(&request).unwrap();
    assert!(json_str.contains("Hello, agent!"));
    assert!(json_str.contains("user-123"));
}

#[tokio::test]
async fn test_chat_request_deserialization() {
    let json_str = r#"{
        "message": "Test message",
        "user_id": "user-456",
        "stream": true,
        "metadata": {"test": true}
    }"#;

    let request: ChatMessageRequest = serde_json::from_str(json_str).unwrap();
    assert_eq!(request.message, "Test message");
    assert_eq!(request.user_id, Some("user-456".to_string()));
    assert_eq!(request.stream, true);
    assert!(request.metadata.is_some());
}

#[tokio::test]
async fn test_chat_request_minimal() {
    let json_str = r#"{"message": "Minimal test"}"#;
    let request: ChatMessageRequest = serde_json::from_str(json_str).unwrap();
    assert_eq!(request.message, "Minimal test");
    assert_eq!(request.user_id, None);
    assert_eq!(request.stream, false); // default value
    assert_eq!(request.metadata, None);
}

#[test]
fn test_chat_response_structure() {
    let response = ChatMessageResponse {
        message_id: "msg-123".to_string(),
        content: "Hello!".to_string(),
        memories_updated: true,
        memories_count: 3,
        tool_calls: None,
        processing_time_ms: 150,
    };

    assert_eq!(response.message_id, "msg-123");
    assert_eq!(response.content, "Hello!");
    assert_eq!(response.memories_updated, true);
    assert_eq!(response.memories_count, 3);
    assert_eq!(response.processing_time_ms, 150);
}

#[test]
fn test_chat_response_serialization() {
    let response = ChatMessageResponse {
        message_id: "msg-456".to_string(),
        content: "Response content".to_string(),
        memories_updated: false,
        memories_count: 0,
        tool_calls: None,
        processing_time_ms: 200,
    };

    let json_str = serde_json::to_string(&response).unwrap();
    assert!(json_str.contains("msg-456"));
    assert!(json_str.contains("Response content"));
    assert!(json_str.contains("\"memories_updated\":false"));
}

// TODO: 添加完整的端到端测试
// 需要：
// 1. 启动测试服务器
// 2. 创建测试用户和 Agent
// 3. 发送 Chat 请求
// 4. 验证响应
// 5. 验证记忆存储
//
// 示例：
// #[tokio::test]
// async fn test_chat_api_end_to_end() {
//     // 1. 启动测试服务器
//     let app = create_test_app().await;
//     
//     // 2. 创建测试用户
//     let user = create_test_user(&app).await;
//     
//     // 3. 创建测试 Agent
//     let agent = create_test_agent(&app, &user).await;
//     
//     // 4. 发送 Chat 请求
//     let response = app
//         .post(&format!("/api/v1/agents/{}/chat", agent.id))
//         .json(&json!({
//             "message": "Hello, how are you?"
//         }))
//         .send()
//         .await
//         .unwrap();
//     
//     // 5. 验证响应
//     assert_eq!(response.status(), 200);
//     let body: ChatMessageResponse = response.json().await.unwrap();
//     assert!(!body.content.is_empty());
//     assert!(!body.message_id.is_empty());
// }

