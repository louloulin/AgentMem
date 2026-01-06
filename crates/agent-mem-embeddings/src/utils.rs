//! 嵌入工具函数

use agent_mem_traits::{AgentMemError, Result};

/// 嵌入工具集
pub struct EmbeddingUtils;

impl EmbeddingUtils {
    /// 验证嵌入向量的维度
    pub fn validate_embedding_dimension(embedding: &[f32], expected_dim: usize) -> Result<()> {
        if embedding.len() != expected_dim {
            return Err(AgentMemError::validation_error(format!(
                "Embedding dimension mismatch: expected {}, got {}",
                expected_dim,
                embedding.len()
            )));
        }
        Ok(())
    }

    /// 批量验证嵌入向量的维度
    pub fn validate_embeddings_dimension(
        embeddings: &[Vec<f32>],
        expected_dim: usize,
    ) -> Result<()> {
        for (i, embedding) in embeddings.iter().enumerate() {
            if embedding.len() != expected_dim {
                return Err(AgentMemError::validation_error(format!(
                    "Embedding {} dimension mismatch: expected {}, got {}",
                    i,
                    expected_dim,
                    embedding.len()
                )));
            }
        }
        Ok(())
    }

    /// 标准化嵌入向量（L2范数）
    pub fn normalize_embedding(embedding: &mut [f32]) -> Result<()> {
        let norm = Self::l2_norm(embedding);
        if norm == 0.0 {
            return Err(AgentMemError::validation_error(
                "Cannot normalize zero embedding",
            ));
        }

        for value in embedding.iter_mut() {
            *value /= norm;
        }

        Ok(())
    }

    /// 批量标准化嵌入向量
    pub fn normalize_embeddings(embeddings: &mut [Vec<f32>]) -> Result<()> {
        for embedding in embeddings.iter_mut() {
            Self::normalize_embedding(embedding)?;
        }
        Ok(())
    }

    /// 计算嵌入向量的L2范数
    pub fn l2_norm(embedding: &[f32]) -> f32 {
        embedding.iter().map(|x| x * x).sum::<f32>().sqrt()
    }

    /// 计算两个嵌入向量的余弦相似度
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> Result<f32> {
        if a.len() != b.len() {
            return Err(AgentMemError::validation_error(
                "Embedding dimensions must match",
            ));
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a = Self::l2_norm(a);
        let norm_b = Self::l2_norm(b);

        if norm_a == 0.0 || norm_b == 0.0 {
            return Ok(0.0);
        }

        Ok(dot_product / (norm_a * norm_b))
    }

    /// 计算嵌入向量的平均值
    pub fn average_embeddings(embeddings: &[Vec<f32>]) -> Result<Vec<f32>> {
        if embeddings.is_empty() {
            return Err(AgentMemError::validation_error(
                "Cannot average empty embedding list",
            ));
        }

        let dimension = embeddings[0].len();
        Self::validate_embeddings_dimension(embeddings, dimension)?;

        let mut result = vec![0.0; dimension];
        for embedding in embeddings {
            for (i, value) in embedding.iter().enumerate() {
                result[i] += value;
            }
        }

        let count = embeddings.len() as f32;
        for value in result.iter_mut() {
            *value /= count;
        }

        Ok(result)
    }

    /// 检查嵌入向量是否为零向量
    pub fn is_zero_embedding(embedding: &[f32], epsilon: f32) -> bool {
        embedding.iter().all(|&x| x.abs() < epsilon)
    }

    /// 计算嵌入向量的统计信息
    pub fn embedding_stats(embedding: &[f32]) -> EmbeddingStats {
        if embedding.is_empty() {
            return EmbeddingStats::default();
        }

        let mut min = embedding[0];
        let mut max = embedding[0];
        let mut sum = 0.0;

        for &value in embedding {
            if value < min {
                min = value;
            }
            if value > max {
                max = value;
            }
            sum += value;
        }

        let mean = sum / embedding.len() as f32;
        let variance =
            embedding.iter().map(|&x| (x - mean).powi(2)).sum::<f32>() / embedding.len() as f32;

        EmbeddingStats {
            dimension: embedding.len(),
            min,
            max,
            mean,
            variance,
            std_dev: variance.sqrt(),
            l2_norm: Self::l2_norm(embedding),
        }
    }

    /// 将文本分割成适合嵌入的块
    pub fn split_text_for_embedding(text: &str, max_tokens: usize) -> Vec<String> {
        // 简单的基于单词的分割，实际实现可能需要更复杂的tokenization
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut chunks = Vec::new();
        let mut current_chunk = Vec::new();
        let mut current_tokens = 0;

        for word in words {
            let word_tokens = Self::estimate_tokens(word);

            if current_tokens + word_tokens > max_tokens && !current_chunk.is_empty() {
                chunks.push(current_chunk.join(" "));
                current_chunk.clear();
                current_tokens = 0;
            }

            current_chunk.push(word);
            current_tokens += word_tokens;
        }

        if !current_chunk.is_empty() {
            chunks.push(current_chunk.join(" "));
        }

        chunks
    }

    /// 估算文本的token数量（简单实现）
    fn estimate_tokens(text: &str) -> usize {
        // 简单估算：平均每4个字符一个token
        text.len().div_ceil(4)
    }

    /// 创建零嵌入向量
    pub fn create_zero_embedding(dimension: usize) -> Vec<f32> {
        vec![0.0; dimension]
    }

    /// 创建随机嵌入向量（用于测试）
    pub fn create_random_embedding(dimension: usize) -> Vec<f32> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        dimension.hash(&mut hasher);
        let seed = hasher.finish();

        // 使用简单的线性同余生成器
        let mut rng = seed;
        (0..dimension)
            .map(|_| {
                rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
                ((rng / 65536) % 32768) as f32 / 32768.0 - 0.5
            })
            .collect()
    }
}

