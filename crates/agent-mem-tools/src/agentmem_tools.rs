//! AgentMem 特定工具
//!
//! 这些工具提供 AgentMem 的核心功能，包括记忆管理、搜索、对话等

use crate::config::get_api_url;
use crate::error::{ToolError, ToolResult};
use crate::executor::{ExecutionContext, Tool};
use crate::schema::{PropertySchema, ToolSchema};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

/// 检查后端健康状态
async fn check_backend_health(api_url: &str) -> Result<(), String> {
    let url = format!("{}/health", api_url);
    let timeout = std::time::Duration::from_secs(5);

    let result = tokio::task::spawn_blocking(move || ureq::get(&url).timeout(timeout).call())
        .await
        .map_err(|e| format!("Join error: {}", e))?;

    match result {
        Ok(resp) if resp.status() == 200 => Ok(()),
        Ok(resp) => Err(format!("Backend unhealthy: status {}", resp.status())),
        Err(e) => Err(format!("Health check failed: {}", e)),
    }
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
            // 🆕 Phase 5: 新增scope_type参数（推荐）
            .add_parameter(
                "scope_type",
                PropertySchema::string("作用域类型（可选）：user, agent, run, session, organization。如不指定则根据其他参数自动判断"),
                false,
            )
            .add_parameter(
                "user_id",
                PropertySchema::string("用户 ID"),
                true,
            )
            .add_parameter(
                "agent_id",
                PropertySchema::string("Agent ID（可选，用于agent/run/session scope）"),
                false,
            )
            // 🆕 Phase 5: 新增run_id参数
            .add_parameter(
                "run_id",
                PropertySchema::string("Run ID（可选，用于run scope）"),
                false,
            )
            .add_parameter(
                "session_id",
                PropertySchema::string("会话 ID（可选，用于session scope）"),
                false,
            )
            // 🆕 Phase 5: 新增组织相关参数
            .add_parameter(
                "org_id",
                PropertySchema::string("Organization ID（可选，用于organization scope）"),
                false,
            )
            .add_parameter(
                "memory_type",
                PropertySchema::string("记忆类型（首字母必须大写）：Episodic, Semantic, Procedural, Factual, Core, Working, Resource, Knowledge, Contextual。默认：Episodic"),
                false,
            )
            .add_parameter(
                "metadata",
                PropertySchema::string("额外的元数据（JSON 字符串，可选）"),
                false,
            )
    }

    async fn execute(&self, args: Value, _context: &ExecutionContext) -> ToolResult<Value> {
        // 🆕 健康检查
        let api_url = get_api_url();

        if let Err(e) = check_backend_health(&api_url).await {
            tracing::warn!("Backend health check failed: {}", e);
            return Ok(json!({
                "success": false,
                "error": "backend_unavailable",
                "message": "AgentMem backend is currently unavailable. Please check if the service is running.",
                "details": e
            }));
        }

        let content = args["content"].as_str().ok_or_else(|| {
            crate::error::ToolError::InvalidArgument("content is required".to_string())
        })?;

        let user_id = args["user_id"].as_str().ok_or_else(|| {
            crate::error::ToolError::InvalidArgument("user_id is required".to_string())
        })?;

        // 🆕 Phase 5: 提取scope相关参数
        let scope_type = args["scope_type"].as_str().unwrap_or("auto");
        let agent_id_arg = args["agent_id"].as_str();
        let run_id = args["run_id"].as_str();
        let session_id = args["session_id"].as_str();
        let org_id = args["org_id"].as_str();

        // 🆕 Phase 5: 构建metadata（包含scope信息）
        let mut metadata_map = std::collections::HashMap::new();

        // 根据scope_type或自动推断
        let actual_scope_type = match scope_type {
            "user" => {
                metadata_map.insert("scope_type".to_string(), "user".to_string());
                "user"
            }
            "agent" => {
                metadata_map.insert("scope_type".to_string(), "agent".to_string());
                "agent"
            }
            "run" => {
                metadata_map.insert("scope_type".to_string(), "run".to_string());
                if let Some(rid) = run_id {
                    metadata_map.insert("run_id".to_string(), rid.to_string());
                }
                "run"
            }
            "session" => {
                metadata_map.insert("scope_type".to_string(), "session".to_string());
                if let Some(sid) = session_id {
                    metadata_map.insert("session_id".to_string(), sid.to_string());
                }
                "session"
            }
            "organization" => {
                metadata_map.insert("scope_type".to_string(), "organization".to_string());
                if let Some(oid) = org_id {
                    metadata_map.insert("org_id".to_string(), oid.to_string());
                }
                "organization"
            }
            "auto" | _ => {
                // 自动推断（当前逻辑）
                if let Some(rid) = run_id {
                    metadata_map.insert("scope_type".to_string(), "run".to_string());
                    metadata_map.insert("run_id".to_string(), rid.to_string());
                    "run"
                } else if let Some(sid) = session_id {
                    metadata_map.insert("scope_type".to_string(), "session".to_string());
                    metadata_map.insert("session_id".to_string(), sid.to_string());
                    "session"
                } else if agent_id_arg.is_some() {
                    metadata_map.insert("scope_type".to_string(), "agent".to_string());
                    "agent"
                } else {
                    metadata_map.insert("scope_type".to_string(), "user".to_string());
                    "user"
                }
            }
        };

        // 🆕 智能Agent ID处理：根据scope决定是否需要agent_id
        let agent_id = if actual_scope_type == "agent" || agent_id_arg.is_some() {
            agent_id_arg.map(|s| s.to_string()).unwrap_or_else(|| {
                std::env::var("AGENTMEM_DEFAULT_AGENT_ID")
                    .unwrap_or_else(|_| format!("agent-{}", user_id))
            })
        } else {
            format!("default-agent-{}", user_id)
        };

        let memory_type = args["memory_type"].as_str().unwrap_or("Episodic");

        // 合并用户提供的metadata
        if let Some(user_metadata_str) = args["metadata"].as_str() {
            if let Ok(user_metadata) =
                serde_json::from_str::<std::collections::HashMap<String, String>>(user_metadata_str)
            {
                metadata_map.extend(user_metadata);
            }
        }

        // 🆕 确保Agent存在（自动创建）- 仅当需要agent时
        if actual_scope_type == "agent" || agent_id_arg.is_some() {
            ensure_agent_exists(&api_url, &agent_id, user_id).await?;
        }

        // 调用 AgentMem Backend API (使用同步 HTTP 客户端避免 stdio 冲突)
        let api_url = get_api_url();
        let url = format!("{}/api/v1/memories", api_url);

        let request_body = json!({
            "content": content,
            "user_id": user_id,
            "agent_id": agent_id,
            "memory_type": memory_type,
            "importance": 0.5,
            "metadata": metadata_map  // 🆕 Phase 5: 包含scope信息的metadata
        });

        tracing::debug!("Calling API: POST {}", url);
        tracing::debug!(
            "Request body: {}",
            serde_json::to_string(&request_body).unwrap_or_default()
        );

        // 使用 spawn_blocking 运行同步 HTTP 请求
        let api_response = tokio::task::spawn_blocking(move || {
            let response = ureq::post(&url)
                .set("Content-Type", "application/json")
                .send_json(&request_body);

            match response {
                Ok(resp) => resp
                    .into_json::<Value>()
                    .map_err(|e| format!("Failed to parse response: {}", e)),
                Err(ureq::Error::Status(code, resp)) => {
                    let text = resp
                        .into_string()
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    Err(format!("API returned error {}: {}", code, text))
                }
                Err(e) => Err(format!("HTTP request failed: {}", e)),
            }
        })
        .await
        .map_err(|e| crate::error::ToolError::ExecutionFailed(format!("Task join error: {}", e)))?
        .map_err(|e| crate::error::ToolError::ExecutionFailed(e))?;

        // 提取 memory_id 从响应中
        let memory_id = api_response["data"]["id"]
            .as_str()
            .or_else(|| api_response["data"]["memory_id"].as_str())
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
            "scope_type": actual_scope_type,  // 🆕 Phase 5: 返回scope信息
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
            .add_parameter("query", PropertySchema::string("搜索查询"), true)
            .add_parameter("user_id", PropertySchema::string("用户 ID（可选）"), false)
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
        // 🆕 健康检查
        let api_url = get_api_url();

        if let Err(e) = check_backend_health(&api_url).await {
            tracing::warn!("Backend health check failed: {}", e);
            return Ok(json!({
                "success": false,
                "error": "backend_unavailable",
                "message": "AgentMem backend is currently unavailable. Please check if the service is running.",
                "details": e
            }));
        }

        let query = args["query"].as_str().ok_or_else(|| {
            crate::error::ToolError::InvalidArgument("query is required".to_string())
        })?;

        let limit = args["limit"].as_i64().unwrap_or(10) as usize;

        // 提取 user_id 参数（如果未提供，使用默认值"default"）
        let user_id = args["user_id"].as_str().unwrap_or("default");

        tracing::debug!(
            "Searching memories: query='{}', user_id='{}', limit={}",
            query,
            user_id,
            limit
        );
        let url = format!("{}/api/v1/memories/search", api_url);

        let request_body = json!({
            "query": query,
            "user_id": user_id,
            "limit": limit
        });

        tracing::debug!("Calling API: POST {}", url);

        // 使用 spawn_blocking 运行同步 HTTP 请求
        let api_response = tokio::task::spawn_blocking(move || {
            let response = ureq::post(&url)
                .set("Content-Type", "application/json")
                .send_json(&request_body);

            match response {
                Ok(resp) => resp
                    .into_json::<Value>()
                    .map_err(|e| format!("Failed to parse response: {}", e)),
                Err(ureq::Error::Status(code, resp)) => {
                    let text = resp
                        .into_string()
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    Err(format!("API returned error {}: {}", code, text))
                }
                Err(e) => Err(format!("HTTP request failed: {}", e)),
            }
        })
        .await
        .map_err(|e| crate::error::ToolError::ExecutionFailed(format!("Task join error: {}", e)))?
        .map_err(|e| crate::error::ToolError::ExecutionFailed(e))?;

        // 提取搜索结果
        // 注意：API返回的是 {"data": [...]}，不是 {"data": {"memories": [...]}}
        let memories = api_response["data"].as_array().cloned().unwrap_or_default();

        let results: Vec<Value> = memories
            .iter()
            .map(|mem| {
                json!({
                    "memory_id": mem["id"].as_str().unwrap_or("unknown"),
                    "content": mem["content"].as_str().unwrap_or(""),
                    "relevance_score": mem["score"].as_f64().unwrap_or(0.0),
                    "memory_type": mem["memory_type"].as_str().unwrap_or("Episodic"),
                    "timestamp": mem["created_at"].as_str().unwrap_or("")
                })
            })
            .collect();

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
            .add_parameter("message", PropertySchema::string("用户消息"), true)
            .add_parameter("user_id", PropertySchema::string("用户 ID"), true)
            .add_parameter(
                "agent_id",
                PropertySchema::string("Agent ID（可选，默认使用环境变量配置）"),
                false,
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
        // 🆕 健康检查
        let api_url = get_api_url();

        if let Err(e) = check_backend_health(&api_url).await {
            tracing::warn!("Backend health check failed: {}", e);
            return Ok(json!({
                "success": false,
                "error": "backend_unavailable",
                "message": "AgentMem backend is currently unavailable. Please check if the service is running.",
                "details": e
            }));
        }

        let message = args["message"].as_str().ok_or_else(|| {
            crate::error::ToolError::InvalidArgument("message is required".to_string())
        })?;

        let user_id = args["user_id"].as_str().ok_or_else(|| {
            crate::error::ToolError::InvalidArgument("user_id is required".to_string())
        })?;

        // 使用环境变量或默认 agent ID
        let default_agent = std::env::var("AGENTMEM_DEFAULT_AGENT_ID")
            .unwrap_or_else(|_| "agent-92070062-78bb-4553-9701-9a7a4a89d87a".to_string());
        let agent_id = &default_agent;

        // 调用 AgentMem Backend API (使用同步 HTTP 客户端)
        let api_url = get_api_url();
        let url = format!("{}/api/v1/agents/{}/chat", api_url, agent_id);

        let request_body = json!({
            "message": message,
            "user_id": user_id,
            "stream": false
        });

        tracing::debug!("Calling API: POST {}", url);

        // 使用 spawn_blocking 运行同步 HTTP 请求
        let api_response = tokio::task::spawn_blocking(move || {
            let response = ureq::post(&url)
                .set("Content-Type", "application/json")
                .send_json(&request_body);

            match response {
                Ok(resp) => resp
                    .into_json::<Value>()
                    .map_err(|e| format!("Failed to parse response: {}", e)),
                Err(ureq::Error::Status(code, resp)) => {
                    let text = resp
                        .into_string()
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    Err(format!("API returned error {}: {}", code, text))
                }
                Err(e) => Err(format!("HTTP request failed: {}", e)),
            }
        })
        .await
        .map_err(|e| crate::error::ToolError::ExecutionFailed(format!("Task join error: {}", e)))?
        .map_err(|e| crate::error::ToolError::ExecutionFailed(e))?;

        // 提取响应内容
        let response_content = api_response["data"]["content"]
            .as_str()
            .unwrap_or("No response")
            .to_string();

        let memories_count = api_response["data"]["memories_count"].as_u64().unwrap_or(0);

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
            .add_parameter("user_id", PropertySchema::string("用户 ID"), true)
            .add_parameter(
                "context",
                PropertySchema::string("上下文描述（可选）"),
                false,
            )
    }

    async fn execute(&self, args: Value, _context: &ExecutionContext) -> ToolResult<Value> {
        // 🆕 健康检查
        let api_url = get_api_url();

        if let Err(e) = check_backend_health(&api_url).await {
            tracing::warn!("Backend health check failed: {}", e);
            return Ok(json!({
                "success": false,
                "error": "backend_unavailable",
                "message": "AgentMem backend is currently unavailable. Please check if the service is running.",
                "details": e
            }));
        }

        let user_id = args["user_id"].as_str().ok_or_else(|| {
            crate::error::ToolError::InvalidArgument("user_id is required".to_string())
        })?;

        let context = args["context"].as_str().unwrap_or("");
        let url = format!("{}/api/v1/memories/search", api_url);

        let search_query = if !context.is_empty() {
            format!("用户偏好和背景信息 {}", context)
        } else {
            "用户偏好和背景信息".to_string()
        };

        let request_body = json!({
            "query": search_query,
            "limit": 10
        });

        tracing::debug!("Calling API: POST {}", url);

        // 使用 spawn_blocking 运行同步 HTTP 请求
        let api_response = tokio::task::spawn_blocking(move || {
            let response = ureq::post(&url)
                .set("Content-Type", "application/json")
                .send_json(&request_body);

            match response {
                Ok(resp) => resp
                    .into_json::<Value>()
                    .map_err(|e| format!("Failed to parse response: {}", e)),
                Err(ureq::Error::Status(code, resp)) => {
                    let text = resp
                        .into_string()
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    Err(format!("API returned error {}: {}", code, text))
                }
                Err(e) => Err(format!("HTTP request failed: {}", e)),
            }
        })
        .await
        .map_err(|e| crate::error::ToolError::ExecutionFailed(format!("Task join error: {}", e)))?
        .map_err(|e| crate::error::ToolError::ExecutionFailed(e))?;

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
    executor
        .register_tool(Arc::new(GetSystemPromptTool))
        .await?;

    // 🆕 注册Agent管理工具
    executor
        .register_tool(Arc::new(crate::agent_tools::ListAgentsTool))
        .await?;

    Ok(())
}

/// 🆕 确保Agent存在，如果不存在则自动创建
async fn ensure_agent_exists(api_url: &str, agent_id: &str, user_id: &str) -> ToolResult<()> {
    let check_url = format!("{}/api/v1/agents/{}", api_url, agent_id);

    // 1. 检查Agent是否存在
    let exists = tokio::task::spawn_blocking({
        let check_url = check_url.clone();
        move || match ureq::get(&check_url).call() {
            Ok(_) => true,
            Err(ureq::Error::Status(404, _)) => false,
            Err(e) => {
                tracing::warn!("Failed to check agent existence: {}", e);
                false
            }
        }
    })
    .await
    .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

    if exists {
        tracing::debug!("Agent {} already exists", agent_id);
        return Ok(());
    }

    // 2. Agent不存在，自动创建
    tracing::info!("🤖 Agent {} 不存在，自动创建", agent_id);

    let create_url = format!("{}/api/v1/agents", api_url);
    let create_body = json!({
        "id": agent_id,
        "name": format!("Auto Agent for {}", user_id),
        "description": "Automatically created agent for memory management via MCP",
        "user_id": user_id
    });

    let result = tokio::task::spawn_blocking({
        let create_url = create_url.clone();
        let create_body = create_body.clone();
        move || {
            ureq::post(&create_url)
                .set("Content-Type", "application/json")
                .send_json(&create_body)
        }
    })
    .await
    .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

    match result {
        Ok(_) => {
            tracing::info!("✅ Agent {} 创建成功", agent_id);
            Ok(())
        }
        Err(e) => {
            tracing::error!("❌ Agent {} 创建失败: {}", agent_id, e);
            Err(ToolError::ExecutionFailed(format!(
                "Failed to create agent: {}",
                e
            )))
        }
    }
}
