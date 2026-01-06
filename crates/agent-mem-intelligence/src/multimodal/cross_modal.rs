/// 跨模态记忆关联模块
///
/// 实现跨模态嵌入对齐、相似性计算和融合算法
use agent_mem_traits::{AgentMemError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::ContentType;

/// 跨模态对齐配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossModalConfig {
    /// 是否启用跨模态对齐
    pub enable_alignment: bool,
    /// 对齐算法类型
    pub alignment_algorithm: AlignmentAlgorithm,
    /// 相似性阈值
    pub similarity_threshold: f32,
    /// 融合策略
    pub fusion_strategy: FusionStrategy,
    /// 模态权重
    pub modality_weights: HashMap<String, f32>,
}

impl Default for CrossModalConfig {
    fn default() -> Self {
        let mut modality_weights = HashMap::new();
        modality_weights.insert("text".to_string(), 1.0);
        modality_weights.insert("image".to_string(), 0.8);
        modality_weights.insert("audio".to_string(), 0.7);
        modality_weights.insert("video".to_string(), 0.9);

        Self {
            enable_alignment: true,
            alignment_algorithm: AlignmentAlgorithm::LinearProjection,
            similarity_threshold: 0.7,
            fusion_strategy: FusionStrategy::WeightedAverage,
            modality_weights,
        }
    }
}

/// 对齐算法类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlignmentAlgorithm {
    /// 线性投影
    LinearProjection,
    /// CCA (典型相关分析)
    CCA,
    /// 深度对齐网络
    DeepAlignment,
    /// 注意力机制
    Attention,
}

/// 融合策略
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FusionStrategy {
    /// 加权平均
    WeightedAverage,
    /// 最大池化
    MaxPooling,
    /// 注意力融合
    AttentionFusion,
    /// 级联融合
    ConcatenateFusion,
}

/// 跨模态嵌入对齐器
pub struct CrossModalAligner {
    config: CrossModalConfig,
    /// 模态间的转换矩阵
    transformation_matrices: HashMap<(ContentType, ContentType), Vec<Vec<f32>>>,
}

impl CrossModalAligner {
    /// 创建新的跨模态对齐器
    pub fn new(config: CrossModalConfig) -> Self {
        Self {
            config,
            transformation_matrices: HashMap::new(),
        }
    }

    /// 对齐两个不同模态的嵌入
    ///
    /// # Arguments
    /// * `source_embedding` - 源模态嵌入
    /// * `source_type` - 源模态类型
    /// * `target_type` - 目标模态类型
    ///
    /// # Returns
    /// * 对齐后的嵌入向量
    pub fn align_embeddings(
        &self,
        source_embedding: &[f32],
        source_type: &ContentType,
        target_type: &ContentType,
    ) -> Result<Vec<f32>> {
        if source_type == target_type {
            return Ok(source_embedding.to_vec());
        }

        match self.config.alignment_algorithm {
            AlignmentAlgorithm::LinearProjection => {
                self.linear_projection(source_embedding, source_type, target_type)
            }
            AlignmentAlgorithm::CCA => {
                self.cca_alignment(source_embedding, source_type, target_type)
            }
            AlignmentAlgorithm::DeepAlignment => {
                self.deep_alignment(source_embedding, source_type, target_type)
            }
            AlignmentAlgorithm::Attention => {
                self.attention_alignment(source_embedding, source_type, target_type)
            }
        }
    }

    /// 线性投影对齐
    fn linear_projection(
        &self,
        embedding: &[f32],
        source_type: &ContentType,
        target_type: &ContentType,
    ) -> Result<Vec<f32>> {
        // 获取或生成转换矩阵
        let key = (source_type.clone(), target_type.clone());

        if let Some(matrix) = self.transformation_matrices.get(&key) {
            // 应用转换矩阵
            self.apply_transformation(embedding, matrix)
        } else {
            // 如果没有预训练的转换矩阵，使用恒等映射
            Ok(embedding.to_vec())
        }
    }

