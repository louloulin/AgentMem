//! 向量相似度计算函数

use agent_mem_traits::{AgentMemError, Result};

/// 相似度计算方法枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimilarityMetric {
    /// 余弦相似度
    Cosine,
    /// 欧几里得距离
    Euclidean,
    /// 曼哈顿距离
    Manhattan,
    /// 点积相似度
    DotProduct,
    /// 汉明距离（用于二进制向量）
    Hamming,
}

/// 相似度计算工具集
pub struct SimilarityCalculator;

impl SimilarityCalculator {
    /// 计算余弦相似度
    /// 返回值范围：[-1, 1]，1表示完全相同，-1表示完全相反，0表示正交
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> Result<f32> {
        if a.len() != b.len() {
            return Err(AgentMemError::validation_error(
                "Vector dimensions must match",
            ));
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return Ok(0.0);
        }

        Ok(dot_product / (norm_a * norm_b))
    }

    /// 计算欧几里得距离
    /// 返回值范围：[0, +∞)，0表示完全相同，值越大表示差异越大
    pub fn euclidean_distance(a: &[f32], b: &[f32]) -> Result<f32> {
        if a.len() != b.len() {
            return Err(AgentMemError::validation_error(
                "Vector dimensions must match",
            ));
        }

        let distance = a
            .iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt();

        Ok(distance)
    }

    /// 计算曼哈顿距离（L1距离）
    /// 返回值范围：[0, +∞)，0表示完全相同，值越大表示差异越大
    pub fn manhattan_distance(a: &[f32], b: &[f32]) -> Result<f32> {
        if a.len() != b.len() {
            return Err(AgentMemError::validation_error(
                "Vector dimensions must match",
            ));
        }

        let distance = a.iter().zip(b.iter()).map(|(x, y)| (x - y).abs()).sum();

        Ok(distance)
    }

    /// 计算点积相似度
    /// 返回值范围：(-∞, +∞)，值越大表示相似度越高
    pub fn dot_product_similarity(a: &[f32], b: &[f32]) -> Result<f32> {
        if a.len() != b.len() {
            return Err(AgentMemError::validation_error(
                "Vector dimensions must match",
            ));
        }

        let dot_product = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        Ok(dot_product)
    }

    /// 计算汉明距离（用于二进制向量）
    /// 返回值范围：[0, n]，其中n是向量维度，0表示完全相同
    pub fn hamming_distance(a: &[bool], b: &[bool]) -> Result<usize> {
        if a.len() != b.len() {
            return Err(AgentMemError::validation_error(
                "Vector dimensions must match",
            ));
        }

        let distance = a
            .iter()
            .zip(b.iter())
            .map(|(x, y)| if x != y { 1 } else { 0 })
            .sum();

        Ok(distance)
    }

    /// 将距离转换为相似度（0-1范围）
    /// 使用公式：similarity = 1 / (1 + distance)
    pub fn distance_to_similarity(distance: f32) -> f32 {
        1.0 / (1.0 + distance)
    }

    /// 将相似度转换为距离
    /// 使用公式：distance = (1 - similarity) / similarity
    pub fn similarity_to_distance(similarity: f32) -> Result<f32> {
        if similarity <= 0.0 {
            return Err(AgentMemError::validation_error(
                "Similarity must be positive",
            ));
        }
        if similarity > 1.0 {
            return Err(AgentMemError::validation_error(
                "Similarity must not exceed 1.0",
            ));
        }

        Ok((1.0 - similarity) / similarity)
    }

    /// 批量计算相似度
    pub fn batch_similarity(
        query: &[f32],
        vectors: &[Vec<f32>],
        metric: SimilarityMetric,
    ) -> Result<Vec<f32>> {
        let mut results = Vec::with_capacity(vectors.len());

        for vector in vectors {
            let similarity = match metric {
                SimilarityMetric::Cosine => Self::cosine_similarity(query, vector)?,
                SimilarityMetric::Euclidean => {
                    let distance = Self::euclidean_distance(query, vector)?;
                    Self::distance_to_similarity(distance)
                }
                SimilarityMetric::Manhattan => {
                    let distance = Self::manhattan_distance(query, vector)?;
                    Self::distance_to_similarity(distance)
                }
                SimilarityMetric::DotProduct => Self::dot_product_similarity(query, vector)?,
                SimilarityMetric::Hamming => {
                    return Err(AgentMemError::validation_error(
                        "Hamming distance requires boolean vectors",
                    ));
                }
            };
            results.push(similarity);
        }

        Ok(results)
    }

    /// 找到最相似的向量索引
    pub fn find_most_similar(
        query: &[f32],
        vectors: &[Vec<f32>],
        metric: SimilarityMetric,
    ) -> Result<Option<usize>> {
        if vectors.is_empty() {
            return Ok(None);
        }

        let similarities = Self::batch_similarity(query, vectors, metric)?;

        let max_index = similarities
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(index, _)| index);

