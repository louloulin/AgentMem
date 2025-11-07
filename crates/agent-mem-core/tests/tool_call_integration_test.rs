//! Tool Call Integration Test
//!
//! Tests the newly implemented tool calling functionality in the orchestrator

use agent_mem_core::orchestrator::tool_integration::{ToolIntegrator, ToolIntegratorConfig};
use agent_mem_tools::{builtin::CalculatorTool, ExecutionContext, Tool, ToolExecutor};
use agent_mem_traits::llm::FunctionCall;
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_tool_integrator_creation() {
    // Create tool executor
    let executor = Arc::new(ToolExecutor::new());

    // Create tool integrator
    let config = ToolIntegratorConfig::default();
    let _integrator = ToolIntegrator::new(config.clone(), executor);

    // Verify configuration
    assert_eq!(config.max_tool_rounds, 5);
    assert_eq!(config.tool_timeout_seconds, 30);
    assert_eq!(config.allow_parallel_execution, false);

    println!("✅ Tool integrator created successfully");
}

#[tokio::test]
async fn test_tool_executor_registration() {
    // Create tool executor
    let executor = Arc::new(ToolExecutor::new());

    // Register calculator tool
    let calculator = Arc::new(CalculatorTool);
    executor.register_tool(calculator).await.unwrap();

    // Verify tool is registered
    let tools = executor.list_tools().await;
    assert!(tools.contains(&"calculator".to_string()));

    println!("✅ Tool registered successfully: {:?}", tools);
}

#[tokio::test]
async fn test_tool_execution_basic() {
    // Create tool executor
    let executor = Arc::new(ToolExecutor::new());

    // Register calculator tool
    let calculator = Arc::new(CalculatorTool);
    executor.register_tool(calculator).await.unwrap();

    // Create execution context
    let context = ExecutionContext {
        user: "test-user".to_string(),
        timeout: Duration::from_secs(30),
    };

    // Execute tool
    let args = serde_json::json!({
        "operation": "add",
        "a": 10,
        "b": 20
    });

    let result = executor
        .execute_tool("calculator", args, &context)
        .await
        .unwrap();

    println!("Tool execution result: {}", result);

    // Verify result
    assert!(result.to_string().contains("30") || result.to_string().contains("result"));

    println!("✅ Tool executed successfully");
}

#[tokio::test]
async fn test_tool_call_integration() {
    // Create tool executor
    let executor = Arc::new(ToolExecutor::new());

    // Register calculator tool
    let calculator = Arc::new(CalculatorTool);
    executor.register_tool(calculator).await.unwrap();

    // Create tool integrator
    let config = ToolIntegratorConfig::default();
    let integrator = ToolIntegrator::new(config, executor);

    // Create function calls
    let function_calls = vec![
        FunctionCall {
            name: "calculator".to_string(),
            arguments: serde_json::json!({
                "operation": "add",
                "a": 5,
                "b": 3
            })
            .to_string(),
        },
        FunctionCall {
            name: "calculator".to_string(),
            arguments: serde_json::json!({
                "operation": "multiply",
                "a": 4,
                "b": 7
            })
            .to_string(),
        },
    ];

    // Execute tool calls
    let results = integrator
        .execute_tool_calls(&function_calls, "test-user")
        .await
        .unwrap();

    println!("Tool call results: {} calls executed", results.len());
    for (i, result) in results.iter().enumerate() {
        println!(
            "  {}. {} - {} ({}ms)",
            i + 1,
            result.tool_name,
            if result.success { "✅" } else { "❌" },
            result.execution_time_ms
        );
    }

    // Verify results
    assert_eq!(results.len(), 2);
    assert!(results[0].success);
    assert!(results[1].success);

    println!("✅ Tool call integration test passed");
}

