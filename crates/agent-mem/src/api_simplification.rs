//! API Simplification Module
//!
//! Phase 3.1: API简化
//! - 零配置启动增强
//! - 智能默认值
//! - 链式调用支持
//! - 错误处理改进
//!
//! 参考Mem0的API设计，提升易用性

use crate::memory::Memory;
use crate::types::{AddMemoryOptions, AddResult, SearchOptions};
use agent_mem_traits::{AgentMemError, Result};
use std::fmt;
use tracing::{debug, info};

/// 链式调用的Memory包装器
///
/// Phase 3.1: 支持链式调用，让API更简洁
///
/// # 示例
///
/// ```rust,no_run
/// use agent_mem::Memory;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mem = Memory::new().await?;
///     
///     // 链式调用
///     let results = mem
///         .add("I love pizza")
///         .await?
///         .search("What do I like?")
///         .await?;
///     
///     Ok(())
/// }
/// ```
pub struct FluentMemory {
    inner: Memory,
}

impl FluentMemory {
    /// 从Memory创建FluentMemory
    pub fn new(memory: Memory) -> Self {
        Self { inner: memory }
    }

    /// 添加记忆（链式调用）
    ///
    /// 返回Result，成功时返回Self以支持链式调用
    pub async fn add(self, content: impl Into<String>) -> Result<Self> {
        self.inner.add(content).await?;
        Ok(self)
    }

    /// 添加记忆（带选项，链式调用）
    pub async fn add_with_options(
        self,
        content: impl Into<String>,
        options: AddMemoryOptions,
    ) -> Result<Self> {
        self.inner.add_with_options(content, options).await?;
        Ok(self)
    }

    /// 搜索记忆（链式调用）
    ///
    /// 注意：搜索会返回结果，所以这个方法主要用于执行搜索操作
    pub async fn search(self, query: impl Into<String>) -> Result<Self> {
        let _ = self.inner.search(query).await?;
        Ok(self)
    }

    /// 获取内部Memory实例
    pub fn into_inner(self) -> Memory {
        self.inner
    }

    /// 获取内部Memory的引用
    pub fn inner(&self) -> &Memory {
        &self.inner
    }
}

impl From<Memory> for FluentMemory {
    fn from(memory: Memory) -> Self {
        Self::new(memory)
    }
}

/// 增强的错误类型
///
/// Phase 3.1: 提供更友好的错误信息和恢复建议
#[derive(Debug)]
pub struct EnhancedError {
    /// 原始错误消息
    pub error_message: String,
    /// 用户友好的错误消息
    pub user_message: String,
    /// 恢复建议
    pub suggestions: Vec<String>,
    /// 错误上下文
    pub context: Option<String>,
}

impl fmt::Display for EnhancedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.user_message)?;
        if !self.suggestions.is_empty() {
            write!(f, "\n\n建议:")?;
            for (i, suggestion) in self.suggestions.iter().enumerate() {
                write!(f, "\n  {}. {}", i + 1, suggestion)?;
            }
        }
        if let Some(context) = &self.context {
            write!(f, "\n\n上下文: {}", context)?;
        }
        Ok(())
    }
}

impl std::error::Error for EnhancedError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

/// 错误增强器
///
/// Phase 3.1: 将原始错误转换为用户友好的错误
pub struct ErrorEnhancer;

impl ErrorEnhancer {
    /// 增强错误信息
    pub fn enhance(error: AgentMemError, context: Option<String>) -> EnhancedError {
        let error_message = error.to_string();
        let (user_message, suggestions) = Self::analyze_error(&error);

        EnhancedError {
            error_message,
            user_message,
            suggestions,
            context,
        }
    }

