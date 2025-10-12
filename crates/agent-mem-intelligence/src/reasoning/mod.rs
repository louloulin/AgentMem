//! 记忆推理和关联分析模块

pub mod advanced;

pub use advanced::{
    AdvancedReasoner, AnalogyResult, CounterfactualResult, MultiHopCausalResult,
    CausalStep, CausalRelationType, AnalogicalDomain, AnalogicalMapping, MappingType,
};

use agent_mem_traits::{AgentMemError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// 推理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningResult {
    /// 推理类型
    pub reasoning_type: String,
    /// 置信度 (0.0 - 1.0)
    pub confidence: f32,
    /// 推理结论
    pub conclusion: String,
    /// 支持证据
    pub evidence: Vec<String>,
    /// 相关记忆ID
    pub related_memory_ids: Vec<String>,
    /// 推理时间
    pub reasoned_at: DateTime<Utc>,
}

/// 记忆关联
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAssociation {
    /// 源记忆ID
    pub source_memory_id: String,
    /// 目标记忆ID
    pub target_memory_id: String,
    /// 关联类型
    pub association_type: String,
    /// 关联强度 (0.0 - 1.0)
    pub strength: f32,
    /// 关联原因
    pub reason: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// 推理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningConfig {
    /// 最小置信度阈值
    pub min_confidence: f32,
    /// 最大推理深度
    pub max_depth: usize,
    /// 相似度阈值
    pub similarity_threshold: f32,
    /// 时间窗口（天）
    pub time_window_days: i64,
    /// 最大关联数量
    pub max_associations: usize,
}

impl Default for ReasoningConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.6,
            max_depth: 3,
            similarity_threshold: 0.7,
            time_window_days: 30,
            max_associations: 10,
        }
    }
}

/// 记忆推理器
pub struct MemoryReasoner {
    config: ReasoningConfig,
}

impl MemoryReasoner {
    /// 创建新的记忆推理器
    pub fn new(config: ReasoningConfig) -> Self {
        Self { config }
    }

    /// 使用默认配置创建
    pub fn default() -> Self {
        Self::new(ReasoningConfig::default())
    }

    /// 基于相似度的推理
    pub fn reason_by_similarity(
        &self,
        query_memory: &MemoryData,
        candidate_memories: &[MemoryData],
    ) -> Result<Vec<ReasoningResult>> {
        let mut results = Vec::new();

        if let Some(ref query_embedding) = query_memory.embedding {
            for candidate in candidate_memories {
                if candidate.id == query_memory.id {
                    continue;
                }

                if let Some(ref candidate_embedding) = candidate.embedding {
                    let similarity =
                        self.cosine_similarity(query_embedding, candidate_embedding)?;

                    if similarity >= self.config.similarity_threshold {
                        let confidence = similarity;
                        let conclusion = format!(
                            "Memory '{}' is highly similar to memory '{}'",
                            query_memory.id, candidate.id
                        );

                        let evidence = vec![
                            format!("Cosine similarity: {:.3}", similarity),
                            format!("Content overlap detected"),
                        ];

                        results.push(ReasoningResult {
                            reasoning_type: "similarity".to_string(),
                            confidence,
                            conclusion,
                            evidence,
                            related_memory_ids: vec![candidate.id.clone()],
                            reasoned_at: Utc::now(),
                        });
                    }
                }
            }
        }

        // 按置信度排序
        results.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(self.config.max_associations);

        Ok(results)
    }

    /// 基于时间的推理
    pub fn reason_by_temporal_patterns(
        &self,
        memories: &[MemoryData],
    ) -> Result<Vec<ReasoningResult>> {
        let mut results = Vec::new();
        let current_time = Utc::now();

        // 按时间排序
        let mut sorted_memories = memories.to_vec();
        sorted_memories.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        // 寻找时间模式
        for window_size in 2..=3 {
            for i in 0..=sorted_memories.len().saturating_sub(window_size) {
                let window = &sorted_memories[i..i + window_size];

                if let Some(pattern) = self.detect_temporal_pattern(window) {
                    let confidence = self.calculate_temporal_confidence(&pattern);

                    if confidence >= self.config.min_confidence {
                        let memory_ids: Vec<String> = window.iter().map(|m| m.id.clone()).collect();

                        results.push(ReasoningResult {
                            reasoning_type: "temporal".to_string(),
                            confidence,
                            conclusion: pattern.description,
                            evidence: pattern.evidence,
                            related_memory_ids: memory_ids,
                            reasoned_at: current_time,
                        });
                    }
                }
            }
        }

        Ok(results)
    }

