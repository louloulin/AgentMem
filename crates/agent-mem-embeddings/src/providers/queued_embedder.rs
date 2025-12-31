//! 队列化嵌入器包装器
//! 
//! 自动收集并发请求，批量处理嵌入生成，减少 Mutex 锁竞争

use agent_mem_traits::{Embedder, Result};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::debug;

use super::embedding_queue::EmbeddingQueue;

/// 队列化嵌入器包装器
/// 
/// 包装底层嵌入器，自动使用队列批量处理请求
pub struct QueuedEmbedder {
    /// 底层嵌入器（用于批量操作和直接操作）
    inner: Arc<dyn Embedder + Send + Sync>,
    
    /// 嵌入队列（用于单个嵌入请求）
    queue: EmbeddingQueue,
    
    /// 是否启用队列（可以通过配置控制）
    queue_enabled: bool,
}

impl QueuedEmbedder {
    /// 创建新的队列化嵌入器
    /// 
    /// # 参数
    /// - `embedder`: 底层嵌入器
    /// - `batch_size`: 批处理大小（默认 32）
    /// - `batch_interval_ms`: 批处理间隔（毫秒，默认 10ms）
    /// - `queue_enabled`: 是否启用队列（默认 true）
    pub fn new(
        embedder: Arc<dyn Embedder + Send + Sync>,
        batch_size: usize,
        batch_interval_ms: u64,
        queue_enabled: bool,
    ) -> Self {
        let queue = if queue_enabled {
            EmbeddingQueue::new(embedder.clone(), batch_size, batch_interval_ms)
        } else {
            // 如果队列未启用，创建一个空的队列（不会使用）
            EmbeddingQueue::new(embedder.clone(), 1, 1)
        };
        
        Self {
            inner: embedder,
            queue,
            queue_enabled,
        }
    }
    
    /// 创建默认配置的队列化嵌入器
    pub fn with_defaults(embedder: Arc<dyn Embedder + Send + Sync>) -> Self {
        Self::new(embedder, 32, 10, true)
    }
}

#[async_trait]
impl Embedder for QueuedEmbedder {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        if self.queue_enabled {
            // 使用队列批量处理
            debug!("通过队列处理嵌入请求: {} 字符", text.len());
            self.queue.embed(text.to_string()).await
        } else {
            // 直接调用底层嵌入器
            self.inner.embed(text).await
        }
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        // 批量操作直接使用底层嵌入器（已经是批量，不需要队列）
        debug!("批量嵌入请求: {} 个文本（直接处理）", texts.len());
        self.inner.embed_batch(texts).await
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
    use crate::config::EmbeddingConfig;
    use crate::providers::FastEmbedProvider;
    
    #[tokio::test]
    #[ignore] // 需要下载模型，默认跳过
    async fn test_queued_embedder() {
        let config = EmbeddingConfig {
            provider: "fastembed".to_string(),
            model: "all-MiniLM-L6-v2".to_string(),
            dimension: 384,
            batch_size: 32,
            ..Default::default()
        };
        
        let embedder = Arc::new(FastEmbedProvider::new(config).await.unwrap());
        let queued = QueuedEmbedder::with_defaults(embedder);
        
        // 测试单个嵌入
        let embedding = queued.embed("Hello, world!").await.unwrap();
        assert_eq!(embedding.len(), 384);
        
        // 测试批量嵌入
        let texts = vec!["Hello".to_string(), "World".to_string()];
        let embeddings = queued.embed_batch(&texts).await.unwrap();
        assert_eq!(embeddings.len(), 2);
        assert_eq!(embeddings[0].len(), 384);
        
        // 测试并发嵌入（应该通过队列批量处理）
        let mut tasks = Vec::new();
        for i in 0..20 {
            let queued_clone = &queued;
            let task = tokio::spawn(async move {
                queued_clone
                    .embed(&format!("Test text {}", i))
                    .await
            });
            tasks.push(task);
        }
        
        let mut success_count = 0;
        for task in tasks {
            if task.await.unwrap().is_ok() {
                success_count += 1;
            }
        }
        
        assert_eq!(success_count, 20);
    }
}
