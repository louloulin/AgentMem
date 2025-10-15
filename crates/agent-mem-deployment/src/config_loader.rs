//! 配置文件加载器
//!
//! 提供配置文件的加载、验证和解析功能

use agent_mem_config::{DeploymentMode, EmbeddedModeConfig, ServerModeConfig, VectorServiceType};
use agent_mem_traits::{AgentMemError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// 配置文件格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    /// TOML 格式
    Toml,
    /// JSON 格式
    Json,
    /// YAML 格式
    Yaml,
}

impl ConfigFormat {
    /// 从文件扩展名推断格式
    pub fn from_extension(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| match ext.to_lowercase().as_str() {
                "toml" => Some(ConfigFormat::Toml),
                "json" => Some(ConfigFormat::Json),
                "yaml" | "yml" => Some(ConfigFormat::Yaml),
                _ => None,
            })
    }
}

/// 配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    /// 部署配置
    pub deployment: DeploymentConfig,
    /// 嵌入式配置（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedded: Option<EmbeddedConfig>,
    /// 服务器配置（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<ServerConfig>,
    /// 日志配置（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<LoggingConfig>,
    /// 性能配置（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance: Option<PerformanceConfig>,
    /// 安全配置（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<SecurityConfig>,
    /// 功能配置（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<FeaturesConfig>,
}

/// 部署配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    /// 部署模式
    pub mode: String,
}

/// 嵌入式配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedConfig {
    /// 数据库路径
    pub database_path: String,
    /// 向量存储路径
    pub vector_path: String,
    /// 向量维度
    pub vector_dimension: usize,
    /// 是否启用 WAL
    #[serde(default = "default_true")]
    pub enable_wal: bool,
    /// 缓存大小（KB）
    #[serde(default = "default_cache_size")]
    pub cache_size_kb: usize,
}

/// 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// 数据库 URL
    pub database_url: String,
    /// 向量维度
    pub vector_dimension: usize,
    /// 向量服务类型
    #[serde(default = "default_vector_service")]
    pub vector_service: String,
    /// 索引类型
    #[serde(default = "default_index_type")]
    pub index_type: String,
    /// 连接池配置
    #[serde(default)]
    pub pool: PoolConfig,
    /// 索引参数
    #[serde(default)]
    pub index_params: IndexParams,
}

/// 连接池配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    /// 最大连接数
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    /// 最小连接数
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,
    /// 连接超时（秒）
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout_secs: u64,
    /// 空闲超时（秒）
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout_secs: u64,
    /// 最大生命周期（秒）
    #[serde(default = "default_max_lifetime")]
    pub max_lifetime_secs: u64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: default_max_connections(),
            min_connections: default_min_connections(),
            connect_timeout_secs: default_connect_timeout(),
            idle_timeout_secs: default_idle_timeout(),
            max_lifetime_secs: default_max_lifetime(),
        }
    }
}

/// 索引参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexParams {
    /// IVFFlat lists
    #[serde(default = "default_ivfflat_lists")]
    pub ivfflat_lists: u32,
    /// HNSW m
    #[serde(default = "default_hnsw_m")]
    pub hnsw_m: u32,
    /// HNSW ef_construction
    #[serde(default = "default_hnsw_ef_construction")]
    pub hnsw_ef_construction: u32,
}

impl Default for IndexParams {
    fn default() -> Self {
        Self {
            ivfflat_lists: default_ivfflat_lists(),
            hnsw_m: default_hnsw_m(),
            hnsw_ef_construction: default_hnsw_ef_construction(),
        }
    }
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// 日志级别
    #[serde(default = "default_log_level")]
    pub level: String,
    /// 日志格式
    #[serde(default = "default_log_format")]
    pub format: String,
    /// 输出目标
    #[serde(default = "default_log_output")]
    pub output: String,
}

/// 性能配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// 是否启用监控
    #[serde(default)]
    pub enable_monitoring: bool,
    /// 慢查询阈值（毫秒）
    #[serde(default = "default_slow_query_threshold")]
    pub slow_query_threshold_ms: u64,
    /// 是否记录查询
    #[serde(default)]
    pub log_queries: bool,
}

/// 安全配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// 是否启用 API 密钥认证
    #[serde(default)]
    pub enable_api_key_auth: bool,
    /// 是否启用 CORS
    #[serde(default)]
    pub enable_cors: bool,
    /// 允许的源
    #[serde(default)]
    pub allowed_origins: Vec<String>,
}

/// 功能配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturesConfig {
    /// 是否启用自动备份
    #[serde(default)]
    pub enable_auto_backup: bool,
    /// 备份间隔（小时）
    #[serde(default = "default_backup_interval")]
    pub backup_interval_hours: u64,
    /// 备份保留天数
    #[serde(default = "default_backup_retention")]
    pub backup_retention_days: u64,
    /// 是否启用自动清理
    #[serde(default)]
    pub enable_auto_cleanup: bool,
    /// 清理间隔（小时）
    #[serde(default = "default_cleanup_interval")]
    pub cleanup_interval_hours: u64,
    /// 记忆保留天数
    #[serde(default = "default_memory_retention")]
    pub memory_retention_days: u64,
}

