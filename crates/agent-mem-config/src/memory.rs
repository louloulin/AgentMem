//! Memory configuration

use agent_mem_traits::{LLMConfig, VectorStoreConfig};
use serde::{Deserialize, Serialize};

/// Main configuration for memory management
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryConfig {
    /// LLM provider configuration
    pub llm: LLMConfig,

    /// Vector store configuration
    pub vector_store: VectorStoreConfig,

    /// Graph store configuration (optional)
    pub graph_store: Option<GraphStoreConfig>,

    /// Embedder configuration
    pub embedder: EmbedderConfig,

    /// Session configuration
    pub session: SessionConfig,

    /// Intelligence configuration
    pub intelligence: IntelligenceConfig,

    /// Performance configuration
    pub performance: PerformanceConfig,
}

/// Graph store configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStoreConfig {
    pub provider: String,
    pub uri: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub database: Option<String>,
}

impl Default for GraphStoreConfig {
    fn default() -> Self {
        Self {
            provider: "neo4j".to_string(),
            uri: "bolt://localhost:7687".to_string(),
            username: None,
            password: None,
            database: Some("neo4j".to_string()),
        }
    }
}

/// Embedder configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedderConfig {
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub dimension: usize,
}

impl Default for EmbedderConfig {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(),
            model: "text-embedding-ada-002".to_string(),
            api_key: None,
            base_url: None,
            dimension: 1536,
        }
    }
}

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub default_user_id: Option<String>,
    pub default_agent_id: Option<String>,
    pub session_timeout_seconds: u64,
    pub enable_telemetry: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            default_user_id: None,
            default_agent_id: None,
            session_timeout_seconds: 3600, // 1 hour
            enable_telemetry: true,
        }
    }
}

/// Intelligence configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceConfig {
    // 现有字段
    pub similarity_threshold: f32,
    pub clustering_threshold: f32,
    pub enable_conflict_detection: bool,
    pub enable_memory_summarization: bool,
    pub importance_scoring: bool,

    // 新增智能功能开关
    /// 启用智能事实提取 (使用 LLM 从内容中提取结构化事实)
    pub enable_intelligent_extraction: bool,

    /// 启用智能决策引擎 (自动决定 ADD/UPDATE/DELETE/MERGE 操作)
    pub enable_decision_engine: bool,

    /// 启用记忆去重 (自动检测和合并重复记忆)
    pub enable_deduplication: bool,

    /// 事实提取配置
    pub fact_extraction: FactExtractionConfig,

    /// 决策引擎配置
    pub decision_engine: DecisionEngineConfig,

    /// 去重配置
    pub deduplication: DeduplicationConfig,
}

impl Default for IntelligenceConfig {
    fn default() -> Self {
        Self {
            // 现有默认值
            similarity_threshold: 0.8,
            clustering_threshold: 0.7,
            enable_conflict_detection: true,
            enable_memory_summarization: true,
            importance_scoring: true,

            // 新增默认值 (默认启用智能功能)
            enable_intelligent_extraction: true,
            enable_decision_engine: true,
            enable_deduplication: false, // 去重默认关闭，可选启用

            fact_extraction: FactExtractionConfig::default(),
            decision_engine: DecisionEngineConfig::default(),
            deduplication: DeduplicationConfig::default(),
        }
    }
}

/// 事实提取配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactExtractionConfig {
    /// 最小置信度阈值 (0.0-1.0)
    pub min_confidence: f32,

    /// 是否提取实体
    pub extract_entities: bool,

    /// 是否提取关系
    pub extract_relations: bool,

    /// 最大提取事实数量
    pub max_facts_per_message: usize,
}

impl Default for FactExtractionConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.7,
            extract_entities: true,
            extract_relations: true,
            max_facts_per_message: 10,
        }
    }
}

/// 决策引擎配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionEngineConfig {
    /// 相似度阈值 (用于判断是否为重复记忆)
    pub similarity_threshold: f32,

    /// 最小决策置信度
    pub min_decision_confidence: f32,

    /// 是否启用智能合并
    pub enable_intelligent_merge: bool,

    /// 查找相似记忆的数量限制
    pub max_similar_memories: usize,
}

impl Default for DecisionEngineConfig {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.85,
            min_decision_confidence: 0.6,
            enable_intelligent_merge: true,
            max_similar_memories: 5,
        }
    }
}

/// 去重配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeduplicationConfig {
    /// 去重相似度阈值
    pub similarity_threshold: f32,

    /// 时间窗口 (秒) - 只在此时间窗口内查找重复
    pub time_window_seconds: Option<i64>,

    /// 合并策略
    pub merge_strategy: String, // "keep_latest" | "keep_most_important" | "intelligent_merge"
}

impl Default for DeduplicationConfig {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.9,
            time_window_seconds: Some(3600), // 1 hour
            merge_strategy: "intelligent_merge".to_string(),
        }
    }
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Number of retry attempts for failed operations
    pub retry_attempts: Option<u32>,
    /// Base delay in milliseconds for exponential backoff
    pub base_delay_ms: Option<u64>,
    /// Maximum delay in milliseconds for exponential backoff
    pub max_delay_ms: Option<u64>,
    /// Maximum number of concurrent operations
    pub max_concurrent_operations: Option<usize>,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            retry_attempts: Some(3),
            base_delay_ms: Some(100),
            max_delay_ms: Some(5000),
            max_concurrent_operations: Some(10),
        }
    }
}
