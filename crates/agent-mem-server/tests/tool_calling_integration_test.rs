//! 工具调用集成测试
//!
//! 测试工具注册和执行功能

use agent_mem_tools::{builtin::register_all_builtin_tools, ToolExecutor};

#[tokio::test]
async fn test_tool_executor_has_builtin_tools() {
    // 创建 ToolExecutor 并注册内置工具
    let tool_executor = ToolExecutor::new();
    register_all_builtin_tools(&tool_executor)
        .await
        .expect("Failed to register builtin tools");

    // 验证工具已注册
    let tools = tool_executor.list_tools().await;

    println!("Registered tools: {:?}", tools);

    // 验证基础工具
    assert!(
        tools.contains(&"calculator".to_string()),
        "calculator tool should be registered"
    );
    assert!(
        tools.contains(&"echo".to_string()),
        "echo tool should be registered"
    );
    assert!(
        tools.contains(&"json_parser".to_string()),
        "json_parser tool should be registered"
    );
    assert!(
        tools.contains(&"string_ops".to_string()),
        "string_ops tool should be registered"
    );
    assert!(
        tools.contains(&"time_ops".to_string()),
        "time_ops tool should be registered"
    );

    // 验证高级工具
    assert!(
        tools.contains(&"search".to_string()),
        "search tool should be registered"
    );
    assert!(
        tools.contains(&"file_read".to_string()),
        "file_read tool should be registered"
    );
    assert!(
        tools.contains(&"file_write".to_string()),
        "file_write tool should be registered"
    );
    assert!(
        tools.contains(&"http_request".to_string()),
        "http_request tool should be registered"
    );

    // 验证工具总数
    assert_eq!(tools.len(), 9, "Should have 9 builtin tools");
}

#[tokio::test]
async fn test_calculator_tool_execution() {
    use agent_mem_tools::ExecutionContext;
    use serde_json::json;
    use std::time::Duration;

    // 创建 ToolExecutor 并注册内置工具
    let tool_executor = ToolExecutor::new();
    register_all_builtin_tools(&tool_executor)
        .await
        .expect("Failed to register builtin tools");

    // 设置权限
    tool_executor
        .permissions()
        .assign_role("test_user", "admin")
        .await;

    // 创建执行上下文
    let context = ExecutionContext {
        user: "test_user".to_string(),
        timeout: Duration::from_secs(30),
    };

    // 执行计算器工具
    let result = tool_executor
        .execute_tool(
            "calculator",
            json!({
                "operation": "add",
                "a": 10.0,
                "b": 20.0
            }),
            &context,
        )
        .await
        .expect("Failed to execute calculator tool");

    println!("Calculator result: {}", result);

    // 验证结果
    assert_eq!(result["result"], 30.0, "10 + 20 should equal 30");
}

#[tokio::test]
async fn test_echo_tool_execution() {
    use agent_mem_tools::ExecutionContext;
    use serde_json::json;
    use std::time::Duration;

    // 创建 ToolExecutor 并注册内置工具
    let tool_executor = ToolExecutor::new();
    register_all_builtin_tools(&tool_executor)
        .await
        .expect("Failed to register builtin tools");

    // 设置权限
    tool_executor
        .permissions()
        .assign_role("test_user", "admin")
        .await;

    // 创建执行上下文
    let context = ExecutionContext {
        user: "test_user".to_string(),
        timeout: Duration::from_secs(30),
    };

    // 执行 echo 工具
    let result = tool_executor
        .execute_tool(
            "echo",
            json!({
                "message": "Hello, AgentMem!"
            }),
            &context,
        )
        .await
        .expect("Failed to execute echo tool");

    println!("Echo result: {}", result);

    // 验证结果（echo 工具返回的字段是 "echo"，不是 "message"）
    assert_eq!(
        result["echo"], "Hello, AgentMem!",
        "Echo should return the same message"
    );
    assert_eq!(result["length"], 16, "Length should be 16");
}

#[tokio::test]
async fn test_string_ops_tool_execution() {
    use agent_mem_tools::ExecutionContext;
    use serde_json::json;
    use std::time::Duration;

    // 创建 ToolExecutor 并注册内置工具
    let tool_executor = ToolExecutor::new();
    register_all_builtin_tools(&tool_executor)
        .await
        .expect("Failed to register builtin tools");

    // 设置权限
    tool_executor
        .permissions()
        .assign_role("test_user", "admin")
        .await;

    // 创建执行上下文
    let context = ExecutionContext {
        user: "test_user".to_string(),
        timeout: Duration::from_secs(30),
    };

    // 执行字符串操作工具（转大写）
    let result = tool_executor
        .execute_tool(
            "string_ops",
            json!({
                "operation": "uppercase",
                "text": "hello world"
            }),
            &context,
        )
        .await
        .expect("Failed to execute string_ops tool");

    println!("String ops result: {}", result);

    // 验证结果
    assert_eq!(
        result["result"], "HELLO WORLD",
        "Should convert to uppercase"
    );
}

// 注意：完整的端到端测试需要真实的 LLM API 调用，
// 这里我们只测试工具注册和执行的基础功能
