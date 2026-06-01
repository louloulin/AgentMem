//! Scheduler models and state

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::task::{ScheduledTask, TaskId, TaskStatus};

/// State of the task scheduler
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SchedulerState {
    /// Scheduler is stopped
    Stopped,
    /// Scheduler is starting
    Starting,
    /// Scheduler is running
    Running,
    /// Scheduler is stopping
    Stopping,
    /// Scheduler encountered an error
    Error(String),
}

impl Default for SchedulerState {
    fn default() -> Self {
        SchedulerState::Stopped
    }
}

/// Scheduler statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SchedulerStats {
    /// Total tasks scheduled
    pub total_tasks: u64,
    /// Tasks running currently
    pub running_tasks: u64,
    /// Tasks completed successfully
    pub completed_tasks: u64,
    /// Tasks failed
    pub failed_tasks: u64,
    /// Tasks cancelled
    pub cancelled_tasks: u64,
    /// Total execution time in milliseconds
    pub total_execution_time_ms: u64,
    /// Last error message
    pub last_error: Option<String>,
}

impl SchedulerStats {
    /// Increment running tasks.
    pub fn record_start(&mut self) {
        self.running_tasks += 1;
    }

    /// Increment completed tasks
    pub fn record_completion(&mut self, duration_ms: u64) {
        self.running_tasks = self.running_tasks.saturating_sub(1);
        self.completed_tasks += 1;
        self.total_execution_time_ms += duration_ms;
    }

    /// Increment failed tasks
    pub fn record_failure(&mut self, error: String) {
        self.running_tasks = self.running_tasks.saturating_sub(1);
        self.failed_tasks += 1;
        self.last_error = Some(error);
    }

    /// Increment cancelled tasks
    pub fn record_cancellation(&mut self) {
        self.running_tasks = self.running_tasks.saturating_sub(1);
        self.cancelled_tasks += 1;
    }
}

/// Internal scheduler state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerStateInner {
    /// Current state
    pub state: SchedulerState,
    /// All registered tasks
    pub tasks: HashMap<TaskId, ScheduledTask>,
    /// Statistics
    pub stats: SchedulerStats,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl Default for SchedulerStateInner {
    fn default() -> Self {
        Self {
            state: SchedulerState::default(),
            tasks: HashMap::new(),
            stats: SchedulerStats::default(),
            updated_at: Utc::now(),
        }
    }
}

impl SchedulerStateInner {
    /// Create new scheduler state
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a task
    pub fn add_task(&mut self, task: ScheduledTask) -> Option<ScheduledTask> {
        self.stats.total_tasks += 1;
        self.tasks.insert(task.id.clone(), task)
    }

    /// Remove a task
    pub fn remove_task(&mut self, task_id: &str) -> Option<ScheduledTask> {
        self.tasks.remove(task_id)
    }

    /// Get a task
    pub fn get_task(&self, task_id: &str) -> Option<&ScheduledTask> {
        self.tasks.get(task_id)
    }

    /// Get a task mutable
    pub fn get_task_mut(&mut self, task_id: &str) -> Option<&mut ScheduledTask> {
        self.tasks.get_mut(task_id)
    }

    /// List all tasks
    pub fn list_tasks(&self) -> Vec<&ScheduledTask> {
        self.tasks.values().collect()
    }

    /// List enabled tasks
    pub fn enabled_tasks(&self) -> Vec<&ScheduledTask> {
        self.tasks
            .values()
            .filter(|t| t.enabled && t.status != TaskStatus::Disabled)
            .collect()
    }

    /// Update task status
    pub fn update_task_status(&mut self, task_id: &str, status: TaskStatus) {
        if let Some(task) = self.tasks.get_mut(task_id) {
            task.status = status;
            task.updated_at = Utc::now();
        }
    }

    /// Set scheduler state
    pub fn set_state(&mut self, state: SchedulerState) {
        self.state = state;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ProactiveTask;

    #[test]
    fn test_scheduler_state() {
        let state = SchedulerStateInner::new();
        assert_eq!(state.state, SchedulerState::Stopped);
        assert!(state.tasks.is_empty());
    }

    #[test]
    fn test_add_task() {
        let mut state = SchedulerStateInner::new();
        let task = ScheduledTask::new(ProactiveTask::HealthCheck, "*/5 * * * *".to_string());

        let old = state.add_task(task.clone());
        assert!(old.is_none());
        assert_eq!(state.tasks.len(), 1);
        assert_eq!(state.stats.total_tasks, 1);
    }

    #[test]
    fn test_remove_task() {
        let mut state = SchedulerStateInner::new();
        let task = ScheduledTask::new(ProactiveTask::HealthCheck, "*/5 * * * *".to_string());
        let task_id = task.id.clone();

        state.add_task(task);
        let removed = state.remove_task(&task_id);
        assert!(removed.is_some());
        assert!(state.tasks.is_empty());
    }

    #[test]
    fn test_enabled_tasks() {
        let mut state = SchedulerStateInner::new();

        let mut task1 = ScheduledTask::new(ProactiveTask::HealthCheck, "*/5 * * * *".to_string());
        task1.enable();

        let mut task2 = ScheduledTask::new(ProactiveTask::DedupeMerge, "*/10 * * * *".to_string());
        task2.disable();

        state.add_task(task1);
        state.add_task(task2);

        let enabled = state.enabled_tasks();
        assert_eq!(enabled.len(), 1);
    }
}
