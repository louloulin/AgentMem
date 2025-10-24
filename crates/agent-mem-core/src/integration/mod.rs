use serde::{Deserialize, Serialize};
/// AgentMem 7.0 系统集成模块
///
/// 本模块负责集成所有记忆组件，提供统一的API接口和配置管理系统。
/// 参考 MIRIX 的系统架构设计，但针对 Rust 的特性进行了优化。
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::coordination::MetaMemoryManager;
use crate::managers::{
    ContextualMemoryManager, CoreMemoryManager, KnowledgeVaultManager, ResourceMemoryManager,
};
use crate::retrieval::ActiveRetrievalSystem;
use agent_mem_traits::{AgentMemError, MemoryType, Result};

/// 系统配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    /// 系统名称
    pub name: String,
    /// 版本信息
    pub version: String,
    /// 启用的记忆类型
    pub enabled_memory_types: Vec<MemoryType>,
    /// 最大并发数
    pub max_concurrent_operations: usize,
    /// 缓存配置
    pub cache_config: CacheConfig,
    /// 监控配置
    pub monitoring_config: MonitoringConfig,
    /// 安全配置
    pub security_config: SecurityConfig,
}

/// 缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// 缓存大小限制 (MB)
    pub max_size_mb: usize,
    /// 缓存TTL (秒)
    pub ttl_seconds: u64,
    /// 启用压缩
    pub enable_compression: bool,
}

/// 监控配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// 启用性能监控
    pub enable_performance_monitoring: bool,
    /// 启用健康检查
    pub enable_health_checks: bool,
    /// 监控间隔 (秒)
    pub monitoring_interval_seconds: u64,
    /// 指标保留时间 (小时)
    pub metrics_retention_hours: u64,
}

/// 安全配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// 启用加密
    pub enable_encryption: bool,
    /// 启用审计日志
    pub enable_audit_logging: bool,
    /// 访问控制级别
    pub access_control_level: AccessControlLevel,
}

/// 访问控制级别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessControlLevel {
    /// 无访问控制
    None,
    /// 基础访问控制
    Basic,
    /// 严格访问控制
    Strict,
}

/// 系统状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    /// 系统ID
    pub system_id: Uuid,
    /// 运行状态
    pub status: SystemState,
    /// 启动时间
    pub startup_time: chrono::DateTime<chrono::Utc>,
    /// 最后健康检查时间
    pub last_health_check: chrono::DateTime<chrono::Utc>,
    /// 活跃组件数量
    pub active_components: usize,
    /// 系统指标
    pub metrics: SystemMetrics,
}

/// 系统状态枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SystemState {
    /// 初始化中
    Initializing,
    /// 运行中
    Running,
    /// 暂停中
    Paused,
    /// 关闭中
    Shutting,
    /// 已关闭
    Stopped,
    /// 错误状态
    Error,
}

/// 系统指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// 总内存使用量 (MB)
    pub memory_usage_mb: f64,
    /// CPU使用率 (%)
    pub cpu_usage_percent: f64,
    /// 活跃连接数
    pub active_connections: usize,
    /// 处理的请求总数
    pub total_requests: u64,
    /// 成功请求数
    pub successful_requests: u64,
    /// 失败请求数
    pub failed_requests: u64,
    /// 平均响应时间 (ms)
    pub average_response_time_ms: f64,
}

/// 组件健康状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// 组件名称
    pub component_name: String,
    /// 健康状态
    pub status: HealthStatus,
    /// 最后检查时间
    pub last_check: chrono::DateTime<chrono::Utc>,
    /// 错误信息
    pub error_message: Option<String>,
    /// 性能指标
    pub performance_metrics: HashMap<String, f64>,
}

/// 健康状态枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    /// 健康
    Healthy,
    /// 警告
    Warning,
    /// 不健康
    Unhealthy,
    /// 未知
    Unknown,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            name: "AgentMem 7.0".to_string(),
            version: "7.0.0".to_string(),
            enabled_memory_types: vec![
                MemoryType::Episodic,
                MemoryType::Semantic,
                MemoryType::Procedural,
                MemoryType::Working,
                MemoryType::Core,
                MemoryType::Resource,
                MemoryType::Knowledge,
                MemoryType::Contextual,
            ],
            max_concurrent_operations: 100,
            cache_config: CacheConfig::default(),
            monitoring_config: MonitoringConfig::default(),
            security_config: SecurityConfig::default(),
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size_mb: 512,
            ttl_seconds: 3600,
            enable_compression: true,
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enable_performance_monitoring: true,
            enable_health_checks: true,
            monitoring_interval_seconds: 60,
            metrics_retention_hours: 24,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_encryption: true,
            enable_audit_logging: true,
            access_control_level: AccessControlLevel::Basic,
        }
    }
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
            active_connections: 0,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_response_time_ms: 0.0,
        }
    }
}

/// 系统集成管理器
///
/// 负责协调所有记忆组件，提供统一的API接口
pub struct SystemIntegrationManager {
    /// 系统配置
    config: SystemConfig,
    /// 系统状态
    status: Arc<RwLock<SystemStatus>>,
    /// 核心记忆管理器
    core_memory_manager: Arc<CoreMemoryManager>,
    /// 资源记忆管理器
    resource_memory_manager: Arc<ResourceMemoryManager>,
    /// 知识库管理器
    knowledge_vault_manager: Arc<KnowledgeVaultManager>,
    /// 上下文记忆管理器
    contextual_memory_manager: Arc<ContextualMemoryManager>,
    /// 元记忆管理器
    meta_memory_manager: Arc<MetaMemoryManager>,
    /// 主动检索系统
    active_retrieval_system: Arc<ActiveRetrievalSystem>,
    /// 组件健康状态
    component_health: Arc<RwLock<HashMap<String, ComponentHealth>>>,
}

impl SystemIntegrationManager {
    /// 创建新的系统集成管理器
    pub async fn new(config: SystemConfig) -> Result<Self> {
        let system_id = Uuid::new_v4();
        let startup_time = chrono::Utc::now();

        let status = SystemStatus {
            system_id,
            status: SystemState::Initializing,
            startup_time,
            last_health_check: startup_time,
            active_components: 0,
            metrics: SystemMetrics::default(),
        };

        // 初始化各个管理器
        let core_memory_manager = Arc::new(CoreMemoryManager::new());
        let resource_memory_manager = Arc::new(
            ResourceMemoryManager::new().map_err(|e| AgentMemError::Other(anyhow::anyhow!(e)))?,
        );
        let knowledge_vault_manager = Arc::new(
            KnowledgeVaultManager::new(Default::default())
                .map_err(|e| AgentMemError::Other(anyhow::anyhow!(e)))?,
        );
        let contextual_memory_manager = Arc::new(ContextualMemoryManager::new(Default::default()));
        let meta_memory_manager = Arc::new(MetaMemoryManager::new(Default::default()));
        let active_retrieval_system =
            Arc::new(ActiveRetrievalSystem::new(Default::default()).await?);

        Ok(Self {
            config,
            status: Arc::new(RwLock::new(status)),
            core_memory_manager,
            resource_memory_manager,
            knowledge_vault_manager,
            contextual_memory_manager,
            meta_memory_manager,
            active_retrieval_system,
            component_health: Arc::new(RwLock::new(HashMap::new())),
        })
    }
}

pub mod api_interface;
pub mod config_manager;
pub mod health_checker;
pub mod performance_monitor;
pub mod system_manager;

#[cfg(test)]
pub mod tests;
