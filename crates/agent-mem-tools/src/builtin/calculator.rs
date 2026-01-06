//! Calculator tool - perform basic arithmetic operations

use crate::schema::PropertySchema;
use crate::{ExecutionContext, Tool, ToolError, ToolResult, ToolSchema};
use async_trait::async_trait;
use serde_json::{json, Value};

/// Calculator tool
pub struct CalculatorTool;

#[async_trait]
impl Tool for CalculatorTool {
    fn name(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "Perform basic arithmetic operations (add, subtract, multiply, divide, power, sqrt)"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(self.name(), self.description())
            .add_parameter(
                "operation",
                PropertySchema::string("The operation to perform").with_enum(vec![
                    "add".to_string(),
                    "subtract".to_string(),
                    "multiply".to_string(),
                    "divide".to_string(),
                    "power".to_string(),
                    "sqrt".to_string(),
                ]),
                true,
            )
            .add_parameter("a", PropertySchema::number("First operand"), true)
            .add_parameter(
                "b",
                PropertySchema::number("Second operand (not required for sqrt)"),
                false,
            )
    }

    fn category(&self) -> &str {
        "math"
    }

    async fn execute(&self, args: Value, _context: &ExecutionContext) -> ToolResult<Value> {
        let operation = args["operation"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidArgument("Missing operation".to_string()))?;

        let a = args["a"]
            .as_f64()
            .ok_or_else(|| ToolError::InvalidArgument("Invalid operand 'a'".to_string()))?;

        let result = match operation {
            "add" => {
                let b = args["b"]
                    .as_f64()
                    .ok_or_else(|| ToolError::InvalidArgument("Missing operand 'b'".to_string()))?;
                a + b
            }
            "subtract" => {
                let b = args["b"]
                    .as_f64()
                    .ok_or_else(|| ToolError::InvalidArgument("Missing operand 'b'".to_string()))?;
                a - b
            }
            "multiply" => {
                let b = args["b"]
                    .as_f64()
                    .ok_or_else(|| ToolError::InvalidArgument("Missing operand 'b'".to_string()))?;
                a * b
            }
            "divide" => {
                let b = args["b"]
                    .as_f64()
                    .ok_or_else(|| ToolError::InvalidArgument("Missing operand 'b'".to_string()))?;
                if b == 0.0 {
                    return Err(ToolError::ExecutionFailed("Division by zero".to_string()));
                }
                a / b
            }
            "power" => {
                let b = args["b"]
                    .as_f64()
                    .ok_or_else(|| ToolError::InvalidArgument("Missing operand 'b'".to_string()))?;
                a.powf(b)
            }
            "sqrt" => {
                if a < 0.0 {
                    return Err(ToolError::ExecutionFailed(
                        "Cannot take square root of negative number".to_string(),
                    ));
                }
                a.sqrt()
            }
            _ => {
                return Err(ToolError::InvalidArgument(format!(
                    "Unknown operation: {operation}"
                )));
            }
        };

        Ok(json!({
            "result": result,
            "operation": operation,
            "operands": if operation == "sqrt" {
                json!([a])
            } else {
                json!([a, args["b"]])
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_calculator_add() {
        let tool = CalculatorTool;
        let context = ExecutionContext {
            user: "test".to_string(),
            timeout: Duration::from_secs(30),
        };

        let result = tool
            .execute(
                json!({
                    "operation": "add",
                    "a": 10.0,
                    "b": 20.0
                }),
                &context,
            )
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output["result"], 30.0);
    }

    #[tokio::test]
    async fn test_calculator_divide_by_zero() {
        let tool = CalculatorTool;
        let context = ExecutionContext {
            user: "test".to_string(),
            timeout: Duration::from_secs(30),
        };

        let result = tool
            .execute(
                json!({
                    "operation": "divide",
                    "a": 10.0,
                    "b": 0.0
                }),
                &context,
            )
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_calculator_sqrt() {
        let tool = CalculatorTool;
        let context = ExecutionContext {
            user: "test".to_string(),
            timeout: Duration::from_secs(30),
        };

        let result = tool
            .execute(
                json!({
                    "operation": "sqrt",
                    "a": 16.0
                }),
                &context,
            )
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output["result"], 4.0);
    }
}
