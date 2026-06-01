//! ProactiveAgent facade and lifecycle orchestration.

use std::str::FromStr;

use crate::error::Result;
use crate::executors::{
    AutoCategorizeExecutor, DedupeMergeExecutor, GenerateSummariesExecutor, HealthCheckExecutor,
    IndexOptimizationExecutor, ResourceArchivalExecutor,
};
use crate::models::{
    ProactiveConfig, ProactiveTask, ScheduledTask, SchedulerState, SchedulerStats, TaskResult,
    TaskSchedule, TaskScheduleConfig,
};
use crate::scheduler::TaskScheduler;

/// High-level facade for bootstrapping and operating background proactive work.
pub struct ProactiveAgent {
    config: ProactiveConfig,
    scheduler: TaskScheduler,
}

impl ProactiveAgent {
    /// Create a new proactive agent from configuration.
    pub fn new(config: ProactiveConfig) -> Self {
        let scheduler = TaskScheduler::new(config.clone());
        Self { config, scheduler }
    }

    /// Create a proactive agent with production defaults.
    pub fn with_default_config() -> Self {
        Self::new(ProactiveConfig::default())
    }

    /// Access the agent configuration.
    pub fn config(&self) -> &ProactiveConfig {
        &self.config
    }

    /// Access the underlying scheduler.
    pub fn scheduler(&self) -> &TaskScheduler {
        &self.scheduler
    }

    /// Register the built-in task executors.
    pub async fn register_default_executors(&self) {
        self.scheduler
            .register_executor(AutoCategorizeExecutor::default())
            .await;
        self.scheduler
            .register_executor(DedupeMergeExecutor::default())
            .await;
        self.scheduler
            .register_executor(GenerateSummariesExecutor::default())
            .await;
        self.scheduler
            .register_executor(IndexOptimizationExecutor::default())
            .await;
        self.scheduler
            .register_executor(ResourceArchivalExecutor::default())
            .await;
        self.scheduler
            .register_executor(HealthCheckExecutor::default())
            .await;
    }

    /// Bootstrap the agent with executors and task schedules.
    pub async fn initialize(&self) -> Result<()> {
        self.register_default_executors().await;

        if self.config.task_schedules.is_empty() {
            self.scheduler.add_default_tasks().await?;
        } else {
            for task_config in &self.config.task_schedules {
                self.ensure_task_from_config(task_config).await?;
            }
        }

        Ok(())
    }

    /// Start the proactive background loop.
    pub async fn start(&mut self) -> Result<()> {
        self.scheduler.start().await
    }

    /// Stop the proactive background loop.
    pub async fn stop(&mut self) -> Result<()> {
        self.scheduler.stop().await
    }

    /// List all registered scheduled tasks.
    pub async fn list_tasks(&self) -> Vec<ScheduledTask> {
        self.scheduler.list_tasks().await
    }

    /// Get scheduler state.
    pub async fn state(&self) -> SchedulerState {
        self.scheduler.state().await
    }

    /// Get scheduler statistics.
    pub async fn stats(&self) -> SchedulerStats {
        self.scheduler.stats().await
    }

    /// Run a scheduled task immediately.
    pub async fn run_task_now(&self, task_id: &str) -> Result<TaskResult> {
        self.scheduler.run_task_now(task_id).await
    }

    /// Trigger an event-driven task.
    pub async fn trigger_task(&self, task_id: &str) -> Result<()> {
        self.scheduler.trigger_task(task_id).await
    }

    /// Cancel a queued or running background task.
    pub async fn cancel_task(&self, task_id: &str) -> Result<()> {
        self.scheduler.cancel_task(task_id).await
    }

    async fn ensure_task_from_config(&self, task_config: &TaskScheduleConfig) -> Result<()> {
        let task_type = ProactiveTask::from_str(&task_config.task_type)?;
        let existing = self
            .scheduler
            .list_tasks()
            .await
            .into_iter()
            .find(|task| task.task_type == task_type);

        let task_id = if let Some(task) = existing {
            task.id
        } else {
            self.schedule_task(task_type, task_config.schedule.clone())
                .await?
        };

        if task_config.enabled {
            self.scheduler.enable_task(&task_id).await?;
        } else {
            self.scheduler.disable_task(&task_id).await?;
        }

        Ok(())
    }