    /// 分析错误并提供建议
    fn analyze_error(error: &AgentMemError) -> (String, Vec<String>) {
        match error {
            AgentMemError::StorageError(msg) => {
                let user_msg = format!("存储错误: {}", msg);
                let suggestions = vec![
                    "检查数据库连接是否正常".to_string(),
                    "确认存储路径有写入权限".to_string(),
                    "尝试使用内存存储进行测试: Memory::builder().with_storage(\"memory://\")".to_string(),
                ];
                (user_msg, suggestions)
            }
            AgentMemError::NotFound(msg) => {
                let user_msg = format!("未找到: {}", msg);
                let suggestions = vec![
                    "检查ID是否正确".to_string(),
                    "确认记忆是否已创建".to_string(),
                    "尝试使用search()方法搜索相关记忆".to_string(),
                ];
                (user_msg, suggestions)
            }
            AgentMemError::InvalidInput(msg) => {
                let user_msg = format!("输入无效: {}", msg);
                let suggestions = vec![
                    "检查输入参数格式是否正确".to_string(),
                    "确认必需参数是否已提供".to_string(),
                    "查看API文档了解正确的参数格式".to_string(),
                ];
                (user_msg, suggestions)
            }
            AgentMemError::LLMError(msg) => {
                let user_msg = format!("LLM错误: {}", msg);
                let suggestions = vec![
                    "检查API Key是否正确配置".to_string(),
                    "确认网络连接是否正常".to_string(),
                    "尝试使用本地模型: Memory::builder().with_embedder(\"fastembed\", \"BAAI/bge-small-en-v1.5\")".to_string(),
                ];
                (user_msg, suggestions)
            }
            _ => {
                let user_msg = format!("错误: {}", error);
                let suggestions = vec!["查看详细错误日志".to_string()];
                (user_msg, suggestions)
            }
        }
    }
}

/// 智能默认值配置器
///
/// Phase 3.1: 根据环境自动选择最佳默认值
pub struct SmartDefaults;

impl SmartDefaults {
    /// 检测环境并返回智能默认配置
    pub async fn detect() -> SmartDefaultConfig {
        let mut config = SmartDefaultConfig::default();

        // 检测API Key
        config.has_openai_key = std::env::var("OPENAI_API_KEY").is_ok();
        config.has_deepseek_key = std::env::var("DEEPSEEK_API_KEY").is_ok();
        config.has_anthropic_key = std::env::var("ANTHROPIC_API_KEY").is_ok();

        // 检测存储后端
        config.has_postgres = std::env::var("DATABASE_URL").is_ok();
        config.has_redis = std::env::var("REDIS_URL").is_ok();

        // 检测运行环境
        config.is_production = std::env::var("ENV")
            .map(|e| e == "production")
            .unwrap_or(false);

        // 根据检测结果设置推荐配置
        config.recommended_llm = if config.has_openai_key {
            Some(("openai", "gpt-4"))
        } else if config.has_deepseek_key {
            Some(("deepseek", "deepseek-chat"))
        } else if config.has_anthropic_key {
            Some(("anthropic", "claude-3-opus"))
        } else {
            None
        };

        config.recommended_storage = if config.has_postgres {
            Some("postgres")
        } else {
            Some("libsql")
        };

        config.recommended_embedder = if config.has_openai_key {
            Some(("openai", "text-embedding-3-small"))
        } else {
            Some(("fastembed", "BAAI/bge-small-en-v1.5"))
        };

        info!("智能默认值检测完成: {:?}", config);
        config
    }

    /// 应用智能默认值到MemoryBuilder
    pub async fn apply_to_builder(builder: crate::builder::MemoryBuilder) -> Result<crate::builder::MemoryBuilder> {
        let defaults = Self::detect().await;
        let mut builder = builder;

        // 应用LLM配置
        if let Some((provider, model)) = defaults.recommended_llm {
            builder = builder.with_llm(provider, model);
        }

        // 应用存储配置
        if let Some(storage) = defaults.recommended_storage {
            if storage == "postgres" {
                if let Ok(url) = std::env::var("DATABASE_URL") {
                    builder = builder.with_storage(&url);
                }
            } else {
                builder = builder.with_storage("libsql://./data/agentmem.db");
            }
        }

        // 应用嵌入器配置
        if let Some((provider, model)) = defaults.recommended_embedder {
            builder = builder.with_embedder(provider, model);
        }

        // 根据环境启用/禁用智能功能
        if defaults.has_openai_key || defaults.has_deepseek_key || defaults.has_anthropic_key {
            builder = builder.enable_intelligent_features();
        }

        Ok(builder)
    }
}

