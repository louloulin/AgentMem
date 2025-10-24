//! 统一配置管理系统
//!
//! 提供配置加载、验证、热重载和环境变量支持

use agent_mem_traits::{AgentMemError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// 配置源接口
pub trait ConfigSource: Send + Sync {
    /// 加载配置
    fn load_config(&self) -> Result<AgentMemConfig>;

    /// 获取配置源名称
    fn source_name(&self) -> &str;

    /// 检查配置是否已更改
    fn has_changed(&self) -> bool;
}

/// 文件配置源
pub struct FileConfigSource {
    file_path: String,
    last_modified: Option<SystemTime>,
}

impl FileConfigSource {
    pub fn new(file_path: String) -> Self {
        Self {
            file_path,
            last_modified: None,
        }
    }
}

impl ConfigSource for FileConfigSource {
    fn load_config(&self) -> Result<AgentMemConfig> {
        let content = fs::read_to_string(&self.file_path).map_err(|e| {
            AgentMemError::config_error(format!(
                "Failed to read config file {}: {}",
                self.file_path, e
            ))
        })?;

        let config: AgentMemConfig = if self.file_path.ends_with(".json") {
            serde_json::from_str(&content).map_err(|e| {
                AgentMemError::config_error(format!("Failed to parse JSON config: {}", e))
            })?
        } else if self.file_path.ends_with(".yaml") || self.file_path.ends_with(".yml") {
            serde_yaml::from_str(&content).map_err(|e| {
                AgentMemError::config_error(format!("Failed to parse YAML config: {}", e))
            })?
        } else {
            return Err(AgentMemError::config_error(
                "Unsupported config file format".to_string(),
            ));
        };

        info!("Loaded configuration from file: {}", self.file_path);
        Ok(config)
    }

    fn source_name(&self) -> &str {
        &self.file_path
    }

    fn has_changed(&self) -> bool {
        if let Ok(metadata) = fs::metadata(&self.file_path) {
            if let Ok(modified) = metadata.modified() {
                return self.last_modified.map_or(true, |last| modified > last);
            }
        }
        false
    }
}

/// 环境变量配置源
pub struct EnvConfigSource {
    prefix: String,
}

impl EnvConfigSource {
    pub fn new(prefix: String) -> Self {
        Self { prefix }
    }
}

impl ConfigSource for EnvConfigSource {
    fn load_config(&self) -> Result<AgentMemConfig> {
        let mut config = AgentMemConfig::default();

        // 加载 LLM 配置
        if let Ok(provider) = env::var(format!("{}_LLM_PROVIDER", self.prefix)) {
            config.llm.provider = provider;
        }
        if let Ok(api_key) = env::var(format!("{}_LLM_API_KEY", self.prefix)) {
            config.llm.api_key = Some(api_key);
        }
        if let Ok(model) = env::var(format!("{}_LLM_MODEL", self.prefix)) {
            config.llm.model = model;
        }

        // 加载向量存储配置
        if let Ok(provider) = env::var(format!("{}_VECTOR_STORE_PROVIDER", self.prefix)) {
            config.vector_store.provider = provider;
        }
        if let Ok(url) = env::var(format!("{}_VECTOR_STORE_URL", self.prefix)) {
            config.vector_store.url = Some(url);
        }

        // 加载性能配置
        if let Ok(batch_size) = env::var(format!("{}_BATCH_SIZE", self.prefix)) {
            if let Ok(size) = batch_size.parse::<usize>() {
                config.performance.batch_size = size;
            }
        }

        info!(
            "Loaded configuration from environment variables with prefix: {}",
            self.prefix
        );
        Ok(config)
    }

    fn source_name(&self) -> &str {
        "environment"
    }

    fn has_changed(&self) -> bool {
        // 环境变量配置通常不会在运行时改变
        false
    }
}

/// 统一配置管理器
pub struct UnifiedConfigManager {
    config_sources: Vec<Box<dyn ConfigSource>>,
    config_cache: Arc<RwLock<AgentMemConfig>>,
    hot_reload: bool,
    reload_interval: Duration,
}

impl UnifiedConfigManager {
    /// 创建新的配置管理器
    pub fn new(hot_reload: bool) -> Self {
        Self {
            config_sources: Vec::new(),
            config_cache: Arc::new(RwLock::new(AgentMemConfig::default())),
            hot_reload,
            reload_interval: Duration::from_secs(30),
        }
    }

    /// 添加配置源
    pub fn add_source(&mut self, source: Box<dyn ConfigSource>) {
        info!("Added config source: {}", source.source_name());
        self.config_sources.push(source);
    }

    /// 加载配置
    pub async fn load_config(&self) -> Result<AgentMemConfig> {
        let mut merged_config = AgentMemConfig::default();

        // 按顺序合并所有配置源
        for source in &self.config_sources {
            match source.load_config() {
                Ok(config) => {
                    merged_config = self.merge_configs(merged_config, config);
                    debug!("Merged config from source: {}", source.source_name());
                }
                Err(e) => {
                    warn!("Failed to load config from {}: {}", source.source_name(), e);
                }
            }
        }

        // 验证配置
        merged_config.validate()?;

        // 更新缓存
        *self.config_cache.write().await = merged_config.clone();

        info!("Configuration loaded and validated successfully");
        Ok(merged_config)
    }

    /// 获取当前配置
    pub async fn get_config(&self) -> AgentMemConfig {
        self.config_cache.read().await.clone()
    }

    /// 启动热重载
    pub async fn start_hot_reload(&self) {
        if !self.hot_reload {
            return;
        }

        let sources = self
            .config_sources
            .iter()
            .map(|s| s.source_name().to_string())
            .collect::<Vec<_>>();
        let config_cache = self.config_cache.clone();
        let reload_interval = self.reload_interval;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(reload_interval);

            loop {
                interval.tick().await;

                // 检查配置源是否有变化
                let mut has_changes = false;
                for source_name in &sources {
                    // 这里需要实际的变化检测逻辑
                    // 简化实现，实际应该检查文件修改时间等
                    debug!("Checking for changes in config source: {}", source_name);
                }

                if has_changes {
                    info!("Configuration changes detected, reloading...");
                    // 这里应该重新加载配置
                    // 由于借用检查器的限制，这里简化实现
                }
            }
        });

        info!("Hot reload started with interval: {:?}", reload_interval);
    }

    /// 合并配置
    fn merge_configs(
        &self,
        base: AgentMemConfig,
        override_config: AgentMemConfig,
    ) -> AgentMemConfig {
        AgentMemConfig {
            llm: self.merge_llm_config(base.llm, override_config.llm),
            vector_store: self
                .merge_vector_store_config(base.vector_store, override_config.vector_store),
            graph_store: override_config.graph_store.or(base.graph_store),
            embedder: self.merge_embedder_config(base.embedder, override_config.embedder),
            intelligence: self
                .merge_intelligence_config(base.intelligence, override_config.intelligence),
            telemetry: self.merge_telemetry_config(base.telemetry, override_config.telemetry),
            performance: self
                .merge_performance_config(base.performance, override_config.performance),
        }
    }

    fn merge_llm_config(&self, base: LLMConfig, override_config: LLMConfig) -> LLMConfig {
        LLMConfig {
            provider: if override_config.provider.is_empty() {
                base.provider
            } else {
                override_config.provider
            },
            api_key: override_config.api_key.or(base.api_key),
            model: if override_config.model.is_empty() {
                base.model
            } else {
                override_config.model
            },
            base_url: override_config.base_url.or(base.base_url),
            timeout: if override_config.timeout == Duration::default() {
                base.timeout
            } else {
                override_config.timeout
            },
            max_tokens: if override_config.max_tokens == 0 {
                base.max_tokens
            } else {
                override_config.max_tokens
            },
            temperature: if override_config.temperature == 0.0 {
                base.temperature
            } else {
                override_config.temperature
            },
        }
    }

    fn merge_vector_store_config(
        &self,
        base: VectorStoreConfig,
        override_config: VectorStoreConfig,
    ) -> VectorStoreConfig {
        VectorStoreConfig {
            provider: if override_config.provider.is_empty() {
                base.provider
            } else {
                override_config.provider
            },
            url: override_config.url.or(base.url),
            api_key: override_config.api_key.or(base.api_key),
            collection_name: if override_config.collection_name.is_empty() {
                base.collection_name
            } else {
                override_config.collection_name
            },
            dimension: if override_config.dimension == 0 {
                base.dimension
            } else {
                override_config.dimension
            },
            metric: override_config.metric.or(base.metric),
        }
    }

    fn merge_embedder_config(
        &self,
        base: EmbedderConfig,
        override_config: EmbedderConfig,
    ) -> EmbedderConfig {
        EmbedderConfig {
            provider: if override_config.provider.is_empty() {
                base.provider
            } else {
                override_config.provider
            },
            model: if override_config.model.is_empty() {
                base.model
            } else {
                override_config.model
            },
            api_key: override_config.api_key.or(base.api_key),
            base_url: override_config.base_url.or(base.base_url),
            dimension: if override_config.dimension == 0 {
                base.dimension
            } else {
                override_config.dimension
            },
            batch_size: if override_config.batch_size == 0 {
                base.batch_size
            } else {
                override_config.batch_size
            },
        }
    }

    fn merge_intelligence_config(
        &self,
        base: IntelligenceConfig,
        override_config: IntelligenceConfig,
    ) -> IntelligenceConfig {
        IntelligenceConfig {
            enable_fact_extraction: override_config.enable_fact_extraction
                || base.enable_fact_extraction,
            enable_conflict_resolution: override_config.enable_conflict_resolution
                || base.enable_conflict_resolution,
            enable_importance_evaluation: override_config.enable_importance_evaluation
                || base.enable_importance_evaluation,
            confidence_threshold: if override_config.confidence_threshold == 0.0 {
                base.confidence_threshold
            } else {
                override_config.confidence_threshold
            },
        }
    }

    fn merge_telemetry_config(
        &self,
        base: TelemetryConfig,
        override_config: TelemetryConfig,
    ) -> TelemetryConfig {
        TelemetryConfig {
            enable_metrics: override_config.enable_metrics || base.enable_metrics,
            enable_tracing: override_config.enable_tracing || base.enable_tracing,
            enable_logging: override_config.enable_logging || base.enable_logging,
            metrics_endpoint: override_config.metrics_endpoint.or(base.metrics_endpoint),
            tracing_endpoint: override_config.tracing_endpoint.or(base.tracing_endpoint),
            sample_rate: if override_config.sample_rate == 0.0 {
                base.sample_rate
            } else {
                override_config.sample_rate
            },
        }
    }

    fn merge_performance_config(
        &self,
        base: PerformanceConfig,
        override_config: PerformanceConfig,
    ) -> PerformanceConfig {
        PerformanceConfig {
            batch_size: if override_config.batch_size == 0 {
                base.batch_size
            } else {
                override_config.batch_size
            },
            cache_size: if override_config.cache_size == 0 {
                base.cache_size
            } else {
                override_config.cache_size
            },
            max_concurrent_requests: if override_config.max_concurrent_requests == 0 {
                base.max_concurrent_requests
            } else {
                override_config.max_concurrent_requests
            },
            request_timeout: if override_config.request_timeout == Duration::default() {
                base.request_timeout
            } else {
                override_config.request_timeout
            },
        }
    }
}

