//! Working memory service configuration

use serde::{Deserialize, Serialize};

/// Configuration for WorkingMemoryService
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingMemoryConfig {
    /// Maximum items per session (default: 100)
    pub max_items_per_session: usize,

    /// Default time-to-live in seconds (default: 300 = 5 minutes)
    pub default_ttl_seconds: i64,

    /// Cleanup interval in seconds (default: 60 = 1 minute)
    pub cleanup_interval_seconds: u64,

    /// Enable automatic expiration cleanup
    pub enable_auto_cleanup: bool,

    /// Enable EventBus integration
    pub enable_event_bus: bool,

    /// Maximum sessions (default: 10,000)
    pub max_sessions: usize,
}

impl Default for WorkingMemoryConfig {
    fn default() -> Self {
        Self {
            max_items_per_session: 100,
            default_ttl_seconds: 300,
            cleanup_interval_seconds: 60,
            enable_auto_cleanup: true,
            enable_event_bus: true,
            max_sessions: 10_000,
        }
    }
}

impl WorkingMemoryConfig {
    /// Set maximum items per session
    pub fn with_max_items(mut self, max: usize) -> Self {
        self.max_items_per_session = max;
        self
    }

    /// Set default TTL in seconds
    pub fn with_ttl(mut self, ttl_seconds: i64) -> Self {
        self.default_ttl_seconds = ttl_seconds;
        self
    }

    /// Set cleanup interval in seconds
    pub fn with_cleanup_interval(mut self, interval_seconds: u64) -> Self {
        self.cleanup_interval_seconds = interval_seconds;
        self
    }

    /// Disable automatic cleanup
    pub fn without_cleanup(mut self) -> Self {
        self.enable_auto_cleanup = false;
        self
    }

    /// Disable EventBus integration
    pub fn without_event_bus(mut self) -> Self {
        self.enable_event_bus = false;
        self
    }

    /// Set maximum sessions
    pub fn with_max_sessions(mut self, max: usize) -> Self {
        self.max_sessions = max;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = WorkingMemoryConfig::default();
        assert_eq!(config.max_items_per_session, 100);
        assert_eq!(config.default_ttl_seconds, 300);
        assert_eq!(config.cleanup_interval_seconds, 60);
        assert!(config.enable_auto_cleanup);
        assert!(config.enable_event_bus);
        assert_eq!(config.max_sessions, 10_000);
    }

    #[test]
    fn test_config_builder() {
        let config = WorkingMemoryConfig::default()
            .with_max_items(200)
            .with_ttl(600)
            .with_cleanup_interval(120)
            .without_cleanup()
            .without_event_bus()
            .with_max_sessions(5000);

        assert_eq!(config.max_items_per_session, 200);
        assert_eq!(config.default_ttl_seconds, 600);
        assert_eq!(config.cleanup_interval_seconds, 120);
        assert!(!config.enable_auto_cleanup);
        assert!(!config.enable_event_bus);
        assert_eq!(config.max_sessions, 5000);
    }
}
