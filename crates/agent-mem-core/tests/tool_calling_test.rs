//! 工具调用集成测试

use agent_mem_core::orchestrator::{AgentOrchestrator, ChatRequest, OrchestratorConfig};
use agent_mem_core::{
    engine::{MemoryEngine, MemoryEngineConfig},
    storage::message_repository::MessageRepository,
};
use agent_mem_llm::LLMClient;
use agent_mem_tools::{ExecutionContext, Tool, ToolExecutor, ToolResult, ToolSchema};
use agent_mem_traits::{
    llm::{FunctionCall, FunctionCallResponse, FunctionDefinition},
    LLMConfig, LLMProvider, Message, MessageRole, ModelInfo, Result,
};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio;

/// Mock LLM Provider for testing
struct MockLLMProvider {
    response: String,
    function_calls: Vec<FunctionCall>,
}

#[async_trait]
impl LLMProvider for MockLLMProvider {
    async fn generate(&self, _messages: &[Message]) -> Result<String> {
        Ok(self.response.clone())
    }

    async fn generate_stream(
        &self,
        _messages: &[Message],
    ) -> Result<Box<dyn futures::Stream<Item = Result<String>> + Send + Unpin>> {
        unimplemented!("Streaming not needed for tests")
    }

    fn get_model_info(&self) -> ModelInfo {
        ModelInfo {
            provider: "mock".to_string(),
            model: "mock-model".to_string(),
            max_tokens: 1000,
            supports_streaming: false,
            supports_functions: true,
        }
    }

    async fn generate_with_functions(
        &self,
        _messages: &[Message],
        _functions: &[FunctionDefinition],
    ) -> Result<FunctionCallResponse> {
        Ok(FunctionCallResponse {
            text: Some(self.response.clone()),
            function_calls: self.function_calls.clone(),
        })
    }

    fn validate_config(&self) -> Result<()> {
        Ok(())
    }
}

/// Mock Calculator Tool for testing
struct CalculatorTool;

#[async_trait]
impl Tool for CalculatorTool {
    fn name(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "Perform basic arithmetic operations"
    }

    fn schema(&self) -> ToolSchema {
        use agent_mem_tools::schema::{ParameterSchema, PropertySchema};
        use std::collections::HashMap;

        let mut properties = HashMap::new();
        properties.insert(
            "operation".to_string(),
            PropertySchema {
                prop_type: "string".to_string(),
                description: "The operation to perform".to_string(),
                enum_values: Some(vec![
                    "add".to_string(),
                    "subtract".to_string(),
                    "multiply".to_string(),
                    "divide".to_string(),
                ]),
                default: None,
                minimum: None,
                maximum: None,
                items: None,
            },
        );
        properties.insert(
            "a".to_string(),
            PropertySchema {
                prop_type: "number".to_string(),
                description: "First number".to_string(),
                enum_values: None,
                default: None,
                minimum: None,
                maximum: None,
                items: None,
            },
        );
        properties.insert(
            "b".to_string(),
            PropertySchema {
                prop_type: "number".to_string(),
                description: "Second number".to_string(),
                enum_values: None,
                default: None,
                minimum: None,
                maximum: None,
                items: None,
            },
        );

        ToolSchema {
            name: "calculator".to_string(),
            description: "Perform basic arithmetic operations".to_string(),
            parameters: ParameterSchema {
                param_type: "object".to_string(),
                properties,
                required: vec!["operation".to_string(), "a".to_string(), "b".to_string()],
            },
        }
    }

    async fn execute(&self, args: Value, _context: &ExecutionContext) -> ToolResult<Value> {
        let operation = args["operation"].as_str().unwrap();
        let a = args["a"].as_f64().unwrap();
        let b = args["b"].as_f64().unwrap();

        let result = match operation {
            "add" => a + b,
            "subtract" => a - b,
            "multiply" => a * b,
            "divide" => {
                if b == 0.0 {
                    return Err(agent_mem_tools::ToolError::InvalidArgument(
                        "Division by zero".to_string(),
                    ));
                }
                a / b
            }
            _ => {
                return Err(agent_mem_tools::ToolError::InvalidArgument(format!(
                    "Unknown operation: {}",
                    operation
                )))
            }
        };

        Ok(json!({ "result": result }))
    }
}