    /// 基于内容的推理
    pub fn reason_by_content_analysis(
        &self,
        memories: &[MemoryData],
    ) -> Result<Vec<ReasoningResult>> {
        let mut results = Vec::new();

        // 提取关键词
        let memory_keywords: HashMap<String, HashSet<String>> = memories
            .iter()
            .map(|memory| {
                let keywords = self.extract_keywords(&memory.content);
                (memory.id.clone(), keywords)
            })
            .collect();

        // 寻找内容关联
        for (i, memory1) in memories.iter().enumerate() {
            for memory2 in memories.iter().skip(i + 1) {
                if let (Some(keywords1), Some(keywords2)) = (
                    memory_keywords.get(&memory1.id),
                    memory_keywords.get(&memory2.id),
                ) {
                    let overlap = self.calculate_keyword_overlap(keywords1, keywords2);

                    if overlap >= self.config.similarity_threshold {
                        let confidence = overlap;
                        let shared_keywords: Vec<String> =
                            keywords1.intersection(keywords2).cloned().collect();

                        let conclusion = format!(
                            "Memories '{}' and '{}' share common themes",
                            memory1.id, memory2.id
                        );

                        let evidence = vec![
                            format!("Keyword overlap: {:.3}", overlap),
                            format!("Shared keywords: {}", shared_keywords.join(", ")),
                        ];

                        results.push(ReasoningResult {
                            reasoning_type: "content".to_string(),
                            confidence,
                            conclusion,
                            evidence,
                            related_memory_ids: vec![memory1.id.clone(), memory2.id.clone()],
                            reasoned_at: Utc::now(),
                        });
                    }
                }
            }
        }

        Ok(results)
    }

    /// 发现记忆关联
    pub fn discover_associations(&self, memories: &[MemoryData]) -> Result<Vec<MemoryAssociation>> {
        let mut associations = Vec::new();

        // 基于相似度的关联
        let similarity_results = self.reason_by_similarity_all(memories)?;
        for result in similarity_results {
            if result.related_memory_ids.len() >= 2 {
                associations.push(MemoryAssociation {
                    source_memory_id: result.related_memory_ids[0].clone(),
                    target_memory_id: result.related_memory_ids[1].clone(),
                    association_type: "similarity".to_string(),
                    strength: result.confidence,
                    reason: result.conclusion,
                    created_at: Utc::now(),
                });
            }
        }

        // 基于内容的关联
        let content_results = self.reason_by_content_analysis(memories)?;
        for result in content_results {
            if result.related_memory_ids.len() >= 2 {
                associations.push(MemoryAssociation {
                    source_memory_id: result.related_memory_ids[0].clone(),
                    target_memory_id: result.related_memory_ids[1].clone(),
                    association_type: "content".to_string(),
                    strength: result.confidence,
                    reason: result.conclusion,
                    created_at: Utc::now(),
                });
            }
        }

        Ok(associations)
    }

    /// 为所有记忆进行相似度推理
    fn reason_by_similarity_all(&self, memories: &[MemoryData]) -> Result<Vec<ReasoningResult>> {
        let mut all_results = Vec::new();

        for memory in memories {
            let results = self.reason_by_similarity(memory, memories)?;
            all_results.extend(results);
        }

        Ok(all_results)
    }

    /// 检测时间模式
    fn detect_temporal_pattern(&self, window: &[MemoryData]) -> Option<TemporalPattern> {
        if window.len() < 2 {
            return None;
        }

        // 计算时间间隔
        let mut intervals = Vec::new();
        for i in 1..window.len() {
            let interval = window[i]
                .created_at
                .signed_duration_since(window[i - 1].created_at);
            intervals.push(interval.num_hours());
        }

        // 检查是否有规律性
        let avg_interval = intervals.iter().sum::<i64>() as f64 / intervals.len() as f64;
        let variance = intervals
            .iter()
            .map(|&x| (x as f64 - avg_interval).powi(2))
            .sum::<f64>()
            / intervals.len() as f64;

        if variance < 100.0 {
            // 低方差表示规律性
            Some(TemporalPattern {
                description: format!(
                    "Regular pattern with average interval of {:.1} hours",
                    avg_interval
                ),
                evidence: vec![
                    format!("Average interval: {:.1} hours", avg_interval),
                    format!("Variance: {:.2}", variance),
                ],
            })
        } else {
            None
        }
    }

    /// 计算时间模式置信度
    fn calculate_temporal_confidence(&self, pattern: &TemporalPattern) -> f32 {
        // 简化的置信度计算
        if pattern.description.contains("Regular") {
            0.8
        } else {
            0.5
        }
    }

    /// 提取关键词
    fn extract_keywords(&self, content: &str) -> HashSet<String> {
        content
            .to_lowercase()
            .split_whitespace()
            .filter(|word| word.len() > 3) // 过滤短词
            .map(|word| {
                word.trim_matches(|c: char| !c.is_alphanumeric())
                    .to_string()
            })
            .filter(|word| !word.is_empty())
            .collect()
    }