    /// CCA 对齐
    fn cca_alignment(
        &self,
        embedding: &[f32],
        _source_type: &ContentType,
        _target_type: &ContentType,
    ) -> Result<Vec<f32>> {
        // CCA 对齐的简化实现
        // 在实际应用中，这需要预训练的 CCA 模型
        Ok(embedding.to_vec())
    }

    /// 深度对齐
    fn deep_alignment(
        &self,
        embedding: &[f32],
        _source_type: &ContentType,
        _target_type: &ContentType,
    ) -> Result<Vec<f32>> {
        // 深度对齐网络的简化实现
        // 在实际应用中，这需要预训练的神经网络
        Ok(embedding.to_vec())
    }

    /// 注意力对齐
    fn attention_alignment(
        &self,
        embedding: &[f32],
        _source_type: &ContentType,
        _target_type: &ContentType,
    ) -> Result<Vec<f32>> {
        // 注意力机制对齐的简化实现
        Ok(embedding.to_vec())
    }

    /// 应用转换矩阵
    fn apply_transformation(&self, embedding: &[f32], matrix: &[Vec<f32>]) -> Result<Vec<f32>> {
        if matrix.is_empty() || matrix[0].len() != embedding.len() {
            return Err(AgentMemError::ProcessingError(
                "Transformation matrix dimensions mismatch".to_string(),
            ));
        }

        let result: Vec<f32> = matrix
            .iter()
            .map(|row| row.iter().zip(embedding.iter()).map(|(a, b)| a * b).sum())
            .collect();

        Ok(result)
    }

    /// 学习转换矩阵
    ///
    /// # Arguments
    /// * `source_embeddings` - 源模态嵌入集合
    /// * `target_embeddings` - 目标模态嵌入集合
    /// * `source_type` - 源模态类型
    /// * `target_type` - 目标模态类型
    pub fn learn_transformation(
        &mut self,
        source_embeddings: &[Vec<f32>],
        target_embeddings: &[Vec<f32>],
        source_type: ContentType,
        target_type: ContentType,
    ) -> Result<()> {
        if source_embeddings.len() != target_embeddings.len() {
            return Err(AgentMemError::ValidationError(
                "Source and target embeddings must have the same length".to_string(),
            ));
        }

        // 简化的转换矩阵学习（实际应用中应使用更复杂的算法）
        let dim = source_embeddings[0].len();
        let matrix = vec![vec![1.0; dim]; dim];

        self.transformation_matrices
            .insert((source_type, target_type), matrix);

        Ok(())
    }
}

/// 模态间相似性计算器
pub struct ModalSimilarityCalculator {
    config: CrossModalConfig,
    aligner: CrossModalAligner,
}

impl ModalSimilarityCalculator {
    /// 创建新的相似性计算器
    pub fn new(config: CrossModalConfig) -> Self {
        let aligner = CrossModalAligner::new(config.clone());
        Self { config, aligner }
    }

    /// 计算跨模态相似性
    ///
    /// # Arguments
    /// * `embedding1` - 第一个嵌入
    /// * `type1` - 第一个模态类型
    /// * `embedding2` - 第二个嵌入
    /// * `type2` - 第二个模态类型
    ///
    /// # Returns
    /// * 相似性分数 (0.0 - 1.0)
    pub fn calculate_cross_modal_similarity(
        &self,
        embedding1: &[f32],
        type1: &ContentType,
        embedding2: &[f32],
        type2: &ContentType,
    ) -> Result<f32> {
        // 如果是同模态，直接计算余弦相似度
        if type1 == type2 {
            return self.cosine_similarity(embedding1, embedding2);
        }

        // 对齐嵌入到同一空间
        let aligned_embedding1 = self.aligner.align_embeddings(embedding1, type1, type2)?;

        // 计算对齐后的相似性
        let similarity = self.cosine_similarity(&aligned_embedding1, embedding2)?;

        // 应用模态权重调整
        let weight1 = self.get_modality_weight(type1);
        let weight2 = self.get_modality_weight(type2);
        let adjusted_similarity = similarity * (weight1 * weight2).sqrt();

        Ok(adjusted_similarity)
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

    /// 获取模态权重
    fn get_modality_weight(&self, content_type: &ContentType) -> f32 {
        let key = match content_type {
            ContentType::Text => "text",
            ContentType::Image => "image",
            ContentType::Audio => "audio",
            ContentType::Video => "video",
            ContentType::Document => "text",
            ContentType::Unknown => "text",
        };

        *self.config.modality_weights.get(key).unwrap_or(&1.0)
    }
}

/// 多模态嵌入项
#[derive(Debug, Clone)]
pub struct MultimodalEmbedding {
    /// 嵌入向量
    pub embedding: Vec<f32>,
    /// 模态类型
    pub modality: ContentType,
    /// 权重
    pub weight: f32,
    /// 置信度
    pub confidence: f32,
}

/// 多模态融合引擎
pub struct MultimodalFusionEngine {
    config: CrossModalConfig,
    aligner: CrossModalAligner,
}

impl MultimodalFusionEngine {
    /// 创建新的融合引擎
    pub fn new(config: CrossModalConfig) -> Self {
        let aligner = CrossModalAligner::new(config.clone());
        Self { config, aligner }
    }

