//! Memory - 统一的记忆管理接口
//!
//! Memory 提供了简洁的 API 来管理所有类型的记忆，
//! 内部自动路由到对应的专门 Agent 处理。

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
        self.add_with_options(content, AddMemoryOptions::default()).await
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
        orchestrator.add_memory_v2(
            content,
            options.agent_id.unwrap_or_else(|| self.default_agent_id.clone()),
            options.user_id.or_else(|| self.default_user_id.clone()),
            options.run_id,
            options.metadata,
            options.infer,
            options.memory_type,
            options.prompt,
        ).await
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
        orchestrator.get_all_memories_v2(
            options.agent_id.unwrap_or_else(|| self.default_agent_id.clone()),
            options.user_id.or_else(|| self.default_user_id.clone()),
            options.run_id,
            options.limit,
        ).await
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
        orchestrator.delete_all_memories(
            options.agent_id.unwrap_or_else(|| self.default_agent_id.clone()),
            options.user_id.or_else(|| self.default_user_id.clone()),
            options.run_id,
        ).await
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
        self.search_with_options(query, SearchOptions::default()).await
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
        orchestrator.search_memories(
            query,
            self.default_agent_id.clone(),
            options.user_id.or_else(|| self.default_user_id.clone()),
            options.limit.unwrap_or(10),
            None,  // memory_type 已从 SearchOptions 移除
        ).await
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
        orchestrator.get_stats(
            self.default_user_id.clone(),
        ).await
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
}

