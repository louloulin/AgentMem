//! AgentOrchestrator 集成测试
//!
//! 测试完整的对话循环，包括记忆检索、LLM 调用、记忆提取

use agent_mem_core::{
    orchestrator::{AgentOrchestrator, ChatRequest, OrchestratorConfig},
    engine::{MemoryEngine, MemoryEngineConfig},
    storage::{
        message_repository::MessageRepository,
        models::Message as StorageMessage,
        repository::Repository,
    },
    Memory, MemoryType,
};
use agent_mem_llm::LLMClient;
use agent_mem_tools::ToolExecutor;
use agent_mem_traits::{
    FunctionCall, FunctionCallResponse, FunctionDefinition, LLMConfig, LLMProvider, Message,
    ModelInfo, Result as TraitResult,
};
use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

/// Mock LLM Client for testing
struct MockLLMClient {
    response: String,
    should_fail: bool,
}

impl MockLLMClient {
    fn new(response: &str) -> Self {
        Self {
            response: response.to_string(),
            should_fail: false,
        }
    }

    fn with_failure() -> Self {
        Self {
            response: String::new(),
            should_fail: true,
        }
    }
}

#[async_trait::async_trait]
impl LLMProvider for MockLLMClient {
    async fn generate(&self, _messages: &[Message]) -> TraitResult<String> {
        if self.should_fail {
            return Err(agent_mem_traits::Error::Provider(
                "Mock LLM failure".to_string(),
            ));
        }
        Ok(self.response.clone())
    }

    async fn generate_with_functions(
        &self,
        _messages: &[Message],
        _functions: &[FunctionDefinition],
    ) -> TraitResult<FunctionCallResponse> {
        if self.should_fail {
            return Err(agent_mem_traits::Error::Provider(
                "Mock LLM failure".to_string(),
            ));
        }
        Ok(FunctionCallResponse {
            text: Some(self.response.clone()),
            function_calls: Vec::new(),
        })
    }

    async fn generate_stream(
        &self,
        _messages: &[Message],
    ) -> TraitResult<Box<dyn futures::Stream<Item = TraitResult<String>> + Send + Unpin>> {
        unimplemented!("Stream not needed for tests")
    }

    fn get_model_info(&self) -> ModelInfo {
        ModelInfo {
            provider: "mock".to_string(),
            model: "test-model".to_string(),
            max_tokens: 4096,
            supports_streaming: false,
            supports_functions: true,
        }
    }

    fn validate_config(&self) -> TraitResult<()> {
        Ok(())
    }
}

/// 辅助函数：创建测试用的数据库连接池
async fn create_test_pool() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/agentmem_test".to_string());

    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

/// 辅助函数：清理测试数据
async fn cleanup_test_data(pool: &PgPool, agent_id: &str) {
    // 删除测试消息
    let _ = sqlx::query("DELETE FROM messages WHERE agent_id = $1")
        .bind(agent_id)
        .execute(pool)
        .await;
}

