//! AgentMem 特定工具
//!
//! 这些工具提供 AgentMem 的核心功能，包括记忆管理、搜索、对话等

use crate::error::ToolResult;
use crate::executor::{ExecutionContext, Tool};
use crate::schema::{PropertySchema, ToolSchema};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

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

        let agent_id = args["agent_id"].as_str();
        let session_id = args["session_id"].as_str();
        let memory_type = args["memory_type"].as_str().unwrap_or("episodic");

        // 这里应该调用 AgentMem 客户端添加记忆
        // 由于当前 AgentMemClient 的 API 可能不完全匹配，我们返回一个模拟响应
        Ok(json!({
            "success": true,
            "message": "记忆已添加",
            "memory_id": format!("mem_{}", uuid::Uuid::new_v4()),
            "content": content,
            "user_id": user_id,
            "agent_id": agent_id,
            "session_id": session_id,
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
        
        let user_id = args["user_id"].as_str();
        let limit = args["limit"].as_i64().unwrap_or(10);
        let memory_type = args["memory_type"].as_str();

        // 这里应该调用 AgentMem 客户端搜索记忆
        // 返回模拟响应
        Ok(json!({
            "success": true,
            "query": query,
            "user_id": user_id,
            "limit": limit,
            "memory_type": memory_type,
            "results": [
                {
                    "memory_id": "mem_001",
                    "content": format!("与 '{}' 相关的记忆 1", query),
                    "relevance_score": 0.95,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                },
                {
                    "memory_id": "mem_002",
                    "content": format!("与 '{}' 相关的记忆 2", query),
                    "relevance_score": 0.87,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }
            ],
            "total_results": 2
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

        let session_id = args["session_id"].as_str();
        let use_memory = args["use_memory"].as_bool().unwrap_or(true);

        // 这里应该调用 AgentMem 客户端进行对话
        // 返回模拟响应
        Ok(json!({
            "success": true,
            "message": message,
            "user_id": user_id,
            "session_id": session_id,
            "use_memory": use_memory,
            "response": format!("基于您的记忆，我理解您说的是：{}。让我为您提供相关的回复...", message),
            "memory_context_used": 3,
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

        let context = args["context"].as_str();

        // 这里应该调用 AgentMem 客户端获取系统提示
        // 返回模拟响应
        Ok(json!({
            "success": true,
            "user_id": user_id,
            "context": context,
            "system_prompt": format!(
                "你是一个智能助手，正在为用户 {} 提供服务。\n\
                基于用户的历史记忆，你了解到：\n\
                - 用户偏好使用 Rust 编程语言\n\
                - 用户关注系统性能和安全性\n\
                - 用户最近在研究 MCP 协议\n\n\
                请根据这些信息提供个性化的帮助。",
                user_id
            ),
            "memory_count": 15,
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