        Ok(max_index)
    }

    /// 找到前K个最相似的向量索引
    pub fn find_top_k_similar(
        query: &[f32],
        vectors: &[Vec<f32>],
        k: usize,
        metric: SimilarityMetric,
    ) -> Result<Vec<(usize, f32)>> {
        if vectors.is_empty() {
            return Ok(Vec::new());
        }

        let similarities = Self::batch_similarity(query, vectors, metric)?;

        let mut indexed_similarities: Vec<(usize, f32)> =
            similarities.into_iter().enumerate().collect();

        // 按相似度降序排序
        indexed_similarities
            .sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

        // 取前K个
        indexed_similarities.truncate(k);

        Ok(indexed_similarities)
    }

    /// 计算向量集合的平均相似度
    pub fn average_similarity(vectors: &[Vec<f32>], metric: SimilarityMetric) -> Result<f32> {
        if vectors.len() < 2 {
            return Ok(0.0);
        }

        let mut total_similarity = 0.0;
        let mut count = 0;

        for i in 0..vectors.len() {
            for j in (i + 1)..vectors.len() {
                let similarity = match metric {
                    SimilarityMetric::Cosine => Self::cosine_similarity(&vectors[i], &vectors[j])?,
                    SimilarityMetric::Euclidean => {
                        let distance = Self::euclidean_distance(&vectors[i], &vectors[j])?;
                        Self::distance_to_similarity(distance)
                    }
                    SimilarityMetric::Manhattan => {
                        let distance = Self::manhattan_distance(&vectors[i], &vectors[j])?;
                        Self::distance_to_similarity(distance)
                    }
                    SimilarityMetric::DotProduct => {
                        Self::dot_product_similarity(&vectors[i], &vectors[j])?
                    }
                    SimilarityMetric::Hamming => {
                        return Err(AgentMemError::validation_error(
                            "Hamming distance requires boolean vectors",
                        ));
                    }
                };
                total_similarity += similarity;
                count += 1;
            }
        }

        Ok(total_similarity / count as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        // 测试相同向量
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let sim = SimilarityCalculator::cosine_similarity(&a, &b).unwrap();
        assert!((sim - 1.0).abs() < 1e-6);

        // 测试正交向量
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let sim = SimilarityCalculator::cosine_similarity(&a, &b).unwrap();
        assert!((sim - 0.0).abs() < 1e-6);

        // 测试相反向量
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![-1.0, 0.0, 0.0];
        let sim = SimilarityCalculator::cosine_similarity(&a, &b).unwrap();
        assert!((sim - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn test_euclidean_distance() {
        let a = vec![0.0, 0.0];
        let b = vec![3.0, 4.0];
        let dist = SimilarityCalculator::euclidean_distance(&a, &b).unwrap();
        assert!((dist - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_manhattan_distance() {
        let a = vec![0.0, 0.0];
        let b = vec![3.0, 4.0];
        let dist = SimilarityCalculator::manhattan_distance(&a, &b).unwrap();
        assert!((dist - 7.0).abs() < 1e-6);
    }

    #[test]
    fn test_dot_product_similarity() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        let sim = SimilarityCalculator::dot_product_similarity(&a, &b).unwrap();
        assert!((sim - 32.0).abs() < 1e-6); // 1*4 + 2*5 + 3*6 = 32
    }

    #[test]
    fn test_hamming_distance() {
        let a = vec![true, false, true, false];
        let b = vec![true, true, false, false];
        let dist = SimilarityCalculator::hamming_distance(&a, &b).unwrap();
        assert_eq!(dist, 2); // 两个位置不同
    }

    #[test]
    fn test_distance_to_similarity() {
        let sim = SimilarityCalculator::distance_to_similarity(0.0);
        assert!((sim - 1.0).abs() < 1e-6);

        let sim = SimilarityCalculator::distance_to_similarity(1.0);
        assert!((sim - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_batch_similarity() {
        let query = vec![1.0, 0.0];
        let vectors = vec![vec![1.0, 0.0], vec![0.0, 1.0], vec![-1.0, 0.0]];

        let similarities =
            SimilarityCalculator::batch_similarity(&query, &vectors, SimilarityMetric::Cosine)
                .unwrap();

        assert!((similarities[0] - 1.0).abs() < 1e-6); // 相同
        assert!((similarities[1] - 0.0).abs() < 1e-6); // 正交
        assert!((similarities[2] - (-1.0)).abs() < 1e-6); // 相反
    }

    #[test]
    fn test_find_most_similar() {
        let query = vec![1.0, 0.0];
        let vectors = vec![
            vec![0.0, 1.0],  // 正交
            vec![1.0, 0.0],  // 相同
            vec![-1.0, 0.0], // 相反
        ];

        let most_similar =
            SimilarityCalculator::find_most_similar(&query, &vectors, SimilarityMetric::Cosine)
                .unwrap();

        assert_eq!(most_similar, Some(1)); // 索引1是最相似的
    }

    #[test]
    fn test_find_top_k_similar() {
        let query = vec![1.0, 0.0];
        let vectors = vec![
            vec![0.0, 1.0],  // 正交，相似度0
            vec![1.0, 0.0],  // 相同，相似度1
            vec![-1.0, 0.0], // 相反，相似度-1
            vec![0.5, 0.0],  // 部分相似，相似度0.5
        ];

        let top_k =
            SimilarityCalculator::find_top_k_similar(&query, &vectors, 2, SimilarityMetric::Cosine)
                .unwrap();

        assert_eq!(top_k.len(), 2);
        assert_eq!(top_k[0].0, 1); // 最相似的是索引1
        assert_eq!(top_k[1].0, 3); // 第二相似的是索引3
    }

    #[test]
    fn test_dimension_mismatch() {
        let a = vec![1.0, 2.0];
        let b = vec![1.0, 2.0, 3.0];

        assert!(SimilarityCalculator::cosine_similarity(&a, &b).is_err());
        assert!(SimilarityCalculator::euclidean_distance(&a, &b).is_err());
        assert!(SimilarityCalculator::manhattan_distance(&a, &b).is_err());
        assert!(SimilarityCalculator::dot_product_similarity(&a, &b).is_err());
    }
}
