//! Tool executor - core tool execution and management
//!
//! This module provides the main tool execution framework, inspired by
//! MIRIX's ToolExecutor but optimized for Rust's async and type safety.

use crate::error::{ToolError, ToolResult};
use crate::permissions::PermissionManager;
use crate::sandbox::SandboxManager;
use crate::schema::ToolSchema;
use crate::ToolStats;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Execution context for tool calls
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// User ID
    pub user: String,
    /// Execution timeout
    pub timeout: Duration,
}

/// Tool trait - all tools must implement this
#[async_trait]
pub trait Tool: Send + Sync {
    /// Tool name (unique identifier)
    fn name(&self) -> &str;

    /// Tool description
    fn description(&self) -> &str;

    /// Tool schema (parameters and validation)
    fn schema(&self) -> ToolSchema;

    /// Execute the tool
    async fn execute(&self, args: Value, context: &ExecutionContext) -> ToolResult<Value>;

    /// Optional: Tool version
    fn version(&self) -> &str {
        "1.0.0"
    }

    /// Optional: Tool category
    fn category(&self) -> &str {
        "general"
    }
}

/// Tool executor - manages tool registration and execution
pub struct ToolExecutor {
    /// Registered tools
    tools: Arc<RwLock<HashMap<String, Arc<dyn Tool>>>>,
    /// Tool schemas
    schemas: Arc<RwLock<HashMap<String, ToolSchema>>>,
    /// Permission manager
    permissions: Arc<PermissionManager>,
    /// Sandbox manager
    sandbox: Arc<SandboxManager>,
    /// Tool execution statistics
    stats: Arc<RwLock<HashMap<String, ToolStats>>>,
}

impl ToolExecutor {
    /// Create a new tool executor
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            schemas: Arc::new(RwLock::new(HashMap::new())),
            permissions: Arc::new(PermissionManager::new()),
            sandbox: Arc::new(SandboxManager::default()),
            stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create with custom sandbox and permission managers
    pub fn with_managers(sandbox: SandboxManager, permissions: PermissionManager) -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            schemas: Arc::new(RwLock::new(HashMap::new())),
            permissions: Arc::new(permissions),
            sandbox: Arc::new(sandbox),
            stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a tool
    pub async fn register_tool(&self, tool: Arc<dyn Tool>) -> ToolResult<()> {
        let name = tool.name().to_string();
        let schema = tool.schema();

        info!("Registering tool: {}", name);

        // Check if tool already exists
        let tools = self.tools.read().await;
        if tools.contains_key(&name) {
            return Err(ToolError::AlreadyRegistered(name));
        }
        drop(tools);

        // Register tool
        let mut tools = self.tools.write().await;
        let mut schemas = self.schemas.write().await;
        let mut stats = self.stats.write().await;

        tools.insert(name.clone(), tool);
        schemas.insert(name.clone(), schema);
        stats.insert(
            name.clone(),
            ToolStats {
                tool_name: name.clone(),
                total_executions: 0,
                successful_executions: 0,
                failed_executions: 0,
                avg_execution_time_ms: 0.0,
                last_execution: None,
            },
        );

        info!("Tool '{}' registered successfully", name);
        Ok(())
    }

    /// Unregister a tool
    pub async fn unregister_tool(&self, name: &str) -> ToolResult<()> {
        info!("Unregistering tool: {}", name);

        let mut tools = self.tools.write().await;
        let mut schemas = self.schemas.write().await;

        if tools.remove(name).is_none() {
            return Err(ToolError::NotFound(name.to_string()));
        }
        schemas.remove(name);

        info!("Tool '{}' unregistered successfully", name);
        Ok(())
    }

    /// Get a tool by name
    pub async fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool>> {
        let tools = self.tools.read().await;
        tools.get(name).cloned()
    }

    /// List all registered tools
    pub async fn list_tools(&self) -> Vec<String> {
        let tools = self.tools.read().await;
        tools.keys().cloned().collect()
    }

    /// Get tool schema
    pub async fn get_schema(&self, name: &str) -> Option<ToolSchema> {
        let schemas = self.schemas.read().await;
        schemas.get(name).cloned()
    }

