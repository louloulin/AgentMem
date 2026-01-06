//! Structured logging with tracing
//!
//! This module provides high-performance structured logging.

use crate::error::{ObservabilityError, ObservabilityResult};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

/// Initialize logging
pub fn init_logging(log_level: &str, json_format: bool) -> ObservabilityResult<()> {
    // Create env filter
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(log_level))
        .map_err(|e| ObservabilityError::LoggingInitFailed(e.to_string()))?;

    if json_format {
        // JSON format for production
        let fmt_layer = tracing_subscriber::fmt::layer()
            .json()
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
            .with_current_span(true)
            .with_span_list(true)
            .with_target(true)
            .with_file(true)
            .with_line_number(true);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .try_init()
            .map_err(|e| ObservabilityError::LoggingInitFailed(e.to_string()))?;
    } else {
        // Human-readable format for development
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
            .with_target(true)
            .with_file(true)
            .with_line_number(true)
            .pretty();

        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .try_init()
            .map_err(|e| ObservabilityError::LoggingInitFailed(e.to_string()))?;
    }

    tracing::info!(
        log_level = %log_level,
        json_format = json_format,
        "Logging initialized"
    );

    Ok(())
}

/// Log level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// Trace level
    Trace,
    /// Debug level
    Debug,
    /// Info level
    Info,
    /// Warn level
    Warn,
    /// Error level
    Error,
}

impl LogLevel {
    /// Convert to string
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for LogLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "trace" => Ok(LogLevel::Trace),
            "debug" => Ok(LogLevel::Debug),
            "info" => Ok(LogLevel::Info),
            "warn" => Ok(LogLevel::Warn),
            "error" => Ok(LogLevel::Error),
            _ => Err(format!("Invalid log level: {s}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_as_str() {
        assert_eq!(LogLevel::Trace.as_str(), "trace");
        assert_eq!(LogLevel::Debug.as_str(), "debug");
        assert_eq!(LogLevel::Info.as_str(), "info");
        assert_eq!(LogLevel::Warn.as_str(), "warn");
        assert_eq!(LogLevel::Error.as_str(), "error");
    }

    #[test]
    fn test_log_level_from_str() {
        assert_eq!("trace".parse::<LogLevel>().unwrap(), LogLevel::Trace);
        assert_eq!("debug".parse::<LogLevel>().unwrap(), LogLevel::Debug);
        assert_eq!("info".parse::<LogLevel>().unwrap(), LogLevel::Info);
        assert_eq!("warn".parse::<LogLevel>().unwrap(), LogLevel::Warn);
        assert_eq!("error".parse::<LogLevel>().unwrap(), LogLevel::Error);
        assert!("invalid".parse::<LogLevel>().is_err());
    }

    #[test]
    fn test_log_level_display() {
        assert_eq!(LogLevel::Info.to_string(), "info");
        assert_eq!(LogLevel::Error.to_string(), "error");
    }
}
