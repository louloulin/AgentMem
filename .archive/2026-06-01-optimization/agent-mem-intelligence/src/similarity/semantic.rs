//! 语义相似度计算实现

use agent_mem_traits::{AgentMemError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 相似度结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityResult {
    /// 相似度分数 (0.0 - 1.0)
    pub similarity: f32,
    /// 距离分数 (0.0 - ∞)
    pub distance: f32,
    /// 相似度类型
    pub similarity_type: String,
    /// 阈值
    pub threshold: f32,
    /// 是否超过阈值
    pub is_similar: bool,
}

impl SimilarityResult {
    pub fn new(similarity: f32, distance: f32, similarity_type: &str, threshold: f32) -> Self {
        Self {
            similarity,
            distance,
            similarity_type: similarity_type.to_string(),
            threshold,
            is_similar: similarity >= threshold,
        }
    }
}

/// 语义相似度配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticSimilarityConfig {
    /// 相似度算法类型
    pub algorithm: String,
    /// 余弦相似度阈值
    pub cosine_threshold: f32,
    /// 欧几里得距离阈值
    pub euclidean_threshold: f32,
    /// 混合算法权重
    pub hybrid_weights: HashMap<String, f32>,
}

impl Default for SemanticSimilarityConfig {
    fn default() -> Self {
        let mut hybrid_weights = HashMap::new();
        hybrid_weights.insert("cosine".to_string(), 0.7);
        hybrid_weights.insert("euclidean".to_string(), 0.3);

        Self {
            algorithm: "cosine".to_string(),
            cosine_threshold: 0.8,
            euclidean_threshold: 0.5,
            hybrid_weights,
        }
    }
}

/// 语义相似度计算器
pub struct SemanticSimilarity {
    config: SemanticSimilarityConfig,
}

impl SemanticSimilarity {
    /// 创建新的语义相似度计算器
    pub fn new(config: SemanticSimilarityConfig) -> Self {
        Self { config }
    }

    /// 使用默认配置创建
    pub fn default() -> Self {
        Self::new(SemanticSimilarityConfig::default())
    }

    /// 计算两个文本的语义相似度
    pub async fn calculate_similarity(&self, text1: &str, text2: &str) -> Result<f32> {
        // 简化的相似度计算实现
        // 在实际应用中，这里会使用嵌入模型计算语义相似度

        // 基于词汇重叠的简单相似度计算
        let words1: std::collections::HashSet<&str> = text1.split_whitespace().collect();
        let words2: std::collections::HashSet<&str> = text2.split_whitespace().collect();

        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();

        if union > 0 {
            Ok(intersection as f32 / union as f32)
        } else {
            Ok(0.0)
        }
    }

    /// 检测两个向量的相似性
    pub fn detect_similarity(&self, vector1: &[f32], vector2: &[f32]) -> Result<SimilarityResult> {
        if vector1.len() != vector2.len() {
            return Err(AgentMemError::validation_error(
                "Vector dimensions must match",
            ));
        }

        match self.config.algorithm.as_str() {
            "cosine" => self.detect_cosine_similarity(vector1, vector2),
            "euclidean" => self.detect_euclidean_similarity(vector1, vector2),
            "hybrid" => self.detect_hybrid_similarity(vector1, vector2),
            _ => self.detect_cosine_similarity(vector1, vector2),
        }
    }

    /// 余弦相似度检测
    fn detect_cosine_similarity(
        &self,
        vector1: &[f32],
        vector2: &[f32],
    ) -> Result<SimilarityResult> {
        let similarity = self.cosine_similarity(vector1, vector2)?;
        let distance = 1.0 - similarity;

        Ok(SimilarityResult::new(
            similarity,
            distance,
            "cosine",
            self.config.cosine_threshold,
        ))
    }

    /// 欧几里得相似度检测
    fn detect_euclidean_similarity(
        &self,
        vector1: &[f32],
        vector2: &[f32],
    ) -> Result<SimilarityResult> {
        let distance = self.euclidean_distance(vector1, vector2)?;
        let similarity = 1.0 / (1.0 + distance);

        Ok(SimilarityResult::new(
            similarity,
            distance,
            "euclidean",
            self.config.euclidean_threshold,
        ))
    }

    /// 混合相似度检测
    fn detect_hybrid_similarity(
        &self,
        vector1: &[f32],
        vector2: &[f32],
    ) -> Result<SimilarityResult> {
        let cosine_result = self.detect_cosine_similarity(vector1, vector2)?;
        let euclidean_result = self.detect_euclidean_similarity(vector1, vector2)?;

        let cosine_weight = self.config.hybrid_weights.get("cosine").unwrap_or(&0.7);
        let euclidean_weight = self.config.hybrid_weights.get("euclidean").unwrap_or(&0.3);

        let hybrid_similarity = cosine_result.similarity * cosine_weight
            + euclidean_result.similarity * euclidean_weight;

        let hybrid_distance =
            cosine_result.distance * cosine_weight + euclidean_result.distance * euclidean_weight;

        let hybrid_threshold = self.config.cosine_threshold * cosine_weight
            + self.config.euclidean_threshold * euclidean_weight;

        Ok(SimilarityResult::new(
            hybrid_similarity,
            hybrid_distance,
            "hybrid",
            hybrid_threshold,
        ))
    }

    /// 计算余弦相似度
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> Result<f32> {
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return Ok(0.0);
        }

