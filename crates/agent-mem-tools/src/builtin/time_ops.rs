//! Time operations tool

use crate::schema::PropertySchema;
use crate::{ExecutionContext, Tool, ToolError, ToolResult, ToolSchema};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::{json, Value};

/// Time operations tool
pub struct TimeOpsTool;

#[async_trait]
impl Tool for TimeOpsTool {
    fn name(&self) -> &str {
        "time_ops"
    }

    fn description(&self) -> &str {
        "Perform time operations (current_time, parse_time, format_time)"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(self.name(), self.description())
            .add_parameter(
                "operation",
                PropertySchema::string("The operation to perform").with_enum(vec![
                    "current_time".to_string(),
                    "parse_time".to_string(),
                    "format_time".to_string(),
                ]),
                true,
            )
            .add_parameter(
                "time_string",
                PropertySchema::string("Time string (for parse_time)"),
                false,
            )
            .add_parameter(
                "format",
                PropertySchema::string("Format string (for format_time)"),
                false,
            )
    }

    fn category(&self) -> &str {
        "utility"
    }

    async fn execute(&self, args: Value, _context: &ExecutionContext) -> ToolResult<Value> {
        let operation = args["operation"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidArgument("Missing operation".to_string()))?;

        match operation {
            "current_time" => {
                let now = Utc::now();
                Ok(json!({
                    "timestamp": now.to_rfc3339(),
                    "unix_timestamp": now.timestamp(),
                    "operation": operation
                }))
            }
            "parse_time" => {
                let time_string = args["time_string"]
                    .as_str()
                    .ok_or_else(|| ToolError::InvalidArgument("Missing time_string".to_string()))?;

                let parsed: DateTime<Utc> = time_string
                    .parse()
                    .map_err(|e| ToolError::ExecutionFailed(format!("Invalid time format: {e}")))?;

                Ok(json!({
                    "timestamp": parsed.to_rfc3339(),
                    "unix_timestamp": parsed.timestamp(),
                    "operation": operation
                }))
            }
            "format_time" => {
                let time_string = args["time_string"]
                    .as_str()
                    .ok_or_else(|| ToolError::InvalidArgument("Missing time_string".to_string()))?;

                let format = args["format"].as_str().unwrap_or("%Y-%m-%d %H:%M:%S");

                let parsed: DateTime<Utc> = time_string
                    .parse()
                    .map_err(|e| ToolError::ExecutionFailed(format!("Invalid time format: {e}")))?;

                Ok(json!({
                    "formatted": parsed.format(format).to_string(),
                    "operation": operation
                }))
            }
            _ => Err(ToolError::InvalidArgument(format!(
                "Unknown operation: {operation}"
            ))),
        }
    }
}
