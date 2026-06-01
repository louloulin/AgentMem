//! ProactiveTask definitions

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::config::TaskSchedule;

/// Unique task identifier
pub type TaskId = String;

/// Represents the type of proactive task
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProactiveTask {
    /// Auto-categorize new memory items
    AutoCategorize,
    /// Deduplicate and merge similar memories
    DedupeMerge,
    /// Generate summaries for categories
    GenerateSummaries,
    /// Optimize search indices
    IndexOptimization,
    /// Archive old resources
    ResourceArchival,
    /// Health check for memory system
    HealthCheck,
    /// Custom task
    Custom(String),
}

impl ProactiveTask {
    /// Get the display name for the task
    pub fn display_name(&self) -> &str {
        match self {
            ProactiveTask::AutoCategorize => "Auto Categorize",
            ProactiveTask::DedupeMerge => "Dedupe Merge",
            ProactiveTask::GenerateSummaries => "Generate Summaries",
            ProactiveTask::IndexOptimization => "Index Optimization",
            ProactiveTask::ResourceArchival => "Resource Archival",
            ProactiveTask::HealthCheck => "Health Check",
            ProactiveTask::Custom(name) => name,
        }
    }

    /// Get default interval for the task (in minutes)
    pub fn default_interval_minutes(&self) -> Option<u64> {
        match self {
            ProactiveTask::AutoCategorize => None, // Event-driven
            ProactiveTask::DedupeMerge => Some(5),
            ProactiveTask::GenerateSummaries => Some(60), // Once per hour
            ProactiveTask::IndexOptimization => Some(1440), // Once per day
            ProactiveTask::ResourceArchival => Some(10080), // Once per week
            ProactiveTask::HealthCheck => Some(5),
            ProactiveTask::Custom(_) => None,
        }
    }

    /// Check if this task is CPU-intensive
    pub fn is_cpu_intensive(&self) -> bool {
        matches!(
            self,
            ProactiveTask::DedupeMerge
                | ProactiveTask::GenerateSummaries
                | ProactiveTask::IndexOptimization
        )
    }

    /// Check if this task should be gated by the batch window.
    pub fn is_batch_task(&self) -> bool {
        matches!(
            self,
            ProactiveTask::DedupeMerge
                | ProactiveTask::GenerateSummaries
                | ProactiveTask::IndexOptimization
                | ProactiveTask::ResourceArchival
        )
    }
}

/// Status of a proactive task
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// Task is pending (scheduled but not started)
    Pending,
    /// Task is currently running
    Running,
    /// Task completed successfully
    Completed,
    /// Task failed
    Failed,
    /// Task was cancelled
    Cancelled,
    /// Task is disabled
    Disabled,
}

/// Result of task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// Task ID
    pub task_id: TaskId,
    /// Task type
    pub task_type: ProactiveTask,
    /// Execution status
    pub status: TaskStatus,
    /// Number of items processed
    pub items_processed: u64,
    /// Number of items affected (e.g., categorized, merged)
    pub items_affected: u64,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
    /// Timestamp when task started
    pub started_at: DateTime<Utc>,
    /// Timestamp when task completed
    pub completed_at: DateTime<Utc>,
}

impl TaskResult {
    /// Create a new task result
    pub fn new(task_id: TaskId, task_type: ProactiveTask, started_at: DateTime<Utc>) -> Self {
        Self {
            task_id,
            task_type,
            status: TaskStatus::Running,
            items_processed: 0,
            items_affected: 0,
            error_message: None,
            duration_ms: 0,
            started_at,
            completed_at: started_at,
        }
    }

    /// Mark task as completed
    pub fn completed(&mut self, items_processed: u64, items_affected: u64) {
        self.status = TaskStatus::Completed;
        self.items_processed = items_processed;
        self.items_affected = items_affected;
        self.completed_at = Utc::now();
        self.duration_ms = (self.completed_at - self.started_at).num_milliseconds() as u64;
    }

    /// Mark task as failed
    pub fn failed(&mut self, error: String) {
        self.status = TaskStatus::Failed;
        self.error_message = Some(error);
        self.completed_at = Utc::now();
        self.duration_ms = (self.completed_at - self.started_at).num_milliseconds() as u64;
    }
}

/// Context passed to task executors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecutionContext {
    /// User ID for the task
    pub user_id: String,
    /// Agent ID (optional)
    pub agent_id: Option<String>,
    /// Task-specific configuration
    pub config: TaskConfig,
    /// Maximum CPU usage (0-100)
    pub max_cpu_percent: u8,
    /// Maximum memory usage in MB
    pub max_memory_mb: u64,
    /// Whether to run in dry-run mode
    pub dry_run: bool,
}

/// Configuration for task execution
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskConfig {
    /// For AutoCategorize: categories to process (None = all)
    pub categories: Option<Vec<String>>,
    /// For DedupeMerge: similarity threshold (0.0-1.0)
    pub similarity_threshold: Option<f32>,
    /// For GenerateSummaries: categories to update (None = all stale)
    pub stale_categories_only: Option<bool>,
    /// For IndexOptimization: force rebuild
    pub force_rebuild: Option<bool>,
    /// For ResourceArchival: age threshold in days
    pub age_threshold_days: Option<u32>,
    /// Custom parameters as JSON
    pub custom: Option<serde_json::Value>,
}