    /// 融合多个模态的嵌入
    ///
    /// # Arguments
    /// * `embeddings` - 多模态嵌入列表
    ///
    /// # Returns
    /// * 融合后的嵌入向量
    pub fn fuse_embeddings(&self, embeddings: &[MultimodalEmbedding]) -> Result<Vec<f32>> {
        if embeddings.is_empty() {
            return Err(AgentMemError::ValidationError(
                "No embeddings to fuse".to_string(),
            ));
        }

        match self.config.fusion_strategy {
            FusionStrategy::WeightedAverage => self.weighted_average_fusion(embeddings),
            FusionStrategy::MaxPooling => self.max_pooling_fusion(embeddings),
            FusionStrategy::AttentionFusion => self.attention_fusion(embeddings),
            FusionStrategy::ConcatenateFusion => self.concatenate_fusion(embeddings),
        }
    }

    /// 加权平均融合
    fn weighted_average_fusion(&self, embeddings: &[MultimodalEmbedding]) -> Result<Vec<f32>> {
        // 对齐所有嵌入到同一空间（使用第一个模态作为目标空间）
        let target_type = &embeddings[0].modality;
        let dim = embeddings[0].embedding.len();

        let mut fused = vec![0.0; dim];
        let mut total_weight = 0.0;

        for emb in embeddings {
            let aligned =
                self.aligner
                    .align_embeddings(&emb.embedding, &emb.modality, target_type)?;

            let weight = emb.weight * emb.confidence;
            total_weight += weight;

            for (i, val) in aligned.iter().enumerate() {
                fused[i] += val * weight;
            }
        }

        // 归一化
        if total_weight > 0.0 {
            for val in &mut fused {
                *val /= total_weight;
            }
        }

        Ok(fused)
    }

    /// 最大池化融合
    fn max_pooling_fusion(&self, embeddings: &[MultimodalEmbedding]) -> Result<Vec<f32>> {
        let target_type = &embeddings[0].modality;
        let dim = embeddings[0].embedding.len();

        let mut fused = vec![f32::MIN; dim];

        for emb in embeddings {
            let aligned =
                self.aligner
                    .align_embeddings(&emb.embedding, &emb.modality, target_type)?;

            for (i, val) in aligned.iter().enumerate() {
                fused[i] = fused[i].max(*val);
            }
        }

        Ok(fused)
    }

    /// 注意力融合
    fn attention_fusion(&self, embeddings: &[MultimodalEmbedding]) -> Result<Vec<f32>> {
        // 计算注意力权重
        let attention_weights = self.calculate_attention_weights(embeddings)?;

        let target_type = &embeddings[0].modality;
        let dim = embeddings[0].embedding.len();

        let mut fused = vec![0.0; dim];

        for (emb, &attention_weight) in embeddings.iter().zip(attention_weights.iter()) {
            let aligned =
                self.aligner
                    .align_embeddings(&emb.embedding, &emb.modality, target_type)?;

            let combined_weight = attention_weight * emb.weight * emb.confidence;

            for (i, val) in aligned.iter().enumerate() {
                fused[i] += val * combined_weight;
            }
        }

        Ok(fused)
    }