#[tokio::test]
#[ignore] // 需要数据库连接，使用 --ignored 运行
async fn test_orchestrator_basic_conversation() {
    // 1. 设置测试环境
    let pool = create_test_pool().await;
    let agent_id = format!("test-agent-{}", Uuid::new_v4());
    let user_id = format!("test-user-{}", Uuid::new_v4());
    let org_id = format!("test-org-{}", Uuid::new_v4());

    // 2. 创建依赖组件
    let memory_engine = Arc::new(MemoryEngine::new(MemoryEngineConfig::default()));
    let message_repo = Arc::new(MessageRepository::new(pool.clone()));
    let mock_llm = Arc::new(MockLLMClient::new("Hello! How can I help you today?"));
    let llm_client = LLMClient::new(mock_llm);
    let tool_executor = Arc::new(ToolExecutor::new());

    // 3. 创建 AgentOrchestrator
    let config = OrchestratorConfig {
        max_memories: 10,
        auto_extract_memories: true,
        enable_tool_calling: false,
        max_tool_rounds: 5,
        memory_extraction_threshold: 0.5,
    };

    let orchestrator = AgentOrchestrator::new(
        config,
        memory_engine.clone(),
        message_repo.clone(),
        llm_client,
        tool_executor,
    );

    // 4. 创建聊天请求
    let request = ChatRequest {
        agent_id: agent_id.clone(),
        user_id: user_id.clone(),
        message: "What's the weather like?".to_string(),
        stream: false,
        max_memories: 10,
    };

    // 5. 执行对话循环
    let response = orchestrator
        .step(request)
        .await
        .expect("Failed to execute conversation step");

    // 6. 验证响应
    assert!(!response.message_id.is_empty(), "Message ID should not be empty");
    assert_eq!(
        response.content, "Hello! How can I help you today?",
        "Response content should match mock LLM output"
    );
    assert_eq!(
        response.memories_updated, true,
        "Memories should be updated when auto_extract is enabled"
    );

    // 7. 验证消息已保存
    let saved_message = message_repo
        .read(&response.message_id)
        .await
        .expect("Failed to read message")
        .expect("Message should exist");

    assert_eq!(saved_message.agent_id, agent_id);
    assert_eq!(saved_message.role, "assistant");

    // 8. 清理测试数据
    cleanup_test_data(&pool, &agent_id).await;

    println!("✅ test_orchestrator_basic_conversation passed");
}

#[tokio::test]
#[ignore] // 需要数据库连接
async fn test_orchestrator_with_memory_retrieval() {
    // 1. 设置测试环境
    let pool = create_test_pool().await;
    let agent_id = format!("test-agent-{}", Uuid::new_v4());
    let user_id = format!("test-user-{}", Uuid::new_v4());

    // 2. 创建依赖组件
    let memory_engine = Arc::new(MemoryEngine::new(MemoryEngineConfig::default()));
    let message_repo = Arc::new(MessageRepository::new(pool.clone()));
    let mock_llm = Arc::new(MockLLMClient::new(
        "Based on your previous preference, I recommend Italian food.",
    ));
    let llm_client = LLMClient::new(mock_llm);
    let tool_executor = Arc::new(ToolExecutor::new());

    // 3. 预先添加一些记忆
    let memory = Memory {
        id: Uuid::new_v4().to_string(),
        content: "User likes Italian food".to_string(),
        memory_type: MemoryType::Semantic,
        agent_id: Some(agent_id.clone()),
        user_id: Some(user_id.clone()),
        metadata: serde_json::json!({}),
        score: Some(0.9),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        ..Default::default()
    };

    memory_engine
        .add_memory(memory)
        .await
        .expect("Failed to add memory");

    // 4. 创建 AgentOrchestrator
    let config = OrchestratorConfig {
        max_memories: 10,
        auto_extract_memories: false, // 关闭自动提取以简化测试
        enable_tool_calling: false,
        max_tool_rounds: 5,
        memory_extraction_threshold: 0.5,
    };

    let orchestrator = AgentOrchestrator::new(
        config,
        memory_engine.clone(),
        message_repo.clone(),
        llm_client,
        tool_executor,
    );

    // 5. 创建聊天请求
    let request = ChatRequest {
        agent_id: agent_id.clone(),
        user_id: user_id.clone(),
        message: "What kind of food should I eat?".to_string(),
        stream: false,
        max_memories: 10,
    };

    // 6. 执行对话循环
    let response = orchestrator
        .step(request)
        .await
        .expect("Failed to execute conversation step");

    // 7. 验证响应
    assert!(!response.message_id.is_empty());
    assert!(response.content.contains("Italian"));

    // 8. 清理测试数据
    cleanup_test_data(&pool, &agent_id).await;

    println!("✅ test_orchestrator_with_memory_retrieval passed");
}

