//! AgentMem Tools - Tool Execution Framework
//!
//! This crate provides a complete tool execution framework for AgentMem, inspired by
//! MIRIX's tool system but optimized for Rust's performance and type safety.
//!
//! # Features
//!
//! - **Dynamic Tool Registration**: Register and discover tools at runtime
//! - **Schema Generation**: Automatic JSON Schema generation for tool parameters
//! - **Sandboxed Execution**: Safe tool execution with timeout and resource limits
//! - **Permission Management**: Fine-grained access control for tool execution
//! - **Tool Chaining**: Support for dependent tool calls
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Tool Execution Framework                  │
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
//! │  │   Tool      │  │   Schema    │  │  Sandbox    │         │
//! │  │  Registry   │  │  Generator  │  │  Manager    │         │
//! │  └─────────────┘  └─────────────┘  └─────────────┘         │
//! │         │                │                 │                 │
//! │         └────────────────┴─────────────────┘                 │
//! │                          │                                   │
//! │                  ┌───────▼────────┐                          │
//! │                  │ Tool Executor  │                          │
//! │                  └────────────────┘                          │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Example
//!
//! ```rust
//! use agent_mem_tools::{Tool, ToolExecutor, ExecutionContext};
//! use async_trait::async_trait;
//! use serde_json::{json, Value};
//! use std::sync::Arc;
//! use std::time::Duration;
//!
//! // Define a custom tool
//! struct CalculatorTool;
//!
//! #[async_trait]
//! impl Tool for CalculatorTool {
//!     fn name(&self) -> &str {
//!         "calculator"
//!     }
//!
//!     fn description(&self) -> &str {
//!         "Perform basic arithmetic operations"
//!     }
//!
//!     fn schema(&self) -> agent_mem_tools::ToolSchema {
//!         // Schema definition
//!         todo!()
//!     }
//!
//!     async fn execute(
//!         &self,
//!         args: Value,
//!         _context: &ExecutionContext,
//!     ) -> agent_mem_tools::ToolResult<Value> {
//!         let operation = args["operation"].as_str().unwrap();
//!         let a = args["a"].as_f64().unwrap();
//!         let b = args["b"].as_f64().unwrap();
//!
//!         let result = match operation {
//!             "add" => a + b,
//!             "subtract" => a - b,
//!             "multiply" => a * b,
//!             "divide" => a / b,
//!             _ => return Err(agent_mem_tools::ToolError::InvalidArgument(
//!                 format!("Unknown operation: {}", operation)
//!             )),
//!         };
//!
//!         Ok(json!({ "result": result }))
//!     }
//! }
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create executor and register tool
//! let executor = ToolExecutor::new();
//! executor.register_tool(Arc::new(CalculatorTool)).await?;
//!
//! // Execute tool
//! let context = ExecutionContext {
//!     user: "user123".to_string(),
//!     timeout: Duration::from_secs(30),
//! };
//!
//! let result = executor.execute_tool(
//!     "calculator",
//!     json!({
//!         "operation": "add",
//!         "a": 10,
//!         "b": 20
//!     }),
//!     &context,
//! ).await?;
//!
//! println!("Result: {}", result);
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

/// Tool execution and management
pub mod executor;

/// Tool schema definition and validation
pub mod schema;

/// Sandboxed execution environment
pub mod sandbox;

/// Tool execution sandbox (advanced)
pub mod execution_sandbox;

/// Permission management
pub mod permissions;

/// Error types
pub mod error;

/// Built-in tools
pub mod builtin;

/// MCP (Model Context Protocol) support
pub mod mcp;

// Re-export main types
pub use error::{ToolError, ToolResult};
pub use execution_sandbox::{
    LocalSandboxConfig, SandboxRunResult, SandboxStatus, SandboxType, ToolExecutionSandbox,
};
pub use executor::{ExecutionContext, Tool, ToolExecutor};
pub use mcp::{McpClient, McpClientManager, McpError, McpResult, McpServerConfig, McpTool};
pub use permissions::{Permission, PermissionManager};
pub use sandbox::{CommandOutput, SandboxConfig, SandboxManager, SandboxStats};
pub use schema::{ParameterSchema, PropertySchema, ToolSchema};

/// Tool execution statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolStats {
    /// Tool name
    pub tool_name: String,
    /// Total executions
    pub total_executions: u64,
    /// Successful executions
    pub successful_executions: u64,
    /// Failed executions
    pub failed_executions: u64,
    /// Average execution time (ms)
    pub avg_execution_time_ms: f64,
    /// Last execution time
    pub last_execution: Option<chrono::DateTime<chrono::Utc>>,
}