    /// Execute a tool
    pub async fn execute_tool(
        &self,
        name: &str,
        args: Value,
        context: &ExecutionContext,
    ) -> ToolResult<Value> {
        debug!("Executing tool '{}' for user '{}'", name, context.user);

        let start = Instant::now();

        // 1. Check if tool exists
        let tool = {
            let tools = self.tools.read().await;
            tools
                .get(name)
                .cloned()
                .ok_or_else(|| ToolError::NotFound(name.to_string()))?
        };

        // 2. Check permissions
        self.permissions
            .check_permission(name, &context.user)
            .await?;

        // 3. Validate arguments
        let schema = {
            let schemas = self.schemas.read().await;
            schemas
                .get(name)
                .cloned()
                .ok_or_else(|| ToolError::NotFound(name.to_string()))?
        };
        schema.validate(&args)?;

        // 4. Execute in sandbox
        let tool_clone = tool.clone();
        let context_clone = context.clone();
        let args_clone = args.clone();

        let result = self
            .sandbox
            .execute(
                async move { tool_clone.execute(args_clone, &context_clone).await },
                context.timeout,
            )
            .await;

        // 5. Update statistics
        let duration = start.elapsed();
        self.update_stats(name, result.is_ok(), duration).await;

        match &result {
            Ok(_) => {
                info!("Tool '{}' executed successfully in {:?}", name, duration);
            }
            Err(e) => {
                warn!("Tool '{}' execution failed: {}", name, e);
            }
        }

        result
    }

    /// Update tool statistics
    async fn update_stats(&self, tool_name: &str, success: bool, duration: Duration) {
        let mut stats = self.stats.write().await;
        if let Some(tool_stats) = stats.get_mut(tool_name) {
            tool_stats.total_executions += 1;
            if success {
                tool_stats.successful_executions += 1;
            } else {
                tool_stats.failed_executions += 1;
            }

            // Update average execution time
            let total_time =
                tool_stats.avg_execution_time_ms * (tool_stats.total_executions - 1) as f64;
            let new_time = duration.as_secs_f64() * 1000.0;
            tool_stats.avg_execution_time_ms =
                (total_time + new_time) / tool_stats.total_executions as f64;

            tool_stats.last_execution = Some(chrono::Utc::now());
        }
    }

    /// Get tool statistics
    pub async fn get_stats(&self, tool_name: &str) -> Option<ToolStats> {
        let stats = self.stats.read().await;
        stats.get(tool_name).cloned()
    }

    /// Get all statistics
    pub async fn get_all_stats(&self) -> Vec<ToolStats> {
        let stats = self.stats.read().await;
        stats.values().cloned().collect()
    }

    /// Get permission manager
    pub fn permissions(&self) -> &Arc<PermissionManager> {
        &self.permissions
    }

    /// Get sandbox manager
    pub fn sandbox(&self) -> &Arc<SandboxManager> {
        &self.sandbox
    }
}

impl Default for ToolExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::PropertySchema;
    use serde_json::json;

    struct TestTool;

    #[async_trait]
    impl Tool for TestTool {
        fn name(&self) -> &str {
            "test_tool"
        }

        fn description(&self) -> &str {
            "A test tool"
        }

        fn schema(&self) -> ToolSchema {
            ToolSchema::new("test_tool", "A test tool").add_parameter(
                "input",
                PropertySchema::string("Input value"),
                true,
            )
        }

        async fn execute(&self, args: Value, _context: &ExecutionContext) -> ToolResult<Value> {
            let input = args["input"].as_str().unwrap();
            Ok(json!({ "output": format!("Processed: {}", input) }))
        }
    }

    #[tokio::test]
    async fn test_tool_registration() {
        let executor = ToolExecutor::new();
        let tool = Arc::new(TestTool);

        let result = executor.register_tool(tool).await;
        assert!(result.is_ok());

        let tools = executor.list_tools().await;
        assert_eq!(tools.len(), 1);
        assert!(tools.contains(&"test_tool".to_string()));
    }

    #[tokio::test]
    async fn test_tool_execution() {
        let executor = ToolExecutor::new();
        let tool = Arc::new(TestTool);

        executor.register_tool(tool).await.unwrap();

        // Assign admin role to user
        executor.permissions().assign_role("user1", "admin").await;

        let context = ExecutionContext {
            user: "user1".to_string(),
            timeout: Duration::from_secs(30),
        };

        let result = executor
            .execute_tool("test_tool", json!({ "input": "hello" }), &context)
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output["output"], "Processed: hello");
    }

    #[tokio::test]
    async fn test_tool_not_found() {
        let executor = ToolExecutor::new();

        let context = ExecutionContext {
            user: "user1".to_string(),
            timeout: Duration::from_secs(30),
        };

        let result = executor
            .execute_tool("nonexistent", json!({}), &context)
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ToolError::NotFound(_)));
    }
}
