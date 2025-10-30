//! LRU 缓存实现
//!
//! 为智能功能提供 LRU (Least Recently Used) 缓存，减少 LLM 调用

use agent_mem_traits::{CacheStats, ExtractedFact, IntelligenceCache, MemoryDecision};
use async_trait::async_trait;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

/// LRU 缓存实现
pub struct LRUIntelligenceCache {
    /// 事实提取缓存
    facts_cache: Arc<RwLock<LruCache<String, Vec<ExtractedFact>>>>,
    
    /// 决策缓存
    decision_cache: Arc<RwLock<LruCache<String, MemoryDecision>>>,
    
    /// 缓存命中次数
    hits: Arc<AtomicU64>,
    
    /// 缓存未命中次数
    misses: Arc<AtomicU64>,
}

impl LRUIntelligenceCache {
    /// 创建新的 LRU 缓存
    ///
    /// # Arguments
    /// * `max_size` - 每个缓存的最大条目数
    pub fn new(max_size: usize) -> Self {
        let size = NonZeroUsize::new(max_size).unwrap_or(NonZeroUsize::new(1000).unwrap());
        
        Self {
            facts_cache: Arc::new(RwLock::new(LruCache::new(size))),
            decision_cache: Arc::new(RwLock::new(LruCache::new(size))),
            hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
        }
    }
    
    /// 创建默认缓存 (1000 条目)
    pub fn default() -> Self {
        Self::new(1000)
    }
}

#[async_trait]
impl IntelligenceCache for LRUIntelligenceCache {
    async fn get_facts(&self, key: &str) -> Option<Vec<ExtractedFact>> {
        let mut cache = self.facts_cache.write().await;
        
        if let Some(facts) = cache.get(key) {
            self.hits.fetch_add(1, Ordering::Relaxed);
            Some(facts.clone())
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
            None
        }
    }
    
    async fn set_facts(&self, key: &str, facts: Vec<ExtractedFact>) {
        let mut cache = self.facts_cache.write().await;
        cache.put(key.to_string(), facts);
    }
    
    async fn get_decision(&self, key: &str) -> Option<MemoryDecision> {
        let mut cache = self.decision_cache.write().await;
        
        if let Some(decision) = cache.get(key) {
            self.hits.fetch_add(1, Ordering::Relaxed);
            Some(decision.clone())
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
            None
        }
    }
    
    async fn set_decision(&self, key: &str, decision: MemoryDecision) {
        let mut cache = self.decision_cache.write().await;
        cache.put(key.to_string(), decision);
    }
    
    async fn clear(&self) {
        let mut facts_cache = self.facts_cache.write().await;
        let mut decision_cache = self.decision_cache.write().await;
        
        facts_cache.clear();
        decision_cache.clear();
        
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
    }
    
    async fn stats(&self) -> CacheStats {
        let facts_cache = self.facts_cache.read().await;
        let decision_cache = self.decision_cache.read().await;
        
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let size = facts_cache.len() + decision_cache.len();
        
        let total = hits + misses;
        let hit_rate = if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        };
        
        CacheStats {
            hits,
            misses,
            size,
            hit_rate,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    #[tokio::test]
    async fn test_facts_cache() {
        let cache = LRUIntelligenceCache::new(10);
        
        // 测试缓存未命中
        assert!(cache.get_facts("key1").await.is_none());
        
        // 设置缓存
        let facts = vec![ExtractedFact {
            content: "test fact".to_string(),
            confidence: 0.9,
            category: "test".to_string(),
            metadata: HashMap::new(),
        }];
        cache.set_facts("key1", facts.clone()).await;
        
        // 测试缓存命中
        let cached = cache.get_facts("key1").await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().len(), 1);
        
        // 检查统计
        let stats = cache.stats().await;
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert!(stats.hit_rate > 0.0);
    }
    
    #[tokio::test]
    async fn test_decision_cache() {
        let cache = LRUIntelligenceCache::new(10);
        
        // 测试缓存未命中
        assert!(cache.get_decision("key1").await.is_none());
        
        // 设置缓存
        let decision = MemoryDecision {
            action: agent_mem_traits::MemoryActionType::Add {
                content: "test".to_string(),
                importance: 0.8,
                metadata: HashMap::new(),
            },
            confidence: 0.9,
            reasoning: "test reasoning".to_string(),
        };
        cache.set_decision("key1", decision.clone()).await;
        
        // 测试缓存命中
        let cached = cache.get_decision("key1").await;
        assert!(cached.is_some());
        
        // 检查统计
        let stats = cache.stats().await;
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }
    
    #[tokio::test]
    async fn test_cache_clear() {
        let cache = LRUIntelligenceCache::new(10);
        
        // 添加一些数据
        let facts = vec![ExtractedFact {
            content: "test".to_string(),
            confidence: 0.9,
            category: "test".to_string(),
            metadata: HashMap::new(),
        }];
        cache.set_facts("key1", facts).await;
        
        // 清空缓存
        cache.clear().await;
        
        // 验证缓存已清空
        assert!(cache.get_facts("key1").await.is_none());
        
        let stats = cache.stats().await;
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.size, 0);
    }
    
    #[tokio::test]
    async fn test_lru_eviction() {
        let cache = LRUIntelligenceCache::new(2); // 只能存 2 个条目
        
        // 添加 3 个条目
        let fact1 = vec![ExtractedFact {
            content: "fact1".to_string(),
            confidence: 0.9,
            category: "test".to_string(),
            metadata: HashMap::new(),
        }];
        let fact2 = vec![ExtractedFact {
            content: "fact2".to_string(),
            confidence: 0.9,
            category: "test".to_string(),
            metadata: HashMap::new(),
        }];
        let fact3 = vec![ExtractedFact {
            content: "fact3".to_string(),
            confidence: 0.9,
            category: "test".to_string(),
            metadata: HashMap::new(),
        }];
        
        cache.set_facts("key1", fact1).await;
        cache.set_facts("key2", fact2).await;
        cache.set_facts("key3", fact3).await; // 应该驱逐 key1
        
        // key1 应该被驱逐
        assert!(cache.get_facts("key1").await.is_none());
        
        // key2 和 key3 应该还在
        assert!(cache.get_facts("key2").await.is_some());
        assert!(cache.get_facts("key3").await.is_some());
    }
}

