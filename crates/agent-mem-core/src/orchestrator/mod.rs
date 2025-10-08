//! Agent Orchestrator - 对话循环编排
//!
//! 这是 AgentMem 的核心对话循环实现，参考 MIRIX 的 AgentWrapper.step() 设计
//! 集成所有现有模块：MemoryEngine, LLMClient, ToolExecutor, MessageRepository

use crate::{
    engine::MemoryEngine,
    storage::message_repository::MessageRepository,
    Memory,
};
use agent_mem_llm::LLMClient;
use agent_mem_tools::{ToolExecutor, ExecutionContext};
use agent_mem_traits::{
    llm::{FunctionDefinition, FunctionCall},
    AgentMemError, Message, MessageRole, Result,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn, error};
use uuid::Uuid;

pub mod memory_integration;
pub mod tool_integration;
pub mod memory_extraction;

use memory_integration::MemoryIntegrator;
use memory_extraction::MemoryExtractor;
use tool_integration::{ToolIntegrator, ToolIntegratorConfig};

/// 对话请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    /// 用户消息
    pub message: String,
    
    /// Agent ID
    pub agent_id: String,
    
    /// 用户 ID
    pub user_id: String,
    
    /// 是否流式响应
    pub stream: bool,
    
    /// 最大记忆检索数量
    pub max_memories: usize,
}

/// 对话响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    /// 消息 ID
    pub message_id: String,
    
    /// Agent 响应内容
    pub content: String,
    
    /// 是否更新了记忆
    pub memories_updated: bool,
    
    /// 更新的记忆数量
    pub memories_count: usize,
    
    /// 工具调用（如果有）
    pub tool_calls: Option<Vec<ToolCallInfo>>,
}

/// 工具调用信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallInfo {
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub result: Option<String>,
}

/// Agent 编排器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    /// 最大工具调用轮数
    pub max_tool_rounds: usize,

    /// 最大记忆检索数量
    pub max_memories: usize,

    /// 是否自动提取记忆
    pub auto_extract_memories: bool,

    /// 记忆提取阈值
    pub memory_extraction_threshold: f32,

    /// 是否启用工具调用
    pub enable_tool_calling: bool,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            max_tool_rounds: 5,
            max_memories: 10,
            auto_extract_memories: true,
            memory_extraction_threshold: 0.5,
            enable_tool_calling: false, // 默认关闭，需要显式启用
        }
    }
}

/// Agent 编排器 - 核心对话循环
///
/// 参考 MIRIX 的 AgentWrapper.step() 实现
/// 集成所有现有模块实现完整的对话循环
pub struct AgentOrchestrator {
    config: OrchestratorConfig,
    memory_engine: Arc<MemoryEngine>,
    message_repo: Arc<MessageRepository>,
    llm_client: Arc<LLMClient>,
    tool_executor: Arc<ToolExecutor>,
    memory_integrator: MemoryIntegrator,
    memory_extractor: MemoryExtractor,
    tool_integrator: ToolIntegrator,
}

impl AgentOrchestrator {
    /// 创建新的编排器
    pub fn new(
        config: OrchestratorConfig,
        memory_engine: Arc<MemoryEngine>,
        message_repo: Arc<MessageRepository>,
        llm_client: Arc<LLMClient>,
        tool_executor: Arc<ToolExecutor>,
    ) -> Self {
        // 创建记忆集成器
        let memory_integrator = MemoryIntegrator::with_default_config(memory_engine.clone());

        // 创建记忆提取器
        let memory_extractor = MemoryExtractor::with_default_config(
            llm_client.clone(),
            memory_engine.clone(),
        );

        // 创建工具集成器
        let tool_config = ToolIntegratorConfig {
            max_tool_rounds: config.max_tool_rounds,
            tool_timeout_seconds: 30,
            allow_parallel_execution: false,
        };
        let tool_integrator = ToolIntegrator::new(tool_config, tool_executor.clone());

        Self {
            config,
            memory_engine,
            message_repo,
            llm_client,
            tool_executor,
            memory_integrator,
            memory_extractor,
            tool_integrator,
        }
    }