        Ok(dot_product / (norm_a * norm_b))
    }

    /// 计算欧几里得距离
    fn euclidean_distance(&self, a: &[f32], b: &[f32]) -> Result<f32> {
        let distance = a
            .iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt();

        Ok(distance)
    }

    /// 批量相似度计算
    pub fn batch_similarity(
        &self,
        query: &[f32],
        vectors: &[Vec<f32>],
    ) -> Result<Vec<SimilarityResult>> {
        let mut results = Vec::new();

        for vector in vectors {
            let result = self.detect_similarity(query, vector)?;
            results.push(result);
        }

        Ok(results)
    }

    /// 找到最相似的向量
    pub fn find_most_similar(
        &self,
        query: &[f32],
        vectors: &[Vec<f32>],
    ) -> Result<Option<(usize, SimilarityResult)>> {
        if vectors.is_empty() {
            return Ok(None);
        }

        let results = self.batch_similarity(query, vectors)?;

        let max_result = results.iter().enumerate().max_by(|(_, a), (_, b)| {
            a.similarity
                .partial_cmp(&b.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if let Some((index, result)) = max_result {
            Ok(Some((index, result.clone())))
        } else {
            Ok(None)
        }
    }

    /// 找到前K个最相似的向量
    pub fn find_top_k_similar(
        &self,
        query: &[f32],
        vectors: &[Vec<f32>],
        k: usize,
    ) -> Result<Vec<(usize, SimilarityResult)>> {
        if vectors.is_empty() {
            return Ok(Vec::new());
        }

        let results = self.batch_similarity(query, vectors)?;

        let mut indexed_results: Vec<(usize, SimilarityResult)> =
            results.into_iter().enumerate().collect();

        // 按相似度降序排序
        indexed_results.sort_by(|(_, a), (_, b)| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 取前K个
        indexed_results.truncate(k);

        Ok(indexed_results)
    }

    /// 计算向量集合的平均相似度
    pub fn average_similarity(&self, vectors: &[Vec<f32>]) -> Result<f32> {
        if vectors.len() < 2 {
            return Ok(0.0);
        }

        let mut total_similarity = 0.0;
        let mut count = 0;

        for i in 0..vectors.len() {
            for j in (i + 1)..vectors.len() {
                let result = self.detect_similarity(&vectors[i], &vectors[j])?;
                total_similarity += result.similarity;
                count += 1;
            }
        }

        Ok(total_similarity / count as f32)
    }

    /// 更新配置
    pub fn update_config(&mut self, config: SemanticSimilarityConfig) {
        self.config = config;
    }

    /// 获取当前配置
    pub fn get_config(&self) -> &SemanticSimilarityConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_similarity_creation() {
        let similarity = SemanticSimilarity::default();
        assert_eq!(similarity.config.algorithm, "cosine");
        assert_eq!(similarity.config.cosine_threshold, 0.8);
    }

    #[test]
    fn test_cosine_similarity() {
        let similarity = SemanticSimilarity::default();

        // 测试相同向量
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let result = similarity.detect_similarity(&a, &b).unwrap();
        assert!((result.similarity - 1.0).abs() < 1e-6);
        assert_eq!(result.similarity_type, "cosine");

        // 测试正交向量
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let result = similarity.detect_similarity(&a, &b).unwrap();
        assert!((result.similarity - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_euclidean_similarity() {
        let mut config = SemanticSimilarityConfig::default();
        config.algorithm = "euclidean".to_string();
        let similarity = SemanticSimilarity::new(config);

        let a = vec![0.0, 0.0];
        let b = vec![3.0, 4.0];
        let result = similarity.detect_similarity(&a, &b).unwrap();
        assert!((result.distance - 5.0).abs() < 1e-6);
        assert_eq!(result.similarity_type, "euclidean");
    }

    #[test]
    fn test_hybrid_similarity() {
        let mut config = SemanticSimilarityConfig::default();
        config.algorithm = "hybrid".to_string();
        let similarity = SemanticSimilarity::new(config);

        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let result = similarity.detect_similarity(&a, &b).unwrap();
        assert!(result.similarity > 0.9);
        assert_eq!(result.similarity_type, "hybrid");
    }

    #[test]
    fn test_batch_similarity() {
        let similarity = SemanticSimilarity::default();
        let query = vec![1.0, 0.0];
        let vectors = vec![vec![1.0, 0.0], vec![0.0, 1.0], vec![-1.0, 0.0]];

        let results = similarity.batch_similarity(&query, &vectors).unwrap();
        assert_eq!(results.len(), 3);
        assert!((results[0].similarity - 1.0).abs() < 1e-6);
        assert!((results[1].similarity - 0.0).abs() < 1e-6);
        assert!((results[2].similarity - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn test_find_most_similar() {
        let similarity = SemanticSimilarity::default();
        let query = vec![1.0, 0.0];
        let vectors = vec![
            vec![0.0, 1.0],  // 正交
            vec![1.0, 0.0],  // 相同
            vec![-1.0, 0.0], // 相反
        ];

        let result = similarity.find_most_similar(&query, &vectors).unwrap();
        assert!(result.is_some());
        let (index, sim_result) = result.unwrap();
        assert_eq!(index, 1);
        assert!((sim_result.similarity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_find_top_k_similar() {
        let similarity = SemanticSimilarity::default();
        let query = vec![1.0, 0.0];
        let vectors = vec![
            vec![0.0, 1.0],  // 正交，相似度0
            vec![1.0, 0.0],  // 相同，相似度1
            vec![-1.0, 0.0], // 相反，相似度-1
            vec![0.5, 0.0],  // 部分相似
        ];

        let results = similarity.find_top_k_similar(&query, &vectors, 2).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, 1); // 最相似的是索引1
        assert_eq!(results[1].0, 3); // 第二相似的是索引3
    }

    #[test]
    fn test_dimension_mismatch() {
        let similarity = SemanticSimilarity::default();
        let a = vec![1.0, 2.0];
        let b = vec![1.0, 2.0, 3.0];

        let result = similarity.detect_similarity(&a, &b);
        assert!(result.is_err());
    }
}
