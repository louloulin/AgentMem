//! ✅ P1: LLM Connection Pool Manager
//!
//! 轻量级连接池实现，用于复用 LLM provider 实例
//!
//! **设计原则**:
//! - 最佳最小方式：简单的 Arc 包装，无需复杂依赖
//! - 高内聚：所有池逻辑集中在此模块
//! - 低耦合：不依赖外部连接池库
//!
//! **性能提升**:
//! - 减少 provider 创建开销
//! - 支持并发 LLM 调用
//! - 自动连接复用

use agent_mem_traits::{LLMConfig, LLMProvider, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// ✅ P1: LLM 连接池管理器
///
/// 轻量级连接池，用于复用 LLM provider 实例
/// 避免为每个请求创建新的 provider，减少初始化开销
///
/// # 线程安全
///
/// 内部使用 `RwLock` 保护连接映射，支持并发读写
///
/// # 示例
///
/// ```no_run
/// # use agent_mem_llm::pool::LLMPoolManager;
/// # use agent_mem_traits::LLMConfig;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let pool = LLMPoolManager::new();
/// let config = LLMConfig::default();
///
/// // 从池中获取或创建 provider
/// let provider = pool.get_or_create_provider(&config).unwrap();
///
/// // 使用 provider...
/// # Ok(())
/// # }
/// ```
pub struct LLMPoolManager {
    /// 连接池：配置 -> Provider
    /// 使用 RwLock 支持并发访问
    pool: RwLock<HashMap<String, Arc<dyn LLMProvider + Send + Sync>>>,
}

impl LLMPoolManager {
    /// 创建新的连接池管理器
    pub fn new() -> Self {
        Self {
            pool: RwLock::new(HashMap::new()),
        }
    }

    /// ✅ P1: 从池中获取或创建 provider
    ///
    /// 如果配置对应的 provider 已存在，则返回缓存的实例
    /// 否则创建新的 provider 并缓存
    ///
    /// # 参数
    /// - `config`: LLM 配置
    ///
    /// # 返回
    /// - 缓存的或新创建的 provider 实例
    ///
    /// # 线程安全
    ///
    /// 此方法使用 `RwLock` 确保线程安全
    pub async fn get_or_create_provider(
        &self,
        config: &LLMConfig,
    ) -> Result<Arc<dyn LLMProvider + Send + Sync>> {
        // 生成配置的唯一键（基于 provider 和 model）
        let pool_key = Self::generate_pool_key(config);

        // 先尝试读锁（快速路径：已缓存）
        {
            let pool_read = self.pool.read().await;
            if let Some(provider) = pool_read.get(&pool_key) {
                tracing::debug!("✅ P1 LLM Pool: 复用缓存的 provider: {}", pool_key);
                return Ok(provider.clone());
            }
        }

        // 未命中缓存，创建新 provider
        tracing::debug!("🔧 P1 LLM Pool: 创建新 provider: {}", pool_key);

        // 使用 crate::LLMFactory::create_provider 创建
        let provider = crate::LLMFactory::create_provider(config)?;

        // 写入缓存
        let mut pool_write = self.pool.write().await;
        pool_write.insert(pool_key.clone(), provider.clone());

        tracing::debug!("✅ P1 LLM Pool: 已缓存 provider: {}", pool_key);

        Ok(provider)
    }

    /// ✅ P1: 清理缓存的 provider
    ///
    /// 移除指定配置的 provider 缓存
    ///
    /// # 用途
    /// - 配置更新后清理旧缓存
    /// - 释放资源
    ///
    /// # 参数
    /// - `config`: 要清理的 LLM 配置
    pub async fn clear_provider(&self, config: &LLMConfig) {
        let pool_key = Self::generate_pool_key(config);
        let mut pool_write = self.pool.write().await;
        pool_write.remove(&pool_key);
        tracing::debug!("🗑️  P1 LLM Pool: 已清理 provider: {}", pool_key);
    }

    /// ✅ P1: 清空所有缓存的 providers
    ///
    /// 移除所有缓存的 provider 实例
    ///
    /// # 用途
    /// - 应用关闭时清理
    /// - 配置重置
    pub async fn clear_all(&self) {
        let mut pool_write = self.pool.write().await;
        let count = pool_write.len();
        pool_write.clear();
        tracing::debug!("🗑️  P1 LLM Pool: 已清理所有 providers (共 {} 个)", count);
    }

    /// ✅ P1: 获取池统计信息
    ///
    /// 返回当前缓存的 provider 数量
    ///
    /// # 返回
    /// - 缓存的 provider 数量
    pub async fn pool_size(&self) -> usize {
        let pool_read = self.pool.read().await;
        pool_read.len()
    }

    /// ✅ P1: Helper: 生成配置的唯一键
    ///
    /// 基于 provider 和 model 生成唯一的池键
    ///
    /// # 格式
    ///
    /// ```text
    /// "{provider}/{model}"
    /// ```
    ///
    /// # 示例
    ///
    /// - `"openai/gpt-4"`
    /// - `"anthropic/claude-3-opus-20240229"`
    fn generate_pool_key(config: &LLMConfig) -> String {
        format!("{}/{}", config.provider, config.model)
    }
}

impl Default for LLMPoolManager {
    fn default() -> Self {
        Self::new()
    }
}

// ✅ P1: 单元测试

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pool_manager_creation() {
        let pool = LLMPoolManager::new();
        assert_eq!(pool.pool_size().await, 0);
    }

    #[tokio::test]
    async fn test_pool_key_generation() {
        let mut config = LLMConfig::default();
        config.provider = "openai".into();
        config.model = "gpt-4".into();

        let key = LLMPoolManager::generate_pool_key(&config);
        assert_eq!(key, "openai/gpt-4");
    }

    #[tokio::test]
    async fn test_pool_size_tracking() {
        let pool = LLMPoolManager::new();

        // 初始大小为 0
        assert_eq!(pool.pool_size().await, 0);

        // 清空所有（即使是空的）
        pool.clear_all().await;
        assert_eq!(pool.pool_size().await, 0);
    }

    #[tokio::test]
    async fn test_clear_provider() {
        let pool = LLMPoolManager::new();

        let mut config = LLMConfig::default();
        config.provider = "test".into();
        config.model = "test-model".into();

        // 清理不存在的 provider 不会 panic
        pool.clear_provider(&config).await;

        // 大小仍然为 0
        assert_eq!(pool.pool_size().await, 0);
    }
}
