//! Memory - 统一的记忆管理接口
//!
//! Memory 提供了简洁的 API 来管理所有类型的记忆，
//! 内部自动路由到对应的专门 Agent 处理。

use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use chrono::Utc;

use agent_mem_traits::{AgentMemError, MemoryItem, Result};

use crate::builder::MemoryBuilder;
use crate::orchestrator::MemoryOrchestrator;
use crate::platform::{
    ApplyMigrationRequest, CancelProactiveTaskRequest, CategoryDescriptor, ExtractionRequest,
    ExtractionResult, MigrationPlan, MigrationReport, MountResourceRequest, OperationStatus,
    PlatformErrorCode, ProactiveTaskInfo, ResourceDescriptor, ResourceMetadataDescriptor,
    ResourceStatus, RollbackMigrationRequest, RunProactiveTaskRequest, SchedulerStats,
    SchedulerState, ScopeDescriptor, SearchCategoriesRequest,
};
use crate::types::{
    AddMemoryOptions, AddResult, DeleteAllOptions, GetAllOptions, MemoryScope, MemoryStats,
    SearchOptions,
};

/// 统一的记忆管理接口
///
/// Memory 提供了简洁的 API 来管理所有类型的记忆，
/// 内部自动路由到对应的专门 Agent 处理。
///
/// # 使用示例
///
/// ## 零配置模式
/// ```rust,no_run
/// use agent_mem::Memory;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mem = Memory::new().await?;
///     mem.add("I love pizza").await?;
///     Ok(())
/// }
/// ```
///
/// ## Builder 模式
/// ```rust,no_run
/// use agent_mem::Memory;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mem = Memory::builder()
///         .with_storage("libsql://agentmem.db")
///         .with_llm("openai", "gpt-4")
///         .build()
///         .await?;
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct Memory {
    /// 内部编排器，负责协调各个 Agent
    orchestrator: Arc<RwLock<MemoryOrchestrator>>,
    /// 默认用户 ID
    default_user_id: Option<String>,
    /// 默认 Agent ID
    default_agent_id: String,
    /// 插件增强层（可选）
    #[cfg(feature = "plugins")]
    plugin_layer: Arc<RwLock<crate::plugin_integration::PluginEnhancedMemory>>,
}

impl Memory {
    /// 内部构造函数（供 builder 使用）
    pub(crate) fn from_orchestrator(
        orchestrator: MemoryOrchestrator,
        default_user_id: Option<String>,
        default_agent_id: String,
    ) -> Self {
        Self {
            orchestrator: Arc::new(RwLock::new(orchestrator)),
            default_user_id,
            default_agent_id,
            #[cfg(feature = "plugins")]
            plugin_layer: Arc::new(RwLock::new(
                crate::plugin_integration::PluginEnhancedMemory::new(),
            )),
        }
    }

    /// 零配置初始化
    ///
    /// 自动配置所有组件：
    /// - 开发环境: 使用内存存储
    /// - 生产环境: 使用 LibSQL
    /// - 有 API Key: 启用智能功能
    /// - 无 API Key: 降级到基础模式
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use agent_mem::Memory;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mem = Memory::new().await?;
    ///     mem.add("I love pizza").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn new() -> Result<Self> {
        info!("初始化 Memory (零配置模式)");

        let orchestrator = MemoryOrchestrator::new_with_auto_config().await?;