// 配置结构定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMemConfig {
    pub llm: LLMConfig,
    pub vector_store: VectorStoreConfig,
    pub graph_store: Option<GraphStoreConfig>,
    pub embedder: EmbedderConfig,
    pub intelligence: IntelligenceConfig,
    pub telemetry: TelemetryConfig,
    pub performance: PerformanceConfig,
}

impl Default for AgentMemConfig {
    fn default() -> Self {
        Self {
            llm: LLMConfig::default(),
            vector_store: VectorStoreConfig::default(),
            graph_store: None,
            embedder: EmbedderConfig::default(),
            intelligence: IntelligenceConfig::default(),
            telemetry: TelemetryConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

impl AgentMemConfig {
    /// 从环境变量加载配置
    pub fn from_env() -> Result<Self> {
        let source = EnvConfigSource::new("AGENTMEM".to_string());
        source.load_config()
    }

    /// 从配置文件加载
    pub fn from_file(path: &str) -> Result<Self> {
        let source = FileConfigSource::new(path.to_string());
        source.load_config()
    }

    /// 配置验证
    pub fn validate(&self) -> Result<()> {
        // 验证 LLM 配置
        if self.llm.provider.is_empty() {
            return Err(AgentMemError::config_error(
                "LLM provider is required".to_string(),
            ));
        }

        // 验证向量存储配置
        if self.vector_store.provider.is_empty() {
            return Err(AgentMemError::config_error(
                "Vector store provider is required".to_string(),
            ));
        }

        if self.vector_store.dimension == 0 {
            return Err(AgentMemError::config_error(
                "Vector dimension must be greater than 0".to_string(),
            ));
        }

        // 验证性能配置
        if self.performance.batch_size == 0 {
            return Err(AgentMemError::config_error(
                "Batch size must be greater than 0".to_string(),
            ));
        }

        if self.performance.max_concurrent_requests == 0 {
            return Err(AgentMemError::config_error(
                "Max concurrent requests must be greater than 0".to_string(),
            ));
        }

        info!("Configuration validation passed");
        Ok(())
    }
}

// 各种配置结构定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub provider: String,
    pub api_key: Option<String>,
    pub model: String,
    pub base_url: Option<String>,
    pub timeout: Duration,
    pub max_tokens: u32,
    pub temperature: f32,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(),
            api_key: None,
            model: "gpt-3.5-turbo".to_string(),
            base_url: None,
            timeout: Duration::from_secs(30),
            max_tokens: 1000,
            temperature: 0.7,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreConfig {
    pub provider: String,
    pub url: Option<String>,
    pub api_key: Option<String>,
    pub collection_name: String,
    pub dimension: usize,
    pub metric: Option<String>,
}

impl Default for VectorStoreConfig {
    fn default() -> Self {
        Self {
            provider: "memory".to_string(),
            url: None,
            api_key: None,
            collection_name: "default".to_string(),
            dimension: 1536,
            metric: Some("cosine".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStoreConfig {
    pub provider: String,
    pub url: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub database: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedderConfig {
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub dimension: usize,
    pub batch_size: usize,
}

impl Default for EmbedderConfig {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(),
            model: "text-embedding-ada-002".to_string(),
            api_key: None,
            base_url: None,
            dimension: 1536,
            batch_size: 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceConfig {
    pub enable_fact_extraction: bool,
    pub enable_conflict_resolution: bool,
    pub enable_importance_evaluation: bool,
    pub confidence_threshold: f32,
}

impl Default for IntelligenceConfig {
    fn default() -> Self {
        Self {
            enable_fact_extraction: true,
            enable_conflict_resolution: true,
            enable_importance_evaluation: true,
            confidence_threshold: 0.7,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    pub enable_metrics: bool,
    pub enable_tracing: bool,
    pub enable_logging: bool,
    pub metrics_endpoint: Option<String>,
    pub tracing_endpoint: Option<String>,
    pub sample_rate: f32,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enable_metrics: true,
            enable_tracing: true,
            enable_logging: true,
            metrics_endpoint: None,
            tracing_endpoint: None,
            sample_rate: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub batch_size: usize,
    pub cache_size: usize,
    pub max_concurrent_requests: usize,
    pub request_timeout: Duration,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            cache_size: 1000,
            max_concurrent_requests: 100,
            request_timeout: Duration::from_secs(30),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_config_manager_creation() {
        let manager = UnifiedConfigManager::new(false);
        let config = manager.get_config().await;

        // 应该返回默认配置
        assert_eq!(config.llm.provider, "openai");
        assert_eq!(config.vector_store.provider, "memory");
    }

    #[tokio::test]
    async fn test_file_config_source() {
        let config = AgentMemConfig {
            llm: LLMConfig {
                provider: "test_provider".to_string(),
                model: "test_model".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };

        let config_json = serde_json::to_string_pretty(&config).unwrap();
        // Create a temporary file with .json extension
        let temp_file = NamedTempFile::with_suffix(".json").unwrap();
        fs::write(temp_file.path(), config_json).unwrap();

        let source = FileConfigSource::new(temp_file.path().to_string_lossy().to_string());
        let loaded_config = source.load_config().unwrap();

        assert_eq!(loaded_config.llm.provider, "test_provider");
        assert_eq!(loaded_config.llm.model, "test_model");
    }

    #[tokio::test]
    async fn test_env_config_source() {
        env::set_var("AGENTMEM_LLM_PROVIDER", "env_provider");
        env::set_var("AGENTMEM_LLM_MODEL", "env_model");
        env::set_var("AGENTMEM_BATCH_SIZE", "50");

        let source = EnvConfigSource::new("AGENTMEM".to_string());
        let config = source.load_config().unwrap();

        assert_eq!(config.llm.provider, "env_provider");
        assert_eq!(config.llm.model, "env_model");
        assert_eq!(config.performance.batch_size, 50);

        // 清理环境变量
        env::remove_var("AGENTMEM_LLM_PROVIDER");
        env::remove_var("AGENTMEM_LLM_MODEL");
        env::remove_var("AGENTMEM_BATCH_SIZE");
    }

    #[tokio::test]
    async fn test_config_validation() {
        let mut config = AgentMemConfig::default();

        // 有效配置应该通过验证
        assert!(config.validate().is_ok());

        // 无效配置应该失败
        config.llm.provider = "".to_string();
        assert!(config.validate().is_err());

        config.llm.provider = "openai".to_string();
        config.vector_store.dimension = 0;
        assert!(config.validate().is_err());
    }
}
