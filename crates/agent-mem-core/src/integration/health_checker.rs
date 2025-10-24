use serde::{Deserialize, Serialize};
/// 健康检查器
///
/// 负责监控系统各组件的健康状态，提供实时健康检查和告警功能
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::interval;

use super::{HealthStatus, SystemIntegrationManager};
use agent_mem_traits::{AgentMemError, Result};

/// 健康检查配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// 检查间隔 (秒)
    pub check_interval_seconds: u64,
    /// 超时时间 (秒)
    pub timeout_seconds: u64,
    /// 重试次数
    pub retry_count: u32,
    /// 失败阈值
    pub failure_threshold: u32,
    /// 恢复阈值
    pub recovery_threshold: u32,
}

/// 健康检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// 组件名称
    pub component_name: String,
    /// 检查状态
    pub status: HealthStatus,
    /// 检查时间
    pub check_time: chrono::DateTime<chrono::Utc>,
    /// 响应时间 (ms)
    pub response_time_ms: u64,
    /// 错误信息
    pub error_message: Option<String>,
    /// 详细指标
    pub metrics: HashMap<String, f64>,
}

/// 健康检查器
pub struct HealthChecker {
    /// 配置
    config: HealthCheckConfig,
    /// 检查历史
    check_history: Arc<RwLock<HashMap<String, Vec<HealthCheckResult>>>>,
    /// 组件失败计数
    failure_counts: Arc<RwLock<HashMap<String, u32>>>,
    /// 是否正在运行
    is_running: Arc<RwLock<bool>>,
}

impl HealthChecker {
    /// 创建新的健康检查器
    pub fn new(config: HealthCheckConfig) -> Self {
        Self {
            config,
            check_history: Arc::new(RwLock::new(HashMap::new())),
            failure_counts: Arc::new(RwLock::new(HashMap::new())),
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// 启动健康检查
    pub async fn start(&self, system_manager: Arc<SystemIntegrationManager>) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Err(AgentMemError::HealthCheckerAlreadyRunning);
        }
        *is_running = true;
        drop(is_running);

        let config = self.config.clone();
        let check_history = Arc::clone(&self.check_history);
        let failure_counts = Arc::clone(&self.failure_counts);
        let is_running = Arc::clone(&self.is_running);

        // 启动健康检查任务
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(config.check_interval_seconds));

