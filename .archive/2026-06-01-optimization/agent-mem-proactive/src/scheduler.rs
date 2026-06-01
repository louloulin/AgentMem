//! TaskScheduler implementation

use async_trait::async_trait;
use chrono::{DateTime, NaiveTime, Timelike, Utc};
use croner::Cron;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot, RwLock};
use tokio::time::interval;
use tracing::{error, info, warn};

use crate::error::{ProactiveError, Result};
use crate::models::{
    ProactiveConfig, ProactiveTask, ScheduledTask, SchedulerState, SchedulerStateInner,
    SchedulerStats, TaskConfig, TaskExecutionContext, TaskId, TaskResult, TaskSchedule,
    TriggerType,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DispatchKind {
    Scheduled,
    Startup,
    Manual,
}

/// Task executor trait - implemented by different task types
#[async_trait]
pub trait TaskExecutor: Send + Sync {
    /// Get the task type this executor handles
    fn task_type(&self) -> ProactiveTask;

    /// Execute the task
    async fn execute(&self, context: &TaskExecutionContext) -> Result<TaskResult>;
}

/// Main scheduler for proactive tasks
pub struct TaskScheduler {
    /// Internal state
    state: Arc<RwLock<SchedulerStateInner>>,
    /// Configuration
    config: ProactiveConfig,
    /// Task executors (protected by RwLock for thread-safety)
    executors: Arc<RwLock<HashMap<String, Arc<dyn TaskExecutor>>>>,
    /// Cancellation channels for running background tasks
    cancellation_txs: Arc<RwLock<HashMap<String, oneshot::Sender<()>>>>,
    /// Shutdown signal sender
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl TaskScheduler {
    /// Create a new scheduler
    pub fn new(config: ProactiveConfig) -> Self {
        Self {
            state: Arc::new(RwLock::new(SchedulerStateInner::new())),
            config,
            executors: Arc::new(RwLock::new(HashMap::new())),
            cancellation_txs: Arc::new(RwLock::new(HashMap::new())),
            shutdown_tx: None,
        }
    }

    /// Create scheduler with default config
    pub fn with_default_config() -> Self {
        Self::new(ProactiveConfig::default())
    }

    /// Register a task executor
    pub async fn register_executor<E: TaskExecutor + 'static>(&self, executor: E) {
        let task_type = executor.task_type().to_string();
        info!("Registering executor for task type: {}", task_type);
        self.executors
            .write()
            .await
            .insert(task_type, Arc::new(executor));
    }

    /// Get scheduler state
    pub async fn state(&self) -> SchedulerState {
        self.state.read().await.state.clone()
    }

    /// Get scheduler stats
    pub async fn stats(&self) -> SchedulerStats {
        self.state.read().await.stats.clone()
    }

    /// Get all tasks
    pub async fn list_tasks(&self) -> Vec<ScheduledTask> {
        self.state
            .read()
            .await
            .list_tasks()
            .into_iter()
            .cloned()
            .collect()
    }

    /// Get task by ID
    pub async fn get_task(&self, task_id: &str) -> Option<ScheduledTask> {
        self.state.read().await.get_task(task_id).cloned()
    }

    /// Schedule a new task
    pub async fn schedule_task(
        &self,
        task_type: ProactiveTask,
        schedule: TaskSchedule,
    ) -> Result<TaskId> {
        let mut task = ScheduledTask::from_schedule(task_type.clone(), schedule.clone());
        task.next_run = self.calculate_next_run(&schedule, Utc::now())?;

        // Validate task type has executor
        let task_type_str = task_type.to_string();
        if !self.executors.read().await.contains_key(&task_type_str) {
            warn!("No executor registered for task type: {}", task_type_str);
        }

        let task_id = task.id.clone();
        self.state.write().await.add_task(task);

        info!(
            "Scheduled task {} with ID {}",
            task_type.display_name(),
            task_id
        );
        Ok(task_id)
    }

    /// Unschedule a task
    pub async fn unschedule_task(&self, task_id: &str) -> Result<()> {
        let task = self
            .state
            .write()
            .await
            .remove_task(task_id)
            .ok_or_else(|| ProactiveError::TaskNotFound(task_id.to_string()))?;

        info!("Unscheduled task: {}", task.task_type.display_name());
        Ok(())
    }

    /// Enable a task
    pub async fn enable_task(&self, task_id: &str) -> Result<()> {
        let mut state = self.state.write().await;
        let task = state
            .get_task_mut(task_id)
            .ok_or_else(|| ProactiveError::TaskNotFound(task_id.to_string()))?;

        task.enable();
        info!("Enabled task: {}", task.task_type.display_name());
        Ok(())
    }

    /// Disable a task
    pub async fn disable_task(&self, task_id: &str) -> Result<()> {
        let mut state = self.state.write().await;
        let task = state
            .get_task_mut(task_id)
            .ok_or_else(|| ProactiveError::TaskNotFound(task_id.to_string()))?;

        task.disable();
        info!("Disabled task: {}", task.task_type.display_name());
        Ok(())
    }

    /// Trigger an event-driven task.
    pub async fn trigger_task(&self, task_id: &str) -> Result<()> {
        {
            let mut state = self.state.write().await;
            let task = state
                .get_task_mut(task_id)
                .ok_or_else(|| ProactiveError::TaskNotFound(task_id.to_string()))?;

            if task.schedule_config.trigger_type != TriggerType::Event {
                return Err(ProactiveError::InvalidConfig(format!(
                    "Task {} is not event-driven",
                    task.task_type.display_name()
                )));
            }

            if !task.enabled {
                return Err(ProactiveError::TaskExecution(format!(
                    "Task {} is disabled",
                    task.task_type.display_name()
                )));
            }

            task.queue_run();
        }

        self.dispatch_if_due(task_id).await
    }

    /// Cancel a running or queued background task.
    pub async fn cancel_task(&self, task_id: &str) -> Result<()> {
        let cancel_tx = self.cancellation_txs.write().await.remove(task_id);
        if let Some(cancel_tx) = cancel_tx {
            cancel_tx.send(()).map_err(|_| {
                ProactiveError::TaskExecution(format!("Task {} could not be cancelled", task_id))
            })?;
            return Ok(());
        }

        let mut state = self.state.write().await;
        let cancelled_pending = {
            let task = state
                .get_task_mut(task_id)
                .ok_or_else(|| ProactiveError::TaskNotFound(task_id.to_string()))?;

            if task.pending_runs > 0 {
                task.mark_cancelled();
                true
            } else {
                false
            }
        };

        if cancelled_pending {
            state.stats.record_cancellation();
            return Ok(());
        }

        Err(ProactiveError::TaskExecution(format!(
            "Task {} is not running",
            task_id
        )))
    }

    /// Run a task immediately
    pub async fn run_task_now(&self, task_id: &str) -> Result<TaskResult> {
        let task = self
            .prepare_task_for_dispatch(task_id, DispatchKind::Manual)
            .await?
            .ok_or_else(|| {
                ProactiveError::TaskExecution(format!("Task {} cannot start right now", task_id))
            })?;

        self.execute_task(&task, None).await
    }

    /// Execute a task
    async fn execute_task(
        &self,
        task: &ScheduledTask,
        override_config: Option<TaskConfig>,
    ) -> Result<TaskResult> {
        let task_type_str = task.task_type.to_string();
        let executor = {
            let executors = self.executors.read().await;
            executors.get(&task_type_str).cloned().ok_or_else(|| {
                ProactiveError::TaskExecution(format!(
                    "No executor for task type: {}",
                    task_type_str
                ))
            })?
        };

        let context = TaskExecutionContext {
            user_id: "system".to_string(), // TODO: Make configurable
            agent_id: None,
            config: override_config.unwrap_or_default(),
            max_cpu_percent: self.config.default_cpu_limit,
            max_memory_mb: self.config.default_memory_limit_mb,
            dry_run: false,
        };

        let result = executor.execute(&context).await;
        let completed_at = Utc::now();

        let (task_result, error_msg) = match result {
            Ok(task_result) => (task_result, None),
            Err(err) => {
                let mut task_result =
                    TaskResult::new(task.id.clone(), task.task_type.clone(), task.updated_at);
                task_result.failed(err.to_string());
                let error_msg = task_result.error_message.clone();
                self.finish_task_execution(task, task_result.clone(), error_msg.clone())
                    .await;
                return Err(err);
            }
        };

        let mut final_result = task_result;
        final_result.completed_at = completed_at;
        self.finish_task_execution(task, final_result.clone(), error_msg)
            .await;

        Ok(final_result)
    }

    /// Start the scheduler - runs tasks on their schedules
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting task scheduler...");

        {
            let mut state = self.state.write().await;
            state.set_state(SchedulerState::Starting);
        }

        let (tx, mut rx) = mpsc::channel::<()>(1);
        self.shutdown_tx = Some(tx);

        {
            let mut state = self.state.write().await;
            state.set_state(SchedulerState::Running);
        }

        self.dispatch_startup_tasks().await?;

        info!("Task scheduler started successfully");

        let mut tick_interval = interval(Duration::from_secs(30));

        loop {
            tokio::select! {
                _ = rx.recv() => {
                    info!("Shutdown signal received, stopping scheduler");
                    break;
                }
                _ = tick_interval.tick() => {
                    if let Err(err) = self.check_and_execute_tasks().await {
                        error!("Failed to dispatch scheduled tasks: {}", err);
                    }
                }
            }
        }

        {
            let mut state = self.state.write().await;
            state.set_state(SchedulerState::Stopped);
        }

        info!("Task scheduler stopped");
        Ok(())
    }

    /// Stop the scheduler
    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping task scheduler...");

        {
            let mut state = self.state.write().await;
            state.set_state(SchedulerState::Stopping);
        }

        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }

        let senders = {
            let mut cancellations = self.cancellation_txs.write().await;
            std::mem::take(&mut *cancellations)
        };
        for (_, sender) in senders {
            let _ = sender.send(());
        }

        {
            let mut state = self.state.write().await;
            state.set_state(SchedulerState::Stopped);
        }

        info!("Task scheduler stopped");
        Ok(())
    }

    /// Add default task schedules based on config
    pub async fn add_default_tasks(&self) -> Result<()> {
        let default_tasks = [
            ProactiveTask::AutoCategorize,
            ProactiveTask::DedupeMerge,
            ProactiveTask::GenerateSummaries,
            ProactiveTask::IndexOptimization,
            ProactiveTask::ResourceArchival,
            ProactiveTask::HealthCheck,
        ];

        for task_type in default_tasks {
            let schedule = match task_type.default_interval_minutes() {
                Some(minutes) => TaskSchedule::interval(minutes),
                None => TaskSchedule::event(),
            };
            self.schedule_task(task_type, schedule).await?;
        }

        Ok(())
    }

    async fn check_and_execute_tasks(&self) -> Result<()> {
        let tasks = self.list_tasks().await;

        for task in tasks {
            let dispatch_count = self.ready_dispatch_count(&task);
            for _ in 0..dispatch_count {
                self.spawn_task_execution(task.id.clone(), DispatchKind::Scheduled)
                    .await?;
            }
        }

        Ok(())
    }

    async fn dispatch_startup_tasks(&self) -> Result<()> {
        let tasks = self.list_tasks().await;
        for task in tasks
            .into_iter()
            .filter(|task| task.enabled && task.schedule_config.run_on_startup)
        {
            self.spawn_task_execution(task.id, DispatchKind::Startup)
                .await?;
        }

        Ok(())
    }

    async fn dispatch_if_due(&self, task_id: &str) -> Result<()> {
        let Some(task) = self.get_task(task_id).await else {
            return Ok(());
        };

        let dispatch_count = self.ready_dispatch_count(&task);
        for _ in 0..dispatch_count {
            self.spawn_task_execution(task.id.clone(), DispatchKind::Scheduled)
                .await?;
        }

        Ok(())
    }

    async fn spawn_task_execution(
        &self,
        task_id: String,
        dispatch_kind: DispatchKind,
    ) -> Result<()> {
        let Some(task) = self
            .prepare_task_for_dispatch(&task_id, dispatch_kind)
            .await?
        else {
            return Ok(());
        };

        let (cancel_tx, mut cancel_rx) = oneshot::channel();
        self.cancellation_txs
            .write()
            .await
            .insert(task_id.clone(), cancel_tx);

        let scheduler = self.clone_inner();
        tokio::spawn(async move {
            let execution = scheduler.execute_task(&task, None);
            tokio::pin!(execution);

            tokio::select! {
                _ = &mut cancel_rx => {
                    scheduler.mark_task_cancelled(&task_id).await;
                    info!("Cancelled task {}", task_id);
                }
                result = &mut execution => {
                    if let Err(err) = result {
                        error!("Task {} failed: {}", task_id, err);
                    }
                }
            }

            scheduler.cancellation_txs.write().await.remove(&task_id);
        });

        Ok(())
    }

    async fn prepare_task_for_dispatch(
        &self,
        task_id: &str,
        dispatch_kind: DispatchKind,
    ) -> Result<Option<ScheduledTask>> {
        let now = Utc::now();
        let mut state = self.state.write().await;
        let task_snapshot = {
            let task = state
                .get_task_mut(task_id)
                .ok_or_else(|| ProactiveError::TaskNotFound(task_id.to_string()))?;

            let max_concurrent = task.schedule_config.max_concurrent.max(1);
            if task.running_count >= max_concurrent {
                return Ok(None);
            }

            match dispatch_kind {
                DispatchKind::Scheduled => match task.schedule_config.trigger_type {
                    TriggerType::Event => {
                        if !task.enabled || task.pending_runs == 0 {
                            return Ok(None);
                        }
                        task.pending_runs -= 1;
                    }
                    TriggerType::Interval | TriggerType::Cron => {
                        if !task.enabled {
                            return Ok(None);
                        }
                        task.next_run = self.calculate_next_run(&task.schedule_config, now)?;
                    }
                    TriggerType::Manual => return Ok(None),
                },
                DispatchKind::Startup => {
                    if !task.enabled {
                        return Ok(None);
                    }

                    if matches!(
                        task.schedule_config.trigger_type,
                        TriggerType::Interval | TriggerType::Cron
                    ) {
                        task.next_run = self.calculate_next_run(&task.schedule_config, now)?;
                    }
                }
                DispatchKind::Manual => {}
            }

            task.mark_running();
            task.clone()
        };
        state.stats.record_start();

        Ok(Some(task_snapshot))
    }

    async fn finish_task_execution(
        &self,
        task: &ScheduledTask,
        task_result: TaskResult,
        error_msg: Option<String>,
    ) {
        let mut state = self.state.write().await;
        if let Some(stored_task) = state.get_task_mut(&task.id) {
            stored_task.mark_finished(task_result.clone());
        }

        if let Some(error_msg) = error_msg {
            state.stats.record_failure(error_msg);
        } else {
            state.stats.record_completion(task_result.duration_ms);
        }
    }

    async fn mark_task_cancelled(&self, task_id: &str) {
        let mut state = self.state.write().await;
        let found = if let Some(task) = state.get_task_mut(task_id) {
            task.mark_cancelled();
            true
        } else {
            false
        };

        if found {
            state.stats.record_cancellation();
        }
    }

    fn ready_dispatch_count(&self, task: &ScheduledTask) -> u32 {
        if !task.enabled {
            return 0;
        }

        let available_slots = task
            .schedule_config
            .max_concurrent
            .max(1)
            .saturating_sub(task.running_count);
        if available_slots == 0 {
            return 0;
        }

        match task.schedule_config.trigger_type {
            TriggerType::Event => task.pending_runs.min(available_slots),
            TriggerType::Manual => 0,
            TriggerType::Interval | TriggerType::Cron => {
                let due = task
                    .next_run
                    .map(|next_run| next_run <= Utc::now())
                    .unwrap_or(false);
                let batch_ready =
                    !task.task_type.is_batch_task() || self.is_in_batch_window(Utc::now());

                if due && batch_ready {
                    1
                } else {
                    0
                }
            }
        }
    }

    fn calculate_next_run(
        &self,
        schedule: &TaskSchedule,
        from: DateTime<Utc>,
    ) -> Result<Option<DateTime<Utc>>> {
        match schedule.trigger_type {
            TriggerType::Interval => Ok(schedule
                .interval_minutes
                .map(|minutes| from + chrono::TimeDelta::minutes(minutes as i64))),
            TriggerType::Cron => {
                let cron_expr = schedule.cron.as_deref().ok_or_else(|| {
                    ProactiveError::InvalidConfig(
                        "Cron trigger missing cron expression".to_string(),
                    )
                })?;
                let mut cron = Cron::new(cron_expr);
                let cron = cron.parse().map_err(|err| {
                    ProactiveError::InvalidConfig(format!(
                        "Invalid cron expression `{}`: {}",
                        cron_expr, err
                    ))
                })?;

                cron.find_next_occurrence(&from, false)
                    .map(Some)
                    .map_err(|err| {
                        ProactiveError::InvalidConfig(format!(
                            "Failed to compute next cron occurrence for `{}`: {}",
                            cron_expr, err
                        ))
                    })
            }
            TriggerType::Event | TriggerType::Manual => Ok(None),
        }
    }

    fn is_in_batch_window(&self, now: DateTime<Utc>) -> bool {
        let Some(window) = self.config.batch_window.as_deref() else {
            return true;
        };

        let Some((start, end)) = Self::parse_batch_window(window) else {
            warn!("Ignoring invalid batch window `{}`", window);
            return true;
        };

        let current =
            NaiveTime::from_hms_opt(now.hour(), now.minute(), 0).unwrap_or_else(|| now.time());

        if start <= end {
            current >= start && current <= end
        } else {
            current >= start || current <= end
        }
    }

    fn parse_batch_window(window: &str) -> Option<(NaiveTime, NaiveTime)> {
        let (start, end) = window.split_once('-')?;
        let start = NaiveTime::parse_from_str(start, "%H:%M").ok()?;
        let end = NaiveTime::parse_from_str(end, "%H:%M").ok()?;
        Some((start, end))
    }

    /// Clone the scheduler for use in spawned tasks
    fn clone_inner(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            config: self.config.clone(),
            executors: Arc::clone(&self.executors),
            cancellation_txs: Arc::clone(&self.cancellation_txs),
            shutdown_tx: None,
        }
    }
}

