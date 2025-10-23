//! 带缓存的 Embedder
//!
//! P1 优化 #20: 为向量嵌入添加缓存层

use agent_mem_intelligence::caching::{CacheConfig, LruCacheWrapper};
use agent_mem_traits::{Embedder, Result};
use std::sync::Arc;
use tracing::{debug, info};

/// 带缓存的 Embedder
pub struct CachedEmbedder {
    inner: Arc<dyn Embedder + Send + Sync>,
    cache: Arc<LruCacheWrapper<Vec<f32>>>,
}

impl CachedEmbedder {
    /// 创建新的带缓存的 Embedder
    pub fn new(embedder: Arc<dyn Embedder + Send + Sync>, cache_config: CacheConfig) -> Self {
        info!(
            "创建带缓存的 Embedder，缓存大小: {}, TTL: {}秒",
            cache_config.size, cache_config.ttl_secs
        );

        Self {
            inner: embedder,
            cache: Arc::new(LruCacheWrapper::new(cache_config)),
        }
    }

    /// 获取缓存统计
    pub fn cache_stats(&self) -> agent_mem_intelligence::caching::CacheStats {
        self.cache.stats()
    }

    /// 清空缓存
    pub fn clear_cache(&self) {
        self.cache.clear();
    }
}

#[async_trait::async_trait]
impl Embedder for CachedEmbedder {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // P1 优化 #20: 检查缓存
        let cache_key = LruCacheWrapper::<Vec<f32>>::compute_key(text);
        
        if let Some(cached_embedding) = self.cache.get(&cache_key) {
            debug!("✅ 嵌入向量缓存命中: {}", &cache_key[..16]);
            return Ok(cached_embedding);
        }

        debug!("缓存未命中，生成新的嵌入向量: {}", &cache_key[..16]);
        
        // 调用实际的 embedder
        let embedding = self.inner.embed(text).await?;

        // P1 优化 #20: 写入缓存
        self.cache.put(cache_key.clone(), embedding.clone());
        debug!("✅ 嵌入向量已缓存: {}", &cache_key[..16]);

        Ok(embedding)
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::new();
        let mut uncached_indices = Vec::new();
        let mut uncached_texts = Vec::new();

        // 检查哪些文本已缓存
        for (idx, text) in texts.iter().enumerate() {
            let cache_key = LruCacheWrapper::<Vec<f32>>::compute_key(text);
            
            if let Some(cached_embedding) = self.cache.get(&cache_key) {
                results.push((idx, cached_embedding));
            } else {
                uncached_indices.push(idx);
                uncached_texts.push(text.clone());
            }
        }

        // 批量生成未缓存的嵌入
        if !uncached_texts.is_empty() {
            debug!("批量生成 {} 个未缓存的嵌入向量", uncached_texts.len());
            let new_embeddings = self.inner.embed_batch(&uncached_texts).await?;

            // 缓存新生成的嵌入
            for (text, embedding) in uncached_texts.iter().zip(new_embeddings.iter()) {
                let cache_key = LruCacheWrapper::<Vec<f32>>::compute_key(text);
                self.cache.put(cache_key, embedding.clone());
            }

            // 添加到结果中
            for (idx, embedding) in uncached_indices.iter().zip(new_embeddings.into_iter()) {
                results.push((*idx, embedding));
            }
        }

        // 按原始顺序排序
        results.sort_by_key(|(idx, _)| *idx);
        
        Ok(results.into_iter().map(|(_, emb)| emb).collect())
    }

    fn dimension(&self) -> usize {
        self.inner.dimension()
    }

    fn provider_name(&self) -> &str {
        self.inner.provider_name()
    }

    fn model_name(&self) -> &str {
        self.inner.model_name()
    }

    async fn health_check(&self) -> Result<bool> {
        self.inner.health_check().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 简单的 Mock Embedder 用于测试
    struct MockEmbedder {
        dimension: usize,
    }

    impl MockEmbedder {
        fn new(dimension: usize) -> Self {
            Self { dimension }
        }
    }

    #[async_trait::async_trait]
    impl Embedder for MockEmbedder {
        async fn embed(&self, text: &str) -> Result<Vec<f32>> {
            // 返回基于文本哈希的确定性向量
            let hash = text.len() as f32;
            Ok(vec![hash / self.dimension as f32; self.dimension])
        }

        async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
            let mut embeddings = Vec::with_capacity(texts.len());
            for text in texts {
                embeddings.push(self.embed(text).await?);
            }
            Ok(embeddings)
        }

        fn dimension(&self) -> usize {
            self.dimension
        }

        fn provider_name(&self) -> &str {
            "mock"
        }

        fn model_name(&self) -> &str {
            "mock-model"
        }

        async fn health_check(&self) -> Result<bool> {
            Ok(true)
        }
    }

    #[tokio::test]
    async fn test_cached_embedder_basic() {
        let mock_embedder = Arc::new(MockEmbedder::new(384));
        let cache_config = CacheConfig {
            size: 10,
            ttl_secs: 60,
            enabled: true,
        };

        let cached_embedder = CachedEmbedder::new(mock_embedder, cache_config);

        // 第一次调用
        let embedding1 = cached_embedder.embed("test text").await.unwrap();
        let stats1 = cached_embedder.cache_stats();
        assert_eq!(stats1.misses, 1);
        assert_eq!(stats1.hits, 0);

        // 第二次调用相同文本（应该命中缓存）
        let embedding2 = cached_embedder.embed("test text").await.unwrap();
        let stats2 = cached_embedder.cache_stats();
        assert_eq!(stats2.hits, 1);

        // 结果应该相同
        assert_eq!(embedding1, embedding2);
    }

    #[tokio::test]
    async fn test_cached_embedder_batch() {
        let mock_embedder = Arc::new(MockEmbedder::new(384));
        let cache_config = CacheConfig {
            size: 10,
            ttl_secs: 60,
            enabled: true,
        };

        let cached_embedder = CachedEmbedder::new(mock_embedder, cache_config);

        // 批量嵌入
        let texts = vec![
            "text1".to_string(),
            "text2".to_string(),
            "text3".to_string(),
        ];

        let embeddings1 = cached_embedder.embed_batch(&texts).await.unwrap();
        assert_eq!(embeddings1.len(), 3);

        // 再次嵌入（应该全部命中缓存）
        let embeddings2 = cached_embedder.embed_batch(&texts).await.unwrap();
        
        let stats = cached_embedder.cache_stats();
        assert_eq!(stats.hits, 3); // 第二次应该全部命中

        assert_eq!(embeddings1, embeddings2);
    }
}

