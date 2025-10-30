//! AgentMem 特定工具
//!
//! 这些工具提供 AgentMem 的核心功能，包括记忆管理、搜索、对话等

use crate::error::ToolResult;
use crate::executor::{ExecutionContext, Tool};
use crate::schema::{PropertySchema, ToolSchema};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

/// Get AgentMem API URL from environment or use default
fn get_api_url() -> String {
    std::env::var("AGENTMEM_API_URL").unwrap_or_else(|_| "http://localhost:8080".to_string())
}

/// 添加记忆工具
pub struct AddMemoryTool;

#[async_trait]
impl Tool for AddMemoryTool {
    fn name(&self) -> &str {
        "agentmem_add_memory"
    }

    fn description(&self) -> &str {
        "添加一条新的记忆到 AgentMem 系统中"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(self.name(), self.description())
            .add_parameter(
                "content",
                PropertySchema::string("记忆内容"),
                true,
            )
            .add_parameter(
                "user_id",
                PropertySchema::string("用户 ID"),
                true,
            )
            .add_parameter(
                "agent_id",
                PropertySchema::string("Agent ID（可选）"),
                false,
            )
            .add_parameter(
                "session_id",
                PropertySchema::string("会话 ID（可选）"),
                false,
            )
            .add_parameter(
                "memory_type",
                PropertySchema::string("记忆类型：episodic, semantic, procedural, core, working, resource, declarative, contextual"),
                false,
            )
    }

    async fn execute(&self, args: Value, _context: &ExecutionContext) -> ToolResult<Value> {
        let content = args["content"]
            .as_str()
            .ok_or_else(|| crate::error::ToolError::InvalidArgument("content is required".to_string()))?;

        let user_id = args["user_id"]
            .as_str()
            .ok_or_else(|| crate::error::ToolError::InvalidArgument("user_id is required".to_string()))?;

        // 如果没有提供 agent_id，使用环境变量或默认值
        let default_agent = std::env::var("AGENTMEM_DEFAULT_AGENT_ID")
            .unwrap_or_else(|_| "agent-92070062-78bb-4553-9701-9a7a4a89d87a".to_string());
        let agent_id = args["agent_id"].as_str().unwrap_or(&default_agent);
        let memory_type = args["memory_type"].as_str().unwrap_or("Episodic");

        // 调用 AgentMem Backend API
        let api_url = get_api_url();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| crate::error::ToolError::ExecutionFailed(format!("Failed to create HTTP client: {}", e)))?;

        let request_body = json!({
            "content": content,
            "user_id": user_id,
            "agent_id": agent_id,
            "memory_type": memory_type,
            "importance": 0.5
        });

