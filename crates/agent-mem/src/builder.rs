//! Memory Builder - 流畅的配置接口

use crate::memory::Memory;
use crate::orchestrator::{MemoryOrchestrator, OrchestratorConfig};
use agent_mem_traits::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Memory 构建器
///
/// 提供流畅的 API 来配置 Memory 实例
///
/// # 示例
///
/// ```rust,no_run
/// use agent_mem::Memory;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mem = Memory::builder()
///         .with_storage("libsql://agentmem.db")
///         .with_llm("openai", "gpt-4")
///         .with_embedder("openai", "text-embedding-3-small")
///         .enable_intelligent_features()
///         .build()
///         .await?;
///     Ok(())
/// }
/// ```
pub struct MemoryBuilder {
    config: OrchestratorConfig,
    default_user_id: Option<String>,
    default_agent_id: String,
}

impl MemoryBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            config: OrchestratorConfig::default(),
            default_user_id: None,
            default_agent_id: "default".to_string(),
        }
    }

    /// 配置存储后端
    ///
    /// 支持的 URL 格式：
    /// - `memory://` - 内存存储 (开发测试)
    /// - `libsql://path/to/db` - LibSQL (推荐)
    /// - `libsql://path/to/db?mode=file` - LibSQL 文件模式
    /// - `postgres://user:pass@host/db` - PostgreSQL (企业级)
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # use agent_mem::Memory;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mem = Memory::builder()
    ///     .with_storage("libsql://agentmem.db")
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_storage(mut self, url: impl Into<String>) -> Self {
        self.config.storage_url = Some(url.into());
        self
    }

    /// 配置 LLM 提供商
    ///
    /// 支持的提供商：
    /// - `openai` - OpenAI (GPT-4, GPT-3.5)
    /// - `anthropic` - Anthropic (Claude)
    /// - `deepseek` - DeepSeek
    /// - `ollama` - Ollama (本地模型)
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # use agent_mem::Memory;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mem = Memory::builder()
    ///     .with_llm("openai", "gpt-4")
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_llm(mut self, provider: impl Into<String>, model: impl Into<String>) -> Self {
        self.config.llm_provider = Some(provider.into());
        self.config.llm_model = Some(model.into());
        self
    }

    /// 配置 Embedder
    ///
    /// 支持的提供商：
    /// - `openai` - OpenAI (text-embedding-3-small, text-embedding-3-large)
    /// - `ollama` - Ollama (本地嵌入模型)
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # use agent_mem::Memory;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mem = Memory::builder()
    ///     .with_embedder("openai", "text-embedding-3-small")
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_embedder(mut self, provider: impl Into<String>, model: impl Into<String>) -> Self {
        self.config.embedder_provider = Some(provider.into());
        self.config.embedder_model = Some(model.into());
        self
    }

    /// 配置向量存储
    ///
    /// 支持的 URL 格式：
    /// - `lancedb://path/to/db` - LanceDB (默认)
    /// - `qdrant://host:port` - Qdrant
    /// - `pinecone://api-key` - Pinecone
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # use agent_mem::Memory;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mem = Memory::builder()
    ///     .with_vector_store("lancedb://./vector_db")
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_vector_store(mut self, url: impl Into<String>) -> Self {
        self.config.vector_store_url = Some(url.into());
        self
    }

    /// 启用智能功能
    ///
    /// 启用后将自动：
    /// - 提取事实
    /// - 智能决策 (ADD/UPDATE/DELETE)
    /// - 记忆去重
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # use agent_mem::Memory;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mem = Memory::builder()
    ///     .enable_intelligent_features()
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn enable_intelligent_features(mut self) -> Self {
        self.config.enable_intelligent_features = true;
        self
    }

    /// 禁用智能功能
    ///
    /// 禁用后将使用基础模式，不进行事实提取和智能决策。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # use agent_mem::Memory;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mem = Memory::builder()
    ///     .disable_intelligent_features()
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn disable_intelligent_features(mut self) -> Self {
        self.config.enable_intelligent_features = false;
        self
    }

    /// 设置默认用户
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # use agent_mem::Memory;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mem = Memory::builder()
    ///     .with_user("alice")
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_user(mut self, user_id: impl Into<String>) -> Self {
        self.default_user_id = Some(user_id.into());
        self
    }

    /// 设置默认 Agent
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # use agent_mem::Memory;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mem = Memory::builder()
    ///     .with_agent("my-agent")
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_agent(mut self, agent_id: impl Into<String>) -> Self {
        self.default_agent_id = agent_id.into();
        self
    }

    /// 构建 Memory 实例
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # use agent_mem::Memory;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mem = Memory::builder()
    ///     .with_storage("libsql://agentmem.db")
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn build(self) -> Result<Memory> {
        info!("构建 Memory 实例");
        info!("配置: {:?}", self.config);

        let orchestrator = MemoryOrchestrator::new_with_config(self.config).await?;

        Ok(Memory::from_orchestrator(
            orchestrator,
            self.default_user_id,
            self.default_agent_id,
        ))
    }
}

impl Default for MemoryBuilder {
    fn default() -> Self {
        Self::new()
    }
}
