//! Retry logic for HTTP requests

use crate::error::{ClientError, ClientResult};
use backoff::{backoff::Backoff, ExponentialBackoff};
use std::time::Duration;
use tracing::{debug, warn};

/// Retry policy configuration
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    pub max_retries: u32,

    /// Base delay for exponential backoff
    pub base_delay: Duration,

    /// Maximum delay between retries
    pub max_delay: Duration,

    /// Multiplier for exponential backoff
    pub multiplier: f64,

    /// Jitter to add randomness to delays
    pub jitter: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            multiplier: 2.0,
            jitter: true,
        }
    }
}

impl RetryPolicy {
    /// Create a new retry policy
    pub fn new(max_retries: u32) -> Self {
        Self {
            max_retries,
            ..Default::default()
        }
    }

    /// Set base delay
    pub fn with_base_delay(mut self, delay: Duration) -> Self {
        self.base_delay = delay;
        self
    }

    /// Set maximum delay
    pub fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    /// Set multiplier
    pub fn with_multiplier(mut self, multiplier: f64) -> Self {
        self.multiplier = multiplier;
        self
    }

    /// Enable or disable jitter
    pub fn with_jitter(mut self, jitter: bool) -> Self {
        self.jitter = jitter;
        self
    }
}

/// Retry executor
pub struct RetryExecutor {
    policy: RetryPolicy,
}

impl RetryExecutor {
    /// Create a new retry executor
    pub fn new(policy: RetryPolicy) -> Self {
        Self { policy }
    }

    /// Execute a function with retry logic
    pub async fn execute<F, Fut, T>(&self, mut operation: F) -> ClientResult<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = ClientResult<T>>,
    {
        let mut backoff = ExponentialBackoff {
            initial_interval: self.policy.base_delay,
            max_interval: self.policy.max_delay,
            multiplier: self.policy.multiplier,
            max_elapsed_time: None,
            ..Default::default()
        };

        let mut attempts = 0;
        let mut last_error = None;

        loop {
            attempts += 1;

            match operation().await {
                Ok(result) => {
                    if attempts > 1 {
                        debug!("Operation succeeded after {} attempts", attempts);
                    }
                    return Ok(result);
                }
                Err(error) => {
                    last_error = Some(error.to_string());

                    // Check if we should retry
                    if attempts > self.policy.max_retries || !error.is_retryable() {
                        warn!(
                            "Operation failed after {} attempts, last error: {}",
                            attempts, error
                        );

                        if attempts > self.policy.max_retries {
                            return Err(ClientError::RetryExhausted {
                                attempts,
                                last_error: error.to_string(),
                            });
                        } else {
                            return Err(error);
                        }
                    }

                    // Calculate delay for next attempt
                    if let Some(delay) = backoff.next_backoff() {
                        debug!(
                            "Retrying operation (attempt {}/{}) after {:?}, error: {}",
                            attempts,
                            self.policy.max_retries + 1,
                            delay,
                            error
                        );

                        tokio::time::sleep(delay).await;
                    } else {
                        // Backoff exhausted
                        return Err(ClientError::RetryExhausted {
                            attempts,
                            last_error: error.to_string(),
                        });
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_retry_success_on_first_attempt() {
        let policy = RetryPolicy::new(3);
        let executor = RetryExecutor::new(policy);

        let result = executor
            .execute(|| async { Ok::<i32, ClientError>(42) })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_retry_success_after_failures() {
        let policy = RetryPolicy::new(3).with_base_delay(Duration::from_millis(1));
        let executor = RetryExecutor::new(policy);

        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = executor
            .execute(move || {
                let counter = counter_clone.clone();
                async move {
                    let count = counter.fetch_add(1, Ordering::SeqCst);
                    if count < 2 {
                        Err(ClientError::TimeoutError("Simulated timeout".to_string()))
                    } else {
                        Ok(42)
                    }
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_exhausted() {
        let policy = RetryPolicy::new(2).with_base_delay(Duration::from_millis(1));
        let executor = RetryExecutor::new(policy);

        let result = executor
            .execute(|| async {
                Err::<i32, ClientError>(ClientError::TimeoutError("Always fails".to_string()))
            })
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ClientError::RetryExhausted { attempts, .. } => {
                assert_eq!(attempts, 3); // 1 initial + 2 retries
            }
            _ => panic!("Expected RetryExhausted error"),
        }
    }

    #[tokio::test]
    async fn test_non_retryable_error() {
        let policy = RetryPolicy::new(3);
        let executor = RetryExecutor::new(policy);

        let result = executor
            .execute(|| async {
                Err::<i32, ClientError>(ClientError::AuthError("Invalid token".to_string()))
            })
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ClientError::AuthError(_) => {
                // Expected - should not retry auth errors
            }
            _ => panic!("Expected AuthError"),
        }
    }
}