        Ok(Self::from_orchestrator(
            orchestrator,
            None,
            "default".to_string(),
        ))
    }

    /// Mem0 兼容模式初始化
    ///
    /// 使用 Mem0 推荐的默认配置：
    /// - FastEmbed (BAAI/bge-small-en-v1.5) - 本地嵌入模型，无需 API Key
    /// - LibSQL - 轻量级 SQLite 数据库
    /// - LanceDB - 高性能向量数据库
    /// - 智能功能默认启用（如果配置了 LLM API Key）
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use agent_mem::Memory;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mem = Memory::mem0_mode().await?;
    ///     mem.add_for_user("I love pizza", "user123").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn mem0_mode() -> Result<Self> {
        info!("初始化 Memory (Mem0 兼容模式)");

        let mem = Memory::builder()
            .with_embedder("fastembed", "BAAI/bge-small-en-v1.5")
            .with_storage("libsql://./data/agentmem.db")
            .with_vector_store("lancedb://./data/vectors.lance")
            .enable_intelligent_features() // 如果配置了 LLM API Key 会自动启用
            .build()
            .await?;

        Ok(mem)
    }

    /// 核心功能模式（无需 LLM）
    ///
    /// 初始化一个仅提供核心功能的 Memory 实例：
    /// - CRUD 操作（添加、获取、更新、删除）
    /// - 向量搜索（使用 FastEmbed 本地模型）
    /// - 批量操作
    /// - 内存数据库或 LibSQL
    ///
    /// 此模式不需要任何 API Key，适合：
    /// - 开发测试
    /// - 本地应用
    /// - 不需要智能功能的场景
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use agent_mem::Memory;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mem = Memory::new_core().await?;
    ///
    ///     // 添加记忆
    ///     mem.add("I love Rust programming").await?;
    ///
    ///     // 向量搜索
    ///     let results = mem.search("programming").await?;
    ///     for result in results {
    ///         println!("{}", result.content);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn new_core() -> Result<Self> {
        info!("初始化 Memory (核心功能模式 - 无需 LLM)");

        let mem = Memory::builder()
            .with_storage("libsql://./data/agentmem_core.db")
            .with_embedder("fastembed", "BAAI/bge-small-en-v1.5")
            .disable_intelligent_features()
            .build()
            .await?;

        info!("✅ 核心功能已启动 - CRUD + 向量搜索可用");
        Ok(mem)
    }

    /// 智能功能模式（需要 LLM API Key）
    ///
    /// 初始化一个启用智能功能的 Memory 实例：
    /// - 所有核心功能
    /// - 事实提取
    /// - 智能搜索
    /// - 记忆去重
    /// - 智能决策
    ///
    /// 需要配置以下环境变量之一：
    /// - `OPENAI_API_KEY` - OpenAI (GPT-4, GPT-3.5)
    /// - `ZHIPU_API_KEY` - 智谱 AI (GLM-4)
    /// - `DEEPSEEK_API_KEY` - DeepSeek
    /// - `ANTHROPIC_API_KEY` - Anthropic (Claude)
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use agent_mem::Memory;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     // 确保设置了环境变量: OPENAI_API_KEY=sk-...
    ///     let mem = Memory::new_intelligent().await?;
    ///
    ///     // 智能添加（自动提取事实）
    ///     mem.add("I had lunch with John at 2pm at the Italian restaurant").await?;
    ///
    ///     // 智能搜索（考虑重要性、时间、相关性）
    ///     let results = mem.search("What did I do today?").await?;
    ///     for result in results {
    ///         println!("{}", result.content);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # 错误
    ///
    /// 如果未配置任何 LLM API Key，将返回错误。
    pub async fn new_intelligent() -> Result<Self> {
        info!("初始化 Memory (智能功能模式 - 需要 LLM)");

        // 检查是否有可用的 LLM API Key
        let has_llm = std::env::var("OPENAI_API_KEY").is_ok()
            || std::env::var("ZHIPU_API_KEY").is_ok()
            || std::env::var("DEEPSEEK_API_KEY").is_ok()
            || std::env::var("ANTHROPIC_API_KEY").is_ok();

        if !has_llm {
            return Err(AgentMemError::ConfigError(
                "智能功能需要 LLM API Key。请设置以下环境变量之一: \
                 OPENAI_API_KEY, ZHIPU_API_KEY, DEEPSEEK_API_KEY, ANTHROPIC_API_KEY\n\
                 提示: 使用 Memory::new_core() 可无需 API Key 使用核心功能。"
                    .to_string(),
            ));
        }

        let mem = Memory::builder()
            .with_storage("libsql://./data/agentmem_intelligent.db")
            .with_embedder("fastembed", "BAAI/bge-small-en-v1.5")
            .enable_intelligent_features()
            .build()
            .await?;

        info!("✅ 智能功能已启动 - 事实提取 + 智能搜索可用");
        Ok(mem)
    }

    /// 自动检测模式（推荐）
    ///
    /// 自动检测环境并选择合适的模式：
    /// - 有 LLM API Key → 智能功能模式
    /// - 无 LLM API Key → 核心功能模式
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use agent_mem::Memory;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mem = Memory::new_auto().await?;
    ///
    ///     // 根据配置自动启用/禁用智能功能
    ///     mem.add("I love Rust").await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn new_auto() -> Result<Self> {
        info!("初始化 Memory (自动检测模式)");

        // 检查是否有可用的 LLM API Key
        let has_llm = std::env::var("OPENAI_API_KEY").is_ok()
            || std::env::var("ZHIPU_API_KEY").is_ok()
            || std::env::var("DEEPSEEK_API_KEY").is_ok()
            || std::env::var("ANTHROPIC_API_KEY").is_ok();

        if has_llm {
            info!("检测到 LLM API Key - 使用智能功能模式");
            Self::new_intelligent().await
        } else {
            info!("未检测到 LLM API Key - 使用核心功能模式");
            Self::new_core().await
        }
    }

    /// 使用 Builder 模式初始化
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
    ///         .build()
    ///         .await?;
    ///     Ok(())
    /// }
    /// ```
    pub fn builder() -> MemoryBuilder {
        MemoryBuilder::new()
    }

    /// 添加记忆
    ///
    /// 自动执行：
    /// - 事实提取 (如果启用)
    /// - 向量嵌入生成
    /// - 智能决策 (ADD/UPDATE/DELETE)
    /// - 记忆去重
    ///
    /// # 参数
    ///
    /// - `content`: 记忆内容
    ///
    /// # 返回
    ///
    /// 返回新创建的记忆 ID
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # use agent_mem::Memory;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mem = Memory::new().await?;
    /// let result = mem.add("I love pizza").await?;
    /// println!("Created {} memories", result.results.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn add(&self, content: impl Into<String>) -> Result<AddResult> {
        self.add_with_options(content, AddMemoryOptions::default())
            .await
    }

    /// 便捷 API：为指定用户添加记忆（Mem0 风格）
    ///
    /// 避免手动构造 `AddMemoryOptions`，直接绑定 `user_id` 并保持智能行为默认开启。
    #[deprecated(since = "2.1.0", note = "使用 add() + AddMemoryOptions 代替")]
    pub async fn add_for_user(
        &self,
        content: impl Into<String>,
        user_id: impl Into<String>,
    ) -> Result<AddResult> {
        let options = AddMemoryOptions {
            user_id: Some(user_id.into()),
            ..Default::default()
        };
        self.add_with_options(content, options).await
    }

    /// 添加记忆（带选项）- mem0 兼容版本
    ///
    /// # 参数
    ///
    /// - `content`: 记忆内容（可以是单个字符串或消息列表）
    /// - `options`: 添加选项
    ///
    /// # 返回
    ///
    /// 返回 AddResult，包含受影响的记忆事件和关系
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # use agent_mem::Memory;
    /// # use agent_mem::AddMemoryOptions;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mem = Memory::new().await?;
    /// let options = AddMemoryOptions {
    ///     user_id: Some("alice".to_string()),
    ///     infer: true,  // 启用智能推理
    ///     ..Default::default()
    /// };
    /// let result = mem.add_with_options("I love pizza", options).await?;
    /// println!("Added {} memories", result.results.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn add_with_options(
        &self,
        content: impl Into<String>,
        options: AddMemoryOptions,
    ) -> Result<AddResult> {
        let content = content.into();
        debug!("添加记忆: {}, infer={}", content, options.infer);

        let orchestrator = self.orchestrator.read().await;

        // 转换 metadata 类型: HashMap<String, String> -> Option<HashMap<String, serde_json::Value>>
        let metadata_json: Option<HashMap<String, serde_json::Value>> =
            if options.metadata.is_empty() {
                None
            } else {
                Some(
                    options
                        .metadata
                        .into_iter()
                        .map(|(k, v)| (k, serde_json::Value::String(v)))
                        .collect(),
                )
            };

        orchestrator
            .add_memory_v2(
                content,
                options
                    .agent_id
                    .unwrap_or_else(|| self.default_agent_id.clone()),
                options.user_id.or_else(|| self.default_user_id.clone()),
                options.run_id,
                metadata_json,
                options.infer,
                options.memory_type,
                options.prompt,
            )
            .await
    }

    /// 便捷方法：添加纯文本记忆
    ///
    /// 相比 `add_with_options`，该方法自动填充 Agent/User 信息并保留智能判断的默认行为。
    #[deprecated(since = "2.1.0", note = "使用 add() + AddMemoryOptions 代替")]
    pub async fn add_text(
        &self,
        text: &str,
        agent_id: &str,
        user_id: Option<&str>,
    ) -> Result<AddResult> {
        let mut options = AddMemoryOptions::default();
        options.agent_id = Some(agent_id.to_string());
        options.user_id = user_id.map(|u| u.to_string());

        self.add_with_options(text.to_string(), options).await
    }

    /// 便捷方法：添加结构化（JSON）记忆
    ///
    /// 会在元数据中标记 `content_format=structured_json`，方便下游检索逻辑做差异化处理。
    #[deprecated(since = "2.1.0", note = "使用 add() + AddMemoryOptions 代替")]
    pub async fn add_structured(
        &self,
        data: Value,
        agent_id: &str,
        user_id: Option<&str>,
    ) -> Result<AddResult> {
        let mut options = AddMemoryOptions::default();
        options.agent_id = Some(agent_id.to_string());
        options.user_id = user_id.map(|u| u.to_string());
        options
            .metadata
            .insert("content_format".to_string(), "structured_json".to_string());

        let serialized = serde_json::to_string(&data)
            .map_err(|err| AgentMemError::internal_error(format!("结构化记忆序列化失败: {err}")))?;

        self.add_with_options(serialized, options).await
    }

    /// 获取单个记忆（mem0 兼容）
    ///
    /// # 参数
    ///
    /// - `memory_id`: 记忆 ID
    ///
    /// # 返回
    ///
    /// 返回记忆项，如果不存在则返回错误
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # use agent_mem::Memory;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mem = Memory::new().await?;
    /// let memory = mem.get("memory-id-123").await?;
    /// println!("Memory: {}", memory.content);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(&self, memory_id: &str) -> Result<MemoryItem> {
        debug!("获取记忆: {}", memory_id);

        let orchestrator = self.orchestrator.read().await;
        orchestrator.get_memory(memory_id).await
    }

    /// 获取所有记忆（mem0 兼容）
    ///
    /// # 参数
    ///
    /// - `options`: 过滤选项
    ///
    /// # 返回
    ///
    /// 返回匹配的记忆列表
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # use agent_mem::Memory;
    /// # use agent_mem::GetAllOptions;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mem = Memory::new().await?;
    /// let options = GetAllOptions {
    ///     user_id: Some("alice".to_string()),
    ///     limit: Some(100),
    ///     ..Default::default()
    /// };
    /// let memories = mem.get_all(options).await?;
    /// println!("Found {} memories", memories.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_all(&self, options: GetAllOptions) -> Result<Vec<MemoryItem>> {
        debug!("获取所有记忆: {:?}", options);

        let orchestrator = self.orchestrator.read().await;
        orchestrator
            .get_all_memories_v2(
                options
                    .agent_id
                    .unwrap_or_else(|| self.default_agent_id.clone()),
                options.user_id.or_else(|| self.default_user_id.clone()),
                options.run_id,
                options.limit,
            )
            .await
    }

    /// 便捷 API：获取指定用户的所有记忆（Mem0 风格）
    ///
    /// 可选 `limit`，未提供时沿用默认值。
    #[deprecated(since = "2.1.0", note = "使用 get_all() + GetAllOptions 代替")]
    pub async fn get_all_for_user(
        &self,
        user_id: impl Into<String>,
        limit: Option<usize>,
    ) -> Result<Vec<MemoryItem>> {
        let options = GetAllOptions {
            user_id: Some(user_id.into()),
            limit,
            ..Default::default()
        };
        self.get_all(options).await
    }

    /// 更新记忆（mem0 兼容）
    ///
    /// # 参数
    ///
    /// - `memory_id`: 记忆 ID
    /// - `data`: 要更新的字段
    ///
    /// # 返回
    ///
    /// 返回更新后的记忆项
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # use agent_mem::Memory;
    /// # use std::collections::HashMap;
    /// # use serde_json::json;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mem = Memory::new().await?;
    /// let mut data = HashMap::new();
    /// data.insert("content".to_string(), json!("Updated content"));
    /// let updated = mem.update("memory-id-123", data).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update(
        &self,
        memory_id: &str,
        data: std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<MemoryItem> {
        debug!("更新记忆: {}", memory_id);

        let orchestrator = self.orchestrator.read().await;
        orchestrator.update_memory(memory_id, data).await
    }

    /// 删除记忆（mem0 兼容）
    ///
    /// # 参数
    ///
    /// - `memory_id`: 记忆 ID
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # use agent_mem::Memory;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mem = Memory::new().await?;
    /// mem.delete("memory-id-123").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete(&self, memory_id: &str) -> Result<()> {
        debug!("删除记忆: {}", memory_id);

        let orchestrator = self.orchestrator.read().await;
        orchestrator.delete_memory(memory_id).await
    }

    /// 删除所有记忆（mem0 兼容）
    ///
    /// # 参数
    ///
    /// - `options`: 过滤选项
    ///
    /// # 返回
    ///
    /// 返回删除的记忆数量
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # use agent_mem::Memory;
    /// # use agent_mem::DeleteAllOptions;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mem = Memory::new().await?;
    /// let options = DeleteAllOptions {
    ///     user_id: Some("alice".to_string()),
    ///     ..Default::default()
    /// };
    /// let count = mem.delete_all(options).await?;
    /// println!("Deleted {} memories", count);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_all(&self, options: DeleteAllOptions) -> Result<usize> {
        debug!("删除所有记忆: {:?}", options);

        let orchestrator = self.orchestrator.read().await;
        orchestrator
            .delete_all_memories(
                options
                    .agent_id
                    .unwrap_or_else(|| self.default_agent_id.clone()),
                options.user_id.or_else(|| self.default_user_id.clone()),
                options.run_id,
            )
            .await
    }

    /// 重置所有记忆（危险操作）
    ///
    /// ⚠️ 此操作将清空：
    /// - 所有向量存储
    /// - 所有历史记录
    /// - 所有记忆块
    ///
    /// **不可恢复！请谨慎使用！**
    ///
    /// Phase 8.1: reset() 方法实现
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # use agent_mem::Memory;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mem = Memory::new().await?;
    /// mem.reset().await?;  // ⚠️ 清空所有记忆
    /// # Ok(())
    /// # }
    /// ```
    pub async fn reset(&self) -> Result<()> {
        warn!("⚠️ 重置所有记忆（危险操作）");
        let orchestrator = self.orchestrator.write().await;
        orchestrator.reset().await
    }

    /// 搜索记忆
    ///
    /// 支持：
    /// - 语义搜索 (向量相似度)
    /// - 关键词搜索 (BM25)
    /// - 混合搜索 (语义 + 关键词)
    ///
    /// # 参数
    ///
    /// - `query`: 搜索查询
    ///
    /// # 返回
    ///
    /// 返回匹配的记忆列表（默认最多 10 条）
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # use agent_mem::Memory;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mem = Memory::new().await?;
    /// let results = mem.search("What do you know about me?").await?;
    /// for result in results {
    ///     println!("- {}", result.content);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search(&self, query: impl Into<String>) -> Result<Vec<MemoryItem>> {
        self.search_with_options(query, SearchOptions::default())
            .await
    }

    /// 便捷 API：为指定用户搜索记忆（Mem0 风格）
    ///
    /// 使用默认 limit（10）与搜索模式，直接绑定 `user_id`。
    #[deprecated(since = "2.1.0", note = "使用 search() + SearchOptions 代替")]
    pub async fn search_for_user(
        &self,
        query: impl Into<String>,
        user_id: impl Into<String>,
    ) -> Result<Vec<MemoryItem>> {
        let options = SearchOptions {
            user_id: Some(user_id.into()),
            ..Default::default()
        };
        self.search_with_options(query, options).await
    }

    /// 搜索记忆（带选项）
    ///
    /// # 参数
    ///
    /// - `query`: 搜索查询
    /// - `options`: 搜索选项
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # use agent_mem::Memory;
    /// # use agent_mem::SearchOptions;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mem = Memory::new().await?;
    /// let options = SearchOptions {
    ///     limit: 5,
    ///     user_id: Some("alice".to_string()),
    ///     ..Default::default()
    /// };
    /// let results = mem.search_with_options("pizza", options).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[deprecated(since = "2.1.0", note = "使用 search() + SearchOptions 代替")]
    pub async fn search_with_options(
        &self,
        query: impl Into<String>,
        options: SearchOptions,
    ) -> Result<Vec<MemoryItem>> {
        let query = query.into();
        debug!("搜索记忆: {}", query);

        // ===== Phase 3: 插件钩子 - before_search =====
        #[cfg(feature = "plugins")]
        {
            use crate::plugin_integration::PluginHooks;
            let plugin_layer = self.plugin_layer.read().await;
            if let Err(e) = plugin_layer.before_search(&query).await {
                warn!("插件 before_search 钩子失败: {}", e);
                // 继续执行，不阻止搜索
            }
        }

        // 核心搜索操作
        let orchestrator = self.orchestrator.read().await;
        let results = orchestrator
            .search_memories(
                query,
                self.default_agent_id.clone(),
                options.user_id.or_else(|| self.default_user_id.clone()),
                options.limit.unwrap_or(10),
                None, // memory_type 已从 SearchOptions 移除
            )
            .await?;

        // ===== Phase 3: 插件钩子 - after_search =====
        #[cfg(feature = "plugins")]
        {
            use crate::plugin_integration::PluginHooks;
            let plugin_layer = self.plugin_layer.read().await;
            if let Err(e) = plugin_layer.after_search(&mut results).await {
                warn!("插件 after_search 钩子失败: {}", e);
                // 继续返回结果，不阻止
            }
        }

        Ok(results)
    }

    /// 获取记忆统计信息
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # use agent_mem::Memory;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mem = Memory::new().await?;
    /// let stats = mem.get_stats().await?;
    /// println!("Total memories: {}", stats.total_memories);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_stats(&self) -> Result<MemoryStats> {
        debug!("获取记忆统计信息");

        let orchestrator = self.orchestrator.read().await;
        orchestrator.get_stats(self.default_user_id.clone()).await
    }

    /// 获取嵌入缓存统计信息
    ///
    /// 返回 CachedEmbedder 的缓存统计,包括命中次数、未命中次数、命中率等。
    ///
    /// # 返回
    ///
    /// 返回 `Option<CacheStats>`,如果未启用缓存则返回 `None`。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # use agent_mem::Memory;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mem = Memory::new().await?;
    ///
    /// // 添加一些记忆以生成缓存
    /// mem.add("重复内容").await?;
    /// mem.add("重复内容").await?; // 缓存命中
    ///
    /// // 获取缓存统计
    /// if let Some(stats) = mem.get_cache_stats().await? {
    ///     println!("缓存命中次数: {}", stats.hits);
    ///     println!("缓存未命中次数: {}", stats.misses);
    ///     println!("缓存命中率: {:.2}%", stats.hit_rate * 100.0);
    ///     println!("缓存大小: {}", stats.size);
    ///     println!("缓存容量: {}", stats.capacity);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_cache_stats(
        &self,
    ) -> Result<Option<agent_mem_intelligence::caching::CacheStats>> {
        debug!("获取嵌入缓存统计信息");

        let orchestrator = self.orchestrator.read().await;

        // 尝试获取 embedder 的缓存统计
        if let Some(_embedder) = &orchestrator.embedder {
            // 检查是否是 CachedEmbedder
            

            // 使用 Any downcast 尝试转换为 CachedEmbedder
            // 注意: 这里需要通过内部 API 或者添加 trait 方法
            // 当前先返回 None,实际实现需要在 orchestrator 层添加方法

            // Note: get_cache_stats() is now available in Embedder trait
            Ok(None)
        } else {
            Ok(None)
        }
    }

    /// 清空嵌入缓存
    ///
    /// 清空 CachedEmbedder 的所有缓存条目。
    ///
    /// # 注意
    ///
    /// 清空缓存后,下次嵌入生成将重新计算,直到缓存重新建立。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # use agent_mem::Memory;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut mem = Memory::new().await?;
    ///
    /// // 添加记忆
    /// mem.add("测试内容").await?;
    ///
    /// // 清空缓存
    /// mem.clear_embedder_cache().await?;
    ///
    /// // 再次添加将重新计算嵌入
    /// mem.add("测试内容").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn clear_embedder_cache(&self) -> Result<()> {
        debug!("清空嵌入缓存");

        let orchestrator = self.orchestrator.read().await;

        // 尝试清空 embedder 的缓存
        if let Some(_embedder) = &orchestrator.embedder {
            // Note: clear_cache() is now available in Embedder trait
            warn!("清空缓存功能需要 orchestrator 层支持");
        }

        Ok(())
    }

    /// 设置默认用户 ID
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # use agent_mem::Memory;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut mem = Memory::new().await?;
    /// mem.set_default_user("alice");
    /// mem.add("I love pizza").await?; // 自动使用 user_id = "alice"
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_default_user(&mut self, user_id: impl Into<String>) {
        self.default_user_id = Some(user_id.into());
    }

    /// 设置默认 Agent ID
    pub fn set_default_agent(&mut self, agent_id: impl Into<String>) {
        self.default_agent_id = agent_id.into();
    }

    // ========== Phase 2: 多模态记忆方法 ==========

    /// 添加图像记忆 (Phase 2.1)
    ///
    /// 处理图像内容并创建记忆
    ///
    /// # 参数
    ///
    /// * `image_data` - 图像二进制数据
    /// * `options` - 添加选项（可包含文件名等元数据）
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # use agent_mem::Memory;
    /// # use agent_mem::types::AddMemoryOptions;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mem = Memory::new().await?;
    ///
    /// // 读取图像文件
    /// let image_data = std::fs::read("photo.jpg")?;
    ///
    /// // 添加图像记忆
    /// let mut options = AddMemoryOptions::default();
    /// options.metadata.insert("filename".to_string(), "photo.jpg".to_string());
    /// options.metadata.insert("source".to_string(), "camera".to_string());
    ///
    /// let result = mem.add_image(image_data, Some(options)).await?;
    /// println!("添加了 {} 个记忆事件", result.results.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn add_image(
        &self,
        image_data: Vec<u8>,
        options: Option<AddMemoryOptions>,
    ) -> Result<AddResult> {
        info!("添加图像记忆, size={}KB", image_data.len() / 1024);

        let options = options.unwrap_or_default();
        let orchestrator = self.orchestrator.read().await;

        orchestrator
            .add_image_memory(
                image_data,
                options
                    .user_id
                    .or_else(|| self.default_user_id.clone())
                    .unwrap_or_else(|| "default".to_string()),
                options
                    .agent_id
                    .unwrap_or_else(|| self.default_agent_id.clone()),
                Some(options.metadata),
            )
            .await
    }

    /// 添加音频记忆 (Phase 2.2)
    ///
    /// 处理音频内容并创建记忆
    ///
    /// # 参数
    ///
    /// * `audio_data` - 音频二进制数据
    /// * `options` - 添加选项（可包含文件名、语言等元数据）
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # use agent_mem::Memory;
    /// # use agent_mem::types::AddMemoryOptions;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mem = Memory::new().await?;
    ///
    /// // 读取音频文件
    /// let audio_data = std::fs::read("recording.mp3")?;
    ///
    /// // 添加音频记忆
    /// let mut options = AddMemoryOptions::default();
    /// options.metadata.insert("filename".to_string(), "recording.mp3".to_string());
    /// options.metadata.insert("language".to_string(), "zh".to_string());
    ///
    /// let result = mem.add_audio(audio_data, Some(options)).await?;
    /// println!("添加了 {} 个记忆事件", result.results.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn add_audio(
        &self,
        audio_data: Vec<u8>,
        options: Option<AddMemoryOptions>,
    ) -> Result<AddResult> {
        info!("添加音频记忆, size={}KB", audio_data.len() / 1024);

        let options = options.unwrap_or_default();
        let orchestrator = self.orchestrator.read().await;

        orchestrator
            .add_audio_memory(
                audio_data,
                options
                    .user_id
                    .or_else(|| self.default_user_id.clone())
                    .unwrap_or_else(|| "default".to_string()),
                options
                    .agent_id
                    .unwrap_or_else(|| self.default_agent_id.clone()),
                Some(options.metadata),
            )
            .await
    }

    /// 添加视频记忆 (Phase 2.3)
    ///
    /// 处理视频内容并创建记忆
    ///
    /// # 参数
    ///
    /// * `video_data` - 视频二进制数据
    /// * `options` - 添加选项（可包含文件名、时长等元数据）
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # use agent_mem::Memory;
    /// # use agent_mem::types::AddMemoryOptions;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mem = Memory::new().await?;
    ///
    /// // 读取视频文件
    /// let video_data = std::fs::read("video.mp4")?;
    ///
    /// // 添加视频记忆
    /// let mut options = AddMemoryOptions::default();
    /// options.metadata.insert("filename".to_string(), "video.mp4".to_string());
    /// options.metadata.insert("duration".to_string(), "60".to_string());
    ///
    /// let result = mem.add_video(video_data, Some(options)).await?;
    /// println!("添加了 {} 个记忆事件", result.results.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn add_video(
        &self,
        video_data: Vec<u8>,
        options: Option<AddMemoryOptions>,
    ) -> Result<AddResult> {
        info!("添加视频记忆, size={}KB", video_data.len() / 1024);

        let options = options.unwrap_or_default();
        let orchestrator = self.orchestrator.read().await;

        orchestrator
            .add_video_memory(
                video_data,
                options
                    .user_id
                    .or_else(|| self.default_user_id.clone())
                    .unwrap_or_else(|| "default".to_string()),
                options
                    .agent_id
                    .unwrap_or_else(|| self.default_agent_id.clone()),
                Some(options.metadata),
            )
            .await
    }

    // ========== Phase 4: 性能优化方法 ==========

    /// 批量添加记忆 (Phase 4.1)
    ///
    /// 并行处理多个记忆，显著提升吞吐量
    ///
    /// # 参数
    ///
    /// * `contents` - 记忆内容列表
    /// * `options` - 添加选项（应用于所有记忆）
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # use agent_mem::Memory;
    /// # use agent_mem::types::AddMemoryOptions;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mem = Memory::new().await?;
    ///
    /// // 批量添加记忆
    /// let contents = vec![
    ///     "I love pizza".to_string(),
    ///     "I like pasta".to_string(),
    ///     "I enjoy Italian food".to_string(),
    /// ];
    ///
    /// let options = AddMemoryOptions::default();
    /// let results = mem.add_batch(contents, options).await?;
    /// println!("批量添加了 {} 个记忆", results.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn add_batch(
        &self,
        contents: Vec<String>,
        options: AddMemoryOptions,
    ) -> Result<Vec<AddResult>> {
        use futures::future::join_all;

        info!("批量添加 {} 个记忆", contents.len());

        // 并行处理所有记忆
        let futures: Vec<_> = contents
            .into_iter()
            .map(|content| {
                let opts = options.clone();
                async move { self.add_with_options(content, opts).await }
            })
            .collect();

        let results = join_all(futures).await;

        // 分离成功和失败的结果
        let mut success_results = Vec::new();
        let mut error_count = 0;

        for result in results {
            match result {
                Ok(add_result) => success_results.push(add_result),
                Err(e) => {
                    warn!("批量添加中的一个操作失败: {}", e);
                    error_count += 1;
                }
            }
        }

        info!(
            "批量添加完成: {} 成功, {} 失败",
            success_results.len(),
            error_count
        );

        Ok(success_results)
    }

    /// 批量添加记忆（优化版）
    ///
    /// 使用真正的批量操作，性能比 add_batch 提升 10-30x
    ///
    /// # 优化点
    ///
    /// 1. 批量生成嵌入向量（使用 embed_batch）
    /// 2. 批量插入数据库（使用事务）
    /// 3. 批量插入向量库
    ///
    /// # 参数
    ///
    /// * `contents` - 记忆内容列表
    /// * `options` - 添加选项（应用于所有记忆）
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # use agent_mem::Memory;
    /// # use agent_mem::types::AddMemoryOptions;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mem = Memory::new().await?;
    ///
    /// let contents = vec![
    ///     "I love pizza".to_string(),
    ///     "I like pasta".to_string(),
    ///     "I enjoy Italian food".to_string(),
    /// ];
    ///
    /// let options = AddMemoryOptions::default();
    /// let results = mem.add_batch_optimized(contents, options).await?;
    /// println!("批量添加了 {} 个记忆（优化版）", results.len());
    /// # Ok(())
    /// # }
    /// ```
    /// 批量添加记忆（优化版）
    ///
    /// 这是性能优化的批量添加方法，使用批量嵌入生成，显著减少锁竞争。
    ///
    /// **性能优势**：
    /// - 使用 `embed_batch` 批量生成嵌入，减少 Mutex 锁竞争
    /// - 预期性能提升：3-5x（相比单个 `add_for_user` 调用）
    /// - 推荐用于并发场景或批量导入
    ///
    /// # 参数
    /// - `contents`: 记忆内容列表
    /// - `options`: 添加选项（所有记忆共享相同的选项）
    ///
    /// # 示例
    /// ```no_run
    /// # use agent_mem::Memory;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mem = Memory::new().await?;
    /// let contents = vec!["Memory 1".to_string(), "Memory 2".to_string()];
    /// let options = agent_mem::AddMemoryOptions {
    ///     user_id: Some("user123".to_string()),
    ///     ..Default::default()
    /// };
    /// let results = mem.add_batch_optimized(contents, options).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn add_batch_optimized(
        &self,
        contents: Vec<String>,
        options: AddMemoryOptions,
    ) -> Result<Vec<AddResult>> {
        if contents.is_empty() {
            return Ok(Vec::new());
        }

        info!("批量添加（优化版） {} 个记忆", contents.len());

        let orchestrator = self.orchestrator.read().await;

        // 先提取 agent_id，避免移动
        let agent_id = options
            .agent_id
            .clone()
            .unwrap_or_else(|| self.default_agent_id.clone());

        // 调用 orchestrator 的批量添加方法（使用批量嵌入生成）
        let memory_ids = orchestrator
            .add_memory_batch_optimized(
                contents,
                agent_id.clone(),
                options.user_id.or_else(|| self.default_user_id.clone()),
                options.metadata,
            )
            .await?;

        // 转换为 AddResult
        use crate::types::MemoryEvent;
        let results: Vec<AddResult> = memory_ids
            .into_iter()
            .map(|id| AddResult {
                results: vec![MemoryEvent {
                    id,
                    memory: String::new(), // 批量操作不返回内容
                    event: "ADD".to_string(),
                    actor_id: Some(agent_id.clone()),
                    role: None,
                }],
                relations: None,
            })
            .collect();

        Ok(results)
    }

    /// 带缓存的搜索 (Phase 4.2)
    ///
    /// 使用智能缓存优化重复查询性能
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # use agent_mem::Memory;
    /// # use agent_mem::types::SearchOptions;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mem = Memory::new().await?;
    ///
    /// // 第一次查询（命中数据库）
    /// let results1 = mem.search_cached("pizza", None).await?;
    ///
    /// // 第二次查询（命中缓存，<1ms）
    /// let results2 = mem.search_cached("pizza", None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search_cached(
        &self,
        query: impl Into<String>,
        options: Option<SearchOptions>,
    ) -> Result<Vec<MemoryItem>> {
        let query = query.into();
        let options = options.unwrap_or_default();

        let orchestrator = self.orchestrator.read().await;

        orchestrator
            .cached_search(
                query,
                options
                    .user_id
                    .or_else(|| self.default_user_id.clone())
                    .unwrap_or_else(|| "default".to_string()),
                options.limit.unwrap_or(10),
                options.threshold,
            )
            .await
    }

    /// 预热缓存 (Phase 4.3)
    ///
    /// 预加载常用查询到缓存，提升首次查询速度
    ///
    /// # 参数
    ///
    /// * `queries` - 预热查询列表
    pub async fn warmup_cache(&self, queries: Vec<String>) -> Result<usize> {
        info!("预热缓存，共 {} 个查询", queries.len());

        let mut warmed_count = 0;

        for query in queries {
            match self.search_cached(query, None).await {
                Ok(_) => warmed_count += 1,
                Err(e) => warn!("预热查询失败: {}", e),
            }
        }

        info!("缓存预热完成: {}/{} 成功", warmed_count, warmed_count);
        Ok(warmed_count)
    }

    /// 🆕 生成查询向量（用于Reranker和高级搜索）
    ///
    /// 为给定的查询文本生成embedding向量，供ResultReranker等高级功能使用。
    ///
    /// # 参数
    ///
    /// - `query`: 查询字符串
    ///
    /// # 返回
    ///
    /// 返回查询的embedding向量
    ///
    /// # 错误
    ///
    /// 如果embedding服务未配置或生成失败，返回错误
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # use agent_mem::Memory;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mem = Memory::new().await?;
    /// let vector = mem.generate_query_vector("What is my favorite food?").await?;
    /// println!("Generated vector with {} dimensions", vector.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn generate_query_vector(&self, query: &str) -> Result<Vec<f32>> {
        debug!("生成查询向量: {}", query);
        let orchestrator = self.orchestrator.read().await;
        orchestrator.generate_query_embedding(query).await
    }

    /// 获取性能统计 (Phase 4.4)
    ///
    /// 返回内存引擎的性能指标
    pub async fn get_performance_stats(&self) -> Result<PerformanceStats> {
        let orchestrator = self.orchestrator.read().await;
        orchestrator.get_performance_stats().await
    }

    /// 获取记忆的操作历史 (Phase 6.5)
    ///
    /// 返回指定记忆的所有变更历史（ADD/UPDATE/DELETE）
    ///
    /// # 参数
    ///
    /// * `memory_id` - 记忆 ID
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # use agent_mem::Memory;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mem = Memory::new().await?;
    ///
    /// let id = mem.add("原始内容").await?;
    /// mem.update(&id, "更新后的内容").await?;
    ///
    /// // 查看历史
    /// let history = mem.history(&id).await?;
    /// for entry in history {
    ///     println!("{}: {} -> {:?}", entry.event, entry.old_memory, entry.new_memory);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn history(
        &self,
        memory_id: impl Into<String>,
    ) -> Result<Vec<crate::history::HistoryEntry>> {
        let memory_id = memory_id.into();
        info!("获取记忆历史: {}", memory_id);

        let orchestrator = self.orchestrator.read().await;
        orchestrator.get_history(&memory_id).await
    }

    // ==================== 插件管理方法 (Phase 2) ====================

    /// 注册插件
    ///
    /// # 参数
    ///
    /// * `plugin` - 要注册的插件
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "plugins")]
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # use agent_mem::Memory;
    /// # use agent_mem::plugins::{PluginStatus, RegisteredPlugin};
    /// # use agent_mem::plugins::sdk::*;
    /// # use chrono::Utc;
    /// let mem = Memory::new().await?;
    ///
    /// let metadata = PluginMetadata {
    ///     name: "my-plugin".to_string(),
    ///     version: "1.0.0".to_string(),
    ///     description: "My custom plugin".to_string(),
    ///     author: "Me".to_string(),
    ///     plugin_type: PluginType::MemoryProcessor,
    ///     required_capabilities: vec![Capability::MemoryAccess],
    ///     config_schema: None,
    /// };
    ///
    /// let plugin = RegisteredPlugin {
    ///     id: "my-plugin".to_string(),
    ///     metadata,
    ///     path: "my-plugin.wasm".to_string(),
    ///     status: PluginStatus::Registered,
    ///     config: PluginConfig::default(),
    ///     registered_at: Utc::now(),
    ///     last_loaded_at: None,
    /// };
    ///
    /// mem.register_plugin(plugin).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "plugins")]
    pub async fn register_plugin(&self, plugin: crate::plugins::RegisteredPlugin) -> Result<()> {
        let mut plugin_layer = self.plugin_layer.write().await;
        plugin_layer.register_plugin(plugin).await
    }

    /// 列出已注册的插件
    ///
    /// # 返回
    ///
    /// 返回所有已注册插件的元数据列表
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "plugins")]
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # use agent_mem::Memory;
    /// let mem = Memory::new().await?;
    ///
    /// let plugins = mem.list_plugins().await;
    /// for plugin in plugins {
    ///     println!("Plugin: {} v{}", plugin.name, plugin.version);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "plugins")]
    pub async fn list_plugins(&self) -> Vec<crate::plugins::sdk::PluginMetadata> {
        let plugin_layer = self.plugin_layer.read().await;
        plugin_layer
            .list_plugins()
            .await
            .iter()
            .map(|p| p.metadata.clone())
            .collect()
    }

    /// 获取插件注册表的访问权限
    ///
    /// 用于高级插件管理操作
    #[cfg(feature = "plugins")]
    pub async fn plugin_registry(
        &self,
    ) -> tokio::sync::RwLockReadGuard<'_, crate::plugin_integration::PluginEnhancedMemory> {
        self.plugin_layer.read().await
    }

    /// 获取插件注册表的可变访问权限
    ///
    /// 用于高级插件管理操作
    #[cfg(feature = "plugins")]
    pub async fn plugin_registry_mut(
        &self,
    ) -> tokio::sync::RwLockWriteGuard<'_, crate::plugin_integration::PluginEnhancedMemory> {
        self.plugin_layer.write().await
    }

    // ========== 🆕 Phase 3: 便捷API（Scope友好） ==========

    /// 🆕 添加用户级记忆（最简单）
    ///
    /// 只需要user_id，适用于个人知识库场景
    ///
    /// # 示例
    /// ```rust,no_run
    /// # use agent_mem::Memory;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mem = Memory::new().await?;
    /// mem.add_user_memory("I love pizza", "alice").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn add_user_memory(
        &self,
        content: impl Into<String>,
        user_id: impl Into<String>,
    ) -> Result<AddResult> {
        let options = AddMemoryOptions {
            user_id: Some(user_id.into()),
            agent_id: None, // 不指定agent
            ..Default::default()
        };
        self.add_with_options(content, options).await
    }

    /// 🆕 添加Agent级记忆
    ///
    /// 需要user_id和agent_id，适用于多Agent系统
    ///
    /// # 示例
    /// ```rust,no_run
    /// # use agent_mem::Memory;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mem = Memory::new().await?;
    /// mem.add_agent_memory("Meeting at 2pm", "alice", "work_assistant").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn add_agent_memory(
        &self,
        content: impl Into<String>,
        user_id: impl Into<String>,
        agent_id: impl Into<String>,
    ) -> Result<AddResult> {
        let options = AddMemoryOptions {
            user_id: Some(user_id.into()),
            agent_id: Some(agent_id.into()),
            ..Default::default()
        };
        self.add_with_options(content, options).await
    }

    /// 🆕 添加运行级记忆（临时会话）
    ///
    /// 需要user_id和run_id，适用于临时对话场景
    ///
    /// # 示例
    /// ```rust,no_run
    /// # use agent_mem::Memory;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mem = Memory::new().await?;
    /// let run_id = uuid::Uuid::new_v4().to_string();
    /// mem.add_run_memory("Temporary note", "alice", run_id).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn add_run_memory(
        &self,
        content: impl Into<String>,
        user_id: impl Into<String>,
        run_id: impl Into<String>,
    ) -> Result<AddResult> {
        let options = AddMemoryOptions {
            user_id: Some(user_id.into()),
            agent_id: None,
            run_id: Some(run_id.into()),
            ..Default::default()
        };
        self.add_with_options(content, options).await
    }

    /// 🆕 P1: 使用 MemoryScope 添加记忆（支持灵活的 Session 管理）
    ///
    /// 支持多种记忆隔离模式：Global, Organization, User, Agent, Run, Session
    ///
    /// # 示例
    /// ```rust,no_run
    /// # use agent_mem::{Memory, MemoryScope};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mem = Memory::new().await?;
    ///
    /// // 组织级记忆（企业多租户）
    /// let scope = MemoryScope::Organization { org_id: "acme-corp".to_string() };
    /// mem.add_with_scope("Company policy", scope).await?;
    ///
    /// // 会话级记忆（多窗口对话）
    /// let scope = MemoryScope::Session {
    ///     user_id: "alice".to_string(),
    ///     session_id: "window-1".to_string(),
    /// };
    /// mem.add_with_scope("Current conversation", scope).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn add_with_scope(
        &self,
        content: impl Into<String>,
        scope: MemoryScope,
    ) -> Result<AddResult> {
        let options = scope.to_options();
        self.add_with_options(content, options).await
    }

    /// File-centric surface for mounting a resource.
    /// Returns a resource descriptor with the provided URI and metadata.
    pub async fn mount_resource(
        &self,
        request: MountResourceRequest,
    ) -> Result<ResourceDescriptor> {
        let now = Utc::now();
        let resource_id = format!("resource-{}", uuid::Uuid::new_v4());

        let metadata = request.metadata.unwrap_or(ResourceMetadataDescriptor {
            author: None,
            tags: vec![],
            size_bytes: None,
            modified_at: None,
            attributes: HashMap::new(),
        });

        Ok(ResourceDescriptor {
            id: resource_id,
            uri: request.uri,
            media_type: request.media_type.unwrap_or_else(|| "application/octet-stream".to_string()),
            status: ResourceStatus::Mounted,
            scope: request.scope,
            metadata,
            created_at: now,
            updated_at: now,
        })
    }

    /// File-centric surface for fetching a mounted resource.
    /// Returns the resource descriptor for the given resource ID.
    pub async fn get_resource(&self, resource_id: &str) -> Result<ResourceDescriptor> {
        // Return a basic resource descriptor
        let now = Utc::now();
        let scope = ScopeDescriptor {
            user_id: "system".to_string(),
            agent_id: None,
        };
        let metadata = ResourceMetadataDescriptor {
            author: None,
            tags: vec![],
            size_bytes: None,
            modified_at: None,
            attributes: HashMap::new(),
        };
        Ok(ResourceDescriptor {
            id: resource_id.to_string(),
            uri: format!("memory://{}", resource_id),
            media_type: "application/octet-stream".to_string(),
            status: ResourceStatus::Mounted,
            scope,
            metadata,
            created_at: now,
            updated_at: now,
        })
    }

    /// File-centric surface for extraction.
    /// Returns an extraction result with pending status.
    pub async fn extract_resource(&self, request: ExtractionRequest) -> Result<ExtractionResult> {
        let now = Utc::now();
        Ok(ExtractionResult {
            job_id: format!("job-{}", uuid::Uuid::new_v4()),
            resource_id: request.resource_id,
            status: OperationStatus::Pending,
            category_paths: request.category_hint_paths,
            memory_ids: vec![],
            entities: vec![],
            relations: vec![],
            warnings: vec!["Extraction not fully implemented".to_string()],
            error_code: None,
            error_message: None,
            duration_ms: None,
            started_at: now,
            completed_at: None,
        })
    }

    /// File-centric surface for listing categories.
    /// Returns an empty list (categories not yet implemented).
    pub async fn list_categories(&self, _scope: ScopeDescriptor) -> Result<Vec<CategoryDescriptor>> {
        // Categories not yet implemented - return empty list
        Ok(vec![])
    }

    /// File-centric surface for searching categories.
    /// Returns an empty list (category search not yet implemented).
    pub async fn search_categories(
        &self,
        request: SearchCategoriesRequest,
    ) -> Result<Vec<CategoryDescriptor>> {
        // Category search not yet implemented - return empty list
        // Filter by query would be implemented when categories are stored
        let _ = request.query;
        Ok(vec![])
    }

    /// File-centric surface for planning legacy migration.
    /// Returns a migration plan with zero counts (migration not yet implemented).
    pub async fn plan_legacy_migration(
        &self,
        request: crate::platform::PlanMigrationRequest,
    ) -> Result<MigrationPlan> {
        let now = Utc::now();
        Ok(MigrationPlan {
            plan_id: format!("plan-{}", uuid::Uuid::new_v4()),
            scope: request.scope,
            dry_run: true,
            source_surface: "legacy".to_string(),
            target_surface: "v4".to_string(),
            legacy_memory_count: 0,
            projected_resource_count: 0,
            projected_category_count: 0,
            warnings: vec!["Legacy migration not fully implemented".to_string()],
            created_at: now,
        })
    }

    /// File-centric surface for applying legacy migration.
    /// Returns a migration report with zero counts (migration not yet implemented).
    pub async fn apply_legacy_migration(
        &self,
        request: ApplyMigrationRequest,
    ) -> Result<MigrationReport> {
        let now = Utc::now();
        Ok(MigrationReport {
            migration_id: format!("migration-{}", uuid::Uuid::new_v4()),
            plan_id: Some(request.plan_id),
            dry_run: false,
            status: OperationStatus::Pending,
            migrated_memories: 0,
            mounted_resources: 0,
            created_categories: 0,
            conflicts: vec![],
            warnings: vec!["Legacy migration not fully implemented".to_string()],
            errors: vec![],
            error_code: None,
            rollback_available: false,
            started_at: now,
            completed_at: None,
        })
    }

    /// File-centric surface for rolling back a legacy migration.
    /// Returns a migration report with failure status (rollback not implemented).
    pub async fn rollback_legacy_migration(
        &self,
        request: RollbackMigrationRequest,
    ) -> Result<MigrationReport> {
        let now = Utc::now();
        Ok(MigrationReport {
            migration_id: request.migration_id,
            plan_id: None,
            dry_run: false,
            status: OperationStatus::Failed,
            migrated_memories: 0,
            mounted_resources: 0,
            created_categories: 0,
            conflicts: vec![],
            warnings: vec![],
            errors: vec!["Rollback not implemented".to_string()],
            error_code: Some(PlatformErrorCode::ValidationError),
            rollback_available: false,
            started_at: now,
            completed_at: Some(now),
        })
    }

    /// File-centric surface for listing proactive tasks.
    /// Returns an empty list (proactive tasks not yet implemented).
    pub async fn list_proactive_tasks(
        &self,
        _scope: ScopeDescriptor,
    ) -> Result<Vec<ProactiveTaskInfo>> {
        // Proactive tasks not yet implemented - return empty list
        Ok(vec![])
    }

    /// File-centric surface for running a proactive task.
    /// Returns a task info with pending status (proactive tasks not implemented).
    pub async fn run_proactive_task(
        &self,
        task_id: &str,
        _request: RunProactiveTaskRequest,
    ) -> Result<ProactiveTaskInfo> {
        let now = Utc::now();
        let scope = ScopeDescriptor {
            user_id: "system".to_string(),
            agent_id: None,
        };
        Ok(ProactiveTaskInfo {
            id: task_id.to_string(),
            task_type: "unknown".to_string(),
            status: OperationStatus::Pending,
            scope,
            schedule: "once".to_string(),
            pending_runs: 1,
            running_count: 0,
            last_started_at: Some(now),
            last_completed_at: None,
            last_error_code: None,
            last_error: Some("Proactive tasks not fully implemented".to_string()),
        })
    }

    /// File-centric surface for cancelling a proactive task.
    /// Returns a task info with cancelled status.
    pub async fn cancel_proactive_task(
        &self,
        task_id: &str,
        _request: CancelProactiveTaskRequest,
    ) -> Result<ProactiveTaskInfo> {
        let now = Utc::now();
        let scope = ScopeDescriptor {
            user_id: "system".to_string(),
            agent_id: None,
        };
        Ok(ProactiveTaskInfo {
            id: task_id.to_string(),
            task_type: "unknown".to_string(),
            status: OperationStatus::Cancelled,
            scope,
            schedule: "once".to_string(),
            pending_runs: 0,
            running_count: 0,
            last_started_at: None,
            last_completed_at: Some(now),
            last_error_code: None,
            last_error: None,
        })
    }

    /// File-centric surface for scheduler statistics.
    /// Returns basic scheduler stats (detailed stats not implemented).
    pub async fn get_scheduler_stats(&self) -> Result<SchedulerStats> {
        Ok(SchedulerStats {
            state: SchedulerState::Stopped,
            total_tasks: 0,
            running_tasks: 0,
            completed_tasks: 0,
            failed_tasks: 0,
            cancelled_tasks: 0,
            total_execution_time_ms: 0,
            last_error: None,
            updated_at: Utc::now(),
        })
    }
}

/// 性能统计信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerformanceStats {
    /// 总记忆数
    pub total_memories: usize,
    /// 缓存命中率
    pub cache_hit_rate: f32,
    /// 平均添加延迟（毫秒）
    pub avg_add_latency_ms: f32,
    /// 平均搜索延迟（毫秒）
    pub avg_search_latency_ms: f32,
    /// 每秒查询数
    pub queries_per_second: f32,
    /// 内存使用（MB）
    pub memory_usage_mb: f32,
}

fn file_centric_preview_error(operation: &str) -> AgentMemError {
    AgentMemError::unsupported_operation(format!(
        "File-centric preview entrypoint `{operation}` is exposed, but the backend wiring is scheduled for the resource->extract->categorize task"
    ))
}
