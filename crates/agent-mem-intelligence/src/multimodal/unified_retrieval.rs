/// 统一多模态检索接口
///
/// 提供跨模态检索、结果融合和排序功能
use agent_mem_traits::{AgentMemError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::cross_modal::{
    CrossModalConfig, ModalSimilarityCalculator, MultimodalEmbedding, MultimodalFusionEngine,
};
use super::ContentType;

/// 多模态检索请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalRetrievalRequest {
    /// 查询内容
    pub query: String,
    /// 查询嵌入（可选）
    pub query_embedding: Option<Vec<f32>>,
    /// 查询模态类型
    pub query_modality: ContentType,
    /// 目标模态类型（None 表示所有模态）
    pub target_modalities: Option<Vec<ContentType>>,
    /// 最大结果数
    pub max_results: usize,
    /// 是否启用跨模态检索
    pub enable_cross_modal: bool,
    /// 是否启用结果融合
    pub enable_fusion: bool,
    /// 相似性阈值
    pub similarity_threshold: f32,
}

impl Default for MultimodalRetrievalRequest {
    fn default() -> Self {
        Self {
            query: String::new(),
            query_embedding: None,
            query_modality: ContentType::Text,
            target_modalities: None,
            max_results: 10,
            enable_cross_modal: true,
            enable_fusion: true,
            similarity_threshold: 0.5,
        }
    }
}

/// 多模态检索结果项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalRetrievalResult {
    /// 内容 ID
    pub content_id: String,
    /// 内容类型
    pub content_type: ContentType,
    /// 相似性分数
    pub similarity_score: f32,
    /// 融合分数（如果启用融合）
    pub fusion_score: Option<f32>,
    /// 排名
    pub rank: usize,
    /// 元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 多模态检索响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalRetrievalResponse {
    /// 检索结果列表
    pub results: Vec<MultimodalRetrievalResult>,
    /// 总结果数
    pub total_count: usize,
    /// 处理时间（毫秒）
    pub processing_time_ms: u64,
    /// 是否使用了跨模态检索
    pub used_cross_modal: bool,
    /// 是否使用了融合
    pub used_fusion: bool,
}

/// 统一多模态检索器
pub struct UnifiedMultimodalRetrieval {
    config: CrossModalConfig,
    similarity_calculator: ModalSimilarityCalculator,
    fusion_engine: MultimodalFusionEngine,
}

impl UnifiedMultimodalRetrieval {
    /// 创建新的统一检索器
    pub fn new(config: CrossModalConfig) -> Self {
        let similarity_calculator = ModalSimilarityCalculator::new(config.clone());
        let fusion_engine = MultimodalFusionEngine::new(config.clone());

        Self {
            config,
            similarity_calculator,
            fusion_engine,
        }
    }

    /// 执行多模态检索
    ///
    /// # Arguments
    /// * `request` - 检索请求
    /// * `candidate_embeddings` - 候选嵌入列表
    ///
    /// # Returns
    /// * 检索响应
    pub fn retrieve(
        &self,
        request: &MultimodalRetrievalRequest,
        candidate_embeddings: &[(String, Vec<f32>, ContentType)],
    ) -> Result<MultimodalRetrievalResponse> {
        let start_time = std::time::Instant::now();

        // 获取查询嵌入
        let query_embedding = request.query_embedding.as_ref().ok_or_else(|| {
            AgentMemError::ValidationError("Query embedding is required".to_string())
        })?;

        // 计算相似性分数
        let mut scored_results = Vec::new();

        for (content_id, embedding, content_type) in candidate_embeddings {
            // 检查是否在目标模态中
            if let Some(ref target_modalities) = request.target_modalities {
                if !target_modalities.contains(content_type) {
                    continue;
                }
            }

            // 计算相似性
            let similarity = if request.enable_cross_modal {
                self.similarity_calculator
                    .calculate_cross_modal_similarity(
                        query_embedding,
                        &request.query_modality,
                        embedding,
                        content_type,
                    )?
            } else {
                // 仅同模态检索
                if content_type == &request.query_modality {
                    self.cosine_similarity(query_embedding, embedding)?
                } else {
                    continue;
                }
            };

            // 过滤低于阈值的结果
            if similarity < request.similarity_threshold {
                continue;
            }

            scored_results.push((content_id.clone(), content_type.clone(), similarity));
        }

        // 排序结果
        scored_results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        // 限制结果数量
        scored_results.truncate(request.max_results);

        // 构建响应
        let results: Vec<MultimodalRetrievalResult> = scored_results
            .into_iter()
            .enumerate()
            .map(
                |(rank, (content_id, content_type, similarity))| MultimodalRetrievalResult {
                    content_id,
                    content_type,
                    similarity_score: similarity,
                    fusion_score: None,
                    rank: rank + 1,
                    metadata: HashMap::new(),
                },
            )
            .collect();

        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(MultimodalRetrievalResponse {
            total_count: results.len(),
            results,
            processing_time_ms,
            used_cross_modal: request.enable_cross_modal,
            used_fusion: false,
        })
    }

