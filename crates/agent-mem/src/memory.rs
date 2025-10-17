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
    AddMemoryOptions, MemoryStats, SearchOptions,
    // TODO: 在任务 2.1 和 2.2 中使用这些类型
    // ChatOptions, MemoryVisualization,
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
    /// let id = mem.add("I love pizza").await?;
    /// println!("Created memory: {}", id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn add(&self, content: impl Into<String>) -> Result<String> {
        self.add_with_options(content, AddMemoryOptions::default()).await
    }
    
    /// 添加记忆（带选项）
    ///
    /// # 参数
    ///
    /// - `content`: 记忆内容
    /// - `options`: 添加选项
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # use agent_mem::Memory;
    /// # use agent_mem::AddMemoryOptions;
    /// # use agent_mem::MemoryType;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mem = Memory::new().await?;
    /// let options = AddMemoryOptions {
    ///     memory_type: Some(MemoryType::Core),
    ///     user_id: Some("alice".to_string()),
    ///     ..Default::default()
    /// };
    /// let id = mem.add_with_options("I love pizza", options).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn add_with_options(
        &self,
        content: impl Into<String>,
        options: AddMemoryOptions,
    ) -> Result<String> {
        let content = content.into();
        debug!("添加记忆: {}", content);
        
        let orchestrator = self.orchestrator.read().await;
        orchestrator.add_memory(
            content,
            self.default_agent_id.clone(),
            options.user_id.or_else(|| self.default_user_id.clone()),
            options.memory_type,
            options.metadata,
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
            options.limit,
            options.memory_type,
        ).await
    }
    
    /// 获取所有记忆
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # use agent_mem::Memory;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mem = Memory::new().await?;
    /// let all_memories = mem.get_all().await?;
    /// println!("Total memories: {}", all_memories.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_all(&self) -> Result<Vec<MemoryItem>> {
        debug!("获取所有记忆");
        
        let orchestrator = self.orchestrator.read().await;
        orchestrator.get_all_memories(
            self.default_agent_id.clone(),
            self.default_user_id.clone(),
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