    /// 级联融合
    fn concatenate_fusion(&self, embeddings: &[MultimodalEmbedding]) -> Result<Vec<f32>> {
        let target_type = &embeddings[0].modality;
        let mut fused = Vec::new();

        for emb in embeddings {
            let aligned =
                self.aligner
                    .align_embeddings(&emb.embedding, &emb.modality, target_type)?;
            fused.extend(aligned);
        }

        Ok(fused)
    }

    /// 计算注意力权重
    fn calculate_attention_weights(&self, embeddings: &[MultimodalEmbedding]) -> Result<Vec<f32>> {
        // 基于置信度和模态权重计算注意力
        let mut weights: Vec<f32> = embeddings
            .iter()
            .map(|emb| {
                let modality_weight = self.get_modality_weight(&emb.modality);
                emb.confidence * modality_weight
            })
            .collect();

        // Softmax 归一化
        let max_weight = weights.iter().cloned().fold(f32::MIN, f32::max);
        let exp_sum: f32 = weights.iter().map(|w| (w - max_weight).exp()).sum();

        for w in &mut weights {
            *w = (*w - max_weight).exp() / exp_sum;
        }

        Ok(weights)
    }

    /// 获取模态权重
    fn get_modality_weight(&self, content_type: &ContentType) -> f32 {
        let key = match content_type {
            ContentType::Text => "text",
            ContentType::Image => "image",
            ContentType::Audio => "audio",
            ContentType::Video => "video",
            ContentType::Document => "text",
            ContentType::Unknown => "text",
        };

        *self.config.modality_weights.get(key).unwrap_or(&1.0)
    }

    /// 自适应调整模态权重
    ///
    /// # Arguments
    /// * `embeddings` - 多模态嵌入列表
    /// * `query_embedding` - 查询嵌入
    ///
    /// # Returns
    /// * 调整后的嵌入列表
    pub fn adaptive_weight_adjustment(
        &self,
        embeddings: &[MultimodalEmbedding],
        query_embedding: &[f32],
    ) -> Result<Vec<MultimodalEmbedding>> {
        let mut adjusted = Vec::new();

        for emb in embeddings {
            // 计算与查询的相似度
            let similarity = self.cosine_similarity(&emb.embedding, query_embedding)?;

            // 基于相似度调整权重
            let adjusted_weight = emb.weight * (1.0 + similarity) / 2.0;

            adjusted.push(MultimodalEmbedding {
                embedding: emb.embedding.clone(),
                modality: emb.modality.clone(),
                weight: adjusted_weight,
                confidence: emb.confidence,
            });
        }

        Ok(adjusted)
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cross_modal_aligner() {
        let config = CrossModalConfig::default();
        let aligner = CrossModalAligner::new(config);

        let embedding = vec![1.0, 2.0, 3.0, 4.0];
        let result = aligner
            .align_embeddings(&embedding, &ContentType::Text, &ContentType::Image)
            .unwrap();

        assert_eq!(result.len(), embedding.len());
    }

    #[test]
    fn test_modal_similarity_calculator() {
        let config = CrossModalConfig::default();
        let calculator = ModalSimilarityCalculator::new(config);

        let emb1 = vec![1.0, 0.0, 0.0];
        let emb2 = vec![1.0, 0.0, 0.0];

        let similarity = calculator
            .calculate_cross_modal_similarity(&emb1, &ContentType::Text, &emb2, &ContentType::Text)
            .unwrap();

        assert!((similarity - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_multimodal_fusion() {
        let config = CrossModalConfig::default();
        let fusion_engine = MultimodalFusionEngine::new(config);

        let embeddings = vec![
            MultimodalEmbedding {
                embedding: vec![1.0, 0.0, 0.0],
                modality: ContentType::Text,
                weight: 1.0,
                confidence: 0.9,
            },
            MultimodalEmbedding {
                embedding: vec![0.0, 1.0, 0.0],
                modality: ContentType::Image,
                weight: 0.8,
                confidence: 0.85,
            },
        ];

        let fused = fusion_engine.fuse_embeddings(&embeddings).unwrap();
        assert_eq!(fused.len(), 3);
    }
}