        let url = format!("{}/api/v1/memories", api_url);
        tracing::debug!("Calling API: POST {}", url);
        tracing::debug!("Request body: {}", serde_json::to_string(&request_body).unwrap_or_default());

        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| crate::error::ToolError::ExecutionFailed(format!("Failed to call API: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(crate::error::ToolError::ExecutionFailed(
                format!("API returned error {}: {}", status, error_text)
            ));
        }

        let api_response: Value = response.json().await
            .map_err(|e| crate::error::ToolError::ExecutionFailed(format!("Failed to parse response: {}", e)))?;

        // 提取 memory_id 从响应中
        let memory_id = api_response["data"]["memory_id"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        Ok(json!({
            "success": true,
            "message": "记忆已添加",
            "memory_id": memory_id,
            "content": content,
            "user_id": user_id,
            "agent_id": agent_id,
            "memory_type": memory_type,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }
}

/// 搜索记忆工具
pub struct SearchMemoriesTool;

#[async_trait]
impl Tool for SearchMemoriesTool {
    fn name(&self) -> &str {
        "agentmem_search_memories"
    }

    fn description(&self) -> &str {
        "在 AgentMem 系统中搜索相关记忆"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(self.name(), self.description())
            .add_parameter(
                "query",
                PropertySchema::string("搜索查询"),
                true,
            )
            .add_parameter(
                "user_id",
                PropertySchema::string("用户 ID（可选）"),
                false,
            )
            .add_parameter(
                "limit",
                PropertySchema::number("返回结果数量限制（默认 10）"),
                false,
            )
            .add_parameter(
                "memory_type",
                PropertySchema::string("记忆类型过滤（可选）"),
                false,
            )
    }

    async fn execute(&self, args: Value, _context: &ExecutionContext) -> ToolResult<Value> {
        let query = args["query"]
            .as_str()
            .ok_or_else(|| crate::error::ToolError::InvalidArgument("query is required".to_string()))?;

        let limit = args["limit"].as_i64().unwrap_or(10) as usize;

        // 调用 AgentMem Backend API
        let api_url = get_api_url();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| crate::error::ToolError::ExecutionFailed(format!("Failed to create HTTP client: {}", e)))?;

        let request_body = json!({
            "query": query,
            "limit": limit
        });

        let url = format!("{}/api/v1/memories/search", api_url);
        tracing::debug!("Calling API: POST {}", url);

        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| crate::error::ToolError::ExecutionFailed(format!("Failed to call API: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(crate::error::ToolError::ExecutionFailed(
                format!("API returned error {}: {}", status, error_text)
            ));
        }

        let api_response: Value = response.json().await
            .map_err(|e| crate::error::ToolError::ExecutionFailed(format!("Failed to parse response: {}", e)))?;

        // 提取搜索结果
        let memories = api_response["data"]["memories"]
            .as_array()
            .cloned()
            .unwrap_or_default();

        let results: Vec<Value> = memories.iter().map(|mem| {
            json!({
                "memory_id": mem["id"].as_str().unwrap_or("unknown"),
                "content": mem["content"].as_str().unwrap_or(""),
                "relevance_score": mem["score"].as_f64().unwrap_or(0.0),
                "memory_type": mem["memory_type"].as_str().unwrap_or("Episodic"),
                "timestamp": mem["created_at"].as_str().unwrap_or("")
            })
        }).collect();

        Ok(json!({
            "success": true,
            "query": query,
            "limit": limit,
            "results": results,
            "total_results": results.len()
        }))
    }
}

/// 智能对话工具
pub struct ChatTool;

#[async_trait]
impl Tool for ChatTool {
    fn name(&self) -> &str {
        "agentmem_chat"
    }

    fn description(&self) -> &str {
        "与 AgentMem 进行智能对话，基于记忆上下文生成回复"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(self.name(), self.description())
            .add_parameter(
                "message",
                PropertySchema::string("用户消息"),
                true,
            )
            .add_parameter(
                "user_id",
                PropertySchema::string("用户 ID"),
                true,
            )
            .add_parameter(
                "session_id",
                PropertySchema::string("会话 ID（可选）"),
                false,
            )
            .add_parameter(
                "use_memory",
                PropertySchema::boolean("是否使用记忆上下文（默认 true）"),
                false,
            )
    }

    async fn execute(&self, args: Value, _context: &ExecutionContext) -> ToolResult<Value> {
        let message = args["message"]
            .as_str()
            .ok_or_else(|| crate::error::ToolError::InvalidArgument("message is required".to_string()))?;

        let user_id = args["user_id"]
            .as_str()
            .ok_or_else(|| crate::error::ToolError::InvalidArgument("user_id is required".to_string()))?;

        // 使用环境变量或默认 agent ID
        let default_agent = std::env::var("AGENTMEM_DEFAULT_AGENT_ID")
            .unwrap_or_else(|_| "agent-92070062-78bb-4553-9701-9a7a4a89d87a".to_string());
        let agent_id = &default_agent;

        // 调用 AgentMem Backend API
        let api_url = get_api_url();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| crate::error::ToolError::ExecutionFailed(format!("Failed to create HTTP client: {}", e)))?;

        let request_body = json!({
            "message": message,
            "user_id": user_id,
            "stream": false
        });