#[tokio::test]
#[ignore] // 需要数据库连接
async fn test_orchestrator_memory_extraction() {
    // 1. 设置测试环境
    let pool = create_test_pool().await;
    let agent_id = format!("test-agent-{}", Uuid::new_v4());
    let user_id = format!("test-user-{}", Uuid::new_v4());

    // 2. 创建依赖组件
    let memory_engine = Arc::new(MemoryEngine::new(MemoryEngineConfig::default()));
    let message_repo = Arc::new(MessageRepository::new(pool.clone()));

    // Mock LLM 返回包含可提取信息的响应
    let mock_llm = Arc::new(MockLLMClient::new(
        "I understand you prefer vegetarian food. I'll remember that for next time!",
    ));
    let llm_client = LLMClient::new(mock_llm);
    let tool_executor = Arc::new(ToolExecutor::new());

    // 3. 创建 AgentOrchestrator（启用自动记忆提取）
    let config = OrchestratorConfig {
        max_memories: 10,
        auto_extract_memories: true, // 启用自动提取
        enable_tool_calling: false,
        max_tool_rounds: 5,
        memory_extraction_threshold: 0.5,
    };

    let orchestrator = AgentOrchestrator::new(
        config,
        memory_engine.clone(),
        message_repo.clone(),
        llm_client,
        tool_executor,
    );

    // 4. 创建包含重要信息的聊天请求
    let request = ChatRequest {
        agent_id: agent_id.clone(),
        user_id: user_id.clone(),
        message: "I'm a vegetarian and I love pasta.".to_string(),
        stream: false,
        max_memories: 10,
    };

    // 5. 执行对话循环
    let response = orchestrator
        .step(request)
        .await
        .expect("Failed to execute conversation step");

    // 6. 验证响应
    assert!(!response.message_id.is_empty());
    assert_eq!(response.memories_updated, true, "Memories should be extracted");
    assert!(
        response.memories_count > 0,
        "At least one memory should be extracted"
    );

    // 7. 清理测试数据
    cleanup_test_data(&pool, &agent_id).await;

    println!("✅ test_orchestrator_memory_extraction passed");
}

#[tokio::test]
#[ignore] // 需要数据库连接
async fn test_orchestrator_error_handling() {
    // 1. 设置测试环境
    let pool = create_test_pool().await;
    let agent_id = format!("test-agent-{}", Uuid::new_v4());
    let user_id = format!("test-user-{}", Uuid::new_v4());

    // 2. 创建依赖组件（使用会失败的 Mock LLM）
    let memory_engine = Arc::new(MemoryEngine::new(MemoryEngineConfig::default()));
    let message_repo = Arc::new(MessageRepository::new(pool.clone()));
    let mock_llm = Arc::new(MockLLMClient::with_failure());
    let llm_client = LLMClient::new(mock_llm);
    let tool_executor = Arc::new(ToolExecutor::new());

    // 3. 创建 AgentOrchestrator
    let config = OrchestratorConfig::default();

    let orchestrator = AgentOrchestrator::new(
        config,
        memory_engine.clone(),
        message_repo.clone(),
        llm_client,
        tool_executor,
    );

    // 4. 创建聊天请求
    let request = ChatRequest {
        agent_id: agent_id.clone(),
        user_id: user_id.clone(),
        message: "This should fail".to_string(),
        stream: false,
        max_memories: 10,
    };

    // 5. 执行对话循环（应该失败）
    let result = orchestrator.step(request).await;

    // 6. 验证错误处理
    assert!(result.is_err(), "Should return error when LLM fails");

    // 7. 清理测试数据
    cleanup_test_data(&pool, &agent_id).await;

    println!("✅ test_orchestrator_error_handling passed");
}