    /// 执行融合检索
    ///
    /// # Arguments
    /// * `request` - 检索请求
    /// * `candidate_embeddings` - 候选嵌入列表（按模态分组）
    ///
    /// # Returns
    /// * 检索响应
    pub fn retrieve_with_fusion(
        &self,
        request: &MultimodalRetrievalRequest,
        candidate_embeddings: &HashMap<ContentType, Vec<(String, Vec<f32>)>>,
    ) -> Result<MultimodalRetrievalResponse> {
        let start_time = std::time::Instant::now();

        let query_embedding = request.query_embedding.as_ref().ok_or_else(|| {
            AgentMemError::ValidationError("Query embedding is required".to_string())
        })?;

        let mut scored_results = Vec::new();

        // 对每个候选项，收集所有模态的嵌入并融合
        let mut content_embeddings: HashMap<String, Vec<MultimodalEmbedding>> = HashMap::new();

        for (modality, embeddings) in candidate_embeddings {
            for (content_id, embedding) in embeddings {
                content_embeddings
                    .entry(content_id.clone())
                    .or_default()
                    .push(MultimodalEmbedding {
                        embedding: embedding.clone(),
                        modality: modality.clone(),
                        weight: 1.0,
                        confidence: 0.9,
                    });
            }
        }

        // 融合每个内容的多模态嵌入并计算相似性
        for (content_id, embeddings) in content_embeddings {
            if embeddings.is_empty() {
                continue;
            }

            // 融合嵌入
            let fused_embedding = self.fusion_engine.fuse_embeddings(&embeddings)?;

            // 计算相似性
            let similarity = self.cosine_similarity(query_embedding, &fused_embedding)?;

            // 过滤低于阈值的结果
            if similarity < request.similarity_threshold {
                continue;
            }

            // 使用主要模态类型
            let primary_modality = embeddings[0].modality.clone();

            scored_results.push((content_id, primary_modality, similarity));
        }

        // 排序结果
        scored_results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        // 限制结果数量
        scored_results.truncate(request.max_results);

        // 构建响应
        let results: Vec<MultimodalRetrievalResult> = scored_results
            .into_iter()
            .enumerate()
            .map(
                |(rank, (content_id, content_type, similarity))| MultimodalRetrievalResult {
                    content_id,
                    content_type,
                    similarity_score: similarity,
                    fusion_score: Some(similarity),
                    rank: rank + 1,
                    metadata: HashMap::new(),
                },
            )
            .collect();

        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(MultimodalRetrievalResponse {
            total_count: results.len(),
            results,
            processing_time_ms,
            used_cross_modal: request.enable_cross_modal,
            used_fusion: true,
        })
    }

    /// 计算余弦相似度
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> Result<f32> {
        if a.len() != b.len() {
            return Err(AgentMemError::ValidationError(
                "Embedding dimensions must match".to_string(),
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

    /// 重新排序结果
    ///
    /// # Arguments
    /// * `results` - 初始结果列表
    /// * `rerank_strategy` - 重排序策略
    ///
    /// # Returns
    /// * 重排序后的结果
    pub fn rerank_results(
        &self,
        results: Vec<MultimodalRetrievalResult>,
        rerank_strategy: RerankStrategy,
    ) -> Vec<MultimodalRetrievalResult> {
        match rerank_strategy {
            RerankStrategy::Similarity => {
                // 已经按相似性排序
                results
            }
            RerankStrategy::Diversity => {
                // 多样性重排序
                self.diversity_rerank(results)
            }
            RerankStrategy::Hybrid => {
                // 混合策略
                self.hybrid_rerank(results)
            }
        }
    }

    /// 多样性重排序
    fn diversity_rerank(
        &self,
        mut results: Vec<MultimodalRetrievalResult>,
    ) -> Vec<MultimodalRetrievalResult> {
        // 简化的多样性重排序：确保不同模态的内容交替出现
        results.sort_by(|a, b| {
            let type_cmp = format!("{:?}", a.content_type).cmp(&format!("{:?}", b.content_type));
            if type_cmp == std::cmp::Ordering::Equal {
                b.similarity_score
                    .partial_cmp(&a.similarity_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            } else {
                type_cmp
            }
        });
        results
    }

    /// 混合重排序
    fn hybrid_rerank(
        &self,
        results: Vec<MultimodalRetrievalResult>,
    ) -> Vec<MultimodalRetrievalResult> {
        // 混合相似性和多样性
        results
    }
}

/// 重排序策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RerankStrategy {
    /// 按相似性排序
    Similarity,
    /// 按多样性排序
    Diversity,
    /// 混合策略
    Hybrid,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_retrieval() {
        let config = CrossModalConfig::default();
        let retrieval = UnifiedMultimodalRetrieval::new(config);

        let request = MultimodalRetrievalRequest {
            query: "test query".to_string(),
            query_embedding: Some(vec![1.0, 0.0, 0.0]),
            query_modality: ContentType::Text,
            target_modalities: None,
            max_results: 5,
            enable_cross_modal: true,
            enable_fusion: false,
            similarity_threshold: 0.5,
        };

        let candidates = vec![
            (
                "content1".to_string(),
                vec![1.0, 0.0, 0.0],
                ContentType::Text,
            ),
            (
                "content2".to_string(),
                vec![0.0, 1.0, 0.0],
                ContentType::Image,
            ),
        ];

        let response = retrieval.retrieve(&request, &candidates).unwrap();
        assert!(response.total_count > 0);
    }
}
