//! Health check executor
//!
//! Performs periodic health checks on the memory system.

use async_trait::async_trait;
use chrono::Utc;
use tracing::{info, warn};

use crate::error::{ProactiveError, Result};
use crate::models::{ProactiveTask, TaskExecutionContext, TaskResult, TaskStatus};
use crate::scheduler::TaskExecutor;

/// Health check executor
///
/// Performs system health checks including:
/// - Memory usage
/// - Database connectivity
/// - Index integrity
/// - Task queue status
pub struct HealthCheckExecutor {
    /// Whether to check database connectivity
    check_database: bool,
    /// Whether to check index integrity
    check_indexes: bool,
    /// Whether to check task queue
    check_task_queue: bool,
}

impl HealthCheckExecutor {
    /// Create a new health check executor
    pub fn new() -> Self {
        Self {
            check_database: true,
            check_indexes: true,
            check_task_queue: true,
        }
    }

    /// Create with custom configuration
    pub fn with_config(check_database: bool, check_indexes: bool, check_task_queue: bool) -> Self {
        Self {
            check_database,
            check_indexes,
            check_task_queue,
        }
    }

    /// Perform the health check
    async fn perform_check(&self, context: &TaskExecutionContext) -> Result<TaskResult> {
        let started_at = Utc::now();
        let task_id = format!("health-check-{}", started_at.timestamp());

        info!("Starting health check...");

        let mut items_checked: u64 = 0;
        let mut items_healthy: u64 = 0;
        let mut issues: Vec<String> = Vec::new();

        // Check 1: Memory usage
        if let Ok(memory_info) = self.check_memory_usage().await {
            items_checked += 1;
            if memory_info.healthy {
                items_healthy += 1;
            } else {
                issues.push(memory_info.message);
            }
        }

        // Check 2: Database connectivity (if enabled)
        if self.check_database {
            if let Ok(db_health) = self.check_database_connectivity().await {
                items_checked += 1;
                if db_health.healthy {
                    items_healthy += 1;
                } else {
                    issues.push(db_health.message);
                }
            }
        }

        // Check 3: Index integrity (if enabled)
        if self.check_indexes {
            if let Ok(index_health) = self.check_index_integrity().await {
                items_checked += 1;
                if index_health.healthy {
                    items_healthy += 1;
                } else {
                    issues.push(index_health.message);
                }
            }
        }

        // Check 4: Task queue (if enabled)
        if self.check_task_queue {
            if let Ok(queue_health) = self.check_task_queue_status().await {
                items_checked += 1;
                if queue_health.healthy {
                    items_healthy += 1;
                } else {
                    issues.push(queue_health.message);
                }
            }
        }

        let mut result = TaskResult::new(task_id, ProactiveTask::HealthCheck, started_at);

        if issues.is_empty() {
            result.completed(items_checked, items_healthy);
            info!(
                "Health check completed: {}/{} checks healthy",
                items_healthy, items_checked
            );
        } else {
            result.completed(items_checked, items_healthy);
            for issue in &issues {
                warn!("Health issue detected: {}", issue);
            }
        }

        Ok(result)
    }

    /// Check memory usage
    async fn check_memory_usage(&self) -> Result<HealthStatus> {
        // TODO: Integrate with actual memory monitoring
        // For now, return a mock healthy status
        Ok(HealthStatus {
            healthy: true,
            message: "Memory usage within limits".to_string(),
        })
    }

    /// Check database connectivity
    async fn check_database_connectivity(&self) -> Result<HealthStatus> {
        // TODO: Integrate with actual database health check
        // For now, return a mock healthy status
        Ok(HealthStatus {
            healthy: true,
            message: "Database connectivity OK".to_string(),
        })
    }

    /// Check index integrity
    async fn check_index_integrity(&self) -> Result<HealthStatus> {
        // TODO: Integrate with actual index health check
        // For now, return a mock healthy status
        Ok(HealthStatus {
            healthy: true,
            message: "Index integrity OK".to_string(),
        })
    }

    /// Check task queue status
    async fn check_task_queue_status(&self) -> Result<HealthStatus> {
        // TODO: Integrate with actual task queue health check
        // For now, return a mock healthy status
        Ok(HealthStatus {
            healthy: true,
            message: "Task queue OK".to_string(),
        })
    }
}

impl Default for HealthCheckExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TaskExecutor for HealthCheckExecutor {
    fn task_type(&self) -> ProactiveTask {
        ProactiveTask::HealthCheck
    }

    async fn execute(&self, context: &TaskExecutionContext) -> Result<TaskResult> {
        self.perform_check(context).await
    }
}

/// Health status result
struct HealthStatus {
    healthy: bool,
    message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check_executor() {
        let executor = HealthCheckExecutor::new();
        let context = TaskExecutionContext {
            user_id: "system".to_string(),
            agent_id: None,
            config: Default::default(),
            max_cpu_percent: 5,
            max_memory_mb: 512,
            dry_run: false,
        };

        let result = executor.execute(&context).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.status, TaskStatus::Completed);
    }

    #[test]
    fn test_health_check_task_type() {
        let executor = HealthCheckExecutor::new();
        assert_eq!(executor.task_type(), ProactiveTask::HealthCheck);
    }
}
