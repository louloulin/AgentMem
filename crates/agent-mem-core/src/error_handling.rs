//! Error Handling Utilities
//!
//! This module provides helper functions and trait implementations
//! for safe error handling without unwrap/expect.

use crate::{CoreError, CoreResult};
use std::sync::PoisonError;

// ═══════════════════════════════════════════════════════════════════════════════
// Lock Error Conversions
// ═══════════════════════════════════════════════════════════════════════════════

/// Convert PoisonError to CoreError for Mutex
impl<T> From<PoisonError<std::sync::MutexGuard<'_, T>>> for CoreError {
    fn from(e: PoisonError<std::sync::MutexGuard<'_, T>>) -> Self {
        CoreError::LockError(format!("Mutex poisoned: {}", e))
    }
}

/// Convert PoisonError to CoreError for RwLock read
impl<T> From<PoisonError<std::sync::RwLockReadGuard<'_, T>>> for CoreError {
    fn from(e: PoisonError<std::sync::RwLockReadGuard<'_, T>>) -> Self {
        CoreError::LockError(format!("RwLock read poisoned: {}", e))
    }
}

/// Convert PoisonError to CoreError for RwLock write
impl<T> From<PoisonError<std::sync::RwLockWriteGuard<'_, T>>> for CoreError {
    fn from(e: PoisonError<std::sync::RwLockWriteGuard<'_, T>>) -> Self {
        CoreError::LockError(format!("RwLock write poisoned: {}", e))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Lock Helper Functions
// ═══════════════════════════════════════════════════════════════════════════════

/// Safely lock a Mutex with proper error handling
///
/// # Example
/// ```rust
/// use agent_mem_core::error_handling::safe_lock;
///
/// let data = safe_lock(&self.mutex, "data_cache")?;
/// ```
pub fn safe_lock<'a, T>(
    mutex: &'a std::sync::Mutex<T>,
    context: &str,
) -> CoreResult<std::sync::MutexGuard<'a, T>> {
    mutex.lock().map_err(|e| {
        CoreError::LockError(format!(
            "Failed to acquire lock for {}: {}",
            context, e
        ))
    })
}

/// Safely lock a RwLock for reading with proper error handling
pub fn safe_read<'a, T>(
    rwlock: &'a std::sync::RwLock<T>,
    context: &str,
) -> CoreResult<std::sync::RwLockReadGuard<'a, T>> {
    rwlock.read().map_err(|e| {
        CoreError::LockError(format!(
            "Failed to acquire read lock for {}: {}",
            context, e
        ))
    })
}

/// Safely lock a RwLock for writing with proper error handling
pub fn safe_write<'a, T>(
    rwlock: &'a std::sync::RwLock<T>,
    context: &str,
) -> CoreResult<std::sync::RwLockWriteGuard<'a, T>> {
    rwlock.write().map_err(|e| {
        CoreError::LockError(format!(
            "Failed to acquire write lock for {}: {}",
            context, e
        ))
    })
}

// ═══════════════════════════════════════════════════════════════════════════════
// Option Helper Functions
// ═══════════════════════════════════════════════════════════════════════════════

/// Safely unwrap an Option with a required field error
///
/// # Example
/// ```rust
/// use agent_mem_core::error_handling::require_some;
///
/// let api_key = require_some(config.api_key.as_ref(), "api_key")?;
/// ```
pub fn require_some<T>(option: Option<&T>, field_name: &str) -> CoreResult<&T> {
    option.ok_or_else(|| {
        CoreError::InvalidInput(format!(
            "Required field '{}' is missing",
            field_name
        ))
    })
}

/// Safely unwrap an Option with a configuration error
pub fn require_config<T>(option: Option<T>, field_name: &str) -> CoreResult<T> {
    option.ok_or_else(|| {
        CoreError::ConfigurationError(format!(
            "Required configuration field '{}' is not set",
            field_name
        ))
    })
}

/// Get an Option value or a default with context
pub fn unwrap_or_default<T>(option: Option<T>, default: T) -> T {
    option.unwrap_or(default)
}

/// Get an Option value or compute a default
pub fn unwrap_or_else<T, F: FnOnce() -> T>(option: Option<T>, default: F) -> T {
    option.unwrap_or_else(default)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Regex Helper Functions
// ═══════════════════════════════════════════════════════════════════════════════

/// Safely compile a regex with proper error handling
///
/// # Example
/// ```rust
/// use agent_mem_core::error_handling::compile_regex;
///
/// let regex = compile_regex(r"^\d+$")?;
/// ```
pub fn compile_regex(pattern: &str) -> CoreResult<regex::Regex> {
    regex::Regex::new(pattern).map_err(|e| {
        CoreError::InvalidInput(format!("Invalid regex pattern '{}': {}", pattern, e))
    })
}

/// Compile a regex or return a static one (for testing/known-good patterns)
///
/// # Safety
/// Only use this for static, compile-time verified patterns
pub const unsafe fn compile_regex_unchecked(pattern: &str) -> regex::Regex {
    // SAFETY: Caller must ensure pattern is valid
    regex::Regex::new(pattern).unwrap_unchecked()
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
#[cfg(feature = "inline_tests")]
mod tests {
    use super::*;
    use std::sync::{Mutex, RwLock};

    #[test]
    fn test_safe_lock_success() {
        let mutex = Mutex::new(42);
        let guard = safe_lock(&mutex, "test_mutex").unwrap();
        assert_eq!(*guard, 42);
    }

    #[test]
    fn test_safe_read_success() {
        let rwlock = RwLock::new(42);
        let guard = safe_read(&rwlock, "test_rwlock").unwrap();
        assert_eq!(*guard, 42);
    }

    #[test]
    fn test_safe_write_success() {
        let rwlock = RwLock::new(42);
        {
            let mut guard = safe_write(&rwlock, "test_rwlock").unwrap();
            *guard = 100;
        }
        let guard = safe_read(&rwlock, "test_rwlock").unwrap();
        assert_eq!(*guard, 100);
    }

    #[test]
    fn test_require_some_success() {
        let value = Some(42);
        let result = require_some(value.as_ref(), "test_field").unwrap();
        assert_eq!(*result, 42);
    }

    #[test]
    fn test_require_some_error() {
        let value: Option<i32> = None;
        let result = require_some(value.as_ref(), "test_field");
        assert!(result.is_err());
    }

    #[test]
    fn test_require_config_success() {
        let value = Some("api_key");
        let result = require_config(value, "api_key").unwrap();
        assert_eq!(result, "api_key");
    }

    #[test]
    fn test_require_config_error() {
        let value: Option<&str> = None;
        let result = require_config(value, "api_key");
        assert!(result.is_err());
    }

    #[test]
    fn test_unwrap_or_default() {
        assert_eq!(unwrap_or_default(Some(42), 0), 42);
        assert_eq!(unwrap_or_default(None::<i32>, 0), 0);
    }

    #[test]
    fn test_unwrap_or_else() {
        assert_eq!(unwrap_or_else(Some(42), || 0), 42);
        assert_eq!(unwrap_or_else(None::<i32>, || 100), 100);
    }

    #[test]
    fn test_compile_regex_success() {
        let regex = compile_regex(r"^\d+$").unwrap();
        assert!(regex.is_match("123"));
        assert!(!regex.is_match("abc"));
    }

    #[test]
    fn test_compile_regex_error() {
        let result = compile_regex(r"(?P<invalid");
        assert!(result.is_err());
    }
}
