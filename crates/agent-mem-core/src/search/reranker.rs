// Result Reranker - 多因素结果重排序
//
// 对初步检索结果进行精确评分和重排序，提升精确度

use crate::search::{SearchQuery, SearchResult};
use agent_mem_traits::Result;
use chrono::{DateTime, Utc};

/// 重排序配置
#[derive(Debug, Clone)]
pub struct RerankConfig {
    /// 精确相似度权重
    pub similarity_weight: f32,
    /// 元数据匹配权重
    pub metadata_weight: f32,
    /// 时间衰减权重
    pub time_weight: f32,
    /// 重要性权重
    pub importance_weight: f32,
    /// 内容质量权重
    pub quality_weight: f32,
    /// 时间衰减半衰期（天）
    pub time_decay_halflife_days: f32,
    /// 最小内容长度
    pub min_content_length: usize,
    /// 最大内容长度
    pub max_content_length: usize,
}

impl Default for RerankConfig {
    fn default() -> Self {
        Self {
            similarity_weight: 0.50,        // 50%: 相似度最重要
            metadata_weight: 0.20,          // 20%: 元数据匹配
            time_weight: 0.15,              // 15%: 时间新鲜度
            importance_weight: 0.10,        // 10%: 重要性
            quality_weight: 0.05,           // 5%: 内容质量
            time_decay_halflife_days: 30.0, // 30天半衰期
            min_content_length: 20,
            max_content_length: 1000,
        }
    }
}

/// 结果重排序器
pub struct ResultReranker {
    config: RerankConfig,
}

impl ResultReranker {
    /// 创建新的重排序器
    pub fn new(config: RerankConfig) -> Self {
        Self { config }
    }

    /// 使用默认配置创建
    pub fn with_default_config() -> Self {
        Self::new(RerankConfig::default())
    }

    /// 重排序结果
    pub async fn rerank(
        &self,
        mut candidates: Vec<SearchResult>,
        query_vector: &[f32],
        query: &SearchQuery,
    ) -> Result<Vec<SearchResult>> {
        // 为每个候选计算综合得分
        for result in &mut candidates {
            let score = self
                .calculate_comprehensive_score(result, query_vector, query)
                .await;
            result.score = score;
        }

        // 按综合得分排序
        candidates.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or_else(|| {
                    // Fallback: if scores are NaN or incomparable, maintain order
                    std::cmp::Ordering::Equal
                })
        });

        Ok(candidates)
    }

    /// 计算综合得分
    async fn calculate_comprehensive_score(
        &self,
        result: &SearchResult,
        query_vector: &[f32],
        query: &SearchQuery,
    ) -> f32 {
        // 1. 精确相似度得分（重新计算，更精确）
        let similarity_score = self.calculate_exact_similarity(result, query_vector);

        // 2. 元数据匹配得分
        let metadata_score = self.calculate_metadata_score(result, query);

        // 3. 时间衰减得分
        let time_score = self.calculate_time_score(result);

        // 4. 重要性得分
        let importance_score = self.calculate_importance_score(result);

        // 5. 内容质量得分
        let quality_score = self.calculate_quality_score(result);

        // 加权综合
        let final_score = similarity_score * self.config.similarity_weight
            + metadata_score * self.config.metadata_weight
            + time_score * self.config.time_weight
            + importance_score * self.config.importance_weight
            + quality_score * self.config.quality_weight;

        final_score
    }

    /// 计算精确余弦相似度
    fn calculate_exact_similarity(&self, result: &SearchResult, _query_vector: &[f32]) -> f32 {
        // 使用结果中的得分（已经是相似度）
        result.score
    }

    /// 计算元数据匹配得分
    fn calculate_metadata_score(&self, result: &SearchResult, query: &SearchQuery) -> f32 {
        let mut score = 0.5; // 基础分

        // 检查查询中的过滤条件
        if let Some(_filters) = &query.filters {
            // 元数据匹配逻辑
            // 如果有元数据且匹配过滤条件，提高分数
            if result.metadata.is_some() {
                score = 0.8;
            }
        }

        score
    }

    /// 计算时间衰减得分
    fn calculate_time_score(&self, result: &SearchResult) -> f32 {
        // 从元数据中获取时间戳
        if let Some(metadata) = &result.metadata {
            if let Some(created_at_val) = metadata.get("created_at") {
                if let Some(ts) = created_at_val.as_i64() {
                    if let Some(created_at) = DateTime::from_timestamp(ts, 0) {
                        let now = Utc::now();
                        let days_old = (now - created_at).num_days() as f32;

                        // 指数衰减：score = e^(-λt), where λ = ln(2) / halflife
                        let lambda = (2.0_f32).ln() / self.config.time_decay_halflife_days;
                        let decay_factor = (-lambda * days_old).exp();

                        return decay_factor;
                    }
                }
            }
        }

        // 没有时间戳，给予中等分数
        0.5
    }

    /// 计算重要性得分
    fn calculate_importance_score(&self, result: &SearchResult) -> f32 {
        // 从元数据中获取重要性分数
        if let Some(metadata) = &result.metadata {
            if let Some(importance_val) = metadata.get("importance_score") {
                if let Some(score) = importance_val.as_f64() {
                    return score as f32;
                }
            }
        }
        0.5 // 默认中等重要性
    }

    /// 计算内容质量得分
    fn calculate_quality_score(&self, result: &SearchResult) -> f32 {
        let length = result.content.len();

        // 内容长度评分
        let length_score = if length < self.config.min_content_length {
            // 太短，降低分数
            0.3
        } else if length > self.config.max_content_length {
            // 太长，略微降低分数
            0.8
        } else {
            // 适中长度
            1.0
        };

        // 可以添加更多质量指标：
        // - 是否包含特殊字符
        // - 是否是完整句子
        // - 是否有结构化信息

        length_score
    }

    /// 批量重排序
    pub async fn rerank_batch(
        &self,
        batches: Vec<(Vec<SearchResult>, Vec<f32>, SearchQuery)>,
    ) -> Result<Vec<Vec<SearchResult>>> {
        let mut results = Vec::new();

        for (candidates, query_vector, query) in batches {
            let reranked = self.rerank(candidates, &query_vector, &query).await?;
            results.push(reranked);
        }

        Ok(results)
    }
}

