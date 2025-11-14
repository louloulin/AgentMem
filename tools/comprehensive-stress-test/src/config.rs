//! 压测配置模块

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressTestConfig {
    pub memory_creation: MemoryCreationConfig,
    pub memory_retrieval: MemoryRetrievalConfig,
    pub concurrent_ops: ConcurrentOpsConfig,
    pub graph_reasoning: GraphReasoningConfig,
    pub intelligence_processing: IntelligenceProcessingConfig,
    pub cache_performance: CachePerformanceConfig,
    pub batch_operations: BatchOperationsConfig,
    pub stability: StabilityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryCreationConfig {
    pub concurrency: usize,
    pub total_memories: usize,
    pub memory_sizes: Vec<usize>, // 字节
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRetrievalConfig {
    pub dataset_size: usize,
    pub concurrency: usize,
    pub query_types: Vec<String>, // "vector", "fulltext", "hybrid"
    pub top_k: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcurrentOpsConfig {
    pub concurrent_users: usize,
    pub duration_seconds: u64,
    pub read_ratio: f32,  // 0.0 - 1.0
    pub write_ratio: f32, // 0.0 - 1.0
    pub update_ratio: f32, // 0.0 - 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphReasoningConfig {
    pub nodes: usize,
    pub edges: usize,
    pub query_types: Vec<String>, // "shortest_path", "neighbors", "reasoning"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceProcessingConfig {
    pub concurrency: usize,
    pub llm_providers: Vec<String>, // "openai", "anthropic", "deepseek"
    pub processing_types: Vec<String>, // "fact_extraction", "conflict_detection"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachePerformanceConfig {
    pub cache_size_mb: usize,
    pub access_patterns: Vec<String>, // "hot", "random", "sequential"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOperationsConfig {
    pub batch_size: usize,
    pub operation_types: Vec<String>, // "create", "update", "delete", "search"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StabilityConfig {
    pub duration_hours: u64,
    pub baseline_load: usize,
    pub peak_load: usize,
    pub peak_interval_minutes: u64,
}

impl Default for StressTestConfig {
    fn default() -> Self {
        Self {
            memory_creation: MemoryCreationConfig {
                concurrency: 100,
                total_memories: 10000,
                memory_sizes: vec![100, 1000, 10000],
            },
            memory_retrieval: MemoryRetrievalConfig {
                dataset_size: 100000,
                concurrency: 100,
                query_types: vec!["vector".to_string(), "fulltext".to_string(), "hybrid".to_string()],
                top_k: 10,
            },
            concurrent_ops: ConcurrentOpsConfig {
                concurrent_users: 1000,
                duration_seconds: 300,
                read_ratio: 0.7,
                write_ratio: 0.2,
                update_ratio: 0.1,
            },
            graph_reasoning: GraphReasoningConfig {
                nodes: 10000,
                edges: 50000,
                query_types: vec!["shortest_path".to_string(), "neighbors".to_string()],
            },
            intelligence_processing: IntelligenceProcessingConfig {
                concurrency: 10,
                llm_providers: vec!["openai".to_string()],
                processing_types: vec!["fact_extraction".to_string()],
            },
            cache_performance: CachePerformanceConfig {
                cache_size_mb: 500,
                access_patterns: vec!["hot".to_string(), "random".to_string()],
            },
            batch_operations: BatchOperationsConfig {
                batch_size: 100,
                operation_types: vec!["create".to_string(), "search".to_string()],
            },
            stability: StabilityConfig {
                duration_hours: 24,
                baseline_load: 1000,
                peak_load: 5000,
                peak_interval_minutes: 60,
            },
        }
    }
}

impl StressTestConfig {
    pub fn load(path: &str) -> Result<Self> {
        if std::path::Path::new(path).exists() {
            let content = fs::read_to_string(path)?;
            let config: Self = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            // 创建默认配置文件
            let config = Self::default();
            let content = serde_json::to_string_pretty(&config)?;
            fs::write(path, content)?;
            Ok(config)
        }
    }

    pub fn save(&self, path: &str) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}