#[tokio::test]
async fn test_memory_integrator_format_memories() {
    use agent_mem_core::orchestrator::memory_integration::{
        MemoryIntegrator, MemoryIntegratorConfig,
    };

    // 1. 创建 MemoryEngine 和 MemoryIntegrator
    let memory_engine = Arc::new(MemoryEngine::new(MemoryEngineConfig::default()));
    let config = MemoryIntegratorConfig::default();
    let integrator = MemoryIntegrator::new(memory_engine, config);

    // 2. 创建测试记忆
    let memories = vec![
        Memory {
            id: Uuid::new_v4().to_string(),
            content: "User likes coffee".to_string(),
            memory_type: MemoryType::Semantic,
            score: Some(0.9),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            ..Default::default()
        },
        Memory {
            id: Uuid::new_v4().to_string(),
            content: "User met John yesterday".to_string(),
            memory_type: MemoryType::Episodic,
            score: Some(0.8),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            ..Default::default()
        },
    ];

    // 3. 格式化记忆
    let formatted = integrator.format_memories_for_prompt(&memories);

    // 4. 验证格式化结果
    assert!(formatted.contains("Semantic"));
    assert!(formatted.contains("coffee"));
    assert!(formatted.contains("Episodic"));
    assert!(formatted.contains("John"));

    println!("✅ test_memory_integrator_format_memories passed");
}

#[tokio::test]
async fn test_memory_integrator_filter_by_relevance() {
    use agent_mem_core::orchestrator::memory_integration::{
        MemoryIntegrator, MemoryIntegratorConfig,
    };

    // 1. 创建 MemoryEngine 和 MemoryIntegrator
    let memory_engine = Arc::new(MemoryEngine::new(MemoryEngineConfig::default()));
    let config = MemoryIntegratorConfig {
        relevance_threshold: 0.7,
        ..Default::default()
    };
    let integrator = MemoryIntegrator::new(memory_engine, config);

    // 2. 创建测试记忆（不同的相关性分数）
    let memories = vec![
        Memory {
            id: Uuid::new_v4().to_string(),
            content: "High relevance".to_string(),
            memory_type: MemoryType::Semantic,
            score: Some(0.9),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            ..Default::default()
        },
        Memory {
            id: Uuid::new_v4().to_string(),
            content: "Low relevance".to_string(),
            memory_type: MemoryType::Semantic,
            score: Some(0.5),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            ..Default::default()
        },
        Memory {
            id: Uuid::new_v4().to_string(),
            content: "Medium relevance".to_string(),
            memory_type: MemoryType::Semantic,
            score: Some(0.75),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            ..Default::default()
        },
    ];

    // 3. 过滤记忆
    let filtered = integrator.filter_by_relevance(memories);

    // 4. 验证过滤结果（只保留 score >= 0.7 的记忆）
    assert_eq!(filtered.len(), 2, "Should keep 2 memories with score >= 0.7");
    assert!(filtered.iter().all(|m| m.score.unwrap_or(0.0) >= 0.7));

    println!("✅ test_memory_integrator_filter_by_relevance passed");
}

#[tokio::test]
async fn test_memory_integrator_sort_memories() {
    use agent_mem_core::orchestrator::memory_integration::{
        MemoryIntegrator, MemoryIntegratorConfig,
    };

    // 1. 创建 MemoryEngine 和 MemoryIntegrator
    let memory_engine = Arc::new(MemoryEngine::new(MemoryEngineConfig::default()));
    let config = MemoryIntegratorConfig::default();
    let integrator = MemoryIntegrator::new(memory_engine, config);

    // 2. 创建测试记忆（不同的分数）
    let memories = vec![
        Memory {
            id: Uuid::new_v4().to_string(),
            content: "Low score".to_string(),
            memory_type: MemoryType::Semantic,
            score: Some(0.5),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            ..Default::default()
        },
        Memory {
            id: Uuid::new_v4().to_string(),
            content: "High score".to_string(),
            memory_type: MemoryType::Semantic,
            score: Some(0.9),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            ..Default::default()
        },
        Memory {
            id: Uuid::new_v4().to_string(),
            content: "Medium score".to_string(),
            memory_type: MemoryType::Semantic,
            score: Some(0.7),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            ..Default::default()
        },
    ];

    // 3. 排序记忆
    let sorted = integrator.sort_memories(memories);

    // 4. 验证排序结果（按分数降序）
    assert_eq!(sorted.len(), 3);
    assert_eq!(sorted[0].content, "High score");
    assert_eq!(sorted[1].content, "Medium score");
    assert_eq!(sorted[2].content, "Low score");

    println!("✅ test_memory_integrator_sort_memories passed");
}

