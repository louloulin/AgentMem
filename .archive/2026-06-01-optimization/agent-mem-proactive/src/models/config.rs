//! Configuration for ProactiveAgent

use chrono::Duration;
use serde::{Deserialize, Serialize};

/// Trigger type for proactive tasks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TriggerType {
    /// Timer-based trigger (cron expression)
    Cron,
    /// Interval-based trigger (every N minutes)
    Interval,
    /// Event-based trigger (e.g., new memory added)
    Event,
    /// Manual trigger (on-demand)
    Manual,
}

/// Task schedule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSchedule {
    /// Trigger type
    pub trigger_type: TriggerType,
    /// Cron expression (for Cron trigger)
    pub cron: Option<String>,
    /// Interval in minutes (for Interval trigger)
    pub interval_minutes: Option<u64>,
    /// Whether to run on startup
    pub run_on_startup: bool,
    /// Maximum concurrent executions
    pub max_concurrent: u32,
    /// Retry configuration
    pub retry: Option<RetryConfig>,
}

impl TaskSchedule {
    /// Create a cron-based schedule
    pub fn cron(cron_expr: &str) -> Self {
        Self {
            trigger_type: TriggerType::Cron,
            cron: Some(cron_expr.to_string()),
            interval_minutes: None,
            run_on_startup: false,
            max_concurrent: 1,
            retry: None,
        }
    }

    /// Create an interval-based schedule
    pub fn interval(minutes: u64) -> Self {
        Self {
            trigger_type: TriggerType::Interval,
            cron: None,
            interval_minutes: Some(minutes),
            run_on_startup: false,
            max_concurrent: 1,
            retry: None,
        }
    }

    /// Create an event-based schedule
    pub fn event() -> Self {
        Self {
            trigger_type: TriggerType::Event,
            cron: None,
            interval_minutes: None,
            run_on_startup: false,
            max_concurrent: 1,
            retry: None,
        }
    }

    /// Create a manual (on-demand) schedule
    pub fn manual() -> Self {
        Self {
            trigger_type: TriggerType::Manual,
            cron: None,
            interval_minutes: None,
            run_on_startup: false,
            max_concurrent: 1,
            retry: None,
        }
    }

    /// Enable run on startup
    pub fn with_run_on_startup(mut self, run: bool) -> Self {
        self.run_on_startup = run;
        self
    }

    /// Set max concurrent executions
    pub fn with_max_concurrent(mut self, max: u32) -> Self {
        self.max_concurrent = max;
        self
    }

    /// Set retry configuration
    pub fn with_retry(mut self, retry: RetryConfig) -> Self {
        self.retry = Some(retry);
        self
    }

    /// Render a stable schedule string for display/debugging.
    pub fn schedule_string(&self) -> String {
        match self.trigger_type {
            TriggerType::Cron => self.cron.clone().unwrap_or_else(|| "* * * * *".to_string()),
            TriggerType::Interval => {
                format!("interval:{}min", self.interval_minutes.unwrap_or(60))
            }
            TriggerType::Event => "event".to_string(),
            TriggerType::Manual => "manual".to_string(),
        }
    }
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retries
    pub max_retries: u32,
    /// Initial backoff in seconds
    pub initial_backoff_secs: u64,
    /// Maximum backoff in seconds
    pub max_backoff_secs: u64,
    /// Backoff multiplier
    pub multiplier: f64,
}

impl RetryConfig {
    /// Create default retry config (3 retries, exponential backoff)
    pub fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff_secs: 1,
            max_backoff_secs: 60,
            multiplier: 2.0,
        }
    }

    /// Calculate backoff for given attempt
    pub fn calculate_backoff(&self, attempt: u32) -> Duration {
        let backoff = self.initial_backoff_secs as f64 * self.multiplier.powi(attempt as i32);
        let backoff = backoff.min(self.max_backoff_secs as f64);
        Duration::seconds(backoff as i64)
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self::default()
    }
}

