/// 测试输入验证功能
///
/// 验证 ChatRequest 的所有验证规则
use agent_mem_core::orchestrator::ChatRequest;

#[test]
fn test_valid_chat_request() {
    let request = ChatRequest {
        message: "Hello, world!".to_string(),
        agent_id: "test-agent".to_string(),
        user_id: "test-user".to_string(),
        organization_id: "test-org".to_string(),
        session_id: "test-session".to_string(),
        stream: false,
        max_memories: 10,
    };

    assert!(request.validate().is_ok());
}

#[test]
fn test_empty_message() {
    let request = ChatRequest {
        message: "".to_string(),
        agent_id: "test-agent".to_string(),
        user_id: "test-user".to_string(),
        organization_id: "test-org".to_string(),
        session_id: "test-session".to_string(),
        stream: false,
        max_memories: 10,
    };

    let result = request.validate();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Message cannot be empty"));
}

#[test]
fn test_whitespace_only_message() {
    let request = ChatRequest {
        message: "   ".to_string(),
        agent_id: "test-agent".to_string(),
        user_id: "test-user".to_string(),
        organization_id: "test-org".to_string(),
        session_id: "test-session".to_string(),
        stream: false,
        max_memories: 10,
    };

    let result = request.validate();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Message cannot be empty"));
}

#[test]
fn test_message_too_long() {
    let request = ChatRequest {
        message: "a".repeat(100_001), // 100KB + 1 byte
        agent_id: "test-agent".to_string(),
        user_id: "test-user".to_string(),
        organization_id: "test-org".to_string(),
        session_id: "test-session".to_string(),
        stream: false,
        max_memories: 10,
    };

    let result = request.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Message too long"));
}

#[test]
fn test_empty_agent_id() {
    let request = ChatRequest {
        message: "Hello".to_string(),
        agent_id: "".to_string(),
        user_id: "test-user".to_string(),
        organization_id: "test-org".to_string(),
        session_id: "test-session".to_string(),
        stream: false,
        max_memories: 10,
    };

    let result = request.validate();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Agent ID cannot be empty"));
}

#[test]
fn test_agent_id_too_long() {
    let request = ChatRequest {
        message: "Hello".to_string(),
        agent_id: "a".repeat(256), // 256 characters
        user_id: "test-user".to_string(),
        organization_id: "test-org".to_string(),
        session_id: "test-session".to_string(),
        stream: false,
        max_memories: 10,
    };

    let result = request.validate();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Agent ID too long"));
}

#[test]
fn test_empty_user_id() {
    let request = ChatRequest {
        message: "Hello".to_string(),
        agent_id: "test-agent".to_string(),
        user_id: "".to_string(),
        organization_id: "test-org".to_string(),
        session_id: "test-session".to_string(),
        stream: false,
        max_memories: 10,
    };

    let result = request.validate();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("User ID cannot be empty"));
}

#[test]
fn test_user_id_too_long() {
    let request = ChatRequest {
        message: "Hello".to_string(),
        agent_id: "test-agent".to_string(),
        user_id: "a".repeat(256), // 256 characters
        organization_id: "test-org".to_string(),
        session_id: "test-session".to_string(),
        stream: false,
        max_memories: 10,
    };

    let result = request.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("User ID too long"));
}

#[test]
fn test_empty_organization_id() {
    let request = ChatRequest {
        message: "Hello".to_string(),
        agent_id: "test-agent".to_string(),
        user_id: "test-user".to_string(),
        organization_id: "".to_string(),
        session_id: "test-session".to_string(),
        stream: false,
        max_memories: 10,
    };

    let result = request.validate();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Organization ID cannot be empty"));
}

#[test]
fn test_organization_id_too_long() {
    let request = ChatRequest {
        message: "Hello".to_string(),
        agent_id: "test-agent".to_string(),
        user_id: "test-user".to_string(),
        organization_id: "a".repeat(256), // 256 characters
        session_id: "test-session".to_string(),
        stream: false,
        max_memories: 10,
    };

    let result = request.validate();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Organization ID too long"));
}

#[test]
fn test_max_memories_zero() {
    let request = ChatRequest {
        message: "Hello".to_string(),
        agent_id: "test-agent".to_string(),
        user_id: "test-user".to_string(),
        organization_id: "test-org".to_string(),
        session_id: "test-session".to_string(),
        stream: false,
        max_memories: 0,
    };

    let result = request.validate();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("max_memories must be at least 1"));
}

#[test]
fn test_max_memories_too_large() {
    let request = ChatRequest {
        message: "Hello".to_string(),
        agent_id: "test-agent".to_string(),
        user_id: "test-user".to_string(),
        organization_id: "test-org".to_string(),
        session_id: "test-session".to_string(),
        stream: false,
        max_memories: 1001,
    };

    let result = request.validate();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("max_memories too large"));
}

#[test]
fn test_max_memories_boundary_values() {
    // Test min boundary (1)
    let request_min = ChatRequest {
        message: "Hello".to_string(),
        agent_id: "test-agent".to_string(),
        user_id: "test-user".to_string(),
        organization_id: "test-org".to_string(),
        session_id: "test-session".to_string(),
        stream: false,
        max_memories: 1,
    };
    assert!(request_min.validate().is_ok());

    // Test max boundary (1000)
    let request_max = ChatRequest {
        message: "Hello".to_string(),
        agent_id: "test-agent".to_string(),
        user_id: "test-user".to_string(),
        organization_id: "test-org".to_string(),
        session_id: "test-session".to_string(),
        stream: false,
        max_memories: 1000,
    };
    assert!(request_max.validate().is_ok());
}

#[test]
fn test_message_length_boundary() {
    // Test max allowed length (100KB)
    let request = ChatRequest {
        message: "a".repeat(100_000), // Exactly 100KB
        agent_id: "test-agent".to_string(),
        user_id: "test-user".to_string(),
        organization_id: "test-org".to_string(),
        session_id: "test-session".to_string(),
        stream: false,
        max_memories: 10,
    };
    assert!(request.validate().is_ok());
}

#[test]
fn test_id_length_boundary() {
    // Test max allowed length (255 characters)
    let long_id = "a".repeat(255);

    let request = ChatRequest {
        message: "Hello".to_string(),
        agent_id: long_id.clone(),
        user_id: long_id.clone(),
        organization_id: long_id,
        session_id: "test-session".to_string(),
        stream: false,
        max_memories: 10,
    };
    assert!(request.validate().is_ok());
}