    /// 计算关键词重叠度
    fn calculate_keyword_overlap(
        &self,
        keywords1: &HashSet<String>,
        keywords2: &HashSet<String>,
    ) -> f32 {
        let intersection_size = keywords1.intersection(keywords2).count();
        let union_size = keywords1.union(keywords2).count();

        if union_size == 0 {
            0.0
        } else {
            intersection_size as f32 / union_size as f32
        }
    }

    /// 计算余弦相似度
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> Result<f32> {
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

    /// 更新配置
    pub fn update_config(&mut self, config: ReasoningConfig) {
        self.config = config;
    }

    /// 获取当前配置
    pub fn get_config(&self) -> &ReasoningConfig {
        &self.config
    }
}

/// 时间模式
#[derive(Debug, Clone)]
struct TemporalPattern {
    description: String,
    evidence: Vec<String>,
}

/// 记忆数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryData {
    pub id: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub embedding: Option<Vec<f32>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_memory(id: &str, content: &str, embedding: Vec<f32>) -> MemoryData {
        MemoryData {
            id: id.to_string(),
            content: content.to_string(),
            created_at: Utc::now(),
            embedding: Some(embedding),
        }
    }

    #[test]
    fn test_memory_reasoner_creation() {
        let reasoner = MemoryReasoner::default();
        assert_eq!(reasoner.config.min_confidence, 0.6);
        assert_eq!(reasoner.config.max_depth, 3);
    }

    #[test]
    fn test_reason_by_similarity() {
        let reasoner = MemoryReasoner::default();

        let query_memory = create_test_memory("1", "test content", vec![1.0, 0.0, 0.0]);
        let candidates = vec![
            create_test_memory("2", "similar content", vec![0.9, 0.1, 0.0]),
            create_test_memory("3", "different content", vec![0.0, 0.0, 1.0]),
        ];

        let results = reasoner
            .reason_by_similarity(&query_memory, &candidates)
            .unwrap();
        assert!(!results.is_empty());

        for result in &results {
            assert_eq!(result.reasoning_type, "similarity");
            assert!(result.confidence >= reasoner.config.similarity_threshold);
        }
    }

    #[test]
    fn test_extract_keywords() {
        let reasoner = MemoryReasoner::default();
        let content = "This is a test content with some important keywords";
        let keywords = reasoner.extract_keywords(content);

        assert!(keywords.contains("test"));
        assert!(keywords.contains("content"));
        assert!(keywords.contains("important"));
        assert!(keywords.contains("keywords"));
        assert!(!keywords.contains("is")); // 短词应该被过滤
    }

    #[test]
    fn test_calculate_keyword_overlap() {
        let reasoner = MemoryReasoner::default();

        let keywords1: HashSet<String> = ["hello", "world", "test"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let keywords2: HashSet<String> = ["hello", "world", "example"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        let overlap = reasoner.calculate_keyword_overlap(&keywords1, &keywords2);
        assert!(overlap > 0.0);
        assert!(overlap < 1.0);
    }

    #[test]
    fn test_reason_by_content_analysis() {
        let reasoner = MemoryReasoner::default();

        let memories = vec![
            create_test_memory("1", "machine learning algorithms", vec![1.0, 0.0]),
            create_test_memory("2", "deep learning neural networks", vec![0.0, 1.0]),
            create_test_memory("3", "cooking recipes", vec![0.5, 0.5]),
        ];

        let results = reasoner.reason_by_content_analysis(&memories).unwrap();

        for result in &results {
            assert_eq!(result.reasoning_type, "content");
            assert!(result.confidence >= 0.0);
            assert!(result.confidence <= 1.0);
        }
    }

    #[test]
    fn test_discover_associations() {
        let reasoner = MemoryReasoner::default();

        let memories = vec![
            create_test_memory(
                "1",
                "artificial intelligence machine learning",
                vec![1.0, 0.0],
            ),
            create_test_memory("2", "machine learning deep learning", vec![0.9, 0.1]),
        ];

        let associations = reasoner.discover_associations(&memories).unwrap();

        for association in &associations {
            assert!(!association.source_memory_id.is_empty());
            assert!(!association.target_memory_id.is_empty());
            assert!(association.strength >= 0.0);
            assert!(association.strength <= 1.0);
        }
    }

    #[test]
    fn test_cosine_similarity() {
        let reasoner = MemoryReasoner::default();

        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let similarity = reasoner.cosine_similarity(&a, &b).unwrap();
        assert!((similarity - 1.0).abs() < 1e-6);

        let c = vec![0.0, 1.0, 0.0];
        let similarity2 = reasoner.cosine_similarity(&a, &c).unwrap();
        assert!((similarity2 - 0.0).abs() < 1e-6);
    }
}
