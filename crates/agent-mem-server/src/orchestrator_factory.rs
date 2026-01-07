//! AgentOrchestrator 工厂模块
//!
//! 提供创建 AgentOrchestrator 实例的工厂方法，
//! 供 Chat API 和 Agents API 共享使用。

use crate::error::{ServerError, ServerResult};
use agent_mem_core::{
    engine::{MemoryEngine, MemoryEngineConfig},
    orchestrator::{AgentOrchestrator, OrchestratorConfig},
    storage::factory::Repositories,
    storage::models::Agent,
};
use agent_mem_llm::LLMClient;
use agent_mem_tools::ToolExecutor;
use agent_mem_traits::LLMConfig;
use std::sync::Arc;
use tracing::{debug, error, info};

/// 从 Agent 配置中解析 LLM 配置
pub fn parse_llm_config(agent: &Agent) -> ServerResult<LLMConfig> {
    // agent.llm_config 是 Option<JSON> 格式
    let llm_config_value = agent
        .llm_config
        .clone()
        .ok_or_else(|| ServerError::bad_request("Agent LLM config not set"))?;

    let mut llm_config: LLMConfig = serde_json::from_value(llm_config_value)
        .map_err(|e| ServerError::internal_error(format!("Failed to parse LLM config: {}", e)))?;

    // 验证必要字段
    if llm_config.provider.is_empty() {
        return Err(ServerError::bad_request("LLM provider not configured"));
    }

    if llm_config.model.is_empty() {
        return Err(ServerError::bad_request("LLM model not configured"));
    }

    // 🔧 从环境变量读取API key（如果配置中没有）
    if llm_config.api_key.is_none() {
        let env_var_name = format!("{}_API_KEY", llm_config.provider.to_uppercase());
        if let Ok(api_key) = std::env::var(&env_var_name) {
            debug!("Loaded API key from environment variable: {}", env_var_name);
            llm_config.api_key = Some(api_key);
        } else {
            debug!("No API key found in environment variable: {}", env_var_name);
        }
    }

    debug!(
        "Parsed LLM config: provider={}, model={}, has_api_key={}",
        llm_config.provider,
        llm_config.model,
        llm_config.api_key.is_some()
    );

    Ok(llm_config)
}

/// 创建 AgentOrchestrator 实例
///
/// 这个函数创建一个完整配置的 AgentOrchestrator，包括：
/// - LLMClient（基于 agent 的 llm_config）
/// - MemoryEngine（用于记忆管理）
/// - MessageRepository（用于消息存储）
/// - ToolExecutor（用于工具调用）
///
/// # Arguments
///
/// * `agent` - Agent 实例，包含 LLM 配置
/// * `repositories` - 数据仓库集合
///
/// # Returns
///
/// 返回配置好的 AgentOrchestrator 实例
pub async fn create_orchestrator(
    agent: &Agent,
    repositories: &Arc<Repositories>,
) -> ServerResult<AgentOrchestrator> {
    info!("Creating AgentOrchestrator for agent: {}", agent.id);

    // 1. 解析 LLM 配置
    let llm_config = parse_llm_config(agent)?;

    // 2. 创建 LLMClient
    let llm_client =
        Arc::new(LLMClient::new(&llm_config).map_err(|e| {
            ServerError::internal_error(format!("Failed to create LLM client: {}", e))
        })?);
    debug!("Created LLMClient with provider: {}", llm_config.provider);

    // 3. 创建 MemoryEngine（注入 LibSQL memory_repository 以支持持久化搜索）
    let memory_engine_config = MemoryEngineConfig::default();
    let memory_repository = repositories.memories.clone();
    let memory_engine = Arc::new(MemoryEngine::with_repository(
        memory_engine_config,
        memory_repository,
    ));
    info!("Created MemoryEngine with LibSQL repository for persistent memory search");

    // 4. 获取 MessageRepository
    let message_repo = repositories.messages.clone();
    debug!("Got MessageRepository");

    // 5. 创建 ToolExecutor 并注册内置工具
    let tool_executor = Arc::new(ToolExecutor::new());
    debug!("Created ToolExecutor");

    // 注册所有内置工具
    use agent_mem_tools::builtin::register_all_builtin_tools;
    register_all_builtin_tools(&tool_executor)
        .await
        .map_err(|e| {
            error!("Failed to register builtin tools: {}", e);
            ServerError::internal_error(format!("Failed to register builtin tools: {}", e))
        })?;
    info!("Registered all builtin tools");

    // 6. 创建 OrchestratorConfig
    let orchestrator_config = OrchestratorConfig {
        max_tool_rounds: 5,
        max_memories: 10,
        auto_extract_memories: true,
        memory_extraction_threshold: 0.5,
        enable_tool_calling: true, // ✅ 启用工具调用
        enable_adaptive: false,    // ✅ Task 1.1-1.2优化后可启用
        token_budget: 8000,        // ✅ Token预算
        ttfb_threshold_ms: 500,    // ✅ TTFB阈值（毫秒）
    };
    debug!("Created OrchestratorConfig: {:?}", orchestrator_config);

    // 7. 获取 Working Memory Store（统一抽象，与其他 repositories 平级）
    // ✅ 直接从 repositories 获取，保持抽象层次一致
    let working_store = Some(repositories.working_memory.clone());
    debug!("✅ Got WorkingMemoryStore from repositories (uses unified memories table)");

    // 8. 创建 AgentOrchestrator
    let orchestrator = AgentOrchestrator::new(
        orchestrator_config,
        memory_engine,
        message_repo,
        llm_client,
        tool_executor,
        working_store,
    );

    info!(
        "Successfully created AgentOrchestrator with Working Memory support for agent: {}",
        agent.id
    );
    Ok(orchestrator)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_agent(llm_config: Option<serde_json::Value>) -> Agent {
        Agent {
            id: "test-agent".to_string(),
            organization_id: "test-org".to_string(),
            agent_type: Some("chat".to_string()),
            name: Some("Test Agent".to_string()),
            description: Some("Test".to_string()),
            system: None,
            topic: None,
            message_ids: None,
            metadata_: None,
            llm_config,
            embedding_config: None,
            tool_rules: None,
            mcp_tools: None,
            state: Some("idle".to_string()),
            last_active_at: None,
            error_message: None,
            is_deleted: false,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            created_by_id: Some("test-user".to_string()),
            last_updated_by_id: Some("test-user".to_string()),
        }
    }

    #[test]
    fn test_parse_llm_config_success() {
        let agent = create_test_agent(Some(json!({
            "provider": "openai",
            "model": "gpt-4",
            "api_key": "test-key"
        })));

        let result = parse_llm_config(&agent);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.provider, "openai");
        assert_eq!(config.model, "gpt-4");
    }

    #[test]
    fn test_parse_llm_config_missing_provider() {
        let agent = create_test_agent(Some(json!({
            "provider": "",
            "model": "gpt-4"
        })));

        let result = parse_llm_config(&agent);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_llm_config_missing_model() {
        let agent = create_test_agent(Some(json!({
            "provider": "openai",
            "model": ""
        })));

        let result = parse_llm_config(&agent);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_llm_config_none() {
        let agent = create_test_agent(None);

        let result = parse_llm_config(&agent);
        assert!(result.is_err());
    }
}
