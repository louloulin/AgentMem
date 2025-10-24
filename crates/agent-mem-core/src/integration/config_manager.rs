use serde::{Deserialize, Serialize};
/// 配置管理器
///
/// 负责系统配置的加载、验证、更新和持久化
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::SystemConfig;
use agent_mem_traits::{AgentMemError, Result};

/// 配置源类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigSource {
    /// 文件配置
    File { path: PathBuf },
    /// 环境变量配置
    Environment,
    /// 内存配置
    Memory,
    /// 远程配置
    Remote { url: String },
}

/// 配置变更事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChangeEvent {
    /// 变更时间
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// 变更类型
    pub change_type: ConfigChangeType,
    /// 变更的配置项
    pub config_key: String,
    /// 旧值
    pub old_value: Option<serde_json::Value>,
    /// 新值
    pub new_value: serde_json::Value,
    /// 变更来源
    pub source: String,
}

/// 配置变更类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigChangeType {
    /// 创建
    Create,
    /// 更新
    Update,
    /// 删除
    Delete,
}

/// 配置验证规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigValidationRule {
    /// 配置项路径
    pub config_path: String,
    /// 验证类型
    pub validation_type: ValidationType,
    /// 验证参数
    pub parameters: HashMap<String, serde_json::Value>,
    /// 错误消息
    pub error_message: String,
}

/// 验证类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationType {
    /// 范围验证
    Range { min: f64, max: f64 },
    /// 枚举验证
    Enum { values: Vec<String> },
    /// 正则表达式验证
    Regex { pattern: String },
    /// 自定义验证
    Custom { validator: String },
}

/// 配置管理器
pub struct ConfigManager {
    /// 当前配置
    current_config: Arc<RwLock<SystemConfig>>,
    /// 配置源
    config_sources: Vec<ConfigSource>,
    /// 配置变更历史
    change_history: Arc<RwLock<Vec<ConfigChangeEvent>>>,
    /// 验证规则
    validation_rules: Vec<ConfigValidationRule>,
    /// 配置文件路径
    config_file_path: Option<PathBuf>,
}

impl ConfigManager {
    /// 创建新的配置管理器
    pub fn new() -> Self {
        Self {
            current_config: Arc::new(RwLock::new(SystemConfig::default())),
            config_sources: vec![ConfigSource::Memory],
            change_history: Arc::new(RwLock::new(Vec::new())),
            validation_rules: Self::create_default_validation_rules(),
            config_file_path: None,
        }
    }

    /// 从文件加载配置
    pub async fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref().to_path_buf();

        if !path.exists() {
            return Err(AgentMemError::ConfigFileNotFound(
                path.to_string_lossy().to_string(),
            ));
        }

        let content =
            fs::read_to_string(&path).map_err(|e| AgentMemError::ConfigLoadError(e.to_string()))?;

        let config: SystemConfig = if path.extension().and_then(|s| s.to_str()) == Some("yaml")
            || path.extension().and_then(|s| s.to_str()) == Some("yml")
        {
            serde_yaml::from_str(&content)
                .map_err(|e| AgentMemError::ConfigParseError(e.to_string()))?
        } else {
            serde_json::from_str(&content)
                .map_err(|e| AgentMemError::ConfigParseError(e.to_string()))?
        };

        // 验证配置
        self.validate_config(&config).await?;

        // 更新配置
        let mut current_config = self.current_config.write().await;
        *current_config = config;

        self.config_file_path = Some(path.clone());
        self.config_sources.push(ConfigSource::File { path });

