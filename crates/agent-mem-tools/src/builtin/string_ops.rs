//! String operations tool

use crate::schema::PropertySchema;
use crate::{ExecutionContext, Tool, ToolError, ToolResult, ToolSchema};
use async_trait::async_trait;
use serde_json::{json, Value};

/// String operations tool
pub struct StringOpsTool;

#[async_trait]
impl Tool for StringOpsTool {
    fn name(&self) -> &str {
        "string_ops"
    }

    fn description(&self) -> &str {
        "Perform string operations (uppercase, lowercase, reverse, length, trim)"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(self.name(), self.description())
            .add_parameter(
                "operation",
                PropertySchema::string("The operation to perform").with_enum(vec![
                    "uppercase".to_string(),
                    "lowercase".to_string(),
                    "reverse".to_string(),
                    "length".to_string(),
                    "trim".to_string(),
                ]),
                true,
            )
            .add_parameter("text", PropertySchema::string("Input text"), true)
    }

    fn category(&self) -> &str {
        "text"
    }

    async fn execute(&self, args: Value, _context: &ExecutionContext) -> ToolResult<Value> {
        let operation = args["operation"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidArgument("Missing operation".to_string()))?;

        let text = args["text"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidArgument("Missing text".to_string()))?;

        let result = match operation {
            "uppercase" => text.to_uppercase(),
            "lowercase" => text.to_lowercase(),
            "reverse" => text.chars().rev().collect(),
            "length" => return Ok(json!({ "length": text.len() })),
            "trim" => text.trim().to_string(),
            _ => {
                return Err(ToolError::InvalidArgument(format!(
                    "Unknown operation: {operation}"
                )));
            }
        };

        Ok(json!({
            "result": result,
            "operation": operation
        }))
    }
}
