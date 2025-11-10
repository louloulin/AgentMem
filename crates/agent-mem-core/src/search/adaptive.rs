//! 自适应搜索优化
//!
//! 提供基于查询特征的自适应搜索权重调整

use super::SearchQuery;
use super::SearchResult;
use agent_mem_traits::Result;
use agent_mem_config::agentmem_config::{AgentMemConfig, SearchConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// 查询特征
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryFeatures {
    /// 是否包含精确匹配词（如引号、特殊符号）
    pub has_exact_terms: bool,

    /// 语义复杂度 (0.0-1.0)
    pub semantic_complexity: f32,

    /// 是否有时间指示词
    pub has_temporal_indicator: bool,

    /// 实体数量
    pub entity_count: usize,

    /// 查询长度
    pub query_length: usize,

    /// 是否是问句
    pub is_question: bool,
}

impl QueryFeatures {
    /// 从查询文本中提取特征
    pub fn extract_from_query(query: &str) -> Self {
        let query_lower = query.to_lowercase();

        // 检测精确匹配词
        let has_exact_terms = query.contains('"')
            || query.contains('\'')
            || query.contains('@')
            || query.contains('#');

        // 检测时间指示词
        let temporal_keywords = [
            "yesterday",
            "today",
            "tomorrow",
            "last week",
            "last month",
            "recently",
            "昨天",
            "今天",
            "最近",
        ];
        let has_temporal_indicator = temporal_keywords
            .iter()
            .any(|&keyword| query_lower.contains(keyword));

        // 检测问句
        let is_question = query.ends_with('?')
            || query_lower.starts_with("what")
            || query_lower.starts_with("how")
            || query_lower.starts_with("why")
            || query_lower.starts_with("when")
            || query_lower.starts_with("where")
            || query_lower.starts_with("谁")
            || query_lower.starts_with("什么")
            || query_lower.starts_with("怎么")
            || query_lower.starts_with("为什么");

        // 计算语义复杂度（基于句子结构和长度）
        let word_count = query.split_whitespace().count();
        let semantic_complexity = if word_count <= 3 {
            0.2 // 简单查询
        } else if word_count <= 10 {
            0.5 // 中等复杂度
        } else {
            0.8 // 高复杂度
        };

        // 简单的实体计数（大写词、@提及等）
        let entity_count = query
            .split_whitespace()
            .filter(|word| word.chars().next().map_or(false, |c| c.is_uppercase()))
            .count();

        Self {
            has_exact_terms,
            semantic_complexity,
            has_temporal_indicator,
            entity_count,
            query_length: query.len(),
            is_question,
        }
    }
}

/// 搜索权重
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchWeights {
    /// 向量搜索权重
    pub vector_weight: f32,

    /// 全文搜索权重
    pub fulltext_weight: f32,

    /// 置信度
    pub confidence: f32,
}

impl SearchWeights {
    /// 归一化权重
    pub fn normalize(&mut self) {
        let total = self.vector_weight + self.fulltext_weight;
        if total > 0.0 {
            self.vector_weight /= total;
            self.fulltext_weight /= total;
        }
    }
}

/// 权重预测器
pub struct WeightPredictor {
    /// 学习到的权重映射
    learned_weights: HashMap<String, SearchWeights>,
    /// 配置（从配置文件加载，非硬编码）
    config: Arc<SearchConfig>,
}

impl WeightPredictor {
    /// 创建新的权重预测器（使用配置）
    pub fn new(config: Arc<SearchConfig>) -> Self {
        Self {
            learned_weights: HashMap::new(),
            config,
        }
    }

    /// 基于查询特征预测最优权重（使用配置值，非硬编码）
    pub fn predict(&self, features: &QueryFeatures) -> SearchWeights {
        let mut vector_weight = self.config.vector_weight;
        let mut fulltext_weight = self.config.fulltext_weight;
        let mut confidence = self.config.confidence_base;

        // 规则1: 精确匹配查询 → 提高全文权重
        if features.has_exact_terms {
            fulltext_weight += self.config.keyword_match_boost;
            confidence += self.config.confidence_boost;
        }

        // 规则2: 语义复杂查询 → 提高向量权重
        if features.semantic_complexity > 0.6 {
            vector_weight += self.config.semantic_complexity_boost * features.semantic_complexity;
            confidence += self.config.confidence_boost * 0.5;
        }

        // 规则3: 简单查询 → 提高全文权重
        if features.query_length < 20 && !features.is_question {
            fulltext_weight += self.config.exact_match_boost;
        }

        // 规则4: 问句查询 → 提高向量权重（语义理解更重要）
        if features.is_question {
            vector_weight += self.config.exact_match_boost;
            confidence += self.config.confidence_boost * 0.5;
        }

        // 规则5: 包含实体 → 平衡权重
        if features.entity_count > 0 {
            let entity_boost = (features.entity_count as f32 * 0.05).min(self.config.exact_match_boost);
            fulltext_weight += entity_boost;
        }

        // 规则6: 时间相关查询 → 稍微提高向量权重（时序推理）
        if features.has_temporal_indicator {
            vector_weight += self.config.confidence_boost;
        }

        let mut weights = SearchWeights {
            vector_weight,
            fulltext_weight,
            confidence: confidence.min(1.0_f32),
        };

        weights.normalize();
        weights
    }

    /// 从历史数据学习权重（简化版）
    pub fn learn_from_feedback(&mut self, query: &str, weights: SearchWeights, effectiveness: f32) {
        if effectiveness > 0.7 {
            // 只记录有效的权重配置
            self.learned_weights.insert(query.to_string(), weights);
        }
    }
}

impl Default for WeightPredictor {
    fn default() -> Self {
        let config = Arc::new(AgentMemConfig::default().search);
        Self::new(config)
    }
}

/// 自适应搜索优化器
pub struct AdaptiveSearchOptimizer {
    /// 权重预测器
    weight_predictor: WeightPredictor,

    /// 是否启用学习（从配置读取）
    enable_learning: bool,
    
    /// 配置
    config: Arc<SearchConfig>,
}

impl AdaptiveSearchOptimizer {
    /// 创建新的自适应搜索优化器（使用配置）
    pub fn new(config: Arc<SearchConfig>) -> Self {
        let enable_learning = config.adaptive_learning;
        Self {
            weight_predictor: WeightPredictor::new(config.clone()),
            enable_learning,
            config,
        }
    }
    
    /// 创建新的自适应搜索优化器（向后兼容）
    pub fn new_with_learning(enable_learning: bool) -> Self {
        let config = Arc::new(AgentMemConfig::default().search);
        Self {
            weight_predictor: WeightPredictor::new(config.clone()),
            enable_learning,
            config,
        }
    }

    /// 优化搜索查询
    pub fn optimize_query(&self, query: &SearchQuery) -> (SearchQuery, SearchWeights) {
        // 提取查询特征
        let features = QueryFeatures::extract_from_query(&query.query);

        // 预测最优权重
        let weights = self.weight_predictor.predict(&features);

        // 创建优化后的查询（保持原查询不变，只返回权重）
        (query.clone(), weights)
    }

    /// 记录搜索反馈（用于持续学习）
    pub fn record_feedback(&mut self, query: &str, weights: SearchWeights, effectiveness: f32) {
        if self.enable_learning {
            self.weight_predictor
                .learn_from_feedback(query, weights, effectiveness);
        }
    }
}

impl Default for AdaptiveSearchOptimizer {
    fn default() -> Self {
        let config = Arc::new(AgentMemConfig::default().search);
        Self::new(config)
    }
}

/// 搜索结果重排序器
pub struct SearchReranker;

impl SearchReranker {
    /// 创建新的重排序器
    pub fn new() -> Self {
        Self
    }

    /// 重排序搜索结果（基于多因素）
    pub fn rerank(&self, mut results: Vec<SearchResult>, _query: &SearchQuery) -> Vec<SearchResult> {
        // 为每个结果计算综合得分
        for result in &mut results {
            let mut final_score = result.score;

            // 因素1: 时间衰减（如果有时间戳）
            if let Some(metadata) = &result.metadata {
                if let Some(timestamp) = metadata.get("created_at") {
                    if let Some(created_at) = timestamp.as_i64() {
                        let now = chrono::Utc::now().timestamp();
                        let age_days = (now - created_at) / 86400;

                        // 时间衰减因子：越新的记忆得分越高
                        let time_decay = (1.0 / (1.0 + age_days as f32 / 30.0)).max(0.5);
                        final_score *= time_decay;
                    }
                }

                // 因素2: 重要性加权
                if let Some(importance) = metadata.get("importance") {
                    if let Some(importance_score) = importance.as_f64() {
                        final_score *= 1.0 + (importance_score as f32 * 0.2);
                    }
                }
            }

            // 因素3: 内容长度惩罚（太短或太长的结果略微降权）
            let content_len = result.content.len();
            let length_penalty = if content_len < 20 {
                0.9 // 太短
            } else if content_len > 1000 {
                0.95 // 太长
            } else {
                1.0 // 适中
            };
            final_score *= length_penalty;

            result.score = final_score;
        }

        // 重新排序
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }
}

impl Default for SearchReranker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_feature_extraction() {
        // 精确匹配查询
        let features1 = QueryFeatures::extract_from_query("user@example.com");
        assert!(features1.has_exact_terms);

        // 时间查询
        let features2 = QueryFeatures::extract_from_query("what happened yesterday?");
        assert!(features2.has_temporal_indicator);
        assert!(features2.is_question);

        // 复杂语义查询
        let features3 = QueryFeatures::extract_from_query(
            "How can I improve the performance of vector search in large datasets?",
        );
        assert!(features3.semantic_complexity > 0.5);
        assert!(features3.is_question);
    }

    #[test]
    fn test_weight_prediction() {
        let predictor = WeightPredictor::new();

        // 精确匹配查询应该提高全文权重
        let features1 = QueryFeatures {
            has_exact_terms: true,
            semantic_complexity: 0.3,
            has_temporal_indicator: false,
            entity_count: 0,
            query_length: 20,
            is_question: false,
        };
        let weights1 = predictor.predict(&features1);
        assert!(weights1.fulltext_weight > weights1.vector_weight);

        // 语义复杂查询应该提高向量权重
        let features2 = QueryFeatures {
            has_exact_terms: false,
            semantic_complexity: 0.8,
            has_temporal_indicator: false,
            entity_count: 0,
            query_length: 100,
            is_question: true,
        };
        let weights2 = predictor.predict(&features2);
        assert!(weights2.vector_weight > weights2.fulltext_weight);
    }

    #[test]
    fn test_weight_normalization() {
        let mut weights = SearchWeights {
            vector_weight: 0.7,
            fulltext_weight: 0.5,
            confidence: 0.8,
        };

        weights.normalize();

        assert!((weights.vector_weight + weights.fulltext_weight - 1.0).abs() < 0.001);
    }
}