    /// 执行完整的对话循环
    ///
    /// 这是核心方法，参考 MIRIX 的 AgentWrapper.step() 实现：
    /// 1. 创建用户消息
    /// 2. 检索相关记忆
    /// 3. 构建 prompt（注入记忆）
    /// 4. 调用 LLM
    /// 5. 处理工具调用（如果有）- TODO: 待实现
    /// 6. 保存 assistant 消息
    /// 7. 提取和更新记忆
    /// 8. 返回响应
    pub async fn step(&self, request: ChatRequest) -> Result<ChatResponse> {
        info!("Starting conversation step for agent_id={}, user_id={}",
              request.agent_id, request.user_id);

        // 1. 创建用户消息
        let user_message_id = self.create_user_message(&request).await?;
        debug!("Created user message: {}", user_message_id);

        // 2. 检索相关记忆
        let memories = self.retrieve_memories(&request).await?;
        info!("Retrieved {} memories", memories.len());

        // 3. 构建 prompt（注入记忆）
        let messages = self.build_messages_with_memories(&request, &memories).await?;
        debug!("Built {} messages with memories", messages.len());

        // 4. 调用 LLM
        let response = self.llm_client.generate(&messages).await?;
        debug!("Got LLM response: {} chars", response.len());

        // 5. 处理工具调用（如果有）
        // TODO: 实现工具调用逻辑
        // 目前先跳过，后续实现
        let tool_calls_info = Vec::new();

        // 6. 保存 assistant 消息
        let assistant_message_id = self.create_assistant_message(
            &request.agent_id,
            &response,
        ).await?;
        debug!("Created assistant message: {}", assistant_message_id);

        // 7. 提取和更新记忆
        let memories_count = if self.config.auto_extract_memories {
            self.extract_and_update_memories(&request, &messages).await?
        } else {
            0
        };
        info!("Extracted and updated {} memories", memories_count);

        // 8. 返回响应
        Ok(ChatResponse {
            message_id: assistant_message_id,
            content: response,
            memories_updated: memories_count > 0,
            memories_count,
            tool_calls: if tool_calls_info.is_empty() {
                None
            } else {
                Some(tool_calls_info)
            },
        })
    }

    /// 执行带工具调用的对话循环
    ///
    /// 这个方法支持完整的工具调用流程：
    /// 1. 创建用户消息
    /// 2. 检索相关记忆
    /// 3. 构建 prompt（注入记忆）
    /// 4. 调用 LLM（带工具定义）
    /// 5. 如果有工具调用，执行工具并继续循环
    /// 6. 保存 assistant 消息
    /// 7. 提取和更新记忆
    /// 8. 返回响应
    pub async fn step_with_tools(
        &self,
        request: ChatRequest,
        available_tools: &[FunctionDefinition],
    ) -> Result<ChatResponse> {
        info!(
            "Starting conversation step with tools for agent_id={}, user_id={}",
            request.agent_id, request.user_id
        );

        // 1. 创建用户消息
        let user_message_id = self.create_user_message(&request).await?;
        debug!("Created user message: {}", user_message_id);

        // 2. 检索相关记忆
        let memories = self.retrieve_memories(&request).await?;
        info!("Retrieved {} memories", memories.len());

        // 3. 构建 prompt（注入记忆）
        let mut messages = self.build_messages_with_memories(&request, &memories).await?;
        debug!("Built {} messages with memories", messages.len());

        let mut tool_calls_info = Vec::new();
        let mut final_response = String::new();
        let mut round = 0;

        // 工具调用循环
        loop {
            round += 1;
            if round > self.config.max_tool_rounds {
                warn!("Reached max tool rounds ({}), stopping", self.config.max_tool_rounds);
                break;
            }

            // 4. 调用 LLM（带工具定义）
            let llm_response = self
                .llm_client
                .generate_with_functions(&messages, available_tools)
                .await?;

            // 检查是否有文本响应
            if let Some(text) = &llm_response.text {
                final_response = text.clone();
                debug!("Got LLM text response: {} chars", text.len());
            }

            // 检查是否有工具调用
            if llm_response.function_calls.is_empty() {
                debug!("No tool calls, ending loop");
                break;
            }

            info!("Got {} tool calls", llm_response.function_calls.len());

            // 5. 执行工具调用
            let tool_results = self
                .tool_integrator
                .execute_tool_calls(&llm_response.function_calls, &request.user_id)
                .await?;

            // 记录工具调用信息
            for result in &tool_results {
                tool_calls_info.push(ToolCallInfo {
                    tool_name: result.tool_name.clone(),
                    arguments: serde_json::from_str(&result.arguments)
                        .unwrap_or(serde_json::json!({})),
                    result: Some(result.result.clone()),
                });
            }

            // 将工具结果添加到消息历史
            let tool_results_text = self.tool_integrator.format_tool_results(&tool_results);
            messages.push(Message {
                role: agent_mem_traits::MessageRole::Assistant,
                content: tool_results_text,
                timestamp: Some(chrono::Utc::now()),
            });

            // 如果所有工具都失败了，停止循环
            if tool_results.iter().all(|r| !r.success) {
                warn!("All tools failed, stopping loop");
                break;
            }
        }

        // 6. 保存 assistant 消息
        let assistant_message_id = self
            .create_assistant_message(&request.agent_id, &final_response)
            .await?;
        debug!("Created assistant message: {}", assistant_message_id);

        // 7. 提取和更新记忆
        let memories_count = if self.config.auto_extract_memories {
            self.extract_and_update_memories(&request, &messages).await?
        } else {
            0
        };
        info!("Extracted and updated {} memories", memories_count);

        // 8. 返回响应
        Ok(ChatResponse {
            message_id: assistant_message_id,
            content: final_response,
            memories_updated: memories_count > 0,
            memories_count,
            tool_calls: if tool_calls_info.is_empty() {
                None
            } else {
                Some(tool_calls_info)
            },
        })
    }