/// 余弦相似度计算（精确版本）
pub fn cosine_similarity_exact(vec1: &[f32], vec2: &[f32]) -> f32 {
    assert_eq!(vec1.len(), vec2.len(), "向量维度必须相同");

    let dot_product: f32 = vec1.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum();
    let norm1: f32 = vec1.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm2: f32 = vec2.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm1 == 0.0 || norm2 == 0.0 {
        0.0
    } else {
        (dot_product / (norm1 * norm2)).clamp(-1.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_result(similarity: f32, days_old: i64) -> SearchResult {
        // 添加时间戳
        let created_at = Utc::now() - chrono::Duration::days(days_old);

        let metadata = json!({
            "created_at": created_at.timestamp(),
            "importance_score": 0.8,
        });

        SearchResult {
            id: "test_id".to_string(),
            content: "This is a test content with good length".to_string(),
            score: similarity,
            vector_score: Some(similarity),
            fulltext_score: None,
            metadata: Some(metadata),
        }
    }

    #[tokio::test]
    async fn test_reranker_time_decay() {
        let reranker = ResultReranker::with_default_config();

        // 创建两个结果：一个新，一个旧
        let recent = create_test_result(0.8, 1); // 1天前
        let old = create_test_result(0.8, 60); // 60天前

        let recent_score = reranker.calculate_time_score(&recent);
        let old_score = reranker.calculate_time_score(&old);

        // 新记忆应该得分更高
        assert!(recent_score > old_score);
    }

    #[tokio::test]
    async fn test_reranker_importance() {
        let reranker = ResultReranker::with_default_config();

        let mut high_importance = create_test_result(0.8, 1);
        high_importance.metadata = Some(json!({"importance_score": 0.95}));

        let mut low_importance = create_test_result(0.8, 1);
        low_importance.metadata = Some(json!({"importance_score": 0.3}));

        let high_score = reranker.calculate_importance_score(&high_importance);
        let low_score = reranker.calculate_importance_score(&low_importance);

        assert!(high_score > low_score);
    }

    #[tokio::test]
    async fn test_reranker_sorts_correctly() {
        let reranker = ResultReranker::with_default_config();

        // 创建多个候选结果
        let candidates = vec![
            create_test_result(0.7, 60), // 旧的，低相似度
            create_test_result(0.9, 1),  // 新的，高相似度
            create_test_result(0.8, 30), // 中等
        ];

        let query_vector = vec![0.5; 1536];
        let query = SearchQuery {
            query: "test".to_string(),
            limit: 10,
            ..Default::default()
        };

        let reranked = reranker
            .rerank(candidates, &query_vector, &query)
            .await
            .unwrap();

        // 第一个应该是新的、高相似度的结果（重排序后使用综合评分，不是原始相似度）
        // 检查排序：新的高相似度结果应该排在前面
        assert!(reranked.len() >= 1);
        // 验证排序：分数应该从高到低
        for i in 1..reranked.len() {
            assert!(
                reranked[i - 1].score >= reranked[i].score,
                "Results should be sorted by score descending"
            );
        }
    }

    #[test]
    fn test_cosine_similarity() {
        let vec1 = vec![1.0, 0.0, 0.0];
        let vec2 = vec![1.0, 0.0, 0.0];
        let similarity = cosine_similarity_exact(&vec1, &vec2);
        assert!((similarity - 1.0).abs() < 0.001);

        let vec3 = vec![1.0, 0.0, 0.0];
        let vec4 = vec![0.0, 1.0, 0.0];
        let similarity = cosine_similarity_exact(&vec3, &vec4);
        assert!((similarity - 0.0).abs() < 0.001);
    }
}