    async fn schedule_task(
        &self,
        task_type: ProactiveTask,
        schedule: TaskSchedule,
    ) -> Result<String> {
        self.scheduler.schedule_task(task_type, schedule).await
    }
}

impl Default for ProactiveAgent {
    fn default() -> Self {
        Self::with_default_config()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TaskStatus;
    use crate::ProactiveError;

    #[tokio::test]
    async fn test_initialize_with_default_tasks() {
        let agent = ProactiveAgent::with_default_config();
        agent.initialize().await.unwrap();

        let tasks = agent.list_tasks().await;
        assert_eq!(tasks.len(), 6);
        assert!(tasks
            .iter()
            .any(|task| task.task_type == ProactiveTask::AutoCategorize));
        assert!(tasks
            .iter()
            .any(|task| task.task_type == ProactiveTask::DedupeMerge));
        assert!(tasks
            .iter()
            .any(|task| task.task_type == ProactiveTask::GenerateSummaries));
        assert!(tasks
            .iter()
            .any(|task| task.task_type == ProactiveTask::IndexOptimization));
        assert!(tasks
            .iter()
            .any(|task| task.task_type == ProactiveTask::ResourceArchival));
        assert!(tasks
            .iter()
            .any(|task| task.task_type == ProactiveTask::HealthCheck));
    }

    #[tokio::test]
    async fn test_initialize_with_configured_tasks() {
        let mut config = ProactiveConfig::test();
        config.task_schedules = vec![
            TaskScheduleConfig::new("health_check", TaskSchedule::interval(5)),
            TaskScheduleConfig::new("index_optimization", TaskSchedule::manual()).disabled(),
        ];

        let agent = ProactiveAgent::new(config);
        agent.initialize().await.unwrap();

        let tasks = agent.list_tasks().await;
        assert_eq!(tasks.len(), 2);

        let health_check = tasks
            .iter()
            .find(|task| task.task_type == ProactiveTask::HealthCheck)
            .unwrap();
        assert!(health_check.enabled);

        let index_optimization = tasks
            .iter()
            .find(|task| task.task_type == ProactiveTask::IndexOptimization)
            .unwrap();
        assert_eq!(index_optimization.status, TaskStatus::Disabled);
        assert!(!index_optimization.enabled);
    }

    #[tokio::test]
    async fn test_run_task_now_with_registered_executor() {
        let mut config = ProactiveConfig::test();
        config.task_schedules = vec![TaskScheduleConfig::new(
            "health_check",
            TaskSchedule::manual(),
        )];

        let agent = ProactiveAgent::new(config);
        agent.initialize().await.unwrap();

        let task_id = agent.list_tasks().await[0].id.clone();
        let result = agent.run_task_now(&task_id).await.unwrap();

        assert_eq!(result.task_type, ProactiveTask::HealthCheck);
    }

    #[tokio::test]
    async fn test_initialize_rejects_unknown_task_type() {
        let mut config = ProactiveConfig::test();
        config.task_schedules = vec![TaskScheduleConfig::new(
            "unknown_task",
            TaskSchedule::manual(),
        )];

        let agent = ProactiveAgent::new(config);
        let error = agent.initialize().await.unwrap_err();

        assert!(matches!(error, ProactiveError::InvalidConfig(_)));
    }

    #[tokio::test]
    async fn test_trigger_task_forwards_to_scheduler() {
        let mut config = ProactiveConfig::test();
        config.task_schedules = vec![TaskScheduleConfig::new(
            "auto_categorize",
            TaskSchedule::event(),
        )];

        let agent = ProactiveAgent::new(config);
        agent.initialize().await.unwrap();

        let task_id = agent.list_tasks().await[0].id.clone();
        agent.trigger_task(&task_id).await.unwrap();

        let task = agent.scheduler().get_task(&task_id).await.unwrap();
        assert!(task.pending_runs <= 1);
    }
}