    /// 创建用户消息
    async fn create_user_message(&self, request: &ChatRequest) -> Result<String> {
        // TODO: 调用 MessageRepository 创建消息
        // 这里需要等待 MessageRepository 的完整实现
        Ok(Uuid::new_v4().to_string())
    }

    /// 创建 assistant 消息
    async fn create_assistant_message(
        &self,
        agent_id: &str,
        content: &str,
    ) -> Result<String> {
        // TODO: 调用 MessageRepository 创建消息
        Ok(Uuid::new_v4().to_string())
    }

    /// 检索相关记忆
    async fn retrieve_memories(&self, request: &ChatRequest) -> Result<Vec<Memory>> {
        // 使用 MemoryIntegrator 检索记忆
        let max_count = self.config.max_memories;
        let memories = self.memory_integrator
            .retrieve_relevant_memories(&request.message, &request.agent_id, max_count)
            .await?;

        // 过滤和排序
        let memories = self.memory_integrator.filter_by_relevance(memories);
        let memories = self.memory_integrator.sort_memories(memories);

        Ok(memories)
    }

    /// 构建包含记忆的消息列表
    async fn build_messages_with_memories(
        &self,
        request: &ChatRequest,
        memories: &[Memory],
    ) -> Result<Vec<Message>> {
        let mut messages = Vec::new();

        // 添加系统消息（包含记忆）
        if !memories.is_empty() {
            let memory_context = self.memory_integrator.inject_memories_to_prompt(memories);
            messages.push(Message::system(&memory_context));
        }

        // 添加用户消息
        messages.push(Message::user(&request.message));

        Ok(messages)
    }

