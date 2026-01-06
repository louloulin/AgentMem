//! Batch operations trait definitions for Mem5

use crate::{BatchResult, EnhancedAddRequest, HealthStatus, MemorySearchResult, Result};
use async_trait::async_trait;
use std::collections::HashMap;

/// Trait for batch memory operations (Mem5 enhancement)
#[async_trait]
pub trait BatchMemoryOperations: Send + Sync {
    /// Add multiple memories in batch
    async fn add_batch(&self, requests: Vec<EnhancedAddRequest>) -> Result<BatchResult>;

    /// Update multiple memories in batch
    async fn update_batch(&self, updates: Vec<MemoryUpdate>) -> Result<BatchResult>;

    /// Delete multiple memories in batch
    async fn delete_batch(&self, ids: Vec<String>) -> Result<BatchResult>;

    /// Search multiple queries in batch
    async fn search_batch(&self, queries: Vec<String>) -> Result<Vec<Vec<MemorySearchResult>>>;
}

/// Memory update request for batch operations
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryUpdate {
    pub id: String,
    pub content: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub importance: Option<f64>,
}

impl MemoryUpdate {
    pub fn new(id: String) -> Self {
        Self {
            id,
            content: None,
            metadata: None,
            importance: None,
        }
    }

    pub fn with_content(mut self, content: String) -> Self {
        self.content = Some(content);
        self
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn with_importance(mut self, importance: f64) -> Self {
        self.importance = Some(importance);
        self
    }

    /// Validate the update request
    pub fn validate(&self) -> Result<()> {
        if self.id.trim().is_empty() {
            return Err(crate::AgentMemError::ValidationError(
                "Empty memory ID".to_string(),
            ));
        }

        if let Some(content) = &self.content {
            if content.trim().is_empty() {
                return Err(crate::AgentMemError::ValidationError(
                    "Empty content".to_string(),
                ));
            }
        }

        Ok(())
    }
}

/// Trait for performance monitoring and health checks
#[async_trait]
pub trait HealthCheckProvider: Send + Sync {
    /// Check the health status of the component
    async fn health_check(&self) -> Result<HealthStatus>;

    /// Get component metrics
    async fn get_metrics(&self) -> Result<HashMap<String, serde_json::Value>>;

    /// Reset component state (for testing/maintenance)
    async fn reset(&self) -> Result<()>;
}

/// Trait for retry and error recovery operations
#[async_trait]
pub trait RetryableOperations: Send + Sync {
    /// Execute operation with retry logic
    async fn execute_with_retry<T, F>(&self, operation: F, max_retries: usize) -> Result<T>
    where
        F: Fn() -> futures::future::BoxFuture<'static, Result<T>> + Send + Sync;

    /// Check if an error is retryable
    fn is_retryable_error(&self, error: &crate::AgentMemError) -> bool;

    /// Calculate retry delay with exponential backoff
    fn calculate_retry_delay(&self, attempt: usize) -> std::time::Duration;
}

/// Trait for advanced search with filtering
#[async_trait]
pub trait AdvancedSearch: Send + Sync {
    /// Search with complex filters and thresholds
    async fn search_with_filters(
        &self,
        query: &str,
        filters: HashMap<String, serde_json::Value>,
        limit: usize,
        threshold: Option<f32>,
    ) -> Result<Vec<MemorySearchResult>>;

    /// Search similar memories to a given memory ID
    async fn search_similar(
        &self,
        memory_id: &str,
        limit: usize,
    ) -> Result<Vec<MemorySearchResult>>;

    /// Get memory recommendations based on context
    async fn get_recommendations(
        &self,
        context: &str,
        user_id: Option<&str>,
        limit: usize,
    ) -> Result<Vec<MemorySearchResult>>;
}

/// Trait for telemetry and monitoring
#[async_trait]
pub trait TelemetryProvider: Send + Sync {
    /// Track an operation with timing
    async fn track_operation(&self, operation: &str, duration: std::time::Duration, success: bool);

    /// Increment a counter metric
    async fn increment_counter(&self, metric: &str, value: u64);

    /// Record a gauge metric
    async fn record_gauge(&self, metric: &str, value: f64);

    /// Record a histogram metric
    async fn record_histogram(&self, metric: &str, value: f64);

    /// Get current metrics snapshot
    async fn get_metrics_snapshot(&self) -> Result<HashMap<String, serde_json::Value>>;
}

/// Configuration trait for dynamic configuration management
#[async_trait]
pub trait ConfigurationProvider: Send + Sync {
    /// Get configuration value by key
    async fn get_config(&self, key: &str) -> Result<Option<serde_json::Value>>;

    /// Set configuration value
    async fn set_config(&self, key: &str, value: serde_json::Value) -> Result<()>;

    /// Delete configuration key
    async fn delete_config(&self, key: &str) -> Result<()>;

    /// List all configuration keys with optional prefix
    async fn list_config_keys(&self, prefix: Option<&str>) -> Result<Vec<String>>;

    /// Watch for configuration changes
    async fn watch_config(
        &self,
        key: &str,
    ) -> Result<std::sync::mpsc::Receiver<Option<serde_json::Value>>>;
}

/// Trait for memory lifecycle management
#[async_trait]
pub trait MemoryLifecycle: Send + Sync {
    /// Mark memory as accessed
    async fn mark_accessed(&self, memory_id: &str) -> Result<()>;

    /// Archive old memories based on criteria
    async fn archive_memories(&self, criteria: ArchiveCriteria) -> Result<Vec<String>>;

    /// Restore archived memories
    async fn restore_memories(&self, memory_ids: Vec<String>) -> Result<Vec<String>>;

    /// Permanently delete archived memories
    async fn purge_memories(&self, memory_ids: Vec<String>) -> Result<Vec<String>>;

    /// Get memory statistics
    async fn get_memory_stats(&self) -> Result<MemoryStats>;
}

/// Criteria for archiving memories
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArchiveCriteria {
    pub older_than_days: Option<u32>,
    pub access_count_less_than: Option<u32>,
    pub importance_less_than: Option<f64>,
    pub user_id: Option<String>,
    pub memory_type: Option<crate::MemoryType>,
}

impl Default for ArchiveCriteria {
    fn default() -> Self {
        Self {
            older_than_days: Some(365), // Default: archive memories older than 1 year
            access_count_less_than: Some(1), // Default: archive memories accessed less than once
            importance_less_than: Some(0.3), // Default: archive low importance memories
            user_id: None,
            memory_type: None,
        }
    }
}

/// Memory statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryStats {
    pub total_memories: u64,
    pub active_memories: u64,
    pub archived_memories: u64,
    pub total_size_bytes: u64,
    pub average_importance: f64,
    pub most_accessed_memory_id: Option<String>,
    pub oldest_memory_date: Option<chrono::DateTime<chrono::Utc>>,
    pub newest_memory_date: Option<chrono::DateTime<chrono::Utc>>,
}
