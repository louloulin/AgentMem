//! 多租户隔离系统
//!
//! 实现企业级多租户隔离，包括数据隔离、资源隔离、网络隔离和计费隔离。

use agent_mem_traits::AgentMemError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// 租户标识符
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TenantId(pub String);

impl TenantId {
    /// 创建新的租户ID
    pub fn new(id: String) -> Self {
        Self(id)
    }

    /// 生成随机租户ID
    pub fn generate() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// 获取租户ID字符串
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 资源限制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// 最大内存数量
    pub max_memories: usize,
    /// 最大存储大小 (字节)
    pub max_storage_bytes: u64,
    /// 最大并发请求数
    pub max_concurrent_requests: u32,
    /// 每秒最大请求数
    pub max_requests_per_second: u32,
    /// 最大嵌入维度
    pub max_embedding_dimensions: usize,
    /// 最大批量操作大小
    pub max_batch_size: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memories: 10_000,
            max_storage_bytes: 1_000_000_000, // 1GB
            max_concurrent_requests: 100,
            max_requests_per_second: 1000,
            max_embedding_dimensions: 1536,
            max_batch_size: 100,
        }
    }
}

/// 安全策略配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    /// 是否启用数据加密
    pub encryption_enabled: bool,
    /// 是否启用审计日志
    pub audit_logging_enabled: bool,
    /// 是否启用访问控制
    pub access_control_enabled: bool,
    /// 数据保留期限 (天)
    pub data_retention_days: u32,
    /// 是否允许跨租户访问
    pub cross_tenant_access_allowed: bool,
    /// IP 白名单
    pub allowed_ip_ranges: Vec<String>,
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            encryption_enabled: true,
            audit_logging_enabled: true,
            access_control_enabled: true,
            data_retention_days: 365,
            cross_tenant_access_allowed: false,
            allowed_ip_ranges: vec!["0.0.0.0/0".to_string()], // 默认允许所有IP
        }
    }
}

/// 加密配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// 加密算法
    pub algorithm: String,
    /// 密钥ID
    pub key_id: String,
    /// 是否启用传输加密
    pub encrypt_in_transit: bool,
    /// 是否启用静态加密
    pub encrypt_at_rest: bool,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            algorithm: "AES-256-GCM".to_string(),
            key_id: "default".to_string(),
            encrypt_in_transit: true,
            encrypt_at_rest: true,
        }
    }
}

/// 租户配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantConfig {
    /// 租户ID
    pub tenant_id: TenantId,
    /// 租户名称
    pub name: String,
    /// 命名空间
    pub namespace: String,
    /// 资源限制
    pub resource_limits: ResourceLimits,
    /// 安全策略
    pub security_policy: SecurityPolicy,
    /// 数据加密配置
    pub data_encryption: EncryptionConfig,
    /// 创建时间
    pub created_at: i64,
    /// 是否激活
    pub is_active: bool,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

impl TenantConfig {
    /// 创建新的租户配置
    pub fn new(tenant_id: TenantId, name: String) -> Self {
        let namespace = format!("tenant-{}", tenant_id.as_str());
        Self {
            tenant_id,
            name,
            namespace,
            resource_limits: ResourceLimits::default(),
            security_policy: SecurityPolicy::default(),
            data_encryption: EncryptionConfig::default(),
            created_at: chrono::Utc::now().timestamp(),
            is_active: true,
            metadata: HashMap::new(),
        }
    }

    /// 验证租户配置
    pub fn validate(&self) -> Result<(), AgentMemError> {
        if self.name.is_empty() {
            return Err(AgentMemError::validation_error(
                "Tenant name cannot be empty",
            ));
        }

        if self.namespace.is_empty() {
            return Err(AgentMemError::validation_error(
                "Tenant namespace cannot be empty",
            ));
        }

        if self.resource_limits.max_memories == 0 {
            return Err(AgentMemError::validation_error(
                "Max memories must be greater than 0",
            ));
        }

        Ok(())
    }
}