#[tokio::test]
async fn test_tool_calling_integration() {
    // 1. 创建 Mock LLM Provider（返回工具调用）
    let _mock_provider = MockLLMProvider {
        response: "I'll calculate that for you.".to_string(),
        function_calls: vec![FunctionCall {
            name: "calculator".to_string(),
            arguments: json!({
                "operation": "add",
                "a": 10,
                "b": 20
            })
            .to_string(),
        }],
    };

    // 2. 创建 ToolExecutor 并注册工具
    let tool_executor = Arc::new(ToolExecutor::new());
    tool_executor
        .register_tool(Arc::new(CalculatorTool))
        .await
        .unwrap();

    // 3. 创建 MemoryEngine（使用默认配置）
    let memory_config = MemoryEngineConfig::default();
    let _memory_engine = Arc::new(MemoryEngine::new(memory_config));

    // 4. 创建 MessageRepository（Mock）
    // TODO: 使用真实的 MessageRepository
    // 暂时使用 placeholder

    // 5. 创建 LLMClient（使用 Mock Provider）
    // TODO: 需要修改 LLMClient 以支持注入 Mock Provider
    // 暂时跳过这个测试

    println!("Tool calling integration test setup complete");
    println!("Note: Full integration test requires LLMClient mock support");
}

#[tokio::test]
async fn test_calculator_tool() {
    // 测试计算器工具
    let tool = CalculatorTool;
    let context = ExecutionContext {
        user: "test_user".to_string(),
        timeout: std::time::Duration::from_secs(30),
    };

    // 测试加法
    let result = tool
        .execute(
            json!({
                "operation": "add",
                "a": 10,
                "b": 20
            }),
            &context,
        )
        .await
        .unwrap();

    assert_eq!(result["result"], 30.0);

    // 测试减法
    let result = tool
        .execute(
            json!({
                "operation": "subtract",
                "a": 50,
                "b": 20
            }),
            &context,
        )
        .await
        .unwrap();

    assert_eq!(result["result"], 30.0);

    // 测试乘法
    let result = tool
        .execute(
            json!({
                "operation": "multiply",
                "a": 5,
                "b": 6
            }),
            &context,
        )
        .await
        .unwrap();

    assert_eq!(result["result"], 30.0);

    // 测试除法
    let result = tool
        .execute(
            json!({
                "operation": "divide",
                "a": 60,
                "b": 2
            }),
            &context,
        )
        .await
        .unwrap();

    assert_eq!(result["result"], 30.0);

    // 测试除以零错误
    let result = tool
        .execute(
            json!({
                "operation": "divide",
                "a": 10,
                "b": 0
            }),
            &context,
        )
        .await;

    assert!(result.is_err());
}

#[test]
fn test_function_definition_creation() {
    // 测试函数定义创建
    let function_def = FunctionDefinition {
        name: "calculator".to_string(),
        description: "Perform arithmetic operations".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "operation": { "type": "string" },
                "a": { "type": "number" },
                "b": { "type": "number" }
            }
        }),
    };

    assert_eq!(function_def.name, "calculator");
    assert!(function_def.parameters.is_object());
}

#[test]
fn test_function_call_parsing() {
    // 测试函数调用解析
    let function_call = FunctionCall {
        name: "calculator".to_string(),
        arguments: json!({
            "operation": "add",
            "a": 10,
            "b": 20
        })
        .to_string(),
    };

    assert_eq!(function_call.name, "calculator");

    // 解析参数
    let args: Value = serde_json::from_str(&function_call.arguments).unwrap();
    assert_eq!(args["operation"], "add");
    assert_eq!(args["a"], 10);
    assert_eq!(args["b"], 20);
}
