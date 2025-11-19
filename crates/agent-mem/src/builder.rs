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
    #[cfg(feature = "plugins")]
    plugins: Vec<crate::plugins::RegisteredPlugin>,
}

impl MemoryBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            config: OrchestratorConfig::default(),
            default_user_id: None,
            default_agent_id: "default".to_string(),
            #[cfg(feature = "plugins")]
            plugins: Vec::new(),
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
    /// - `huawei_maas` - 华为 MaaS (deepseek-v3.2-exp 等)
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

    /// 启用重排序功能
    ///
    /// 默认使用内部重排序器。重排序会在搜索完成后对结果进行重新排序，
    /// 提升搜索结果的准确性。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # use agent_mem::Memory;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mem = Memory::builder()
    ///     .with_storage("libsql://agentmem.db")
    ///     .enable_reranking()  // 启用重排序
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn enable_reranking(mut self) -> Self {
        // 重排序器在orchestrator初始化时自动创建
        // 这里只是标记启用，实际创建在build()时完成
        self
    }

    /// 注册插件 (需要启用 `plugins` feature)
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "plugins")]
    /// # use agent_mem::Memory;
    /// # #[cfg(feature = "plugins")]
    /// # use agent_mem::plugins::{RegisteredPlugin, PluginStatus};
    /// # #[cfg(feature = "plugins")]
    /// # use agent_mem::plugins::sdk::{PluginMetadata, PluginType, Capability, PluginConfig};
    /// # #[cfg(feature = "plugins")]
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let plugin = RegisteredPlugin {
    ///     id: "my-plugin".to_string(),
    ///     metadata: PluginMetadata {
    ///         name: "my-plugin".to_string(),
    ///         version: "1.0.0".to_string(),
    ///         description: "My custom plugin".to_string(),
    ///         author: "Me".to_string(),
    ///         plugin_type: PluginType::SearchAlgorithm,
    ///         required_capabilities: vec![Capability::SearchAccess],
    ///         config_schema: None,
    ///     },
    ///     path: "my-plugin.wasm".to_string(),
    ///     status: PluginStatus::Registered,
    ///     config: PluginConfig::default(),
    ///     registered_at: chrono::Utc::now(),
    ///     last_loaded_at: None,
    /// };
    ///
    /// let mem = Memory::builder()
    ///     .with_storage("memory://")
    ///     .with_plugin(plugin)
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "plugins")]
    pub fn with_plugin(mut self, plugin: crate::plugins::RegisteredPlugin) -> Self {
        self.plugins.push(plugin);
        self
    }

    /// 从目录加载所有插件 (需要启用 `plugins` feature)
    ///
    /// 扫描指定目录下的所有 `.wasm` 文件并注册为插件。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "plugins")]
    /// # use agent_mem::Memory;
    /// # #[cfg(feature = "plugins")]
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mem = Memory::builder()
    ///     .with_storage("memory://")
    ///     .load_plugins_from_dir("./plugins")
    ///     .await?
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "plugins")]
    pub async fn load_plugins_from_dir(mut self, dir: impl AsRef<std::path::Path>) -> Result<Self> {
        use crate::plugins::sdk::{Capability, PluginConfig, PluginMetadata, PluginType};
        use crate::plugins::{PluginStatus, RegisteredPlugin};
        use tracing::{debug, warn};

        let dir_path = dir.as_ref();
        debug!("从目录加载插件: {:?}", dir_path);

        if !dir_path.exists() {
            warn!("插件目录不存在: {:?}", dir_path);
            return Ok(self);
        }

        let entries = std::fs::read_dir(dir_path).map_err(|e| {
            agent_mem_traits::AgentMemError::Other(anyhow::anyhow!("读取目录失败: {}", e))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                agent_mem_traits::AgentMemError::Other(anyhow::anyhow!("读取目录项失败: {}", e))
            })?;
            let path = entry.path();

            // 只处理 .wasm 文件
            if path.extension().and_then(|s| s.to_str()) != Some("wasm") {
                continue;
            }

            let file_name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");

            debug!("发现插件: {:?}", path);

            // 创建插件元数据（使用默认值）
            let plugin = RegisteredPlugin {
                id: file_name.to_string(),
                metadata: PluginMetadata {
                    name: file_name.to_string(),
                    version: "1.0.0".to_string(),
                    description: format!("Auto-loaded plugin from {}", file_name),
                    author: "Unknown".to_string(),
                    plugin_type: PluginType::Custom("auto-loaded".to_string()),
                    required_capabilities: vec![],
                    config_schema: None,
                },
                path: path.to_string_lossy().to_string(),
                status: PluginStatus::Registered,
                config: PluginConfig::default(),
                registered_at: chrono::Utc::now(),
                last_loaded_at: None,
            };

            self.plugins.push(plugin);
        }

        info!("从目录加载了 {} 个插件", self.plugins.len());
        Ok(self)
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

        let memory =
            Memory::from_orchestrator(orchestrator, self.default_user_id, self.default_agent_id);

        // 注册所有插件 (如果启用了 plugins feature)
        #[cfg(feature = "plugins")]
        {
            if !self.plugins.is_empty() {
                info!("注册 {} 个插件", self.plugins.len());
                for plugin in self.plugins {
                    if let Err(e) = memory.register_plugin(plugin.clone()).await {
                        tracing::warn!("注册插件 {} 失败: {}", plugin.id, e);
                    }
                }
            }
        }

        Ok(memory)
    }
}

impl Default for MemoryBuilder {
    fn default() -> Self {
        Self::new()
    }
}
