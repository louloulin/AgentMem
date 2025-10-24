//! Basic usage example for agent-mem-tools
//!
//! This example demonstrates:
//! - Tool registration
//! - Tool execution
//! - Permission management
//! - Statistics collection

use agent_mem_tools::{builtin::register_all_builtin_tools, ExecutionContext, ToolExecutor};
use serde_json::json;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("=== AgentMem Tools - Basic Usage Example ===\n");

    // 1. Create tool executor
    println!("1. Creating tool executor...");
    let executor = ToolExecutor::new();

    // 2. Register built-in tools
    println!("2. Registering built-in tools...");
    register_all_builtin_tools(&executor).await?;

    let tools = executor.list_tools().await;
    println!("   Registered {} tools: {:?}\n", tools.len(), tools);

    // 3. Set up permissions
    println!("3. Setting up permissions...");
    executor.permissions().assign_role("alice", "admin").await;
    executor.permissions().assign_role("bob", "user").await;
    println!("   - alice: admin role");
    println!("   - bob: user role\n");

    // 4. Execute calculator tool
    println!("4. Executing calculator tool (add)...");
    let context = ExecutionContext {
        user: "alice".to_string(),
        timeout: Duration::from_secs(30),
    };

    let result = executor
        .execute_tool(
            "calculator",
            json!({
                "operation": "add",
                "a": 10.0,
                "b": 20.0
            }),
            &context,
        )
        .await?;

    println!("   Result: {result}\n");

    // 5. Execute string operations tool
    println!("5. Executing string_ops tool (uppercase)...");
    let result = executor
        .execute_tool(
            "string_ops",
            json!({
                "operation": "uppercase",
                "text": "hello world"
            }),
            &context,
        )
        .await?;

    println!("   Result: {result}\n");

    // 6. Execute time operations tool
    println!("6. Executing time_ops tool (current_time)...");
    let result = executor
        .execute_tool(
            "time_ops",
            json!({
                "operation": "current_time"
            }),
            &context,
        )
        .await?;

    println!("   Result: {result}\n");

    // 7. Execute echo tool
    println!("7. Executing echo tool...");
    let result = executor
        .execute_tool(
            "echo",
            json!({
                "message": "Hello from AgentMem Tools!"
            }),
            &context,
        )
        .await?;

    println!("   Result: {result}\n");

    // 8. Execute JSON parser tool
    println!("8. Executing json_parser tool...");
    let result = executor
        .execute_tool(
            "json_parser",
            json!({
                "json_string": r#"{"name": "Alice", "age": 30}"#
            }),
            &context,
        )
        .await?;

    println!("   Result: {result}\n");

    // 9. Show statistics
    println!("9. Tool execution statistics:");
    let all_stats = executor.get_all_stats().await;
    for stats in all_stats {
        if stats.total_executions > 0 {
            println!(
                "   - {}: {} executions, avg time: {:.2}ms",
                stats.tool_name, stats.total_executions, stats.avg_execution_time_ms
            );
        }
    }

    println!("\n=== Example completed successfully! ===");

    Ok(())
}
