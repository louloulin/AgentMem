//! 混合相似度计算实现

use super::{SemanticSimilarity, TextualSimilarity};
use agent_mem_traits::{AgentMemError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 混合相似度结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSimilarityResult {
    /// 最终相似度分数 (0.0 - 1.0)
    pub similarity: f32,
    /// 语义相似度分数
    pub semantic_similarity: f32,
    /// 文本相似度分数
    pub textual_similarity: f32,
    /// 各组件权重
    pub weights: HashMap<String, f32>,
    /// 是否相似
    pub is_similar: bool,
    /// 阈值
    pub threshold: f32,
}

/// 混合相似度配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSimilarityConfig {
    /// 语义相似度权重
    pub semantic_weight: f32,
    /// 文本相似度权重
    pub textual_weight: f32,
    /// 相似度阈值
    pub threshold: f32,
    /// 是否启用自适应权重
    pub adaptive_weights: bool,
}

impl Default for HybridSimilarityConfig {
    fn default() -> Self {
        Self {
            semantic_weight: 0.7,
            textual_weight: 0.3,
            threshold: 0.6,
            adaptive_weights: false,
        }
    }
}

/// 混合相似度计算器
pub struct HybridSimilarity {
    config: HybridSimilarityConfig,
    semantic_similarity: SemanticSimilarity,
    textual_similarity: TextualSimilarity,
}

impl HybridSimilarity {
    /// 创建新的混合相似度计算器
    pub fn new(
        config: HybridSimilarityConfig,
        semantic_similarity: SemanticSimilarity,
        textual_similarity: TextualSimilarity,
    ) -> Self {
        Self {
            config,
            semantic_similarity,
            textual_similarity,
        }
    }

    /// 使用默认配置创建
    pub fn default() -> Self {
        Self::new(
            HybridSimilarityConfig::default(),
            SemanticSimilarity::default(),
            TextualSimilarity::default(),
        )
    }

    /// 计算混合相似度
    pub fn calculate_similarity(
        &self,
        text1: &str,
        text2: &str,
        vector1: &[f32],
        vector2: &[f32],
    ) -> Result<HybridSimilarityResult> {
        // 计算语义相似度
        let semantic_result = self
            .semantic_similarity
            .detect_similarity(vector1, vector2)?;

        // 计算文本相似度
        let textual_result = self.textual_similarity.calculate_similarity(text1, text2)?;

        // 计算权重（自适应或固定）
        let (semantic_weight, textual_weight) = if self.config.adaptive_weights {
            self.calculate_adaptive_weights(&semantic_result.similarity, &textual_result.similarity)
        } else {
            (self.config.semantic_weight, self.config.textual_weight)
        };

        // 计算加权平均相似度
        let final_similarity = semantic_result.similarity * semantic_weight
            + textual_result.similarity * textual_weight;

        // 构建权重映射
        let mut weights = HashMap::new();
        weights.insert("semantic".to_string(), semantic_weight);
        weights.insert("textual".to_string(), textual_weight);

        Ok(HybridSimilarityResult {
            similarity: final_similarity,
            semantic_similarity: semantic_result.similarity,
            textual_similarity: textual_result.similarity,
            weights,
            is_similar: final_similarity >= self.config.threshold,
            threshold: self.config.threshold,
        })
    }

    /// 计算自适应权重
    fn calculate_adaptive_weights(&self, semantic_sim: &f32, textual_sim: &f32) -> (f32, f32) {
        // 基于相似度分数的置信度调整权重
        let semantic_confidence = self.calculate_confidence(*semantic_sim);
        let textual_confidence = self.calculate_confidence(*textual_sim);

        let total_confidence = semantic_confidence + textual_confidence;

        if total_confidence == 0.0 {
            return (self.config.semantic_weight, self.config.textual_weight);
        }

        let semantic_weight = semantic_confidence / total_confidence;
        let textual_weight = textual_confidence / total_confidence;

        (semantic_weight, textual_weight)
    }