        let url = format!("{}/api/v1/agents/{}/chat", api_url, agent_id);
        tracing::debug!("Calling API: POST {}", url);

        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| crate::error::ToolError::ExecutionFailed(format!("Failed to call API: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(crate::error::ToolError::ExecutionFailed(
                format!("API returned error {}: {}", status, error_text)
            ));
        }

        let api_response: Value = response.json().await
            .map_err(|e| crate::error::ToolError::ExecutionFailed(format!("Failed to parse response: {}", e)))?;

        // 提取响应内容
        let response_content = api_response["data"]["content"]
            .as_str()
            .unwrap_or("No response")
            .to_string();

        let memories_count = api_response["data"]["memories_count"]
            .as_u64()
            .unwrap_or(0);

        Ok(json!({
            "success": true,
            "message": message,
            "user_id": user_id,
            "response": response_content,
            "memory_context_used": memories_count,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }
}

/// 获取系统提示工具
pub struct GetSystemPromptTool;

#[async_trait]
impl Tool for GetSystemPromptTool {
    fn name(&self) -> &str {
        "agentmem_get_system_prompt"
    }

    fn description(&self) -> &str {
        "获取基于用户记忆的系统提示词"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(self.name(), self.description())
            .add_parameter(
                "user_id",
                PropertySchema::string("用户 ID"),
                true,
            )
            .add_parameter(
                "context",
                PropertySchema::string("上下文描述（可选）"),
                false,
            )
    }

    async fn execute(&self, args: Value, _context: &ExecutionContext) -> ToolResult<Value> {
        let user_id = args["user_id"]
            .as_str()
            .ok_or_else(|| crate::error::ToolError::InvalidArgument("user_id is required".to_string()))?;

        let context = args["context"].as_str().unwrap_or("");

        // 调用 AgentMem Backend API 搜索用户记忆
        let api_url = get_api_url();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| crate::error::ToolError::ExecutionFailed(format!("Failed to create HTTP client: {}", e)))?;

        let search_query = if !context.is_empty() {
            format!("用户偏好和背景信息 {}", context)
        } else {
            "用户偏好和背景信息".to_string()
        };

        let request_body = json!({
            "query": search_query,
            "limit": 10
        });

        let url = format!("{}/api/v1/memories/search", api_url);
        tracing::debug!("Calling API: POST {}", url);

        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| crate::error::ToolError::ExecutionFailed(format!("Failed to call API: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(crate::error::ToolError::ExecutionFailed(
                format!("API returned error {}: {}", status, error_text)
            ));
        }

        let api_response: Value = response.json().await
            .map_err(|e| crate::error::ToolError::ExecutionFailed(format!("Failed to parse response: {}", e)))?;

        // 提取记忆内容
        let memories = api_response["data"]["memories"]
            .as_array()
            .cloned()
            .unwrap_or_default();

        let memory_count = memories.len();

        // 构建系统提示
        let mut system_prompt = format!("你是一个智能助手，正在为用户 {} 提供服务。\n", user_id);

        if memory_count > 0 {
            system_prompt.push_str("\n基于用户的历史记忆，你了解到：\n");
            for (i, mem) in memories.iter().take(5).enumerate() {
                if let Some(content) = mem["content"].as_str() {
                    system_prompt.push_str(&format!("{}. {}\n", i + 1, content));
                }
            }
        } else {
            system_prompt.push_str("\n这是你与该用户的首次交互。\n");
        }

        system_prompt.push_str("\n请根据这些信息提供个性化的帮助。");

        Ok(json!({
            "success": true,
            "user_id": user_id,
            "context": context,
            "system_prompt": system_prompt,
            "memory_count": memory_count,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }
}

/// 注册所有 AgentMem 工具
pub async fn register_agentmem_tools(executor: &crate::executor::ToolExecutor) -> ToolResult<()> {
    executor.register_tool(Arc::new(AddMemoryTool)).await?;
    executor.register_tool(Arc::new(SearchMemoriesTool)).await?;
    executor.register_tool(Arc::new(ChatTool)).await?;
    executor.register_tool(Arc::new(GetSystemPromptTool)).await?;
    Ok(())
}

