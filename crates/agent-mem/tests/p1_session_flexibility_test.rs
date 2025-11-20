//! P1 Session 管理灵活性测试
//!
//! 验证 MemoryScope 枚举和灵活的 Session 管理功能

use agent_mem::{AddMemoryOptions, Memory, MemoryScope};

#[test]
fn test_memory_scope_from_options() {
    // 测试 Global scope
    let options = AddMemoryOptions::default();
    let scope = MemoryScope::from_options(&options);
    assert!(matches!(scope, MemoryScope::Global));

    // 测试 User scope
    let options = AddMemoryOptions {
        user_id: Some("alice".to_string()),
        ..Default::default()
    };
    let scope = MemoryScope::from_options(&options);
    match scope {
        MemoryScope::User { user_id } => assert_eq!(user_id, "alice"),
        _ => panic!("Expected User scope"),
    }

    // 测试 Agent scope
    let options = AddMemoryOptions {
        user_id: Some("alice".to_string()),
        agent_id: Some("assistant".to_string()),
        ..Default::default()
    };
    let scope = MemoryScope::from_options(&options);
    match scope {
        MemoryScope::Agent { user_id, agent_id } => {
            assert_eq!(user_id, "alice");
            assert_eq!(agent_id, "assistant");
        }
        _ => panic!("Expected Agent scope"),
    }

    // 测试 Run scope
    let options = AddMemoryOptions {
        user_id: Some("alice".to_string()),
        run_id: Some("run-123".to_string()),
        ..Default::default()
    };
    let scope = MemoryScope::from_options(&options);
    match scope {
        MemoryScope::Run { user_id, run_id } => {
            assert_eq!(user_id, "alice");
            assert_eq!(run_id, "run-123");
        }
        _ => panic!("Expected Run scope"),
    }

    // 测试 Organization scope
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("org_id".to_string(), "acme-corp".to_string());
    let options = AddMemoryOptions {
        metadata,
        ..Default::default()
    };
    let scope = MemoryScope::from_options(&options);
    match scope {
        MemoryScope::Organization { org_id } => assert_eq!(org_id, "acme-corp"),
        _ => panic!("Expected Organization scope"),
    }

    // 测试 Session scope
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("session_id".to_string(), "window-1".to_string());
    let options = AddMemoryOptions {
        user_id: Some("alice".to_string()),
        metadata,
        ..Default::default()
    };
    let scope = MemoryScope::from_options(&options);
    match scope {
        MemoryScope::Session {
            user_id,
            session_id,
        } => {
            assert_eq!(user_id, "alice");
            assert_eq!(session_id, "window-1");
        }
        _ => panic!("Expected Session scope"),
    }
}

#[test]
fn test_memory_scope_to_options() {
    // 测试 Global -> Options
    let scope = MemoryScope::Global;
    let options = scope.to_options();
    assert_eq!(options.user_id, None);
    assert_eq!(options.agent_id, None);
    assert_eq!(options.run_id, None);
    assert!(options.metadata.is_empty());

    // 测试 User -> Options
    let scope = MemoryScope::User {
        user_id: "alice".to_string(),
    };
    let options = scope.to_options();
    assert_eq!(options.user_id, Some("alice".to_string()));
    assert_eq!(options.agent_id, None);

    // 测试 Agent -> Options
    let scope = MemoryScope::Agent {
        user_id: "alice".to_string(),
        agent_id: "assistant".to_string(),
    };
    let options = scope.to_options();
    assert_eq!(options.user_id, Some("alice".to_string()));
    assert_eq!(options.agent_id, Some("assistant".to_string()));

    // 测试 Organization -> Options
    let scope = MemoryScope::Organization {
        org_id: "acme-corp".to_string(),
    };
    let options = scope.to_options();
    assert_eq!(
        options.metadata.get("org_id"),
        Some(&"acme-corp".to_string())
    );

    // 测试 Session -> Options
    let scope = MemoryScope::Session {
        user_id: "alice".to_string(),
        session_id: "window-1".to_string(),
    };
    let options = scope.to_options();
    assert_eq!(options.user_id, Some("alice".to_string()));
    assert_eq!(
        options.metadata.get("session_id"),
        Some(&"window-1".to_string())
    );
}

#[test]
fn test_add_memory_options_to_scope() {
    let options = AddMemoryOptions {
        user_id: Some("alice".to_string()),
        agent_id: Some("assistant".to_string()),
        ..Default::default()
    };
    let scope = options.to_scope();
    match scope {
        MemoryScope::Agent { user_id, agent_id } => {
            assert_eq!(user_id, "alice");
            assert_eq!(agent_id, "assistant");
        }
        _ => panic!("Expected Agent scope"),
    }
}

#[tokio::test]
async fn test_add_with_scope() {
    let mem = Memory::new().await.expect("初始化失败");

    // 测试 User scope
    let scope = MemoryScope::User {
        user_id: "alice".to_string(),
    };
    let result = mem.add_with_scope("I love pizza", scope).await;
    assert!(result.is_ok(), "User scope 添加应该成功");

    // 测试 Organization scope
    let scope = MemoryScope::Organization {
        org_id: "acme-corp".to_string(),
    };
    let result = mem.add_with_scope("Company policy", scope).await;
    assert!(result.is_ok(), "Organization scope 添加应该成功");

    // 测试 Session scope
    let scope = MemoryScope::Session {
        user_id: "alice".to_string(),
        session_id: "window-1".to_string(),
    };
    let result = mem.add_with_scope("Current conversation", scope).await;
    assert!(result.is_ok(), "Session scope 添加应该成功");
}
