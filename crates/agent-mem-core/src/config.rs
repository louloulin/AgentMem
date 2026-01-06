//! Configuration loader for AgentMem V4.0
//! Week 3-4: Eliminate hardcoding by loading all configs from file

use serde::{Deserialize, Serialize};
use std::path::Path;

// 复用现有配置结构
use crate::compression::CompressionConfig;
use crate::importance_scorer::ImportanceScorerConfig;
use crate::intelligence::IntelligenceConfig;
use crate::orchestrator::memory_integration::MemoryIntegratorConfig;
use crate::search::adaptive_threshold::AdaptiveThresholdConfig;
// Use EnhancedHybridConfig from enhanced_hybrid_v2 which is always available
use crate::search::EnhancedHybridConfig as HybridSearchConfig;

/// 主配置 - 整合所有子模块配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct AgentMemConfig {
    /// 混合搜索配置
    #[serde(default)]
    pub hybrid_search: HybridSearchConfig,

    /// 重要性评分配置
    #[serde(default)]
    pub importance_scorer: ImportanceScorerConfig,

    /// 记忆集成配置
    #[serde(default)]
    pub memory_integration: MemoryIntegratorConfig,

    /// 智能配置
    #[serde(default)]
    pub intelligence: IntelligenceConfig,

    /// 压缩配置
    #[serde(default)]
    pub compression: CompressionConfig,

    /// 自适应阈值配置
    #[serde(default)]
    pub adaptive_threshold: AdaptiveThresholdConfig,
}


impl AgentMemConfig {
    /// 从TOML文件加载配置
    pub fn from_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: AgentMemConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// 从TOML字符串加载配置
    pub fn from_toml_str(toml: &str) -> anyhow::Result<Self> {
        let config: AgentMemConfig = toml::from_str(toml)?;
        Ok(config)
    }

    /// 保存配置到文件
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// 从环境变量覆盖配置
    pub fn apply_env_overrides(&mut self) {
        // 混合搜索权重
        if let Ok(val) = std::env::var("AGENTMEM_VECTOR_WEIGHT") {
            if let Ok(weight) = val.parse::<f32>() {
                self.hybrid_search.vector_weight = weight;
            }
        }
        if let Ok(val) = std::env::var("AGENTMEM_FULLTEXT_WEIGHT") {
            if let Ok(weight) = val.parse::<f32>() {
                self.hybrid_search.fulltext_weight = weight;
            }
        }

        // 重要性权重
        if let Ok(val) = std::env::var("AGENTMEM_RECENCY_WEIGHT") {
            if let Ok(weight) = val.parse::<f64>() {
                self.importance_scorer.recency_weight = weight;
            }
        }
    }

    /// 验证配置合法性
    pub fn validate(&self) -> anyhow::Result<()> {
        // 检查搜索权重总和
        let total = self.hybrid_search.vector_weight + self.hybrid_search.fulltext_weight;
        if (total - 1.0).abs() > 0.01 {
            anyhow::bail!(
                "Hybrid search weights must sum to 1.0, got {} (vector: {}, fulltext: {})",
                total,
                self.hybrid_search.vector_weight,
                self.hybrid_search.fulltext_weight
            );
        }

        // 检查重要性权重总和
        let importance_total = self.importance_scorer.recency_weight
            + self.importance_scorer.frequency_weight
            + self.importance_scorer.relevance_weight
            + self.importance_scorer.emotional_weight
            + self.importance_scorer.context_weight
            + self.importance_scorer.interaction_weight;

        if (importance_total - 1.0).abs() > 0.01 {
            anyhow::bail!(
                "Importance weights must sum to 1.0, got {importance_total}"
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AgentMemConfig::default();

        // 验证混合搜索默认值
        assert_eq!(config.hybrid_search.vector_weight, 0.7);
        assert_eq!(config.hybrid_search.fulltext_weight, 0.3);
        assert_eq!(config.hybrid_search.rrf_k, 60.0);

        // 验证重要性评分默认值
        assert_eq!(config.importance_scorer.recency_weight, 0.25);
        assert_eq!(config.importance_scorer.frequency_weight, 0.20);
    }

    #[test]
    fn test_config_validation() {
        let config = AgentMemConfig::default();
        assert!(config.validate().is_ok());

        // 测试非法配置
        let mut bad_config = AgentMemConfig::default();
        bad_config.hybrid_search.vector_weight = 0.9; // 总和 > 1
        assert!(bad_config.validate().is_err());
    }

    #[test]
    fn test_config_serialization() {
        let config = AgentMemConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();

        // 应该包含所有配置段
        assert!(toml_str.contains("[hybrid_search]"));
        assert!(toml_str.contains("[importance_scorer]"));
        assert!(toml_str.contains("vector_weight"));
    }

    #[test]
    fn test_env_overrides() {
        std::env::set_var("AGENTMEM_VECTOR_WEIGHT", "0.8");
        std::env::set_var("AGENTMEM_FULLTEXT_WEIGHT", "0.2");

        let mut config = AgentMemConfig::default();
        config.apply_env_overrides();

        assert_eq!(config.hybrid_search.vector_weight, 0.8);
        assert_eq!(config.hybrid_search.fulltext_weight, 0.2);

        std::env::remove_var("AGENTMEM_VECTOR_WEIGHT");
        std::env::remove_var("AGENTMEM_FULLTEXT_WEIGHT");
    }
}
