//! Echo tool - simple echo for testing

use crate::schema::PropertySchema;
use crate::{ExecutionContext, Tool, ToolError, ToolResult, ToolSchema};
use async_trait::async_trait;
use serde_json::{json, Value};

/// Echo tool
pub struct EchoTool;

#[async_trait]
impl Tool for EchoTool {
    fn name(&self) -> &str {
        "echo"
    }

    fn description(&self) -> &str {
        "Echo back the input message"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(self.name(), self.description()).add_parameter(
            "message",
            PropertySchema::string("Message to echo"),
            true,
        )
    }

    fn category(&self) -> &str {
        "utility"
    }

    async fn execute(&self, args: Value, _context: &ExecutionContext) -> ToolResult<Value> {
        let message = args["message"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidArgument("Missing message".to_string()))?;

        Ok(json!({
            "echo": message,
            "length": message.len()
        }))
    }
}
