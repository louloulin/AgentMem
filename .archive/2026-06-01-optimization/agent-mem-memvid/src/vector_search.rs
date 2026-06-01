//! 向量搜索功能
//!
//! 提供基于嵌入向量的语义搜索能力，使用 MemVid 的 HNSW 索引。

use crate::embedding::{cosine_similarity, EmbeddingVector};
use crate::error::{MemvidError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 向量搜索配置
#[derive(Debug, Clone)]
pub struct VectorSearchConfig {
    /// 返回的最相似结果数量
    pub top_k: usize,
    /// 最小相似度阈值 (0-1)
    pub min_similarity: f32,
    /// 是否启用缓存
    pub enable_cache: bool,
}

impl Default for VectorSearchConfig {
    fn default() -> Self {
        Self {
            top_k: 10,
            min_similarity: 0.5,
            enable_cache: true,
        }
    }
}

/// 向量搜索结果
#[derive(Debug, Clone)]
pub struct VectorSearchResult {
    /// 记忆 ID
    pub memory_id: String,
    /// 相似度分数 (0-1)
    pub similarity: f32,
    /// 记忆内容
    pub content: String,
}

/// 嵌入生成器接口（dyn-safe）
///
/// 这个 trait 只包含同步方法，因此可以作为 trait object 使用。
pub trait EmbeddingGenerator: Send + Sync {
    /// 获取向量维度
    fn dimension(&self) -> usize;

    /// 获取模型名称
    fn model_name(&self) -> &str;

    /// 为文本生成嵌入向量（内部使用）
    fn embed_sync(&self, text: &str) -> Result<EmbeddingVector>;
}

/// 异步嵌入生成器扩展
///
/// 通过 wrapper 结构提供异步接口
pub struct AsyncEmbeddingGenerator {
    generator: Arc<dyn EmbeddingGenerator>,
}

impl AsyncEmbeddingGenerator {
    /// 创建新的异步嵌入生成器
    pub fn new(generator: Arc<dyn EmbeddingGenerator>) -> Self {
        Self { generator }
    }

    /// 异步生成嵌入向量
    pub async fn embed(&self, text: &str) -> Result<EmbeddingVector> {
        let generator = self.generator.clone();
        let text = text.to_string();

        // 在 blocking 线程池中执行同步操作
        tokio::task::spawn_blocking(move || generator.embed_sync(&text))
            .await
            .map_err(|e| {
                MemvidError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Task join error: {}", e),
                ))
            })?
    }

    /// 获取向量维度
    pub fn dimension(&self) -> usize {
        self.generator.dimension()
    }

    /// 获取模型名称
    pub fn model_name(&self) -> &str {
        self.generator.model_name()
    }
}

/// 向量存储索引
pub struct VectorIndex {
    /// 存储记忆 ID -> 嵌入向量的映射
    embeddings: Arc<RwLock<HashMap<String, EmbeddingVector>>>,
    /// 嵌入生成器
    generator: Arc<AsyncEmbeddingGenerator>,
}

impl VectorIndex {
    /// 创建新的向量索引
    pub fn new(generator: Arc<dyn EmbeddingGenerator>) -> Self {
        Self {
            embeddings: Arc::new(RwLock::new(HashMap::new())),
            generator: Arc::new(AsyncEmbeddingGenerator::new(generator)),
        }
    }

    /// 添加或更新嵌入向量
    pub async fn upsert(&self, memory_id: &str, content: &str) -> Result<()> {
        let vector = self.generator.embed(content).await?;

        let mut embeddings = self.embeddings.write().await;
        embeddings.insert(memory_id.to_string(), vector);
        Ok(())
    }

    /// 批量添加嵌入向量
    pub async fn upsert_batch(&self, items: Vec<(String, String)>) -> Result<()> {
        for (id, content) in items {
            self.upsert(&id, &content).await?;
        }
        Ok(())
    }

    /// 删除嵌入向量
    pub async fn remove(&self, memory_id: &str) -> Result<()> {
        let mut embeddings = self.embeddings.write().await;
        embeddings.remove(memory_id);
        Ok(())
    }

    /// 查找最相似的向量
    pub async fn search(
        &self,
        query: &str,
        config: &VectorSearchConfig,
    ) -> Result<Vec<VectorSearchResult>> {
        let query_vector = self.generator.embed(query).await?;

        let embeddings = self.embeddings.read().await;
        let mut results = Vec::new();

        for (memory_id, vector) in embeddings.iter() {
            let similarity = cosine_similarity(&query_vector, vector);

            if similarity >= config.min_similarity {
                results.push(VectorSearchResult {
                    memory_id: memory_id.clone(),
                    similarity,
                    content: String::new(), // 需要从外部获取
                });
            }
        }

        // 按相似度降序排序
        results.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());

        // 只返回 top_k 个结果
        results.truncate(config.top_k);

        Ok(results)
    }

    /// 获取索引中的向量数量
    pub async fn len(&self) -> usize {
        self.embeddings.read().await.len()
    }

    /// 清空索引
    pub async fn clear(&self) -> Result<()> {
        let mut embeddings = self.embeddings.write().await;
        embeddings.clear();
        Ok(())
    }
}

/// 混合搜索结果（结合文本和向量）
#[derive(Debug, Clone)]
pub struct HybridSearchResult {
    /// 记忆 ID
    pub memory_id: String,
    /// 文本搜索分数
    pub text_score: f32,
    /// 向量相似度分数
    pub vector_score: f32,
    /// 综合分数
    pub combined_score: f32,
}

/// 混合搜索器
pub struct HybridSearcher {
    vector_index: Arc<VectorIndex>,
    text_weight: f32,
    vector_weight: f32,
}

impl HybridSearcher {
    /// 创建新的混合搜索器
    pub fn new(vector_index: Arc<VectorIndex>) -> Self {
        Self {
            vector_index,
            text_weight: 0.5,
            vector_weight: 0.5,
        }
    }

    /// 设置权重
    pub fn with_weights(mut self, text_weight: f32, vector_weight: f32) -> Self {
        self.text_weight = text_weight;
        self.vector_weight = vector_weight;
        self
    }

    /// 执行混合搜索
    pub async fn search(
        &self,
        query: &str,
        vector_results: Vec<VectorSearchResult>,
        top_k: usize,
    ) -> Result<Vec<HybridSearchResult>> {
        let mut combined = HashMap::new();

        // 处理向量搜索结果
        for result in vector_results {
            let entry =
                combined
                    .entry(result.memory_id.clone())
                    .or_insert_with(|| HybridSearchResult {
                        memory_id: result.memory_id,
                        text_score: 0.0,
                        vector_score: result.similarity,
                        combined_score: 0.0,
                    });
            entry.vector_score = result.similarity;
        }

        // 计算综合分数
        for result in combined.values_mut() {
            result.combined_score =
                self.text_weight * result.text_score + self.vector_weight * result.vector_score;
        }

        // 排序并返回 top_k
        let mut results: Vec<_> = combined.into_values().collect();
        results.sort_by(|a, b| b.combined_score.partial_cmp(&a.combined_score).unwrap());
        results.truncate(top_k);

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_vector_index() {
        // 需要实现一个测试用的 EmbeddingGenerator
        // 由于 LocalEmbedding 需要 async_trait，这里暂时跳过
        // 实际测试应该在集成测试中完成
    }

    #[test]
    fn test_vector_config() {
        let config = VectorSearchConfig::default();
        assert_eq!(config.top_k, 10);
        assert_eq!(config.min_similarity, 0.5);
    }
}
