//! Models for ProactiveAgent

pub mod config;
pub mod scheduler;
pub mod task;

// Re-export from task module
pub use task::{
    ProactiveTask, ScheduledTask, TaskConfig, TaskExecutionContext, TaskId, TaskResult, TaskStatus,
};

// Re-export from scheduler module
pub use scheduler::{SchedulerState, SchedulerStateInner, SchedulerStats};

// Re-export from config module
pub use config::{ProactiveConfig, RetryConfig, TaskSchedule, TaskScheduleConfig, TriggerType};