    /// 提取和更新记忆
    async fn extract_and_update_memories(
        &self,
        request: &ChatRequest,
        messages: &[Message],
    ) -> Result<usize> {
        // 使用 MemoryExtractor 提取记忆
        let memories = self.memory_extractor
            .extract_from_conversation(messages, &request.agent_id, &request.user_id)
            .await?;

        // 保存记忆
        let count = self.memory_extractor.save_memories(memories).await?;

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_request_creation() {
        let request = ChatRequest {
            message: "Hello, how are you?".to_string(),
            agent_id: "agent-123".to_string(),
            user_id: "user-456".to_string(),
            stream: false,
            max_memories: 10,
        };

        assert_eq!(request.message, "Hello, how are you?");
        assert_eq!(request.agent_id, "agent-123");
        assert_eq!(request.user_id, "user-456");
        assert!(!request.stream);
        assert_eq!(request.max_memories, 10);
    }

    #[test]
    fn test_chat_request_serialization() {
        let request = ChatRequest {
            message: "Test message".to_string(),
            agent_id: "agent-1".to_string(),
            user_id: "user-1".to_string(),
            stream: true,
            max_memories: 5,
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: ChatRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.message, deserialized.message);
        assert_eq!(request.stream, deserialized.stream);
    }

    #[test]
    fn test_chat_response_creation() {
        let response = ChatResponse {
            message_id: "msg-123".to_string(),
            content: "I'm doing well, thank you!".to_string(),
            memories_updated: true,
            memories_count: 3,
            tool_calls: None,
        };

        assert_eq!(response.message_id, "msg-123");
        assert!(response.memories_updated);
        assert_eq!(response.memories_count, 3);
        assert!(response.tool_calls.is_none());
    }

    #[test]
    fn test_chat_response_with_tool_calls() {
        let tool_call = ToolCallInfo {
            tool_name: "search".to_string(),
            arguments: serde_json::json!({"query": "weather"}),
            result: Some("Sunny, 25°C".to_string()),
        };

        let response = ChatResponse {
            message_id: "msg-456".to_string(),
            content: "The weather is sunny".to_string(),
            memories_updated: false,
            memories_count: 0,
            tool_calls: Some(vec![tool_call]),
        };

        assert!(response.tool_calls.is_some());
        assert_eq!(response.tool_calls.as_ref().unwrap().len(), 1);
        assert_eq!(response.tool_calls.as_ref().unwrap()[0].tool_name, "search");
    }

    #[test]
    fn test_tool_call_info_creation() {
        let tool_call = ToolCallInfo {
            tool_name: "calculator".to_string(),
            arguments: serde_json::json!({"operation": "add", "a": 5, "b": 3}),
            result: Some("8".to_string()),
        };

        assert_eq!(tool_call.tool_name, "calculator");
        assert!(tool_call.result.is_some());
        assert_eq!(tool_call.arguments["operation"], "add");
    }

    #[test]
    fn test_orchestrator_config_default() {
        let config = OrchestratorConfig::default();

        assert_eq!(config.max_tool_rounds, 5);
        assert_eq!(config.max_memories, 10);
        assert!(config.auto_extract_memories);
        assert_eq!(config.memory_extraction_threshold, 0.5);
        assert!(!config.enable_tool_calling);
    }

    #[test]
    fn test_orchestrator_config_custom() {
        let config = OrchestratorConfig {
            max_tool_rounds: 3,
            max_memories: 20,
            auto_extract_memories: false,
            memory_extraction_threshold: 0.7,
            enable_tool_calling: true,
        };

        assert_eq!(config.max_tool_rounds, 3);
        assert_eq!(config.max_memories, 20);
        assert!(!config.auto_extract_memories);
        assert_eq!(config.memory_extraction_threshold, 0.7);
        assert!(config.enable_tool_calling);
    }

    #[test]
    fn test_orchestrator_config_serialization() {
        let config = OrchestratorConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: OrchestratorConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.max_tool_rounds, deserialized.max_tool_rounds);
        assert_eq!(config.max_memories, deserialized.max_memories);
    }

    #[test]
    fn test_chat_request_with_empty_message() {
        let request = ChatRequest {
            message: "".to_string(),
            agent_id: "agent-1".to_string(),
            user_id: "user-1".to_string(),
            stream: false,
            max_memories: 5,
        };

        assert!(request.message.is_empty());
    }

    #[test]
    fn test_chat_request_with_long_message() {
        let long_message = "A".repeat(10000);
        let request = ChatRequest {
            message: long_message.clone(),
            agent_id: "agent-1".to_string(),
            user_id: "user-1".to_string(),
            stream: false,
            max_memories: 5,
        };

        assert_eq!(request.message.len(), 10000);
    }

    #[test]
    fn test_chat_response_serialization() {
        let response = ChatResponse {
            message_id: "msg-1".to_string(),
            content: "Response content".to_string(),
            memories_updated: true,
            memories_count: 2,
            tool_calls: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: ChatResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response.message_id, deserialized.message_id);
        assert_eq!(response.memories_updated, deserialized.memories_updated);
    }

    #[test]
    fn test_tool_call_info_serialization() {
        let tool_call = ToolCallInfo {
            tool_name: "test_tool".to_string(),
            arguments: serde_json::json!({"param": "value"}),
            result: Some("success".to_string()),
        };

        let json = serde_json::to_string(&tool_call).unwrap();
        let deserialized: ToolCallInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(tool_call.tool_name, deserialized.tool_name);
        assert_eq!(tool_call.result, deserialized.result);
    }

    #[tokio::test]
    async fn test_orchestrator_creation() {
        // TODO: 添加完整的集成测试
        // 需要 mock LLMClient, MemoryEngine, MessageRepository, ToolExecutor
    }
}

