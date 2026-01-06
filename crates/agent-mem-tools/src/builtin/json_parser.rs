//! JSON parser tool - parse and validate JSON

use crate::schema::PropertySchema;
use crate::{ExecutionContext, Tool, ToolError, ToolResult, ToolSchema};
use async_trait::async_trait;
use serde_json::{json, Value};

/// JSON parser tool
pub struct JsonParserTool;

#[async_trait]
impl Tool for JsonParserTool {
    fn name(&self) -> &str {
        "json_parser"
    }

    fn description(&self) -> &str {
        "Parse and validate JSON strings"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(self.name(), self.description()).add_parameter(
            "json_string",
            PropertySchema::string("JSON string to parse"),
            true,
        )
    }

    fn category(&self) -> &str {
        "data"
    }

    async fn execute(&self, args: Value, _context: &ExecutionContext) -> ToolResult<Value> {
        let json_string = args["json_string"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidArgument("Missing json_string".to_string()))?;

        let parsed: Value = serde_json::from_str(json_string)
            .map_err(|e| ToolError::ExecutionFailed(format!("Invalid JSON: {e}")))?;

        Ok(json!({
            "parsed": parsed,
            "valid": true
        }))
    }
}