/// 嵌入统计信息结构
#[derive(Debug, Clone, PartialEq)]
pub struct EmbeddingStats {
    pub dimension: usize,
    pub min: f32,
    pub max: f32,
    pub mean: f32,
    pub variance: f32,
    pub std_dev: f32,
    pub l2_norm: f32,
}

impl Default for EmbeddingStats {
    fn default() -> Self {
        Self {
            dimension: 0,
            min: 0.0,
            max: 0.0,
            mean: 0.0,
            variance: 0.0,
            std_dev: 0.0,
            l2_norm: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_embedding_dimension() {
        let embedding = vec![1.0, 2.0, 3.0];
        assert!(EmbeddingUtils::validate_embedding_dimension(&embedding, 3).is_ok());
        assert!(EmbeddingUtils::validate_embedding_dimension(&embedding, 2).is_err());
    }

    #[test]
    fn test_normalize_embedding() {
        let mut embedding = vec![3.0, 4.0];
        EmbeddingUtils::normalize_embedding(&mut embedding).unwrap();

        // 3-4-5三角形，标准化后应该是[0.6, 0.8]
        assert!((embedding[0] - 0.6).abs() < 1e-6);
        assert!((embedding[1] - 0.8).abs() < 1e-6);

        // 验证L2范数为1
        let norm = EmbeddingUtils::l2_norm(&embedding);
        assert!((norm - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let sim = EmbeddingUtils::cosine_similarity(&a, &b).unwrap();
        assert!((sim - 1.0).abs() < 1e-6);

        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let sim = EmbeddingUtils::cosine_similarity(&a, &b).unwrap();
        assert!((sim - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_average_embeddings() {
        let embeddings = vec![vec![1.0, 2.0], vec![3.0, 4.0], vec![5.0, 6.0]];
        let result = EmbeddingUtils::average_embeddings(&embeddings).unwrap();
        assert_eq!(result, vec![3.0, 4.0]);
    }

    #[test]
    fn test_is_zero_embedding() {
        let zero_embedding = vec![0.0, 0.0, 0.0];
        assert!(EmbeddingUtils::is_zero_embedding(&zero_embedding, 1e-6));

        let non_zero_embedding = vec![0.0, 0.0, 0.1];
        assert!(!EmbeddingUtils::is_zero_embedding(
            &non_zero_embedding,
            1e-6
        ));
    }

    #[test]
    fn test_embedding_stats() {
        let embedding = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = EmbeddingUtils::embedding_stats(&embedding);

        assert_eq!(stats.dimension, 5);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
        assert_eq!(stats.mean, 3.0);
    }

    #[test]
    fn test_split_text_for_embedding() {
        let text = "This is a long text that needs to be split into smaller chunks for embedding";
        let chunks = EmbeddingUtils::split_text_for_embedding(text, 20);

        assert!(!chunks.is_empty());
        for chunk in &chunks {
            let estimated_tokens = EmbeddingUtils::estimate_tokens(chunk);
            assert!(estimated_tokens <= 20);
        }
    }

    #[test]
    fn test_create_zero_embedding() {
        let embedding = EmbeddingUtils::create_zero_embedding(5);
        assert_eq!(embedding.len(), 5);
        assert!(embedding.iter().all(|&x| x == 0.0));
    }

    #[test]
    fn test_create_random_embedding() {
        let embedding = EmbeddingUtils::create_random_embedding(10);
        assert_eq!(embedding.len(), 10);

        // 检查不是全零
        assert!(!embedding.iter().all(|&x| x == 0.0));

        // 检查值在合理范围内
        assert!(embedding.iter().all(|&x| x >= -0.5 && x <= 0.5));
    }

    #[test]
    fn test_dimension_mismatch_errors() {
        let a = vec![1.0, 2.0];
        let b = vec![1.0, 2.0, 3.0];

        assert!(EmbeddingUtils::cosine_similarity(&a, &b).is_err());
    }
}
