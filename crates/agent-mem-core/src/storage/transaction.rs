//! Transaction support and retry mechanisms
//!
//! This module provides transaction management and retry logic inspired by MIRIX's
//! retry_db_operation decorator.

use sqlx::{PgPool, Postgres, Transaction};
use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;

use crate::{CoreError, CoreResult};

/// Transaction manager for database operations
pub struct TransactionManager {
    pool: PgPool,
}

impl TransactionManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Begin a new transaction
    pub async fn begin(&self) -> CoreResult<Transaction<'static, Postgres>> {
        self.pool
            .begin()
            .await
            .map_err(|e| CoreError::Database(format!("Failed to begin transaction: {}", e)))
    }

    /// Execute a function within a transaction with automatic retry
    ///
    /// This is inspired by MIRIX's retry_db_operation decorator.
    /// It will retry the operation up to max_retries times with exponential backoff.
    pub async fn execute_with_retry<F, Fut, T>(
        &self,
        max_retries: u32,
        base_delay_ms: u64,
        max_delay_ms: u64,
        backoff_factor: f64,
        operation: F,
    ) -> CoreResult<T>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = CoreResult<T>>,
    {
        let mut last_error = None;

        for attempt in 0..=max_retries {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e.clone());

                    // Check if this is a retryable error
                    if !is_retryable_error(&e) {
                        return Err(e);
                    }

                    // Don't sleep after the last attempt
                    if attempt < max_retries {
                        let delay = calculate_backoff_delay(
                            attempt,
                            base_delay_ms,
                            max_delay_ms,
                            backoff_factor,
                        );

                        tracing::warn!(
                            "Database operation failed (attempt {}/{}), retrying in {}ms: {}",
                            attempt + 1,
                            max_retries + 1,
                            delay,
                            e
                        );

                        sleep(Duration::from_millis(delay)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            CoreError::Database("Operation failed after all retries".to_string())
        }))
    }

    /// Execute a function within a transaction
    pub async fn execute_in_transaction<F, Fut, T>(&self, operation: F) -> CoreResult<T>
    where
        F: FnOnce(Transaction<'static, Postgres>) -> Fut,
        Fut: Future<Output = CoreResult<(Transaction<'static, Postgres>, T)>>,
    {
        let tx = self.begin().await?;

        match operation(tx).await {
            Ok((tx, result)) => {
                tx.commit().await.map_err(|e| {
                    CoreError::Database(format!("Failed to commit transaction: {}", e))
                })?;
                Ok(result)
            }
            Err(e) => {
                // Transaction will be rolled back when dropped
                Err(e)
            }
        }
    }
}

/// Check if an error is retryable
fn is_retryable_error(error: &CoreError) -> bool {
    match error {
        CoreError::Database(msg) => {
            let msg_lower = msg.to_lowercase();
            // Check for database locked errors (similar to MIRIX)
            msg_lower.contains("database is locked")
                || msg_lower.contains("database locked")
                || msg_lower.contains("could not obtain lock")
                || msg_lower.contains("busy")
                || msg_lower.contains("locked")
                || msg_lower.contains("deadlock")
                || msg_lower.contains("connection")
                || msg_lower.contains("timeout")
        }
        _ => false,
    }
}

/// Calculate exponential backoff delay
fn calculate_backoff_delay(
    attempt: u32,
    base_delay_ms: u64,
    max_delay_ms: u64,
    backoff_factor: f64,
) -> u64 {
    let delay = (base_delay_ms as f64) * backoff_factor.powi(attempt as i32);
    delay.min(max_delay_ms as f64) as u64
}

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 100,
            max_delay_ms: 2000,
            backoff_factor: 2.0,
        }
    }
}

impl RetryConfig {
    /// Create a new retry configuration
    pub fn new(
        max_retries: u32,
        base_delay_ms: u64,
        max_delay_ms: u64,
        backoff_factor: f64,
    ) -> Self {
        Self {
            max_retries,
            base_delay_ms,
            max_delay_ms,
            backoff_factor,
        }
    }

