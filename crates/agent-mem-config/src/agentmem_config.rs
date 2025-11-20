//! AgentMem V4.0 统一配置加载器
//! 消除所有硬编码常量

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// AgentMem完整配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMemConfig {
    pub system: SystemConfig,
    pub search: SearchConfig,
    pub threshold: ThresholdConfig,
    pub importance: ImportanceConfig,
    pub decision: DecisionConfig,
    pub relation: RelationConfig,
    pub context: ContextConfig,
    pub performance: PerformanceConfig,
    pub adaptive: AdaptiveConfig,
    pub embedding: EmbeddingConfig,
    pub hierarchy: HierarchyConfig,
    pub llm: LlmConfig,
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub version: String,
    pub environment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    pub vector_weight: f32,
    pub fulltext_weight: f32,
    pub bm25_weight: f32,
    pub semantic_complexity_boost: f32,
    pub keyword_match_boost: f32,
    pub exact_match_boost: f32,
    pub confidence_base: f32,
    pub confidence_boost: f32,
    pub rrf_k: u32,
    pub default_threshold: f32,
    pub min_threshold: f32,
    pub max_threshold: f32,
    pub adaptive_learning: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdConfig {
    pub query_complexity_low_adjustment: f32,
    pub query_complexity_high_adjustment: f32,
    pub result_count_low_adjustment: f32,
    pub result_count_high_adjustment: f32,
    pub baseline_threshold: f32,
    pub has_keywords_boost: f32,
    pub has_filters_boost: f32,
    pub high_complexity_boost: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportanceConfig {
    pub recency_weight: f32,
    pub frequency_weight: f32,
    pub relevance_weight: f32,
    pub emotional_weight: f32,
    pub context_weight: f32,
    pub interaction_weight: f32,
    pub low_threshold: f32,
    pub medium_threshold: f32,
    pub high_threshold: f32,
    pub enable_dynamic_weights: bool,
    pub learning_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionConfig {
    pub importance_threshold: f32,
    pub conflict_threshold: f32,
    pub merge_similarity: f32,
    pub confidence_min: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationConfig {
    pub min_relation_strength: f32,
    pub temporal_window_secs: u64,
    pub decay_halflife_days: f32,
    pub semantic_similarity_weight: f32,
    pub temporal_proximity_weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    pub base_confidence: f32,
    pub keyword_boost_per_keyword: f32,
    pub temporal_relevance_boost: f32,
    pub spatial_relevance_boost: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub max_concurrent_searches: usize,
    pub cache_ttl_seconds: u64,
    pub batch_size: usize,
    pub processing_interval_seconds: u64,
    pub max_batch_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveConfig {
    pub enable_bandit: bool,
    pub exploration_rate: f32,
    pub decay_factor: f32,
    pub alpha_prior: f32,
    pub beta_prior: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub default_dimension: usize,
    pub similarity_threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchyConfig {
    pub enable_inheritance: bool,
    pub decay_factor: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub default_temperature: f32,
    pub max_tokens: usize,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub enable_compression: bool,
    pub enable_deduplication: bool,
    pub dedup_similarity_threshold: f32,
}

impl AgentMemConfig {
    /// 从文件加载配置
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    /// 从默认路径加载配置
    pub fn load_default() -> Result<Self, Box<dyn std::error::Error>> {
        // 尝试多个路径
        let paths = vec![
            "config/agentmem.toml",
            "../config/agentmem.toml",
            "../../config/agentmem.toml",
        ];

        for path in paths {
            if Path::new(path).exists() {
                return Self::from_file(path);
            }
        }

        // 如果都找不到，使用默认配置
        Ok(Self::default())
    }

    /// 获取默认配置
    pub fn default() -> Self {
        Self {
            system: SystemConfig {
                version: "4.0.0".to_string(),
                environment: "production".to_string(),
            },
            search: SearchConfig {
                vector_weight: 0.5,
                fulltext_weight: 0.5,
                bm25_weight: 0.0,
                semantic_complexity_boost: 0.2,
                keyword_match_boost: 0.25,
                exact_match_boost: 0.15,
                confidence_base: 0.7,
                confidence_boost: 0.1,
                rrf_k: 60,
                default_threshold: 0.3,
                min_threshold: 0.0,
                max_threshold: 0.9,
                adaptive_learning: true,
            },
            threshold: ThresholdConfig {
                query_complexity_low_adjustment: -0.2,
                query_complexity_high_adjustment: -0.15,
                result_count_low_adjustment: -0.2,
                result_count_high_adjustment: -0.15,
                baseline_threshold: 0.4,
                has_keywords_boost: 0.2,
                has_filters_boost: 0.2,
                high_complexity_boost: 0.2,
            },
            importance: ImportanceConfig {
                recency_weight: 0.25,
                frequency_weight: 0.20,
                relevance_weight: 0.25,
                emotional_weight: 0.15,
                context_weight: 0.10,
                interaction_weight: 0.05,
                low_threshold: 0.4,
                medium_threshold: 0.6,
                high_threshold: 0.8,
                enable_dynamic_weights: true,
                learning_rate: 0.01,
            },
            decision: DecisionConfig {
                importance_threshold: 0.7,
                conflict_threshold: 0.75,
                merge_similarity: 0.9,
                confidence_min: 0.6,
            },
            relation: RelationConfig {
                min_relation_strength: 0.3,
                temporal_window_secs: 86400,
                decay_halflife_days: 30.0,
                semantic_similarity_weight: 0.6,
                temporal_proximity_weight: 0.4,
            },
            context: ContextConfig {
                base_confidence: 0.5,
                keyword_boost_per_keyword: 0.033,
                temporal_relevance_boost: 0.2,
                spatial_relevance_boost: 0.2,
            },
            performance: PerformanceConfig {
                max_concurrent_searches: 100,
                cache_ttl_seconds: 3600,
                batch_size: 50,
                processing_interval_seconds: 300,
                max_batch_size: 100,
            },
            adaptive: AdaptiveConfig {
                enable_bandit: true,
                exploration_rate: 0.1,
                decay_factor: 0.95,
                alpha_prior: 1.0,
                beta_prior: 1.0,
            },
            embedding: EmbeddingConfig {
                default_dimension: 384,
                similarity_threshold: 0.7,
            },
            hierarchy: HierarchyConfig {
                enable_inheritance: true,
                decay_factor: 0.9,
            },
            llm: LlmConfig {
                default_temperature: 0.7,
                max_tokens: 2000,
                timeout_seconds: 30,
            },
            storage: StorageConfig {
                enable_compression: false,
                enable_deduplication: true,
                dedup_similarity_threshold: 0.95,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AgentMemConfig::default();
        assert_eq!(config.system.version, "4.0.0");
        assert_eq!(config.search.vector_weight, 0.5);
    }

    #[test]
    fn test_load_config() {
        // Test will pass if config file exists
        if let Ok(config) = AgentMemConfig::load_default() {
            assert!(!config.system.version.is_empty());
        }
    }
}