// 默认值函数
fn default_true() -> bool { true }
fn default_cache_size() -> usize { 10240 }
fn default_vector_service() -> String { "pgvector".to_string() }
fn default_index_type() -> String { "ivfflat".to_string() }
fn default_max_connections() -> u32 { 20 }
fn default_min_connections() -> u32 { 5 }
fn default_connect_timeout() -> u64 { 30 }
fn default_idle_timeout() -> u64 { 600 }
fn default_max_lifetime() -> u64 { 1800 }
fn default_ivfflat_lists() -> u32 { 100 }
fn default_hnsw_m() -> u32 { 16 }
fn default_hnsw_ef_construction() -> u32 { 64 }
fn default_log_level() -> String { "info".to_string() }
fn default_log_format() -> String { "pretty".to_string() }
fn default_log_output() -> String { "stdout".to_string() }
fn default_slow_query_threshold() -> u64 { 1000 }
fn default_backup_interval() -> u64 { 24 }
fn default_backup_retention() -> u64 { 7 }
fn default_cleanup_interval() -> u64 { 24 }
fn default_memory_retention() -> u64 { 30 }

/// 配置加载器
pub struct ConfigLoader;

impl ConfigLoader {
    /// 从文件加载配置
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<DeploymentMode> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .map_err(|e| AgentMemError::ConfigError(format!("Failed to read config file: {}", e)))?;

        let format = ConfigFormat::from_extension(path)
            .ok_or_else(|| AgentMemError::ConfigError("Unknown config file format".to_string()))?;

        Self::load_from_str(&content, format)
    }

    /// 从字符串加载配置
    pub fn load_from_str(content: &str, format: ConfigFormat) -> Result<DeploymentMode> {
        let config: ConfigFile = match format {
            ConfigFormat::Toml => toml::from_str(content)
                .map_err(|e| AgentMemError::ConfigError(format!("Failed to parse TOML: {}", e)))?,
            ConfigFormat::Json => serde_json::from_str(content)
                .map_err(|e| AgentMemError::ConfigError(format!("Failed to parse JSON: {}", e)))?,
            ConfigFormat::Yaml => serde_yaml::from_str(content)
                .map_err(|e| AgentMemError::ConfigError(format!("Failed to parse YAML: {}", e)))?,
        };

        Self::validate_and_convert(config)
    }

    /// 验证并转换配置
    fn validate_and_convert(config: ConfigFile) -> Result<DeploymentMode> {
        match config.deployment.mode.as_str() {
            "embedded" => {
                let embedded = config.embedded
                    .ok_or_else(|| AgentMemError::ConfigError("Missing embedded configuration".to_string()))?;

                let mode_config = EmbeddedModeConfig {
                    database_path: PathBuf::from(embedded.database_path),
                    vector_path: PathBuf::from(embedded.vector_path),
                    vector_dimension: embedded.vector_dimension,
                    enable_wal: embedded.enable_wal,
                    cache_size_kb: embedded.cache_size_kb,
                };

                Ok(DeploymentMode::Embedded(mode_config))
            }
            "server" => {
                let server = config.server
                    .ok_or_else(|| AgentMemError::ConfigError("Missing server configuration".to_string()))?;

                let vector_service = Self::parse_vector_service(&server.vector_service)?;

                let pool_config = agent_mem_config::PoolConfig {
                    min_connections: server.pool.min_connections,
                    max_connections: server.pool.max_connections,
                    connect_timeout_seconds: server.pool.connect_timeout_secs,
                    idle_timeout_seconds: server.pool.idle_timeout_secs,
                    max_lifetime_seconds: server.pool.max_lifetime_secs,
                };

                let mode_config = ServerModeConfig {
                    database_url: server.database_url,
                    pool_config,
                    vector_service,
                    vector_config: Default::default(),
                    vector_dimension: server.vector_dimension,
                };

                Ok(DeploymentMode::Server(mode_config))
            }
            mode => Err(AgentMemError::ConfigError(format!("Unknown deployment mode: {}", mode))),
        }
    }

    /// 解析向量服务类型
    fn parse_vector_service(service: &str) -> Result<VectorServiceType> {
        match service.to_lowercase().as_str() {
            "pgvector" => Ok(VectorServiceType::PgVector),
            "lancedb" => Ok(VectorServiceType::LanceDB),
            "pinecone" => Ok(VectorServiceType::Pinecone),
            "qdrant" => Ok(VectorServiceType::Qdrant),
            "milvus" => Ok(VectorServiceType::Milvus),
            "weaviate" => Ok(VectorServiceType::Weaviate),
            "chroma" => Ok(VectorServiceType::Chroma),
            "elasticsearch" => Ok(VectorServiceType::Elasticsearch),
            "redis" => Ok(VectorServiceType::Redis),
            "mongodb" => Ok(VectorServiceType::MongoDB),
            "supabase" => Ok(VectorServiceType::Supabase),
            "faiss" => Ok(VectorServiceType::FAISS),
            "azure_ai_search" => Ok(VectorServiceType::AzureAISearch),
            "memory" => Ok(VectorServiceType::Memory),
            _ => Err(AgentMemError::ConfigError(format!("Unknown vector service: {}", service))),
        }
    }
}