    /// Create a configuration for aggressive retries (more attempts, shorter delays)
    pub fn aggressive() -> Self {
        Self {
            max_retries: 5,
            base_delay_ms: 50,
            max_delay_ms: 1000,
            backoff_factor: 1.5,
        }
    }

    /// Create a configuration for conservative retries (fewer attempts, longer delays)
    pub fn conservative() -> Self {
        Self {
            max_retries: 2,
            base_delay_ms: 200,
            max_delay_ms: 5000,
            backoff_factor: 3.0,
        }
    }
}

/// Retry a database operation with the given configuration
pub async fn retry_operation<F, Fut, T>(config: RetryConfig, operation: F) -> CoreResult<T>
where
    F: Fn() -> Fut,
    Fut: Future<Output = CoreResult<T>>,
{
    let mut last_error = None;

    for attempt in 0..=config.max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e.clone());

                // Check if this is a retryable error
                if !is_retryable_error(&e) {
                    return Err(e);
                }

                // Don't sleep after the last attempt
                if attempt < config.max_retries {
                    let delay = calculate_backoff_delay(
                        attempt,
                        config.base_delay_ms,
                        config.max_delay_ms,
                        config.backoff_factor,
                    );

                    tracing::warn!(
                        "Database operation failed (attempt {}/{}), retrying in {}ms: {}",
                        attempt + 1,
                        config.max_retries + 1,
                        delay,
                        e
                    );

                    sleep(Duration::from_millis(delay)).await;
                }
            }
        }
    }

    Err(last_error
        .unwrap_or_else(|| CoreError::Database("Operation failed after all retries".to_string())))
}

/// Macro to retry a database operation with default configuration
#[macro_export]
macro_rules! retry_db {
    ($operation:expr) => {
        $crate::storage::transaction::retry_operation(
            $crate::storage::transaction::RetryConfig::default(),
            || async { $operation },
        )
        .await
    };
}

/// Macro to retry a database operation with custom configuration
#[macro_export]
macro_rules! retry_db_with_config {
    ($config:expr, $operation:expr) => {
        $crate::storage::transaction::retry_operation($config, || async { $operation }).await
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_backoff_delay() {
        // Test exponential backoff
        assert_eq!(calculate_backoff_delay(0, 100, 2000, 2.0), 100);
        assert_eq!(calculate_backoff_delay(1, 100, 2000, 2.0), 200);
        assert_eq!(calculate_backoff_delay(2, 100, 2000, 2.0), 400);
        assert_eq!(calculate_backoff_delay(3, 100, 2000, 2.0), 800);
        assert_eq!(calculate_backoff_delay(4, 100, 2000, 2.0), 1600);

        // Test max delay cap
        assert_eq!(calculate_backoff_delay(5, 100, 2000, 2.0), 2000);
        assert_eq!(calculate_backoff_delay(10, 100, 2000, 2.0), 2000);
    }

    #[test]
    fn test_is_retryable_error() {
        // Retryable errors
        assert!(is_retryable_error(&CoreError::Database(
            "database is locked".to_string()
        )));
        assert!(is_retryable_error(&CoreError::Database(
            "could not obtain lock".to_string()
        )));
        assert!(is_retryable_error(&CoreError::Database(
            "connection timeout".to_string()
        )));

        // Non-retryable errors
        assert!(!is_retryable_error(&CoreError::ValidationError(
            "invalid input".to_string()
        )));
        assert!(!is_retryable_error(&CoreError::Database(
            "unique constraint violation".to_string()
        )));
    }

    #[test]
    fn test_retry_config() {
        let default_config = RetryConfig::default();
        assert_eq!(default_config.max_retries, 3);
        assert_eq!(default_config.base_delay_ms, 100);

        let aggressive_config = RetryConfig::aggressive();
        assert_eq!(aggressive_config.max_retries, 5);
        assert_eq!(aggressive_config.base_delay_ms, 50);

        let conservative_config = RetryConfig::conservative();
        assert_eq!(conservative_config.max_retries, 2);
        assert_eq!(conservative_config.base_delay_ms, 200);
    }
}
