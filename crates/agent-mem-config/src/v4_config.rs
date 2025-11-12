//! V4.0 Configuration System
//!
//! Eliminates all hardcoded values by moving them to TOML configuration files.
//! This addresses the core issue of 196+ hardcoded constants in the codebase.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Master configuration for AgentMem V4.0
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMemConfig {
    /// Search configuration
    pub search: SearchConfig,
    
    /// Importance scoring configuration
    pub importance: ImportanceConfig,
    
    /// Intelligence configuration
    pub intelligence: IntelligenceConfig,
    
    /// Memory integration configuration
    pub memory_integration: MemoryIntegrationConfig,
    
    /// Adaptive threshold configuration
    pub adaptive_threshold: AdaptiveThresholdConfig,
    
    /// Performance configuration
    pub performance: PerformanceConfig,
    
    /// Storage configuration
    pub storage: StorageConfig,
}

impl Default for AgentMemConfig {
    fn default() -> Self {
        Self {
            search: SearchConfig::default(),
            importance: ImportanceConfig::default(),
            intelligence: IntelligenceConfig::default(),
            memory_integration: MemoryIntegrationConfig::default(),
            adaptive_threshold: AdaptiveThresholdConfig::default(),
            performance: PerformanceConfig::default(),
            storage: StorageConfig::default(),
        }
    }
}

impl AgentMemConfig {
    /// Load configuration from TOML file
    pub fn from_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }
    
    /// Load configuration from TOML string
    pub fn from_str(content: &str) -> anyhow::Result<Self> {
        let config = toml::from_str(content)?;
        Ok(config)
    }
    
    /// Save configuration to TOML file
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
    
    /// Load from environment variables with prefix AGENTMEM_
    pub fn from_env() -> anyhow::Result<Self> {
        // Start with default config
        let mut config = Self::default();
        
        // Override with environment variables
        if let Ok(val) = std::env::var("AGENTMEM_SEARCH_VECTOR_WEIGHT") {
            config.search.vector_weight = val.parse()?;
        }
        if let Ok(val) = std::env::var("AGENTMEM_SEARCH_FULLTEXT_WEIGHT") {
            config.search.fulltext_weight = val.parse()?;
        }
        if let Ok(val) = std::env::var("AGENTMEM_IMPORTANCE_RECENCY_WEIGHT") {
            config.importance.recency_weight = val.parse()?;
        }
        
        Ok(config)
    }
}

/// Search configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Vector search weight (0.0-1.0)
    pub vector_weight: f32,
    
    /// Fulltext search weight (0.0-1.0)
    pub fulltext_weight: f32,
    
    /// Graph search weight (0.0-1.0)
    pub graph_weight: f32,
    
    /// Enable adaptive weight learning
    pub adaptive_learning: bool,
    
    /// Learning rate for weight adjustment
    pub learning_rate: f32,
    
    /// Default similarity threshold
    pub default_threshold: f32,
    
    /// Maximum results to return
    pub max_results: usize,
    
    /// Timeout for search queries (seconds)
    pub timeout_seconds: u64,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            vector_weight: 0.7,
            fulltext_weight: 0.3,
            graph_weight: 0.0,
            adaptive_learning: true,
            learning_rate: 0.01,
            default_threshold: 0.3,
            max_results: 10,
            timeout_seconds: 30,
        }
    }
}

/// Importance scoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportanceConfig {
    /// Recency weight (0.0-1.0)
    pub recency_weight: f64,
    
    /// Frequency weight (0.0-1.0)
    pub frequency_weight: f64,
    
    /// Relevance weight (0.0-1.0)
    pub relevance_weight: f64,
    
    /// Emotional weight (0.0-1.0)
    pub emotional_weight: f64,
    
    /// Context weight (0.0-1.0)
    pub context_weight: f64,
    
    /// Interaction weight (0.0-1.0)
    pub interaction_weight: f64,
    
    /// Enable dynamic weight adjustment
    pub enable_dynamic_weights: bool,
    
    /// Learning rate for weight adjustment
    pub learning_rate: f64,
    
    /// Minimum score threshold
    pub min_score_threshold: f64,
    
    /// Maximum score cap
    pub max_score_cap: f64,
}

impl Default for ImportanceConfig {
    fn default() -> Self {
        Self {
            recency_weight: 0.25,
            frequency_weight: 0.20,
            relevance_weight: 0.25,
            emotional_weight: 0.15,
            context_weight: 0.10,
            interaction_weight: 0.05,
            enable_dynamic_weights: true,
            learning_rate: 0.01,
            min_score_threshold: 0.0,
            max_score_cap: 10.0,
        }
    }
}