/// 租户隔离引擎
#[derive(Debug)]
pub struct IsolationEngine {
    /// 数据分区策略
    data_partitioning: DataPartitioningStrategy,
    /// 资源隔离策略
    resource_isolation: ResourceIsolationStrategy,
    /// 网络隔离策略
    network_isolation: NetworkIsolationStrategy,
}

/// 数据分区策略
#[derive(Debug, Clone)]
pub enum DataPartitioningStrategy {
    /// 基于租户ID的分区
    TenantBased,
    /// 基于哈希的分区
    HashBased { partitions: u32 },
    /// 基于范围的分区
    RangeBased { ranges: Vec<String> },
}

/// 资源隔离策略
#[derive(Debug, Clone)]
pub enum ResourceIsolationStrategy {
    /// 软限制 (警告但不阻止)
    SoftLimits,
    /// 硬限制 (严格执行)
    HardLimits,
    /// 动态限制 (根据负载调整)
    DynamicLimits,
}

/// 网络隔离策略
#[derive(Debug, Clone)]
pub enum NetworkIsolationStrategy {
    /// 无隔离
    None,
    /// VPC 隔离
    VpcIsolation,
    /// 安全组隔离
    SecurityGroupIsolation,
    /// 完全隔离
    FullIsolation,
}

impl Default for IsolationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl IsolationEngine {
    /// 创建新的隔离引擎
    pub fn new() -> Self {
        Self {
            data_partitioning: DataPartitioningStrategy::TenantBased,
            resource_isolation: ResourceIsolationStrategy::HardLimits,
            network_isolation: NetworkIsolationStrategy::SecurityGroupIsolation,
        }
    }

    /// 获取租户的数据分区键
    pub fn get_partition_key(&self, tenant_id: &TenantId) -> String {
        match &self.data_partitioning {
            DataPartitioningStrategy::TenantBased => {
                format!("tenant_{}", tenant_id.as_str())
            }
            DataPartitioningStrategy::HashBased { partitions } => {
                let hash = self.hash_tenant_id(tenant_id);
                format!("partition_{}", hash % partitions)
            }
            DataPartitioningStrategy::RangeBased { ranges } => {
                // 简化实现：使用第一个范围
                ranges
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "default".to_string())
            }
        }
    }

    /// 验证资源使用是否在限制内
    pub fn check_resource_limits(
        &self,
        tenant_id: &TenantId,
        current_usage: &ResourceUsage,
        limits: &ResourceLimits,
    ) -> Result<(), AgentMemError> {
        match self.resource_isolation {
            ResourceIsolationStrategy::SoftLimits => {
                // 软限制：只记录警告
                if current_usage.memory_count > limits.max_memories {
                    tracing::warn!(
                        "Tenant {} exceeded memory limit: {} > {}",
                        tenant_id.as_str(),
                        current_usage.memory_count,
                        limits.max_memories
                    );
                }
                Ok(())
            }
            ResourceIsolationStrategy::HardLimits => {
                // 硬限制：严格执行
                if current_usage.memory_count >= limits.max_memories {
                    return Err(AgentMemError::validation_error(format!(
                        "Memory count limit exceeded: {} >= {}",
                        current_usage.memory_count, limits.max_memories
                    )));
                }

                if current_usage.storage_bytes >= limits.max_storage_bytes {
                    return Err(AgentMemError::validation_error(format!(
                        "Storage limit exceeded: {} >= {}",
                        current_usage.storage_bytes, limits.max_storage_bytes
                    )));
                }

                Ok(())
            }
            ResourceIsolationStrategy::DynamicLimits => {
                // 动态限制：根据系统负载调整
                let adjusted_limits = self.adjust_limits_for_load(limits);
                self.check_resource_limits(tenant_id, current_usage, &adjusted_limits)
            }
        }
    }

    /// 哈希租户ID
    fn hash_tenant_id(&self, tenant_id: &TenantId) -> u32 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        tenant_id.as_str().hash(&mut hasher);
        hasher.finish() as u32
    }

    /// 根据负载调整限制
    fn adjust_limits_for_load(&self, limits: &ResourceLimits) -> ResourceLimits {
        // 简化实现：根据系统负载动态调整
        // 实际实现中应该考虑当前系统负载、历史使用模式等
        let load_factor = 0.8; // 假设当前负载为80%

        ResourceLimits {
            max_memories: (limits.max_memories as f64 * load_factor) as usize,
            max_storage_bytes: (limits.max_storage_bytes as f64 * load_factor) as u64,
            max_concurrent_requests: (limits.max_concurrent_requests as f64 * load_factor) as u32,
            max_requests_per_second: (limits.max_requests_per_second as f64 * load_factor) as u32,
            max_embedding_dimensions: limits.max_embedding_dimensions,
            max_batch_size: limits.max_batch_size,
        }
    }
}

