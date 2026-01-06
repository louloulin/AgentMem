//! Unified Error Handler
//!
//! This module provides a unified error handling system with:
//! - Error context and stack traces
//! - Friendly error messages
//! - Error monitoring support
//! - Error conversion utilities

use crate::error::{ErrorContext, ServerError, ServerResult};
use std::backtrace::Backtrace;
use std::fmt;
use tracing::{error, warn};

/// Error handler trait for consistent error handling
pub trait ErrorHandler {
    /// Convert error to ServerError with context
    fn to_server_error(self, context: impl Into<String>) -> ServerError;
    
    /// Convert error to ServerError with detailed context
    fn to_server_error_with_details(
        self,
        context: impl Into<String>,
        details: impl Into<String>,
    ) -> ServerError;
}

impl<E: std::error::Error + Send + Sync + 'static> ErrorHandler for E {
    fn to_server_error(self, context: impl Into<String>) -> ServerError {
        let context_str = context.into();
        ServerError::Internal {
            message: format!("{}: {}", context_str.clone(), self),
            source: Some(Box::new(self)),
            context: Some(ErrorContext::new(context_str)),
            backtrace: Some(Backtrace::capture()),
        }
    }

    fn to_server_error_with_details(
        self,
        context: impl Into<String>,
        details: impl Into<String>,
    ) -> ServerError {
        let context_str = context.into();
        let details_str = details.into();
        ServerError::Internal {
            message: format!("{}: {}", context_str.clone(), self),
            source: Some(Box::new(self)),
            context: Some(ErrorContext::new(context_str).with_details(details_str)),
            backtrace: Some(Backtrace::capture()),
        }
    }
}

/// Helper macro for error handling with context
#[macro_export]
macro_rules! handle_error {
    ($result:expr, $context:expr) => {
        $result.map_err(|e| {
            tracing::error!("Error in {}: {}", $context, e);
            e.to_server_error($context)
        })
    };
    ($result:expr, $context:expr, $details:expr) => {
        $result.map_err(|e| {
            tracing::error!("Error in {}: {} ({})", $context, e, $details);
            e.to_server_error_with_details($context, $details)
        })
    };
}

/// Safe unwrap with error context
pub fn safe_unwrap<T, E: std::error::Error + Send + Sync + 'static>(
    result: Result<T, E>,
    context: impl Into<String>,
) -> ServerResult<T> {
    result.map_err(|e| {
        let ctx = context.into();
        error!("Failed to unwrap result in {}: {}", ctx, e);
        e.to_server_error(ctx)
    })
}

/// Safe expect with error context
pub fn safe_expect<T>(
    option: Option<T>,
    context: impl Into<String>,
    details: Option<impl Into<String>>,
) -> ServerResult<T> {
    option.ok_or_else(|| {
        let ctx = context.into();
        let msg = if let Some(d) = details {
            format!("{}: {}", ctx, d.into())
        } else {
            ctx.clone()
        };
        error!("Unexpected None in {}: {}", ctx, msg);
        ServerError::Internal {
            message: msg,
            source: None,
            context: Some(ErrorContext::new(ctx)),
            backtrace: Some(Backtrace::capture()),
        }
    })
}

/// Error monitoring and alerting
pub struct ErrorMonitor {
    /// Error threshold for alerting
    threshold: u32,
    /// Error count
    count: std::sync::atomic::AtomicU32,
    /// Last error time
    last_error: std::sync::Mutex<Option<std::time::Instant>>,
}

impl ErrorMonitor {
    /// Create a new error monitor
    pub fn new(threshold: u32) -> Self {
        Self {
            threshold,
            count: std::sync::atomic::AtomicU32::new(0),
            last_error: std::sync::Mutex::new(None),
        }
    }

    /// Record an error
    pub fn record_error(&self, error: &ServerError) {
        let count = self.count.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
        
        // Update last error time
        if let Ok(mut last) = self.last_error.lock() {
            *last = Some(std::time::Instant::now());
        }

        // Check threshold and alert
        if count >= self.threshold {
            warn!(
                "Error threshold exceeded: {} errors (threshold: {})",
                count, self.threshold
            );
            // TODO: Send alert to monitoring system
        }

        // Log error details
        error!("Error recorded: {:?}", error);
    }

    /// Get error count
    pub fn error_count(&self) -> u32 {
        self.count.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Reset error count
    pub fn reset(&self) {
        self.count.store(0, std::sync::atomic::Ordering::Relaxed);
        if let Ok(mut last) = self.last_error.lock() {
            *last = None;
        }
    }
}

impl Default for ErrorMonitor {
    fn default() -> Self {
        Self::new(100) // Default threshold: 100 errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_expect_some() {
        let value = Some(42);
        let result = safe_expect(value, "test context", None::<String>);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_safe_expect_none() {
        let value: Option<i32> = None;
        let result = safe_expect(value, "test context", Some("value is None"));
        assert!(result.is_err());
    }

    #[test]
    fn test_error_monitor() {
        let monitor = ErrorMonitor::new(5);
        assert_eq!(monitor.error_count(), 0);

        let error = ServerError::Internal {
            message: "Test error".to_string(),
            source: None,
            context: None,
            backtrace: None,
        };

        for _ in 0..3 {
            monitor.record_error(&error);
        }

        assert_eq!(monitor.error_count(), 3);

        monitor.reset();
        assert_eq!(monitor.error_count(), 0);
    }
}
