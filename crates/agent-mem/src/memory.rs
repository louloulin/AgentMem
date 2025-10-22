//! Memory - 统一的记忆管理接口
//!
//! Memory 提供了简洁的 API 来管理所有类型的记忆，
//! 内部自动路由到对应的专门 Agent 处理。

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use agent_mem_traits::{AgentMemError, MemoryItem, MemoryType, Result};

use crate::builder::MemoryBuilder;
use crate::orchestrator::MemoryOrchestrator;
use crate::types::{
    AddMemoryOptions, AddResult, DeleteAllOptions, GetAllOptions, MemoryEvent, MemoryStats,
    RelationEvent, SearchOptions,
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
pub struct Memory {
    /// 内部编排器，负责协调各个 Agent
    orchestrator: Arc<RwLock<MemoryOrchestrator>>,
    /// 默认用户 ID
    default_user_id: Option<String>,
    /// 默认 Agent ID
    default_agent_id: String,
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
    pub async fn search_with_options(
        &self,
        query: impl Into<String>,
        options: SearchOptions,
    ) -> Result<Vec<MemoryItem>> {
        let query = query.into();
        debug!("搜索记忆: {}", query);

        let orchestrator = self.orchestrator.read().await;
        orchestrator
            .search_memories(
                query,
                self.default_agent_id.clone(),
                options.user_id.or_else(|| self.default_user_id.clone()),
                options.limit.unwrap_or(10),
                None, // memory_type 已从 SearchOptions 移除
            )
            .await
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
