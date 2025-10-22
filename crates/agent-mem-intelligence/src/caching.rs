//! LRU 缓存模块
//!
//! P1 优化 #1, #20: 为事实提取和向量嵌入添加缓存层
//!
//! 功能：
//! - 基于内容Hash的LRU缓存
//! - 可配置的缓存大小和TTL
//! - 线程安全的缓存访问
//! - 缓存命中率统计

use lru::LruCache;
use sha2::{Digest, Sha256};
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, info};

/// 缓存配置
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// 缓存大小
    pub size: usize,
    /// TTL（秒）
    pub ttl_secs: u64,
    /// 是否启用缓存
    pub enabled: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            size: 1000,
            ttl_secs: 3600, // 1小时
            enabled: true,
        }
    }
}

/// 缓存条目
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    value: T,
    cached_at: Instant,
    ttl: Duration,
}

impl<T> CacheEntry<T> {
    fn new(value: T, ttl: Duration) -> Self {
        Self {
            value,
            cached_at: Instant::now(),
            ttl,
        }
    }

    fn is_expired(&self) -> bool {
        self.cached_at.elapsed() > self.ttl
    }
}

/// LRU缓存
pub struct LruCacheWrapper<T: Clone> {
    cache: Arc<Mutex<LruCache<String, CacheEntry<T>>>>,
    config: CacheConfig,
    hits: Arc<Mutex<u64>>,
    misses: Arc<Mutex<u64>>,
}

impl<T: Clone> LruCacheWrapper<T> {
    /// 创建新的缓存
    pub fn new(config: CacheConfig) -> Self {
        let size = NonZeroUsize::new(config.size.max(1)).unwrap();
        Self {
            cache: Arc::new(Mutex::new(LruCache::new(size))),
            config,
            hits: Arc::new(Mutex::new(0)),
            misses: Arc::new(Mutex::new(0)),
        }
    }

    /// 计算缓存key
    pub fn compute_key(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 获取缓存值
    pub fn get(&self, key: &str) -> Option<T> {
        if !self.config.enabled {
            return None;
        }

        let mut cache = self.cache.lock().unwrap();
        
        if let Some(entry) = cache.get(key) {
            if entry.is_expired() {
                debug!("缓存过期: {}", &key[..16]);
                cache.pop(key);
                *self.misses.lock().unwrap() += 1;
                None
            } else {
                debug!("缓存命中: {}", &key[..16]);
                *self.hits.lock().unwrap() += 1;
                Some(entry.value.clone())
            }
        } else {
            debug!("缓存未命中: {}", &key[..16]);
            *self.misses.lock().unwrap() += 1;
            None
        }
    }

    /// 设置缓存值
    pub fn put(&self, key: String, value: T) {
        if !self.config.enabled {
            return;
        }

        let entry = CacheEntry::new(
            value,
            Duration::from_secs(self.config.ttl_secs),
        );

        let mut cache = self.cache.lock().unwrap();
        cache.put(key, entry);
        debug!("已缓存数据");
    }

    /// 清空缓存
    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
        *self.hits.lock().unwrap() = 0;
        *self.misses.lock().unwrap() = 0;
        info!("缓存已清空");
    }

    /// 获取缓存统计
    pub fn stats(&self) -> CacheStats {
        let hits = *self.hits.lock().unwrap();
        let misses = *self.misses.lock().unwrap();
        let total = hits + misses;
        let hit_rate = if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        };

        CacheStats {
            hits,
            misses,
            hit_rate,
            size: self.cache.lock().unwrap().len(),
            capacity: self.config.size,
        }
    }
}

/// 缓存统计
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
    pub size: usize,
    pub capacity: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_basic() {
        let config = CacheConfig {
            size: 10,
            ttl_secs: 60,
            enabled: true,
        };
        let cache: LruCacheWrapper<String> = LruCacheWrapper::new(config);

        let key = LruCacheWrapper::<String>::compute_key("test content");
        
        // 第一次获取应该未命中
        assert!(cache.get(&key).is_none());

        // 设置值
        cache.put(key.clone(), "test value".to_string());

        // 第二次获取应该命中
        assert_eq!(cache.get(&key), Some("test value".to_string()));

        // 检查统计
        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate, 0.5);
    }

    #[test]
    fn test_cache_expiration() {
        let config = CacheConfig {
            size: 10,
            ttl_secs: 1, // 1秒过期
            enabled: true,
        };
        let cache: LruCacheWrapper<String> = LruCacheWrapper::new(config);

        let key = LruCacheWrapper::<String>::compute_key("test content");
        cache.put(key.clone(), "test value".to_string());

        // 立即获取应该成功
        assert!(cache.get(&key).is_some());

        // 等待过期
        std::thread::sleep(Duration::from_secs(2));

        // 过期后应该未命中
        assert!(cache.get(&key).is_none());
    }

    #[test]
    fn test_cache_lru_eviction() {
        let config = CacheConfig {
            size: 2, // 只能存2个
            ttl_secs: 60,
            enabled: true,
        };
        let cache: LruCacheWrapper<String> = LruCacheWrapper::new(config);

        let key1 = "key1".to_string();
        let key2 = "key2".to_string();
        let key3 = "key3".to_string();

        cache.put(key1.clone(), "value1".to_string());
        cache.put(key2.clone(), "value2".to_string());
        cache.put(key3.clone(), "value3".to_string());

        // key1应该被淘汰
        assert!(cache.get(&key1).is_none());
        assert!(cache.get(&key2).is_some());
        assert!(cache.get(&key3).is_some());
    }

    #[test]
    fn test_cache_disabled() {
        let config = CacheConfig {
            size: 10,
            ttl_secs: 60,
            enabled: false, // 禁用缓存
        };
        let cache: LruCacheWrapper<String> = LruCacheWrapper::new(config);

        let key = LruCacheWrapper::<String>::compute_key("test content");
        cache.put(key.clone(), "test value".to_string());

        // 禁用时应该总是未命中
        assert!(cache.get(&key).is_none());
    }

    #[test]
    fn test_compute_key_deterministic() {
        let content = "test content";
        let key1 = LruCacheWrapper::<String>::compute_key(content);
        let key2 = LruCacheWrapper::<String>::compute_key(content);

        // 相同内容应该产生相同的key
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_compute_key_different() {
        let key1 = LruCacheWrapper::<String>::compute_key("content1");
        let key2 = LruCacheWrapper::<String>::compute_key("content2");

        // 不同内容应该产生不同的key
        assert_ne!(key1, key2);
    }
}