        Ok(())
    }

    /// 从环境变量加载配置
    pub async fn load_from_environment(&mut self) -> Result<()> {
        let mut config = self.current_config.read().await.clone();

        // 系统配置
        if let Ok(name) = std::env::var("AGENTMEM_SYSTEM_NAME") {
            config.name = name;
        }
        if let Ok(version) = std::env::var("AGENTMEM_SYSTEM_VERSION") {
            config.version = version;
        }
        if let Ok(max_concurrent) = std::env::var("AGENTMEM_MAX_CONCURRENT_OPERATIONS") {
            config.max_concurrent_operations =
                max_concurrent
                    .parse()
                    .map_err(|e: std::num::ParseIntError| {
                        AgentMemError::ConfigParseError(e.to_string())
                    })?;
        }

        // 缓存配置
        if let Ok(cache_size) = std::env::var("AGENTMEM_CACHE_MAX_SIZE_MB") {
            config.cache_config.max_size_mb =
                cache_size.parse().map_err(|e: std::num::ParseIntError| {
                    AgentMemError::ConfigParseError(e.to_string())
                })?;
        }
        if let Ok(cache_ttl) = std::env::var("AGENTMEM_CACHE_TTL_SECONDS") {
            config.cache_config.ttl_seconds =
                cache_ttl.parse().map_err(|e: std::num::ParseIntError| {
                    AgentMemError::ConfigParseError(e.to_string())
                })?;
        }

        // 监控配置
        if let Ok(enable_monitoring) = std::env::var("AGENTMEM_ENABLE_PERFORMANCE_MONITORING") {
            config.monitoring_config.enable_performance_monitoring = enable_monitoring
                .parse()
                .map_err(|e: std::str::ParseBoolError| {
                    AgentMemError::ConfigParseError(e.to_string())
                })?;
        }

        // 安全配置
        if let Ok(enable_encryption) = std::env::var("AGENTMEM_ENABLE_ENCRYPTION") {
            config.security_config.enable_encryption =
                enable_encryption
                    .parse()
                    .map_err(|e: std::str::ParseBoolError| {
                        AgentMemError::ConfigParseError(e.to_string())
                    })?;
        }

        // 验证配置
        self.validate_config(&config).await?;

        // 更新配置
        let mut current_config = self.current_config.write().await;
        *current_config = config;

        if !self
            .config_sources
            .iter()
            .any(|s| matches!(s, ConfigSource::Environment))
        {
            self.config_sources.push(ConfigSource::Environment);
        }

        Ok(())
    }

    /// 保存配置到文件
    pub async fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        let config = self.current_config.read().await;

        let content = if path.extension().and_then(|s| s.to_str()) == Some("yaml")
            || path.extension().and_then(|s| s.to_str()) == Some("yml")
        {
            serde_yaml::to_string(&*config)
                .map_err(|e| AgentMemError::ConfigSerializeError(e.to_string()))?
        } else {
            serde_json::to_string_pretty(&*config)
                .map_err(|e| AgentMemError::ConfigSerializeError(e.to_string()))?
        };

        fs::write(path, content).map_err(|e| AgentMemError::ConfigSaveError(e.to_string()))?;

        Ok(())
    }

    /// 获取当前配置
    pub async fn get_config(&self) -> SystemConfig {
        self.current_config.read().await.clone()
    }

    /// 更新配置
    pub async fn update_config(&self, new_config: SystemConfig) -> Result<()> {
        // 验证新配置
        self.validate_config(&new_config).await?;

        let old_config = {
            let mut current_config = self.current_config.write().await;
            let old = current_config.clone();
            *current_config = new_config.clone();
            old
        };

        // 记录配置变更
        self.record_config_changes(&old_config, &new_config).await;

        // 如果有配置文件路径，自动保存
        if let Some(ref path) = self.config_file_path {
            self.save_to_file(path).await?;
        }

        Ok(())
    }

    /// 部分更新配置
    pub async fn update_config_partial(
        &self,
        updates: HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        let mut config = self.get_config().await;

        for (key, value) in updates {
            self.apply_config_update(&mut config, &key, value)?;
        }

        self.update_config(config).await
    }

    /// 验证配置
    pub async fn validate_config(&self, config: &SystemConfig) -> Result<()> {
        // 基本验证
        if config.name.is_empty() {
            return Err(AgentMemError::ConfigValidationError(
                "系统名称不能为空".to_string(),
            ));
        }

        if config.max_concurrent_operations == 0 {
            return Err(AgentMemError::ConfigValidationError(
                "最大并发操作数必须大于0".to_string(),
            ));
        }

        if config.cache_config.max_size_mb == 0 {
            return Err(AgentMemError::ConfigValidationError(
                "缓存大小必须大于0".to_string(),
            ));
        }

        if config.cache_config.ttl_seconds == 0 {
            return Err(AgentMemError::ConfigValidationError(
                "缓存TTL必须大于0".to_string(),
            ));
        }

        // 应用验证规则
        for rule in &self.validation_rules {
            self.apply_validation_rule(config, rule)?;
        }

        Ok(())
    }

    /// 获取配置变更历史
    pub async fn get_change_history(&self, limit: Option<usize>) -> Vec<ConfigChangeEvent> {
        let history = self.change_history.read().await;
        let limit = limit.unwrap_or(history.len());
        history.iter().rev().take(limit).cloned().collect()
    }

    /// 重置配置为默认值
    pub async fn reset_to_default(&self) -> Result<()> {
        let default_config = SystemConfig::default();
        self.update_config(default_config).await
    }

    /// 创建配置备份
    pub async fn create_backup<P: AsRef<Path>>(&self, backup_path: P) -> Result<()> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_file = backup_path
            .as_ref()
            .join(format!("config_backup_{timestamp}.json"));
        self.save_to_file(backup_file).await
    }

    /// 从备份恢复配置
    pub async fn restore_from_backup<P: AsRef<Path>>(&mut self, backup_path: P) -> Result<()> {
        self.load_from_file(backup_path).await
    }

    /// 私有辅助方法
    fn create_default_validation_rules() -> Vec<ConfigValidationRule> {
        vec![
            ConfigValidationRule {
                config_path: "max_concurrent_operations".to_string(),
                validation_type: ValidationType::Range {
                    min: 1.0,
                    max: 10000.0,
                },
                parameters: HashMap::new(),
                error_message: "最大并发操作数必须在1-10000之间".to_string(),
            },
            ConfigValidationRule {
                config_path: "cache_config.max_size_mb".to_string(),
                validation_type: ValidationType::Range {
                    min: 1.0,
                    max: 10240.0,
                },
                parameters: HashMap::new(),
                error_message: "缓存大小必须在1MB-10GB之间".to_string(),
            },
        ]
    }

    fn apply_config_update(
        &self,
        config: &mut SystemConfig,
        key: &str,
        value: serde_json::Value,
    ) -> Result<()> {
        match key {
            "name" => {
                config.name = value
                    .as_str()
                    .ok_or_else(|| {
                        AgentMemError::ConfigValidationError("name必须是字符串".to_string())
                    })?
                    .to_string();
            }
            "version" => {
                config.version = value
                    .as_str()
                    .ok_or_else(|| {
                        AgentMemError::ConfigValidationError("version必须是字符串".to_string())
                    })?
                    .to_string();
            }
            "max_concurrent_operations" => {
                config.max_concurrent_operations = value.as_u64().ok_or_else(|| {
                    AgentMemError::ConfigValidationError(
                        "max_concurrent_operations必须是数字".to_string(),
                    )
                })? as usize;
            }
            "cache_config.max_size_mb" => {
                config.cache_config.max_size_mb = value.as_u64().ok_or_else(|| {
                    AgentMemError::ConfigValidationError(
                        "cache_config.max_size_mb必须是数字".to_string(),
                    )
                })? as usize;
            }
            _ => {
                return Err(AgentMemError::ConfigValidationError(format!(
                    "未知的配置项: {key}"
                )));
            }
        }
        Ok(())
    }

    fn apply_validation_rule(
        &self,
        config: &SystemConfig,
        rule: &ConfigValidationRule,
    ) -> Result<()> {
        match &rule.validation_type {
            ValidationType::Range { min, max } => {
                let value = match rule.config_path.as_str() {
                    "max_concurrent_operations" => config.max_concurrent_operations as f64,
                    "cache_config.max_size_mb" => config.cache_config.max_size_mb as f64,
                    _ => return Ok(()), // 跳过未知配置项
                };

                if value < *min || value > *max {
                    return Err(AgentMemError::ConfigValidationError(
                        rule.error_message.clone(),
                    ));
                }
            }
            _ => {} // 其他验证类型暂不实现
        }
        Ok(())
    }

    async fn record_config_changes(&self, old_config: &SystemConfig, new_config: &SystemConfig) {
        let mut changes = Vec::new();
        let timestamp = chrono::Utc::now();

        // 比较配置变更
        if old_config.name != new_config.name {
            changes.push(ConfigChangeEvent {
                timestamp,
                change_type: ConfigChangeType::Update,
                config_key: "name".to_string(),
                old_value: Some(serde_json::Value::String(old_config.name.clone())),
                new_value: serde_json::Value::String(new_config.name.clone()),
                source: "config_manager".to_string(),
            });
        }

        if old_config.max_concurrent_operations != new_config.max_concurrent_operations {
            changes.push(ConfigChangeEvent {
                timestamp,
                change_type: ConfigChangeType::Update,
                config_key: "max_concurrent_operations".to_string(),
                old_value: Some(serde_json::Value::Number(serde_json::Number::from(
                    old_config.max_concurrent_operations,
                ))),
                new_value: serde_json::Value::Number(serde_json::Number::from(
                    new_config.max_concurrent_operations,
                )),
                source: "config_manager".to_string(),
            });
        }

        // 记录变更
        let mut history = self.change_history.write().await;
        history.extend(changes);

        // 保持历史记录在合理范围内
        if history.len() > 1000 {
            history.drain(0..100);
        }
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}