/// 资源使用情况
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// 当前内存数量
    pub memory_count: usize,
    /// 当前存储使用量 (字节)
    pub storage_bytes: u64,
    /// 当前并发请求数
    pub concurrent_requests: u32,
    /// 当前每秒请求数
    pub requests_per_second: u32,
    /// 最后更新时间
    pub last_updated: i64,
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            memory_count: 0,
            storage_bytes: 0,
            concurrent_requests: 0,
            requests_per_second: 0,
            last_updated: chrono::Utc::now().timestamp(),
        }
    }
}

/// 租户注册表
#[derive(Debug)]
pub struct TenantRegistry {
    /// 租户配置存储
    tenants: Arc<RwLock<HashMap<TenantId, TenantConfig>>>,
    /// 租户资源使用情况
    resource_usage: Arc<RwLock<HashMap<TenantId, ResourceUsage>>>,
}

impl Default for TenantRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl TenantRegistry {
    /// 创建新的租户注册表
    pub fn new() -> Self {
        Self {
            tenants: Arc::new(RwLock::new(HashMap::new())),
            resource_usage: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册新租户
    pub async fn register_tenant(&self, config: TenantConfig) -> Result<(), AgentMemError> {
        config.validate()?;

        let mut tenants = self.tenants.write().await;
        let mut usage = self.resource_usage.write().await;

        if tenants.contains_key(&config.tenant_id) {
            return Err(AgentMemError::validation_error(format!(
                "Tenant {} already exists",
                config.tenant_id.as_str()
            )));
        }

        let tenant_id = config.tenant_id.clone();
        tenants.insert(tenant_id.clone(), config);
        usage.insert(tenant_id.clone(), ResourceUsage::default());

        tracing::info!("Registered new tenant: {}", tenant_id.as_str());
        Ok(())
    }

    /// 获取租户配置
    pub async fn get_tenant(&self, tenant_id: &TenantId) -> Option<TenantConfig> {
        let tenants = self.tenants.read().await;
        tenants.get(tenant_id).cloned()
    }

    /// 更新租户配置
    pub async fn update_tenant(&self, config: TenantConfig) -> Result<(), AgentMemError> {
        config.validate()?;

        let mut tenants = self.tenants.write().await;
        if !tenants.contains_key(&config.tenant_id) {
            return Err(AgentMemError::not_found(format!(
                "Tenant {} not found",
                config.tenant_id.as_str()
            )));
        }

        tenants.insert(config.tenant_id.clone(), config);
        Ok(())
    }

    /// 删除租户
    pub async fn delete_tenant(&self, tenant_id: &TenantId) -> Result<(), AgentMemError> {
        let mut tenants = self.tenants.write().await;
        let mut usage = self.resource_usage.write().await;

        if !tenants.contains_key(tenant_id) {
            return Err(AgentMemError::not_found(format!(
                "Tenant {} not found",
                tenant_id.as_str()
            )));
        }

        tenants.remove(tenant_id);
        usage.remove(tenant_id);

        tracing::info!("Deleted tenant: {}", tenant_id.as_str());
        Ok(())
    }

    /// 列出所有租户
    pub async fn list_tenants(&self) -> Vec<TenantConfig> {
        let tenants = self.tenants.read().await;
        tenants.values().cloned().collect()
    }

    /// 获取租户资源使用情况
    pub async fn get_resource_usage(&self, tenant_id: &TenantId) -> Option<ResourceUsage> {
        let usage = self.resource_usage.read().await;
        usage.get(tenant_id).cloned()
    }

    /// 更新租户资源使用情况
    pub async fn update_resource_usage(
        &self,
        tenant_id: &TenantId,
        usage: ResourceUsage,
    ) -> Result<(), AgentMemError> {
        let mut resource_usage = self.resource_usage.write().await;

        if !resource_usage.contains_key(tenant_id) {
            return Err(AgentMemError::not_found(format!(
                "Tenant {} not found",
                tenant_id.as_str()
            )));
        }

        resource_usage.insert(tenant_id.clone(), usage);
        Ok(())
    }

    /// 验证租户是否存在且激活
    pub async fn validate_tenant(&self, tenant_id: &TenantId) -> Result<(), AgentMemError> {
        let tenants = self.tenants.read().await;

        match tenants.get(tenant_id) {
            Some(config) if config.is_active => Ok(()),
            Some(_) => Err(AgentMemError::validation_error("Tenant is not active")),
            None => Err(AgentMemError::not_found(format!(
                "Tenant {} not found",
                tenant_id.as_str()
            ))),
        }
    }
}

/// 资源管理器
#[derive(Debug)]
pub struct ResourceManager {
    /// 租户注册表
    registry: Arc<TenantRegistry>,
    /// 隔离引擎
    isolation_engine: Arc<IsolationEngine>,
}

impl ResourceManager {
    /// 创建新的资源管理器
    pub fn new(registry: Arc<TenantRegistry>) -> Self {
        Self {
            registry,
            isolation_engine: Arc::new(IsolationEngine::new()),
        }
    }

    /// 检查资源限制
    pub async fn check_limits(
        &self,
        tenant_id: &TenantId,
        operation: ResourceOperation,
    ) -> Result<(), AgentMemError> {
        // 验证租户
        self.registry.validate_tenant(tenant_id).await?;

        // 获取租户配置和当前使用情况
        let config = self
            .registry
            .get_tenant(tenant_id)
            .await
            .ok_or_else(|| AgentMemError::not_found("Tenant not found"))?;

        let mut current_usage = self
            .registry
            .get_resource_usage(tenant_id)
            .await
            .unwrap_or_default();

        // 模拟操作对资源使用的影响
        self.apply_operation(&mut current_usage, &operation);

        // 检查限制
        self.isolation_engine.check_resource_limits(
            tenant_id,
            &current_usage,
            &config.resource_limits,
        )?;

        Ok(())
    }

    /// 记录资源使用
    pub async fn record_usage(
        &self,
        tenant_id: &TenantId,
        operation: ResourceOperation,
    ) -> Result<(), AgentMemError> {
        let mut current_usage = self
            .registry
            .get_resource_usage(tenant_id)
            .await
            .unwrap_or_default();

        self.apply_operation(&mut current_usage, &operation);
        current_usage.last_updated = chrono::Utc::now().timestamp();

        self.registry
            .update_resource_usage(tenant_id, current_usage)
            .await?;
        Ok(())
    }

    /// 应用操作到资源使用情况
    fn apply_operation(&self, usage: &mut ResourceUsage, operation: &ResourceOperation) {
        match operation {
            ResourceOperation::AddMemory { size } => {
                usage.memory_count += 1;
                usage.storage_bytes += size;
            }
            ResourceOperation::RemoveMemory { size } => {
                usage.memory_count = usage.memory_count.saturating_sub(1);
                usage.storage_bytes = usage.storage_bytes.saturating_sub(*size);
            }
            ResourceOperation::StartRequest => {
                usage.concurrent_requests += 1;
                usage.requests_per_second += 1;
            }
            ResourceOperation::EndRequest => {
                usage.concurrent_requests = usage.concurrent_requests.saturating_sub(1);
            }
        }
    }

    /// 获取租户的数据分区键
    pub fn get_partition_key(&self, tenant_id: &TenantId) -> String {
        self.isolation_engine.get_partition_key(tenant_id)
    }
}

/// 资源操作类型
#[derive(Debug, Clone)]
pub enum ResourceOperation {
    /// 添加内存
    AddMemory { size: u64 },
    /// 删除内存
    RemoveMemory { size: u64 },
    /// 开始请求
    StartRequest,
    /// 结束请求
    EndRequest,
}

/// 计费记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingRecord {
    /// 租户ID
    pub tenant_id: TenantId,
    /// 操作类型
    pub operation_type: String,
    /// 资源使用量
    pub resource_usage: u64,
    /// 成本 (分)
    pub cost_cents: u64,
    /// 时间戳
    pub timestamp: i64,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

/// 计费追踪器
#[derive(Debug)]
pub struct BillingTracker {
    /// 计费记录存储
    records: Arc<RwLock<Vec<BillingRecord>>>,
    /// 计费规则
    pricing_rules: Arc<RwLock<HashMap<String, u64>>>, // operation_type -> cost_per_unit_cents
}

impl Default for BillingTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl BillingTracker {
    /// 创建新的计费追踪器
    pub fn new() -> Self {
        let mut pricing_rules = HashMap::new();

        // 默认计费规则 (分/单位)
        pricing_rules.insert("memory_storage".to_string(), 1); // 1分/MB/月
        pricing_rules.insert("api_request".to_string(), 1); // 1分/1000请求
        pricing_rules.insert("embedding_generation".to_string(), 5); // 5分/1000嵌入
        pricing_rules.insert("search_operation".to_string(), 2); // 2分/1000搜索

        Self {
            records: Arc::new(RwLock::new(Vec::new())),
            pricing_rules: Arc::new(RwLock::new(pricing_rules)),
        }
    }

    /// 记录计费事件
    pub async fn record_billing_event(
        &self,
        tenant_id: &TenantId,
        operation_type: &str,
        resource_usage: u64,
    ) -> Result<(), AgentMemError> {
        let pricing_rules = self.pricing_rules.read().await;
        let cost_per_unit = pricing_rules.get(operation_type).copied().unwrap_or(0);
        let cost_cents = (resource_usage * cost_per_unit) / 1000; // 按1000单位计费

        let record = BillingRecord {
            tenant_id: tenant_id.clone(),
            operation_type: operation_type.to_string(),
            resource_usage,
            cost_cents,
            timestamp: chrono::Utc::now().timestamp(),
            metadata: HashMap::new(),
        };

        let mut records = self.records.write().await;
        records.push(record);

        tracing::debug!(
            "Recorded billing event for tenant {}: {} units of {} (cost: {} cents)",
            tenant_id.as_str(),
            resource_usage,
            operation_type,
            cost_cents
        );

        Ok(())
    }

    /// 获取租户的计费记录
    pub async fn get_billing_records(
        &self,
        tenant_id: &TenantId,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Vec<BillingRecord> {
        let records = self.records.read().await;

        records
            .iter()
            .filter(|record| {
                record.tenant_id == *tenant_id
                    && start_time.is_none_or(|start| record.timestamp >= start)
                    && end_time.is_none_or(|end| record.timestamp <= end)
            })
            .cloned()
            .collect()
    }

    /// 计算租户的总费用
    pub async fn calculate_total_cost(
        &self,
        tenant_id: &TenantId,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> u64 {
        let records = self
            .get_billing_records(tenant_id, start_time, end_time)
            .await;
        records.iter().map(|record| record.cost_cents).sum()
    }

    /// 更新计费规则
    pub async fn update_pricing_rule(&self, operation_type: String, cost_per_unit_cents: u64) {
        let mut pricing_rules = self.pricing_rules.write().await;
        pricing_rules.insert(operation_type, cost_per_unit_cents);
    }
}

/// 多租户管理器
#[derive(Debug)]
pub struct MultiTenantManager {
    /// 租户注册表
    pub tenant_registry: Arc<TenantRegistry>,
    /// 资源管理器
    pub resource_manager: Arc<ResourceManager>,
    /// 隔离引擎
    pub isolation_engine: Arc<IsolationEngine>,
    /// 计费追踪器
    pub billing_tracker: Arc<BillingTracker>,
}

impl Default for MultiTenantManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiTenantManager {
    /// 创建新的多租户管理器
    pub fn new() -> Self {
        let tenant_registry = Arc::new(TenantRegistry::new());
        let resource_manager = Arc::new(ResourceManager::new(tenant_registry.clone()));
        let isolation_engine = Arc::new(IsolationEngine::new());
        let billing_tracker = Arc::new(BillingTracker::new());

        Self {
            tenant_registry,
            resource_manager,
            isolation_engine,
            billing_tracker,
        }
    }

    /// 创建新租户
    pub async fn create_tenant(
        &self,
        name: String,
        resource_limits: Option<ResourceLimits>,
        security_policy: Option<SecurityPolicy>,
    ) -> Result<TenantId, AgentMemError> {
        let tenant_id = TenantId::generate();
        let mut config = TenantConfig::new(tenant_id.clone(), name);

        if let Some(limits) = resource_limits {
            config.resource_limits = limits;
        }

        if let Some(policy) = security_policy {
            config.security_policy = policy;
        }

        self.tenant_registry.register_tenant(config).await?;

        tracing::info!("Created new tenant: {}", tenant_id.as_str());
        Ok(tenant_id)
    }

    /// 验证租户操作权限
    pub async fn validate_operation(
        &self,
        tenant_id: &TenantId,
        operation: ResourceOperation,
    ) -> Result<(), AgentMemError> {
        // 验证租户存在且激活
        self.tenant_registry.validate_tenant(tenant_id).await?;

        // 检查资源限制
        self.resource_manager
            .check_limits(tenant_id, operation.clone())
            .await?;

        // 记录资源使用
        self.resource_manager
            .record_usage(tenant_id, operation)
            .await?;

        Ok(())
    }

    /// 获取租户的数据分区键
    pub fn get_partition_key(&self, tenant_id: &TenantId) -> String {
        self.resource_manager.get_partition_key(tenant_id)
    }

    /// 记录计费事件
    pub async fn record_billing(
        &self,
        tenant_id: &TenantId,
        operation_type: &str,
        resource_usage: u64,
    ) -> Result<(), AgentMemError> {
        self.billing_tracker
            .record_billing_event(tenant_id, operation_type, resource_usage)
            .await
    }

    /// 获取租户统计信息
    pub async fn get_tenant_stats(
        &self,
        tenant_id: &TenantId,
    ) -> Result<TenantStats, AgentMemError> {
        let config = self
            .tenant_registry
            .get_tenant(tenant_id)
            .await
            .ok_or_else(|| AgentMemError::not_found("Tenant not found"))?;

        let usage = self
            .tenant_registry
            .get_resource_usage(tenant_id)
            .await
            .unwrap_or_default();

        let total_cost = self
            .billing_tracker
            .calculate_total_cost(tenant_id, None, None)
            .await;

        Ok(TenantStats {
            tenant_id: tenant_id.clone(),
            name: config.name,
            resource_usage: usage,
            resource_limits: config.resource_limits,
            total_cost_cents: total_cost,
            is_active: config.is_active,
            created_at: config.created_at,
        })
    }
}

/// 租户统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantStats {
    /// 租户ID
    pub tenant_id: TenantId,
    /// 租户名称
    pub name: String,
    /// 资源使用情况
    pub resource_usage: ResourceUsage,
    /// 资源限制
    pub resource_limits: ResourceLimits,
    /// 总费用 (分)
    pub total_cost_cents: u64,
    /// 是否激活
    pub is_active: bool,
    /// 创建时间
    pub created_at: i64,
}