/// 智能默认配置
#[derive(Debug, Clone)]
pub struct SmartDefaultConfig {
    /// 是否有OpenAI API Key
    pub has_openai_key: bool,
    /// 是否有DeepSeek API Key
    pub has_deepseek_key: bool,
    /// 是否有Anthropic API Key
    pub has_anthropic_key: bool,
    /// 是否有PostgreSQL
    pub has_postgres: bool,
    /// 是否有Redis
    pub has_redis: bool,
    /// 是否为生产环境
    pub is_production: bool,
    /// 推荐的LLM (provider, model)
    pub recommended_llm: Option<(&'static str, &'static str)>,
    /// 推荐的存储后端
    pub recommended_storage: Option<&'static str>,
    /// 推荐的嵌入器 (provider, model)
    pub recommended_embedder: Option<(&'static str, &'static str)>,
}

impl Default for SmartDefaultConfig {
    fn default() -> Self {
        Self {
            has_openai_key: false,
            has_deepseek_key: false,
            has_anthropic_key: false,
            has_postgres: false,
            has_redis: false,
            is_production: false,
            recommended_llm: None,
            recommended_storage: Some("libsql"),
            recommended_embedder: Some(("fastembed", "BAAI/bge-small-en-v1.5")),
        }
    }
}

/// 零配置启动增强
///
/// Phase 3.1: 增强零配置启动，提供更智能的默认值
impl Memory {
    /// 零配置启动（增强版）
    ///
    /// 自动检测环境并应用智能默认值：
    /// - 检测API Key，自动选择LLM
    /// - 检测存储后端，自动选择存储
    /// - 根据环境启用/禁用智能功能
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use agent_mem::Memory;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     // 自动检测环境并配置
    ///     let mem = Memory::new_smart().await?;
    ///     mem.add("I love pizza").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn new_smart() -> Result<Self> {
        info!("初始化 Memory (智能零配置模式)");

        let builder = crate::builder::MemoryBuilder::new();
        let builder = SmartDefaults::apply_to_builder(builder).await?;
        builder.build().await
    }

    /// 创建支持链式调用的Memory
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use agent_mem::Memory;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let fluent = Memory::new().await?.fluent();
    ///     fluent
    ///         .add("I love pizza")
    ///         .await?
    ///         .add("I like coffee")
    ///         .await?;
    ///     Ok(())
    /// }
    /// ```
    pub fn fluent(self) -> FluentMemory {
        FluentMemory::new(self)
    }

    /// 添加记忆（返回增强错误）
    ///
    /// Phase 3.1: 提供更友好的错误信息
    pub async fn add_with_enhanced_error(
        &self,
        content: impl Into<String>,
    ) -> std::result::Result<AddResult, EnhancedError> {
        self.add(content)
            .await
            .map_err(|e| ErrorEnhancer::enhance(e, Some("添加记忆时".to_string())))
    }

    /// 搜索记忆（返回增强错误）
    ///
    /// Phase 3.1: 提供更友好的错误信息
    pub async fn search_with_enhanced_error(
        &self,
        query: impl Into<String>,
    ) -> std::result::Result<Vec<agent_mem_traits::MemoryItem>, EnhancedError> {
        self.search(query)
            .await
            .map_err(|e| ErrorEnhancer::enhance(e, Some("搜索记忆时".to_string())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_smart_defaults_detection() {
        let config = SmartDefaults::detect().await;
        assert!(config.recommended_storage.is_some());
        assert!(config.recommended_embedder.is_some());
    }

    #[tokio::test]
    async fn test_fluent_memory() {
        // 注意：这个测试需要实际的Memory实例，可能需要mock
        // 这里只是测试类型系统
        let _fluent: FluentMemory = todo!("需要实际的Memory实例");
    }

    #[test]
    fn test_error_enhancement() {
        let error = AgentMemError::StorageError("Database connection failed".to_string());
        let enhanced = ErrorEnhancer::enhance(error, None);
        assert!(!enhanced.user_message.is_empty());
        assert!(!enhanced.suggestions.is_empty());
    }
}
