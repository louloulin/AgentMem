//! 搜索结果排序器
//!
//! 提供 RRF (Reciprocal Rank Fusion) 算法和其他排序策略

use super::SearchResult;
use agent_mem_traits::Result;
use std::collections::HashMap;

/// 搜索结果排序器 trait
pub trait SearchResultRanker {
    /// 融合多个搜索结果列表
    ///
    /// # Arguments
    ///
    /// * `results_lists` - 多个搜索结果列表
    /// * `weights` - 每个列表的权重
    ///
    /// # Returns
    ///
    /// 返回融合后的搜索结果列表
    fn fuse(
        &self,
        results_lists: Vec<Vec<SearchResult>>,
        weights: Vec<f32>,
    ) -> Result<Vec<SearchResult>>;
}

/// RRF (Reciprocal Rank Fusion) 排序器
///
/// RRF 算法通过倒数排名融合多个搜索结果列表，公式为：
/// RRF_score(d) = Σ 1 / (k + rank_i(d))
/// 其中 k 是常数（通常为 60），rank_i(d) 是文档 d 在第 i 个列表中的排名
pub struct RRFRanker {
    /// RRF 常数 k（默认 60）
    k: f32,
}

impl RRFRanker {
    /// 创建新的 RRF 排序器
    ///
    /// # Arguments
    ///
    /// * `k` - RRF 常数（默认 60）
    pub fn new(k: f32) -> Self {
        Self { k }
    }

    /// 使用默认参数创建 RRF 排序器
    pub fn default() -> Self {
        Self::new(60.0)
    }

    /// 计算 RRF 分数
    ///
    /// # Arguments
    ///
    /// * `rank` - 排名（从 1 开始）
    ///
    /// # Returns
    ///
    /// 返回 RRF 分数
    fn calculate_rrf_score(&self, rank: usize) -> f32 {
        1.0 / (self.k + rank as f32)
    }
}

impl SearchResultRanker for RRFRanker {
    fn fuse(
        &self,
        results_lists: Vec<Vec<SearchResult>>,
        weights: Vec<f32>,
    ) -> Result<Vec<SearchResult>> {
        if results_lists.is_empty() {
            return Ok(Vec::new());
        }

        // 验证权重数量
        if weights.len() != results_lists.len() {
            return Err(agent_mem_traits::AgentMemError::validation_error(
                "Number of weights must match number of result lists",
            ));
        }

        // 归一化权重
        let total_weight: f32 = weights.iter().sum();
        let normalized_weights: Vec<f32> = weights.iter().map(|w| w / total_weight).collect();

        // 计算每个文档的 RRF 分数
        let mut doc_scores: HashMap<String, (f32, SearchResult)> = HashMap::new();

        for (list_idx, results) in results_lists.iter().enumerate() {
            let weight = normalized_weights[list_idx];

            for (rank, result) in results.iter().enumerate() {
                let rrf_score = self.calculate_rrf_score(rank + 1) * weight;

                doc_scores
                    .entry(result.id.clone())
                    .and_modify(|(score, _)| *score += rrf_score)
                    .or_insert_with(|| (rrf_score, result.clone()));
            }
        }

        // 按分数排序
        let mut final_results: Vec<(f32, SearchResult)> = doc_scores.into_values().collect();
        final_results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        // 更新分数并返回
        Ok(final_results
            .into_iter()
            .map(|(score, mut result)| {
                result.score = score;
                result
            })
            .collect())
    }
}

/// 加权平均排序器
///
/// 使用加权平均分数融合多个搜索结果列表
pub struct WeightedAverageRanker;

impl WeightedAverageRanker {
    /// 创建新的加权平均排序器
    pub fn new() -> Self {
        Self
    }
}

impl Default for WeightedAverageRanker {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchResultRanker for WeightedAverageRanker {
    fn fuse(
        &self,
        results_lists: Vec<Vec<SearchResult>>,
        weights: Vec<f32>,
    ) -> Result<Vec<SearchResult>> {
        if results_lists.is_empty() {
            return Ok(Vec::new());
        }

        // 验证权重数量
        if weights.len() != results_lists.len() {
            return Err(agent_mem_traits::AgentMemError::validation_error(
                "Number of weights must match number of result lists",
            ));
        }

        // 归一化权重
        let total_weight: f32 = weights.iter().sum();
        let normalized_weights: Vec<f32> = weights.iter().map(|w| w / total_weight).collect();

        // 计算每个文档的加权平均分数
        let mut doc_scores: HashMap<String, (f32, f32, SearchResult)> = HashMap::new();

        for (list_idx, results) in results_lists.iter().enumerate() {
            let weight = normalized_weights[list_idx];

            for result in results {
                doc_scores
                    .entry(result.id.clone())
                    .and_modify(|(total_score, total_weight, _)| {
                        *total_score += result.score * weight;
                        *total_weight += weight;
                    })
                    .or_insert_with(|| (result.score * weight, weight, result.clone()));
            }
        }

        // 计算平均分数并排序
        let mut final_results: Vec<(f32, SearchResult)> = doc_scores
            .into_values()
            .map(|(total_score, total_weight, result)| (total_score / total_weight, result))
            .collect();

        final_results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        // 更新分数并返回
        Ok(final_results
            .into_iter()
            .map(|(score, mut result)| {
                result.score = score;
                result
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_result(id: &str, score: f32) -> SearchResult {
        SearchResult {
            id: id.to_string(),
            content: format!("content-{}", id),
            score,
            vector_score: None,
            fulltext_score: None,
            metadata: None,
        }
    }

    #[test]
    fn test_rrf_ranker() {
        let ranker = RRFRanker::default();

        let list1 = vec![
            create_test_result("doc1", 0.9),
            create_test_result("doc2", 0.8),
            create_test_result("doc3", 0.7),
        ];

        let list2 = vec![
            create_test_result("doc2", 0.95),
            create_test_result("doc1", 0.85),
            create_test_result("doc4", 0.75),
        ];

        let results = ranker.fuse(vec![list1, list2], vec![0.7, 0.3]).unwrap();

        assert!(!results.is_empty());
        // doc1 或 doc2 应该排在前两位，因为它们在两个列表中都排名靠前
        assert!(results[0].id == "doc1" || results[0].id == "doc2");
    }

    #[test]
    fn test_weighted_average_ranker() {
        let ranker = WeightedAverageRanker::new();

        let list1 = vec![
            create_test_result("doc1", 0.9),
            create_test_result("doc2", 0.8),
        ];

        let list2 = vec![
            create_test_result("doc1", 0.7),
            create_test_result("doc3", 0.6),
        ];

        let results = ranker.fuse(vec![list1, list2], vec![0.6, 0.4]).unwrap();

        assert!(!results.is_empty());
        // doc1 应该排在第一位，因为它在两个列表中都出现
        assert_eq!(results[0].id, "doc1");
    }

    #[test]
    fn test_rrf_score_calculation() {
        let ranker = RRFRanker::new(60.0);

        let score1 = ranker.calculate_rrf_score(1);
        let score2 = ranker.calculate_rrf_score(2);
        let score10 = ranker.calculate_rrf_score(10);

        assert!(score1 > score2);
        assert!(score2 > score10);
        assert!((score1 - 1.0 / 61.0).abs() < 0.001);
    }
}
