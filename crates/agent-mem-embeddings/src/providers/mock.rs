//! Mock Embedder 实现
//!
//! 用于测试或在真实嵌入模型不可用时提供后备功能。
//! 使用基于文本哈希的确定性向量生成。

use crate::config::EmbeddingConfig;
use agent_mem_traits::{AgentMemError, Embedder, Result};
use async_trait::async_trait;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tracing::info;

/// Mock Embedder 实现
///
/// 使用简单的确定性算法生成嵌入向量：
/// - 基于输入文本的哈希值生成一致的向量
/// - 每次调用返回相同文本的相同向量
///
/// **注意**: 这不是一个语义嵌入器，仅用于测试和后备场景。
/// 真实的语义相似性不会被保留。
pub struct MockEmbedder {
    /// 嵌入维度
    dimension: usize,
    /// 调用计数
    call_count: Arc<AtomicUsize>,
}

impl MockEmbedder {
    /// 创建新的 Mock Embedder
    pub fn new(dimension: usize) -> Self {
        info!("创建 MockEmbedder (dimension: {})", dimension);
        Self {
            dimension,
            call_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// 从配置创建
    pub fn from_config(config: &EmbeddingConfig) -> Self {
        let dim = if config.dimension > 0 { config.dimension } else { 384 };
        Self::new(dim)
    }

    /// 获取调用计数
    pub fn call_count(&self) -> usize {
        self.call_count.load(Ordering::Relaxed)
    }

    /// 简单的字符串哈希函数 - 生成确定性向量
    /// 为每个维度生成不同的值，而不是返回单个哈希
    fn hash_to_vector(text: &str, dimension: usize) -> Vec<f32> {
        let bytes = text.as_bytes();
        let mut vector = Vec::with_capacity(dimension);
        
        for i in 0..dimension {
            // 使用多个哈希轮的组合
            let mut hash: u64 = 0;
            for (j, &byte) in bytes.iter().enumerate() {
                let idx = (i + j) % bytes.len();
                hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
                hash = hash.rotate_right((idx as u32).wrapping_add(i as u32));
            }
            // 将哈希转换为 0-1 之间的浮点数
            let value = (hash as f64 % 1.0).abs() as f32;
            vector.push(value);
        }
        
        // 归一化向量
        let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in &mut vector {
                *v /= norm;
            }
        }
        
        vector
    }
}

#[async_trait::async_trait]
impl Embedder for MockEmbedder {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        self.call_count.fetch_add(1, Ordering::Relaxed);
        // 使用哈希函数生成归一化的确定性向量
        Ok(Self::hash_to_vector(text, self.dimension))
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
        "mock-embedder"
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_embedder_deterministic() {
        let embedder = MockEmbedder::new(384);

        let text = "Hello, world!";
        let embedding1 = embedder.embed(text).await.unwrap();
        let embedding2 = embedder.embed(text).await.unwrap();

        assert_eq!(embedding1, embedding2);
        assert_eq!(embedding1.len(), 384);
    }

    #[tokio::test]
    async fn test_mock_embedder_different_texts() {
        let embedder = MockEmbedder::new(384);

        let embedding1 = embedder.embed("Hello").await.unwrap();
        let embedding2 = embedder.embed("World").await.unwrap();

        assert_ne!(embedding1, embedding2);
    }

    #[tokio::test]
    async fn test_mock_embedder_call_count() {
        let embedder = MockEmbedder::new(384);
        assert_eq!(embedder.call_count(), 0);

        embedder.embed("test").await.unwrap();
        assert_eq!(embedder.call_count(), 1);

        embedder.embed("test").await.unwrap();
        assert_eq!(embedder.call_count(), 2);
    }
}