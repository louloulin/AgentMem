//! 工具集成模块 - 工具调用编排
//!
//! 参考 MIRIX 的工具调用实现，提供完整的工具执行流程：
//! 1. 解析 LLM 返回的工具调用
//! 2. 验证工具参数
//! 3. 执行工具
//! 4. 格式化工具结果
//! 5. 将结果返回给 LLM

use agent_mem_tools::{ExecutionContext, ToolExecutor};
use agent_mem_traits::{llm::FunctionCall, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// 工具调用结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResult {
    /// 工具名称
    pub tool_name: String,
    /// 工具参数（JSON 字符串）
    pub arguments: String,
    /// 执行结果（JSON 字符串）
    pub result: String,
    /// 是否成功
    pub success: bool,
    /// 错误信息（如果失败）
    pub error: Option<String>,
    /// 执行时间（毫秒）
    pub execution_time_ms: u64,
}

/// 工具集成器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolIntegratorConfig {
    /// 最大工具调用轮数
    pub max_tool_rounds: usize,
    /// 单个工具执行超时（秒）
    pub tool_timeout_seconds: u64,
    /// 是否允许并行执行工具
    pub allow_parallel_execution: bool,
}

impl Default for ToolIntegratorConfig {
    fn default() -> Self {
        Self {
            max_tool_rounds: 5,
            tool_timeout_seconds: 30,
            allow_parallel_execution: false,
        }
    }
}

/// 工具集成器
pub struct ToolIntegrator {
    /// 配置
    config: ToolIntegratorConfig,
    /// 工具执行器
    tool_executor: Arc<ToolExecutor>,
}

impl ToolIntegrator {
    /// 创建新的工具集成器
    pub fn new(config: ToolIntegratorConfig, tool_executor: Arc<ToolExecutor>) -> Self {
        Self {
            config,
            tool_executor,
        }
    }

    /// 执行工具调用
    ///
    /// 参考 MIRIX 的实现：
    /// - 解析工具调用
    /// - 执行工具
    /// - 格式化结果
    pub async fn execute_tool_calls(
        &self,
        function_calls: &[FunctionCall],
        user_id: &str,
    ) -> Result<Vec<ToolCallResult>> {
        if function_calls.is_empty() {
            return Ok(Vec::new());
        }

        info!(
            "Executing {} tool call(s) for user {}",
            function_calls.len(),
            user_id
        );

        let mut results = Vec::new();

        // 创建执行上下文
        let context = ExecutionContext {
            user: user_id.to_string(),
            timeout: Duration::from_secs(self.config.tool_timeout_seconds),
        };

        // 执行每个工具调用
        for (index, function_call) in function_calls.iter().enumerate() {
            debug!(
                "Executing tool {}/{}: {}",
                index + 1,
                function_calls.len(),
                function_call.name
            );

            let start_time = std::time::Instant::now();

            // 解析参数
            let args = match serde_json::from_str::<serde_json::Value>(&function_call.arguments) {
                Ok(args) => args,
                Err(e) => {
                    error!("Failed to parse tool arguments: {}", e);
                    results.push(ToolCallResult {
                        tool_name: function_call.name.clone(),
                        arguments: function_call.arguments.clone(),
                        result: String::new(),
                        success: false,
                        error: Some(format!("Invalid JSON arguments: {}", e)),
                        execution_time_ms: start_time.elapsed().as_millis() as u64,
                    });
                    continue;
                }
            };

            // 执行工具
            match self
                .tool_executor
                .execute_tool(&function_call.name, args, &context)
                .await
            {
                Ok(result) => {
                    let execution_time_ms = start_time.elapsed().as_millis() as u64;
                    info!(
                        "Tool {} executed successfully in {}ms",
                        function_call.name, execution_time_ms
                    );

                    results.push(ToolCallResult {
                        tool_name: function_call.name.clone(),
                        arguments: function_call.arguments.clone(),
                        result: result.to_string(),
                        success: true,
                        error: None,
                        execution_time_ms,
                    });
                }
                Err(e) => {
                    let execution_time_ms = start_time.elapsed().as_millis() as u64;
                    error!("Tool {} execution failed: {}", function_call.name, e);

                    results.push(ToolCallResult {
                        tool_name: function_call.name.clone(),
                        arguments: function_call.arguments.clone(),
                        result: String::new(),
                        success: false,
                        error: Some(e.to_string()),
                        execution_time_ms,
                    });
                }
            }
        }

        Ok(results)
    }

    /// 格式化工具结果为 LLM 消息
    ///
    /// 将工具执行结果格式化为可以发送给 LLM 的消息格式
    pub fn format_tool_results(&self, results: &[ToolCallResult]) -> String {
        if results.is_empty() {
            return String::new();
        }

        let mut formatted = String::from("Tool execution results:\n\n");

        for (index, result) in results.iter().enumerate() {
            formatted.push_str(&format!("{}. Tool: {}\n", index + 1, result.tool_name));

            if result.success {
                formatted.push_str(&format!("   Status: ✅ Success\n"));
                formatted.push_str(&format!("   Result: {}\n", result.result));
            } else {
                formatted.push_str(&format!("   Status: ❌ Failed\n"));
                if let Some(error) = &result.error {
                    formatted.push_str(&format!("   Error: {}\n", error));
                }
            }

            formatted.push_str(&format!(
                "   Execution time: {}ms\n\n",
                result.execution_time_ms
            ));
        }

        formatted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_integrator_config_default() {
        let config = ToolIntegratorConfig::default();
        assert_eq!(config.max_tool_rounds, 5);
        assert_eq!(config.tool_timeout_seconds, 30);
        assert_eq!(config.allow_parallel_execution, false);
    }

    #[test]
    fn test_format_tool_results_empty() {
        let config = ToolIntegratorConfig::default();
        let executor = Arc::new(ToolExecutor::new());
        let integrator = ToolIntegrator::new(config, executor);

        let formatted = integrator.format_tool_results(&[]);
        assert_eq!(formatted, "");
    }

    #[test]
    fn test_format_tool_results_success() {
        let config = ToolIntegratorConfig::default();
        let executor = Arc::new(ToolExecutor::new());
        let integrator = ToolIntegrator::new(config, executor);

        let results = vec![ToolCallResult {
            tool_name: "calculator".to_string(),
            arguments: r#"{"operation": "add", "a": 1, "b": 2}"#.to_string(),
            result: r#"{"result": 3}"#.to_string(),
            success: true,
            error: None,
            execution_time_ms: 10,
        }];

        let formatted = integrator.format_tool_results(&results);
        assert!(formatted.contains("calculator"));
        assert!(formatted.contains("Success"));
        assert!(formatted.contains("10ms"));
    }
}
