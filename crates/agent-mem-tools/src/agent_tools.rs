//! Agent管理工具
//!
//! 提供Agent列表和信息查询功能

use crate::config::get_api_url;
use crate::error::{ToolError, ToolResult};
use crate::executor::{ExecutionContext, Tool};
use crate::schema::{PropertySchema, ToolSchema};
use async_trait::async_trait;
use serde_json::{json, Value};

/// 列出可用的Agent
pub struct ListAgentsTool;

#[async_trait]
impl Tool for ListAgentsTool {
    fn name(&self) -> &str {
        "agentmem_list_agents"
    }

    fn description(&self) -> &str {
        "列出AgentMem系统中所有可用的Agent，包括Agent ID、名称、描述和状态"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(self.name(), self.description())
            .add_parameter("user_id", PropertySchema::string("用户ID（可选）"), false)
            .add_parameter(
                "limit",
                PropertySchema::number("返回数量限制（默认20）"),
                false,
            )
    }

    async fn execute(&self, args: Value, _context: &ExecutionContext) -> ToolResult<Value> {
        let api_url = get_api_url();
        let url = format!("{api_url}/api/v1/agents");

        // 在spawn_blocking之前提取所有需要的值
        let user_id = args["user_id"].as_str().map(|s| s.to_string());
        let limit = args["limit"].as_i64().unwrap_or(20);

        tracing::debug!("Listing agents from: {}", url);

        // 使用spawn_blocking执行同步HTTP请求
        let response = tokio::task::spawn_blocking(move || {
            let mut req = ureq::get(&url);

            // 添加查询参数
            if let Some(ref uid) = user_id {
                req = req.query("user_id", uid);
            }
            req = req.query("limit", &limit.to_string());

            // 执行请求
            match req.call() {
                Ok(resp) => resp
                    .into_json::<Value>()
                    .map_err(|e| format!("Failed to parse response: {e}")),
                Err(ureq::Error::Status(code, resp)) => {
                    let text = resp
                        .into_string()
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    Err(format!("API returned error {code}: {text}"))
                }
                Err(e) => Err(format!("HTTP request failed: {e}")),
            }
        })
        .await
        .map_err(|e| ToolError::ExecutionFailed(format!("Task join error: {e}")))?
        .map_err(ToolError::ExecutionFailed)?;

        // 提取Agent列表
        let agents = response["data"].as_array().cloned().unwrap_or_default();

        // 格式化返回结果
        let formatted_agents: Vec<Value> = agents
            .iter()
            .map(|a| {
                json!({
                    "id": a["id"],
                    "name": a["name"],
                    "description": a["description"],
                    "user_id": a["user_id"],
                    "created_at": a["created_at"],
                    "is_active": a.get("is_active").unwrap_or(&json!(true))
                })
            })
            .collect();

        Ok(json!({
            "success": true,
            "total": formatted_agents.len(),
            "agents": formatted_agents
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_name() {
        let tool = ListAgentsTool;
        assert_eq!(tool.name(), "agentmem_list_agents");
    }

    #[test]
    fn test_tool_schema() {
        let tool = ListAgentsTool;
        let schema = tool.schema();
        assert_eq!(schema.name, "agentmem_list_agents");
        // ParameterSchema 使用 HashMap properties，不是 Vec
        assert_eq!(schema.parameters.properties.len(), 2);
    }

    #[test]
    fn test_tool_description() {
        let tool = ListAgentsTool;
        assert!(!tool.description().is_empty());
    }
}
