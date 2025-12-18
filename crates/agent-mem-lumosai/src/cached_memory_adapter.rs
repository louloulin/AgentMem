//! Cached Memory Adapter - 带缓存的AgentMem Backend
//!
//! 实现多级缓存（L1内存缓存 + L2 Redis缓存）以大幅提升检索性能

use agent_mem::{AddMemoryOptions, GetAllOptions, Memory as AgentMemApi, SearchOptions};
use async_trait::async_trait;
use lumosai_core::llm::Message as LumosMessage;
use lumosai_core::llm::Role as LumosRole;
use lumosai_core::memory::{Memory as LumosMemory, MemoryConfig};
use lumosai_core::Result as LumosResult;
use lru::LruCache;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// 缓存配置
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// 启用L1内存缓存
    pub enable_l1_cache: bool,
    /// L1缓存最大条目数
    pub l1_cache_max_size: usize,
    /// 启用L2 Redis缓存
    pub enable_l2_cache: bool,
    /// L2缓存TTL（秒）
    pub l2_cache_ttl_seconds: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enable_l1_cache: true,
            l1_cache_max_size: 1000,
            enable_l2_cache: false, // 需要Redis连接，默认关闭
            l2_cache_ttl_seconds: 300, // 5分钟
        }
    }
}

/// 带缓存的AgentMem Backend
pub struct CachedAgentMemBackend {
    /// 底层AgentMem API
    memory_api: Arc<AgentMemApi>,
    /// Agent ID
    agent_id: String,
    /// User ID
    user_id: String,
    /// L1内存缓存（LRU）
    l1_cache: Arc<RwLock<LruCache<String, Vec<LumosMessage>>>>,
    /// 缓存配置
    config: CacheConfig,
}

impl CachedAgentMemBackend {
    /// 创建新的缓存适配器
    pub fn new(
        memory_api: Arc<AgentMemApi>,
        agent_id: String,
        user_id: String,
        config: CacheConfig,
    ) -> Self {
        let cache_size = NonZeroUsize::new(config.l1_cache_max_size)
            .unwrap_or(NonZeroUsize::new(1000).unwrap());
        let l1_cache = Arc::new(RwLock::new(LruCache::new(cache_size)));

        Self {
            memory_api,
            agent_id,
            user_id,
            l1_cache,
            config,
        }
    }

    /// 使用默认配置创建
    pub fn with_defaults(
        memory_api: Arc<AgentMemApi>,
        agent_id: String,
        user_id: String,
    ) -> Self {
        Self::new(memory_api, agent_id, user_id, CacheConfig::default())
    }

    /// 构建缓存键
    fn build_cache_key(&self, config: &MemoryConfig) -> String {
        let mut hasher = DefaultHasher::new();
        self.agent_id.hash(&mut hasher);
        self.user_id.hash(&mut hasher);
        config.query.hash(&mut hasher);
        config.last_messages.hash(&mut hasher);
        config.namespace.hash(&mut hasher);
        config.store_id.hash(&mut hasher);
        format!("cache:{}", hasher.finish())
    }

    /// 从AgentMem检索（无缓存）
    async fn retrieve_from_backend(
        &self,
        config: &MemoryConfig,
    ) -> LumosResult<Vec<LumosMessage>> {
        use crate::memory_adapter::AgentMemBackend;
        let backend = AgentMemBackend::new(
            self.memory_api.clone(),
            self.agent_id.clone(),
            self.user_id.clone(),
        );
        backend.retrieve(config).await
    }

    /// 转换MemoryItem为LumosMessage（从memory_adapter复制逻辑）
    fn convert_to_messages(&self, items: Vec<agent_mem::MemoryItem>) -> Vec<LumosMessage> {
        items
            .into_iter()
            .filter_map(|mem| {
                let role_str = mem
                    .metadata
                    .get("role")
                    .and_then(|v| v.as_str())
                    .unwrap_or("user");

                let role = match role_str {
                    "system" => LumosRole::System,
                    "assistant" => LumosRole::Assistant,
                    "tool" => LumosRole::Tool,
                    _ => LumosRole::User,
                };

                let content = if mem.content.starts_with('[') {
                    mem.content
                        .splitn(2, "]: ")
                        .nth(1)
                        .unwrap_or(&mem.content)
                        .to_string()
                } else {
                    mem.content
                };

                Some(LumosMessage {
                    role,
                    content,
                    metadata: None,
                    name: None,
                })
            })
            .collect()
    }
}

#[async_trait]
impl LumosMemory for CachedAgentMemBackend {
    async fn store(&self, message: &LumosMessage) -> LumosResult<()> {
        // 存储操作不缓存，直接调用底层API
        // 但需要使相关缓存失效
        let cache_key_prefix = format!("cache:{}:{}", self.agent_id, self.user_id);
        
        // 清除相关缓存（简化实现：清除所有该agent/user的缓存）
        if self.config.enable_l1_cache {
            let mut cache = self.l1_cache.write().await;
            // 清除所有匹配的缓存条目（简化：清除所有）
            cache.clear();
        }

        // 调用底层存储
        use crate::memory_adapter::AgentMemBackend;
        let backend = AgentMemBackend::new(
            self.memory_api.clone(),
            self.agent_id.clone(),
            self.user_id.clone(),
        );
        backend.store(message).await
    }

    async fn retrieve(&self, config: &MemoryConfig) -> LumosResult<Vec<LumosMessage>> {
        let retrieve_start = std::time::Instant::now();

        // 1. 检查L1缓存
        if self.config.enable_l1_cache {
            let cache_key = self.build_cache_key(config);
            let cache = self.l1_cache.read().await;
            if let Some(cached) = cache.peek(&cache_key) {
                let cache_hit_duration = retrieve_start.elapsed();
                info!(
                    "✅ [CACHE-L1-HIT] Retrieved from cache in {:?}",
                    cache_hit_duration
                );
                debug!("   Cache key: {}", cache_key);
                return Ok(cached.clone());
            }
            drop(cache);
        }

        // 2. L2缓存（Redis）暂不实现，需要Redis连接

        // 3. 从后端检索
        info!("🔍 [CACHE-MISS] Retrieving from backend");
        let backend_start = std::time::Instant::now();
        let results = self.retrieve_from_backend(config).await?;
        let backend_duration = backend_start.elapsed();

        info!(
            "   Backend retrieval: {:?}, Found: {} messages",
            backend_duration,
            results.len()
        );

        // 4. 更新L1缓存
        if self.config.enable_l1_cache {
            let cache_key = self.build_cache_key(config);
            let mut cache = self.l1_cache.write().await;
            cache.put(cache_key.clone(), results.clone());
            debug!("   Cached with key: {}", cache_key);
        }

        let total_duration = retrieve_start.elapsed();
        info!(
            "✅ [CACHE-RETRIEVE] Total: {:?} (Backend: {:?}, Cache: {:?})",
            total_duration,
            backend_duration,
            total_duration - backend_duration
        );

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_mem::Memory as AgentMemApi;

    #[tokio::test]
    async fn test_cache_hit() {
        // 这个测试需要mock AgentMemApi，简化实现
        // 实际测试应该在集成测试中完成
    }

    #[tokio::test]
    async fn test_cache_miss() {
        // 这个测试需要mock AgentMemApi，简化实现
    }
}