/// Configuration for ProactiveAgent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProactiveConfig {
    /// Whether the agent is enabled
    pub enabled: bool,
    /// Default CPU limit per task (0-100)
    pub default_cpu_limit: u8,
    /// Default memory limit per task in MB
    pub default_memory_limit_mb: u64,
    /// Maximum total CPU usage (0-100)
    pub max_total_cpu: u8,
    /// Maximum total memory usage in MB
    pub max_total_memory_mb: u64,
    /// Task-specific schedules
    pub task_schedules: Vec<TaskScheduleConfig>,
    /// Time window for batch processing (e.g., "02:00-04:00")
    pub batch_window: Option<String>,
    /// Timezone for scheduling
    pub timezone: String,
    /// Health check interval in seconds
    pub health_check_interval_secs: u64,
}

impl Default for ProactiveConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_cpu_limit: 5, // <5% CPU overhead as per requirement
            default_memory_limit_mb: 512,
            max_total_cpu: 20, // Max 20% total CPU
            max_total_memory_mb: 2048,
            task_schedules: Vec::new(),
            batch_window: Some("02:00-04:00".to_string()), // Late night batch window
            timezone: "UTC".to_string(),
            health_check_interval_secs: 60,
        }
    }
}

impl ProactiveConfig {
    /// Create production config
    pub fn production() -> Self {
        Self::default()
    }

    /// Create development config
    pub fn development() -> Self {
        Self {
            enabled: true,
            default_cpu_limit: 10,
            default_memory_limit_mb: 1024,
            max_total_cpu: 30,
            max_total_memory_mb: 4096,
            task_schedules: Vec::new(),
            batch_window: None, // Run immediately in dev
            timezone: "UTC".to_string(),
            health_check_interval_secs: 30,
        }
    }

    /// Create test config
    pub fn test() -> Self {
        Self {
            enabled: true,
            default_cpu_limit: 50,
            default_memory_limit_mb: 1024,
            max_total_cpu: 80,
            max_total_memory_mb: 4096,
            task_schedules: Vec::new(),
            batch_window: None,
            timezone: "UTC".to_string(),
            health_check_interval_secs: 10,
        }
    }

    /// Get schedule for a specific task type
    pub fn get_schedule(&self, task_type: &str) -> Option<&TaskScheduleConfig> {
        self.task_schedules
            .iter()
            .find(|s| s.task_type == task_type)
    }
}

/// Task-specific schedule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskScheduleConfig {
    /// Task type identifier
    pub task_type: String,
    /// Schedule configuration
    pub schedule: TaskSchedule,
    /// Whether task is enabled
    pub enabled: bool,
    /// Priority (higher = more important)
    pub priority: u8,
}

impl TaskScheduleConfig {
    /// Create a new task schedule config
    pub fn new(task_type: &str, schedule: TaskSchedule) -> Self {
        Self {
            task_type: task_type.to_string(),
            schedule,
            enabled: true,
            priority: 50, // Default priority
        }
    }

    /// Disable this task
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_schedule_cron() {
        let schedule = TaskSchedule::cron("*/5 * * * *");
        assert_eq!(schedule.trigger_type, TriggerType::Cron);
        assert_eq!(schedule.cron, Some("*/5 * * * *".to_string()));
    }

    #[test]
    fn test_task_schedule_interval() {
        let schedule = TaskSchedule::interval(30);
        assert_eq!(schedule.trigger_type, TriggerType::Interval);
        assert_eq!(schedule.interval_minutes, Some(30));
    }

    #[test]
    fn test_retry_config() {
        let retry = RetryConfig::default();
        assert_eq!(retry.max_retries, 3);

        let backoff1 = retry.calculate_backoff(0);
        let backoff2 = retry.calculate_backoff(1);
        let backoff3 = retry.calculate_backoff(2);

        assert!(backoff2 > backoff1);
        assert!(backoff3 > backoff2);
    }

    #[test]
    fn test_proactive_config_default() {
        let config = ProactiveConfig::default();
        assert!(config.enabled);
        assert_eq!(config.default_cpu_limit, 5);
        assert!(config.batch_window.is_some());
    }
}