/// Intelligence configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceConfig {
    /// Recency weight for importance
    pub recency: f64,
    
    /// Frequency weight for importance
    pub frequency: f64,
    
    /// Relevance weight for importance
    pub relevance: f64,
    
    /// Interaction weight for importance
    pub interaction: f64,
    
    /// Conflict detection sensitivity (0.0-1.0)
    pub conflict_sensitivity: f64,
    
    /// Auto-resolution threshold (0.0-1.0)
    pub auto_resolution_threshold: f64,
}

impl Default for IntelligenceConfig {
    fn default() -> Self {
        Self {
            recency: 0.3,
            frequency: 0.2,
            relevance: 0.3,
            interaction: 0.2,
            conflict_sensitivity: 0.8,
            auto_resolution_threshold: 0.9,
        }
    }
}

/// Memory integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryIntegrationConfig {
    /// Maximum memories to retrieve
    pub max_memories: usize,
    
    /// Relevance threshold (0.0-1.0)
    pub relevance_threshold: f32,
    
    /// Include timestamp in results
    pub include_timestamp: bool,
    
    /// Sort by importance
    pub sort_by_importance: bool,
    
    /// Episodic memory weight
    pub episodic_weight: f32,
    
    /// Working memory weight
    pub working_weight: f32,
    
    /// Semantic memory weight
    pub semantic_weight: f32,
}

impl Default for MemoryIntegrationConfig {
    fn default() -> Self {
        Self {
            max_memories: 10,
            relevance_threshold: 0.1,
            include_timestamp: true,
            sort_by_importance: true,
            episodic_weight: 1.2,
            working_weight: 1.0,
            semantic_weight: 0.9,
        }
    }
}

/// Adaptive threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveThresholdConfig {
    /// Base thresholds for different query types
    pub base_thresholds: HashMap<String, f32>,
    
    /// Query length influence factor
    pub length_factor: f32,
    
    /// Query complexity influence factor
    pub complexity_factor: f32,
    
    /// Enable historical feedback
    pub enable_historical_feedback: bool,
    
    /// Minimum threshold
    pub min_threshold: f32,
    
    /// Maximum threshold
    pub max_threshold: f32,
}

impl Default for AdaptiveThresholdConfig {
    fn default() -> Self {
        let mut base_thresholds = HashMap::new();
        base_thresholds.insert("exact_id".to_string(), 0.0);
        base_thresholds.insert("short_keyword".to_string(), 0.1);
        base_thresholds.insert("natural_language".to_string(), 0.3);
        base_thresholds.insert("semantic".to_string(), 0.5);
        base_thresholds.insert("temporal".to_string(), 0.0);
        
        Self {
            base_thresholds,
            length_factor: 0.3,
            complexity_factor: 0.2,
            enable_historical_feedback: true,
            min_threshold: 0.0,
            max_threshold: 0.9,
        }
    }
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Batch size for operations
    pub batch_size: usize,
    
    /// Enable caching
    pub enable_cache: bool,
    
    /// Cache size (number of entries)
    pub cache_size: usize,
    
    /// Cache TTL (seconds)
    pub cache_ttl_seconds: u64,
    
    /// Number of parallel workers
    pub num_workers: usize,
    
    /// Enable connection pooling
    pub enable_connection_pool: bool,
    
    /// Connection pool size
    pub connection_pool_size: usize,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            batch_size: 1000,
            enable_cache: true,
            cache_size: 10000,
            cache_ttl_seconds: 3600,
            num_workers: num_cpus::get(),
            enable_connection_pool: true,
            connection_pool_size: 10,
        }
    }
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Storage backend type
    pub backend: String,
    
    /// Database URL
    pub database_url: String,
    
    /// Vector store type
    pub vector_store: String,
    
    /// Vector store URL
    pub vector_store_url: Option<String>,
    
    /// Enable auto-migration
    pub enable_auto_migration: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            backend: "libsql".to_string(),
            database_url: "agentmem.db".to_string(),
            vector_store: "lancedb".to_string(),
            vector_store_url: None,
            enable_auto_migration: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = AgentMemConfig::default();
        assert_eq!(config.search.vector_weight, 0.7);
        assert_eq!(config.importance.recency_weight, 0.25);
    }
    
    #[test]
    fn test_config_serialization() {
        let config = AgentMemConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let deserialized: AgentMemConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.search.vector_weight, deserialized.search.vector_weight);
    }
    
    #[test]
    fn test_config_from_string() {
        let toml_str = r#"
            [search]
            vector_weight = 0.8
            fulltext_weight = 0.2
            graph_weight = 0.0

            [importance]
            recency_weight = 0.3
            frequency_weight = 0.2
        "#;

        let config = AgentMemConfig::from_str(toml_str).unwrap();
        assert_eq!(config.search.vector_weight, 0.8);
        assert_eq!(config.search.fulltext_weight, 0.2);
        assert_eq!(config.importance.recency_weight, 0.3);
    }
}