    /// 计算置信度分数
    fn calculate_confidence(&self, similarity: f32) -> f32 {
        // 使用sigmoid函数将相似度转换为置信度
        // 相似度越高，置信度越高
        1.0 / (1.0 + (-5.0 * (similarity - 0.5)).exp())
    }

    /// 批量混合相似度计算
    pub fn batch_similarity(
        &self,
        query_text: &str,
        query_vector: &[f32],
        texts: &[String],
        vectors: &[Vec<f32>],
    ) -> Result<Vec<HybridSimilarityResult>> {
        if texts.len() != vectors.len() {
            return Err(AgentMemError::validation_error(
                "Texts and vectors must have the same length",
            ));
        }

        let mut results = Vec::new();

        for (text, vector) in texts.iter().zip(vectors.iter()) {
            let result = self.calculate_similarity(query_text, text, query_vector, vector)?;
            results.push(result);
        }

        Ok(results)
    }

    /// 找到最相似的项目
    pub fn find_most_similar(
        &self,
        query_text: &str,
        query_vector: &[f32],
        texts: &[String],
        vectors: &[Vec<f32>],
    ) -> Result<Option<(usize, HybridSimilarityResult)>> {
        if texts.is_empty() || vectors.is_empty() {
            return Ok(None);
        }

        let results = self.batch_similarity(query_text, query_vector, texts, vectors)?;

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

    /// 找到前K个最相似的项目
    pub fn find_top_k_similar(
        &self,
        query_text: &str,
        query_vector: &[f32],
        texts: &[String],
        vectors: &[Vec<f32>],
        k: usize,
    ) -> Result<Vec<(usize, HybridSimilarityResult)>> {
        if texts.is_empty() || vectors.is_empty() {
            return Ok(Vec::new());
        }

        let results = self.batch_similarity(query_text, query_vector, texts, vectors)?;

        let mut indexed_results: Vec<(usize, HybridSimilarityResult)> =
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

    /// 分析相似度组件贡献
    pub fn analyze_similarity_components(
        &self,
        text1: &str,
        text2: &str,
        vector1: &[f32],
        vector2: &[f32],
    ) -> Result<SimilarityAnalysis> {
        let result = self.calculate_similarity(text1, text2, vector1, vector2)?;

        let semantic_contribution = result.semantic_similarity * result.weights["semantic"];
        let textual_contribution = result.textual_similarity * result.weights["textual"];

        Ok(SimilarityAnalysis {
            final_similarity: result.similarity,
            semantic_similarity: result.semantic_similarity,
            textual_similarity: result.textual_similarity,
            semantic_contribution,
            textual_contribution,
            semantic_weight: result.weights["semantic"],
            textual_weight: result.weights["textual"],
            dominant_component: if semantic_contribution > textual_contribution {
                "semantic".to_string()
            } else {
                "textual".to_string()
            },
        })
    }

    /// 更新配置
    pub fn update_config(&mut self, config: HybridSimilarityConfig) {
        self.config = config;
    }

    /// 获取当前配置
    pub fn get_config(&self) -> &HybridSimilarityConfig {
        &self.config
    }
}

/// 相似度分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityAnalysis {
    pub final_similarity: f32,
    pub semantic_similarity: f32,
    pub textual_similarity: f32,
    pub semantic_contribution: f32,
    pub textual_contribution: f32,
    pub semantic_weight: f32,
    pub textual_weight: f32,
    pub dominant_component: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hybrid_similarity_creation() {
        let hybrid = HybridSimilarity::default();
        assert_eq!(hybrid.config.semantic_weight, 0.7);
        assert_eq!(hybrid.config.textual_weight, 0.3);
        assert_eq!(hybrid.config.threshold, 0.6);
    }

    #[test]
    fn test_calculate_similarity() {
        let hybrid = HybridSimilarity::default();

        let text1 = "hello world";
        let text2 = "hello world";
        let vector1 = vec![1.0, 0.0, 0.0];
        let vector2 = vec![1.0, 0.0, 0.0];

        let result = hybrid
            .calculate_similarity(text1, text2, &vector1, &vector2)
            .unwrap();
        assert!(result.similarity > 0.8);
        assert!(result.is_similar);
        assert_eq!(result.weights.len(), 2);
    }

    #[test]
    fn test_adaptive_weights() {
        let mut config = HybridSimilarityConfig::default();
        config.adaptive_weights = true;

        let hybrid = HybridSimilarity::new(
            config,
            SemanticSimilarity::default(),
            TextualSimilarity::default(),
        );

        let text1 = "hello world";
        let text2 = "hello world";
        let vector1 = vec![1.0, 0.0, 0.0];
        let vector2 = vec![1.0, 0.0, 0.0];

        let result = hybrid
            .calculate_similarity(text1, text2, &vector1, &vector2)
            .unwrap();
        assert!(result.similarity > 0.0);

        // 权重应该根据置信度调整
        let semantic_weight = result.weights["semantic"];
        let textual_weight = result.weights["textual"];
        assert!((semantic_weight + textual_weight - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_calculate_confidence() {
        let hybrid = HybridSimilarity::default();

        let high_sim_confidence = hybrid.calculate_confidence(0.9);
        let low_sim_confidence = hybrid.calculate_confidence(0.1);

        assert!(high_sim_confidence > low_sim_confidence);
        assert!(high_sim_confidence > 0.5);
        assert!(low_sim_confidence < 0.5);
    }

    #[test]
    fn test_batch_similarity() {
        let hybrid = HybridSimilarity::default();

        let query_text = "hello world";
        let query_vector = vec![1.0, 0.0];
        let texts = vec!["hello world".to_string(), "goodbye world".to_string()];
        let vectors = vec![vec![1.0, 0.0], vec![0.0, 1.0]];

        let results = hybrid
            .batch_similarity(query_text, &query_vector, &texts, &vectors)
            .unwrap();
        assert_eq!(results.len(), 2);
        assert!(results[0].similarity > results[1].similarity);
    }

    #[test]
    fn test_find_most_similar() {
        let hybrid = HybridSimilarity::default();

        let query_text = "hello world";
        let query_vector = vec![1.0, 0.0];
        let texts = vec!["goodbye world".to_string(), "hello world".to_string()];
        let vectors = vec![vec![0.0, 1.0], vec![1.0, 0.0]];

        let result = hybrid
            .find_most_similar(query_text, &query_vector, &texts, &vectors)
            .unwrap();
        assert!(result.is_some());
        let (index, _) = result.unwrap();
        assert_eq!(index, 1);
    }

    #[test]
    fn test_analyze_similarity_components() {
        let hybrid = HybridSimilarity::default();

        let text1 = "hello world";
        let text2 = "hello world";
        let vector1 = vec![1.0, 0.0, 0.0];
        let vector2 = vec![1.0, 0.0, 0.0];

        let analysis = hybrid
            .analyze_similarity_components(text1, text2, &vector1, &vector2)
            .unwrap();
        assert!(analysis.final_similarity > 0.8);
        assert!(analysis.semantic_contribution > 0.0);
        assert!(analysis.textual_contribution > 0.0);
        assert!(!analysis.dominant_component.is_empty());
    }

    #[test]
    fn test_dimension_mismatch() {
        let hybrid = HybridSimilarity::default();

        let text1 = "hello";
        let text2 = "world";
        let vector1 = vec![1.0, 0.0];
        let vectors = vec![vec![1.0, 0.0, 0.0]]; // 不同维度
        let texts = vec!["world".to_string()];

        let result = hybrid.batch_similarity(text1, &vector1, &texts, &vectors);
        assert!(result.is_err());
    }
}