            while *is_running.read().await {
                interval.tick().await;

                // 执行健康检查
                if let Err(e) = Self::perform_health_checks(
                    &system_manager,
                    &config,
                    &check_history,
                    &failure_counts,
                )
                .await
                {
                    eprintln!("健康检查失败: {e}");
                }
            }
        });

        Ok(())
    }

    /// 停止健康检查
    pub async fn stop(&self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        Ok(())
    }

    /// 获取组件健康状态
    pub async fn get_component_health(&self, component_name: &str) -> Option<HealthCheckResult> {
        let history = self.check_history.read().await;
        history.get(component_name)?.last().cloned()
    }

    /// 获取所有组件健康状态
    pub async fn get_all_health_status(&self) -> HashMap<String, HealthCheckResult> {
        let history = self.check_history.read().await;
        let mut results = HashMap::new();

        for (component, checks) in history.iter() {
            if let Some(latest) = checks.last() {
                results.insert(component.clone(), latest.clone());
            }
        }

        results
    }

    /// 获取健康检查历史
    pub async fn get_health_history(
        &self,
        component_name: &str,
        limit: Option<usize>,
    ) -> Vec<HealthCheckResult> {
        let history = self.check_history.read().await;
        if let Some(checks) = history.get(component_name) {
            let limit = limit.unwrap_or(checks.len());
            checks.iter().rev().take(limit).cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// 执行单次健康检查
    pub async fn check_component_health(
        &self,
        component_name: &str,
        system_manager: &SystemIntegrationManager,
    ) -> Result<HealthCheckResult> {
        let start_time = Instant::now();
        let check_time = chrono::Utc::now();

        let (status, error_message, metrics) = match component_name {
            "core_memory" => self.check_core_memory_health(system_manager).await,
            "resource_memory" => self.check_resource_memory_health(system_manager).await,
            "knowledge_vault" => self.check_knowledge_vault_health(system_manager).await,
            "contextual_memory" => self.check_contextual_memory_health(system_manager).await,
            "meta_memory" => self.check_meta_memory_health(system_manager).await,
            "active_retrieval" => self.check_active_retrieval_health(system_manager).await,
            _ => (
                HealthStatus::Unknown,
                Some("未知组件".to_string()),
                HashMap::new(),
            ),
        };

        let response_time_ms = start_time.elapsed().as_millis() as u64;

        let result = HealthCheckResult {
            component_name: component_name.to_string(),
            status,
            check_time,
            response_time_ms,
            error_message,
            metrics,
        };

        // 更新检查历史
        self.update_check_history(component_name, result.clone())
            .await;

        // 更新失败计数
        self.update_failure_count(component_name, &result.status)
            .await;

        Ok(result)
    }

    /// 私有方法：执行所有健康检查
    async fn perform_health_checks(
        system_manager: &SystemIntegrationManager,
        config: &HealthCheckConfig,
        check_history: &Arc<RwLock<HashMap<String, Vec<HealthCheckResult>>>>,
        failure_counts: &Arc<RwLock<HashMap<String, u32>>>,
    ) -> Result<()> {
        let components = vec![
            "core_memory",
            "resource_memory",
            "knowledge_vault",
            "contextual_memory",
            "meta_memory",
            "active_retrieval",
        ];

        for component in components {
            let checker = HealthChecker {
                config: config.clone(),
                check_history: Arc::clone(check_history),
                failure_counts: Arc::clone(failure_counts),
                is_running: Arc::new(RwLock::new(true)),
            };

            if let Err(e) = checker
                .check_component_health(component, system_manager)
                .await
            {
                eprintln!("组件 {component} 健康检查失败: {e}");
            }
        }

        Ok(())
    }

    /// 更新检查历史
    async fn update_check_history(&self, component_name: &str, result: HealthCheckResult) {
        let mut history = self.check_history.write().await;
        let component_history = history
            .entry(component_name.to_string())
            .or_insert_with(Vec::new);

        component_history.push(result);

        // 保持历史记录在合理范围内
        if component_history.len() > 100 {
            component_history.remove(0);
        }
    }

    /// 更新失败计数
    async fn update_failure_count(&self, component_name: &str, status: &HealthStatus) {
        let mut failure_counts = self.failure_counts.write().await;
        let count = failure_counts
            .entry(component_name.to_string())
            .or_insert(0);

        match status {
            HealthStatus::Healthy => *count = 0,
            HealthStatus::Warning | HealthStatus::Unhealthy => *count += 1,
            HealthStatus::Unknown => {} // 不改变计数
        }
    }

    /// 检查核心记忆健康状态
    async fn check_core_memory_health(
        &self,
        system_manager: &SystemIntegrationManager,
    ) -> (HealthStatus, Option<String>, HashMap<String, f64>) {
        match system_manager.core_memory_manager.get_stats().await {
            Ok(stats) => {
                let mut metrics = HashMap::new();
                metrics.insert(
                    "persona_blocks".to_string(),
                    stats.persona_blocks_count as f64,
                );
                metrics.insert("human_blocks".to_string(), stats.human_blocks_count as f64);
                metrics.insert(
                    "capacity_usage".to_string(),
                    stats.average_capacity_usage as f64,
                );

                let status = if stats.average_capacity_usage > 0.9 {
                    HealthStatus::Warning
                } else {
                    HealthStatus::Healthy
                };

                (status, None, metrics)
            }
            Err(e) => (HealthStatus::Unhealthy, Some(e.to_string()), HashMap::new()),
        }
    }

    /// 检查资源记忆健康状态
    async fn check_resource_memory_health(
        &self,
        system_manager: &SystemIntegrationManager,
    ) -> (HealthStatus, Option<String>, HashMap<String, f64>) {
        match system_manager.resource_memory_manager.get_stats().await {
            Ok(stats) => {
                let mut metrics = HashMap::new();
                metrics.insert("total_resources".to_string(), stats.total_resources as f64);
                metrics.insert(
                    "total_size_mb".to_string(),
                    stats.total_storage_size as f64 / 1024.0 / 1024.0,
                );
                metrics.insert(
                    "average_file_size_mb".to_string(),
                    stats.average_file_size / 1024.0 / 1024.0,
                );

                (HealthStatus::Healthy, None, metrics)
            }
            Err(e) => (HealthStatus::Unhealthy, Some(e.to_string()), HashMap::new()),
        }
    }

    /// 检查知识库健康状态
    async fn check_knowledge_vault_health(
        &self,
        system_manager: &SystemIntegrationManager,
    ) -> (HealthStatus, Option<String>, HashMap<String, f64>) {
        match system_manager.knowledge_vault_manager.get_stats() {
            Ok(stats) => {
                let mut metrics = HashMap::new();
                metrics.insert("total_entries".to_string(), stats.total_entries as f64);
                use crate::managers::knowledge_vault::SensitivityLevel;
                metrics.insert(
                    "public_entries".to_string(),
                    *stats
                        .entries_by_sensitivity
                        .get(&SensitivityLevel::Public)
                        .unwrap_or(&0) as f64,
                );
                metrics.insert(
                    "confidential_entries".to_string(),
                    *stats
                        .entries_by_sensitivity
                        .get(&SensitivityLevel::Confidential)
                        .unwrap_or(&0) as f64,
                );

                (HealthStatus::Healthy, None, metrics)
            }
            Err(e) => (HealthStatus::Unhealthy, Some(e.to_string()), HashMap::new()),
        }
    }

    /// 检查上下文记忆健康状态
    async fn check_contextual_memory_health(
        &self,
        system_manager: &SystemIntegrationManager,
    ) -> (HealthStatus, Option<String>, HashMap<String, f64>) {
        match system_manager.contextual_memory_manager.get_stats() {
            Ok(stats) => {
                let mut metrics = HashMap::new();
                metrics.insert("total_contexts".to_string(), stats.total_contexts as f64);
                metrics.insert(
                    "total_correlations".to_string(),
                    stats.total_correlations as f64,
                );
                metrics.insert("active_contexts".to_string(), stats.active_contexts as f64);
                metrics.insert("average_importance".to_string(), stats.average_importance);

                (HealthStatus::Healthy, None, metrics)
            }
            Err(e) => (HealthStatus::Unhealthy, Some(e.to_string()), HashMap::new()),
        }
    }

    /// 检查元记忆健康状态
    async fn check_meta_memory_health(
        &self,
        _system_manager: &SystemIntegrationManager,
    ) -> (HealthStatus, Option<String>, HashMap<String, f64>) {
        // 简单的健康检查
        let mut metrics = HashMap::new();
        metrics.insert("status".to_string(), 1.0);
        (HealthStatus::Healthy, None, metrics)
    }

    /// 检查主动检索健康状态
    async fn check_active_retrieval_health(
        &self,
        _system_manager: &SystemIntegrationManager,
    ) -> (HealthStatus, Option<String>, HashMap<String, f64>) {
        // 简单的健康检查
        let mut metrics = HashMap::new();
        metrics.insert("status".to_string(), 1.0);
        (HealthStatus::Healthy, None, metrics)
    }
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            check_interval_seconds: 60,
            timeout_seconds: 30,
            retry_count: 3,
            failure_threshold: 3,
            recovery_threshold: 2,
        }
    }
}
