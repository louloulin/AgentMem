//! LLM 结果缓存模块
//!
//! 用于缓存 LLM 调用结果，减少重复调用，提升性能

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// 缓存的结果
#[derive(Clone, Debug)]
pub struct CachedResult<T> {
    /// 缓存的值
    pub value: T,
    /// 创建时间
    pub created_at: Instant,
    /// 过期时间（TTL）
    pub ttl: Duration,
}

impl<T> CachedResult<T> {
    /// 创建新的缓存结果
    pub fn new(value: T, ttl: Duration) -> Self {
        Self {
            value,
            created_at: Instant::now(),
            ttl,
        }
    }

    /// 检查是否过期
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

/// LLM 结果缓存
pub struct LLMCache<T> {
    /// 缓存存储
    cache: Arc<RwLock<HashMap<String, CachedResult<T>>>>,
    /// 默认 TTL
    default_ttl: Duration,
    /// 最大缓存条目数
    max_entries: usize,
}

impl<T: Clone> LLMCache<T> {
    /// 创建新的缓存
    pub fn new(default_ttl: Duration, max_entries: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            default_ttl,
            max_entries,
        }
    }

    /// 生成缓存键
    pub fn generate_key(content: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// 获取缓存值
    pub async fn get(&self, key: &str) -> Option<T> {
        let cache = self.cache.read().await;
        if let Some(cached) = cache.get(key) {
            if !cached.is_expired() {
                return Some(cached.value.clone());
            }
        }
        None
    }

    /// 设置缓存值
    pub async fn set(&self, key: String, value: T) {
        self.set_with_ttl(key, value, self.default_ttl).await;
    }

    /// 设置缓存值（自定义 TTL）
    pub async fn set_with_ttl(&self, key: String, value: T, ttl: Duration) {
        let mut cache = self.cache.write().await;

        // 如果缓存已满，清理过期条目
        if cache.len() >= self.max_entries {
            self.cleanup_expired(&mut cache);

            // 如果清理后仍然满，删除最旧的条目
            if cache.len() >= self.max_entries {
                if let Some(oldest_key) = cache
                    .iter()
                    .min_by_key(|(_, v)| v.created_at)
                    .map(|(k, _)| k.clone())
                {
                    cache.remove(&oldest_key);
                }
            }
        }

        cache.insert(key, CachedResult::new(value, ttl));
    }

    /// 清理过期条目
    fn cleanup_expired(&self, cache: &mut HashMap<String, CachedResult<T>>) {
        cache.retain(|_, v| !v.is_expired());
    }

    /// 获取或计算值
    pub async fn get_or_compute<F, Fut>(&self, key: &str, compute: F) -> Result<T, String>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, String>>,
    {
        // 检查缓存
        if let Some(cached) = self.get(key).await {
            return Ok(cached);
        }

        // 计算并缓存
        let value = compute().await?;
        self.set(key.to_string(), value.clone()).await;
        Ok(value)
    }

    /// 清空缓存
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// 获取缓存统计信息
    pub async fn stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        let total = cache.len();
        let expired = cache.values().filter(|v| v.is_expired()).count();

        CacheStats {
            total_entries: total,
            expired_entries: expired,
            active_entries: total - expired,
        }
    }
}

/// 缓存统计信息
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// 总条目数
    pub total_entries: usize,
    /// 过期条目数
    pub expired_entries: usize,
    /// 活跃条目数
    pub active_entries: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_cache_basic() {
        let cache = LLMCache::<String>::new(Duration::from_secs(60), 100);

        // 设置值
        cache.set("key1".to_string(), "value1".to_string()).await;

        // 获取值
        let value = cache.get("key1").await;
        assert_eq!(value, Some("value1".to_string()));

        // 获取不存在的值
        let value = cache.get("key2").await;
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let cache = LLMCache::<String>::new(Duration::from_millis(100), 100);

        // 设置值
        cache.set("key1".to_string(), "value1".to_string()).await;

        // 立即获取，应该存在
        let value = cache.get("key1").await;
        assert_eq!(value, Some("value1".to_string()));

        // 等待过期
        sleep(Duration::from_millis(150)).await;

        // 再次获取，应该已过期
        let value = cache.get("key1").await;
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_cache_max_entries() {
        let cache = LLMCache::<String>::new(Duration::from_secs(60), 3);

        // 添加 4 个条目
        cache.set("key1".to_string(), "value1".to_string()).await;
        cache.set("key2".to_string(), "value2".to_string()).await;
        cache.set("key3".to_string(), "value3".to_string()).await;
        cache.set("key4".to_string(), "value4".to_string()).await;

        // 检查统计信息
        let stats = cache.stats().await;
        assert_eq!(stats.total_entries, 3); // 最多 3 个条目
    }

    #[tokio::test]
    async fn test_get_or_compute() {
        let cache = LLMCache::<String>::new(Duration::from_secs(60), 100);

        let mut call_count = 0;

        // 第一次调用，应该计算
        let value = cache
            .get_or_compute("key1", || async {
                call_count += 1;
                Ok::<String, String>("computed_value".to_string())
            })
            .await
            .unwrap();
        assert_eq!(value, "computed_value");
        assert_eq!(call_count, 1);

        // 第二次调用，应该从缓存获取
        let value = cache
            .get_or_compute("key1", || async {
                call_count += 1;
                Ok::<String, String>("computed_value".to_string())
            })
            .await
            .unwrap();
        assert_eq!(value, "computed_value");
        assert_eq!(call_count, 1); // 没有再次计算
    }

    #[test]
    fn test_generate_key() {
        let key1 = LLMCache::<String>::generate_key("test content");
        let key2 = LLMCache::<String>::generate_key("test content");
        let key3 = LLMCache::<String>::generate_key("different content");

        // 相同内容应该生成相同的键
        assert_eq!(key1, key2);

        // 不同内容应该生成不同的键
        assert_ne!(key1, key3);
    }
}