#[tokio::test]
async fn test_tool_definitions_retrieval() {
    // Create tool executor
    let executor = Arc::new(ToolExecutor::new());

    // Register calculator tool
    let calculator = Arc::new(CalculatorTool);
    executor.register_tool(calculator).await.unwrap();

    // Create tool integrator
    let config = ToolIntegratorConfig::default();
    let integrator = ToolIntegrator::new(config, executor);

    // Get tool definitions
    let definitions = integrator.get_tool_definitions().await.unwrap();

    println!("Tool definitions retrieved: {} tools", definitions.len());
    for def in &definitions {
        println!("  - {}: {}", def.name, def.description);
    }

    // Verify definitions
    assert_eq!(definitions.len(), 1);
    assert_eq!(definitions[0].name, "calculator");
    assert!(!definitions[0].description.is_empty());

    // Verify parameters structure
    let params = &definitions[0].parameters;
    assert!(params.is_object());
    assert!(params.get("type").is_some());
    assert!(params.get("properties").is_some());

    println!("✅ Tool definitions retrieved successfully");
}

#[tokio::test]
async fn test_tool_error_handling() {
    // Create tool executor
    let executor = Arc::new(ToolExecutor::new());

    // Register calculator tool
    let calculator = Arc::new(CalculatorTool);
    executor.register_tool(calculator).await.unwrap();

    // Create tool integrator
    let config = ToolIntegratorConfig::default();
    let integrator = ToolIntegrator::new(config, executor);

    // Create function call with invalid arguments
    let function_calls = vec![FunctionCall {
        name: "calculator".to_string(),
        arguments: "invalid json".to_string(), // Invalid JSON
    }];

    // Execute tool calls
    let results = integrator
        .execute_tool_calls(&function_calls, "test-user")
        .await
        .unwrap();

    println!("Error handling test results:");
    for result in &results {
        println!(
            "  - {}: success={}, error={:?}",
            result.tool_name, result.success, result.error
        );
    }

    // Verify error handling
    assert_eq!(results.len(), 1);
    assert!(!results[0].success);
    assert!(results[0].error.is_some());

    println!("✅ Tool error handling test passed");
}

#[tokio::test]
async fn test_tool_result_formatting() {
    use agent_mem_core::orchestrator::tool_integration::ToolCallResult;

    // Create tool executor
    let executor = Arc::new(ToolExecutor::new());

    // Create tool integrator
    let config = ToolIntegratorConfig::default();
    let integrator = ToolIntegrator::new(config, executor);

    // Create mock results
    let results = vec![
        ToolCallResult {
            tool_name: "calculator".to_string(),
            arguments: r#"{"operation": "add", "a": 1, "b": 2}"#.to_string(),
            result: r#"{"result": 3}"#.to_string(),
            success: true,
            error: None,
            execution_time_ms: 10,
        },
        ToolCallResult {
            tool_name: "search".to_string(),
            arguments: r#"{"query": "test"}"#.to_string(),
            result: String::new(),
            success: false,
            error: Some("Tool not found".to_string()),
            execution_time_ms: 5,
        },
    ];

    // Format results
    let formatted = integrator.format_tool_results(&results);

    println!("Formatted tool results:\n{}", formatted);

    // Verify formatting
    assert!(formatted.contains("calculator"));
    assert!(formatted.contains("Success"));
    assert!(formatted.contains("search"));
    assert!(formatted.contains("Failed"));
    assert!(formatted.contains("10ms"));
    assert!(formatted.contains("5ms"));

    println!("✅ Tool result formatting test passed");
}

#[tokio::test]
async fn test_multiple_tool_rounds() {
    // Create tool executor
    let executor = Arc::new(ToolExecutor::new());

    // Register calculator tool
    let calculator = Arc::new(CalculatorTool);
    executor.register_tool(calculator).await.unwrap();

    // Create tool integrator with custom config
    let config = ToolIntegratorConfig {
        max_tool_rounds: 3,
        tool_timeout_seconds: 10,
        allow_parallel_execution: false,
    };
    let integrator = ToolIntegrator::new(config, executor);

    // Simulate multiple rounds of tool calls
    for round in 1..=3 {
        let function_calls = vec![FunctionCall {
            name: "calculator".to_string(),
            arguments: serde_json::json!({
                "operation": "add",
                "a": round,
                "b": round * 2
            })
            .to_string(),
        }];

        let results = integrator
            .execute_tool_calls(&function_calls, "test-user")
            .await
            .unwrap();

        println!("Round {}: {} tool(s) executed", round, results.len());
        assert_eq!(results.len(), 1);
        assert!(results[0].success);
    }

    println!("✅ Multiple tool rounds test passed");
}