impl TaskConfig {
    /// Create default config
    pub fn new() -> Self {
        Self::default()
    }

    /// Create config for auto-categorize
    pub fn auto_categorize(categories: Option<Vec<String>>) -> Self {
        Self {
            categories,
            ..Default::default()
        }
    }

    /// Create config for dedupe merge
    pub fn dedupe_merge(similarity_threshold: f32) -> Self {
        Self {
            similarity_threshold: Some(similarity_threshold),
            ..Default::default()
        }
    }

    /// Create config for generate summaries
    pub fn generate_summaries(stale_only: bool) -> Self {
        Self {
            stale_categories_only: Some(stale_only),
            ..Default::default()
        }
    }
}

/// A scheduled task instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    /// Unique task instance ID
    pub id: TaskId,
    /// The task type
    pub task_type: ProactiveTask,
    /// Current status
    pub status: TaskStatus,
    /// Cron expression or interval
    pub schedule: String,
    /// Full schedule configuration
    pub schedule_config: TaskSchedule,
    /// Whether task is enabled
    pub enabled: bool,
    /// Queued runs waiting to be dispatched
    pub pending_runs: u32,
    /// Number of currently running executions
    pub running_count: u32,
    /// Last execution result
    pub last_result: Option<TaskResult>,
    /// Next scheduled run time
    pub next_run: Option<DateTime<Utc>>,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Updated at
    pub updated_at: DateTime<Utc>,
}

impl ScheduledTask {
    /// Create a new scheduled task
    pub fn new(task_type: ProactiveTask, schedule: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            task_type,
            status: TaskStatus::Pending,
            schedule,
            schedule_config: TaskSchedule::manual(),
            enabled: true,
            pending_runs: 0,
            running_count: 0,
            last_result: None,
            next_run: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a new scheduled task from a structured schedule.
    pub fn from_schedule(task_type: ProactiveTask, schedule_config: TaskSchedule) -> Self {
        let mut task = Self::new(task_type, schedule_config.schedule_string());
        task.schedule_config = schedule_config;
        task
    }

    /// Mark as disabled
    pub fn disable(&mut self) {
        self.enabled = false;
        self.status = TaskStatus::Disabled;
        self.updated_at = Utc::now();
    }

    /// Mark as enabled
    pub fn enable(&mut self) {
        self.enabled = true;
        self.status = TaskStatus::Pending;
        self.updated_at = Utc::now();
    }

    /// Queue an event-driven run.
    pub fn queue_run(&mut self) {
        self.pending_runs = self.pending_runs.saturating_add(1);
        self.updated_at = Utc::now();
    }

    /// Mark a run as dispatched.
    pub fn mark_running(&mut self) {
        self.running_count = self.running_count.saturating_add(1);
        self.status = TaskStatus::Running;
        self.updated_at = Utc::now();
    }

    /// Mark a run as cancelled.
    pub fn mark_cancelled(&mut self) {
        self.running_count = self.running_count.saturating_sub(1);
        self.pending_runs = 0;
        self.status = TaskStatus::Cancelled;
        self.updated_at = Utc::now();
    }

    /// Mark a run as completed or failed.
    pub fn mark_finished(&mut self, result: TaskResult) {
        self.running_count = self.running_count.saturating_sub(1);
        self.status = result.status.clone();
        self.last_result = Some(result);
        self.updated_at = Utc::now();
    }

    /// Check whether the task can start another execution.
    pub fn can_start(&self) -> bool {
        self.enabled && self.running_count < self.schedule_config.max_concurrent.max(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_display_name() {
        assert_eq!(
            ProactiveTask::AutoCategorize.display_name(),
            "Auto Categorize"
        );
        assert_eq!(ProactiveTask::DedupeMerge.display_name(), "Dedupe Merge");
    }

    #[test]
    fn test_task_default_interval() {
        assert_eq!(
            ProactiveTask::DedupeMerge.default_interval_minutes(),
            Some(5)
        );
        assert_eq!(
            ProactiveTask::GenerateSummaries.default_interval_minutes(),
            Some(60)
        );
        assert_eq!(
            ProactiveTask::AutoCategorize.default_interval_minutes(),
            None
        );
    }

    #[test]
    fn test_task_result() {
        let started = Utc::now();
        let mut result = TaskResult::new("test-1".to_string(), ProactiveTask::DedupeMerge, started);

        result.completed(100, 50);

        assert_eq!(result.status, TaskStatus::Completed);
        assert_eq!(result.items_processed, 100);
        assert_eq!(result.items_affected, 50);
        assert!(result.error_message.is_none());
    }

    #[test]
    fn test_scheduled_task() {
        let task = ScheduledTask::new(ProactiveTask::HealthCheck, "*/5 * * * *".to_string());

        assert!(task.enabled);
        assert_eq!(task.status, TaskStatus::Pending);
    }

    #[test]
    fn test_scheduled_task_from_schedule() {
        let task = ScheduledTask::from_schedule(
            ProactiveTask::AutoCategorize,
            TaskSchedule::event().with_max_concurrent(2),
        );

        assert_eq!(task.schedule, "event");
        assert_eq!(task.schedule_config.max_concurrent, 2);
        assert!(task.can_start());
    }
}