/// Extension for ProactiveTask to convert to string
impl std::fmt::Display for ProactiveTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProactiveTask::AutoCategorize => write!(f, "auto_categorize"),
            ProactiveTask::DedupeMerge => write!(f, "dedupe_merge"),
            ProactiveTask::GenerateSummaries => write!(f, "generate_summaries"),
            ProactiveTask::IndexOptimization => write!(f, "index_optimization"),
            ProactiveTask::ResourceArchival => write!(f, "resource_archival"),
            ProactiveTask::HealthCheck => write!(f, "health_check"),
            ProactiveTask::Custom(name) => write!(f, "custom:{}", name),
        }
    }
}

impl std::str::FromStr for ProactiveTask {
    type Err = ProactiveError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "auto_categorize" => Ok(ProactiveTask::AutoCategorize),
            "dedupe_merge" => Ok(ProactiveTask::DedupeMerge),
            "generate_summaries" => Ok(ProactiveTask::GenerateSummaries),
            "index_optimization" => Ok(ProactiveTask::IndexOptimization),
            "resource_archival" => Ok(ProactiveTask::ResourceArchival),
            "health_check" => Ok(ProactiveTask::HealthCheck),
            s if s.starts_with("custom:") => Ok(ProactiveTask::Custom(s[7..].to_string())),
            _ => Err(ProactiveError::InvalidConfig(format!(
                "Unknown task type: {}",
                s
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TaskStatus;
    use chrono::TimeDelta;
    use tokio::time::sleep;

    struct MockExecutor {
        task_type: ProactiveTask,
        delay_ms: u64,
    }

    #[async_trait]
    impl TaskExecutor for MockExecutor {
        fn task_type(&self) -> ProactiveTask {
            self.task_type.clone()
        }

        async fn execute(&self, _context: &TaskExecutionContext) -> Result<TaskResult> {
            if self.delay_ms > 0 {
                sleep(Duration::from_millis(self.delay_ms)).await;
            }

            let started_at = Utc::now();
            let mut result =
                TaskResult::new("test-1".to_string(), self.task_type.clone(), started_at);
            result.completed(1, 1);
            Ok(result)
        }
    }

    #[tokio::test]
    async fn test_scheduler_creation() {
        let scheduler = TaskScheduler::with_default_config();
        assert_eq!(scheduler.state().await, SchedulerState::Stopped);
    }

    #[tokio::test]
    async fn test_schedule_task() {
        let scheduler = TaskScheduler::with_default_config();
        let task_id = scheduler
            .schedule_task(ProactiveTask::HealthCheck, TaskSchedule::interval(5))
            .await
            .unwrap();

        let task = scheduler.get_task(&task_id).await.unwrap();
        assert_eq!(task.schedule, "interval:5min");
        assert!(task.next_run.is_some());
    }

    #[tokio::test]
    async fn test_schedule_task_with_cron_sets_next_run() {
        let scheduler = TaskScheduler::with_default_config();
        let task_id = scheduler
            .schedule_task(
                ProactiveTask::HealthCheck,
                TaskSchedule::cron("*/5 * * * *"),
            )
            .await
            .unwrap();

        let task = scheduler.get_task(&task_id).await.unwrap();
        assert!(task.next_run.is_some());
    }

    #[tokio::test]
    async fn test_enable_disable_task() {
        let scheduler = TaskScheduler::with_default_config();
        let task_id = scheduler
            .schedule_task(ProactiveTask::HealthCheck, TaskSchedule::interval(5))
            .await
            .unwrap();

        scheduler.disable_task(&task_id).await.unwrap();
        let task = scheduler.get_task(&task_id).await.unwrap();
        assert!(!task.enabled);

        scheduler.enable_task(&task_id).await.unwrap();
        let task = scheduler.get_task(&task_id).await.unwrap();
        assert!(task.enabled);
    }

    #[tokio::test]
    async fn test_unschedule_task() {
        let scheduler = TaskScheduler::with_default_config();
        let task_id = scheduler
            .schedule_task(ProactiveTask::HealthCheck, TaskSchedule::interval(5))
            .await
            .unwrap();

        scheduler.unschedule_task(&task_id).await.unwrap();
        let task = scheduler.get_task(&task_id).await;
        assert!(task.is_none());
    }

    #[tokio::test]
    async fn test_trigger_task_executes_event_task() {
        let scheduler = TaskScheduler::with_default_config();
        scheduler
            .register_executor(MockExecutor {
                task_type: ProactiveTask::AutoCategorize,
                delay_ms: 0,
            })
            .await;

        let task_id = scheduler
            .schedule_task(ProactiveTask::AutoCategorize, TaskSchedule::event())
            .await
            .unwrap();

        scheduler.trigger_task(&task_id).await.unwrap();
        sleep(Duration::from_millis(50)).await;

        let task = scheduler.get_task(&task_id).await.unwrap();
        assert_eq!(task.status, TaskStatus::Completed);
        assert_eq!(task.pending_runs, 0);
    }

    #[tokio::test]
    async fn test_cancel_task_cancels_running_execution() {
        let scheduler = TaskScheduler::with_default_config();
        scheduler
            .register_executor(MockExecutor {
                task_type: ProactiveTask::AutoCategorize,
                delay_ms: 250,
            })
            .await;

        let task_id = scheduler
            .schedule_task(ProactiveTask::AutoCategorize, TaskSchedule::event())
            .await
            .unwrap();

        scheduler.trigger_task(&task_id).await.unwrap();
        sleep(Duration::from_millis(25)).await;
        scheduler.cancel_task(&task_id).await.unwrap();
        sleep(Duration::from_millis(25)).await;

        let task = scheduler.get_task(&task_id).await.unwrap();
        assert_eq!(task.status, TaskStatus::Cancelled);
        assert_eq!(scheduler.stats().await.cancelled_tasks, 1);
    }

    #[tokio::test]
    async fn test_batch_window_blocks_due_batch_task() {
        let mut config = ProactiveConfig::test();
        let next_hour = (Utc::now().hour() + 1) % 24;
        let after_next_hour = (next_hour + 1) % 24;
        config.batch_window = Some(format!("{:02}:00-{:02}:00", next_hour, after_next_hour));

        let scheduler = TaskScheduler::new(config);
        let task_id = scheduler
            .schedule_task(ProactiveTask::DedupeMerge, TaskSchedule::interval(1))
            .await
            .unwrap();

        {
            let mut state = scheduler.state.write().await;
            let task = state.get_task_mut(&task_id).unwrap();
            task.next_run = Some(Utc::now() - TimeDelta::minutes(1));
        }

        let task = scheduler.get_task(&task_id).await.unwrap();
        assert_eq!(scheduler.ready_dispatch_count(&task), 0);
    }
}
